//! Process-supervisor lifecycle, resource-bound, and failure taxonomy tests.

use std::{
    fs,
    path::Path,
    sync::{Mutex, MutexGuard},
};

use liquidfun_differential::execute_math_probe_process;
use liquidfun_differential::{OracleExecutable, OraclePreset, SessionProfile};
use liquidfun_test_protocol::{
    HarnessFailure, HarnessFailureKind, HarnessLimits, MathProbeRequestRecord,
    ScenarioRequestRecord, decode_math_probe_request_jsonl, decode_scenario_request_jsonl,
};

#[path = "supervisor_failures/fake_repository.rs"]
mod fake_repository;

use fake_repository::{FakeRepository, TestSupervisor};

const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
static PROCESS_SLOT: Mutex<()> = Mutex::new(());
const REQUEST_BYTES: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/empty-world-request.jsonl");
const MATH_REQUEST_BYTES: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/math-probe-request.jsonl");

fn fixture_request() -> ScenarioRequestRecord {
    decode_scenario_request_jsonl(REQUEST_BYTES, &HarnessLimits::phase2_default_v1())
        .expect("checked-in request should validate")
}

fn math_fixture_request() -> MathProbeRequestRecord {
    decode_math_probe_request_jsonl(MATH_REQUEST_BYTES, &HarnessLimits::phase2_default_v1())
        .expect("checked-in math request should validate")
}

fn fixture_request_with_id(request_id: &str) -> ScenarioRequestRecord {
    let bytes = String::from_utf8(REQUEST_BYTES.to_vec())
        .expect("fixture is UTF-8")
        .replace(
            "\"request_id\":\"empty-world-request\"",
            &format!("\"request_id\":\"{request_id}\""),
        );
    decode_scenario_request_jsonl(bytes.as_bytes(), &HarnessLimits::phase2_default_v1())
        .expect("changed request identity should remain valid")
}

fn fake_repository(behavior: &str) -> FakeRepository {
    FakeRepository::create(behavior).expect("owned fake repository should be creatable")
}

fn supervisor(behavior: &str, profile: SessionProfile) -> TestSupervisor {
    TestSupervisor::create(behavior, profile, REVISION)
}

fn process_slot() -> MutexGuard<'static, ()> {
    // Host startup contention can consume the real five-second startup budget.
    // Serialize test-owned child lifetimes, not production supervision or pipe draining.
    PROCESS_SLOT
        .lock()
        .expect("test process slot should not be poisoned")
}

fn failure(behavior: &str) -> HarnessFailure {
    supervisor(behavior, SessionProfile::OneShot)
        .execute(&fixture_request())
        .expect_err("injected fake behavior should fail")
}

#[test]
fn one_shot_and_reuse_enforce_reset_epochs_and_periodic_cycling() {
    // Arrange
    let _process_slot = process_slot();
    let first_request = fixture_request();
    let second_request = fixture_request_with_id("empty-world-request-2");
    let mut one_shot = supervisor("valid", SessionProfile::OneShot);
    let mut reuse = supervisor("valid", SessionProfile::Reuse);

    // Act
    let isolated_first = one_shot
        .execute(&first_request)
        .expect("one-shot should pass");
    let isolated_second = one_shot
        .execute(&second_request)
        .expect("second one-shot should pass");
    let reused_first = reuse
        .execute(&first_request)
        .expect("first reuse should pass");
    let reused_second = reuse
        .execute(&second_request)
        .expect("second reuse should pass");
    for _ in 2..HarnessLimits::phase2_reuse_v1().request_budget() {
        reuse
            .execute(&first_request)
            .expect("budgeted reuse should pass");
    }
    let cycled = reuse
        .execute(&second_request)
        .expect("cycled process should pass");

    // Assert
    assert_eq!(
        (isolated_first.reset_epoch(), isolated_second.reset_epoch()),
        (1, 1)
    );
    assert_eq!(
        (reused_first.reset_epoch(), reused_second.reset_epoch()),
        (1, 2)
    );
    assert_eq!(cycled.reset_epoch(), 1);
    assert_eq!(one_shot.process_generation(), 2);
    assert_eq!(reuse.process_generation(), 2);
    assert_eq!(reuse.requests_in_current_process(), 1);
}

