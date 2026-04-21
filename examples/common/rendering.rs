use plotters::prelude::*;

use super::state::{CellState, EdgeState, VertexState, visited_gradient_rgb};
use super::vis::GraphVisState;

// =============================================================================
// Maze rendering
// =============================================================================

pub const CELL_SIZE: u32 = 25;

pub fn get_color_palette() -> Vec<[u8; 3]> {
    let mut palette = Vec::with_capacity(256);

    palette.push([0, 0, 0]); // 0: Wall - black
    palette.push([60, 60, 60]); // 1: Unvisited - grey
    palette.push([255, 200, 50]); // 2: In queue/frontier - orange
    palette.push([100, 180, 255]); // 3: Visited - light blue
    palette.push([50, 255, 100]); // 4: Path - green
    palette.push([255, 80, 80]); // 5: Start - red
    palette.push([180, 50, 255]); // 6: End - purple
    palette.push([255, 255, 255]); // 7: Grid lines - white
    palette.push([40, 40, 40]); // 8: Background - dark grey

    // Gradient for visited cells from dark blue (early) to cyan (late)
    for i in 0..64 {
        let (r, g, b) = visited_gradient_rgb(i, 63);
        palette.push([r, g, b]);
    }

    while palette.len() < 256 {
        palette.push([0, 0, 0]);
    }

    palette
}

pub fn cell_state_to_palette_index(state: CellState) -> u8 {
    match state {
        CellState::Wall => 0,
        CellState::Vertex(VertexState::Unvisited) => 1,
        CellState::Vertex(VertexState::InQueue) => 2,
        CellState::Vertex(VertexState::Visited) => 3,
        CellState::Vertex(VertexState::Path) => 4,
        CellState::Vertex(VertexState::Start) => 5,
        CellState::Vertex(VertexState::End) => 6,
    }
}

pub fn visited_gradient_index(order: usize, max_order: usize) -> u8 {
    let t = if max_order > 0 {
        (order as f32 / max_order as f32).min(1.0)
    } else {
        0.0
    };
    9 + (t * 63.0) as u8
}

pub struct RenderParams<'a> {
    pub grid: &'a [Vec<CellState>],
    pub visit_order: &'a [Vec<Option<usize>>],
    pub max_visited: usize,
    pub title: &'a str,
}

pub fn render_frame(
    png_path: &str,
    params: RenderParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let rows = params.grid.len();
    let cols = params.grid[0].len();
    let width = cols as u32 * CELL_SIZE;
    let height = rows as u32 * CELL_SIZE + 40;
    let palette = get_color_palette();

    {
        let root = BitMapBackend::new(png_path, (width, height)).into_drawing_area();
        root.fill(&RGBColor(40, 40, 50))?;

        root.draw(&Text::new(
            params.title,
            (10, 10),
            ("sans-serif", 20).into_font().color(&WHITE),
        ))?;

        let plot_area = root.margin(40, 0, 0, 0);

        for row in 0..rows {
            for col in 0..cols {
                let state = params.grid[row][col];
                let x0 = col as i32 * CELL_SIZE as i32;
                let y0 = row as i32 * CELL_SIZE as i32;
                let x1 = x0 + CELL_SIZE as i32;
                let y1 = y0 + CELL_SIZE as i32;

                let color_idx = if matches!(state, CellState::Vertex(VertexState::Visited)) {
                    if let Some(order) = params.visit_order[row][col] {
                        visited_gradient_index(order, params.max_visited)
                    } else {
                        cell_state_to_palette_index(state)
                    }
                } else {
                    cell_state_to_palette_index(state)
                };

                let [r, g, b] = palette[color_idx as usize];
                plot_area.draw(&Rectangle::new(
                    [(x0, y0), (x1, y1)],
                    RGBColor(r, g, b).filled(),
                ))?;

                plot_area.draw(&Rectangle::new(
                    [(x0, y0), (x1, y1)],
                    RGBColor(80, 80, 90).stroke_width(1),
                ))?;
            }
        }

        root.present()?;
    }

    Ok(())
}

// =============================================================================
// Graph rendering
// =============================================================================

