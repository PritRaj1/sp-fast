use crate::utils::{ApspBuffers, FloatNumber, Graph, MstBuffers, SsspBuffers};
use nalgebra::{DefaultAllocator, Dim, allocator::Allocator};

// =============================================================================
// Config
// =============================================================================

/// Shared SSSP knobs.
#[derive(Clone, Debug, Default)]
pub struct SsspConfig {
    /// Target vertices for Dijkstra multi-target early-stop; empty = no stop.
    pub targets: Vec<usize>,
}

impl SsspConfig {
    pub fn with_target(target: usize) -> Self {
        Self::with_targets(vec![target])
    }

    pub fn with_targets(targets: Vec<usize>) -> Self {
        Self { targets }
    }

    #[inline]
    pub fn is_target(&self, vertex: usize) -> bool {
        self.targets.contains(&vertex)
    }
}

/// Exposes core SSSP knobs through any algorithm's config.
pub trait HasSsspConfig {
    fn sssp_config(&self) -> &SsspConfig;

    #[inline]
    fn targets(&self) -> &[usize] {
        &self.sssp_config().targets
    }

    #[inline]
    fn is_target(&self, vertex: usize) -> bool {
        self.sssp_config().is_target(vertex)
    }
}

impl HasSsspConfig for SsspConfig {
    fn sssp_config(&self) -> &SsspConfig {
        self
    }
}

// =============================================================================
// Results
// =============================================================================

#[derive(Clone, Debug)]
pub struct SsspResult<T: FloatNumber> {
    pub iterations: usize,
    pub negative_cycle: bool,
    pub vertices_reached: usize,
    pub total_distance: T,
}

#[derive(Clone, Debug)]
pub struct MstResult<T: FloatNumber> {
    pub iterations: usize,
    pub vertices_in_mst: usize,
    pub total_weight: T,
    pub is_connected: bool,
}

/// APSP summary. Distances live in caller's [`ApspBuffers`].
#[derive(Clone, Debug)]
pub struct ApspResult<T: FloatNumber> {
    pub iterations: usize,
    pub negative_cycle: bool,
    pub pairs_reached: usize,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: FloatNumber> ApspResult<T> {
    pub fn new(iterations: usize, negative_cycle: bool, pairs_reached: usize) -> Self {
        Self {
            iterations,
            negative_cycle,
            pairs_reached,
            _phantom: std::marker::PhantomData,
        }
    }
}

// =============================================================================
// Events for observer
// =============================================================================

/// Different algorithms emit different subsets:
///
/// - Dijkstra, A*, Prim: `Improved` + `Finalized`
/// - Bellman-Ford:       `Improved` + `Iteration`
/// - Floyd-Warshall:     `Iteration`
#[derive(Clone, Copy, Debug)]
pub enum Event<T: FloatNumber> {
    /// Vertex distance/key just improved during edge relaxation.
    Improved {
        vertex: usize,
        dist: T,
        parent: usize,
    },
    /// Vertex just finalized (popped from queue / added to MST); distance final.
    Finalized {
        vertex: usize,
        dist: T,
        parent: Option<usize>,
    },
    /// Outer iter complete (BF: one pass; FW: one intermediate `k`).
    Iteration(usize),
}

// =============================================================================
// Traits
// =============================================================================

pub trait SsspAlgorithmInfo {
    fn name(&self) -> &'static str;
    fn supports_negative_weights(&self) -> bool;
}

pub trait SsspAlgorithm<T, N, G>: SsspAlgorithmInfo
where
    T: FloatNumber,
    N: Dim,
    G: Graph<T>,
    DefaultAllocator: Allocator<N>,
{
    /// Primitive: runs with observer notified of every progress event.
    fn run_observed<F>(
        &mut self,
        graph: &G,
        source: usize,
        buffers: &mut SsspBuffers<T, N>,
        observer: F,
    ) -> SsspResult<T>
    where
        F: FnMut(Event<T>);

    #[inline]
    fn run(&mut self, graph: &G, source: usize, buffers: &mut SsspBuffers<T, N>) -> SsspResult<T> {
        self.run_observed(graph, source, buffers, |_| {})
    }
}

pub trait MstAlgorithmInfo {
    fn name(&self) -> &'static str;
}

pub trait MstAlgorithm<T, N, G>: MstAlgorithmInfo
where
    T: FloatNumber,
    N: Dim,
    G: Graph<T>,
    DefaultAllocator: Allocator<N>,
{
    fn run_observed<F>(
        &mut self,
        graph: &G,
        source: usize,
        buffers: &mut MstBuffers<T, N>,
        observer: F,
    ) -> MstResult<T>
    where
        F: FnMut(Event<T>);

    #[inline]
    fn run(&mut self, graph: &G, source: usize, buffers: &mut MstBuffers<T, N>) -> MstResult<T> {
        self.run_observed(graph, source, buffers, |_| {})
    }
}

pub trait ApspAlgorithmInfo {
    fn name(&self) -> &'static str;
    fn supports_negative_weights(&self) -> bool;
}

pub trait ApspAlgorithm<T, G>: ApspAlgorithmInfo
where
    T: FloatNumber,
    G: Graph<T>,
{
    fn run_observed<F>(
        &mut self,
        graph: &G,
        buffers: &mut ApspBuffers<T>,
        observer: F,
    ) -> ApspResult<T>
    where
        F: FnMut(Event<T>);

    #[inline]
    fn run(&mut self, graph: &G, buffers: &mut ApspBuffers<T>) -> ApspResult<T> {
        self.run_observed(graph, buffers, |_| {})
    }
}

// =============================================================================
// Runner helpers
// =============================================================================

/// Reset buffers, mark every source with distance zero.
#[inline]
pub fn init_sssp<T, N>(buffers: &mut SsspBuffers<T, N>, sources: &[usize])
where
    T: FloatNumber,
    N: Dim,
    DefaultAllocator: Allocator<N>,
{
    buffers.reset_inf();
    for &s in sources {
        buffers.dist[s] = T::zero();
    }
}

pub fn finalize_sssp<T, N>(
    buffers: &SsspBuffers<T, N>,
    iterations: usize,
    negative_cycle: bool,
) -> SsspResult<T>
where
    T: FloatNumber,
    N: Dim,
    DefaultAllocator: Allocator<N>,
{
    let mut vertices_reached = 0usize;
    let mut total_distance = T::zero();
    for &d in buffers.dist.iter() {
        if !d.is_infinite() {
            vertices_reached += 1;
            total_distance += d;
        }
    }
    SsspResult {
        iterations,
        negative_cycle,
        vertices_reached,
        total_distance,
    }
}

#[inline]
pub fn init_mst<T, N>(buffers: &mut MstBuffers<T, N>, source: usize)
where
    T: FloatNumber,
    N: Dim,
    DefaultAllocator: Allocator<N>,
{
    buffers.reset_inf();
    buffers.set_source(source);
}

pub fn finalize_mst<T, N>(buffers: &MstBuffers<T, N>, iterations: usize, n: usize) -> MstResult<T>
where
    T: FloatNumber,
    N: Dim,
    DefaultAllocator: Allocator<N>,
{
    let vertices_in_mst = buffers.vertices_in_mst();
    MstResult {
        iterations,
        vertices_in_mst,
        total_weight: buffers.total_weight(),
        is_connected: vertices_in_mst == n,
    }
}