#[test]
fn startup_request_exit_signal_and_sanitizer_failures_are_typed() {
    // Arrange
    let _process_slot = process_slot();
    let cases = [
        ("startup_timeout", HarnessFailureKind::StartupTimeout),
        (
            "handshake_malformed",
            HarnessFailureKind::HandshakeMalformed,
        ),
        (
            "unsupported_version",
            HarnessFailureKind::UnsupportedVersion,
        ),
        ("wrong_provenance", HarnessFailureKind::WrongProvenance),
        ("request_timeout", HarnessFailureKind::RequestTimeout),
        ("nonzero", HarnessFailureKind::ChildNonZeroExit),
        ("sanitizer", HarnessFailureKind::SanitizerReport),
    ];
    // Act and Assert
    for (behavior, expected) in cases {
        let actual = failure(behavior);
        assert_eq!(actual.kind(), expected, "behavior {behavior}");
        assert!(actual.evidence().child_reaped(), "behavior {behavior}");
    }
    #[cfg(unix)]
    {
        let actual = failure("signal");
        assert_eq!(actual.kind(), HarnessFailureKind::ChildSignaled);
        assert!(actual.evidence().child_reaped());
    }
}

#[test]
fn framing_and_output_limit_failures_are_typed() {
    // Arrange
    let _process_slot = process_slot();
    let cases = [
        ("eof", HarnessFailureKind::UnexpectedEof),
        ("partial", HarnessFailureKind::PartialRecord),
        ("malformed", HarnessFailureKind::MalformedRecord),
        ("unknown_kind", HarnessFailureKind::UnknownRecordKind),
        ("oversized", HarnessFailureKind::RecordTooLarge),
        ("trace_too_large", HarnessFailureKind::TraceTooLarge),
    ];

    // Act and Assert
    for (behavior, expected) in cases {
        let actual = failure(behavior);
        assert_eq!(actual.kind(), expected, "behavior {behavior}");
        assert!(actual.evidence().child_reaped(), "behavior {behavior}");
    }
}

#[test]
fn observed_output_overflow_takes_precedence_at_request_deadline() {
    // Arrange
    let _process_slot = process_slot();
    let mut supervisor = supervisor("total_overflow", SessionProfile::OneShot);

    // Act
    let failure = supervisor
        .execute(&fixture_request())
        .expect_err("observed output overflow should fail before timeout classification");

    // Assert
    assert_eq!(failure.kind(), HarnessFailureKind::TotalOutputExceeded);
}

#[test]
fn request_identity_sequence_and_reset_failures_are_typed() {
    // Arrange
    let _process_slot = process_slot();
    let cases = [
        ("request_mismatch", HarnessFailureKind::RequestIdMismatch),
        (
            "identity_mismatch",
            HarnessFailureKind::TraceIdentityMismatch,
        ),
        ("sequence", HarnessFailureKind::SequenceViolation),
        ("reset", HarnessFailureKind::AdapterResetFailure),
        ("scenario_rejected", HarnessFailureKind::ScenarioRejected),
        ("cpp_adapter_failure", HarnessFailureKind::CppAdapterFailure),
    ];

    // Act and Assert
    for (behavior, expected) in cases {
        let actual = failure(behavior);
        assert_eq!(actual.kind(), expected, "behavior {behavior}");
        assert!(actual.evidence().child_reaped(), "behavior {behavior}");
        assert_eq!(
            actual
                .evidence()
                .maybe_request_id()
                .map(liquidfun_test_protocol::RequestId::as_str),
            Some("empty-world-request")
        );
    }
}

#[test]
fn poisoned_session_preserves_bounded_stderr_and_kill_reap_evidence() {
    // Arrange
    let _process_slot = process_slot();
    let limits = HarnessLimits::phase2_default_v1();

    // Act
    let actual = failure("large_stderr_malformed");
    let was_killed = actual.evidence().child_killed();
    let was_reaped = actual.evidence().child_reaped();

    // Assert
    assert_eq!(actual.kind(), HarnessFailureKind::MalformedRecord);
    assert!(was_killed);
    assert!(was_reaped);
    assert_eq!(
        actual.evidence().stderr().retained().len(),
        limits.retained_stderr_bytes()
    );
    assert_eq!(actual.evidence().stderr().total_bytes(), 1024 * 1024);
    assert_eq!(
        actual.evidence().stderr().truncated_bytes(),
        1024 * 1024 - limits.retained_stderr_bytes()
    );
}

