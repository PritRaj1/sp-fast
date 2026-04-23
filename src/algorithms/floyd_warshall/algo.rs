use crate::algorithms::{ApspAlgorithm, ApspAlgorithmInfo, ApspResult, Event};
use crate::utils::{ApspBuffers, FloatNumber, Graph};
use rayon::prelude::*;

use super::config::FloydWarshallConfig;

/// Floyd-Warshall APSP. O(n^3) time, O(n^2) space, -ve weights OK; cycle detection optional.
#[derive(Debug)]
pub struct FloydWarshall<T: FloatNumber> {
    config: FloydWarshallConfig,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: FloatNumber> FloydWarshall<T> {
    pub fn new() -> Self {
        Self {
            config: FloydWarshallConfig::default(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn with_config(config: FloydWarshallConfig) -> Self {
        Self {
            config,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn config(&self) -> &FloydWarshallConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut FloydWarshallConfig {
        &mut self.config
    }
}

impl<T: FloatNumber> Default for FloydWarshall<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: FloatNumber> ApspAlgorithmInfo for FloydWarshall<T> {
    fn name(&self) -> &'static str {
        "Floyd-Warshall"
    }

    fn supports_negative_weights(&self) -> bool {
        true
    }
}

impl<T, G> ApspAlgorithm<T, G> for FloydWarshall<T>
where
    T: FloatNumber,
    G: Graph<T> + Sync,
    G::Meta: Sync,
{
    fn run_observed<F>(
        &mut self,
        graph: &G,
        buffers: &mut ApspBuffers<T>,
        mut observer: F,
    ) -> ApspResult<T>
    where
        F: FnMut(Event<T>),
    {
        let n = graph.n();
        debug_assert!(buffers.n == n, "Buffer size mismatch");

        init_from_graph(graph, buffers);
        for k in 0..n {
            update_for_k(buffers, k);
            observer(Event::Iteration(k));
        }

        let negative_cycle = if self.config.detect_negative_cycle {
            buffers.has_negative_cycle()
        } else {
            false
        };

        finalize_apsp(buffers, n, negative_cycle)
    }
}

fn init_from_graph<T, G>(graph: &G, buffers: &mut ApspBuffers<T>)
where
    T: FloatNumber,
    G: Graph<T>,
{
    let n = graph.n();
    buffers.reset();

    // Take min weight across parallel edges.
    for u in 0..n {
        graph.for_each_out_edge(u, |v, w, _meta| {
            if w < buffers.get(u, v) {
                buffers.set(u, v, w);
                buffers.set_next(u, v, v);
            }
        });
    }
    for i in 0..n {
        buffers.set_next(i, i, i);
    }
}

/// Relax all (i,j) through intermediate vertex k. Parallel over rows.
fn update_for_k<T: FloatNumber>(buffers: &mut ApspBuffers<T>, k: usize) {
    let n = buffers.n;
    let row_k: Vec<T> = (0..n).map(|j| buffers.get(k, j)).collect();
    let col_k: Vec<T> = (0..n).map(|i| buffers.get(i, k)).collect();

    let updates: Vec<(usize, usize, T, usize)> = (0..n)
        .into_par_iter()
        .flat_map(|i| {
            let d_ik = col_k[i];
            if d_ik.is_infinite() {
                return Vec::new();
            }
            let next_ik = buffers.get_next(i, k);
            let mut local = Vec::new();
            for (j, &d_kj) in row_k.iter().enumerate() {
                if d_kj.is_infinite() {
                    continue;
                }
                let new_dist = d_ik + d_kj;
                if new_dist < buffers.get(i, j) {
                    local.push((i, j, new_dist, next_ik));
                }
            }
            local
        })
        .collect();

    for (i, j, dist, next_v) in updates {
        buffers.set(i, j, dist);
        buffers.set_next(i, j, next_v);
    }
}

fn finalize_apsp<T: FloatNumber>(
    buffers: &ApspBuffers<T>,
    n: usize,
    negative_cycle: bool,
) -> ApspResult<T> {
    let pairs_reached = buffers.dist.iter().filter(|d| !d.is_infinite()).count();
    ApspResult::new(n, negative_cycle, pairs_reached)
}
