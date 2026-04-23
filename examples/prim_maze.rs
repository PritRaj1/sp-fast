mod common;

use common::encoder::{make_titles, write_maze_gif};
use common::maps::maze;
use common::vis::VisState;
use sssp_fast::{Dyn, Event, MstAlgorithm, MstBuffers, Prim};

const HOLD: usize = 10;
const OUT: &str = "examples/gifs/prim_maze.gif";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (map, start, _) = maze();
    let source = map.to_vertex(start.0, start.1);
    let n = map.rows * map.cols;

    let mut state = VisState::new_mst(&map, start);
    let mut frames = vec![state.clone()];
    let mut buf: MstBuffers<f64, Dyn> = MstBuffers::new_inf(Dyn(n));

    let mut algo: Prim<f64> = Prim::new();
    algo.run_observed(&map, source, &mut buf, |event| match event {
        Event::Improved { vertex, .. } => {
            let (r, c) = map.to_coords(vertex);
            state.mark_in_queue(r, c);
        }
        Event::Finalized { vertex, .. } => {
            let (r, c) = map.to_coords(vertex);
            state.mark_in_mst(r, c);
            frames.push(state.clone());
        }
        Event::Iteration(_) => {}
    });

    for _ in 0..HOLD {
        frames.push(state.clone());
    }

    let total = buf.total_weight();
    let titles = make_titles(
        frames.len(),
        frames.len() - HOLD,
        |i| format!("Prim MST: step {}", i),
        format!("MST weight: {:.0}", total),
    );
    write_maze_gif(OUT, map.rows, map.cols, &frames, &titles)?;
    println!("wrote {} (MST weight {:.0})", OUT, total);
    Ok(())
}
