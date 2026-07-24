//! Long-lived catalog child supervision on the shared bounded I/O machinery.

use std::time::Duration;

use liquidfun_test_protocol::{
    BuildIdentity, CatalogRunRequest, HarnessLimits, Sha256Hex,
    performance::{BenchmarkHarnessFailureKind, BenchmarkRunRequest, BenchmarkRunResult},
};

use crate::{CatalogFailureKind, CatalogRunCapture};

use super::{
    ChildIo, HandshakingChild, OracleExecutable, ReadyChild, SessionProfile, StderrSnapshot,
    Teardown, complete_handshake, enforce_total_output, successful_teardown_failure,
};

mod benchmark;
mod protocol;
use benchmark::{map_harness_kind, run_benchmark_request};
use protocol::{run_catalog_request, tier_satisfies};

/// Complete validated C++ catalog capture and child lifecycle evidence.
#[derive(Debug)]
pub struct CapturedCatalogRun {
    identity: BuildIdentity,
    capture: CatalogRunCapture,
    response_bytes: Box<[u8]>,
    reset_epoch: u64,
    reset_verified: bool,
}

impl CapturedCatalogRun {
    /// Returns the provenance-validated C++ build identity.
    #[must_use]
    pub const fn identity(&self) -> &BuildIdentity {
        &self.identity
    }

    /// Returns the semantic capture built only from validated child records.
    #[must_use]
    pub const fn capture(&self) -> &CatalogRunCapture {
        &self.capture
    }

    /// Returns exact handshake, checkpoint, and terminal JSONL bytes.
    #[must_use]
    pub fn response_bytes(&self) -> &[u8] {
        &self.response_bytes
    }

    /// Returns the child-provided monotonic reset epoch.
    #[must_use]
    pub const fn reset_epoch(&self) -> u64 {
        self.reset_epoch
    }

    /// Returns the child-provided complete reset proof.
    #[must_use]
    pub const fn reset_verified(&self) -> bool {
        self.reset_verified
    }
}

/// Bounded process failure with kill/reap and retained-stderr evidence.
#[derive(Debug, thiserror::Error)]
#[error("catalog oracle failed: {kind:?}")]
pub struct CatalogProcessError {
    kind: CatalogFailureKind,
    retained_stderr: Box<[u8]>,
    stderr_bytes: usize,
    child_killed: bool,
    child_reaped: bool,
    maybe_identity: Option<Box<BuildIdentity>>,
}

