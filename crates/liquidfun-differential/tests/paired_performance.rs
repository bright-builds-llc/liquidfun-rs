//! Paired same-host execution and immutable raw-report contracts.

use liquidfun::{DiagnosticProfileChild, DiagnosticProfileParent, DiagnosticProfileSchema};
use liquidfun_differential::{
    BenchmarkAdapterOutput, NativeBenchmarkAdapter, OracleBenchmarkAdapter, PairedBenchmarkAdapter,
    PairedBenchmarkOutcome, PairedBenchmarkPlan, PairedEngineOrder, PreparedNativeBenchmark,
    RustChildProfileDiagnostic, run_paired_benchmark,
};
use liquidfun_test_protocol::{
    CatalogSlug, CheckpointId, FloatBits, RequestId, ResolveRequest, RunSettings, Sha256Hex,
    performance::{
        BenchmarkCommonParentDiagnostic, BenchmarkCommonParentPhase, BenchmarkHarnessFailure,
        BenchmarkHarnessFailureKind, BenchmarkPerformanceResult, BenchmarkPhysicsMismatch,
        BenchmarkRunOutcome, BenchmarkRunRequest, BenchmarkRunResult, CompatibilityStatus,
        HardwareSession, PerformanceEngineRole, PerformanceReportIdentity,
        PerformanceReportIdentityFields, PerformanceSizePoint, PerformanceWorkloadKind,
        ScalarOptimizationMode, SemanticCheckpointIdentity, benchmark_policy_sha256,
    },
    resolve_catalog,
    scenarios::scenario_definitions,
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const RESOLVED_BYTES: &[u8] = br#"{"scenario":"paired-performance","version":1}"#;
const ORACLE_REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
static TEST_DIRECTORY_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy)]
enum InjectedOutcome {
    Performance,
    Harness(BenchmarkHarnessFailureKind),
    Mismatch,
}

struct RecordingAdapter {
    role: PerformanceEngineRole,
    calls: Vec<(u8, u16)>,
    maybe_injected: Option<(usize, InjectedOutcome)>,
    wrong_reset_at: Option<usize>,
    checkpoint_salt: u8,
    include_profiles: bool,
    cycle_after: Option<usize>,
}

impl RecordingAdapter {
    const fn new(role: PerformanceEngineRole) -> Self {
        Self {
            role,
            calls: Vec::new(),
            maybe_injected: None,
            wrong_reset_at: None,
            checkpoint_salt: 0,
            include_profiles: false,
            cycle_after: None,
        }
    }

    const fn with_injected(mut self, call: usize, outcome: InjectedOutcome) -> Self {
        self.maybe_injected = Some((call, outcome));
        self
    }

    const fn with_profiles(mut self) -> Self {
        self.include_profiles = true;
        self
    }

    const fn with_cycle_after(mut self, calls: usize) -> Self {
        self.cycle_after = Some(calls);
        self
    }
}

impl PairedBenchmarkAdapter for RecordingAdapter {
    fn engine_role(&self) -> PerformanceEngineRole {
        self.role
    }

    fn execute(
        &mut self,
        request: &BenchmarkRunRequest,
        baseline_run: u8,
    ) -> Result<BenchmarkAdapterOutput, BenchmarkHarnessFailureKind> {
        let call = self.calls.len() + 1;
        self.calls
            .push((baseline_run, request.identity().sample_ordinal()));
        let injected = self
            .maybe_injected
            .filter(|(injected_call, _outcome)| *injected_call == call)
            .map_or(InjectedOutcome::Performance, |(_call, outcome)| outcome);
        let checkpoint = checkpoint_identity(request, self.checkpoint_salt);
        let outcome = match injected {
            InjectedOutcome::Performance => {
                let maybe_parents = self.include_profiles.then(|| {
                    vec![
                        BenchmarkCommonParentDiagnostic::new(
                            BenchmarkCommonParentPhase::ContactSolve,
                            71,
                        )
                        .expect("parent diagnostic should validate"),
                    ]
                });
                BenchmarkRunOutcome::Performance(
                    BenchmarkPerformanceResult::new(
                        10_000 + u64::try_from(call).expect("call count should fit"),
                        maybe_parents,
                        checkpoint,
                    )
                    .expect("performance result should validate"),
                )
            }
            InjectedOutcome::Harness(kind) => {
                BenchmarkRunOutcome::HarnessFailure(BenchmarkHarnessFailure::new(kind))
            }
            InjectedOutcome::Mismatch => {
                BenchmarkRunOutcome::PhysicsMismatch(BenchmarkPhysicsMismatch::new(checkpoint))
            }
        };
        let (process_generation, reset_epoch) = if let Some(cycle_after) = self.cycle_after {
            if call > cycle_after {
                (
                    2,
                    u64::try_from(call - cycle_after).expect("cycled call count should fit"),
                )
            } else {
                (1, u64::try_from(call).expect("call count should fit"))
            }
        } else {
            (1, u64::try_from(call).expect("call count should fit"))
        };
        let reset_epoch = if self.wrong_reset_at == Some(call) {
            999
        } else {
            reset_epoch
        };
        let result =
            BenchmarkRunResult::new(request.identity().clone(), self.role, reset_epoch, outcome)
                .expect("fake result should validate");
        let rust_children =
            if self.include_profiles && self.role == PerformanceEngineRole::NativeRust {
                vec![
                    RustChildProfileDiagnostic::new(DiagnosticProfileChild::RustIslandSolve, 37)
                        .expect("Rust child diagnostic should validate"),
                ]
            } else {
                Vec::new()
            };
        BenchmarkAdapterOutput::new_with_process_generation(
            result,
            rust_children,
            process_generation,
        )
    }
}

