use nalgebra::RealField;
use num_traits::float::FloatCore;

/// Sentinel for "no valid vertex".
pub const NO_VERTEX: usize = usize::MAX;

/// Scalar weight.
pub trait FloatNumber: RealField + FloatCore + std::fmt::Debug + Send + Sync + 'static {}

impl<T> FloatNumber for T where T: RealField + FloatCore + std::fmt::Debug + Send + Sync + 'static {}

/// Directed weighted edge. `meta` carries per-edge payload; `()` by default.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge<T: FloatNumber, M = ()> {
    pub to: usize,
    pub w: T,
    pub meta: M,
}

impl<T: FloatNumber> Edge<T, ()> {
    pub fn new(to: usize, w: T) -> Self {
        Self { to, w, meta: () }
    }
}

impl<T: FloatNumber, M> Edge<T, M> {
    pub fn with_meta(to: usize, w: T, meta: M) -> Self {
        Self { to, w, meta }
    }
}

/// Read-only graph. Out-edges expose weight `T` and payload `Self::Meta`.
pub trait Graph<T: FloatNumber> {
    type Meta;
    fn n(&self) -> usize;
    fn for_each_out_edge<F: FnMut(usize, T, &Self::Meta)>(&self, u: usize, f: F);
}

#[derive(Clone, Debug)]
pub struct AdjListGraph<T: FloatNumber, M = ()> {
    n: usize,
    adj: Vec<Vec<Edge<T, M>>>,
}

impl<T: FloatNumber, M> AdjListGraph<T, M> {
    pub fn new(n: usize) -> Self {
        Self {
            n,
            adj: (0..n).map(|_| Vec::new()).collect(),
        }
    }

    pub fn add_edge_with(&mut self, u: usize, v: usize, w: T, meta: M) {
        debug_assert!(u < self.n && v < self.n);
        self.adj[u].push(Edge { to: v, w, meta });
    }

    pub fn neighbors(&self, u: usize) -> &[Edge<T, M>] {
        &self.adj[u]
    }

    pub fn m(&self) -> usize {
        self.adj.iter().map(|edges| edges.len()).sum()
    }
}

/// Shortcut for scalar-only graphs, since no meta to supply.
impl<T: FloatNumber> AdjListGraph<T, ()> {
    pub fn add_edge(&mut self, u: usize, v: usize, w: T) {
        self.add_edge_with(u, v, w, ());
    }
}

impl<T: FloatNumber, M> Graph<T> for AdjListGraph<T, M> {
    type Meta = M;

    fn n(&self) -> usize {
        self.n
    }

    fn for_each_out_edge<F: FnMut(usize, T, &M)>(&self, u: usize, mut f: F) {
        for e in &self.adj[u] {
            f(e.to, e.w, &e.meta);
        }
    }
}
