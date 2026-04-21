mod common;

use std::collections::BinaryHeap;
use std::fs;

use common::gif_utils::{png_to_gif_frame, setup_gif};
use common::graphs::{EuclideanGraph, euclidean_500};
use common::rendering::{
    GRAPH_HEIGHT, GRAPH_WIDTH, GraphRenderParams, TITLE_HEIGHT, render_graph_frame,
};
use common::vis::GraphVisState;

const SCALE: f64 = 1_000_000.0;

#[derive(Clone, Copy, PartialEq)]
struct HeapEntry {
    f: u64, // f = g + h
    g: u64,
    vertex: usize,
}

impl Eq for HeapEntry {}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.f.cmp(&self.f).then_with(|| other.g.cmp(&self.g))
    }
}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn euclidean_heuristic(graph: &EuclideanGraph, v: usize, target: usize) -> u64 {
    let (vx, vy) = graph.positions[v];
    let (tx, ty) = graph.positions[target];
    let dist = ((tx - vx).powi(2) + (ty - vy).powi(2)).sqrt();
    (dist * SCALE) as u64
}

fn astar_visual(
    graph: &EuclideanGraph,
    start: usize,
    end: usize,
) -> (Vec<GraphVisState>, Vec<usize>) {
    let n = graph.n_vertices();

    let mut g_score = vec![u64::MAX; n];
    let mut parent = vec![usize::MAX; n];
    let mut visited = vec![false; n];
    let mut heap = BinaryHeap::new();

    g_score[start] = 0;
    heap.push(HeapEntry {
        f: euclidean_heuristic(graph, start, end),
        g: 0,
        vertex: start,
    });

    let mut frames = Vec::new();
    let mut vis_state = GraphVisState::new(graph, start, end);
    frames.push(vis_state.clone());

    while let Some(HeapEntry { f: _, g, vertex: u }) = heap.pop() {
        if g > g_score[u] || visited[u] {
            continue;
        }

        visited[u] = true;
        let parent_vertex = if parent[u] != usize::MAX {
            Some(parent[u])
        } else {
            None
        };
        vis_state.mark_visited(u, parent_vertex);

        if u == end {
            break;
        }

        for &(v, weight) in graph.neighbors(u) {
            let edge_dist = (weight * SCALE) as u64;
            let tentative_g = g_score[u].saturating_add(edge_dist);

            if tentative_g < g_score[v] {
                g_score[v] = tentative_g;
                parent[v] = u;
                let h = euclidean_heuristic(graph, v, end);
                heap.push(HeapEntry {
                    f: tentative_g + h,
                    g: tentative_g,
                    vertex: v,
                });
                vis_state.mark_in_queue(v);
            }
        }

        frames.push(vis_state.clone());
    }

    let mut path = Vec::new();
    if g_score[end] < u64::MAX {
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

    println!("Running A* with Euclidean heuristic.");
    let (frames, path) = astar_visual(&graph, start, end);
    let num_frames = frames.len();
    let path_len = path.len().saturating_sub(1);

    let width = GRAPH_WIDTH as u16;
    let height = (GRAPH_HEIGHT + TITLE_HEIGHT) as u16;

    let gif_path = "examples/gifs/astar_euclid.gif";
    let png_path = "examples/gifs/_temp_frame.png";

    let mut encoder = setup_gif(gif_path, width, height)?;

    for (i, frame) in frames.iter().enumerate() {
        let title = if i < num_frames - 15 {
            format!("A*: Step {}", i)
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
