//! Behavioral and package-isolation checks for complete performance benchmarks.

use liquidfun_benchmarks::{BenchmarkCaseErrorKind, PairedEngineOrder, paired_benchmark_cases};

#[test]
fn every_case_has_exact_resolved_identity_and_semantic_authority() {
    // Arrange / Act
    let cases = paired_benchmark_cases().expect("performance cases should prepare");

    // Assert
    for case in &cases {
        assert_eq!(
            case.resolved_sha256(),
            case.resolved().identity().content_sha256()
        );
        assert_eq!(
            usize::try_from(case.logical_horizon()).expect("horizon should fit usize"),
            case.resolved().checkpoints().len()
        );
    }
}

#[test]
fn foreign_semantic_authority_never_becomes_a_timing_sample() {
    // Arrange
    let cases = paired_benchmark_cases().expect("performance cases should prepare");

    // Act / Assert
    assert_ne!(
        cases[0].expected_checkpoint(),
        cases
            .iter()
            .find(|case| case.expected_checkpoint() != cases[0].expected_checkpoint())
            .expect("matrix should contain distinct semantic cases")
            .expected_checkpoint()
    );
}

#[test]
fn one_sample_uses_alternating_caller_contract_and_sealed_native_region() {
    // Arrange
    let cases = paired_benchmark_cases().expect("performance cases should prepare");
    let case = &cases[0];

    // Act
    let _duration = case
        .measure_native_sample(0)
        .expect("one native half-sample should execute");

    // Assert
    assert_eq!(case.sample_order(0), PairedEngineOrder::NativeThenOracle);
    assert_eq!(case.sample_order(1), PairedEngineOrder::OracleThenNative);
}

#[test]
fn criterion_closure_is_diagnostic_and_delegates_to_sealed_native_measurement() {
    // Arrange
    let source = include_str!("../benches/catalog.rs");
    let measured = source
        .split("iter_custom")
        .nth(1)
        .expect("diagnostic benchmark must use Criterion iter_custom");

    // Act / Assert
    assert!(source.contains("diagnostic-rust-performance-matrix"));
    assert!(measured.contains("measure_native_iterations"));
    assert!(!measured.contains("resolve_catalog"));
    assert!(!measured.contains("Instant"));
}

#[test]
fn out_of_range_sample_is_rejected_before_execution() {
    // Arrange
    let cases = paired_benchmark_cases().expect("performance cases should prepare");

    // Act
    let error = cases[0]
        .measure_native_sample(10_000)
        .expect_err("first out-of-range sample should fail closed");

    // Assert
    assert_eq!(error.kind(), BenchmarkCaseErrorKind::ResourceLimit);
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
