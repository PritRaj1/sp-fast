mod common;

use std::collections::BinaryHeap;
use std::fs;

use common::gif_utils::{png_to_gif_frame, setup_gif};
use common::maps::{GridMap, maze};
use common::rendering::{CELL_SIZE, RenderParams, render_frame};
use common::vis::VisState;

#[derive(Clone, Copy, PartialEq, Eq)]
struct HeapEntry {
    dist: u64,
    vertex: usize,
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.dist.cmp(&self.dist)
    }
}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn dijkstra_visual(
    map: &GridMap,
    start: (usize, usize),
    end: (usize, usize),
) -> (Vec<VisState>, Vec<(usize, usize)>) {
    let n = map.rows * map.cols;
    let source = map.to_vertex(start.0, start.1);
    let target = map.to_vertex(end.0, end.1);

    let mut dist = vec![u64::MAX; n];
    let mut parent = vec![usize::MAX; n];
    let mut visited = vec![false; n];
    let mut heap = BinaryHeap::new();

    dist[source] = 0;
    heap.push(HeapEntry {
        dist: 0,
        vertex: source,
    });

    let mut frames = Vec::new();
    let mut vis_state = VisState::new(map, start, end);
    frames.push(vis_state.clone());

    let directions: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

    while let Some(HeapEntry { dist: d, vertex: u }) = heap.pop() {
        if d > dist[u] || visited[u] {
            continue;
        }

        visited[u] = true;
        let (row, col) = map.to_coords(u);
        vis_state.mark_visited(row, col);

        if u == target {
            break;
        }

        for (dr, dc) in directions {
            let nr = row as i32 + dr;
            let nc = col as i32 + dc;

            if nr >= 0 && nc >= 0 {
                let nr = nr as usize;
                let nc = nc as usize;

                if map.is_passable(nr, nc) {
                    let v = map.to_vertex(nr, nc);
                    let new_dist = dist[u] + 1;

                    if new_dist < dist[v] {
                        dist[v] = new_dist;
                        parent[v] = u;
                        heap.push(HeapEntry {
                            dist: new_dist,
                            vertex: v,
                        });
                        vis_state.mark_in_queue(nr, nc);
                    }
                }
            }
        }

        frames.push(vis_state.clone());
    }

    let mut path = Vec::new();
    if dist[target] < u64::MAX {
        let mut current = target;
        while current != usize::MAX && current != source {
            let (r, c) = map.to_coords(current);
            path.push((r, c));
            current = parent[current];
        }
        path.push(start);
        path.reverse();
    }

    vis_state.mark_path(&path);
    for _ in 0..10 {
        frames.push(vis_state.clone());
    }

    (frames, path)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (map, start, end) = maze();

    println!("Running Dijkstra.");
    let (frames, path) = dijkstra_visual(&map, start, end);
    let num_frames = frames.len();
    let path_len = path.len().saturating_sub(1);

    let width = (map.cols as u32 * CELL_SIZE) as u16;
    let height = (map.rows as u32 * CELL_SIZE + 40) as u16;

    let gif_path = "examples/gifs/dijkstra_maze.gif";
    let png_path = "examples/gifs/_temp_frame.png";

    let mut encoder = setup_gif(gif_path, width, height)?;

    for (i, frame) in frames.iter().enumerate() {
        let title = if i < num_frames - 10 {
            format!("Dijkstra: Step {}", i)
        } else {
            format!("Path: {} steps", path_len)
        };

        render_frame(
            png_path,
            RenderParams {
                grid: &frame.grid,
                visit_order: &frame.visit_order,
                max_visited: frame.max_visited,
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
