use crate::algorithms::{SsspAlgorithm, SsspResult};
use crate::utils::{FloatNumber, Graph, SsspBuffers};
use nalgebra::{allocator::Allocator, DefaultAllocator, Dim, OVector};
use rayon::prelude::*;

/// Result of multi-source SSSP — one buffer + stats per source.
#[derive(Clone, Debug)]
pub struct MultiSourceResult<T: FloatNumber, N: Dim>
where
    DefaultAllocator: Allocator<N>,
{
    pub sources: Vec<usize>,
    pub buffers: Vec<SsspBuffers<T, N>>,
    pub stats: Vec<SsspResult<T>>,
}

impl<T: FloatNumber, N: Dim> MultiSourceResult<T, N>
where
    DefaultAllocator: Allocator<N>,
{
    #[inline]
    pub fn get(&self, source_idx: usize) -> Option<(&SsspBuffers<T, N>, &SsspResult<T>)> {
        self.buffers.get(source_idx).zip(self.stats.get(source_idx))
    }

    #[inline]
    pub fn dist(&self, source_idx: usize, v: usize) -> Option<T> {
        self.buffers.get(source_idx).map(|b| b.dist[v])
    }

    pub fn path(&self, source_idx: usize, v: usize) -> Option<Vec<usize>> {
        self.buffers.get(source_idx).and_then(|b| b.path_to(v))
    }

    pub fn total_iterations(&self) -> usize {
        self.stats.iter().map(|s| s.iterations).sum()
    }
}

/// Run SSSP from each source in parallel (rayon). Fresh algo instance per source.
pub fn parallel_sssp<T, N, G, A, F>(
    graph: &G,
    sources: &[usize],
    algo_factory: F,
) -> MultiSourceResult<T, N>
where
    T: FloatNumber,
    N: Dim,
    G: Graph<T> + Sync,
    A: SsspAlgorithm<T, N, G> + Send,
    F: Fn() -> A + Sync,
    DefaultAllocator: Allocator<N>,
    OVector<T, N>: Send + Sync,
    OVector<usize, N>: Send + Sync,
{
    let n = graph.n();

    let results: Vec<_> = sources
        .par_iter()
        .map(|&source| {
            let mut algo = algo_factory();
            let mut buf = SsspBuffers::new_inf(N::from_usize(n));
            let result = algo.run(graph, source, &mut buf);
            (buf, result)
        })
        .collect();

    let (buffers, stats): (Vec<_>, Vec<_>) = results.into_iter().unzip();

    MultiSourceResult {
        sources: sources.to_vec(),
        buffers,
        stats,
    }
}

/// All-pairs SSSP via single-source from every vertex. O(n · SSSP) time, O(n²) space.
pub fn all_pairs_sssp<T, N, G, A, F>(graph: &G, algo_factory: F) -> MultiSourceResult<T, N>
where
    T: FloatNumber,
    N: Dim,
    G: Graph<T> + Sync,
    A: SsspAlgorithm<T, N, G> + Send,
    F: Fn() -> A + Sync,
    DefaultAllocator: Allocator<N>,
    OVector<T, N>: Send + Sync,
    OVector<usize, N>: Send + Sync,
{
    let sources: Vec<usize> = (0..graph.n()).collect();
    parallel_sssp(graph, &sources, algo_factory)
}
