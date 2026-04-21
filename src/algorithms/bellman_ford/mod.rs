mod algo;
mod config;

pub use algo::BellmanFord;
pub use config::BellmanFordConfig;

use crate::algorithms::{SsspAlgorithm, SsspResult};
use crate::utils::{FloatNumber, Graph, SsspBuffers};
use nalgebra::{DefaultAllocator, Dim, allocator::Allocator};

/// One-shot Bellman-Ford, no early stop.
pub fn cheeky_bellman_ford<T, N, G>(
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

/// One-shot Bellman-Ford with early-stop at `target`.
pub fn bellman_ford_to<T, N, G>(
    graph: &G,
    source: usize,
    target: usize,
    buffers: &mut SsspBuffers<T, N>,
) -> SsspResult<T>
where
    T: FloatNumber,
    N: Dim,
    G: Graph<T> + Sync,
    G::Meta: Sync,
    DefaultAllocator: Allocator<N>,
{
    BellmanFord::<T>::with_config(BellmanFordConfig::with_target(target))
        .run(graph, source, buffers)
}
