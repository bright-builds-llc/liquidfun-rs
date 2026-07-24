//! Focused orchestration and decision-boundary tests for Phase 12 performance evidence.

#![allow(
    dead_code,
    reason = "the production module is injected as the test subject"
)]

#[path = "../src/performance.rs"]
mod performance;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

use liquidfun_test_protocol::performance::{
    PerformanceMatrix, PerformanceSizePoint, PerformanceWorkloadKind,
};
use performance::analysis::{
    BottleneckKind, CorrectnessHashes, DisallowedBuildMode, OptimizationBuild,
    OptimizationCandidate, OptimizationDecision, WorkloadInterval, evaluate_optimization,
};
use performance::{
    PairedCaseRequest, PairedRunFailure, PairedRunProvider, PerformanceEnvironment,
    run_with_provider,
};
use serde_json::{Value, json};

static TEST_ORDINAL: AtomicU64 = AtomicU64::new(1);

struct FakeProvider {
    calls: Vec<(PerformanceWorkloadKind, PerformanceSizePoint)>,
    maybe_failure_call: Option<usize>,
}

impl FakeProvider {
    fn successful() -> Self {
        Self {
            calls: Vec::new(),
            maybe_failure_call: None,
        }
    }
}

impl PairedRunProvider for FakeProvider {
    fn run_case(&mut self, request: PairedCaseRequest<'_>) -> Result<Value, PairedRunFailure> {
        self.calls.push((request.workload(), request.size_point()));
        let call = self.calls.len();
        if self.maybe_failure_call == Some(call) {
            return Err(PairedRunFailure::Harness);
        }
        Ok(json!({
            "identity": {
                "scenario_id": request.scenario_id(),
                "resolved_sha256": request.resolved_sha256().as_str()
            },
            "compatibility_status": "d2_supported",
            "policy": serde_json::to_value(request.policy()).expect("policy serializes"),
            "profile_schema": "phase12_v1",
            "raw_samples": [{
                "baseline_run": 1,
                "sample_ordinal": 1,
                "native_nanoseconds": call as u64,
                "oracle_nanoseconds": (call as u64) + 100
            }]
        }))
    }
}

#[test]
fn paired_invokes_provider_once_per_sealed_case_and_preserves_raw_values() {
    // Arrange
    let environment = test_environment("paired-success");
    let mut provider = FakeProvider::successful();
    let args = vec!["paired".to_owned()];

    // Act
    let result = run_with_provider(&args, &environment, &mut provider);

    // Assert
    assert_eq!(result, Ok(()));
    let matrix = PerformanceMatrix::reviewed_v1().expect("reviewed matrix");
    let expected = matrix
        .cases()
        .iter()
        .map(|case| (case.workload(), case.size_point()))
        .collect::<Vec<_>>();
    assert_eq!(provider.calls, expected);
    let raw_files = fs::read_dir(environment.output_root().join("raw"))
        .expect("raw directory")
        .collect::<Result<Vec<_>, _>>()
        .expect("raw entries");
    assert_eq!(raw_files.len(), expected.len());
    let first: Value = serde_json::from_slice(
        &fs::read(environment.output_root().join("raw/world_step-fixed.json"))
            .expect("first raw report"),
    )
    .expect("valid JSON");
    assert_eq!(first["raw_samples"][0]["native_nanoseconds"], 1);
    assert_eq!(first["raw_samples"][0]["oracle_nanoseconds"], 101);
}

#[test]
fn paired_rejects_harness_failure_without_writing_completion_identity() {
    // Arrange
    let environment = test_environment("paired-failure");
    let mut provider = FakeProvider {
        calls: Vec::new(),
        maybe_failure_call: Some(2),
    };
    let args = vec!["paired".to_owned()];

    // Act
    let result = run_with_provider(&args, &environment, &mut provider);

    // Assert
    assert_eq!(
        result.expect_err("failure expected").kind(),
        "paired_harness"
    );
    assert!(
        !environment
            .output_root()
            .join("paired-summary.json")
            .exists()
    );
}

#[test]
fn closed_cli_rejects_unknown_modes_and_paths() {
    // Arrange
    let environment = test_environment("closed-cli");
    let mut provider = FakeProvider::successful();
    let attempts = [
        vec!["measure".to_owned()],
        vec!["paired".to_owned(), "/tmp/report.json".to_owned()],
        vec!["validate".to_owned(), "../report.json".to_owned()],
    ];

    // Act
    let results = attempts.map(|args| run_with_provider(&args, &environment, &mut provider));

    // Assert
    assert!(results.into_iter().all(|result| result.is_err()));
    assert!(provider.calls.is_empty());
}

#[test]
fn paired_check_validates_sealed_inputs_without_running_measurements() {
    // Arrange
    let environment = PerformanceEnvironment::production()
        .expect("workspace and reviewed release oracle are available");
    let mut provider = FakeProvider::successful();
    let args = vec!["paired".to_owned(), "--check".to_owned()];

    // Act
    let result = run_with_provider(&args, &environment, &mut provider);

    // Assert
    assert_eq!(result, Ok(()));
    assert!(provider.calls.is_empty());
}

