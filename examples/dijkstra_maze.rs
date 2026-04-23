mod common;

use common::encoder::{make_titles, write_maze_gif};
use common::maps::maze;
use common::vis::VisState;
use sssp_fast::{Dijkstra, DijkstraConfig, Dyn, Event, SsspAlgorithm, SsspBuffers};

const HOLD: usize = 10;
const OUT: &str = "examples/gifs/dijkstra_maze.gif";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (map, start, end) = maze();
    let source = map.to_vertex(start.0, start.1);
    let target = map.to_vertex(end.0, end.1);
    let n = map.rows * map.cols;

    let mut state = VisState::new(&map, start, end);
    let mut frames = vec![state.clone()];
    let mut buf: SsspBuffers<f64, Dyn> = SsspBuffers::new_inf(Dyn(n));

    let mut algo: Dijkstra<f64> = Dijkstra::with_config(DijkstraConfig::with_target(target));
    algo.run_observed(&map, source, &mut buf, |event| match event {
        Event::Improved { vertex, .. } => {
            let (r, c) = map.to_coords(vertex);
            state.mark_in_queue(r, c);
        }
        Event::Finalized { vertex, .. } => {
            let (r, c) = map.to_coords(vertex);
            state.mark_visited(r, c);
            frames.push(state.clone());
        }
        Event::Iteration(_) => {}
    });

    let path: Vec<(usize, usize)> = buf
        .path_to(target)
        .unwrap_or_default()
        .into_iter()
        .map(|v| map.to_coords(v))
        .collect();
    state.mark_path(&path);
    for _ in 0..HOLD {
        frames.push(state.clone());
    }

    let titles = make_titles(
        frames.len(),
        frames.len() - HOLD,
        |i| format!("Dijkstra: step {}", i),
        format!("Path: {} steps", path.len().saturating_sub(1)),
    );
    write_maze_gif(OUT, map.rows, map.cols, &frames, &titles)?;
    println!("wrote {}", OUT);
    Ok(())
}