fn hash(bytes: &[u8]) -> Sha256Hex {
    Sha256Hex::from_digest(Sha256::digest(bytes).into())
}

fn checkpoint_identity(request: &BenchmarkRunRequest, salt: u8) -> SemanticCheckpointIdentity {
    let checkpoint_id =
        CheckpointId::new("checkpoint-0001").expect("checkpoint ID should validate");
    let mut checkpoint_bytes = b"paired-checkpoint".to_vec();
    checkpoint_bytes.push(salt);
    SemanticCheckpointIdentity::new(
        request.identity().request_id().clone(),
        request.identity().resolved_sha256().clone(),
        checkpoint_id,
        hash(&checkpoint_bytes),
    )
}

fn benchmark_request(sample_ordinal: u16) -> BenchmarkRunRequest {
    let identity = liquidfun_test_protocol::performance::BenchmarkRunIdentity::new(
        RequestId::new(format!("supervised-benchmark-{sample_ordinal}"))
            .expect("request ID should validate"),
        hash(RESOLVED_BYTES),
        RunSettings::new(FloatBits::new(0x3c88_8889), 8, 3, 1).expect("settings should validate"),
        PerformanceWorkloadKind::WorldStep,
        PerformanceSizePoint::Fixed,
        ScalarOptimizationMode::ReleaseScalar,
        1,
        1,
        sample_ordinal,
        benchmark_policy_sha256().expect("policy should hash"),
        false,
    )
    .expect("benchmark identity should validate");
    BenchmarkRunRequest::new(identity, RESOLVED_BYTES.to_vec())
        .expect("benchmark request should validate")
}

fn native_benchmark_request() -> (PreparedNativeBenchmark, BenchmarkRunRequest) {
    let settings = RunSettings::new(FloatBits::from_f32(1.0 / 60.0), 8, 3, 8)
        .expect("reviewed native settings should validate");
    let resolved = resolve_catalog(
        &scenario_definitions().expect("catalog definitions should validate"),
        &ResolveRequest::new(
            CatalogSlug::new("rigid-runtime-mutation").expect("slug should validate"),
            None,
            settings,
        ),
    )
    .expect("native performance scenario should resolve");
    let logical_horizon =
        u32::try_from(resolved.checkpoints().len()).expect("checkpoint horizon should fit");
    let prepared = PreparedNativeBenchmark::new(
        resolved.clone(),
        resolved.identity().content_sha256(),
        logical_horizon,
        1,
    )
    .expect("native benchmark should prepare");
    let identity = liquidfun_test_protocol::performance::BenchmarkRunIdentity::new(
        RequestId::new("native-paired-identity").expect("request ID should validate"),
        resolved.identity().content_sha256().clone(),
        settings,
        PerformanceWorkloadKind::WorldStep,
        PerformanceSizePoint::Fixed,
        ScalarOptimizationMode::ReleaseScalar,
        1,
        logical_horizon,
        1,
        benchmark_policy_sha256().expect("policy should hash"),
        false,
    )
    .expect("native run identity should validate");
    let request = BenchmarkRunRequest::new(identity, resolved.canonical_bytes().to_vec())
        .expect("native benchmark request should validate");
    (prepared, request)
}

