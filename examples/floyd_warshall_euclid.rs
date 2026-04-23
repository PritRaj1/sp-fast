mod common;

use common::encoder::{make_titles, write_graph_gif};
use common::graphs::euclidean_500;
use common::vis::GraphVisState;
use sssp_fast::{ApspAlgorithm, ApspBuffers, Event, FloydWarshall};

const HOLD: usize = 15;
const OUT: &str = "examples/gifs/floyd_warshall_euclid.gif";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (g, start, end) = euclidean_500();
    let n = g.n_vertices();
    println!("graph: {} vertices, start={}, end={}", n, start, end);

    let mut state = GraphVisState::new(&g, start, end);
    let mut frames = vec![state.clone()];
    let mut buf: ApspBuffers<f64> = ApspBuffers::new(n);

    let mut algo: FloydWarshall<f64> = FloydWarshall::new();
    algo.run_observed(&g, &mut buf, |event| {
        if let Event::Iteration(k) = event {
            state.mark_visited(k, None);
            frames.push(state.clone());
        }
    });

    let path = buf.path(start, end).unwrap_or_default();
    state.mark_path(&path);
    for _ in 0..HOLD {
        frames.push(state.clone());
    }

    let titles = make_titles(
        frames.len(),
        frames.len() - HOLD,
        |i| format!("Floyd-Warshall: k={}", i),
        format!("Path: {} edges", path.len().saturating_sub(1)),
    );
    write_graph_gif(OUT, &frames, &titles)?;
    println!("wrote {}", OUT);
    Ok(())
}
