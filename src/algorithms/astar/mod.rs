mod algo;
mod config;

pub use algo::AStar;
pub use config::{AStarConfig, FnHeuristic, Heuristic, ZeroHeuristic};

use crate::algorithms::heaps::BinaryHeap;
use crate::algorithms::{SsspAlgorithm, SsspResult};
use crate::utils::{FloatNumber, Graph, SsspBuffers};
use nalgebra::{allocator::Allocator, DefaultAllocator, Dim};

/// One-shot A* with fn-pointer heuristic.
pub fn cheeky_astar<T, N, G>(
    graph: &G,
    source: usize,
    target: usize,
    heuristic: fn(usize, usize) -> T,
    buffers: &mut SsspBuffers<T, N>,
) -> SsspResult<T>
where
    T: FloatNumber,
    N: Dim,
    G: Graph<T>,
    DefaultAllocator: Allocator<N>,
{
    AStar::<T, _, BinaryHeap<T>>::new(target, FnHeuristic::new(heuristic))
        .run(graph, source, buffers)
}

/// One-shot A* with custom heuristic type.
pub fn astar_with<T, N, G, H>(
    graph: &G,
    source: usize,
    target: usize,
    heuristic: H,
    buffers: &mut SsspBuffers<T, N>,
) -> SsspResult<T>
where
    T: FloatNumber,
    N: Dim,
    G: Graph<T>,
    H: Heuristic<T>,
    DefaultAllocator: Allocator<N>,
{
    AStar::<T, H, BinaryHeap<T>>::new(target, heuristic).run(graph, source, buffers)
}
