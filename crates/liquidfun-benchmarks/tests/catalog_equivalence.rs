//! Behavioral and package-isolation checks for canonical catalog benchmarks.

use liquidfun_benchmarks::{BenchmarkCaseErrorKind, representative_catalog_benchmarks};
use liquidfun_test_protocol::ScenarioVersion;

#[test]
fn representative_cases_have_fixed_canonical_identity_and_horizons() {
    // Arrange / Act
    let cases = representative_catalog_benchmarks().expect("benchmark cases should prepare");

    // Assert
    assert_eq!(
        cases
            .iter()
            .map(|case| case.slug().as_str())
            .collect::<Vec<_>>(),
        [
            "rigid-runtime-mutation",
            "joint-distance-behavior",
            "particle-system-pause-action",
            "particle-group-construction-append",
            "particle-contacts-and-coupling",
            "particle-aabb-query-controls",
            "particle-ray-callback-controls",
        ]
    );
    for case in &cases {
        assert_eq!(case.scenario_version(), ScenarioVersion::CURRENT);
        assert_eq!(
            case.resolved_sha256(),
            case.resolved().identity().content_sha256()
        );
        assert_eq!(case.settings(), case.resolved().identity().settings());
        assert!(case.warmup_runs() > 0);
        assert_eq!(
            usize::try_from(case.measured_horizon()).expect("horizon should fit usize"),
            case.resolved().checkpoints().len()
        );
    }
}

#[test]
fn semantic_mismatch_is_rejected_instead_of_becoming_a_timing_sample() {
    // Arrange
    let cases = representative_catalog_benchmarks().expect("benchmark cases should prepare");

    // Act
    let error = cases[0]
        .validate_checkpoint(cases[1].expected_checkpoint())
        .expect_err("foreign checkpoint must be rejected");

    // Assert
    assert_eq!(error.kind(), BenchmarkCaseErrorKind::CheckpointMismatch);
}

#[test]
fn one_sample_runs_only_the_declared_logical_horizon() {
    // Arrange
    let cases = representative_catalog_benchmarks().expect("benchmark cases should prepare");
    let case = &cases[0];

    // Act
    let duration = case
        .measure_iterations(1)
        .expect("one validated sample should execute");

    // Assert
    assert!(!duration.is_zero());
}

#[test]
fn criterion_closure_delegates_to_the_sealed_measured_region() {
    // Arrange
    let source = include_str!("../benches/catalog.rs");
    let measured = source
        .split("iter_custom")
        .nth(1)
        .expect("benchmark must use Criterion iter_custom");

    // Act / Assert
    assert!(measured.contains("measure_iterations"));
    assert!(!measured.contains("resolve_catalog"));
    assert!(!measured.contains("restart"));
    assert!(!measured.contains("Instant"));
}

#[test]
fn criterion_is_confined_to_the_private_non_default_package() {
    // Arrange
    let workspace = include_str!("../../../Cargo.toml");
    let package = include_str!("../Cargo.toml");
    let published = include_str!("../../liquidfun/Cargo.toml");

    // Act / Assert
    assert!(package.contains("publish = false"));
    assert!(package.contains("criterion = \"=0.8.2\""));
    assert!(workspace.contains("default-members = [\"crates/liquidfun\"]"));
    assert!(!published.contains("criterion"));
    assert!(!published.contains("liquidfun-benchmarks"));
}