impl CatalogProcessError {
    /// Returns the stable non-physics failure category.
    #[must_use]
    pub const fn kind(&self) -> CatalogFailureKind {
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

    /// Returns whether poison handling killed the child.
    #[must_use]
    pub const fn child_killed(&self) -> bool {
        self.child_killed
    }

    /// Returns whether every started child was reaped.
    #[must_use]
    pub const fn child_reaped(&self) -> bool {
        self.child_reaped
    }

    /// Returns the validated handshake identity when startup completed.
    #[must_use]
    pub fn maybe_identity(&self) -> Option<&BuildIdentity> {
        match self.maybe_identity.as_ref() {
            Some(identity) => Some(identity.as_ref()),
            None => None,
        }
    }
}

/// Sequential long-lived catalog supervisor with one request in flight.
pub struct CatalogOracleSupervisor {
    executable: OracleExecutable,
    profile: SessionProfile,
    limits: HarnessLimits,
    expected_oracle_revision: Box<str>,
    maybe_ready: Option<ReadyChild>,
    process_generation: u64,
    benchmark_requests: u64,
}

impl CatalogOracleSupervisor {
    /// Creates a lazy supervisor using one reviewed executable and immutable profile.
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
            maybe_ready: None,
            process_generation: 0,
            benchmark_requests: 0,
        }
    }

    /// Starts and provenance-validates the child without executing a scenario.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogProcessError`] for startup, handshake, provenance, timeout, or process
    /// failure. Any failed child is killed and reaped.
    pub fn discover_identity(&mut self) -> Result<BuildIdentity, CatalogProcessError> {
        self.ensure_ready()?;
        self.maybe_ready
            .as_ref()
            .map(|ready| ready.identity.clone())
            .ok_or_else(|| failure_without_child(CatalogFailureKind::ChildProcess))
    }

    /// Returns the immutable resource-profile identity required in catalog requests.
    #[must_use]
    pub fn limits_profile_sha256(&self) -> Sha256Hex {
        self.limits.profile_sha256()
    }

    /// Executes one exact resolved-byte request and validates every checkpoint plus reset proof.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogProcessError`] for provenance, framing, timeout, resource, child, schema,
    /// checkpoint, or reset failure. Deterministic requests are never retried.
    pub fn execute(
        &mut self,
        request: &CatalogRunRequest,
    ) -> Result<CapturedCatalogRun, CatalogProcessError> {
        self.ensure_ready()?;
        self.validate_provenance(request)?;
        let mut ready = self
            .maybe_ready
            .take()
            .ok_or_else(|| failure_without_child(CatalogFailureKind::ChildProcess))?;
        let result = run_catalog_request(&mut ready, request, &self.limits);
        match result {
            Ok((capture, response_bytes, end)) => {
                ready.requests = ready.requests.saturating_add(1);
                let identity = ready.identity.clone();
                if self.profile.keeps_process() {
                    self.maybe_ready = Some(ready);
                } else {
                    let baseline = ready.last_request_baseline;
                    let teardown = ready.io.shutdown(self.limits.request_timeout(), false);
                    if enforce_total_output(teardown.total_output, baseline, &self.limits).is_err()
                        || successful_teardown_failure(&teardown).is_some()
                    {
                        return Err(failure_from_teardown(
                            CatalogFailureKind::ChildProcess,
                            teardown,
                            Some(identity),
                        ));
                    }
                }
                Ok(CapturedCatalogRun {
                    identity,
                    capture,
                    response_bytes: response_bytes.into_boxed_slice(),
                    reset_epoch: end.reset_epoch,
                    reset_verified: end.reset_verified,
                })
            }
            Err(kind) => {
                let identity = ready.identity.clone();
                let teardown = ready.io.shutdown(Duration::ZERO, true);
                Err(failure_from_teardown(kind, teardown, Some(identity)))
            }
        }
    }

    /// Executes one strict benchmark request through the same bounded long-lived child.
    ///
    /// This crate-private seam returns only a validated typed result or a closed failure category;
    /// raw pipes, process handles, and unbounded output never leave the supervisor.
    pub(crate) fn execute_benchmark(
        &mut self,
        request: &BenchmarkRunRequest,
    ) -> Result<BenchmarkRunResult, BenchmarkHarnessFailureKind> {
        self.ensure_ready()
            .map_err(|error| map_catalog_failure(error.kind()))?;
        let mut ready = self
            .maybe_ready
            .take()
            .ok_or(BenchmarkHarnessFailureKind::AdapterFailure)?;
        let expected_reset_epoch = self
            .benchmark_requests
            .checked_add(1)
            .ok_or(BenchmarkHarnessFailureKind::AdapterResetFailure)?;
        let result = run_benchmark_request(&mut ready, request, expected_reset_epoch, &self.limits);
        match result {
            Ok(result) => {
                ready.requests = ready.requests.saturating_add(1);
                self.benchmark_requests = expected_reset_epoch;
                if self.profile.keeps_process() {
                    self.maybe_ready = Some(ready);
                    return Ok(result);
                }
                let baseline = ready.last_request_baseline;
                let teardown = ready.io.shutdown(self.limits.request_timeout(), false);
                let maybe_failure =
                    enforce_total_output(teardown.total_output, baseline, &self.limits)
                        .err()
                        .map(map_harness_kind)
                        .or_else(|| successful_teardown_failure(&teardown).map(map_harness_kind));
                if let Some(kind) = maybe_failure {
                    return Err(kind);
                }
                Ok(result)
            }
            Err(mut kind) => {
                let teardown = ready.io.shutdown(Duration::ZERO, true);
                if !teardown.was_killed
                    && let Some(teardown_kind) = successful_teardown_failure(&teardown)
                {
                    kind = map_harness_kind(teardown_kind);
                }
                Err(kind)
            }
        }
    }

    /// Returns the number of child processes started by this supervisor.
    #[must_use]
    pub const fn process_generation(&self) -> u64 {
        self.process_generation
    }

    fn ensure_ready(&mut self) -> Result<(), CatalogProcessError> {
        if self
            .maybe_ready
            .as_ref()
            .is_some_and(|ready| ready.requests >= self.limits.request_budget())
        {
            let ready = self
                .maybe_ready
                .take()
                .ok_or_else(|| failure_without_child(CatalogFailureKind::ChildProcess))?;
            let identity = ready.identity.clone();
            let baseline = ready.last_request_baseline;
            let teardown = ready.io.shutdown(self.limits.request_timeout(), false);
            if enforce_total_output(teardown.total_output, baseline, &self.limits).is_err()
                || successful_teardown_failure(&teardown).is_some()
            {
                return Err(failure_from_teardown(
                    CatalogFailureKind::ChildProcess,
                    teardown,
                    Some(identity),
                ));
            }
        }
        if self.maybe_ready.is_some() {
            return Ok(());
        }
        self.process_generation = self.process_generation.saturating_add(1);
        self.benchmark_requests = 0;
        let handshaking = spawn_child(&self.executable, self.profile, &self.limits)
            .map_err(|_error| failure_without_child(CatalogFailureKind::ChildProcess))?;
        match complete_handshake(handshaking, &self.expected_oracle_revision, &self.limits) {
            Ok(ready) => {
                self.maybe_ready = Some(ready);
                Ok(())
            }
            Err((_kind, child)) => {
                let teardown = child.io.shutdown(Duration::ZERO, true);
                Err(failure_from_teardown(
                    CatalogFailureKind::Provenance,
                    teardown,
                    None,
                ))
            }
        }
    }

    fn validate_provenance(&self, request: &CatalogRunRequest) -> Result<(), CatalogProcessError> {
        let ready = self
            .maybe_ready
            .as_ref()
            .ok_or_else(|| failure_without_child(CatalogFailureKind::ChildProcess))?;
        let requirements = request.provenance_requirements();
        if requirements.required_identity_sha256() != ready.identity.identity_sha256()
            || requirements.limits_profile_sha256() != &self.limits.profile_sha256()
            || !tier_satisfies(ready.identity.evidence_tier(), requirements.evidence_tier())
        {
            return Err(failure_without_child(CatalogFailureKind::Provenance));
        }
        Ok(())
    }
}

