mod common;

use std::fs;

use common::gif_utils::{png_to_gif_frame, setup_gif};
use common::graphs::{EuclideanGraph, euclidean_500};
use common::rendering::{
    GRAPH_HEIGHT, GRAPH_WIDTH, GraphRenderParams, TITLE_HEIGHT, render_graph_frame,
};
use common::vis::GraphVisState;

fn bellman_ford_visual(
    graph: &EuclideanGraph,
    start: usize,
    end: usize,
) -> (Vec<GraphVisState>, Vec<usize>) {
    let n = graph.n_vertices();

    const SCALE: f64 = 1_000_000.0;

    let mut dist = vec![u64::MAX; n];
    let mut parent = vec![usize::MAX; n];

    dist[start] = 0;

    let mut frames = Vec::new();
    let mut vis_state = GraphVisState::new(graph, start, end);
    frames.push(vis_state.clone());

    for _ in 0..n.saturating_sub(1) {
        let mut any_improved = false;

        for u in 0..n {
            if dist[u] == u64::MAX {
                continue;
            }

            let mut vertex_improved = false;

            for &(v, weight) in graph.neighbors(u) {
                let edge_dist = (weight * SCALE) as u64;
                let new_dist = dist[u].saturating_add(edge_dist);

                if new_dist < dist[v] {
                    dist[v] = new_dist;
                    parent[v] = u;
                    any_improved = true;
                    vertex_improved = true;

                    vis_state.mark_in_queue(v);
                }
            }

            // Add frame when this vertex caused improvement
            if vertex_improved {
                let parent_vertex = if parent[u] != usize::MAX {
                    Some(parent[u])
                } else {
                    None
                };
                vis_state.mark_visited(u, parent_vertex);
                frames.push(vis_state.clone());
            }
        }

        if !any_improved {
            break;
        }
    }

    // Reconstruct path
    let mut path = Vec::new();
    if dist[end] < u64::MAX {
        let mut current = end;
        while current != usize::MAX {
            path.push(current);
            current = parent[current];
        }
        path.reverse();
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

    println!("Running Bellman-Ford.");
    let (frames, path) = bellman_ford_visual(&graph, start, end);
    let num_frames = frames.len();
    let path_len = path.len().saturating_sub(1);

    let width = GRAPH_WIDTH as u16;
    let height = (GRAPH_HEIGHT + TITLE_HEIGHT) as u16;

    let gif_path = "examples/gifs/bellman_ford_euclid.gif";
    let png_path = "examples/gifs/_temp_frame.png";

    let mut encoder = setup_gif(gif_path, width, height)?;

    for (i, frame) in frames.iter().enumerate() {
        let title = if i < num_frames - 15 {
            format!("Bellman-Ford: Step {}", i)
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
