use nalgebra::{Const, DefaultAllocator, Dim, Dyn, allocator::Allocator};
use sssp_fast::{ApspBuffers, FloatNumber, MstBuffers, SsspBuffers};

pub fn dynamic<T: FloatNumber>(n: usize) -> SsspBuffers<T, Dyn> {
    SsspBuffers::new_inf(Dyn(n))
}

pub fn fixed<T: FloatNumber, const N: usize>() -> SsspBuffers<T, Const<N>> {
    SsspBuffers::new_inf(Const::<N>)
}

pub fn create<T, N>(dim: N) -> SsspBuffers<T, N>
where
    T: FloatNumber,
    N: Dim,
    DefaultAllocator: Allocator<N>,
{
    SsspBuffers::new_inf(dim)
}

pub fn mst_dynamic<T: FloatNumber>(n: usize) -> MstBuffers<T, Dyn> {
    MstBuffers::new_inf(Dyn(n))
}

pub fn apsp<T: FloatNumber>(n: usize) -> ApspBuffers<T> {
    ApspBuffers::new(n)
}
