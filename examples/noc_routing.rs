//! Offline congestion-aware route placement on a 5x5 NoC mesh under random
//! traffic.
//!
//! Compares XY (dimension-ordered, real-chip baseline) against sequential
//! adaptive Dijkstra with M/M/1 weight inflation (AEthereal-style
//! compile-time allocator, not a runtime router).
//! Bidirectional links = two directed wires with own load. No VC or
//! deadlock modeling — appropriate for offline placement.
//!
//! Animation: (1) per-flow allocation; (2) steady-state dataflow with packets
//! cycling through allocated routes.

mod common;

use std::cell::Cell;
use std::fs::{File, create_dir_all};
use std::io::BufWriter;
use std::path::Path;

use common::noc::{self, FRAME_H, FRAME_W, Loads, Side, idx, rc};
use gif::{Encoder, Frame, Repeat};
use sssp_fast::{AdjListGraph, Dijkstra, Dyn, Graph, SsspBuffers};

const CAP: u32 = 3;
const RHO_CAP: f64 = 0.95;
const N_FLOWS: usize = 40;
const SEED: u64 = 7;

const ALLOC_DELAY: u16 = 22;
const DATAFLOW_DELAY: u16 = 12;
const HOLD_DELAY: u16 = 20;
const DATAFLOW_FRAMES: usize = 100;
const DATAFLOW_STEP: f64 = 0.12;
const HOLD: usize = 20;

const OUT: &str = "examples/gifs/noc_routing.gif";

#[derive(Debug)]
struct Wire {
    capacity: u32,
    load: Cell<u32>,
}

impl Wire {
    fn new(capacity: u32) -> Self {
        Self {
            capacity,
            load: Cell::new(0),
        }
    }
    fn add(&self) {
        self.load.set(self.load.get() + 1);
    }
}

fn build_mesh() -> AdjListGraph<f64, Wire> {
    let mut g = AdjListGraph::new(noc::N * noc::N);
    for r in 0..noc::N {
        for c in 0..noc::N {
            let v = idx(r, c);
            if c + 1 < noc::N {
                g.add_edge_with(v, v + 1, 1.0, Wire::new(CAP));
                g.add_edge_with(v + 1, v, 1.0, Wire::new(CAP));
            }
            if r + 1 < noc::N {
                g.add_edge_with(v, v + noc::N, 1.0, Wire::new(CAP));
                g.add_edge_with(v + noc::N, v, 1.0, Wire::new(CAP));
            }
        }
    }
    g
}

// M/M/1 latency w / (1 - rho), clamped at rho=0.95 -> 20x base
fn weight_fn(w: f64, m: &Wire) -> f64 {
    let rho = (m.load.get() as f64 / m.capacity as f64).min(RHO_CAP);
    w / (1.0 - rho)
}

fn add_load(g: &AdjListGraph<f64, Wire>, path: &[usize]) {
    for win in path.windows(2) {
        let (u, v) = (win[0], win[1]);
        g.for_each_out_edge(u, |to, _, m| {
            if to == v {
                m.add();
            }
        });
    }
}

fn snapshot(g: &AdjListGraph<f64, Wire>) -> Loads {
    let mut loads = Loads::new();
    for u in 0..g.n() {
        g.for_each_out_edge(u, |v, _, m| {
            loads.insert((u, v), m.load.get());
        });
    }
    loads
}

// XY (dimension-ordered): all column moves, then all row moves
fn xy_route(src: usize, dst: usize) -> Vec<usize> {
    let (mut sr, mut sc) = rc(src);
    let (dr, dc) = rc(dst);
    let mut path = vec![idx(sr, sc)];
    while sc != dc {
        sc = if sc < dc { sc + 1 } else { sc - 1 };
        path.push(idx(sr, sc));
    }
    while sr != dr {
        sr = if sr < dr { sr + 1 } else { sr - 1 };
        path.push(idx(sr, sc));
    }
    path
}

