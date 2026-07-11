//! Confined sequential process supervision for the private C++ oracle.

use std::{
    io::{self, Write},
    process::{Child, ChildStdin, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};

use liquidfun_test_protocol::{
    BuildIdentity, HarnessFailure, HarnessFailureKind, HarnessLimits, LastValidRecord,
    ProtocolSessionValidator, RecordLimit, ScenarioRequestRecord, TraceRecord, TraceValidator,
    ValidatedTrace, decode_handshake_jsonl, decode_trace_record_jsonl, encode_jsonl,
};

mod capture;
mod executable;
mod failure;
mod math_probe;
mod profile;
mod stdio;

pub use capture::CapturedOracleTrace;
pub use executable::{OracleExecutable, OracleExecutableError, OraclePreset};
use failure::{
    build_failure, classify_handshake_decode, classify_poison, classify_trace_decode,
    successful_teardown_failure,
};
pub use math_probe::{CapturedMathProbe, MathProbeProcessError, execute_math_probe_process};
pub use profile::SessionProfile;
use stdio::{IoEvent, IoWorkers, StderrSnapshot};

impl OracleExecutable {
    fn command(&self) -> Command {
        Command::new(&self.resolved)
    }
}

enum SessionState {
    Dormant,
    Starting,
    Handshaking(HandshakingChild),
    Ready(ReadyChild),
    InFlight(ReadyChild),
    Poisoned {
        io: ChildIo,
        maybe_identity: Option<BuildIdentity>,
    },
    Reaped,
    Exited,
}

struct HandshakingChild {
    io: ChildIo,
}

struct ReadyChild {
    io: ChildIo,
    identity: BuildIdentity,
    handshake_jsonl: Box<[u8]>,
    requests: usize,
    output_boundary: usize,
    last_request_baseline: usize,
}

struct ChildIo {
    child: Child,
    maybe_stdin: Option<ChildStdin>,
    workers: IoWorkers,
}

struct Teardown {
    maybe_status: Option<ExitStatus>,
    stderr: StderrSnapshot,
    was_killed: bool,
    was_reaped: bool,
    total_output: usize,
}

struct RequestFailure {
    kind: HarnessFailureKind,
    maybe_last_record: Option<LastValidRecord>,
}

/// One controlling-thread supervisor with exactly one sequential request in flight.
pub struct OracleSupervisor {
    executable: OracleExecutable,
    profile: SessionProfile,
    limits: HarnessLimits,
    expected_oracle_revision: Box<str>,
    state: SessionState,
    process_generation: u64,
    requests_in_current_process: usize,
}

impl OracleSupervisor {
    /// Creates a lazy supervisor for a confined executable and reviewed profile.
    #[must_use]
    pub fn new(
        executable: OracleExecutable,
        profile: SessionProfile,
        expected_oracle_revision: impl Into<Box<str>>,
    ) -> Self {
        Self {
            executable,
            profile,
            limits: profile.limits(),
            expected_oracle_revision: expected_oracle_revision.into(),
            state: SessionState::Dormant,
            process_generation: 0,
            requests_in_current_process: 0,
        }
    }

    /// Executes one request synchronously, poisoning and reaping on every harness failure.
    ///
    /// # Errors
    ///
    /// Returns a typed [`HarnessFailure`] for startup, process, framing, resource, identity,
    /// provenance, sequence, or reset failure. Deterministic requests are never retried.
    pub fn execute(
        &mut self,
        request: &ScenarioRequestRecord,
    ) -> Result<ValidatedTrace, HarnessFailure> {
        self.execute_captured(request)
            .map(CapturedOracleTrace::into_trace)
    }

    /// Executes one request and retains the exact validated oracle JSONL for reviewed staging.
    ///
    /// # Errors
    ///
    /// Returns the same typed harness failures as [`Self::execute`].
    pub fn execute_captured(
        &mut self,
        request: &ScenarioRequestRecord,
    ) -> Result<CapturedOracleTrace, HarnessFailure> {
        let started = Instant::now();
        self.prepare_ready(request, started)?;
        let current = std::mem::replace(&mut self.state, SessionState::Reaped);
        let SessionState::Ready(mut ready) = current else {
            return Err(self.failure_without_child(
                HarnessFailureKind::CppAdapterFailure,
                request,
                started.elapsed(),
            ));
        };
        self.state = SessionState::InFlight(ready);
        let result = {
            let SessionState::InFlight(active) = &mut self.state else {
                return Err(self.failure_without_child(
                    HarnessFailureKind::CppAdapterFailure,
                    request,
                    started.elapsed(),
                ));
            };
            run_request(active, request, &self.limits)
        };

        match result {
            Ok(captured) => {
                ready = take_ready(&mut self.state);
                ready.requests = ready.requests.saturating_add(1);
                self.requests_in_current_process = ready.requests;
                if self.profile.keeps_process() {
                    self.state = SessionState::Ready(ready);
                    return Ok(captured);
                }
                let teardown = ready.io.shutdown(self.limits.request_timeout(), false);
                self.state = SessionState::Exited;
                let maybe_failure_kind = enforce_total_output(
                    teardown.total_output,
                    ready.last_request_baseline,
                    &self.limits,
                )
                .err()
                .or_else(|| successful_teardown_failure(&teardown));
                if let Some(kind) = maybe_failure_kind {
                    return Err(build_failure(
                        kind,
                        request,
                        Some(&ready.identity),
                        None,
                        started.elapsed(),
                        teardown,
                        &self.limits,
                    ));
                }
                Ok(captured)
            }
            Err(failure) => Err(self.poison(
                failure.kind,
                failure.maybe_last_record,
                request,
                started.elapsed(),
            )),
        }
    }

    /// Returns the number of child processes started by this supervisor.
    #[must_use]
    pub const fn process_generation(&self) -> u64 {
        self.process_generation
    }

    /// Returns successful requests handled by the current or most recently exited child.
    #[must_use]
    pub const fn requests_in_current_process(&self) -> usize {
        self.requests_in_current_process
    }

    fn prepare_ready(
        &mut self,
        request: &ScenarioRequestRecord,
        started: Instant,
    ) -> Result<(), HarnessFailure> {
        if matches!(&self.state, SessionState::Ready(ready) if ready.requests >= self.limits.request_budget())
        {
            let ready = take_ready(&mut self.state);
            let identity = ready.identity.clone();
            let teardown = ready.io.shutdown(self.limits.request_timeout(), false);
            self.state = SessionState::Exited;
            let maybe_failure_kind = enforce_total_output(
                teardown.total_output,
                ready.last_request_baseline,
                &self.limits,
            )
            .err()
            .or_else(|| successful_teardown_failure(&teardown));
            if let Some(kind) = maybe_failure_kind {
                return Err(build_failure(
                    kind,
                    request,
                    Some(&identity),
                    None,
                    started.elapsed(),
                    teardown,
                    &self.limits,
                ));
            }
        }
        if matches!(self.state, SessionState::Ready(_)) {
            return Ok(());
        }

        self.state = SessionState::Starting;
        self.process_generation = self.process_generation.saturating_add(1);
        self.requests_in_current_process = 0;
        let Ok(child) = self.spawn_child() else {
            self.state = SessionState::Reaped;
            return Err(self.failure_without_child(
                HarnessFailureKind::CppAdapterFailure,
                request,
                started.elapsed(),
            ));
        };
        self.state = SessionState::Handshaking(child);
        let handshaking = std::mem::replace(&mut self.state, SessionState::Reaped);
        let handshake_result = {
            let SessionState::Handshaking(child) = handshaking else {
                return Err(self.failure_without_child(
                    HarnessFailureKind::HandshakeMalformed,
                    request,
                    started.elapsed(),
                ));
            };
            complete_handshake(child, &self.expected_oracle_revision, &self.limits)
        };
        match handshake_result {
            Ok(ready) => {
                self.state = SessionState::Ready(ready);
                Ok(())
            }
            Err((kind, child)) => {
                self.state = SessionState::Handshaking(child);
                Err(self.poison(kind, None, request, started.elapsed()))
            }
        }
    }

    fn spawn_child(&self) -> io::Result<HandshakingChild> {
        let mut command = self.executable.command();
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if self.profile == SessionProfile::Sanitizer
            || self.executable.preset == OraclePreset::AsanUbsan
        {
            command
                .env("ASAN_OPTIONS", "abort_on_error=1:halt_on_error=1")
                .env("UBSAN_OPTIONS", "halt_on_error=1:print_stacktrace=1");
        }
        let mut child = command.spawn()?;
        let maybe_stdin = child.stdin.take();
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("child stdout was not piped"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| io::Error::other("child stderr was not piped"))?;
        let workers = IoWorkers::spawn(
            stdout,
            stderr,
            self.limits.output_record_bytes(),
            self.limits.retained_stderr_bytes(),
        );
        Ok(HandshakingChild {
            io: ChildIo {
                child,
                maybe_stdin,
                workers,
            },
        })
    }

    fn poison(
        &mut self,
        kind: HarnessFailureKind,
        maybe_last_record: Option<LastValidRecord>,
        request: &ScenarioRequestRecord,
        elapsed: Duration,
    ) -> HarnessFailure {
        let current = std::mem::replace(&mut self.state, SessionState::Reaped);
        let (io, maybe_identity) = match current {
            SessionState::Handshaking(child) => (child.io, None),
            SessionState::Ready(ready) | SessionState::InFlight(ready) => {
                (ready.io, Some(ready.identity))
            }
            SessionState::Poisoned { io, maybe_identity } => (io, maybe_identity),
            _ => return self.failure_without_child(kind, request, elapsed),
        };
        self.state = SessionState::Poisoned { io, maybe_identity };
        let poisoned = std::mem::replace(&mut self.state, SessionState::Reaped);
        let SessionState::Poisoned { io, maybe_identity } = poisoned else {
            return self.failure_without_child(kind, request, elapsed);
        };
        let teardown = io.shutdown(Duration::ZERO, true);
        let classified = classify_poison(kind, &teardown);
        build_failure(
            classified,
            request,
            maybe_identity.as_ref(),
            maybe_last_record,
            elapsed,
            teardown,
            &self.limits,
        )
    }

    fn failure_without_child(
        &self,
        kind: HarnessFailureKind,
        request: &ScenarioRequestRecord,
        elapsed: Duration,
    ) -> HarnessFailure {
        build_failure(
            kind,
            request,
            None,
            None,
            elapsed,
            Teardown {
                maybe_status: None,
                stderr: StderrSnapshot::default(),
                was_killed: false,
                was_reaped: false,
                total_output: 0,
            },
            &self.limits,
        )
    }
}

impl Drop for OracleSupervisor {
    fn drop(&mut self) {
        let current = std::mem::replace(&mut self.state, SessionState::Reaped);
        match current {
            SessionState::Handshaking(child) => {
                let _teardown = child.io.shutdown(Duration::ZERO, true);
            }
            SessionState::Ready(ready) | SessionState::InFlight(ready) => {
                let _teardown = ready.io.shutdown(Duration::ZERO, true);
            }
            SessionState::Poisoned { io, .. } => {
                let _teardown = io.shutdown(Duration::ZERO, true);
            }
            _ => {}
        }
    }
}

fn complete_handshake(
    child: HandshakingChild,
    expected_oracle_revision: &str,
    limits: &HarnessLimits,
) -> Result<ReadyChild, (HarnessFailureKind, HandshakingChild)> {
    let deadline = Instant::now() + limits.startup_timeout();
    let baseline = child.io.workers.total_output();
    let identity_result = (|| {
        loop {
            let event = receive_with_output_precedence(
                &child.io.workers,
                deadline,
                HarnessFailureKind::StartupTimeout,
                baseline,
                limits,
            )?;
            match event {
                IoEvent::StdoutRecord(bytes) => {
                    let handshake = decode_handshake_jsonl(&bytes, limits)
                        .map_err(classify_handshake_decode)?;
                    let mut validator = ProtocolSessionValidator::new(expected_oracle_revision);
                    validator
                        .accept_handshake(handshake)
                        .map_err(|error| error.kind())?;
                    let identity = validator
                        .maybe_build_identity()
                        .cloned()
                        .ok_or(HarnessFailureKind::HandshakeMalformed)?;
                    return Ok((identity, bytes));
                }
                IoEvent::OutputProgress(total) => {
                    enforce_total_output(total, baseline, limits)?;
                }
                IoEvent::SanitizerDetected => {
                    return Err(HarnessFailureKind::SanitizerReport);
                }
                IoEvent::StdoutRecordTooLarge => {
                    return Err(HarnessFailureKind::RecordTooLarge);
                }
                IoEvent::StdoutPartial => return Err(HarnessFailureKind::PartialRecord),
                IoEvent::StdoutEof | IoEvent::ReadFailure => {
                    return Err(HarnessFailureKind::UnexpectedEof);
                }
            }
        }
    })();
    match identity_result {
        Ok((identity, handshake_jsonl)) => Ok(ReadyChild {
            output_boundary: child.io.workers.total_output(),
            last_request_baseline: child.io.workers.total_output(),
            io: child.io,
            identity,
            handshake_jsonl: handshake_jsonl.into_boxed_slice(),
            requests: 0,
        }),
        Err(kind) => Err((kind, child)),
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "the linear receive loop makes one in-flight protocol state machine auditable"
)]
fn run_request(
    ready: &mut ReadyChild,
    request: &ScenarioRequestRecord,
    limits: &HarnessLimits,
) -> Result<CapturedOracleTrace, RequestFailure> {
    let baseline = ready.output_boundary;
    ready.last_request_baseline = baseline;
    let bytes = encode_jsonl(request, limits, RecordLimit::Input).map_err(|_| RequestFailure {
        kind: HarnessFailureKind::CppAdapterFailure,
        maybe_last_record: Some(LastValidRecord::Handshake),
    })?;
    let Some(stdin) = ready.io.maybe_stdin.as_mut() else {
        return Err(RequestFailure {
            kind: HarnessFailureKind::UnexpectedEof,
            maybe_last_record: Some(LastValidRecord::Handshake),
        });
    };
    stdin
        .write_all(&bytes)
        .and_then(|()| stdin.flush())
        .map_err(|_| RequestFailure {
            kind: HarnessFailureKind::UnexpectedEof,
            maybe_last_record: Some(LastValidRecord::Handshake),
        })?;

    let deadline = Instant::now() + limits.request_timeout();
    let mut records = Vec::new();
    let mut jsonl = Vec::from(ready.handshake_jsonl.as_ref());
    let mut trace_bytes = 0_usize;
    let mut stream_state = 0_u8;
    let mut maybe_last_record = Some(LastValidRecord::Handshake);
    loop {
        if ready.io.workers.sanitizer_detected() {
            return Err(RequestFailure {
                kind: HarnessFailureKind::SanitizerReport,
                maybe_last_record,
            });
        }
        let event = receive_with_output_precedence(
            &ready.io.workers,
            deadline,
            HarnessFailureKind::RequestTimeout,
            baseline,
            limits,
        )
        .map_err(|kind| RequestFailure {
            kind,
            maybe_last_record,
        })?;
        match event {
            IoEvent::StdoutRecord(bytes) => {
                trace_bytes = trace_bytes.saturating_add(bytes.len());
                if trace_bytes > limits.complete_trace_bytes() {
                    return Err(RequestFailure {
                        kind: HarnessFailureKind::TraceTooLarge,
                        maybe_last_record,
                    });
                }
                jsonl.extend_from_slice(&bytes);
                let record =
                    decode_trace_record_jsonl(&bytes, limits).map_err(|error| RequestFailure {
                        kind: classify_trace_decode(&error),
                        maybe_last_record,
                    })?;
                let last = match &record {
                    TraceRecord::Begin(_) if stream_state == 0 => {
                        stream_state = 1;
                        LastValidRecord::TraceBegin
                    }
                    TraceRecord::Checkpoint(_) if stream_state == 1 => LastValidRecord::Checkpoint,
                    TraceRecord::End(_) if stream_state == 1 => {
                        stream_state = 2;
                        LastValidRecord::TraceEnd
                    }
                    _ => {
                        return Err(RequestFailure {
                            kind: HarnessFailureKind::SequenceViolation,
                            maybe_last_record,
                        });
                    }
                };
                maybe_last_record = Some(last);
                records.push(record);
                if stream_state == 2 {
                    let trace = TraceValidator::validate(
                        request,
                        &ready.identity,
                        u64::try_from(ready.requests.saturating_add(1)).map_err(|_| {
                            RequestFailure {
                                kind: HarnessFailureKind::AdapterResetFailure,
                                maybe_last_record,
                            }
                        })?,
                        records,
                        limits,
                    )
                    .map_err(|error| RequestFailure {
                        kind: error.kind(),
                        maybe_last_record,
                    })?;
                    ready.output_boundary = reconcile_request_output(
                        &ready.io.workers,
                        deadline,
                        baseline,
                        limits,
                        maybe_last_record,
                    )?;
                    return Ok(CapturedOracleTrace {
                        trace,
                        jsonl: jsonl.into_boxed_slice(),
                    });
                }
            }
            IoEvent::OutputProgress(total) => {
                enforce_total_output(total, baseline, limits).map_err(|kind| RequestFailure {
                    kind,
                    maybe_last_record,
                })?;
            }
            IoEvent::SanitizerDetected => {
                return Err(RequestFailure {
                    kind: HarnessFailureKind::SanitizerReport,
                    maybe_last_record,
                });
            }
            IoEvent::StdoutRecordTooLarge => {
                return Err(RequestFailure {
                    kind: HarnessFailureKind::RecordTooLarge,
                    maybe_last_record,
                });
            }
            IoEvent::StdoutPartial => {
                return Err(RequestFailure {
                    kind: HarnessFailureKind::PartialRecord,
                    maybe_last_record,
                });
            }
            IoEvent::StdoutEof | IoEvent::ReadFailure => {
                return Err(RequestFailure {
                    kind: HarnessFailureKind::UnexpectedEof,
                    maybe_last_record,
                });
            }
        }
    }
}

fn reconcile_request_output(
    workers: &IoWorkers,
    request_deadline: Instant,
    baseline: usize,
    limits: &HarnessLimits,
    maybe_last_record: Option<LastValidRecord>,
) -> Result<usize, RequestFailure> {
    const QUIET_PERIOD: Duration = Duration::from_millis(50);
    loop {
        let quiet_deadline = (Instant::now() + QUIET_PERIOD).min(request_deadline);
        let maybe_event = workers
            .receive_optional_until(quiet_deadline)
            .map_err(|kind| RequestFailure {
                kind,
                maybe_last_record,
            })?;
        let Some(event) = maybe_event else {
            let total = workers.total_output();
            enforce_total_output(total, baseline, limits).map_err(|kind| RequestFailure {
                kind,
                maybe_last_record,
            })?;
            if Instant::now() >= request_deadline {
                return Err(RequestFailure {
                    kind: HarnessFailureKind::RequestTimeout,
                    maybe_last_record,
                });
            }
            return Ok(total);
        };
        match event {
            IoEvent::OutputProgress(total) => {
                enforce_total_output(total, baseline, limits).map_err(|kind| RequestFailure {
                    kind,
                    maybe_last_record,
                })?;
            }
            IoEvent::SanitizerDetected => {
                return Err(RequestFailure {
                    kind: HarnessFailureKind::SanitizerReport,
                    maybe_last_record,
                });
            }
            IoEvent::StdoutRecord(_) => {
                return Err(RequestFailure {
                    kind: HarnessFailureKind::SequenceViolation,
                    maybe_last_record,
                });
            }
            IoEvent::StdoutRecordTooLarge => {
                return Err(RequestFailure {
                    kind: HarnessFailureKind::RecordTooLarge,
                    maybe_last_record,
                });
            }
            IoEvent::StdoutPartial => {
                return Err(RequestFailure {
                    kind: HarnessFailureKind::PartialRecord,
                    maybe_last_record,
                });
            }
            IoEvent::StdoutEof | IoEvent::ReadFailure => {
                return Err(RequestFailure {
                    kind: HarnessFailureKind::UnexpectedEof,
                    maybe_last_record,
                });
            }
        }
    }
}

impl ChildIo {
    fn shutdown(mut self, grace: Duration, force: bool) -> Teardown {
        self.maybe_stdin.take();
        let mut was_killed = false;
        let deadline = Instant::now() + grace;
        let mut maybe_status = None;
        if !force {
            loop {
                match self.child.try_wait() {
                    Ok(Some(status)) => {
                        maybe_status = Some(status);
                        break;
                    }
                    Ok(None) if Instant::now() < deadline => {
                        thread::sleep(Duration::from_millis(2));
                    }
                    _ => break,
                }
            }
        }
        if maybe_status.is_none() {
            if self.child.kill().is_ok() {
                was_killed = true;
            }
            maybe_status = self.child.wait().ok();
        }
        let was_reaped = maybe_status.is_some();
        let stderr = self.workers.join();
        let total_output = self.workers.total_output();
        Teardown {
            maybe_status,
            stderr,
            was_killed,
            was_reaped,
            total_output,
        }
    }
}

fn enforce_total_output(
    total: usize,
    baseline: usize,
    limits: &HarnessLimits,
) -> Result<(), HarnessFailureKind> {
    if total.saturating_sub(baseline) > limits.total_child_output_bytes() {
        return Err(HarnessFailureKind::TotalOutputExceeded);
    }
    Ok(())
}

fn receive_with_output_precedence(
    workers: &IoWorkers,
    deadline: Instant,
    timeout_kind: HarnessFailureKind,
    baseline: usize,
    limits: &HarnessLimits,
) -> Result<IoEvent, HarnessFailureKind> {
    workers
        .receive_until(deadline, timeout_kind)
        .map_err(|kind| match kind {
            HarnessFailureKind::StartupTimeout | HarnessFailureKind::RequestTimeout => {
                enforce_total_output(workers.total_output(), baseline, limits)
                    .err()
                    .unwrap_or(kind)
            }
            _ => kind,
        })
}

fn take_ready(state: &mut SessionState) -> ReadyChild {
    let current = std::mem::replace(state, SessionState::Reaped);
    match current {
        SessionState::Ready(ready) | SessionState::InFlight(ready) => ready,
        _ => panic!("ready child transition must follow a checked state"),
    }
}