#[test]
fn concurrent_stdout_and_large_stderr_drain_without_pipe_deadlock() {
    // Arrange
    let _process_slot = process_slot();
    let mut supervisor = supervisor("large_stderr_valid", SessionProfile::OneShot);

    // Act
    let trace = supervisor
        .execute(&fixture_request())
        .expect("concurrent drains should let the valid trace complete");

    // Assert
    assert_eq!(trace.checkpoints().len(), 2);
}

#[test]
fn math_probe_path_bounds_records_stderr_partial_lines_and_timeouts() {
    // Arrange
    let _process_slot = process_slot();
    let cases = [
        ("startup_timeout", HarnessFailureKind::StartupTimeout),
        ("oversized", HarnessFailureKind::RecordTooLarge),
        ("partial", HarnessFailureKind::PartialRecord),
        ("request_timeout", HarnessFailureKind::RequestTimeout),
        (
            "math_large_stderr_malformed",
            HarnessFailureKind::MalformedRecord,
        ),
    ];
    let request = math_fixture_request();

    // Act and Assert
    for (behavior, expected) in cases {
        let root = fake_repository(behavior);
        let executable = OracleExecutable::resolve(&root, OraclePreset::Debug)
            .expect("confined fake oracle should resolve");
        let failure = execute_math_probe_process(&executable, &request, REVISION)
            .expect_err("injected math-probe behavior should fail");
        assert_eq!(failure.kind(), expected, "behavior {behavior}");
        assert!(failure.child_reaped(), "behavior {behavior}");
        if behavior != "partial" {
            assert!(failure.child_killed(), "behavior {behavior}");
        }
        if behavior == "math_large_stderr_malformed" {
            assert_eq!(failure.stderr_bytes(), 1024 * 1024);
            assert_eq!(
                failure.retained_stderr().len(),
                HarnessLimits::phase2_default_v1().retained_stderr_bytes()
            );
        }
    }
}

#[test]
fn concurrent_overlimit_stderr_fails_one_shot_and_reuse_requests() {
    // Arrange
    let _process_slot = process_slot();
    let request = fixture_request();
    let profiles = [SessionProfile::OneShot, SessionProfile::Reuse];

    // Act and Assert
    for profile in profiles {
        let failure = supervisor("concurrent_total_overflow", profile)
            .execute(&request)
            .expect_err("concurrent over-limit stderr should fail");
        assert_eq!(failure.kind(), HarnessFailureKind::TotalOutputExceeded);
    }
}

#[test]
#[cfg(unix)]
fn executable_resolution_rejects_symlinked_or_out_of_tree_candidates() {
    use std::os::unix::fs::symlink;

    // Arrange
    let root = fake_repository("valid");
    let output = root.join("target/reference/oracle-debug");
    let executable = output.join("liquidfun-reference");
    let real = output.join("real-oracle");
    fs::rename(&executable, &real).expect("fake executable should move");
    symlink(&real, &executable).expect("test symlink should be creatable");

    // Act
    let symlink_result = OracleExecutable::resolve(&root, OraclePreset::Debug);
    let outside_result = OracleExecutable::resolve(Path::new("/"), OraclePreset::Debug);

    // Assert
    assert!(symlink_result.is_err());
    assert!(outside_result.is_err());
}

#[test]
fn reuse_child_is_torn_down_before_repository_cleanup() {
    // Arrange
    let _process_slot = process_slot();
    let request = fixture_request();
    let mut supervisor = supervisor("valid", SessionProfile::Reuse);
    supervisor
        .execute(&request)
        .expect("reuse request should start a live child");
    let root = supervisor.repository_path().to_path_buf();

    // Act
    drop(supervisor);

    // Assert
    assert!(!root.exists());
}

#[test]
fn concurrent_callers_preserve_one_shot_and_reuse_startup() {
    // Arrange
    let profiles = [SessionProfile::OneShot, SessionProfile::Reuse];

    // Act
    let results = std::thread::scope(|scope| {
        let workers = profiles.map(|profile| {
            scope.spawn(move || {
                let _process_slot = process_slot();
                let request = fixture_request();
                let mut child = supervisor("valid", profile);
                let first = child.execute(&request).expect("first request should start");
                let second = child
                    .execute(&request)
                    .expect("second request should start");
                (first.reset_epoch(), second.reset_epoch())
            })
        });
        workers.map(|worker| worker.join().expect("concurrent caller should finish"))
    });

    // Assert
    assert_eq!(results, [(1, 1), (1, 2)]);
    assert_eq!(
        HarnessLimits::phase2_default_v1().startup_timeout(),
        std::time::Duration::from_secs(5)
    );
}
