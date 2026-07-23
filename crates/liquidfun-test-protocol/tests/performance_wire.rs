//! Integration coverage for the strict paired benchmark wire contract.

use liquidfun_test_protocol::{
    CheckpointId, CodecErrorKind, FloatBits, HarnessLimits, RequestId, RunSettings, Sha256Hex,
    performance::{
        BenchmarkCommonParentDiagnostic, BenchmarkCommonParentPhase, BenchmarkHarnessFailure,
        BenchmarkHarnessFailureKind, BenchmarkPerformanceResult, BenchmarkPhysicsMismatch,
        BenchmarkRunIdentity, BenchmarkRunOutcome, BenchmarkRunRequest, BenchmarkRunResult,
        BenchmarkWireErrorKind, PerformanceEngineRole, PerformanceSizePoint,
        PerformanceWorkloadKind, ScalarOptimizationMode, SemanticCheckpointIdentity,
        benchmark_policy_sha256, decode_benchmark_run_request_jsonl,
        decode_benchmark_run_result_jsonl, encode_benchmark_run_request_jsonl,
        encode_benchmark_run_result_jsonl, validate_benchmark_run_pair,
    },
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const RESOLVED_BYTES: &[u8] = br#"{"scenario":"rigid-runtime-mutation","version":1}"#;

fn hash(bytes: &[u8]) -> Sha256Hex {
    Sha256Hex::from_digest(Sha256::digest(bytes).into())
}

fn run_identity(profile_enabled: bool) -> BenchmarkRunIdentity {
    BenchmarkRunIdentity::new(
        RequestId::new("benchmark-request-1").expect("fixture request ID should validate"),
        hash(RESOLVED_BYTES),
        RunSettings::new(FloatBits::new(0x3c88_8889), 8, 3, 1)
            .expect("fixture settings should validate"),
        PerformanceWorkloadKind::WorldStep,
        PerformanceSizePoint::Fixed,
        ScalarOptimizationMode::ReleaseScalar,
        1,
        64,
        7,
        benchmark_policy_sha256().expect("reviewed policy should hash"),
        profile_enabled,
    )
    .expect("fixture run identity should validate")
}

fn checkpoint_identity(identity: &BenchmarkRunIdentity) -> SemanticCheckpointIdentity {
    SemanticCheckpointIdentity::new(
        identity.request_id().clone(),
        identity.resolved_sha256().clone(),
        CheckpointId::new("checkpoint-0064").expect("fixture checkpoint ID should validate"),
        hash(b"semantic-checkpoint"),
    )
}

fn request(profile_enabled: bool) -> BenchmarkRunRequest {
    BenchmarkRunRequest::new(run_identity(profile_enabled), RESOLVED_BYTES.to_vec())
        .expect("fixture request should validate")
}

fn performance_result(profile_enabled: bool) -> BenchmarkRunResult {
    let identity = run_identity(profile_enabled);
    let maybe_diagnostics = profile_enabled.then(|| {
        vec![
            BenchmarkCommonParentDiagnostic::new(BenchmarkCommonParentPhase::BroadPhase, 11_003)
                .expect("fixture diagnostic should validate"),
            BenchmarkCommonParentDiagnostic::new(BenchmarkCommonParentPhase::ContactSolve, 22_007)
                .expect("fixture diagnostic should validate"),
        ]
    });
    let measurement = BenchmarkPerformanceResult::new(
        123_456_789,
        maybe_diagnostics,
        checkpoint_identity(&identity),
    )
    .expect("fixture measurement should validate");
    BenchmarkRunResult::new(
        identity,
        PerformanceEngineRole::NativeRust,
        17,
        BenchmarkRunOutcome::Performance(measurement),
    )
    .expect("fixture result should validate")
}

fn json_line(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).expect("fixture JSON should encode");
    bytes.push(b'\n');
    bytes
}

#[test]
fn complete_request_and_result_round_trip_preserve_raw_values() {
    // Arrange
    let request = request(true);
    let result = performance_result(true);
    let limits = HarnessLimits::phase2_default_v1();

    // Act
    let request_bytes = encode_benchmark_run_request_jsonl(&request, &limits)
        .expect("complete request should encode");
    let result_bytes =
        encode_benchmark_run_result_jsonl(&result, &limits).expect("complete result should encode");
    let decoded_request = decode_benchmark_run_request_jsonl(&request_bytes, &limits)
        .expect("complete request should decode");
    let decoded_result = decode_benchmark_run_result_jsonl(&result_bytes, &limits)
        .expect("complete result should decode");

    // Assert
    assert_eq!(decoded_request, request);
    assert_eq!(decoded_request.resolved_bytes(), RESOLVED_BYTES);
    assert_eq!(decoded_result, result);
    assert_eq!(decoded_result.reset_epoch(), 17);
    validate_benchmark_run_pair(&decoded_request, &decoded_result)
        .expect("the complete pair should share one identity");
    let BenchmarkRunOutcome::Performance(measurement) = decoded_result.outcome() else {
        panic!("fixture should remain a performance result");
    };
    assert_eq!(measurement.unprofiled_nanoseconds(), 123_456_789);
    assert_eq!(
        measurement
            .maybe_common_parent_diagnostics()
            .expect("profile diagnostics should remain present")[0]
            .nanoseconds(),
        11_003
    );
}

