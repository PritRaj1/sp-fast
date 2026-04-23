/// Vertex/cell state for visuals (grids and graphs).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VertexState {
    Unvisited,
    InQueue,
    Visited,
    Path,
    Start,
    End,
}

/// Edge state for graph visual.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EdgeState {
    Default,
    Relaxed,
    Path,
}

/// Grid-specific cell state (includes Wall).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    Wall,
    Vertex(VertexState),
}

impl CellState {
    pub fn unvisited() -> Self {
        CellState::Vertex(VertexState::Unvisited)
    }

    pub fn is_unvisited(&self) -> bool {
        matches!(self, CellState::Vertex(VertexState::Unvisited))
    }

    pub fn is_start(&self) -> bool {
        matches!(self, CellState::Vertex(VertexState::Start))
    }

    pub fn is_end(&self) -> bool {
        matches!(self, CellState::Vertex(VertexState::End))
    }
}

/// Gradient interpolation factor from visit order.
pub fn visit_gradient_t(order: usize, max_order: usize) -> f32 {
    if max_order > 0 {
        (order as f32 / max_order as f32).min(1.0)
    } else {
        0.0
    }
}

/// Gradient RGB from visit order.
pub fn visited_gradient_rgb(order: usize, max_order: usize) -> (u8, u8, u8) {
    let t = visit_gradient_t(order, max_order);
    let r = (30.0 + t * 70.0) as u8;
    let g = (80.0 + t * 120.0) as u8;
    let b = (180.0 + t * 75.0) as u8;
    (r, g, b)
}
