mod common;

use common::encoder::{make_titles, write_graph_gif};
use common::graphs::euclidean_500;
use common::vis::GraphVisState;
use sssp_fast::{Dijkstra, DijkstraConfig, Dyn, Event, SsspAlgorithm, SsspBuffers};

const HOLD: usize = 15;
const OUT: &str = "examples/gifs/dijkstra_euclid.gif";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (g, start, end) = euclidean_500();
    let n = g.n_vertices();
    println!("graph: {} vertices, start={}, end={}", n, start, end);

    let mut state = GraphVisState::new(&g, start, end);
    let mut frames = vec![state.clone()];
    let mut buf: SsspBuffers<f64, Dyn> = SsspBuffers::new_inf(Dyn(n));

    let mut algo: Dijkstra<f64> = Dijkstra::with_config(DijkstraConfig::with_target(end));
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
        |i| format!("Dijkstra: step {}", i),
        format!("Path: {} edges", path.len().saturating_sub(1)),
    );
    write_graph_gif(OUT, &frames, &titles)?;
    println!("wrote {}", OUT);
    Ok(())
}
