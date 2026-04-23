use super::graphs::EuclideanGraph;
use super::state::{EdgeState, VertexState};

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
        state.node_states[end] = VertexState::End;
        state
    }

    pub fn new_mst(graph: &EuclideanGraph, start: usize) -> Self {
        Self::new_from_graph(graph, start)
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
