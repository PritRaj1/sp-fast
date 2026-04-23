use crate::algorithms::heaps::{BinaryHeap, PriorityQueue};
use crate::algorithms::{
    Event, SsspAlgorithm, SsspAlgorithmInfo, SsspResult, finalize_sssp, init_sssp,
};
use crate::utils::{FloatNumber, Graph, RelaxResult, SsspBuffers, relax_with};
use nalgebra::{DefaultAllocator, Dim, allocator::Allocator};
use std::marker::PhantomData;
use std::slice;

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
    fn run_observed<F>(
        &mut self,
        graph: &G,
        source: usize,
        buffers: &mut SsspBuffers<T, N>,
        mut observer: F,
    ) -> SsspResult<T>
    where
        F: FnMut(Event<T>),
    {
        debug_assert!(source < graph.n(), "Source vertex out of bounds");

        let target = self.config.target;

        init_sssp(buffers, slice::from_ref(&source));
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

            // Goal reached, skip relaxation, break.
            if u != target {
                iterations += 1;
                graph.for_each_out_edge(u, |v, w, _meta| {
                    debug_assert!(w >= T::zero(), "A* requires non-negative weights");

                    if let RelaxResult::Improved = relax_with(
                        buffers.dist.as_mut_slice(),
                        buffers.parent.as_mut_slice(),
                        u,
                        g_u,
                        v,
                        w,
                    ) {
                        let new_g = buffers.dist[v];
                        let h_v = self.config.heuristic.estimate(v, target);
                        self.heap.push(new_g + h_v, v);
                        observer(Event::Improved {
                            vertex: v,
                            dist: new_g,
                            parent: u,
                        });
                    }
                });
            }

            observer(Event::Finalized {
                vertex: u,
                dist: g_u,
                parent: buffers.parent_of(u),
            });

            if u == target {
                break;
            }
        }

        finalize_sssp(buffers, iterations, false)
    }
}