#[test]
fn request_rejects_unknown_duplicate_missing_and_oversized_records() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let encoded = encode_benchmark_run_request_jsonl(&request(false), &limits)
        .expect("fixture request should encode");
    let original: Value = serde_json::from_slice(&encoded).expect("fixture JSON should parse");
    let mut unknown = original.clone();
    unknown["private_slot"] = json!(7);
    let duplicate = String::from_utf8(encoded)
        .expect("fixture should be UTF-8")
        .replacen(
            "\"protocol_version\":1",
            "\"protocol_version\":1,\"protocol_version\":1",
            1,
        )
        .into_bytes();
    let mut missing = original;
    missing["identity"]
        .as_object_mut()
        .expect("identity should be an object")
        .remove("settings");
    let mut oversized = vec![b'x'; limits.input_record_bytes()];
    oversized.push(b'\n');

    // Act
    let unknown_error = decode_benchmark_run_request_jsonl(&json_line(&unknown), &limits)
        .expect_err("unknown fields must fail");
    let duplicate_error = decode_benchmark_run_request_jsonl(&duplicate, &limits)
        .expect_err("duplicate fields must fail");
    let missing_error = decode_benchmark_run_request_jsonl(&json_line(&missing), &limits)
        .expect_err("missing fields must fail");
    let oversized_error = decode_benchmark_run_request_jsonl(&oversized, &limits)
        .expect_err("oversized records must fail");

    // Assert
    assert_eq!(
        unknown_error.codec_kind(),
        Some(CodecErrorKind::UnknownField)
    );
    assert_eq!(
        duplicate_error.codec_kind(),
        Some(CodecErrorKind::DuplicateMember)
    );
    assert_eq!(
        missing_error.codec_kind(),
        Some(CodecErrorKind::MalformedRecord)
    );
    assert_eq!(
        oversized_error.codec_kind(),
        Some(CodecErrorKind::RecordTooLarge)
    );
}

#[test]
fn request_and_pair_reject_identity_mismatches_before_execution() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let encoded = encode_benchmark_run_request_jsonl(&request(false), &limits)
        .expect("fixture request should encode");
    let mut wrong_hash: Value =
        serde_json::from_slice(&encoded).expect("fixture JSON should parse");
    wrong_hash["resolved_bytes"] = json!([0, 1, 2, 3]);
    let request = request(false);
    let result_bytes = encode_benchmark_run_result_jsonl(&performance_result(false), &limits)
        .expect("fixture result should encode");
    let mut wrong_ordinal: Value =
        serde_json::from_slice(&result_bytes).expect("fixture JSON should parse");
    wrong_ordinal["identity"]["sample_ordinal"] = json!(8);

    // Act
    let hash_error = decode_benchmark_run_request_jsonl(&json_line(&wrong_hash), &limits)
        .expect_err("contradictory resolved bytes must fail");
    let result = decode_benchmark_run_result_jsonl(&json_line(&wrong_ordinal), &limits)
        .expect("independently valid result should decode");
    let pair_error =
        validate_benchmark_run_pair(&request, &result).expect_err("different identity must fail");

    // Assert
    assert_eq!(
        hash_error.validation_kind(),
        Some(BenchmarkWireErrorKind::ResolvedHashMismatch)
    );
    assert_eq!(
        pair_error.validation_kind(),
        Some(BenchmarkWireErrorKind::RunIdentityMismatch)
    );
}

#[test]
fn result_rejects_profiled_authority_and_checkpoint_contradictions() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let encoded = encode_benchmark_run_result_jsonl(&performance_result(true), &limits)
        .expect("fixture result should encode");
    let original: Value = serde_json::from_slice(&encoded).expect("fixture JSON should parse");
    let mut profiled_total = original.clone();
    profiled_total["outcome"]["outcome"]["profiled_nanoseconds"] = json!(123);
    let mut unknown_outcome = original.clone();
    unknown_outcome["outcome"]["private_slot"] = json!(7);
    let mut wrong_checkpoint = original;
    wrong_checkpoint["outcome"]["outcome"]["semantic_checkpoint_identity"]["resolved_sha256"] =
        json!("f".repeat(64));

    // Act
    let profiled_error = decode_benchmark_run_result_jsonl(&json_line(&profiled_total), &limits)
        .expect_err("profiled totals must never become authority");
    let unknown_outcome_error =
        decode_benchmark_run_result_jsonl(&json_line(&unknown_outcome), &limits)
            .expect_err("unknown outcome fields must fail");
    let checkpoint_error =
        decode_benchmark_run_result_jsonl(&json_line(&wrong_checkpoint), &limits)
            .expect_err("checkpoint identity contradictions must fail");

    // Assert
    assert_eq!(
        profiled_error.codec_kind(),
        Some(CodecErrorKind::UnknownField)
    );
    assert_eq!(
        unknown_outcome_error.codec_kind(),
        Some(CodecErrorKind::MalformedRecord)
    );
    assert_eq!(
        checkpoint_error.validation_kind(),
        Some(BenchmarkWireErrorKind::CheckpointIdentityMismatch)
    );
}

