use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use nalgebra::Dyn;
use sssp_fast::{AdjListGraph, BinaryHeap, Dijkstra, PairingHeap, SsspAlgorithm, SsspBuffers};

fn random_graph(n: usize, avg_degree: usize, seed: u64) -> AdjListGraph<f64> {
    let mut g = AdjListGraph::new(n);
    let mut rng = fastrand::Rng::with_seed(seed);
    for u in 0..n {
        for _ in 0..avg_degree {
            let v = rng.usize(0..n);
            if u != v {
                g.add_edge(u, v, 1.0 + rng.f64() * 9.0);
            }
        }
    }
    g
}

fn bench_heaps(c: &mut Criterion) {
    let mut group = c.benchmark_group("dijkstra_full_scan");
    for &n in &[100usize, 1_000, 10_000] {
        let g = random_graph(n, 5, 42);

        group.bench_with_input(BenchmarkId::new("binary", n), &g, |b, g| {
            let mut algo: Dijkstra<f64, BinaryHeap<f64>> = Dijkstra::new();
            let mut buf: SsspBuffers<f64, Dyn> = SsspBuffers::new_inf(Dyn(n));
            b.iter(|| {
                algo.run(g, 0, &mut buf);
            });
        });

        group.bench_with_input(BenchmarkId::new("pairing", n), &g, |b, g| {
            let mut algo: Dijkstra<f64, PairingHeap<f64>> = Dijkstra::new();
            let mut buf: SsspBuffers<f64, Dyn> = SsspBuffers::new_inf(Dyn(n));
            b.iter(|| {
                algo.run(g, 0, &mut buf);
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_heaps);
criterion_main!(benches);
