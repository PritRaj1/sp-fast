use nalgebra::RealField;
use num_traits::Float;

/// Scalar weight. `RealField` kept for matrix interop (nalgebra OVector buffers).
pub trait FloatNumber: RealField + Float + std::fmt::Debug + Send + Sync + 'static {}

impl<T> FloatNumber for T where T: RealField + Float + std::fmt::Debug + Send + Sync + 'static {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge<T: FloatNumber> {
    pub to: usize,
    pub w: T,
}

impl<T: FloatNumber> Edge<T> {
    pub fn new(to: usize, w: T) -> Self {
        Self { to, w }
    }
}

/// Read-only graph.
pub trait Graph<T: FloatNumber> {
    fn n(&self) -> usize;
    fn for_each_out_edge<F: FnMut(usize, T)>(&self, u: usize, f: F);
}

#[derive(Clone, Debug)]
pub struct AdjListGraph<T: FloatNumber> {
    n: usize,
    adj: Vec<Vec<Edge<T>>>,
}

impl<T: FloatNumber> AdjListGraph<T> {
    pub fn new(n: usize) -> Self {
        Self {
            n,
            adj: vec![Vec::new(); n],
        }
    }

    pub fn add_edge(&mut self, u: usize, v: usize, w: T) {
        debug_assert!(u < self.n && v < self.n);
        self.adj[u].push(Edge::new(v, w));
    }

    pub fn neighbors(&self, u: usize) -> &[Edge<T>] {
        &self.adj[u]
    }

    pub fn m(&self) -> usize {
        self.adj.iter().map(|edges| edges.len()).sum()
    }
}

impl<T: FloatNumber> Graph<T> for AdjListGraph<T> {
    fn n(&self) -> usize {
        self.n
    }

    fn for_each_out_edge<F: FnMut(usize, T)>(&self, u: usize, mut f: F) {
        for e in &self.adj[u] {
            f(e.to, e.w);
        }
    }
}