#[test]
fn typed_bounds_reject_invalid_policy_measurement_and_profile_values() {
    // Arrange
    let base = run_identity(false);
    let identity = |warmup_count, measured_horizon, sample_ordinal| {
        BenchmarkRunIdentity::new(
            base.request_id().clone(),
            base.resolved_sha256().clone(),
            base.settings(),
            base.workload(),
            base.size_point(),
            base.optimization_mode(),
            warmup_count,
            measured_horizon,
            sample_ordinal,
            base.policy_sha256().clone(),
            false,
        )
    };
    let diagnostic =
        BenchmarkCommonParentDiagnostic::new(BenchmarkCommonParentPhase::BroadPhase, 9)
            .expect("fixture diagnostic should validate");
    let duplicate_diagnostics = BenchmarkPerformanceResult::new(
        100,
        Some(vec![diagnostic, diagnostic]),
        checkpoint_identity(&base),
    );
    let oversized_resolved_bytes = BenchmarkRunRequest::new(base.clone(), vec![0; 1024 * 1024 + 1]);
    let profiled_while_disabled =
        BenchmarkPerformanceResult::new(100, Some(vec![diagnostic]), checkpoint_identity(&base))
            .and_then(|measurement| {
                BenchmarkRunResult::new(
                    base.clone(),
                    PerformanceEngineRole::NativeRust,
                    1,
                    BenchmarkRunOutcome::Performance(measurement),
                )
            });

    // Act and Assert
    assert_eq!(
        identity(0, 1, 1)
            .expect_err("zero warmup must fail")
            .validation_kind(),
        Some(BenchmarkWireErrorKind::InvalidWarmupCount)
    );
    assert_eq!(
        identity(1, 4_097, 1)
            .expect_err("oversized horizon must fail")
            .validation_kind(),
        Some(BenchmarkWireErrorKind::InvalidMeasuredHorizon)
    );
    assert_eq!(
        identity(1, 1, 31)
            .expect_err("oversized sample ordinal must fail")
            .validation_kind(),
        Some(BenchmarkWireErrorKind::InvalidSampleOrdinal)
    );
    assert_eq!(
        duplicate_diagnostics
            .expect_err("duplicate phases must fail")
            .validation_kind(),
        Some(BenchmarkWireErrorKind::DuplicateDiagnosticPhase)
    );
    assert_eq!(
        oversized_resolved_bytes
            .expect_err("oversized resolved bytes must fail")
            .validation_kind(),
        Some(BenchmarkWireErrorKind::ResolvedBytesTooLarge)
    );
    assert_eq!(
        profiled_while_disabled
            .expect_err("diagnostics require profile-enabled identity")
            .validation_kind(),
        Some(BenchmarkWireErrorKind::UnexpectedProfileDiagnostics)
    );
}

#[test]
fn terminal_outcomes_keep_performance_mismatch_and_harness_failure_distinct() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let identity = run_identity(false);
    let outcomes = [
        BenchmarkRunOutcome::Performance(
            BenchmarkPerformanceResult::new(100, None, checkpoint_identity(&identity))
                .expect("fixture measurement should validate"),
        ),
        BenchmarkRunOutcome::PhysicsMismatch(BenchmarkPhysicsMismatch::new(checkpoint_identity(
            &identity,
        ))),
        BenchmarkRunOutcome::HarnessFailure(BenchmarkHarnessFailure::new(
            BenchmarkHarnessFailureKind::RequestTimeout,
        )),
    ];

    // Act
    let decoded = outcomes.map(|outcome| {
        let result = BenchmarkRunResult::new(
            identity.clone(),
            PerformanceEngineRole::PinnedCppOracle,
            3,
            outcome,
        )
        .expect("fixture terminal result should validate");
        let bytes = encode_benchmark_run_result_jsonl(&result, &limits)
            .expect("fixture terminal result should encode");
        decode_benchmark_run_result_jsonl(&bytes, &limits)
            .expect("fixture terminal result should decode")
    });

    // Assert
    assert!(matches!(
        decoded[0].outcome(),
        BenchmarkRunOutcome::Performance(_)
    ));
    assert!(matches!(
        decoded[1].outcome(),
        BenchmarkRunOutcome::PhysicsMismatch(_)
    ));
    assert!(matches!(
        decoded[2].outcome(),
        BenchmarkRunOutcome::HarnessFailure(_)
    ));
}
