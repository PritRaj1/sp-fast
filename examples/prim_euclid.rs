mod common;

use std::collections::BinaryHeap;
use std::fs;

use common::gif_utils::{png_to_gif_frame, setup_gif};
use common::graphs::{EuclideanGraph, random_euclidean_graph_connected};
use common::rendering::{
    GRAPH_HEIGHT, GRAPH_WIDTH, GraphRenderParams, TITLE_HEIGHT, render_graph_frame,
};
use common::vis::GraphVisState;

#[derive(Clone, Copy, PartialEq)]
struct HeapEntry {
    key: u64,
    vertex: usize,
}

impl Eq for HeapEntry {}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.key.cmp(&self.key)
    }
}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn prim_visual(graph: &EuclideanGraph, start: usize) -> (Vec<GraphVisState>, f64) {
    let n = graph.n_vertices();
    const SCALE: f64 = 1_000_000.0;

    let mut key = vec![u64::MAX; n];
    let mut parent = vec![usize::MAX; n];
    let mut weight_to = vec![0.0f64; n];
    let mut in_mst = vec![false; n];
    let mut heap = BinaryHeap::new();

    key[start] = 0;
    heap.push(HeapEntry {
        key: 0,
        vertex: start,
    });

    let mut state = GraphVisState::new_mst(graph, start);
    let mut frames = Vec::new();
    let mut total_weight = 0.0f64;
    frames.push(state.clone());

    while let Some(HeapEntry { key: k, vertex: u }) = heap.pop() {
        if k > key[u] || in_mst[u] {
            continue;
        }

        in_mst[u] = true;
        let parent_vertex = if parent[u] != usize::MAX {
            total_weight += weight_to[u];
            Some(parent[u])
        } else {
            None
        };
        state.mark_in_mst(u, parent_vertex);

        for &(v, w) in graph.neighbors(u) {
            let edge_key = (w * SCALE) as u64;

            if !in_mst[v] && edge_key < key[v] {
                key[v] = edge_key;
                parent[v] = u;
                weight_to[v] = w;
                heap.push(HeapEntry {
                    key: edge_key,
                    vertex: v,
                });
                state.mark_in_queue(v);
            }
        }

        frames.push(state.clone());
    }

    for _ in 0..15 {
        frames.push(state.clone());
    }

    (frames, total_weight)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let graph = random_euclidean_graph_connected(500, 6, 0.08, 42);
    let start = 0;

    println!("Graph: {} vertices", graph.n_vertices());
    println!("Running Prim's MST.");

    let (frames, total_weight) = prim_visual(&graph, start);
    let num_frames = frames.len();

    let width = GRAPH_WIDTH as u16;
    let height = (GRAPH_HEIGHT + TITLE_HEIGHT) as u16;

    let gif_path = "examples/gifs/prim_euclid.gif";
    let png_path = "examples/gifs/_temp_frame.png";

    let mut encoder = setup_gif(gif_path, width, height)?;

    for (i, frame) in frames.iter().enumerate() {
        let title = if i < num_frames - 15 {
            format!("Prim MST: {} vertices", frame.max_visited)
        } else {
            format!("MST: {:.4} total", total_weight)
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
    println!("MST weight: {:.4}", total_weight);
    println!("Saved to {}", gif_path);

    Ok(())
}
