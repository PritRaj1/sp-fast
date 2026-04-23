mod algo;
mod config;

pub use algo::BellmanFord;
pub use config::BellmanFordConfig;

use crate::algorithms::{SsspAlgorithm, SsspResult};
use crate::utils::{FloatNumber, Graph, SsspBuffers};
use nalgebra::{DefaultAllocator, Dim, allocator::Allocator};

/// One-shot Bellman-Ford, no early stop.
pub fn bellman_ford<T, N, G>(
    graph: &G,
    source: usize,
    buffers: &mut SsspBuffers<T, N>,
) -> SsspResult<T>
where
    T: FloatNumber,
    N: Dim,
    G: Graph<T> + Sync,
    G::Meta: Sync,
    DefaultAllocator: Allocator<N>,
{
    BellmanFord::<T>::new().run(graph, source, buffers)
}

/// One-shot Bellman-Ford from many sources; each vertex gets distance to nearest source.
pub fn bellman_ford_multi<T, N, G>(
    graph: &G,
    sources: &[usize],
    buffers: &mut SsspBuffers<T, N>,
) -> SsspResult<T>
where
    T: FloatNumber,
    N: Dim,
    G: Graph<T> + Sync,
    G::Meta: Sync,
    DefaultAllocator: Allocator<N>,
{
    BellmanFord::<T>::new().run_from(graph, sources, buffers)
}
