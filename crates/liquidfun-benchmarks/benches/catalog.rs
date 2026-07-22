//! Criterion bridge over prevalidated canonical catalog cases.

use criterion::{Criterion, criterion_group, criterion_main};
use liquidfun_benchmarks::representative_catalog_benchmarks;

fn benchmark_catalog(c: &mut Criterion) {
    let cases = representative_catalog_benchmarks()
        .unwrap_or_else(|error| panic!("catalog benchmark preparation failed: {error}"));
    let mut group = c.benchmark_group("canonical-catalog");
    for case in &cases {
        let benchmark_id = format!(
            "{}-v{}-{}-ticks-{}",
            case.slug().as_str(),
            case.scenario_version().get(),
            case.resolved_sha256().as_str(),
            case.measured_horizon()
        );
        group.bench_function(benchmark_id, |b| {
            b.iter_custom(|iterations| {
                case.measure_iterations(iterations)
                    .unwrap_or_else(|error| panic!("catalog timing sample rejected: {error}"))
            });
        });
    }
    group.finish();
}

criterion_group!(catalog_benches, benchmark_catalog);
criterion_main!(catalog_benches);