#[test]
fn optimization_boundaries_and_required_gates_are_fail_closed() {
    // Arrange
    let base = candidate(1_000, 301, 300);
    let profile_below = candidate(999, 301, 300);
    let profile_exact = candidate(1_000, 301, 300);
    let floor_below = candidate(1_000, 299, 300);
    let noise_dominates = candidate(1_000, 451, 450);
    let interval_crosses = candidate(1_000, 300, 300);
    let regressed = candidate_with_regression();
    let mixed_commits = OptimizationCandidate {
        after_commit: base.before_commit.clone(),
        ..base.clone()
    };
    let missing_gate = OptimizationCandidate {
        correctness: CorrectnessHashes {
            safety: None,
            ..base.correctness.clone()
        },
        ..base.clone()
    };
    let non_scalar = OptimizationCandidate {
        build: OptimizationBuild {
            scalar_release: false,
            ..base.build.clone()
        },
        ..base.clone()
    };
    let profiled_totals = OptimizationCandidate {
        build: OptimizationBuild {
            scalar_release: true,
            disallowed_modes: BTreeSet::from([DisallowedBuildMode::ProfiledTotals]),
        },
        ..base.clone()
    };

    // Act / Assert
    assert_eq!(
        evaluate_optimization(&profile_below),
        OptimizationDecision::RejectProfileOrBottleneck
    );
    assert_eq!(
        evaluate_optimization(&profile_exact),
        OptimizationDecision::Admit
    );
    assert_eq!(
        evaluate_optimization(&floor_below),
        OptimizationDecision::RejectImprovementInterval
    );
    assert_eq!(
        evaluate_optimization(&noise_dominates),
        OptimizationDecision::Admit
    );
    assert_eq!(
        evaluate_optimization(&interval_crosses),
        OptimizationDecision::RejectImprovementInterval
    );
    assert_eq!(
        evaluate_optimization(&regressed),
        OptimizationDecision::RejectWorkloadRegression
    );
    assert_eq!(
        evaluate_optimization(&mixed_commits),
        OptimizationDecision::RejectCommitIdentity
    );
    assert_eq!(
        evaluate_optimization(&missing_gate),
        OptimizationDecision::RejectCorrectnessGate
    );
    assert_eq!(
        evaluate_optimization(&non_scalar),
        OptimizationDecision::RejectOptimizationMode
    );
    assert_eq!(
        evaluate_optimization(&profiled_totals),
        OptimizationDecision::RejectOptimizationMode
    );
}

fn candidate(
    profile_basis_points: u16,
    improvement_lower_basis_points: i32,
    noise_floor_basis_points: u16,
) -> OptimizationCandidate {
    let matrix = PerformanceMatrix::reviewed_v1().expect("reviewed matrix");
    let workloads = matrix
        .cases()
        .iter()
        .map(|case| {
            (
                format!(
                    "{}-{}",
                    case.workload().as_str(),
                    size_point_id(case.size_point())
                ),
                WorkloadInterval {
                    lower_basis_points: improvement_lower_basis_points,
                    estimate_basis_points: improvement_lower_basis_points + 10,
                    upper_basis_points: improvement_lower_basis_points + 20,
                    noise_floor_basis_points,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    OptimizationCandidate {
        before_commit: "1111111111111111111111111111111111111111".to_owned(),
        after_commit: "2222222222222222222222222222222222222222".to_owned(),
        build: OptimizationBuild {
            scalar_release: true,
            disallowed_modes: BTreeSet::new(),
        },
        profile_basis_points,
        maybe_bottleneck: None,
        workloads,
        correctness: CorrectnessHashes {
            differential: Some("a".repeat(64)),
            determinism: Some("b".repeat(64)),
            safety: Some("c".repeat(64)),
            public_api: Some("d".repeat(64)),
        },
    }
}

fn candidate_with_regression() -> OptimizationCandidate {
    let mut value = candidate(1_000, 301, 300);
    value.workloads.insert(
        "world_step-fixed".to_owned(),
        WorkloadInterval {
            lower_basis_points: -401,
            estimate_basis_points: -350,
            upper_basis_points: -301,
            noise_floor_basis_points: 300,
        },
    );
    value.maybe_bottleneck = Some(BottleneckKind::Scaling);
    value
}

fn size_point_id(size: PerformanceSizePoint) -> &'static str {
    match size {
        PerformanceSizePoint::Fixed => "fixed",
        PerformanceSizePoint::Entities128 => "128",
        PerformanceSizePoint::Entities1024 => "1024",
        PerformanceSizePoint::Entities8192 => "8192",
    }
}

fn test_environment(label: &str) -> PerformanceEnvironment {
    let ordinal = TEST_ORDINAL.fetch_add(1, Ordering::Relaxed);
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root");
    let output = root
        .join("target/phase12-performance/tests")
        .join(format!("{label}-{}-{ordinal}", std::process::id()));
    if output.exists() {
        fs::remove_dir_all(&output).expect("remove stale test output");
    }
    let environment =
        PerformanceEnvironment::for_test(&root, &output).expect("confined test environment");
    let mut workloads = BTreeSet::new();
    for workload in PerformanceWorkloadKind::ALL {
        assert!(workloads.insert(workload));
    }
    environment
}