fn fake_oracle_repository(behavior: &str) -> PathBuf {
    let id = TEST_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/paired-performance-tests")
        .join(format!("{}-{id}", std::process::id()));
    let output = root.join("target/reference/oracle-release");
    fs::create_dir_all(&output).expect("fake oracle output should be creatable");
    let executable = output.join(if cfg!(windows) {
        "liquidfun-reference.exe"
    } else {
        "liquidfun-reference"
    });
    fs::copy(env!("CARGO_BIN_EXE_liquidfun-fake-oracle"), executable)
        .expect("fake oracle should copy into the reviewed release path");
    fs::write(output.join("behavior.txt"), behavior).expect("fake behavior should be writable");
    root
}

fn report_identity() -> PerformanceReportIdentity {
    let hardware = HardwareSession::new(
        "same-host-session-1",
        "reviewed-test-cpu",
        8,
        16 * 1024 * 1024 * 1024,
        "test-os",
    )
    .expect("hardware identity should validate");
    PerformanceReportIdentity::new(PerformanceReportIdentityFields::new(
        "paired-performance",
        "rust-revision",
        "oracle-revision",
        "rustc 1.97.0",
        "rust-lld",
        "clang 22.1.8",
        "lld 22.1.8",
        "aarch64-apple-darwin",
        "-C opt-level=3",
        "-Wl,dead_strip",
        "-O3",
        "-Wl,-dead_strip",
        hardware,
        benchmark_policy_sha256().expect("policy should hash"),
        hash(b"matrix"),
        hash(b"catalog"),
        hash(RESOLVED_BYTES),
    ))
    .expect("report identity should validate")
}

fn plan(profile_enabled: bool) -> PairedBenchmarkPlan {
    PairedBenchmarkPlan::new(
        "paired-case",
        RESOLVED_BYTES.to_vec(),
        RunSettings::new(FloatBits::new(0x3c88_8889), 8, 3, 1).expect("settings should validate"),
        PerformanceWorkloadKind::WorldStep,
        PerformanceSizePoint::Fixed,
        ScalarOptimizationMode::ReleaseScalar,
        1,
        profile_enabled,
        report_identity(),
        CompatibilityStatus::D2Supported,
    )
    .expect("paired plan should validate")
}

#[test]
fn five_independent_runs_interleave_exact_order_and_retain_raw_identity() {
    // Arrange
    let mut native = RecordingAdapter::new(PerformanceEngineRole::NativeRust);
    let mut oracle = RecordingAdapter::new(PerformanceEngineRole::PinnedCppOracle);

    // Act
    let outcome = run_paired_benchmark(&plan(false), &mut native, &mut oracle);

    // Assert
    let PairedBenchmarkOutcome::Performance(report) = outcome else {
        panic!("complete paired execution should produce performance evidence");
    };
    assert_eq!(report.independent_runs(), 5);
    assert_eq!(report.raw_samples().len(), 150);
    assert_eq!(
        report.raw_samples()[0].engine_order(),
        PairedEngineOrder::NativeThenOracle
    );
    assert_eq!(
        report.raw_samples()[1].engine_order(),
        PairedEngineOrder::OracleThenNative
    );
    assert_eq!(report.raw_samples()[0].baseline_run(), 1);
    assert_eq!(report.raw_samples()[0].sample_ordinal(), 1);
    assert_eq!(report.raw_samples()[0].native_nanoseconds(), 10_001);
    assert_eq!(report.raw_samples()[0].oracle_nanoseconds(), 10_001);
    assert_eq!(report.raw_samples()[149].baseline_run(), 5);
    assert_eq!(report.raw_samples()[149].sample_ordinal(), 30);
    assert_eq!(native.calls.len(), 150);
    assert_eq!(oracle.calls.len(), 150);
}

#[test]
fn stable_first_harness_failure_stops_execution_without_a_duration() {
    // Arrange
    let mut native = RecordingAdapter::new(PerformanceEngineRole::NativeRust);
    let mut oracle = RecordingAdapter::new(PerformanceEngineRole::PinnedCppOracle).with_injected(
        2,
        InjectedOutcome::Harness(BenchmarkHarnessFailureKind::RequestTimeout),
    );

    // Act
    let outcome = run_paired_benchmark(&plan(false), &mut native, &mut oracle);

    // Assert
    let PairedBenchmarkOutcome::HarnessFailure(failure) = outcome else {
        panic!("timeout must remain a harness failure");
    };
    assert_eq!(failure.kind(), BenchmarkHarnessFailureKind::RequestTimeout);
    assert_eq!(failure.baseline_run(), 1);
    assert_eq!(failure.sample_ordinal(), 2);
    assert_eq!(
        failure.engine_role(),
        PerformanceEngineRole::PinnedCppOracle
    );
    assert_eq!(native.calls.len(), 1);
    assert_eq!(oracle.calls.len(), 2);
}

