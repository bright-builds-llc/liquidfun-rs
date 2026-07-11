//! Bounded one-shot math-probe execution on the shared oracle supervisor machinery.

use std::{io::Write, process::Stdio, time::Instant};

use liquidfun_test_protocol::{
    BuildIdentity, HarnessFailureKind, HarnessLimits, LastValidRecord, MathProbeRequestRecord,
    MathProbeResult, RecordLimit, decode_math_probe_end_jsonl, decode_math_probe_result_jsonl,
    encode_jsonl,
};

use super::{
    ChildIo, HandshakingChild, IoEvent, IoWorkers, OracleExecutable, StderrSnapshot,
    classify_trace_decode, complete_handshake, enforce_total_output,
    receive_with_output_precedence, reconcile_request_output,
};

/// Fully validated bounded one-shot math-probe output.
#[derive(Debug)]
pub struct CapturedMathProbe {
    identity: BuildIdentity,
    results: Box<[MathProbeResult]>,
    response_bytes: Box<[u8]>,
}

impl CapturedMathProbe {
    /// Returns the validated oracle build identity from the startup handshake.
    #[must_use]
    pub const fn identity(&self) -> &BuildIdentity {
        &self.identity
    }

    /// Returns ordered strictly decoded result records.
    #[must_use]
    pub fn results(&self) -> &[MathProbeResult] {
        &self.results
    }

    /// Returns exact handshake, result, and terminal JSONL bytes.
    #[must_use]
    pub fn response_bytes(&self) -> &[u8] {
        &self.response_bytes
    }
}

/// Typed bounded-process failure for the private math-probe path.
#[derive(Debug, thiserror::Error)]
#[error("math-probe oracle failed: {kind:?}")]
pub struct MathProbeProcessError {
    kind: HarnessFailureKind,
    retained_stderr: Box<[u8]>,
    stderr_bytes: usize,
    child_killed: bool,
    child_reaped: bool,
}

impl MathProbeProcessError {
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

    /// Returns the total stderr bytes observed by the concurrent drain.
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

/// Executes one math-probe request with the existing concurrent bounded supervisor primitives.
///
/// # Errors
///
/// Returns [`MathProbeProcessError`] for spawn, timeout, framing, resource, identity, sequence,
/// reset, stderr, or child-exit failure. Every spawned child is killed when necessary and reaped.
pub fn execute_math_probe_process(
    executable: &OracleExecutable,
    request: &MathProbeRequestRecord,
    expected_oracle_revision: &str,
) -> Result<CapturedMathProbe, MathProbeProcessError> {
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

    let result = run_math_probe(&mut ready, request, &limits);
    match result {
        Ok((results, response_bytes)) => {
            ready.io.maybe_stdin.take();
            let teardown = ready.io.shutdown(limits.request_timeout(), false);
            let clean_exit = teardown.maybe_status.is_some_and(|status| status.success())
                && teardown.stderr.total_bytes == 0
                && teardown.was_reaped;
            if !clean_exit {
                return Err(failure_from_teardown(
                    HarnessFailureKind::ChildNonZeroExit,
                    teardown,
                ));
            }
            Ok(CapturedMathProbe {
                identity: ready.identity,
                results: results.into_boxed_slice(),
                response_bytes: response_bytes.into_boxed_slice(),
            })
        }
        Err(kind) => Err(fail_and_reap(kind, ready.io)),
    }
}

fn run_math_probe(
    ready: &mut super::ReadyChild,
    request: &MathProbeRequestRecord,
    limits: &HarnessLimits,
) -> Result<(Vec<MathProbeResult>, Vec<u8>), HarnessFailureKind> {
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
                    let result = decode_math_probe_result_jsonl(&bytes, limits)
                        .map_err(|error| classify_trace_decode(&error))?;
                    results.push(result);
                    continue;
                }
                let end = decode_math_probe_end_jsonl(&bytes, limits)
                    .map_err(|error| classify_trace_decode(&error))?;
                if end.request_id() != request.request_id()
                    || usize::try_from(end.result_count()).ok() != Some(results.len())
                    || end.reset_epoch() != 1
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

fn fail_and_reap(kind: HarnessFailureKind, io: ChildIo) -> MathProbeProcessError {
    failure_from_teardown(kind, io.shutdown(std::time::Duration::ZERO, true))
}

fn failure_from_teardown(
    kind: HarnessFailureKind,
    teardown: super::Teardown,
) -> MathProbeProcessError {
    MathProbeProcessError {
        kind,
        retained_stderr: teardown.stderr.retained.into_boxed_slice(),
        stderr_bytes: teardown.stderr.total_bytes,
        child_killed: teardown.was_killed,
        child_reaped: teardown.was_reaped,
    }
}

fn failure_without_child(kind: HarnessFailureKind) -> MathProbeProcessError {
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
