mod common;

use common::{assertions::EPS_F64 as EPS, *};
use sssp_fast::{AdjListGraph, Dijkstra, SsspAlgorithm, dijkstra_to};

#[test]
fn test_linear_chain() {
    let g = linear(5, 1.0);
    let mut buf = dynamic(5);
    dijkstra(&g, 0, &mut buf);

    for i in 0..5 {
        dist_eq(&buf, i, i as f64, EPS);
    }
}

#[test]
fn test_diamond_selects_shortest() {
    let (g, optimal) = diamond(1.0, 1.0, 3.0, 10.0, 1.0);
    let mut buf = dynamic(4);
    dijkstra(&g, 0, &mut buf);

    dist_eq(&buf, 3, optimal, EPS);
}

#[test]
fn test_disconnected_graph() {
    let g = disconnected(6, 1.0);
    let mut buf = dynamic(6);
    dijkstra(&g, 0, &mut buf);

    dists_eq(&buf, &[(0, 0.0), (1, 1.0), (2, 2.0)], EPS);
    all_unreachable(&buf, &[3, 4, 5]);
}

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
fn test_reusable_instance() {
    let g = linear(5, 1.0);
    let mut algo: Dijkstra<f64> = Dijkstra::new();

    for source in 0..3 {
        let mut buf = dynamic(5);
        algo.run(&g, source, &mut buf);
        dist_eq(&buf, source, 0.0, EPS);
    }
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

#[test]
fn test_grid() {
    let g = grid(5, 5, 1.0);
    let mut buf = dynamic(25);
    dijkstra(&g, 0, &mut buf);

    dist_eq(&buf, 24, 8.0, EPS); // 4 right + 4 down
}
