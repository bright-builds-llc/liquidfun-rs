//! Bounded one-shot collision-probe execution on the shared oracle supervisor machinery.

use std::{io::Write, process::Stdio, time::Instant};

use liquidfun_test_protocol::{
    BuildIdentity, CollectionPolicy, CollisionProbeDiscreteValue, CollisionProbeHorizon,
    CollisionProbeNumericValue, CollisionProbeOperation, CollisionProbeRequestRecord,
    CollisionProbeResult, FloatBits, HarnessFailureKind, HarnessLimits, LastValidRecord,
    RecordLimit, RequestId, encode_jsonl,
};
use serde::Deserialize;

use super::{
    ChildIo, HandshakingChild, IoEvent, IoWorkers, OracleExecutable, StderrSnapshot,
    complete_handshake, enforce_total_output, receive_with_output_precedence,
    reconcile_request_output,
};

/// Fully validated bounded one-shot collision-probe output.
#[derive(Debug)]
pub struct CapturedCollisionProbe {
    identity: BuildIdentity,
    results: Box<[CollisionProbeResult]>,
    response_bytes: Box<[u8]>,
}

impl CapturedCollisionProbe {
    /// Returns the validated oracle build identity from the startup handshake.
    #[must_use]
    pub const fn identity(&self) -> &BuildIdentity {
        &self.identity
    }
    /// Returns ordered strictly decoded result records.
    #[must_use]
    pub fn results(&self) -> &[CollisionProbeResult] {
        &self.results
    }
    /// Returns exact handshake, result, and terminal JSONL bytes.
    #[must_use]
    pub fn response_bytes(&self) -> &[u8] {
        &self.response_bytes
    }
}

/// Typed bounded-process failure for the collision-probe path.
#[derive(Debug, thiserror::Error)]
#[error("collision-probe oracle failed: {kind:?}")]
pub struct CollisionProbeProcessError {
    kind: HarnessFailureKind,
    retained_stderr: Box<[u8]>,
    stderr_bytes: usize,
    child_killed: bool,
    child_reaped: bool,
}

impl CollisionProbeProcessError {
    /// Returns the stable harness failure category.
    #[must_use]
    pub const fn kind(&self) -> HarnessFailureKind {
        self.kind
    }
    /// Returns the bounded first/tail stderr evidence.
    #[must_use]
    pub fn retained_stderr(&self) -> &[u8] {
        &self.retained_stderr
    }
    /// Returns the total stderr byte count.
    #[must_use]
    pub const fn stderr_bytes(&self) -> usize {
        self.stderr_bytes
    }
    /// Returns whether teardown sent a kill signal.
    #[must_use]
    pub const fn child_killed(&self) -> bool {
        self.child_killed
    }
    /// Returns whether teardown reaped the child.
    #[must_use]
    pub const fn child_reaped(&self) -> bool {
        self.child_reaped
    }
}

/// Executes one collision-probe request with the existing concurrent bounded supervisor.
///
/// # Errors
///
/// Returns [`CollisionProbeProcessError`] for spawn, timeout, framing, identity, sequence,
/// reset, stderr, or child-exit failure. Every spawned child is reaped.
pub fn execute_collision_probe_process(
    executable: &OracleExecutable,
    request: &CollisionProbeRequestRecord,
    expected_oracle_revision: &str,
) -> Result<CapturedCollisionProbe, CollisionProbeProcessError> {
    let limits = HarnessLimits::phase2_default_v1();
    let mut command = executable.command();
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|_| failure_without_child(HarnessFailureKind::CppAdapterFailure))?;
    let maybe_stdin = child.stdin.take();
    let stdout = child.stdout.take().ok_or_else(|| {
        let _ = child.kill();
        let _ = child.wait();
        failure_without_child(HarnessFailureKind::CppAdapterFailure)
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        let _ = child.kill();
        let _ = child.wait();
        failure_without_child(HarnessFailureKind::CppAdapterFailure)
    })?;
    let workers = IoWorkers::spawn(
        stdout,
        stderr,
        limits.output_record_bytes(),
        limits.retained_stderr_bytes(),
    );
    let handshaking = HandshakingChild {
        io: ChildIo {
            child,
            maybe_stdin,
            workers,
        },
    };
    let mut ready = match complete_handshake(handshaking, expected_oracle_revision, &limits) {
        Ok(ready) => ready,
        Err((kind, child)) => return Err(fail_and_reap(kind, child.io)),
    };
    let result = run_collision_probe(&mut ready, request, &limits);
    match result {
        Ok((results, response_bytes)) => {
            ready.io.maybe_stdin.take();
            let teardown = ready.io.shutdown(limits.request_timeout(), false);
            if teardown.maybe_status.is_none_or(|status| !status.success())
                || teardown.stderr.total_bytes != 0
                || !teardown.was_reaped
            {
                return Err(failure_from_teardown(
                    HarnessFailureKind::ChildNonZeroExit,
                    teardown,
                ));
            }
            Ok(CapturedCollisionProbe {
                identity: ready.identity,
                results: results.into_boxed_slice(),
                response_bytes: response_bytes.into_boxed_slice(),
            })
        }
        Err(kind) => Err(fail_and_reap(kind, ready.io)),
    }
}

