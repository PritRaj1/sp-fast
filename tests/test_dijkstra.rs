mod common;

use common::{assertions::EPS_F64 as EPS, *};
use sssp_fast::{AdjListGraph, dijkstra_to};

#[test]
fn test_path_reconstruction() {
    let mut g: AdjListGraph<f64> = AdjListGraph::new(3);
    g.add_edge(0, 1, 1.0);
    g.add_edge(1, 2, 1.0);
    g.add_edge(0, 2, 1.0); // shortcut

    let mut buf = dynamic(3);
    dijkstra(&g, 0, &mut buf);

    dist_eq(&buf, 2, 1.0, EPS);
    path_eq(&buf, 2, &[0, 2]);
}

#[test]
fn test_early_stop() {
    let g = linear(10, 1.0);
    let mut buf = dynamic(10);
    let result = dijkstra_to(&g, 0, 3, &mut buf);

    dist_eq(&buf, 3, 3.0, EPS);
    assert!(result.iterations <= 4);
}

#[test]
fn test_parallel_edges() {
    let mut g: AdjListGraph<f64> = AdjListGraph::new(2);
    g.add_edge(0, 1, 5.0);
    g.add_edge(0, 1, 3.0);
    g.add_edge(0, 1, 7.0);

    let mut buf = dynamic(2);
    dijkstra(&g, 0, &mut buf);

    dist_eq(&buf, 1, 3.0, EPS);
}