fn adaptive_route(
    algo: &mut Dijkstra<f64>,
    g: &AdjListGraph<f64, Wire>,
    buf: &mut SsspBuffers<f64, Dyn>,
    src: usize,
    dst: usize,
) -> Vec<usize> {
    algo.config_mut().set_target(dst);
    algo.run_from_weighted(g, std::slice::from_ref(&src), buf, weight_fn);
    buf.path_to(dst).unwrap_or_default()
}

fn manhattan(s: usize, d: usize) -> usize {
    let (sr, sc) = rc(s);
    let (dr, dc) = rc(d);
    sr.abs_diff(dr) + sc.abs_diff(dc)
}

fn random_flows(n_flows: usize, seed: u64) -> Vec<(usize, usize)> {
    let n = noc::N * noc::N;
    let mut rng = fastrand::Rng::with_seed(seed);
    let mut f = Vec::with_capacity(n_flows);
    while f.len() < n_flows {
        let s = rng.usize(0..n);
        let mut d = rng.usize(0..n);
        while d == s {
            d = rng.usize(0..n);
        }
        f.push((s, d));
    }
    f
}

fn max_load(loads: &Loads) -> u32 {
    loads.values().copied().max().unwrap_or(0)
}

fn mean_stretch(flows: &[(usize, usize)], paths: &[Vec<usize>]) -> f64 {
    let mut total = 0.0;
    for (&(s, d), p) in flows.iter().zip(paths) {
        let direct = manhattan(s, d);
        if direct > 0 {
            total += p.len().saturating_sub(1) as f64 / direct as f64;
        }
    }
    total / flows.len() as f64
}

fn route_pass<F: FnMut(usize, usize) -> Vec<usize>>(
    flows: &[(usize, usize)],
    g: &AdjListGraph<f64, Wire>,
    mut route: F,
) -> (Vec<Vec<usize>>, Vec<Loads>) {
    let mut paths = Vec::with_capacity(flows.len());
    let mut history = Vec::with_capacity(flows.len() + 1);
    history.push(snapshot(g));
    for &(s, d) in flows {
        let p = route(s, d);
        add_load(g, &p);
        paths.push(p);
        history.push(snapshot(g));
    }
    (paths, history)
}

fn packet_positions(paths: &[Vec<usize>], t: f64) -> Vec<(usize, usize, f64)> {
    paths
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            let l = p.len().checked_sub(1)?;
            if l == 0 {
                return None;
            }
            let phase = (t + i as f64 * 0.7).rem_euclid(l as f64);
            let k = phase.floor() as usize;
            Some((p[k], p[k + 1], phase - k as f64))
        })
        .collect()
}

