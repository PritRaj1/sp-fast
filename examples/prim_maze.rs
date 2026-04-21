mod common;

use std::collections::BinaryHeap;
use std::fs;

use common::gif_utils::{png_to_gif_frame, setup_gif};
use common::maps::{GridMap, maze};
use common::rendering::{CELL_SIZE, RenderParams, render_frame};
use common::vis::VisState;

#[derive(Clone, Copy, PartialEq, Eq)]
struct HeapEntry {
    key: u64,
    vertex: usize,
}

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

fn prim_visual(map: &GridMap, start: (usize, usize)) -> (Vec<VisState>, usize) {
    let n = map.rows * map.cols;
    let source = map.to_vertex(start.0, start.1);

    let mut key = vec![u64::MAX; n];
    let mut in_mst = vec![false; n];
    let mut heap = BinaryHeap::new();

    key[source] = 0;
    heap.push(HeapEntry {
        key: 0,
        vertex: source,
    });

    let mut state = VisState::new_mst(map, start);
    let mut frames = Vec::new();
    frames.push(state.clone());

    let directions: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    let mut mst_size = 0usize;

    while let Some(HeapEntry { key: k, vertex: u }) = heap.pop() {
        if k > key[u] || in_mst[u] {
            continue;
        }

        in_mst[u] = true;
        let (row, col) = map.to_coords(u);
        state.mark_in_mst(row, col);
        mst_size += 1;

        for (dr, dc) in directions {
            let nr = row as i32 + dr;
            let nc = col as i32 + dc;

            if nr >= 0 && nc >= 0 {
                let nr = nr as usize;
                let nc = nc as usize;

                if map.is_passable(nr, nc) {
                    let v = map.to_vertex(nr, nc);

                    if !in_mst[v] && 1 < key[v] {
                        key[v] = 1;
                        heap.push(HeapEntry { key: 1, vertex: v });
                        state.mark_in_queue(nr, nc);
                    }
                }
            }
        }

        frames.push(state.clone());
    }

    for _ in 0..10 {
        frames.push(state.clone());
    }

    (frames, mst_size)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (map, start, _) = maze();

    println!("Running Prim's MST on grid.");
    let (frames, mst_size) = prim_visual(&map, start);
    let num_frames = frames.len();

    let width = (map.cols as u32 * CELL_SIZE) as u16;
    let height = (map.rows as u32 * CELL_SIZE + 40) as u16;

    let gif_path = "examples/gifs/prim_maze.gif";
    let png_path = "examples/gifs/_temp_frame.png";

    let mut encoder = setup_gif(gif_path, width, height)?;

    for (i, frame) in frames.iter().enumerate() {
        let title = if i < num_frames - 10 {
            format!("Prim MST: {} cells", frame.max_visited)
        } else {
            format!("MST: {} cells", mst_size)
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
    println!("MST covers {} cells", mst_size);
    println!("Saved to {}", gif_path);

    Ok(())
}
