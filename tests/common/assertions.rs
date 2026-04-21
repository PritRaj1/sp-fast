use nalgebra::{DefaultAllocator, Dim, allocator::Allocator};
use sssp_fast::{ApspBuffers, FloatNumber, SsspBuffers};

pub const EPS_F64: f64 = 1e-10;
pub const EPS_F32: f32 = 1e-6;

pub fn approx_eq<T: FloatNumber>(a: T, b: T, eps: T) {
    let diff = if a > b { a - b } else { b - a };
    assert!(
        diff <= eps,
        "Values not approximately equal: {:?} vs {:?} (diff: {:?}, eps: {:?})",
        a,
        b,
        diff,
        eps
    );
}

pub fn dist_eq<T, N>(buffers: &SsspBuffers<T, N>, vertex: usize, expected: T, eps: T)
where
    T: FloatNumber,
    N: Dim,
    DefaultAllocator: Allocator<N>,
{
    approx_eq(buffers.dist[vertex], expected, eps);
}

pub fn dists_eq<T, N>(buffers: &SsspBuffers<T, N>, expected: &[(usize, T)], eps: T)
where
    T: FloatNumber,
    N: Dim,
    DefaultAllocator: Allocator<N>,
{
    for &(v, d) in expected {
        dist_eq(buffers, v, d, eps);
    }
}

pub fn unreachable<T, N>(buffers: &SsspBuffers<T, N>, vertex: usize)
where
    T: FloatNumber,
    N: Dim,
    DefaultAllocator: Allocator<N>,
{
    assert!(
        buffers.dist[vertex].is_infinite(),
        "Expected vertex {} to be unreachable, got dist {:?}",
        vertex,
        buffers.dist[vertex]
    );
}

pub fn all_unreachable<T, N>(buffers: &SsspBuffers<T, N>, vertices: &[usize])
where
    T: FloatNumber,
    N: Dim,
    DefaultAllocator: Allocator<N>,
{
    for &v in vertices {
        unreachable(buffers, v);
    }
}

pub fn path_eq<T, N>(buffers: &SsspBuffers<T, N>, target: usize, expected: &[usize])
where
    T: FloatNumber,
    N: Dim,
    DefaultAllocator: Allocator<N>,
{
    let path = buffers.path_to(target);
    assert_eq!(
        path.as_deref(),
        Some(expected),
        "Path mismatch for vertex {}",
        target
    );
}

pub fn no_path<T, N>(buffers: &SsspBuffers<T, N>, target: usize)
where
    T: FloatNumber,
    N: Dim,
    DefaultAllocator: Allocator<N>,
{
    assert!(
        buffers.path_to(target).is_none(),
        "Expected no path to vertex {}, but found one",
        target
    );
}

// APSP-specific

pub fn apsp_dist_eq<T: FloatNumber>(
    buffers: &ApspBuffers<T>,
    i: usize,
    j: usize,
    expected: T,
    eps: T,
) {
    approx_eq(buffers.get(i, j), expected, eps);
}

pub fn apsp_unreachable<T: FloatNumber>(buffers: &ApspBuffers<T>, i: usize, j: usize) {
    assert!(
        buffers.get(i, j).is_infinite(),
        "Expected ({}, {}) to be unreachable, got dist {:?}",
        i,
        j,
        buffers.get(i, j)
    );
}

pub fn apsp_path_eq<T: FloatNumber>(
    buffers: &ApspBuffers<T>,
    i: usize,
    j: usize,
    expected: &[usize],
) {
    let path = buffers.path(i, j);
    assert_eq!(
        path.as_deref(),
        Some(expected),
        "Path mismatch for ({}, {})",
        i,
        j
    );
}
