use super::graphs::EuclideanGraph;
use super::maps::GridMap;
use super::state::{CellState, EdgeState, VertexState};

/// Maze state for SSSP/MST algorithms.
#[derive(Clone)]
pub struct VisState {
    pub grid: Vec<Vec<CellState>>,
    pub visit_order: Vec<Vec<Option<usize>>>,
    pub max_visited: usize,
}

impl VisState {
    pub fn new(map: &GridMap, start: (usize, usize), end: (usize, usize)) -> Self {
        let mut state = Self::new_from_map(map, start);
        state.grid[end.0][end.1] = CellState::Vertex(VertexState::End); // Start + end (path finding)
        state
    }

    pub fn new_mst(map: &GridMap, start: (usize, usize)) -> Self {
        Self::new_from_map(map, start) // Start only
    }

    fn new_from_map(map: &GridMap, start: (usize, usize)) -> Self {
        let mut grid = vec![vec![CellState::unvisited(); map.cols]; map.rows];

        for (row, row_cells) in grid.iter_mut().enumerate() {
            for (col, cell) in row_cells.iter_mut().enumerate() {
                if map.is_wall(row, col) {
                    *cell = CellState::Wall;
                }
            }
        }

        grid[start.0][start.1] = CellState::Vertex(VertexState::Start);

        Self {
            grid,
            visit_order: vec![vec![None; map.cols]; map.rows],
            max_visited: 0,
        }
    }

    pub fn mark_in_queue(&mut self, row: usize, col: usize) {
        if self.grid[row][col].is_unvisited() {
            self.grid[row][col] = CellState::Vertex(VertexState::InQueue);
        }
    }

    pub fn mark_visited(&mut self, row: usize, col: usize) {
        if !self.grid[row][col].is_start() && !self.grid[row][col].is_end() {
            self.grid[row][col] = CellState::Vertex(VertexState::Visited);
        }
        self.visit_order[row][col] = Some(self.max_visited);
        self.max_visited += 1;
    }

    /// Mark cell as part of MST (uses Path color).
    pub fn mark_in_mst(&mut self, row: usize, col: usize) {
        if !self.grid[row][col].is_start() {
            self.grid[row][col] = CellState::Vertex(VertexState::Path);
        }
        self.visit_order[row][col] = Some(self.max_visited);
        self.max_visited += 1;
    }

    pub fn mark_path(&mut self, path: &[(usize, usize)]) {
        for &(row, col) in path {
            if !self.grid[row][col].is_start() && !self.grid[row][col].is_end() {
                self.grid[row][col] = CellState::Vertex(VertexState::Path);
            }
        }
    }
}

/// Graph state for SSSP/MST algorithms.
#[derive(Clone)]
pub struct GraphVisState {
    pub node_states: Vec<VertexState>,
    pub edge_states: Vec<Vec<EdgeState>>,
    pub visit_order: Vec<Option<usize>>,
    pub max_visited: usize,
    pub graph: EuclideanGraph,
}

impl GraphVisState {
    pub fn new(graph: &EuclideanGraph, start: usize, end: usize) -> Self {
        let mut state = Self::new_from_graph(graph, start);
        state.node_states[end] = VertexState::End; // Start + end (path finding)
        state
    }

    pub fn new_mst(graph: &EuclideanGraph, start: usize) -> Self {
        Self::new_from_graph(graph, start) // Start only
    }

    fn new_from_graph(graph: &EuclideanGraph, start: usize) -> Self {
        let n = graph.n_vertices();
        let mut node_states = vec![VertexState::Unvisited; n];
        node_states[start] = VertexState::Start;

        let edge_states = vec![vec![EdgeState::Default; n]; n];

        Self {
            node_states,
            edge_states,
            visit_order: vec![None; n],
            max_visited: 0,
            graph: graph.clone(),
        }
    }

    pub fn mark_in_queue(&mut self, vertex: usize) {
        if self.node_states[vertex] == VertexState::Unvisited {
            self.node_states[vertex] = VertexState::InQueue;
        }
    }

    pub fn mark_visited(&mut self, vertex: usize, parent: Option<usize>) {
        if self.node_states[vertex] != VertexState::Start
            && self.node_states[vertex] != VertexState::End
        {
            self.node_states[vertex] = VertexState::Visited;
        }
        self.visit_order[vertex] = Some(self.max_visited);
        self.max_visited += 1;

        if let Some(p) = parent {
            self.edge_states[p][vertex] = EdgeState::Relaxed;
            self.edge_states[vertex][p] = EdgeState::Relaxed;
        }
    }

    /// Mark vertex as added to MST with edge from parent.
    pub fn mark_in_mst(&mut self, vertex: usize, parent: Option<usize>) {
        if self.node_states[vertex] != VertexState::Start {
            self.node_states[vertex] = VertexState::Visited;
        }
        self.visit_order[vertex] = Some(self.max_visited);
        self.max_visited += 1;

        if let Some(p) = parent {
            self.edge_states[p][vertex] = EdgeState::Path;
            self.edge_states[vertex][p] = EdgeState::Path;
        }
    }

    pub fn mark_path(&mut self, path: &[usize]) {
        for window in path.windows(2) {
            let u = window[0];
            let v = window[1];
            self.edge_states[u][v] = EdgeState::Path;
            self.edge_states[v][u] = EdgeState::Path;
        }

        for &vertex in path {
            if self.node_states[vertex] != VertexState::Start
                && self.node_states[vertex] != VertexState::End
            {
                self.node_states[vertex] = VertexState::Path;
            }
        }
    }
}
