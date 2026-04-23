mod algo;
mod config;

pub use algo::FloydWarshall;
pub use config::FloydWarshallConfig;

use crate::algorithms::{ApspAlgorithm, ApspResult};
use crate::utils::{ApspBuffers, FloatNumber, Graph};

/// One-shot Floyd-Warshall APSP.
pub fn floyd_warshall<T, G>(graph: &G, buffers: &mut ApspBuffers<T>) -> ApspResult<T>
where
    T: FloatNumber,
    G: Graph<T> + Sync,
    G::Meta: Sync,
{
    FloydWarshall::<T>::new().run(graph, buffers)
}
