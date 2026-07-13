//! Bounded one-shot rigid-world execution on the shared oracle supervisor machinery.

use std::{io::Write, process::Stdio, time::Instant};

use liquidfun_test_protocol::{
    BuildIdentity, HarnessFailureKind, HarnessLimits, LastValidRecord, ProtocolVersion,
    RecordLimit, RequestId, RigidWorldRequestRecord, RigidWorldResultRecord,
    decode_rigid_world_result_jsonl, encode_jsonl, validate_rigid_world_result_against_request,
};
use serde::Deserialize;

use super::{
    ChildIo, HandshakingChild, IoEvent, IoWorkers, OracleExecutable, StderrSnapshot,
    complete_handshake, enforce_total_output, receive_with_output_precedence,
    reconcile_request_output,
};

/// Fully validated bounded one-shot rigid-world oracle output.
#[derive(Debug)]
pub struct CapturedRigidWorld {
    identity: BuildIdentity,
    result: RigidWorldResultRecord,
    response_bytes: Box<[u8]>,
    reset_epoch: u64,
    reset_verified: bool,
}

impl CapturedRigidWorld {
    /// Returns the validated oracle build identity from the startup handshake.
    #[must_use]
    pub const fn identity(&self) -> &BuildIdentity {
        &self.identity
    }

    /// Returns the strict declaration-validated rigid result.
    #[must_use]
    pub const fn result(&self) -> &RigidWorldResultRecord {
        &self.result
    }

    /// Returns exact handshake, result, and terminal JSONL bytes.
    #[must_use]
    pub fn response_bytes(&self) -> &[u8] {
        &self.response_bytes
    }

    /// Returns the exact one-shot reset epoch.
    #[must_use]
    pub const fn reset_epoch(&self) -> u64 {
        self.reset_epoch
    }

    /// Returns the child-provided terminal reset proof.
    #[must_use]
    pub const fn reset_verified(&self) -> bool {
        self.reset_verified
    }
}

/// Typed bounded-process failure for the rigid-world path.
#[derive(Debug, thiserror::Error)]
#[error("rigid-world oracle failed: {kind:?}")]
pub struct RigidWorldProcessError {
    kind: HarnessFailureKind,
    retained_stderr: Box<[u8]>,
    stderr_bytes: usize,
    child_killed: bool,
    child_reaped: bool,
}

impl RigidWorldProcessError {
    /// Returns the stable non-physics harness failure category.
    #[must_use]
    pub const fn kind(&self) -> HarnessFailureKind {
        self.kind
    }

    /// Returns bounded first/tail stderr evidence.
    #[must_use]
    pub fn retained_stderr(&self) -> &[u8] {
        &self.retained_stderr
    }

    /// Returns total observed stderr bytes.
    #[must_use]
    pub const fn stderr_bytes(&self) -> usize {
        self.stderr_bytes
    }

    /// Returns whether teardown sent a kill signal.
    #[must_use]
    pub const fn child_killed(&self) -> bool {
        self.child_killed
    }

    /// Returns whether every spawned child was reaped.
    #[must_use]
    pub const fn child_reaped(&self) -> bool {
        self.child_reaped
    }
}

/// Executes one rigid request with existing handshake, drain, timeout, poison, kill, and reap.
///
/// # Errors
///
/// Returns [`RigidWorldProcessError`] for startup, process, framing, provenance, declaration,
/// sequence, output-bound, terminal, or reset failures. Every spawned child is reaped.
pub fn execute_rigid_world_process(
    executable: &OracleExecutable,
    request: &RigidWorldRequestRecord,
    expected_oracle_revision: &str,
) -> Result<CapturedRigidWorld, RigidWorldProcessError> {
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
        let _ignored = child.kill();
        let _ignored = child.wait();
        failure_without_child(HarnessFailureKind::CppAdapterFailure)
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        let _ignored = child.kill();
        let _ignored = child.wait();
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
    let result = run_rigid_world(&mut ready, request, &limits);
    match result {
        Ok((result, response_bytes, end)) => {
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
            Ok(CapturedRigidWorld {
                identity: ready.identity,
                result,
                response_bytes: response_bytes.into_boxed_slice(),
                reset_epoch: end.reset_epoch,
                reset_verified: end.reset_verified,
            })
        }
        Err(kind) => Err(fail_and_reap(kind, ready.io)),
    }
}

