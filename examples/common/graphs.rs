use sssp_fast::Graph;

#[derive(Clone)]
pub struct EuclideanGraph {
    pub positions: Vec<(f64, f64)>,
    pub adjacency: Vec<Vec<(usize, f64)>>, // (neighbor, weight)
}

impl Graph<f64> for EuclideanGraph {
    type Meta = ();
    fn n(&self) -> usize {
        self.positions.len()
    }
    fn for_each_out_edge<F: FnMut(usize, f64, &())>(&self, u: usize, mut f: F) {
        for &(v, w) in &self.adjacency[u] {
            f(v, w, &());
        }
    }
}

impl EuclideanGraph {
    pub fn new(n: usize) -> Self {
        Self {
            positions: Vec::with_capacity(n),
            adjacency: vec![Vec::new(); n],
        }
    }

    pub fn n_vertices(&self) -> usize {
        self.positions.len()
    }

    pub fn add_edge(&mut self, u: usize, v: usize, weight: f64) {
        self.adjacency[u].push((v, weight));
        self.adjacency[v].push((u, weight));
    }

    pub fn neighbors(&self, u: usize) -> &[(usize, f64)] {
        &self.adjacency[u]
    }

    /// Bounding box: (min_x, min_y, max_x, max_y).
    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for &(x, y) in &self.positions {
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }

        (min_x, min_y, max_x, max_y)
    }
}

/// Random graph with vertices connected by proximity.
pub fn random_euclidean_graph(
    n_vertices: usize,
    connection_radius: f64,
    seed: u64,
) -> EuclideanGraph {
    let mut rng = fastrand::Rng::with_seed(seed);
    let mut graph = EuclideanGraph::new(n_vertices);

    for _ in 0..n_vertices {
        graph.positions.push((rng.f64(), rng.f64()));
    }

    // Connect within radius
    for i in 0..n_vertices {
        for j in (i + 1)..n_vertices {
            let (x1, y1) = graph.positions[i];
            let (x2, y2) = graph.positions[j];
            let dist = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();

            if dist <= connection_radius {
                graph.add_edge(i, j, dist);
            }
        }
    }

    graph
}

/// Connected Euclidean graph by k-nearest + proximity.
pub fn random_euclidean_graph_connected(
    n_vertices: usize,
    k_nearest: usize,
    extra_radius: f64,
    seed: u64,
) -> EuclideanGraph {
    let mut rng = fastrand::Rng::with_seed(seed);
    let mut graph = EuclideanGraph::new(n_vertices);

    for _ in 0..n_vertices {
        graph.positions.push((rng.f64(), rng.f64()));
    }

    // Pairwise euclidean metric
    let mut edges_added = vec![vec![false; n_vertices]; n_vertices];

    // Cross-symmetric writes to `edges_added` prevent iter_mut().enumerate().
    #[allow(clippy::needless_range_loop)]
    for i in 0..n_vertices {
        let mut distances: Vec<(usize, f64)> = (0..n_vertices)
            .filter(|&j| j != i)
            .map(|j| {
                let (x1, y1) = graph.positions[i];
                let (x2, y2) = graph.positions[j];
                let dist = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
                (j, dist)
            })
            .collect();

        distances.sort_by(|a, b| a.1.total_cmp(&b.1));

        for &(j, dist) in distances.iter().take(k_nearest) {
            if !edges_added[i][j] {
                graph.add_edge(i, j, dist);
                edges_added[i][j] = true;
                edges_added[j][i] = true;
            }
        }
    }

    // Add extra edges within radius (for more cobwebbiness).
    #[allow(clippy::needless_range_loop)]
    for i in 0..n_vertices {
        for j in (i + 1)..n_vertices {
            if !edges_added[i][j] {
                let (x1, y1) = graph.positions[i];
                let (x2, y2) = graph.positions[j];
                let dist = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();

                if dist <= extra_radius {
                    graph.add_edge(i, j, dist);
                    edges_added[i][j] = true;
                    edges_added[j][i] = true;
                }
            }
        }
    }

    graph
}

/// Default 500-vertex graph.
pub fn euclidean_500() -> (EuclideanGraph, usize, usize) {
    let graph = random_euclidean_graph_connected(500, 6, 0.08, 42);
    let start = find_vertex_near(&graph, 0.1, 0.9);
    let end = find_vertex_near(&graph, 0.9, 0.1);
    (graph, start, end)
}

fn find_vertex_near(graph: &EuclideanGraph, target_x: f64, target_y: f64) -> usize {
    graph
        .positions
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            let dist_a = (a.0 - target_x).powi(2) + (a.1 - target_y).powi(2);
            let dist_b = (b.0 - target_x).powi(2) + (b.1 - target_y).powi(2);
            dist_a.total_cmp(&dist_b)
        })
        .map(|(i, _)| i)
        .unwrap_or(0)
}
