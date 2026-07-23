//! Closed Phase 12 performance policy, workload matrix, and report contract.

use std::collections::{BTreeMap, BTreeSet};

use liquidfun_test_protocol::{
    CompatibilityStatus, EvidenceTier, HardwareSession, HarnessLimits, PerformanceErrorKind,
    PerformanceMatrix, PerformancePolicy, PerformanceReportIdentity,
    PerformanceReportIdentityFields, PerformanceSizePoint, PerformanceWorkloadKind,
    ScalarOptimizationMode, Sha256Hex, TimingAuthority, decode_canonical_checkpoint_jsonl,
    render_performance_matrix, render_performance_policy_schema,
};
use serde_json::Value;

const TRACKED_POLICY_SCHEMA: &str =
    include_str!("../../../protocol/schemas/performance-policy-v1.schema.json");
const TRACKED_MATRIX: &str = include_str!("../../../protocol/benchmarks/phase12-v1.json");

#[test]
fn workload_vocabulary_is_exact_and_closed() {
    // Arrange
    let expected = [
        "world_step",
        "broad_phase",
        "narrow_phase",
        "contact_solve",
        "ccd",
        "joints",
        "particle_lifecycle",
        "particle_contacts",
        "particle_sort",
        "particle_pressure",
        "large_particle_system",
        "mixed_world",
        "aabb_query",
        "ray_cast",
    ];

    // Act
    let actual = PerformanceWorkloadKind::ALL.map(PerformanceWorkloadKind::as_str);

    // Assert
    assert_eq!(actual, expected);
    assert_eq!(PerformanceWorkloadKind::ALL.len(), 14);
}

#[test]
fn reviewed_policy_fixes_statistical_bounds_and_timing_authority() {
    // Arrange
    let policy = PerformancePolicy::reviewed_v1();

    // Act
    let threshold = policy.regression_threshold_basis_points(475);

    // Assert
    assert_eq!(policy.version().as_str(), "phase12-performance-v1");
    assert_eq!(policy.baseline_runs(), 5);
    assert_eq!(policy.confidence_percent(), 95);
    assert_eq!(policy.practical_floor_basis_points(), 300);
    assert_eq!(threshold, 475);
    assert_eq!(
        policy.timing_authority(),
        TimingAuthority::UnprofiledWallClock
    );
    assert!(policy.is_interleaved());
}

#[test]
fn policy_rejects_values_below_reviewed_minimums() {
    for (runs, confidence, floor, kind) in [
        (4, 95, 300, PerformanceErrorKind::BaselineRunsBelowMinimum),
        (5, 94, 300, PerformanceErrorKind::ConfidenceBelowMinimum),
        (5, 95, 299, PerformanceErrorKind::PracticalFloorBelowMinimum),
    ] {
        // Act
        let error = PerformancePolicy::new(runs, confidence, floor)
            .expect_err("reviewed lower bounds must be enforced");

        // Assert
        assert_eq!(error.kind(), kind);
    }
}

#[test]
fn matrix_covers_each_workload_and_every_required_size_sweep() {
    // Arrange
    let matrix = PerformanceMatrix::reviewed_v1().expect("reviewed matrix should validate");
    let scalable = [
        PerformanceWorkloadKind::BroadPhase,
        PerformanceWorkloadKind::ParticleLifecycle,
        PerformanceWorkloadKind::ParticleContacts,
        PerformanceWorkloadKind::ParticleSort,
        PerformanceWorkloadKind::ParticlePressure,
        PerformanceWorkloadKind::LargeParticleSystem,
        PerformanceWorkloadKind::MixedWorld,
        PerformanceWorkloadKind::AabbQuery,
        PerformanceWorkloadKind::RayCast,
    ];
    let expected_sizes = BTreeSet::from([
        PerformanceSizePoint::Entities128,
        PerformanceSizePoint::Entities1024,
        PerformanceSizePoint::Entities8192,
    ]);

    // Act
    let mut sizes_by_workload = BTreeMap::new();
    for case in matrix.cases() {
        sizes_by_workload
            .entry(case.workload())
            .or_insert_with(BTreeSet::new)
            .insert(case.size_point());
    }

    // Assert
    assert_eq!(sizes_by_workload.len(), 14);
    for workload in scalable {
        assert_eq!(sizes_by_workload.get(&workload), Some(&expected_sizes));
    }
    assert!(matrix.cases().iter().all(|case| {
        !case.catalog_sha256().as_str().is_empty()
            && !case.resolved_sha256().as_str().is_empty()
            && case.logical_horizon() > 0
            && case.optimization_mode() == ScalarOptimizationMode::ReleaseScalar
            && case.regions().is_complete()
    }));
}

