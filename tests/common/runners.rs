use nalgebra::{DefaultAllocator, Dim, allocator::Allocator};
use sssp_fast::{FloatNumber, Graph, SsspAlgorithm, SsspAlgorithmInfo, SsspBuffers, SsspResult};

pub use sssp_fast::cheeky_bellman_ford;
pub use sssp_fast::cheeky_dijkstra;
pub use sssp_fast::cheeky_floyd_warshall;

pub fn run_sssp<T, N, G, A>(
    algo: &mut A,
    graph: &G,
    source: usize,
    buffers: &mut SsspBuffers<T, N>,
) -> SsspResult<T>
where
    T: FloatNumber,
    N: Dim,
    G: Graph<T>,
    A: SsspAlgorithm<T, N, G>,
    DefaultAllocator: Allocator<N>,
{
    algo.run(graph, source, buffers)
}

pub fn check_algo_info<A: SsspAlgorithmInfo>(
    algo: &A,
    expected_name: &str,
    expected_neg_weights: bool,
) {
    assert_eq!(algo.name(), expected_name);
    assert_eq!(algo.supports_negative_weights(), expected_neg_weights);
}