const fn map_catalog_failure(kind: CatalogFailureKind) -> BenchmarkHarnessFailureKind {
    match kind {
        CatalogFailureKind::Provenance => BenchmarkHarnessFailureKind::IdentityMismatch,
        CatalogFailureKind::Timeout => BenchmarkHarnessFailureKind::RequestTimeout,
        CatalogFailureKind::ChildProcess => BenchmarkHarnessFailureKind::ChildNonZeroExit,
        CatalogFailureKind::MalformedRecord | CatalogFailureKind::Protocol => {
            BenchmarkHarnessFailureKind::MalformedRecord
        }
        CatalogFailureKind::ResetFailure => BenchmarkHarnessFailureKind::AdapterResetFailure,
        CatalogFailureKind::ResourceLimit => BenchmarkHarnessFailureKind::OutputLimitExceeded,
        CatalogFailureKind::PhysicsMismatch
        | CatalogFailureKind::HarnessFailure
        | CatalogFailureKind::NativeExecution
        | CatalogFailureKind::Evidence => BenchmarkHarnessFailureKind::AdapterFailure,
    }
}

impl Drop for CatalogOracleSupervisor {
    fn drop(&mut self) {
        if let Some(ready) = self.maybe_ready.take() {
            let _teardown = ready.io.shutdown(Duration::ZERO, true);
        }
    }
}

fn spawn_child(
    executable: &OracleExecutable,
    profile: SessionProfile,
    limits: &HarnessLimits,
) -> std::io::Result<HandshakingChild> {
    use std::process::Stdio;

    let mut command = executable.command();
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if profile == SessionProfile::Sanitizer || executable.preset == super::OraclePreset::AsanUbsan {
        command
            .env("ASAN_OPTIONS", "abort_on_error=1:halt_on_error=1")
            .env("UBSAN_OPTIONS", "halt_on_error=1:print_stacktrace=1");
    }
    let mut child = command.spawn()?;
    let maybe_stdin = child.stdin.take();
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| std::io::Error::other("child stdout was not piped"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| std::io::Error::other("child stderr was not piped"))?;
    let workers = super::IoWorkers::spawn(
        stdout,
        stderr,
        limits.output_record_bytes(),
        limits.retained_stderr_bytes(),
    );
    Ok(HandshakingChild {
        io: ChildIo {
            child,
            maybe_stdin,
            workers,
        },
    })
}

fn failure_from_teardown(
    mut kind: CatalogFailureKind,
    teardown: Teardown,
    maybe_identity: Option<BuildIdentity>,
) -> CatalogProcessError {
    if teardown
        .stderr
        .retained
        .starts_with(b"liquidfun-reference request rejected:")
    {
        kind = CatalogFailureKind::MalformedRecord;
    } else if teardown
        .stderr
        .retained
        .starts_with(b"catalog child failed")
        || (!teardown.was_killed
            && teardown
                .maybe_status
                .as_ref()
                .is_some_and(|status| !status.success()))
    {
        kind = CatalogFailureKind::ChildProcess;
    }
    CatalogProcessError {
        kind,
        retained_stderr: teardown.stderr.retained.into_boxed_slice(),
        stderr_bytes: teardown.stderr.total_bytes,
        child_killed: teardown.was_killed,
        child_reaped: teardown.was_reaped,
        maybe_identity: maybe_identity.map(Box::new),
    }
}

fn failure_without_child(kind: CatalogFailureKind) -> CatalogProcessError {
    failure_from_teardown(
        kind,
        Teardown {
            maybe_status: None,
            stderr: StderrSnapshot::default(),
            was_killed: false,
            was_reaped: false,
            total_output: 0,
        },
        None,
    )
}
