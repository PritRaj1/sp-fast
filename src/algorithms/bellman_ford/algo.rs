use crate::algorithms::{SsspAlgorithm, SsspAlgorithmInfo, SsspResult, finalize_sssp, init_sssp};
use crate::utils::{FloatNumber, Graph, SsspBuffers};
use nalgebra::{DefaultAllocator, Dim, allocator::Allocator};
use rayon::prelude::*;

use super::config::BellmanFordConfig;

/// Proposed relaxation: (target, new dist, parent).
#[derive(Clone, Copy)]
struct Proposal<T: FloatNumber> {
    target: usize,
    dist: T,
    parent: usize,
}

/// Rayon-parallel Bellman-Ford. Handles negative weights, detects negative cycles.
/// Propose-then-apply avoids concurrent writes during parallel relaxation.
#[derive(Debug)]
pub struct BellmanFord<T: FloatNumber> {
    config: BellmanFordConfig,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: FloatNumber> BellmanFord<T> {
    pub fn new() -> Self {
        Self {
            config: BellmanFordConfig::default(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn with_config(config: BellmanFordConfig) -> Self {
        Self {
            config,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn config(&self) -> &BellmanFordConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut BellmanFordConfig {
        &mut self.config
    }
}

impl<T: FloatNumber> Default for BellmanFord<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: FloatNumber> SsspAlgorithmInfo for BellmanFord<T> {
    fn name(&self) -> &'static str {
        "Bellman-Ford"
    }

    fn supports_negative_weights(&self) -> bool {
        true
    }
}

impl<T, N, G> SsspAlgorithm<T, N, G> for BellmanFord<T>
where
    T: FloatNumber,
    N: Dim,
    G: Graph<T> + Sync,
    G::Meta: Sync,
    DefaultAllocator: Allocator<N>,
{
    fn run(&mut self, graph: &G, source: usize, buffers: &mut SsspBuffers<T, N>) -> SsspResult<T> {
        debug_assert!(source < graph.n(), "Source vertex out of bounds");

        init_sssp(buffers, source);

        let n = graph.n();
        let mut iterations = 0usize;

        for _ in 0..n.saturating_sub(1) {
            iterations += 1;
            let proposals = collect_proposals(graph, buffers);
            let any_improved = apply_proposals(buffers, &proposals);

            if self.config.early_termination && !any_improved {
                break;
            }
        }

        let negative_cycle = detect_negative_cycle(graph, buffers);

        finalize_sssp(buffers, iterations, negative_cycle)
    }
}

fn collect_proposals<T, N, G>(graph: &G, buffers: &SsspBuffers<T, N>) -> Vec<Proposal<T>>
where
    T: FloatNumber,
    N: Dim,
    G: Graph<T> + Sync,
    G::Meta: Sync,
    DefaultAllocator: Allocator<N>,
{
    let n = graph.n();
    let dist_slice = buffers.dist.as_slice();

    (0..n)
        .into_par_iter()
        .flat_map(|u| {
            let d_u = dist_slice[u];
            if d_u.is_infinite() {
                return Vec::new();
            }

            let mut local_proposals = Vec::new();
            graph.for_each_out_edge(u, |v, w, _meta| {
                let new_dist = d_u + w;
                if new_dist < dist_slice[v] {
                    local_proposals.push(Proposal {
                        target: v,
                        dist: new_dist,
                        parent: u,
                    });
                }
            });
            local_proposals
        })
        .collect()
}

/// Take best proposal per target, commit. Returns true on any improvement.
fn apply_proposals<T, N>(buffers: &mut SsspBuffers<T, N>, proposals: &[Proposal<T>]) -> bool
where
    T: FloatNumber,
    N: Dim,
    DefaultAllocator: Allocator<N>,
{
    if proposals.is_empty() {
        return false;
    }

    let n = buffers.dist.len();
    let mut best: Vec<Option<Proposal<T>>> = vec![None; n];

    for &prop in proposals {
        match &mut best[prop.target] {
            Some(existing) if prop.dist < existing.dist => {
                *existing = prop;
            }
            None => {
                best[prop.target] = Some(prop);
            }
            _ => {}
        }
    }

    let mut any_improved = false;
    for (v, opt) in best.into_iter().enumerate() {
        if let Some(prop) = opt
            && prop.dist < buffers.dist[v]
        {
            buffers.dist[v] = prop.dist;
            buffers.parent[v] = prop.parent;
            any_improved = true;
        }
    }

    any_improved
}

fn detect_negative_cycle<T, N, G>(graph: &G, buffers: &SsspBuffers<T, N>) -> bool
where
    T: FloatNumber,
    N: Dim,
    G: Graph<T> + Sync,
    G::Meta: Sync,
    DefaultAllocator: Allocator<N>,
{
    let n = graph.n();
    let dist_slice = buffers.dist.as_slice();

    (0..n).into_par_iter().any(|u| {
        let d_u = dist_slice[u];
        if d_u.is_infinite() {
            return false;
        }

        let mut has_cycle = false;
        graph.for_each_out_edge(u, |v, w, _meta| {
            if d_u + w < dist_slice[v] {
                has_cycle = true;
            }
        });
        has_cycle
    })
}
