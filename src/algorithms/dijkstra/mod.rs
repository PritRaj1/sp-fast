mod algo;
mod config;

pub use algo::Dijkstra;
pub use config::DijkstraConfig;

use crate::algorithms::heaps::BinaryHeap;
use crate::algorithms::{SsspAlgorithm, SsspResult};
use crate::utils::{FloatNumber, Graph, SsspBuffers};
use nalgebra::{allocator::Allocator, DefaultAllocator, Dim};

/// One-shot Dijkstra, no early stop.
pub fn cheeky_dijkstra<T, N, G>(
    graph: &G,
    source: usize,
    buffers: &mut SsspBuffers<T, N>,
) -> SsspResult<T>
where
    T: FloatNumber,
    N: Dim,
    G: Graph<T>,
    DefaultAllocator: Allocator<N>,
{
    Dijkstra::<T, BinaryHeap<T>>::new().run(graph, source, buffers)
}

/// One-shot Dijkstra with early-stop at `target`.
pub fn dijkstra_to<T, N, G>(
    graph: &G,
    source: usize,
    target: usize,
    buffers: &mut SsspBuffers<T, N>,
) -> SsspResult<T>
where
    T: FloatNumber,
    N: Dim,
    G: Graph<T>,
    DefaultAllocator: Allocator<N>,
{
    Dijkstra::<T, BinaryHeap<T>>::with_config(DijkstraConfig::with_target(target))
        .run(graph, source, buffers)
}
