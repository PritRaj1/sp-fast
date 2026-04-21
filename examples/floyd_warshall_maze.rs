mod common;

use std::fs;

use common::gif_utils::{png_to_gif_frame, setup_gif};
use common::maps::{GridMap, maze};
use common::rendering::{CELL_SIZE, RenderParams, render_frame};
use common::vis::VisState;

fn floyd_warshall_visual(
    map: &GridMap,
    start: (usize, usize),
    end: (usize, usize),
) -> (Vec<VisState>, Vec<(usize, usize)>) {
    let n = map.rows * map.cols;
    let source = map.to_vertex(start.0, start.1);
    let target = map.to_vertex(end.0, end.1);

    // Distance and next matrices (flat)
    let mut dist = vec![u64::MAX; n * n];
    let mut next = vec![usize::MAX; n * n];

    // Init diag
    for i in 0..n {
        dist[i * n + i] = 0;
        next[i * n + i] = i;
    }

    // Init from edges
    let directions: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    for row in 0..map.rows {
        for col in 0..map.cols {
            if !map.is_passable(row, col) {
                continue;
            }
            let u = map.to_vertex(row, col);
            for (dr, dc) in directions {
                let nr = row as i32 + dr;
                let nc = col as i32 + dc;
                if nr >= 0 && nc >= 0 {
                    let nr = nr as usize;
                    let nc = nc as usize;
                    if map.is_passable(nr, nc) {
                        let v = map.to_vertex(nr, nc);
                        dist[u * n + v] = 1;
                        next[u * n + v] = v;
                    }
                }
            }
        }
    }

    let mut frames = Vec::new();
    let mut vis_state = VisState::new(map, start, end);
    frames.push(vis_state.clone());

    // Intermed vertices k
    for k in 0..n {
        let (kr, kc) = map.to_coords(k);
        if !map.is_passable(kr, kc) {
            continue;
        }

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

                let new_dist = d_ik + d_kj;
                if new_dist < dist[i * n + j] {
                    dist[i * n + j] = new_dist;
                    next[i * n + j] = next[i * n + k];
                    any_update = true;
                }
            }
        }

        // Mark k done
        if any_update {
            vis_state.mark_visited(kr, kc);
            frames.push(vis_state.clone());
        }
    }

    // Reconstruct path
    let mut path = Vec::new();
    if dist[source * n + target] < u64::MAX {
        let mut curr = source;
        while curr != target {
            let (r, c) = map.to_coords(curr);
            path.push((r, c));
            curr = next[curr * n + target];
            if curr == usize::MAX {
                break;
            }
        }
        if curr == target {
            path.push(end);
        }
    }

    vis_state.mark_path(&path);
    for _ in 0..10 {
        frames.push(vis_state.clone());
    }

    (frames, path)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (map, start, end) = maze();

    println!("Running Floyd-Warshall (APSP).");
    let (frames, path) = floyd_warshall_visual(&map, start, end);
    let num_frames = frames.len();
    let path_len = path.len().saturating_sub(1);

    let width = (map.cols as u32 * CELL_SIZE) as u16;
    let height = (map.rows as u32 * CELL_SIZE + 40) as u16;

    let gif_path = "examples/gifs/floyd_warshall_maze.gif";
    let png_path = "examples/gifs/_temp_frame.png";

    let mut encoder = setup_gif(gif_path, width, height)?;

    for (i, frame) in frames.iter().enumerate() {
        let title = if i < num_frames - 10 {
            format!("Floyd-Warshall: k={}", i)
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
