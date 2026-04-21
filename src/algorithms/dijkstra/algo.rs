use crate::algorithms::heaps::{BinaryHeap, PriorityQueue};
use crate::algorithms::HasSsspConfig;
use crate::algorithms::{finalize_sssp, init_sssp, SsspAlgorithm, SsspAlgorithmInfo, SsspResult};
use crate::utils::{relax_with, FloatNumber, Graph, RelaxResult, SsspBuffers};
use nalgebra::{allocator::Allocator, DefaultAllocator, Dim};
use std::marker::PhantomData;

use super::config::DijkstraConfig;

/// Dijkstra SSSP. Non-negative weights. Pluggable priority queue.
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

impl<T, N, G, H> SsspAlgorithm<T, N, G> for Dijkstra<T, H>
where
    T: FloatNumber,
    N: Dim,
    G: Graph<T>,
    H: PriorityQueue<T>,
    DefaultAllocator: Allocator<N>,
{
    fn run(&mut self, graph: &G, source: usize, buffers: &mut SsspBuffers<T, N>) -> SsspResult<T> {
        debug_assert!(source < graph.n(), "Source vertex out of bounds");

        init_sssp(buffers, source);
        self.heap.clear();
        self.heap.push(T::zero(), source);

        let mut iterations = 0usize;

        while let Some(entry) = self.heap.pop() {
            let u = entry.vertex;
            let d_u = entry.dist;

            if self.config.lazy_deletion && d_u > buffers.dist[u] {
                continue;
            }

            if self.config.should_stop(u) {
                break;
            }

            iterations += 1;

            graph.for_each_out_edge(u, |v, w| {
                debug_assert!(w >= T::zero(), "Dijkstra requires non-negative weights");

                if let RelaxResult::Improved = relax_with(
                    buffers.dist.as_mut_slice(),
                    buffers.parent.as_mut_slice(),
                    u,
                    d_u,
                    v,
                    w,
                ) {
                    self.heap.push(buffers.dist[v], v);
                }
            });
        }

        finalize_sssp(buffers, iterations, false)
    }
}
