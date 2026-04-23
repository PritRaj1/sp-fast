mod common;

use common::encoder::{make_titles, write_maze_gif};
use common::maps::maze;
use common::vis::VisState;
use sssp_fast::{ApspAlgorithm, ApspBuffers, Event, FloydWarshall};

const HOLD: usize = 10;
const OUT: &str = "examples/gifs/floyd_warshall_maze.gif";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (map, start, end) = maze();
    let source = map.to_vertex(start.0, start.1);
    let target = map.to_vertex(end.0, end.1);
    let n = map.rows * map.cols;

    let mut state = VisState::new(&map, start, end);
    let mut frames = vec![state.clone()];
    let mut buf: ApspBuffers<f64> = ApspBuffers::new(n);

    let mut algo: FloydWarshall<f64> = FloydWarshall::new();
    algo.run_observed(&map, &mut buf, |event| {
        if let Event::Iteration(k) = event {
            let (r, c) = map.to_coords(k);
            if !map.is_wall(r, c) {
                state.mark_visited(r, c);
                frames.push(state.clone());
            }
        }
    });

    let path: Vec<(usize, usize)> = buf
        .path(source, target)
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
        |i| format!("Floyd-Warshall: k={}", i),
        format!("Path: {} steps", path.len().saturating_sub(1)),
    );
    write_maze_gif(OUT, map.rows, map.cols, &frames, &titles)?;
    println!("wrote {}", OUT);
    Ok(())
}
