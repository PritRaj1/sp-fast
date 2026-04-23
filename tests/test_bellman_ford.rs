mod common;

use common::{assertions::EPS_F64 as EPS, *};
use sssp_fast::AdjListGraph;

#[test]
fn test_linear_chain() {
    let g = linear(5, 1.0);
    let mut buf = dynamic(5);
    bellman_ford(&g, 0, &mut buf);

    for i in 0..5 {
        dist_eq(&buf, i, i as f64, EPS);
    }
}

#[test]
fn test_diamond_selects_shortest() {
    let (g, optimal) = diamond(1.0, 1.0, 3.0, 10.0, 1.0);
    let mut buf = dynamic(4);
    bellman_ford(&g, 0, &mut buf);

    dist_eq(&buf, 3, optimal, EPS);
}

#[test]
fn test_disconnected_graph() {
    let g = disconnected(6, 1.0);
    let mut buf = dynamic(6);
    bellman_ford(&g, 0, &mut buf);

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
    bellman_ford(&g, 0, &mut buf);

    dist_eq(&buf, 2, 1.0, EPS);
    path_eq(&buf, 2, &[0, 2]);
}

#[test]
fn test_negative_weight_edge() {
    let mut g: AdjListGraph<f64> = AdjListGraph::new(3);
    g.add_edge(0, 1, 10.0);
    g.add_edge(1, 2, -5.0);

    let mut buf = dynamic(3);
    let result = bellman_ford(&g, 0, &mut buf);

    dists_eq(&buf, &[(0, 0.0), (1, 10.0), (2, 5.0)], EPS);
    assert!(!result.negative_cycle);
}

#[test]
fn test_negative_weight_shorter_path() {
    // Two paths: 0 -> 1 -> 3 (cost 10), 0 -> 2 -> 3 (cost 5 via negative edge)
    let mut g: AdjListGraph<f64> = AdjListGraph::new(4);
    g.add_edge(0, 1, 5.0);
    g.add_edge(1, 3, 5.0);
    g.add_edge(0, 2, 10.0);
    g.add_edge(2, 3, -5.0);

    let mut buf = dynamic(4);
    let result = bellman_ford(&g, 0, &mut buf);

    dist_eq(&buf, 3, 5.0, EPS);
    path_eq(&buf, 3, &[0, 2, 3]);
    assert!(!result.negative_cycle);
}

#[test]
fn test_simple_negative_cycle() {
    // 0 -> 1 -> 2 -> 0 with total weight -1
    let mut g: AdjListGraph<f64> = AdjListGraph::new(3);
    g.add_edge(0, 1, 1.0);
    g.add_edge(1, 2, 1.0);
    g.add_edge(2, 0, -3.0);

    let mut buf = dynamic(3);
    let result = bellman_ford(&g, 0, &mut buf);

    assert!(result.negative_cycle);
}

#[test]
fn test_self_loop_negative() {
    let mut g: AdjListGraph<f64> = AdjListGraph::new(2);
    g.add_edge(0, 1, 1.0);
    g.add_edge(1, 1, -1.0); // negative self-loop

    let mut buf = dynamic(2);
    let result = bellman_ford(&g, 0, &mut buf);

    assert!(result.negative_cycle);
}
