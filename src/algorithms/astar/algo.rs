use crate::algorithms::heaps::{BinaryHeap, PriorityQueue};
use crate::algorithms::HasSsspConfig;
use crate::algorithms::{finalize_sssp, init_sssp, SsspAlgorithm, SsspAlgorithmInfo, SsspResult};
use crate::utils::{relax_with, FloatNumber, Graph, RelaxResult, SsspBuffers};
use nalgebra::{allocator::Allocator, DefaultAllocator, Dim};
use std::marker::PhantomData;

use super::config::{AStarConfig, Heuristic};

/// A* search. Admissible heuristic required for optimal paths.
#[derive(Debug)]
pub struct AStar<T: FloatNumber, Heur: Heuristic<T>, H: PriorityQueue<T> = BinaryHeap<T>> {
    config: AStarConfig<Heur>,
    heap: H,
    _phantom: PhantomData<T>,
}

impl<T: FloatNumber, Heur: Heuristic<T>, H: PriorityQueue<T>> AStar<T, Heur, H> {
    pub fn new(target: usize, heuristic: Heur) -> Self {
        Self {
            config: AStarConfig::new(target, heuristic),
            heap: H::new(),
            _phantom: PhantomData,
        }
    }

    pub fn with_config(config: AStarConfig<Heur>) -> Self {
        Self {
            config,
            heap: H::new(),
            _phantom: PhantomData,
        }
    }

    pub fn with_capacity(target: usize, heuristic: Heur, capacity: usize) -> Self {
        Self {
            config: AStarConfig::new(target, heuristic),
            heap: H::with_capacity(capacity),
            _phantom: PhantomData,
        }
    }

    pub fn config(&self) -> &AStarConfig<Heur> {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut AStarConfig<Heur> {
        &mut self.config
    }
}

impl<T: FloatNumber, Heur: Heuristic<T> + Default> Default for AStar<T, Heur, BinaryHeap<T>> {
    fn default() -> Self {
        Self {
            config: AStarConfig::default(),
            heap: BinaryHeap::new(),
            _phantom: PhantomData,
        }
    }
}

impl<T: FloatNumber, Heur: Heuristic<T>, H: PriorityQueue<T>> SsspAlgorithmInfo
    for AStar<T, Heur, H>
{
    fn name(&self) -> &'static str {
        "A*"
    }

    fn supports_negative_weights(&self) -> bool {
        false
    }
}

impl<T, N, G, Heur, H> SsspAlgorithm<T, N, G> for AStar<T, Heur, H>
where
    T: FloatNumber,
    N: Dim,
    G: Graph<T>,
    Heur: Heuristic<T>,
    H: PriorityQueue<T>,
    DefaultAllocator: Allocator<N>,
{
    fn run(&mut self, graph: &G, source: usize, buffers: &mut SsspBuffers<T, N>) -> SsspResult<T> {
        debug_assert!(source < graph.n(), "Source vertex out of bounds");

        let target = self.config.target().expect("A* requires a target vertex");

        init_sssp(buffers, source);
        self.heap.clear();

        let h_source = self.config.heuristic.estimate(source, target);
        self.heap.push(h_source, source);

        let mut iterations = 0usize;

        while let Some(entry) = self.heap.pop() {
            let u = entry.vertex;
            let f_u = entry.dist;
            let g_u = buffers.dist[u];

            let h_u = self.config.heuristic.estimate(u, target);
            if self.config.lazy_deletion && f_u > g_u + h_u {
                continue;
            }

            if self.config.should_stop(u) {
                break;
            }

            iterations += 1;

            graph.for_each_out_edge(u, |v, w| {
                debug_assert!(w >= T::zero(), "A* requires non-negative weights");

                if let RelaxResult::Improved = relax_with(
                    buffers.dist.as_mut_slice(),
                    buffers.parent.as_mut_slice(),
                    u,
                    g_u,
                    v,
                    w,
                ) {
                    let h_v = self.config.heuristic.estimate(v, target);
                    let f_v = buffers.dist[v] + h_v;
                    self.heap.push(f_v, v);
                }
            });
        }

        finalize_sssp(buffers, iterations, false)
    }
}
