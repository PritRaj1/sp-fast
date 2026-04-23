//! Dijkstra must agree with Bellman-Ford on every +ve weight graph.

use nalgebra::Dyn;
use proptest::prelude::*;
use sssp_fast::{AdjListGraph, BellmanFord, Dijkstra, Graph, SsspAlgorithm, SsspBuffers};

const EPS: f64 = 1e-9;

fn graph_strategy() -> impl Strategy<Value = (AdjListGraph<f64>, usize)> {
    (2usize..24, 0u64..1 << 20).prop_map(|(n, seed)| {
        let mut rng = fastrand::Rng::with_seed(seed);
        let mut g = AdjListGraph::new(n);

        // approx 3× edges as vertices, random directed; weights in [1, 10).
        for _ in 0..3 * n {
            let u = rng.usize(0..n);
            let v = rng.usize(0..n);
            if u != v {
                g.add_edge(u, v, 1.0 + rng.f64() * 9.0);
            }
        }
        let source = rng.usize(0..n);
        (g, source)
    })
}

proptest! {
    #[test]
    fn test_dijkstra_vs_bellman_ford((g, source) in graph_strategy()) {
        let n = g.n();
        let mut buf_d: SsspBuffers<f64, Dyn> = SsspBuffers::new_inf(Dyn(n));
        let mut buf_bf: SsspBuffers<f64, Dyn> = SsspBuffers::new_inf(Dyn(n));

        let mut dijkstra: Dijkstra<f64> = Dijkstra::new();
        let mut bf: BellmanFord<f64> = BellmanFord::new();

        dijkstra.run(&g, source, &mut buf_d);
        bf.run(&g, source, &mut buf_bf);

        for v in 0..n {
            let d = buf_d.dist[v];
            let b = buf_bf.dist[v];
            prop_assert_eq!(d.is_infinite(), b.is_infinite(), "reachability mismatch at {}", v);
            if d.is_finite() {
                prop_assert!((d - b).abs() < EPS, "dist mismatch at {}: dijkstra={}, bf={}", v, d, b);
            }
        }
    }
}