#[test]
fn semantic_divergence_is_disjoint_from_harness_and_performance() {
    // Arrange
    let mut native = RecordingAdapter::new(PerformanceEngineRole::NativeRust)
        .with_injected(3, InjectedOutcome::Mismatch);
    let mut oracle = RecordingAdapter::new(PerformanceEngineRole::PinnedCppOracle);

    // Act
    let outcome = run_paired_benchmark(&plan(false), &mut native, &mut oracle);

    // Assert
    let PairedBenchmarkOutcome::PhysicsMismatch(mismatch) = outcome else {
        panic!("semantic divergence must remain a physics mismatch");
    };
    assert_eq!(mismatch.baseline_run(), 1);
    assert_eq!(mismatch.sample_ordinal(), 3);
    assert_eq!(mismatch.engine_role(), PerformanceEngineRole::NativeRust);
    assert_eq!(native.calls.len(), 3);
    assert_eq!(oracle.calls.len(), 2);
}

#[test]
fn reset_and_checkpoint_identity_fail_closed_before_raw_acceptance() {
    // Arrange
    let mut native = RecordingAdapter::new(PerformanceEngineRole::NativeRust);
    native.wrong_reset_at = Some(4);
    let mut oracle = RecordingAdapter::new(PerformanceEngineRole::PinnedCppOracle);

    // Act
    let reset_outcome = run_paired_benchmark(&plan(false), &mut native, &mut oracle);
    let mut native = RecordingAdapter::new(PerformanceEngineRole::NativeRust);
    let mut oracle = RecordingAdapter::new(PerformanceEngineRole::PinnedCppOracle);
    oracle.checkpoint_salt = 1;
    let checkpoint_outcome = run_paired_benchmark(&plan(false), &mut native, &mut oracle);

    // Assert
    let PairedBenchmarkOutcome::HarnessFailure(reset_failure) = reset_outcome else {
        panic!("reset disagreement must remain a harness failure");
    };
    assert_eq!(
        reset_failure.kind(),
        BenchmarkHarnessFailureKind::AdapterResetFailure
    );
    let PairedBenchmarkOutcome::PhysicsMismatch(checkpoint_mismatch) = checkpoint_outcome else {
        panic!("semantic checkpoint hash disagreement must be a physics mismatch");
    };
    assert_eq!(checkpoint_mismatch.baseline_run(), 1);
    assert_eq!(checkpoint_mismatch.sample_ordinal(), 1);
}

#[test]
fn parent_and_rust_child_profiles_remain_diagnostic_only() {
    // Arrange
    let mut native = RecordingAdapter::new(PerformanceEngineRole::NativeRust).with_profiles();
    let mut oracle = RecordingAdapter::new(PerformanceEngineRole::PinnedCppOracle).with_profiles();

    // Act
    let outcome = run_paired_benchmark(&plan(true), &mut native, &mut oracle);

    // Assert
    let PairedBenchmarkOutcome::Performance(report) = outcome else {
        panic!("profiled diagnostics must not replace performance authority");
    };
    let sample = &report.raw_samples()[0];
    assert_eq!(report.profile_schema(), DiagnosticProfileSchema::Phase12V1);
    assert_eq!(sample.native_nanoseconds(), 10_001);
    assert_eq!(sample.oracle_nanoseconds(), 10_001);
    assert_eq!(
        sample.native_common_parent_diagnostics()[0].nanoseconds(),
        71
    );
    assert_eq!(
        sample.oracle_common_parent_diagnostics()[0].nanoseconds(),
        71
    );
    assert_eq!(
        sample.rust_child_diagnostics()[0].phase(),
        DiagnosticProfileChild::RustIslandSolve
    );
    assert_eq!(
        sample.rust_child_diagnostics()[0].phase().parent(),
        DiagnosticProfileParent::RigidSolve
    );
    assert_eq!(sample.rust_child_diagnostics()[0].nanoseconds(), 37);
}

#[test]
fn reviewed_process_cycle_restarts_child_epoch_without_losing_raw_identity() {
    // Arrange
    let mut native = RecordingAdapter::new(PerformanceEngineRole::NativeRust).with_cycle_after(100);
    let mut oracle =
        RecordingAdapter::new(PerformanceEngineRole::PinnedCppOracle).with_cycle_after(100);

    // Act
    let outcome = run_paired_benchmark(&plan(false), &mut native, &mut oracle);

    // Assert
    let PairedBenchmarkOutcome::Performance(report) = outcome else {
        panic!("reviewed process cycling should preserve paired evidence");
    };
    assert_eq!(report.raw_samples()[99].native_process_generation(), 1);
    assert_eq!(report.raw_samples()[99].native_reset_epoch(), 100);
    assert_eq!(report.raw_samples()[100].native_process_generation(), 2);
    assert_eq!(report.raw_samples()[100].native_reset_epoch(), 1);
    assert_eq!(report.raw_samples()[100].oracle_process_generation(), 2);
    assert_eq!(report.raw_samples()[100].oracle_reset_epoch(), 1);
}

