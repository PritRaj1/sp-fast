use super::graph::FloatNumber;
use nalgebra::{allocator::Allocator, DefaultAllocator, Dim, OVector, U1};

/// Sentinel: root or unreachable.
pub const PARENT_NONE: usize = usize::MAX;

/// SSSP buffers: distances + tree parents. Reuse via `reset_inf`.
#[derive(Clone, Debug)]
pub struct SsspBuffers<T, N>
where
    T: FloatNumber,
    N: Dim,
    DefaultAllocator: Allocator<N>,
{
    pub dist: OVector<T, N>,
    pub parent: OVector<usize, N>,
}

impl<T, N> SsspBuffers<T, N>
where
    T: FloatNumber,
    N: Dim,
    DefaultAllocator: Allocator<N>,
{
    pub fn new_inf(n: N) -> Self {
        Self {
            dist: OVector::<T, N>::from_element_generic(n, U1, T::infinity()),
            parent: OVector::<usize, N>::from_element_generic(n, U1, PARENT_NONE),
        }
    }

    pub fn reset_inf(&mut self) {
        self.dist.fill(T::infinity());
        self.parent.fill(PARENT_NONE);
    }

    #[inline]
    pub fn set_source(&mut self, s: usize) {
        self.dist[s] = T::zero();
        self.parent[s] = PARENT_NONE;
    }

    #[inline]
    pub fn parent_of(&self, v: usize) -> Option<usize> {
        let p = self.parent[v];
        (p != PARENT_NONE).then_some(p)
    }

    /// Path source -> v, or None if unreachable.
    pub fn path_to(&self, v: usize) -> Option<Vec<usize>> {
        if self.dist[v].is_infinite() {
            return None;
        }
        let mut path = Vec::new();
        let mut curr = v;
        while curr != PARENT_NONE {
            path.push(curr);
            curr = self.parent[curr];
        }
        path.reverse();
        Some(path)
    }
}
