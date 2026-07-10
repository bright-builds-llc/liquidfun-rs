//! Process teardown classification and bounded failure-evidence assembly.

use std::{process::ExitStatus, time::Duration};

use liquidfun_test_protocol::{
    BuildIdentity, CodecErrorKind, HarnessFailure, HarnessFailureEvidence, HarnessFailureKind,
    HarnessLimits, LastValidRecord, ScenarioRequestRecord, Sha256Hex, StderrEvidence,
    TraceDecodeError,
};
use sha2::{Digest, Sha256};

use super::Teardown;

pub(super) fn classify_handshake_decode(error: TraceDecodeError) -> HarnessFailureKind {
    match error {
        TraceDecodeError::Codec(codec) if codec.kind() == CodecErrorKind::UnsupportedVersion => {
            HarnessFailureKind::UnsupportedVersion
        }
        TraceDecodeError::Codec(codec) if codec.kind() == CodecErrorKind::RecordTooLarge => {
            HarnessFailureKind::RecordTooLarge
        }
        TraceDecodeError::Validation(validation) => validation.kind(),
        TraceDecodeError::Codec(_) => HarnessFailureKind::HandshakeMalformed,
    }
}

pub(super) fn classify_trace_decode(error: &TraceDecodeError) -> HarnessFailureKind {
    match error {
        TraceDecodeError::Validation(validation) => validation.kind(),
        TraceDecodeError::Codec(codec) => match codec.kind() {
            CodecErrorKind::UnknownRecordKind => HarnessFailureKind::UnknownRecordKind,
            CodecErrorKind::RecordTooLarge => HarnessFailureKind::RecordTooLarge,
            CodecErrorKind::PartialRecord => HarnessFailureKind::PartialRecord,
            CodecErrorKind::UnsupportedVersion => HarnessFailureKind::UnsupportedVersion,
            _ => HarnessFailureKind::MalformedRecord,
        },
    }
}

pub(super) fn successful_teardown_failure(teardown: &Teardown) -> Option<HarnessFailureKind> {
    if contains_sanitizer_marker(&teardown.stderr.retained) {
        return Some(HarnessFailureKind::SanitizerReport);
    }
    let status = teardown.maybe_status.as_ref()?;
    if status.success() {
        return None;
    }
    Some(classify_exit(*status))
}

pub(super) fn classify_poison(
    initial: HarnessFailureKind,
    teardown: &Teardown,
) -> HarnessFailureKind {
    if contains_sanitizer_marker(&teardown.stderr.retained) {
        return HarnessFailureKind::SanitizerReport;
    }
    if initial != HarnessFailureKind::UnexpectedEof {
        return initial;
    }
    if teardown.stderr.retained.starts_with(b"scenario rejected:") {
        return HarnessFailureKind::ScenarioRejected;
    }
    if teardown
        .stderr
        .retained
        .starts_with(b"cpp adapter failure:")
    {
        return HarnessFailureKind::CppAdapterFailure;
    }
    let Some(status) = teardown.maybe_status.as_ref() else {
        return initial;
    };
    if status.success() {
        return initial;
    }
    classify_exit(*status)
}

#[allow(
    clippy::too_many_arguments,
    reason = "failure evidence has fixed request, process, lifecycle, and limit inputs"
)]
pub(super) fn build_failure(
    kind: HarnessFailureKind,
    request: &ScenarioRequestRecord,
    maybe_identity: Option<&BuildIdentity>,
    maybe_last_record: Option<LastValidRecord>,
    elapsed: Duration,
    teardown: Teardown,
    limits: &HarnessLimits,
) -> HarnessFailure {
    let retained = if teardown.stderr.retained.len() > limits.retained_stderr_bytes() {
        teardown.stderr.retained[..limits.retained_stderr_bytes()].to_vec()
    } else {
        teardown.stderr.retained
    };
    let total_bytes = teardown.stderr.total_bytes.max(retained.len());
    let stderr = StderrEvidence::new(retained, total_bytes, limits)
        .expect("bounded drain construction guarantees valid stderr evidence");
    let request_bytes = serde_json::to_vec(request)
        .expect("validated request serialization is infallible for hash evidence");
    let scenario_bytes = serde_json::to_vec(request.scenario())
        .expect("validated scenario serialization is infallible for hash evidence");
    let mut evidence = HarnessFailureEvidence::new(
        elapsed,
        stderr,
        teardown.was_killed,
        teardown.was_reaped,
        limits,
    )
    .with_request(
        request.request_id().clone(),
        Sha256Hex::from_digest(Sha256::digest(request_bytes).into()),
        Sha256Hex::from_digest(Sha256::digest(scenario_bytes).into()),
    );
    if let Some(identity) = maybe_identity {
        evidence = evidence.with_session_identity(identity.identity_sha256().clone());
    }
    if let Some(status) = teardown.maybe_status.as_ref() {
        evidence = evidence.with_exit_status(exit_evidence_code(*status));
    }
    if let Some(last_record) = maybe_last_record {
        evidence = evidence.with_last_valid_record(last_record);
    }
    HarnessFailure::new(kind, evidence)
}

fn contains_sanitizer_marker(bytes: &[u8]) -> bool {
    [
        b"ERROR: AddressSanitizer".as_slice(),
        b"SUMMARY: AddressSanitizer".as_slice(),
        b"UndefinedBehaviorSanitizer".as_slice(),
        b"runtime error:".as_slice(),
    ]
    .iter()
    .any(|marker| bytes.windows(marker.len()).any(|window| window == *marker))
}

fn classify_exit(status: ExitStatus) -> HarnessFailureKind {
    if exit_signal(status).is_some() {
        return HarnessFailureKind::ChildSignaled;
    }
    HarnessFailureKind::ChildNonZeroExit
}

#[cfg(unix)]
fn exit_signal(status: ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    status.signal()
}

#[cfg(not(unix))]
fn exit_signal(_status: ExitStatus) -> Option<i32> {
    None
}

fn exit_evidence_code(status: ExitStatus) -> i32 {
    if let Some(code) = status.code() {
        return code;
    }
    exit_signal(status).map_or(-1, |signal| -signal)
}