fn write_frame(
    enc: &mut Encoder<BufWriter<File>>,
    rgb: &mut [u8],
    title: &str,
    subtitle: &str,
    xy: &Side,
    ad: &Side,
    heat_max: u32,
    delay: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    noc::render_frame(rgb, title, subtitle, xy, ad, heat_max)?;
    let mut frame = Frame::from_rgb_speed(FRAME_W as u16, FRAME_H as u16, rgb, 10);
    frame.delay = delay;
    enc.write_frame(&frame)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let flows = random_flows(N_FLOWS, SEED);
    let n_flows = flows.len();

    let xy_g = build_mesh();
    let (xy_paths, xy_hist) = route_pass(&flows, &xy_g, xy_route);

    let ad_g = build_mesh();
    let mut algo: Dijkstra<f64> = Dijkstra::new();
    let mut buf: SsspBuffers<f64, Dyn> = SsspBuffers::new_inf(Dyn(noc::N * noc::N));
    let (ad_paths, ad_hist) = route_pass(&flows, &ad_g, |s, d| {
        adaptive_route(&mut algo, &ad_g, &mut buf, s, d)
    });

    let xy_max = max_load(xy_hist.last().unwrap());
    let ad_max = max_load(ad_hist.last().unwrap());
    let xy_str = mean_stretch(&flows, &xy_paths);
    let ad_str = mean_stretch(&flows, &ad_paths);
    let heat_max = xy_max.max(ad_max);
    println!(
        "XY:       max load {}/cap {}, mean stretch {:.2}",
        xy_max, CAP, xy_str
    );
    println!(
        "Adaptive: max load {}/cap {}, mean stretch {:.2}",
        ad_max, CAP, ad_str
    );

    if let Some(dir) = Path::new(OUT).parent() {
        create_dir_all(dir)?;
    }
    let mut enc = Encoder::new(
        BufWriter::new(File::create(OUT)?),
        FRAME_W as u16,
        FRAME_H as u16,
        &[],
    )?;
    enc.set_repeat(Repeat::Infinite)?;
    let mut rgb = vec![0u8; (FRAME_W * FRAME_H * 3) as usize];

    for step in 0..=n_flows {
        let xy_now = max_load(&xy_hist[step]);
        let ad_now = max_load(&ad_hist[step]);
        let (title, subtitle, xy_curr, ad_curr) = if step == 0 {
            (
                "NoC route allocation".to_string(),
                format!(
                    "{}x{} mesh, {n_flows} random flows, wire cap = {CAP}",
                    noc::N,
                    noc::N
                ),
                None,
                None,
            )
        } else {
            let (s, d) = flows[step - 1];
            let (sr, sc) = rc(s);
            let (dr, dc) = rc(d);
            (
                format!("Allocating flow {step}/{n_flows}"),
                format!(
                    "PE({sr},{sc}) -> PE({dr},{dc})    max load: XY {xy_now} · Adaptive {ad_now}"
                ),
                Some(&xy_paths[step - 1]),
                Some(&ad_paths[step - 1]),
            )
        };
        let xy = Side {
            paths: &xy_paths[..step],
            loads: &xy_hist[step],
            current_path: xy_curr,
            packets: &[],
        };
        let ad = Side {
            paths: &ad_paths[..step],
            loads: &ad_hist[step],
            current_path: ad_curr,
            packets: &[],
        };
        write_frame(
            &mut enc,
            &mut rgb,
            &title,
            &subtitle,
            &xy,
            &ad,
            heat_max,
            ALLOC_DELAY,
        )?;
    }

    let xy_loads = xy_hist.last().unwrap();
    let ad_loads = ad_hist.last().unwrap();
    let summary = format!(
        "max load: XY {xy_max} · Adaptive {ad_max}    stretch: XY {xy_str:.2} · Adaptive {ad_str:.2}"
    );
    let title = "Steady-state dataflow".to_string();
    for f in 0..DATAFLOW_FRAMES {
        let t = f as f64 * DATAFLOW_STEP;
        let xy_packets = packet_positions(&xy_paths, t);
        let ad_packets = packet_positions(&ad_paths, t);
        let xy = Side {
            paths: &xy_paths,
            loads: xy_loads,
            current_path: None,
            packets: &xy_packets,
        };
        let ad = Side {
            paths: &ad_paths,
            loads: ad_loads,
            current_path: None,
            packets: &ad_packets,
        };
        write_frame(
            &mut enc,
            &mut rgb,
            &title,
            &summary,
            &xy,
            &ad,
            heat_max,
            DATAFLOW_DELAY,
        )?;
    }

    let xy = Side {
        paths: &xy_paths,
        loads: xy_loads,
        current_path: None,
        packets: &[],
    };
    let ad = Side {
        paths: &ad_paths,
        loads: ad_loads,
        current_path: None,
        packets: &[],
    };
    for _ in 0..HOLD {
        write_frame(
            &mut enc,
            &mut rgb,
            "Final allocation",
            &summary,
            &xy,
            &ad,
            heat_max,
            HOLD_DELAY,
        )?;
    }

    println!("wrote {OUT}");
    Ok(())
}
