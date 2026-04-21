use crate::algorithms::heaps::{BinaryHeap, PriorityQueue};
use crate::algorithms::{MstAlgorithm, MstAlgorithmInfo, MstResult, finalize_mst, init_mst};
use crate::utils::{FloatNumber, Graph, MstBuffers};
use nalgebra::{DefaultAllocator, Dim, allocator::Allocator};
use std::marker::PhantomData;

use super::config::PrimConfig;

/// Prim's MST. Pluggable priority queue. Disconnected graphs yield partial MST.
#[derive(Debug)]
pub struct Prim<T: FloatNumber, H: PriorityQueue<T> = BinaryHeap<T>> {
    config: PrimConfig,
    heap: H,
    _phantom: PhantomData<T>,
}

impl<T: FloatNumber, H: PriorityQueue<T>> Prim<T, H> {
    pub fn new() -> Self {
        Self {
            config: PrimConfig::default(),
            heap: H::new(),
            _phantom: PhantomData,
        }
    }

    pub fn with_config(config: PrimConfig) -> Self {
        Self {
            config,
            heap: H::new(),
            _phantom: PhantomData,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            config: PrimConfig::default(),
            heap: H::with_capacity(capacity),
            _phantom: PhantomData,
        }
    }

    pub fn config(&self) -> &PrimConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut PrimConfig {
        &mut self.config
    }
}

impl<T: FloatNumber> Default for Prim<T, BinaryHeap<T>> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: FloatNumber, H: PriorityQueue<T>> MstAlgorithmInfo for Prim<T, H> {
    fn name(&self) -> &'static str {
        "Prim"
    }
}

impl<T, N, G, H> MstAlgorithm<T, N, G> for Prim<T, H>
where
    T: FloatNumber,
    N: Dim,
    G: Graph<T>,
    H: PriorityQueue<T>,
    DefaultAllocator: Allocator<N>,
{
    fn run(&mut self, graph: &G, source: usize, buffers: &mut MstBuffers<T, N>) -> MstResult<T> {
        debug_assert!(source < graph.n(), "Source vertex out of bounds");

        init_mst(buffers, source);
        self.heap.clear();
        self.heap.push(T::zero(), source);

        let mut iterations = 0usize;

        while let Some(entry) = self.heap.pop() {
            let u = entry.vertex;
            let key_u = entry.dist;

            if self.config.lazy_deletion && (buffers.in_mst[u] || key_u > buffers.key[u]) {
                continue;
            }

            buffers.in_mst[u] = true;
            iterations += 1;

            graph.for_each_out_edge(u, |v, w, _meta| {
                if !buffers.in_mst[v] && w < buffers.key[v] {
                    buffers.key[v] = w;
                    buffers.parent[v] = u;
                    self.heap.push(w, v);
                }
            });
        }

        finalize_mst(buffers, iterations, graph.n())
    }
}