#[test]
fn concrete_native_adapter_preserves_request_aware_checkpoint_identity() {
    // Arrange
    let (prepared, request) = native_benchmark_request();
    let direct = prepared
        .measure_sample_for_request(&request)
        .expect("request-aware native measurement should pass");
    let mut adapter = NativeBenchmarkAdapter::new(prepared);

    // Act
    let output = adapter
        .execute(&request, 1)
        .expect("native paired adapter should return typed performance");

    // Assert
    assert_eq!(output.process_generation(), 1);
    assert_eq!(output.result().reset_epoch(), 1);
    assert_eq!(
        output.result().engine_role(),
        PerformanceEngineRole::NativeRust
    );
    let BenchmarkRunOutcome::Performance(performance) = output.result().outcome() else {
        panic!("native adapter must return only validated performance");
    };
    assert_eq!(
        performance.semantic_checkpoint_identity(),
        direct.semantic_checkpoint_identity()
    );
    assert!(performance.unprofiled_nanoseconds() > 0);
}

#[test]
fn supervised_oracle_poison_recovery_restarts_reset_identity() {
    // Arrange
    let root = fake_oracle_repository("benchmark_second_malformed");
    let mut oracle = OracleBenchmarkAdapter::new(&root, ORACLE_REVISION)
        .expect("confined fake release oracle should resolve");

    // Act
    let first = oracle
        .execute(&benchmark_request(1), 1)
        .expect("first benchmark request should pass");
    let failure = oracle
        .execute(&benchmark_request(2), 1)
        .expect_err("malformed second record should poison the child");
    let recovered = oracle
        .execute(&benchmark_request(3), 1)
        .expect("next request should start a fresh bounded child");

    // Assert
    assert_eq!(first.result().reset_epoch(), 1);
    assert_eq!(failure, BenchmarkHarnessFailureKind::MalformedRecord);
    assert_eq!(recovered.process_generation(), 2);
    assert_eq!(recovered.result().reset_epoch(), 1);
}

#[test]
fn supervised_oracle_classifies_bounded_output_and_sanitizer_failures() {
    // Arrange
    let oversized_root = fake_oracle_repository("benchmark_oversized");
    let sanitizer_root = fake_oracle_repository("benchmark_sanitizer");
    let mut oversized = OracleBenchmarkAdapter::new(&oversized_root, ORACLE_REVISION)
        .expect("oversized fake oracle should resolve");
    let mut sanitizer = OracleBenchmarkAdapter::new(&sanitizer_root, ORACLE_REVISION)
        .expect("sanitizer fake oracle should resolve");

    // Act
    let oversized_failure = oversized
        .execute(&benchmark_request(1), 1)
        .expect_err("oversized output must fail");
    let sanitizer_failure = sanitizer
        .execute(&benchmark_request(1), 1)
        .expect_err("sanitizer output must fail");

    // Assert
    assert_eq!(
        oversized_failure,
        BenchmarkHarnessFailureKind::OutputLimitExceeded
    );
    assert_eq!(
        sanitizer_failure,
        BenchmarkHarnessFailureKind::SanitizerReport
    );
}

#[test]
fn supervised_oracle_classifies_crash_and_provenance_failures() {
    // Arrange
    let crash_root = fake_oracle_repository("benchmark_crash");
    let provenance_root = fake_oracle_repository("wrong_provenance");
    let mut crash = OracleBenchmarkAdapter::new(&crash_root, ORACLE_REVISION)
        .expect("crashing fake oracle should resolve");
    let mut provenance = OracleBenchmarkAdapter::new(&provenance_root, ORACLE_REVISION)
        .expect("wrong-provenance fake oracle should resolve");

    // Act
    let crash_failure = crash
        .execute(&benchmark_request(1), 1)
        .expect_err("child crash must fail");
    let provenance_failure = provenance
        .execute(&benchmark_request(1), 1)
        .expect_err("wrong provenance must fail before benchmark execution");

    // Assert
    assert_eq!(crash_failure, BenchmarkHarnessFailureKind::ChildNonZeroExit);
    assert_eq!(
        provenance_failure,
        BenchmarkHarnessFailureKind::IdentityMismatch
    );
}