pub const GRAPH_WIDTH: u32 = 800;
pub const GRAPH_HEIGHT: u32 = 800;
pub const TITLE_HEIGHT: u32 = 40;
pub const NODE_RADIUS: i32 = 4;

fn node_color(state: VertexState) -> RGBColor {
    match state {
        VertexState::Unvisited => RGBColor(80, 80, 80),
        VertexState::InQueue => RGBColor(255, 200, 50),
        VertexState::Visited => RGBColor(100, 180, 255),
        VertexState::Path => RGBColor(50, 255, 100),
        VertexState::Start => RGBColor(255, 80, 80),
        VertexState::End => RGBColor(180, 50, 255),
    }
}

fn visited_gradient_color(order: usize, max_order: usize) -> RGBColor {
    let (r, g, b) = visited_gradient_rgb(order, max_order);
    RGBColor(r, g, b)
}

fn edge_color(state: EdgeState) -> RGBColor {
    match state {
        EdgeState::Default => RGBColor(50, 50, 50),
        EdgeState::Relaxed => RGBColor(80, 120, 160),
        EdgeState::Path => RGBColor(50, 255, 100),
    }
}

fn edge_width(state: EdgeState) -> u32 {
    match state {
        EdgeState::Default => 1,
        EdgeState::Relaxed => 1,
        EdgeState::Path => 3,
    }
}

pub struct GraphRenderParams<'a> {
    pub state: &'a GraphVisState,
    pub title: &'a str,
}

pub fn render_graph_frame(
    png_path: &str,
    params: GraphRenderParams,
) -> Result<(), Box<dyn std::error::Error>> {
    let width = GRAPH_WIDTH;
    let height = GRAPH_HEIGHT + TITLE_HEIGHT;
    let state = params.state;
    let graph = &state.graph;

    let (min_x, min_y, max_x, max_y) = graph.bounds();
    let padding = 0.05;
    let range_x = max_x - min_x + 2.0 * padding;
    let range_y = max_y - min_y + 2.0 * padding;

    let to_screen = |x: f64, y: f64| -> (i32, i32) {
        let sx = ((x - min_x + padding) / range_x * (width - 20) as f64) as i32 + 10;
        let sy = ((y - min_y + padding) / range_y * (height - TITLE_HEIGHT - 20) as f64) as i32
            + TITLE_HEIGHT as i32
            + 10;
        (sx, sy)
    };

    {
        let root = BitMapBackend::new(png_path, (width, height)).into_drawing_area();
        root.fill(&RGBColor(10, 10, 10))?;

        root.draw(&Text::new(
            params.title,
            (10, 10),
            ("sans-serif", 20).into_font().color(&WHITE),
        ))?;

        // Draw edges
        for (u, neighbors) in graph.adjacency.iter().enumerate() {
            let (x1, y1) = graph.positions[u];
            let (sx1, sy1) = to_screen(x1, y1);

            // Draw each edge once
            for &(v, _) in neighbors {
                if u < v {
                    let (x2, y2) = graph.positions[v];
                    let (sx2, sy2) = to_screen(x2, y2);

                    let edge_state = state.edge_states[u][v];
                    let color = edge_color(edge_state);
                    let width = edge_width(edge_state);

                    root.draw(&PathElement::new(
                        vec![(sx1, sy1), (sx2, sy2)],
                        color.stroke_width(width),
                    ))?;
                }
            }
        }

        // Draw nodes on top
        for (i, &(x, y)) in graph.positions.iter().enumerate() {
            let (sx, sy) = to_screen(x, y);
            let node_state = state.node_states[i];

            let color = if node_state == VertexState::Visited {
                if let Some(order) = state.visit_order[i] {
                    visited_gradient_color(order, state.max_visited)
                } else {
                    node_color(node_state)
                }
            } else {
                node_color(node_state)
            };

            root.draw(&Circle::new((sx, sy), NODE_RADIUS, color.filled()))?;

            // Highlight start/end with bigger outline
            if node_state == VertexState::Start || node_state == VertexState::End {
                root.draw(&Circle::new(
                    (sx, sy),
                    NODE_RADIUS + 2,
                    color.stroke_width(2),
                ))?;
            }
        }

        root.present()?;
    }

    Ok(())
}
