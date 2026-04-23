use crate::algorithms::{
    Event, SsspAlgorithm, SsspAlgorithmInfo, SsspResult, finalize_sssp, init_sssp,
};
use crate::utils::{FloatNumber, Graph, SsspBuffers};
use nalgebra::{DefaultAllocator, Dim, allocator::Allocator};
use rayon::prelude::*;

use super::config::BellmanFordConfig;

/// Proposed relaxation: (target, new dist, parent).
#[derive(Clone, Copy, Debug)]
struct Proposal<T: FloatNumber> {
    target: usize,
    dist: T,
    parent: usize,
}

/// Rayon-parallel Bellman-Ford. Handles -ve weights, detects -ve cycles.
#[derive(Debug)]
pub struct BellmanFord<T: FloatNumber> {
    config: BellmanFordConfig,
    best: Vec<Option<Proposal<T>>>, // Commit scratch, reused across rounds and runs.
}

impl<T: FloatNumber> BellmanFord<T> {
    pub fn new() -> Self {
        Self {
            config: BellmanFordConfig::default(),
            best: Vec::new(),
        }
    }

    pub fn with_config(config: BellmanFordConfig) -> Self {
        Self {
            config,
            best: Vec::new(),
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

impl<T: FloatNumber> BellmanFord<T> {
    /// Multi-source with observer.
    pub fn run_from_observed<N, G, F>(
        &mut self,
        graph: &G,
        sources: &[usize],
        buffers: &mut SsspBuffers<T, N>,
        mut observer: F,
    ) -> SsspResult<T>
    where
        N: Dim,
        G: Graph<T> + Sync,
        G::Meta: Sync,
        DefaultAllocator: Allocator<N>,
        F: FnMut(Event<T>),
    {
        debug_assert!(!sources.is_empty(), "at least one source required");
        for &s in sources {
            debug_assert!(s < graph.n(), "source vertex out of bounds");
        }
        init_sssp(buffers, sources);

        let n = graph.n();
        self.best.clear();
        self.best.resize(n, None);

        let mut iterations = 0usize;
        for round in 0..n.saturating_sub(1) {
            iterations += 1;
            let proposals = collect_proposals(graph, buffers);
            self.best.fill(None);
            let any_improved = apply_proposals(&mut self.best, buffers, &proposals, |p| {
                observer(Event::Improved {
                    vertex: p.target,
                    dist: p.dist,
                    parent: p.parent,
                });
            });
            observer(Event::Iteration(round));
            if self.config.early_termination && !any_improved {
                break;
            }
        }

        let negative_cycle = detect_negative_cycle(graph, buffers);
        finalize_sssp(buffers, iterations, negative_cycle)
    }

    #[inline]
    pub fn run_from<N, G>(
        &mut self,
        graph: &G,
        sources: &[usize],
        buffers: &mut SsspBuffers<T, N>,
    ) -> SsspResult<T>
    where
        N: Dim,
        G: Graph<T> + Sync,
        G::Meta: Sync,
        DefaultAllocator: Allocator<N>,
    {
        self.run_from_observed(graph, sources, buffers, |_| {})
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
    #[inline]
    fn run_observed<F>(
        &mut self,
        graph: &G,
        source: usize,
        buffers: &mut SsspBuffers<T, N>,
        observer: F,
    ) -> SsspResult<T>
    where
        F: FnMut(Event<T>),
    {
        self.run_from_observed(graph, std::slice::from_ref(&source), buffers, observer)
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

/// Commit best proposal per target. `best` must be pre-sized to n, filled `None`.
fn apply_proposals<T, N, F>(
    best: &mut [Option<Proposal<T>>],
    buffers: &mut SsspBuffers<T, N>,
    proposals: &[Proposal<T>],
    mut on_commit: F,
) -> bool
where
    T: FloatNumber,
    N: Dim,
    DefaultAllocator: Allocator<N>,
    F: FnMut(&Proposal<T>),
{
    if proposals.is_empty() {
        return false;
    }

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
    for (v, opt) in best.iter().copied().enumerate() {
        if let Some(prop) = opt
            && prop.dist < buffers.dist[v]
        {
            buffers.dist[v] = prop.dist;
            buffers.parent[v] = prop.parent;
            on_commit(&prop);
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
