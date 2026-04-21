mod common;

use std::fs;

use common::gif_utils::{png_to_gif_frame, setup_gif};
use common::graphs::{EuclideanGraph, euclidean_500};
use common::rendering::{
    GRAPH_HEIGHT, GRAPH_WIDTH, GraphRenderParams, TITLE_HEIGHT, render_graph_frame,
};
use common::vis::GraphVisState;

fn floyd_warshall_visual(
    graph: &EuclideanGraph,
    start: usize,
    end: usize,
) -> (Vec<GraphVisState>, Vec<usize>) {
    let n = graph.n_vertices();

    const SCALE: f64 = 1_000_000.0;

    // Distance and next matrices (flat)
    let mut dist = vec![u64::MAX; n * n];
    let mut next = vec![usize::MAX; n * n];

    // Initi diag
    for i in 0..n {
        dist[i * n + i] = 0;
        next[i * n + i] = i;
    }

    // Init from edges
    for u in 0..n {
        for &(v, weight) in graph.neighbors(u) {
            let d = (weight * SCALE) as u64;
            if d < dist[u * n + v] {
                dist[u * n + v] = d;
                next[u * n + v] = v;
            }
        }
    }

    let mut frames = Vec::new();
    let mut vis_state = GraphVisState::new(graph, start, end);
    frames.push(vis_state.clone());

    // Intermed vertices k
    for k in 0..n {
        let mut any_update = false;

        for i in 0..n {
            let d_ik = dist[i * n + k];
            if d_ik == u64::MAX {
                continue;
            }

            for j in 0..n {
                let d_kj = dist[k * n + j];
                if d_kj == u64::MAX {
                    continue;
                }

                let new_dist = d_ik.saturating_add(d_kj);
                if new_dist < dist[i * n + j] {
                    dist[i * n + j] = new_dist;
                    next[i * n + j] = next[i * n + k];
                    any_update = true;
                }
            }
        }

        // Mark k done
        if any_update {
            vis_state.mark_visited(k, None);
            frames.push(vis_state.clone());
        }
    }

    // Reconstruct path start to end
    let mut path = Vec::new();
    if dist[start * n + end] < u64::MAX {
        let mut curr = start;
        while curr != end {
            path.push(curr);
            curr = next[curr * n + end];
            if curr == usize::MAX {
                break;
            }
        }
        if curr == end {
            path.push(end);
        }
    }

    vis_state.mark_path(&path);
    for _ in 0..15 {
        frames.push(vis_state.clone());
    }

    (frames, path)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (graph, start, end) = euclidean_500();

    println!(
        "Graph: {} vertices, start={}, end={}",
        graph.n_vertices(),
        start,
        end
    );

    println!("Running Floyd-Warshall (APSP).");
    let (frames, path) = floyd_warshall_visual(&graph, start, end);
    let num_frames = frames.len();
    let path_len = path.len().saturating_sub(1);

    let width = GRAPH_WIDTH as u16;
    let height = (GRAPH_HEIGHT + TITLE_HEIGHT) as u16;

    let gif_path = "examples/gifs/floyd_warshall_euclid.gif";
    let png_path = "examples/gifs/_temp_frame.png";

    let mut encoder = setup_gif(gif_path, width, height)?;

    for (i, frame) in frames.iter().enumerate() {
        let title = if i < num_frames - 15 {
            format!("Floyd-Warshall: k={}", i)
        } else {
            format!("Path: {} edges", path_len)
        };

        render_graph_frame(
            png_path,
            GraphRenderParams {
                state: frame,
                title: &title,
            },
        )?;

        let gif_frame = png_to_gif_frame(png_path, width, height)?;
        encoder.write_frame(&gif_frame)?;
    }

    fs::remove_file(png_path).ok();
    println!("Saved to {}", gif_path);

    Ok(())
}