fn run_collision_probe(
    ready: &mut super::ReadyChild,
    request: &CollisionProbeRequestRecord,
    limits: &HarnessLimits,
) -> Result<(Vec<CollisionProbeResult>, Vec<u8>), HarnessFailureKind> {
    let baseline = ready.output_boundary;
    let request_bytes = encode_jsonl(request, limits, RecordLimit::Input)
        .map_err(|_| HarnessFailureKind::CppAdapterFailure)?;
    let stdin = ready
        .io
        .maybe_stdin
        .as_mut()
        .ok_or(HarnessFailureKind::UnexpectedEof)?;
    stdin
        .write_all(&request_bytes)
        .and_then(|()| stdin.flush())
        .map_err(|_| HarnessFailureKind::UnexpectedEof)?;
    let deadline = Instant::now() + limits.request_timeout();
    let expected_results = request.scenario().cases().len();
    let mut results = Vec::with_capacity(expected_results);
    let mut response_bytes = Vec::from(ready.handshake_jsonl.as_ref());
    let mut response_total = 0_usize;
    loop {
        let event = receive_with_output_precedence(
            &ready.io.workers,
            deadline,
            HarnessFailureKind::RequestTimeout,
            baseline,
            limits,
        )?;
        match event {
            IoEvent::StdoutRecord(bytes) => {
                response_total = response_total.saturating_add(bytes.len());
                if response_total > limits.complete_trace_bytes() {
                    return Err(HarnessFailureKind::TraceTooLarge);
                }
                response_bytes.extend_from_slice(&bytes);
                if results.len() < expected_results {
                    results.push(decode_result(&bytes, limits)?);
                    continue;
                }
                let end = decode_end(&bytes, limits)?;
                if end.request_id != *request.request_id()
                    || usize::try_from(end.result_count).ok() != Some(results.len())
                    || end.reset_epoch != 1
                {
                    return Err(HarnessFailureKind::SequenceViolation);
                }
                ready.output_boundary = reconcile_request_output(
                    &ready.io.workers,
                    deadline,
                    baseline,
                    limits,
                    Some(LastValidRecord::TraceEnd),
                )
                .map_err(|failure| failure.kind)?;
                return Ok((results, response_bytes));
            }
            IoEvent::OutputProgress(total) => enforce_total_output(total, baseline, limits)?,
            IoEvent::SanitizerDetected => return Err(HarnessFailureKind::SanitizerReport),
            IoEvent::StdoutRecordTooLarge => return Err(HarnessFailureKind::RecordTooLarge),
            IoEvent::StdoutPartial => return Err(HarnessFailureKind::PartialRecord),
            IoEvent::StdoutEof | IoEvent::ReadFailure => {
                return Err(HarnessFailureKind::UnexpectedEof);
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawNumeric {
    field: Box<str>,
    bits: FloatBits,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDiscrete {
    field: Box<str>,
    value: Box<str>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawResult {
    case_id: Box<str>,
    operation: CollisionProbeOperation,
    policy_path: Box<str>,
    horizon: CollisionProbeHorizon,
    collection_policy: CollectionPolicy,
    numeric: Vec<RawNumeric>,
    discrete: Vec<RawDiscrete>,
    payload_ids: Vec<u32>,
}

fn decode_result(
    bytes: &[u8],
    limits: &HarnessLimits,
) -> Result<CollisionProbeResult, HarnessFailureKind> {
    if bytes.len() > limits.output_record_bytes() {
        return Err(HarnessFailureKind::RecordTooLarge);
    }
    let raw: RawResult =
        serde_json::from_slice(bytes).map_err(|_error| HarnessFailureKind::MalformedRecord)?;
    if raw.policy_path.as_ref() != raw.operation.policy_path()
        || raw.horizon != raw.operation.expected_horizon()
        || raw.collection_policy != raw.operation.expected_collection_policy()
    {
        return Err(HarnessFailureKind::SequenceViolation);
    }
    CollisionProbeResult::new(
        raw.case_id,
        raw.operation,
        raw.numeric
            .into_iter()
            .map(|value| CollisionProbeNumericValue::new(value.field, value.bits))
            .collect(),
        raw.discrete
            .into_iter()
            .map(|value| CollisionProbeDiscreteValue::new(value.field, value.value))
            .collect(),
        raw.payload_ids,
    )
    .map_err(|_| HarnessFailureKind::SequenceViolation)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEnd {
    record_kind: EndKind,
    request_id: RequestId,
    result_count: u32,
    reset_epoch: u64,
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum EndKind {
    CollisionProbeEnd,
}

fn decode_end(bytes: &[u8], limits: &HarnessLimits) -> Result<RawEnd, HarnessFailureKind> {
    if bytes.len() > limits.output_record_bytes() {
        return Err(HarnessFailureKind::RecordTooLarge);
    }
    let end: RawEnd =
        serde_json::from_slice(bytes).map_err(|_error| HarnessFailureKind::MalformedRecord)?;
    let _ = end.record_kind;
    Ok(end)
}

fn fail_and_reap(kind: HarnessFailureKind, io: ChildIo) -> CollisionProbeProcessError {
    failure_from_teardown(kind, io.shutdown(std::time::Duration::ZERO, true))
}
fn failure_from_teardown(
    kind: HarnessFailureKind,
    teardown: super::Teardown,
) -> CollisionProbeProcessError {
    CollisionProbeProcessError {
        kind,
        retained_stderr: teardown.stderr.retained.into_boxed_slice(),
        stderr_bytes: teardown.stderr.total_bytes,
        child_killed: teardown.was_killed,
        child_reaped: teardown.was_reaped,
    }
}
fn failure_without_child(kind: HarnessFailureKind) -> CollisionProbeProcessError {
    failure_from_teardown(
        kind,
        super::Teardown {
            maybe_status: None,
            stderr: StderrSnapshot::default(),
            was_killed: false,
            was_reaped: false,
            total_output: 0,
        },
    )
}
