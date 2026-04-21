use super::graph::FloatNumber;
use nalgebra::{DefaultAllocator, Dim, OVector, U1, allocator::Allocator};

pub const MST_PARENT_NONE: usize = usize::MAX;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MstEdge<T: FloatNumber> {
    pub from: usize,
    pub to: usize,
    pub weight: T,
}

impl<T: FloatNumber> MstEdge<T> {
    pub fn new(from: usize, to: usize, weight: T) -> Self {
        Self { from, to, weight }
    }
}

/// Prim buffers: cheapest incoming edge weight, tree parent, membership flag.
#[derive(Clone, Debug)]
pub struct MstBuffers<T, N>
where
    T: FloatNumber,
    N: Dim,
    DefaultAllocator: Allocator<N>,
{
    pub key: OVector<T, N>,
    pub parent: OVector<usize, N>,
    pub in_mst: OVector<bool, N>,
}

impl<T, N> MstBuffers<T, N>
where
    T: FloatNumber,
    N: Dim,
    DefaultAllocator: Allocator<N>,
{
    pub fn new_inf(n: N) -> Self {
        Self {
            key: OVector::<T, N>::from_element_generic(n, U1, T::infinity()),
            parent: OVector::<usize, N>::from_element_generic(n, U1, MST_PARENT_NONE),
            in_mst: OVector::<bool, N>::from_element_generic(n, U1, false),
        }
    }

    pub fn reset_inf(&mut self) {
        self.key.fill(T::infinity());
        self.parent.fill(MST_PARENT_NONE);
        self.in_mst.fill(false);
    }

    #[inline]
    pub fn set_source(&mut self, s: usize) {
        self.key[s] = T::zero();
        self.parent[s] = MST_PARENT_NONE;
    }

    #[inline]
    pub fn parent_of(&self, v: usize) -> Option<usize> {
        let p = self.parent[v];
        (p != MST_PARENT_NONE).then_some(p)
    }

    /// MST edges (one per non-root vertex in tree).
    pub fn collect_edges(&self) -> Vec<MstEdge<T>> {
        (0..self.parent.len())
            .filter(|&v| self.in_mst[v] && self.parent[v] != MST_PARENT_NONE)
            .map(|v| MstEdge::new(self.parent[v], v, self.key[v]))
            .collect()
    }

    pub fn total_weight(&self) -> T {
        (0..self.key.len())
            .filter(|&v| self.in_mst[v] && self.parent[v] != MST_PARENT_NONE)
            .fold(T::zero(), |acc, v| acc + self.key[v])
    }

    pub fn vertices_in_mst(&self) -> usize {
        self.in_mst.iter().filter(|&&x| x).count()
    }
}
