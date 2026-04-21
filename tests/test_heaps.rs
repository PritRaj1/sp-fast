use sssp_fast::{BinaryHeap, PairingHeap, PriorityQueue};

mod binary_heap {
    use super::*;

    #[test]
    fn test_min_heap_order() {
        let mut heap = BinaryHeap::<f64>::new();
        heap.push(5.0, 0);
        heap.push(1.0, 1);
        heap.push(3.0, 2);

        assert_eq!(heap.pop().unwrap().dist, 1.0);
        assert_eq!(heap.pop().unwrap().dist, 3.0);
        assert_eq!(heap.pop().unwrap().dist, 5.0);
        assert!(heap.pop().is_none());
    }
}

mod pairing_heap {
    use super::*;

    #[test]
    fn test_min_heap_order() {
        let mut heap = PairingHeap::<f64>::new();
        heap.push(5.0, 0);
        heap.push(1.0, 1);
        heap.push(3.0, 2);

        assert_eq!(heap.pop().unwrap().dist, 1.0);
        assert_eq!(heap.pop().unwrap().dist, 3.0);
        assert_eq!(heap.pop().unwrap().dist, 5.0);
        assert!(heap.pop().is_none());
    }

    #[test]
    fn test_duplicate_distances() {
        let mut heap = PairingHeap::<f64>::new();
        heap.push(1.0, 0);
        heap.push(1.0, 1);
        heap.push(1.0, 2);

        assert_eq!(heap.pop().unwrap().dist, 1.0);
        assert_eq!(heap.pop().unwrap().dist, 1.0);
        assert_eq!(heap.pop().unwrap().dist, 1.0);
    }

    #[test]
    fn test_clear() {
        let mut heap = PairingHeap::<f64>::new();
        heap.push(1.0, 0);
        heap.push(2.0, 1);
        heap.push(3.0, 2);

        heap.clear();
        assert!(heap.is_empty());
        assert_eq!(heap.len(), 0);
        assert!(heap.pop().is_none());
    }

    #[test]
    fn test_interleaved_push_pop() {
        let mut heap = PairingHeap::<f64>::new();

        heap.push(5.0, 0);
        heap.push(3.0, 1);
        assert_eq!(heap.pop().unwrap().dist, 3.0);

        heap.push(1.0, 2);
        heap.push(4.0, 3);
        assert_eq!(heap.pop().unwrap().dist, 1.0);
        assert_eq!(heap.pop().unwrap().dist, 4.0);
        assert_eq!(heap.pop().unwrap().dist, 5.0);
    }

    #[test]
    fn test_many_elements() {
        let mut heap = PairingHeap::<f64>::new();
        for i in (0..100).rev() {
            heap.push(i as f64, i);
        }

        for i in 0..100 {
            assert_eq!(heap.pop().unwrap().dist, i as f64);
        }
        assert!(heap.is_empty());
    }
}
