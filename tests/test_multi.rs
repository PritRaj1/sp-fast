mod common;

use common::{assertions::EPS_F64 as EPS, *};
use sssp_fast::{Dijkstra, DijkstraConfig, SsspAlgorithm, dijkstra_multi};

#[test]
fn test_multi_source_nearest() {
    // Sources 0 and 4 on an undirected chain 0-1-2-3-4.
    // Each vertex gets min distance to either source.
    let g = linear_undirected(5, 1.0);
    let mut buf = dynamic(5);
    dijkstra_multi(&g, &[0, 4], &mut buf);

    dists_eq(
        &buf,
        &[(0, 0.0), (1, 1.0), (2, 2.0), (3, 1.0), (4, 0.0)],
        EPS,
    );
}

#[test]
fn test_multi_target_early_stop() {
    // Chain of 10. Targets {3, 6}: loop should exit once both are finalised
    let g = linear(10, 1.0);
    let mut buf = dynamic(10);
    let mut algo: Dijkstra<f64> = Dijkstra::with_config(DijkstraConfig::with_targets(vec![3, 6]));
    algo.run(&g, 0, &mut buf);

    dists_eq(&buf, &[(3, 3.0), (6, 6.0)], EPS);
    all_unreachable(&buf, &[7, 8, 9]);
}
