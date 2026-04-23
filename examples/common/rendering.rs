use plotters::prelude::*;

use super::state::{EdgeState, VertexState, visited_gradient_rgb};
use super::vis::GraphVisState;

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

    let root = BitMapBackend::new(png_path, (width, height)).into_drawing_area();
    root.fill(&RGBColor(10, 10, 10))?;

    root.draw(&Text::new(
        params.title,
        (10, 10),
        ("sans-serif", 20).into_font().color(&WHITE),
    ))?;

    for (u, neighbors) in graph.adjacency.iter().enumerate() {
        let (x1, y1) = graph.positions[u];
        let (sx1, sy1) = to_screen(x1, y1);

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

        if node_state == VertexState::Start || node_state == VertexState::End {
            root.draw(&Circle::new(
                (sx, sy),
                NODE_RADIUS + 2,
                color.stroke_width(2),
            ))?;
        }
    }

    root.present()?;
    Ok(())
}
