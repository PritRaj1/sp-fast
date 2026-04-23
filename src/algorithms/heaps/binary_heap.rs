use crate::utils::FloatNumber;
use ordered_float::NotNan;
use std::cmp::Reverse;
use std::collections::BinaryHeap as StdBinaryHeap;

use super::traits::{HeapEntry, PriorityQueue};

/// Min-heap via `Reverse` over std max-heap; `NotNan` rejects NaN at push.
#[derive(Debug)]
pub struct BinaryHeap<T: FloatNumber> {
    heap: StdBinaryHeap<Reverse<(NotNan<T>, usize)>>,
}

impl<T: FloatNumber> BinaryHeap<T> {
    pub fn new() -> Self {
        Self {
            heap: StdBinaryHeap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            heap: StdBinaryHeap::with_capacity(capacity),
        }
    }
}

impl<T: FloatNumber> Default for BinaryHeap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: FloatNumber> PriorityQueue<T> for BinaryHeap<T> {
    fn new() -> Self {
        BinaryHeap::new()
    }

    fn with_capacity(capacity: usize) -> Self {
        BinaryHeap::with_capacity(capacity)
    }

    #[inline]
    fn push(&mut self, dist: T, vertex: usize) {
        let d = NotNan::new(dist).expect("shortest-path distances must not be NaN");
        self.heap.push(Reverse((d, vertex)));
    }

    #[inline]
    fn pop(&mut self) -> Option<HeapEntry<T>> {
        self.heap
            .pop()
            .map(|Reverse((d, v))| HeapEntry::new(d.into_inner(), v))
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    #[inline]
    fn len(&self) -> usize {
        self.heap.len()
    }

    fn clear(&mut self) {
        self.heap.clear();
    }
}
