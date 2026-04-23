use std::cell::Cell;

use sssp_fast::{AdjListGraph, Dijkstra, Dyn, Graph, SsspBuffers};

struct Cap {
    capacity: u32,
    load: Cell<u32>,
}

impl Cap {
    fn new(capacity: u32) -> Self {
        Self {
            capacity,
            load: Cell::new(0),
        }
    }
}

fn unit_grid(rows: usize, cols: usize, cap: u32) -> AdjListGraph<f64, Cap> {
    let n = rows * cols;
    let mut g = AdjListGraph::new(n);
    for r in 0..rows {
        for c in 0..cols {
            let v = r * cols + c;
            if c + 1 < cols {
                g.add_edge_with(v, v + 1, 1.0, Cap::new(cap));
                g.add_edge_with(v + 1, v, 1.0, Cap::new(cap));
            }
            if r + 1 < rows {
                g.add_edge_with(v, v + cols, 1.0, Cap::new(cap));
                g.add_edge_with(v + cols, v, 1.0, Cap::new(cap));
            }
        }
    }
    g
}

/// Weight inflated by `(1 + load/capacity)`.
fn congested(w: f64, meta: &Cap) -> f64 {
    w * (1.0 + meta.load.get() as f64 / meta.capacity as f64)
}

fn add_load(g: &AdjListGraph<f64, Cap>, path: &[usize]) {
    for win in path.windows(2) {
        let (u, v) = (win[0], win[1]);
        g.for_each_out_edge(u, |to, _, meta| {
            if to == v {
                meta.load.set(meta.load.get() + 1);
            }
        });
        g.for_each_out_edge(v, |to, _, meta| {
            if to == u {
                meta.load.set(meta.load.get() + 1);
            }
        });
    }
}

fn route(
    algo: &mut Dijkstra<f64>,
    g: &AdjListGraph<f64, Cap>,
    src: usize,
    tgt: usize,
    buf: &mut SsspBuffers<f64, Dyn>,
) -> Vec<usize> {
    algo.config_mut().set_target(tgt);
    algo.run_from_weighted(g, std::slice::from_ref(&src), buf, congested);
    buf.path_to(tgt).unwrap_or_default()
}

#[test]
fn test_second_flow_detours() {
    let g = unit_grid(3, 5, 1);
    let n = g.n();
    let mut buf: SsspBuffers<f64, Dyn> = SsspBuffers::new_inf(Dyn(n));
    let mut algo: Dijkstra<f64> = Dijkstra::new();

    let p1 = route(&mut algo, &g, 0, 4, &mut buf);
    assert_eq!(p1.len(), 5);
    add_load(&g, &p1);

    let p2 = route(&mut algo, &g, 0, 4, &mut buf);
    assert!(p2 != p1, "second flow should detour around saturated edges");
}
