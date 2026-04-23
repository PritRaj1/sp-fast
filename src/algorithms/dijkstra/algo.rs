use crate::algorithms::heaps::{BinaryHeap, PriorityQueue};
use crate::algorithms::{
    Event, SsspAlgorithm, SsspAlgorithmInfo, SsspResult, finalize_sssp, init_sssp,
};
use crate::utils::{FloatNumber, Graph, RelaxResult, SsspBuffers, relax_with};
use nalgebra::{DefaultAllocator, Dim, allocator::Allocator};
use std::marker::PhantomData;

use super::config::DijkstraConfig;

/// Dijkstra SSSP. +ve weights. Pluggable priority queue.
#[derive(Debug)]
pub struct Dijkstra<T: FloatNumber, H: PriorityQueue<T> = BinaryHeap<T>> {
    config: DijkstraConfig,
    heap: H,
    _phantom: PhantomData<T>,
}

impl<T: FloatNumber, H: PriorityQueue<T>> Dijkstra<T, H> {
    pub fn new() -> Self {
        Self {
            config: DijkstraConfig::default(),
            heap: H::new(),
            _phantom: PhantomData,
        }
    }

    pub fn with_config(config: DijkstraConfig) -> Self {
        Self {
            config,
            heap: H::new(),
            _phantom: PhantomData,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            config: DijkstraConfig::default(),
            heap: H::with_capacity(capacity),
            _phantom: PhantomData,
        }
    }

    pub fn config(&self) -> &DijkstraConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut DijkstraConfig {
        &mut self.config
    }
}

impl<T: FloatNumber> Default for Dijkstra<T, BinaryHeap<T>> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: FloatNumber, H: PriorityQueue<T>> SsspAlgorithmInfo for Dijkstra<T, H> {
    fn name(&self) -> &'static str {
        "Dijkstra"
    }

    fn supports_negative_weights(&self) -> bool {
        false
    }
}

impl<T: FloatNumber, H: PriorityQueue<T>> Dijkstra<T, H> {
    /// Multi-source primitive. `weight_fn` maps (stored weight, meta) -> weight
    /// used for relaxation: identity for plain Dijkstra, congestion inflation
    /// for capacity-aware routing.
    pub fn run_from_weighted_observed<G, N, W, F>(
        &mut self,
        graph: &G,
        sources: &[usize],
        buffers: &mut SsspBuffers<T, N>,
        weight_fn: W,
        mut observer: F,
    ) -> SsspResult<T>
    where
        G: Graph<T>,
        N: Dim,
        DefaultAllocator: Allocator<N>,
        W: Fn(T, &G::Meta) -> T,
        F: FnMut(Event<T>),
    {
        debug_assert!(!sources.is_empty(), "at least one source required");

        init_sssp(buffers, sources);
        self.heap.clear();
        for &s in sources {
            debug_assert!(s < graph.n(), "source vertex out of bounds");
            self.heap.push(T::zero(), s);
        }

        // Multi-target early-stop: O(1) membership via bool LUT + counter that
        // breaks once every target is finalised. Zero cost when targets unset.
        let targets = self.config.targets.as_slice();
        let mut remaining = targets.len();
        let is_target: Vec<bool> = if remaining == 0 {
            Vec::new()
        } else {
            let mut lut = vec![false; graph.n()];
            for &t in targets {
                lut[t] = true;
            }
            lut
        };

        let mut iterations = 0usize;
        while let Some(entry) = self.heap.pop() {
            let u = entry.vertex;
            let d_u = entry.dist;

            if self.config.lazy_deletion && d_u > buffers.dist[u] {
                continue;
            }

            // Last target -> skip relaxation (no successors needed) and break.
            let last_target = remaining > 0 && is_target[u] && {
                remaining -= 1;
                remaining == 0
            };

            if !last_target {
                iterations += 1;
                graph.for_each_out_edge(u, |v, w, meta| {
                    let w = weight_fn(w, meta);
                    debug_assert!(w >= T::zero(), "Dijkstra requires non-negative weights");
                    if let RelaxResult::Improved = relax_with(
                        buffers.dist.as_mut_slice(),
                        buffers.parent.as_mut_slice(),
                        u,
                        d_u,
                        v,
                        w,
                    ) {
                        let new_dist = buffers.dist[v];
                        self.heap.push(new_dist, v);
                        observer(Event::Improved {
                            vertex: v,
                            dist: new_dist,
                            parent: u,
                        });
                    }
                });
            }

            observer(Event::Finalized {
                vertex: u,
                dist: d_u,
                parent: buffers.parent_of(u),
            });

            if last_target {
                break;
            }
        }

        finalize_sssp(buffers, iterations, false)
    }

    /// Multi-source with observer.
    #[inline]
    pub fn run_from_observed<G, N, F>(
        &mut self,
        graph: &G,
        sources: &[usize],
        buffers: &mut SsspBuffers<T, N>,
        observer: F,
    ) -> SsspResult<T>
    where
        G: Graph<T>,
        N: Dim,
        DefaultAllocator: Allocator<N>,
        F: FnMut(Event<T>),
    {
        self.run_from_weighted_observed(graph, sources, buffers, |w, _| w, observer)
    }

    /// Multi-source with per-edge weight transform (meta-aware weighting).
    #[inline]
    pub fn run_from_weighted<G, N, W>(
        &mut self,
        graph: &G,
        sources: &[usize],
        buffers: &mut SsspBuffers<T, N>,
        weight_fn: W,
    ) -> SsspResult<T>
    where
        G: Graph<T>,
        N: Dim,
        DefaultAllocator: Allocator<N>,
        W: Fn(T, &G::Meta) -> T,
    {
        self.run_from_weighted_observed(graph, sources, buffers, weight_fn, |_| {})
    }

    #[inline]
    pub fn run_from<G, N>(
        &mut self,
        graph: &G,
        sources: &[usize],
        buffers: &mut SsspBuffers<T, N>,
    ) -> SsspResult<T>
    where
        G: Graph<T>,
        N: Dim,
        DefaultAllocator: Allocator<N>,
    {
        self.run_from_weighted_observed(graph, sources, buffers, |w, _| w, |_| {})
    }
}

impl<T, N, G, H> SsspAlgorithm<T, N, G> for Dijkstra<T, H>
where
    T: FloatNumber,
    N: Dim,
    G: Graph<T>,
    H: PriorityQueue<T>,
    DefaultAllocator: Allocator<N>,
{
    #[inline]
    fn run_observed<F>(
        &mut self,
        graph: &G,
        source: usize,
        buffers: &mut SsspBuffers<T, N>,
        observer: F,
    ) -> SsspResult<T>
    where
        F: FnMut(Event<T>),
    {
        self.run_from_observed(graph, std::slice::from_ref(&source), buffers, observer)
    }
}
