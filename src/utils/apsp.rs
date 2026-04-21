use super::graph::FloatNumber;

/// Sentinel for "no next vertex on path".
pub const APSP_NO_PATH: usize = usize::MAX;

/// APSP buffers. V×V distance matrix + next-hop matrix, both row-major flattened.
#[derive(Clone, Debug)]
pub struct ApspBuffers<T: FloatNumber> {
    pub n: usize,
    pub dist: Vec<T>,
    pub next: Vec<usize>,
}

impl<T: FloatNumber> ApspBuffers<T> {
    pub fn new(n: usize) -> Self {
        let size = n * n;
        let mut dist = vec![T::infinity(); size];
        let next = vec![APSP_NO_PATH; size];
        for i in 0..n {
            dist[i * n + i] = T::zero();
        }
        Self { n, dist, next }
    }

    pub fn reset(&mut self) {
        self.dist.fill(T::infinity());
        self.next.fill(APSP_NO_PATH);
        for i in 0..self.n {
            self.dist[i * self.n + i] = T::zero();
        }
    }

    #[inline]
    pub fn get(&self, i: usize, j: usize) -> T {
        self.dist[i * self.n + j]
    }

    #[inline]
    pub fn set(&mut self, i: usize, j: usize, d: T) {
        self.dist[i * self.n + j] = d;
    }

    #[inline]
    pub fn get_next(&self, i: usize, j: usize) -> usize {
        self.next[i * self.n + j]
    }

    #[inline]
    pub fn set_next(&mut self, i: usize, j: usize, v: usize) {
        self.next[i * self.n + j] = v;
    }

    /// Path i -> j, or None if unreachable.
    pub fn path(&self, i: usize, j: usize) -> Option<Vec<usize>> {
        if self.get(i, j).is_infinite() {
            return None;
        }
        if i == j {
            return Some(vec![i]);
        }
        let mut path = vec![i];
        let mut curr = i;
        while curr != j {
            curr = self.get_next(curr, j);
            if curr == APSP_NO_PATH {
                return None;
            }
            path.push(curr);
        }
        Some(path)
    }

    /// Negative cycle iff any diagonal entry < 0.
    pub fn has_negative_cycle(&self) -> bool {
        (0..self.n).any(|i| self.get(i, i) < T::zero())
    }
}
