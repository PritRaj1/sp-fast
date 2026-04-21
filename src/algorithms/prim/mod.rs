mod algo;
mod config;

pub use algo::Prim;
pub use config::PrimConfig;

use crate::algorithms::heaps::BinaryHeap;
use crate::algorithms::{MstAlgorithm, MstResult};
use crate::utils::{FloatNumber, Graph, MstBuffers};
use nalgebra::{DefaultAllocator, Dim, allocator::Allocator};

/// One-shot Prim's MST.
pub fn cheeky_prim<T, N, G>(
    graph: &G,
    source: usize,
    buffers: &mut MstBuffers<T, N>,
) -> MstResult<T>
where
    T: FloatNumber,
    N: Dim,
    G: Graph<T>,
    DefaultAllocator: Allocator<N>,
{
    Prim::<T, BinaryHeap<T>>::new().run(graph, source, buffers)
}
