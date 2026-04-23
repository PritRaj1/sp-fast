/// Vertex state for visualisation.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VertexState {
    Unvisited,
    InQueue,
    Visited,
    Path,
    Start,
    End,
}

/// Edge state for graph visualisation.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EdgeState {
    Default,
    Relaxed,
    Path,
}

/// Visit-order `t` in [0, 1].
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
