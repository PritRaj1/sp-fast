mod common;

use common::{assertions::EPS_F64 as EPS, *};
use sssp_fast::{AdjListGraph, ApspAlgorithm};

#[test]
fn test_shortest_paths() {
    let (g, _) = diamond(1.0, 1.0, 3.0, 10.0, 1.0);
    let mut buf = apsp(4);
    floyd_warshall(&g, &mut buf);

    apsp_dist_eq(&buf, 0, 0, 0.0, EPS);
    apsp_dist_eq(&buf, 0, 1, 1.0, EPS);
    apsp_dist_eq(&buf, 0, 2, 2.0, EPS); // 0->1->2
    apsp_dist_eq(&buf, 0, 3, 3.0, EPS); // 0->1->2->3
}

#[test]
fn test_unreachable_pairs() {
    let g = disconnected(6, 1.0);
    let mut buf = apsp(6);
    floyd_warshall(&g, &mut buf);

    apsp_dist_eq(&buf, 0, 1, 1.0, EPS);
    apsp_unreachable(&buf, 0, 3);
    apsp_unreachable(&buf, 2, 4);
}

#[test]
fn test_path_reconstruction() {
    let mut g: AdjListGraph<f64> = AdjListGraph::new(4);
    g.add_edge(0, 1, 1.0);
    g.add_edge(1, 2, 1.0);
    g.add_edge(2, 3, 1.0);
    g.add_edge(0, 3, 10.0); // longer direct edge

    let mut buf = apsp(4);
    floyd_warshall(&g, &mut buf);

    apsp_dist_eq(&buf, 0, 3, 3.0, EPS);
    apsp_path_eq(&buf, 0, 3, &[0, 1, 2, 3]);
}

#[test]
fn test_negative_weights() {
    let mut g: AdjListGraph<f64> = AdjListGraph::new(4);
    g.add_edge(0, 1, 5.0);
    g.add_edge(1, 3, 5.0);
    g.add_edge(0, 2, 10.0);
    g.add_edge(2, 3, -5.0);

    let mut buf = apsp(4);
    let result = floyd_warshall(&g, &mut buf);

    apsp_dist_eq(&buf, 0, 3, 5.0, EPS);
    apsp_path_eq(&buf, 0, 3, &[0, 2, 3]);
    assert!(!result.negative_cycle);
}

#[test]
fn test_negative_cycle() {
    let mut g: AdjListGraph<f64> = AdjListGraph::new(3);
    g.add_edge(0, 1, 1.0);
    g.add_edge(1, 2, 1.0);
    g.add_edge(2, 0, -3.0);

    let mut buf = apsp(3);
    let result = sssp_fast::FloydWarshall::<f64>::with_config(sssp_fast::FloydWarshallConfig {
        detect_negative_cycle: true,
    })
    .run(&g, &mut buf);

    assert!(result.negative_cycle);
}
