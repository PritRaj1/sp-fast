mod common;

use common::{assertions::EPS_F64 as EPS, *};
use sssp_fast::{
    AdjListGraph, Dyn, Graph, NO_VERTEX, RelaxResult, SsspBuffers, dijkstra, relax_cond,
};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Cap {
    capacity: u32,
}

#[test]
fn test_dijkstra_ignore_meta() {
    let edges = [
        (0, 1, 1.0),
        (1, 2, 1.0),
        (0, 2, 3.0),
        (1, 3, 2.0),
        (2, 3, 1.0),
    ];

    let mut meta_g: AdjListGraph<f64, Cap> = AdjListGraph::new(4);
    for (u, v, w) in edges {
        meta_g.add_edge_with(u, v, w, Cap { capacity: 10 });
    }

    let mut plain_g: AdjListGraph<f64> = AdjListGraph::new(4);
    for (u, v, w) in edges {
        plain_g.add_edge(u, v, w);
    }

    let mut meta_buf: SsspBuffers<f64, Dyn> = SsspBuffers::new_inf(Dyn(4));
    let mut plain_buf: SsspBuffers<f64, Dyn> = SsspBuffers::new_inf(Dyn(4));

    dijkstra(&meta_g, 0, &mut meta_buf);
    dijkstra(&plain_g, 0, &mut plain_buf);

    for v in 0..4 {
        approx_eq(meta_buf.dist[v], plain_buf.dist[v], EPS);
        assert_eq!(meta_buf.parent[v], plain_buf.parent[v]);
    }
}

#[test]
fn test_relax_cond() {
    let mut dist = vec![0.0, f64::INFINITY];
    let mut parent = vec![NO_VERTEX, NO_VERTEX];

    // Gate=false blocks relaxation.
    let r = relax_cond(&mut dist, &mut parent, 0, 0.0, 1, 5.0, || false);
    assert!(matches!(r, RelaxResult::NoChange));
    assert!(dist[1].is_infinite());

    // Gate=true allows it.
    relax_cond(&mut dist, &mut parent, 0, 0.0, 1, 5.0, || true);
    approx_eq(dist[1], 5.0, EPS);
    assert_eq!(parent[1], 0);

    // Gate is skipped when dist wouldn't improve anyway.
    let mut called = false;
    relax_cond(&mut dist, &mut parent, 0, 0.0, 1, 99.0, || {
        called = true;
        true
    });
    assert!(!called);
}

#[test]
fn test_meta_gated_routing() {
    // Direct edge 0->1 is blocked (cap=0); shortest route is 0->2->1 at cost 4.
    let mut g: AdjListGraph<f64, Cap> = AdjListGraph::new(3);
    g.add_edge_with(0, 1, 1.0, Cap { capacity: 0 });
    g.add_edge_with(0, 2, 3.0, Cap { capacity: 10 });
    g.add_edge_with(2, 1, 1.0, Cap { capacity: 10 });

    let mut dist = vec![f64::INFINITY; 3];
    let mut parent = vec![NO_VERTEX; 3];
    dist[0] = 0.0;

    g.for_each_out_edge(0, |v, w, meta| {
        relax_cond(&mut dist, &mut parent, 0, 0.0, v, w, || meta.capacity > 0);
    });
    assert!(dist[1].is_infinite());
    approx_eq(dist[2], 3.0, EPS);

    let d2 = dist[2];
    g.for_each_out_edge(2, |v, w, meta| {
        relax_cond(&mut dist, &mut parent, 2, d2, v, w, || meta.capacity > 0);
    });
    approx_eq(dist[1], 4.0, EPS);
    assert_eq!(parent[1], 2);
}
