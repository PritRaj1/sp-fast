mod common;

use common::assertions::EPS_F64 as EPS;
use common::*;
use nalgebra::Const;
use sssp_fast::{AdjListGraph, MstBuffers, prim};

fn classic_mst_graph() -> AdjListGraph<f64> {
    let mut g = AdjListGraph::new(4);
    add_undirected_edge(&mut g, 0, 1, 4.0);
    add_undirected_edge(&mut g, 0, 3, 5.0);
    add_undirected_edge(&mut g, 1, 2, 2.0);
    add_undirected_edge(&mut g, 1, 3, 1.0);
    add_undirected_edge(&mut g, 2, 3, 3.0);
    g
}

#[test]
fn test_classic_mst() {
    let g = classic_mst_graph();
    let mut buf = mst_dynamic(4);
    let result = prim(&g, 0, &mut buf);

    // MST: 0-1 (4.0), 1-3 (1.0), 1-2 (2.0) = 7.0
    approx_eq(result.total_weight, 7.0, EPS);
    assert!(result.is_connected);
}

#[test]
fn test_disconnected_graph() {
    let g = disconnected_undirected(6, 1.0);
    let mut buf = mst_dynamic(6);
    let result = prim(&g, 0, &mut buf);

    assert_eq!(result.vertices_in_mst, 3);
    approx_eq(result.total_weight, 2.0, EPS);
    assert!(!result.is_connected);
}

#[test]
fn test_parallel_edges() {
    let mut g: AdjListGraph<f64> = AdjListGraph::new(2);
    g.add_edge(0, 1, 5.0);
    g.add_edge(0, 1, 3.0);
    g.add_edge(0, 1, 7.0);
    g.add_edge(1, 0, 5.0);
    g.add_edge(1, 0, 3.0);
    g.add_edge(1, 0, 7.0);

    let mut buf = mst_dynamic(2);
    let result = prim(&g, 0, &mut buf);

    approx_eq(result.total_weight, 3.0, EPS);
}

#[test]
fn test_different_starting_vertices() {
    let g = classic_mst_graph();

    // MST weight same regardless of starting vertex
    for source in 0..4 {
        let mut buf = mst_dynamic(4);
        let result = prim(&g, source, &mut buf);
        approx_eq(result.total_weight, 7.0, EPS);
    }
}

#[test]
fn test_grid() {
    let g = grid_undirected(5, 5, 1.0);
    let mut buf = mst_dynamic(25);
    let result = prim(&g, 0, &mut buf);

    approx_eq(result.total_weight, 24.0, EPS); // n-1 edges
    assert!(result.is_connected);
}

#[test]
fn test_static_dims() {
    let g = classic_mst_graph();
    let mut buf: MstBuffers<f64, Const<4>> = MstBuffers::new_inf(Const::<4>);
    let result = prim(&g, 0, &mut buf);

    approx_eq(result.total_weight, 7.0, EPS);
}