#[test]
fn matrix_rejects_missing_workloads_and_duplicate_case_identities() {
    // Arrange
    let matrix = PerformanceMatrix::reviewed_v1().expect("reviewed matrix should validate");
    let mut missing = matrix.cases().to_vec();
    missing.retain(|case| case.workload() != PerformanceWorkloadKind::RayCast);
    let mut duplicate = matrix.cases().to_vec();
    duplicate.push(
        duplicate
            .first()
            .expect("reviewed matrix is non-empty")
            .clone(),
    );

    // Act
    let missing_error =
        PerformanceMatrix::new(missing).expect_err("missing workload must be rejected");
    let duplicate_error =
        PerformanceMatrix::new(duplicate).expect_err("duplicate identity must be rejected");

    // Assert
    assert_eq!(
        missing_error.kind(),
        PerformanceErrorKind::IncompleteWorkloadMatrix
    );
    assert_eq!(
        duplicate_error.kind(),
        PerformanceErrorKind::DuplicateCaseIdentity
    );
}

#[test]
fn d1_fixture_promotion_is_not_a_performance_compatibility_status() {
    // Act
    let error = CompatibilityStatus::try_from(EvidenceTier::D1Canonical)
        .expect_err("timing data cannot promote canonical physics fixtures");

    // Assert
    assert_eq!(
        error.kind(),
        PerformanceErrorKind::FixturePromotionForbidden
    );
}

#[test]
fn report_identity_binds_all_reproduction_inputs() {
    // Arrange
    let hardware = HardwareSession::new(
        "session-2026-07-23",
        "review-cpu",
        8,
        16 * 1024 * 1024 * 1024,
        "review-os",
    )
    .expect("hardware fixture should validate");
    let hash = || Sha256Hex::new("1".repeat(64)).expect("fixture hash should validate");
    let fields = PerformanceReportIdentityFields::new(
        "world-step-fixed",
        "rust-revision",
        "oracle-revision",
        "rustc 1.97.0",
        "rust-lld",
        "clang 22.1.8",
        "lld 22.1.8",
        "x86_64-unknown-linux-gnu",
        "-C target-cpu=x86-64",
        "-C linker=rust-lld",
        "-O3 -fno-fast-math",
        "-fuse-ld=lld",
        hardware,
        hash(),
        hash(),
        hash(),
        hash(),
    );

    // Act
    let identity =
        PerformanceReportIdentity::new(fields).expect("complete report identity should validate");
    let mut value = serde_json::to_value(identity).expect("identity should serialize");

    // Assert
    for field in [
        "scenario_id",
        "rust_revision",
        "oracle_revision",
        "rust_compiler",
        "rust_linker",
        "oracle_compiler",
        "oracle_linker",
        "target",
        "rust_compile_flags",
        "rust_link_flags",
        "oracle_compile_flags",
        "oracle_link_flags",
        "hardware_session",
        "policy_sha256",
        "matrix_sha256",
        "catalog_sha256",
        "resolved_sha256",
    ] {
        assert!(value.get(field).is_some(), "missing identity field {field}");
    }
    value["identity_sha256"] = Value::from("0".repeat(64));
    assert!(
        serde_json::from_value::<PerformanceReportIdentity>(value).is_err(),
        "a spoofed report identity hash must be rejected"
    );
}

#[test]
fn policy_decoder_rejects_unknown_fields() {
    // Arrange
    let mut value =
        serde_json::to_value(PerformancePolicy::reviewed_v1()).expect("policy should serialize");
    value
        .as_object_mut()
        .expect("policy is an object")
        .insert("duration".to_owned(), Value::from(1));

    // Act
    let error = serde_json::from_value::<PerformancePolicy>(value)
        .expect_err("unknown timing fields must be rejected");

    // Assert
    assert!(error.to_string().contains("unknown field"));
}

#[test]
fn semantic_checkpoint_decoder_rejects_duration_fields() {
    // Arrange
    let mut checkpoint = serde_json::json!({
        "protocol_version": 1,
        "record_kind": "canonical_checkpoint",
        "checkpoint_schema_version": 1,
        "request_id": "performance-checkpoint",
        "resolved_sha256": "1".repeat(64),
        "checkpoint_id": "checkpoint-0001",
        "position": {
            "kind": "logical_step",
            "ordinal": 1
        },
        "simulation_time_bits": 0,
        "observations": [],
        "numeric_observations": [],
        "ordered_occurrences": [],
        "unordered_sets": [],
        "debug_primitives": [],
        "profile_names": []
    });
    checkpoint["duration"] = Value::from(1);
    let mut bytes = serde_json::to_vec(&checkpoint).expect("fixture should encode");
    bytes.push(b'\n');

    // Act
    let result = decode_canonical_checkpoint_jsonl(&bytes, &HarnessLimits::phase2_default_v1());

    // Assert
    assert!(
        result.is_err(),
        "semantic checkpoints must reject duration fields"
    );
}

#[test]
fn generated_schema_and_matrix_match_tracked_bytes() {
    // Act
    let schema = render_performance_policy_schema();
    let matrix = render_performance_matrix().expect("reviewed matrix should render");

    // Assert
    assert_eq!(schema, TRACKED_POLICY_SCHEMA);
    assert_eq!(matrix, TRACKED_MATRIX);
}
