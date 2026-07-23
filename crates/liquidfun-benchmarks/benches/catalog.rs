//! Rust-only diagnostic Criterion bridge over the complete performance matrix.

use criterion::{Criterion, criterion_group, criterion_main};
use liquidfun_benchmarks::paired_benchmark_cases;

fn benchmark_catalog(c: &mut Criterion) {
    let cases = paired_benchmark_cases()
        .unwrap_or_else(|error| panic!("performance matrix preparation failed: {error}"));
    let mut group = c.benchmark_group("diagnostic-rust-performance-matrix");
    for case in &cases {
        group.bench_function(case.diagnostic_id(), |b| {
            b.iter_custom(|iterations| {
                case.measure_native_iterations(iterations)
                    .unwrap_or_else(|error| panic!("diagnostic native sample rejected: {error}"))
            });
        });
    }
    group.finish();
}

criterion_group!(catalog_benches, benchmark_catalog);
criterion_main!(catalog_benches);
