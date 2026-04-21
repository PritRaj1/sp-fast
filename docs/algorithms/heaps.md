# Priority Queues

> Dijkstra, A*, Prim expand smallest-distance vertex via min-heap. Heap choice affects constants, not asymptotics.

| Heap    | push     | pop      | decrease_key | Notes                          |
|---------|----------|----------|--------------|--------------------------------|
| Binary  | O(log n) | O(log n) | O(log n)     | cache-friendly, default        |
| Pairing | O(1)     | O(log n) amort. | O(log n) amort. | simple, competitive in practice |

**BinaryHeap** — default. Fastest for typical sparse graphs.

**PairingHeap** — pointer-based, cheap push. Closer win when pushes dominate pops.

## Usage

```rust
use sssp_fast::{Dijkstra, PairingHeap};

let d = Dijkstra::<f64>::new();                      // BinaryHeap
let d = Dijkstra::<f64, PairingHeap<f64>>::new();    // PairingHeap
```

## Structure

### Binary Heap

Complete binary tree, parent ≤ children, stored as array.

```
        1
       / \
      3   2
     / \
    5   4

Array:    [1, 3, 2, 5, 4]
Parent:   (i-1)/2
Children: 2i+1, 2i+2
```

Push: append, bubble up. Pop: swap root + last, bubble down.

### Pairing Heap

Heap-ordered multi-way tree. Meld = link smaller root over larger. Two-pass pairing on pop.
