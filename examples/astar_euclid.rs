mod common;

use common::encoder::{make_titles, write_graph_gif};
use common::graphs::{EuclideanGraph, euclidean_500};
use common::vis::GraphVisState;
use sssp_fast::{AStar, Dyn, Event, Heuristic, SsspAlgorithm, SsspBuffers};

const HOLD: usize = 15;
const OUT: &str = "examples/gifs/astar_euclid.gif";

#[derive(Clone)]
struct Euclidean<'a> {
    g: &'a EuclideanGraph,
}

impl Heuristic<f64> for Euclidean<'_> {
    fn estimate(&self, v: usize, target: usize) -> f64 {
        let (vx, vy) = self.g.positions[v];
        let (tx, ty) = self.g.positions[target];
        ((tx - vx).powi(2) + (ty - vy).powi(2)).sqrt()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (g, start, end) = euclidean_500();
    let n = g.n_vertices();

    let mut state = GraphVisState::new(&g, start, end);
    let mut frames = vec![state.clone()];
    let mut buf: SsspBuffers<f64, Dyn> = SsspBuffers::new_inf(Dyn(n));

    let mut algo: AStar<f64, Euclidean> = AStar::new(end, Euclidean { g: &g });
    algo.run_observed(&g, start, &mut buf, |event| match event {
        Event::Improved { vertex, .. } => state.mark_in_queue(vertex),
        Event::Finalized { vertex, parent, .. } => {
            state.mark_visited(vertex, parent);
            frames.push(state.clone());
        }
        Event::Iteration(_) => {}
    });

    let path = buf.path_to(end).unwrap_or_default();
    state.mark_path(&path);
    for _ in 0..HOLD {
        frames.push(state.clone());
    }

    let titles = make_titles(
        frames.len(),
        frames.len() - HOLD,
        |i| format!("A*: step {}", i),
        format!("Path: {} edges", path.len().saturating_sub(1)),
    );
    write_graph_gif(OUT, &frames, &titles)?;
    println!("wrote {}", OUT);
    Ok(())
}
