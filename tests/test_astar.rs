mod common;

use common::{assertions::EPS_F64 as EPS, *};
use sssp_fast::{Heuristic, astar, astar_with};

fn zero(_v: usize, _target: usize) -> f64 {
    0.0
}

#[derive(Clone)]
struct ManhattanHeuristic {
    cols: usize,
}

impl Heuristic<f64> for ManhattanHeuristic {
    fn estimate(&self, v: usize, target: usize) -> f64 {
        let (vr, vc) = (v / self.cols, v % self.cols);
        let (tr, tc) = (target / self.cols, target % self.cols);
        (vr.abs_diff(tr) + vc.abs_diff(tc)) as f64
    }
}

#[test]
fn test_grid_with_manhattan() {
    let g = grid(5, 5, 1.0);
    let mut buf = dynamic(25);

    astar_with(&g, 0, 24, ManhattanHeuristic { cols: 5 }, &mut buf);
    dist_eq(&buf, 24, 8.0, EPS);
}

#[test]
fn test_matches_dijkstra() {
    let (g, optimal) = diamond(1.0, 1.0, 3.0, 10.0, 1.0);

    let mut buf_astar = dynamic(4);
    astar(&g, 0, 3, zero, &mut buf_astar);

    let mut buf_dijkstra = dynamic(4);
    dijkstra(&g, 0, &mut buf_dijkstra);

    dist_eq(&buf_astar, 3, optimal, EPS);
    dist_eq(&buf_dijkstra, 3, optimal, EPS);
}

#[test]
fn test_fewer_iters_with_heuristic() {
    let g = grid(10, 10, 1.0);

    let mut buf_zero = dynamic(100);
    let result_zero = astar(&g, 0, 99, zero, &mut buf_zero);

    let mut buf_h = dynamic(100);
    let result_h = astar_with(&g, 0, 99, ManhattanHeuristic { cols: 10 }, &mut buf_h);

    dist_eq(&buf_zero, 99, 18.0, EPS);
    dist_eq(&buf_h, 99, 18.0, EPS);
    assert!(result_h.iterations <= result_zero.iterations);
}

#[test]
fn test_source_equals_target() {
    let g = linear(5, 1.0);
    let mut buf = dynamic(5);
    astar(&g, 2, 2, zero, &mut buf);

    dist_eq(&buf, 2, 0.0, EPS);
}
