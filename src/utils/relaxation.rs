use super::graph::FloatNumber;

#[derive(Clone, Copy, Debug)]
pub enum RelaxResult {
    Improved,
    NoChange,
}

/// Relax edge `u -> v` with weight `w` using `dist[u]`.
#[inline]
pub fn relax<T: FloatNumber>(
    dist: &mut [T],
    parent: &mut [usize],
    u: usize,
    v: usize,
    w: T,
) -> RelaxResult {
    relax_with(dist, parent, u, dist[u], v, w)
}

/// Relax with caller-supplied `d_u` — saves one load of `dist[u]`.
#[inline]
pub fn relax_with<T: FloatNumber>(
    dist: &mut [T],
    parent: &mut [usize],
    u: usize,
    d_u: T,
    v: usize,
    w: T,
) -> RelaxResult {
    let new_dist = d_u + w;
    if new_dist < dist[v] {
        dist[v] = new_dist;
        parent[v] = u;
        RelaxResult::Improved
    } else {
        RelaxResult::NoChange
    }
}