fn run_rigid_world(
    ready: &mut super::ReadyChild,
    request: &RigidWorldRequestRecord,
    limits: &HarnessLimits,
) -> Result<(RigidWorldResultRecord, Vec<u8>, RawEnd), HarnessFailureKind> {
    let baseline = ready.output_boundary;
    let request_bytes = encode_jsonl(request, limits, RecordLimit::Input)
        .map_err(|_error| HarnessFailureKind::CppAdapterFailure)?;
    let stdin = ready
        .io
        .maybe_stdin
        .as_mut()
        .ok_or(HarnessFailureKind::UnexpectedEof)?;
    stdin
        .write_all(&request_bytes)
        .and_then(|()| stdin.flush())
        .map_err(|_error| HarnessFailureKind::UnexpectedEof)?;
    let deadline = Instant::now() + limits.request_timeout();
    let mut maybe_result = None;
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
                if maybe_result.is_none() {
                    let result = decode_rigid_world_result_jsonl(&bytes, limits)
                        .map_err(|_error| HarnessFailureKind::MalformedRecord)?;
                    validate_rigid_world_result_against_request(request, &result)
                        .map_err(|_error| HarnessFailureKind::SequenceViolation)?;
                    maybe_result = Some(result);
                    continue;
                }
                let end = decode_end(&bytes, limits)?;
                if end.protocol_version != ProtocolVersion::SUPPORTED
                    || end.request_id != *request.request_id()
                    || end.result_count != 1
                    || end.reset_epoch != 1
                    || !end.reset_verified
                {
                    return Err(HarnessFailureKind::AdapterResetFailure);
                }
                ready.output_boundary = reconcile_request_output(
                    &ready.io.workers,
                    deadline,
                    baseline,
                    limits,
                    Some(LastValidRecord::TraceEnd),
                )
                .map_err(|failure| failure.kind)?;
                let Some(result) = maybe_result else {
                    return Err(HarnessFailureKind::SequenceViolation);
                };
                return Ok((result, response_bytes, end));
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEnd {
    protocol_version: u32,
    record_kind: EndKind,
    request_id: RequestId,
    result_count: u32,
    reset_epoch: u64,
    reset_verified: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum EndKind {
    RigidWorldEnd,
}

fn decode_end(bytes: &[u8], limits: &HarnessLimits) -> Result<RawEnd, HarnessFailureKind> {
    if bytes.len() > limits.output_record_bytes() {
        return Err(HarnessFailureKind::RecordTooLarge);
    }
    let end: RawEnd =
        serde_json::from_slice(bytes).map_err(|_error| HarnessFailureKind::MalformedRecord)?;
    match end.record_kind {
        EndKind::RigidWorldEnd => {}
    }
    Ok(end)
}

fn fail_and_reap(kind: HarnessFailureKind, io: ChildIo) -> RigidWorldProcessError {
    failure_from_teardown(kind, io.shutdown(std::time::Duration::ZERO, true))
}

fn failure_from_teardown(
    kind: HarnessFailureKind,
    teardown: super::Teardown,
) -> RigidWorldProcessError {
    RigidWorldProcessError {
        kind,
        retained_stderr: teardown.stderr.retained.into_boxed_slice(),
        stderr_bytes: teardown.stderr.total_bytes,
        child_killed: teardown.was_killed,
        child_reaped: teardown.was_reaped,
    }
}

fn failure_without_child(kind: HarnessFailureKind) -> RigidWorldProcessError {
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
