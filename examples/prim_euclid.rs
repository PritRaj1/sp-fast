mod common;

use common::encoder::{make_titles, write_graph_gif};
use common::graphs::random_euclidean_graph_connected;
use common::vis::GraphVisState;
use sssp_fast::{Dyn, Event, MstAlgorithm, MstBuffers, Prim};

const HOLD: usize = 15;
const OUT: &str = "examples/gifs/prim_euclid.gif";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let g = random_euclidean_graph_connected(500, 6, 0.08, 42);
    let start = 0;
    let n = g.n_vertices();
    println!("graph: {} vertices", n);

    let mut state = GraphVisState::new_mst(&g, start);
    let mut frames = vec![state.clone()];
    let mut buf: MstBuffers<f64, Dyn> = MstBuffers::new_inf(Dyn(n));

    let mut algo: Prim<f64> = Prim::new();
    algo.run_observed(&g, start, &mut buf, |event| match event {
        Event::Improved { vertex, .. } => state.mark_in_queue(vertex),
        Event::Finalized { vertex, parent, .. } => {
            state.mark_in_mst(vertex, parent);
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
        format!("MST weight: {:.4}", total),
    );
    write_graph_gif(OUT, &frames, &titles)?;
    println!("wrote {} (MST weight {:.4})", OUT, total);
    Ok(())
}
