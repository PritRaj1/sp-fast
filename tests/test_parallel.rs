mod common;

use common::{assertions::EPS_F64 as EPS, *};
use sssp_fast::{BinaryHeap, Dijkstra, Dyn, parallel_sssp};

#[test]
fn test_multi_source_distances() {
    let g = linear_undirected(5, 1.0);
    let sources = vec![0, 2, 4];

    let result = parallel_sssp::<f64, Dyn, _, _, _>(&g, &sources, || {
        Dijkstra::<f64, BinaryHeap<f64>>::new()
    });

    assert!((result.dist(0, 4).unwrap() - 4.0).abs() < EPS); // 0 -> 4
    assert!((result.dist(1, 0).unwrap() - 2.0).abs() < EPS); // 2 -> 0
    assert!((result.dist(2, 0).unwrap() - 4.0).abs() < EPS); // 4 -> 0
    assert_eq!(result.path(2, 0).unwrap(), vec![4, 3, 2, 1, 0]);
}

#[test]
fn test_multi_source_unreachable() {
    let g = disconnected(6, 1.0);
    let sources = vec![0, 3];

    let result = parallel_sssp::<f64, Dyn, _, _, _>(&g, &sources, || {
        Dijkstra::<f64, BinaryHeap<f64>>::new()
    });

    assert!((result.dist(0, 2).unwrap() - 2.0).abs() < EPS); // 0 can reach 2
    assert!(result.dist(0, 3).unwrap().is_infinite()); // 0 cannot reach 3
    assert!(result.dist(1, 0).unwrap().is_infinite()); // 3 cannot reach 0
}

#[test]
fn test_all_pairs() {
    let g = complete(50, 1.0);
    let sources: Vec<usize> = (0..50).collect();

    let result = parallel_sssp::<f64, Dyn, _, _, _>(&g, &sources, || {
        Dijkstra::<f64, BinaryHeap<f64>>::new()
    });

    for i in 0..50 {
        for j in 0..50 {
            let expected = if i == j { 0.0 } else { 1.0 }; // All dist 1.0
            assert!((result.dist(i, j).unwrap() - expected).abs() < EPS);
        }
    }
}
