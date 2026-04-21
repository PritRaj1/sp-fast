use crate::utils::FloatNumber;

use super::traits::{HeapEntry, PriorityQueue};

#[derive(Debug)]
struct PairNode<T: FloatNumber> {
    entry: HeapEntry<T>,
    child: Option<Box<PairNode<T>>>,
    sibling: Option<Box<PairNode<T>>>,
}

impl<T: FloatNumber> PairNode<T> {
    fn new(dist: T, vertex: usize) -> Self {
        Self {
            entry: HeapEntry::new(dist, vertex),
            child: None,
            sibling: None,
        }
    }
}

/// Merges two heap trees, returning the root with smaller key.
fn meld<T: FloatNumber>(
    a: Option<Box<PairNode<T>>>,
    b: Option<Box<PairNode<T>>>,
) -> Option<Box<PairNode<T>>> {
    match (a, b) {
        (None, b) => b,
        (a, None) => a,
        (Some(mut a), Some(mut b)) => {
            if a.entry.dist <= b.entry.dist {
                b.sibling = a.child.take();
                a.child = Some(b);
                Some(a)
            } else {
                a.sibling = b.child.take();
                b.child = Some(a);
                Some(b)
            }
        }
    }
}

/// Two-pass pairing: pair up siblings left-to-right, then meld right-to-left.
fn merge_pairs<T: FloatNumber>(mut node: Option<Box<PairNode<T>>>) -> Option<Box<PairNode<T>>> {
    node.as_ref()?;

    let mut nodes = Vec::new();
    while let Some(mut n) = node {
        node = n.sibling.take();
        nodes.push(n);
    }

    let mut paired = Vec::with_capacity(nodes.len().div_ceil(2));
    let mut iter = nodes.into_iter();
    while let Some(first) = iter.next() {
        let second = iter.next();
        paired.push(meld(Some(first), second));
    }

    paired
        .into_iter()
        .rev()
        .fold(None, |acc, node| meld(acc, node))
}

#[derive(Debug)]
pub struct PairingHeap<T: FloatNumber> {
    root: Option<Box<PairNode<T>>>,
    len: usize,
}

impl<T: FloatNumber> PairingHeap<T> {
    pub fn new() -> Self {
        Self { root: None, len: 0 }
    }

    pub fn with_capacity(_capacity: usize) -> Self {
        // Pairing heap doesn't pre-allocate
        Self::new()
    }
}

impl<T: FloatNumber> Default for PairingHeap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: FloatNumber> PriorityQueue<T> for PairingHeap<T> {
    fn new() -> Self {
        PairingHeap::new()
    }

    fn with_capacity(capacity: usize) -> Self {
        PairingHeap::with_capacity(capacity)
    }

    #[inline]
    fn push(&mut self, dist: T, vertex: usize) {
        let node = Box::new(PairNode::new(dist, vertex));
        self.root = meld(self.root.take(), Some(node));
        self.len += 1;
    }

    fn pop(&mut self) -> Option<HeapEntry<T>> {
        let root = self.root.take()?;
        let result = root.entry;
        self.root = merge_pairs(root.child);
        self.len -= 1;
        Some(result)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    #[inline]
    fn len(&self) -> usize {
        self.len
    }

    fn clear(&mut self) {
        self.root = None;
        self.len = 0;
    }
}
