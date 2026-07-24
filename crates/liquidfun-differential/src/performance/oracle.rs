//! Reviewed paired-adapter boundary and supervised C++ oracle implementation.

use std::path::Path;

use liquidfun_test_protocol::performance::{
    BenchmarkHarnessFailureKind, BenchmarkPerformanceResult, BenchmarkRunOutcome,
    BenchmarkRunRequest, BenchmarkRunResult, PerformanceEngineRole,
};

use crate::{
    CatalogOracleSupervisor, OracleExecutable, OracleExecutableError, OraclePreset, SessionProfile,
};

use super::native::{PerformanceExecutionErrorKind, PreparedNativeBenchmark};
use super::report::BenchmarkAdapterOutput;

/// One synchronous engine adapter used by the paired same-host runner.
pub trait PairedBenchmarkAdapter {
    /// Returns the immutable role this adapter is allowed to produce.
    fn engine_role(&self) -> PerformanceEngineRole;

    /// Executes one exact request and returns a typed terminal result.
    ///
    /// # Errors
    ///
    /// Returns a closed harness failure for process, protocol, provenance, identity, reset, or
    /// adapter failures. Physics mismatch remains an in-band [`BenchmarkAdapterOutput`] outcome.
    fn execute(
        &mut self,
        request: &BenchmarkRunRequest,
        baseline_run: u8,
    ) -> Result<BenchmarkAdapterOutput, BenchmarkHarnessFailureKind>;
}

/// Concrete native Rust adapter over one sealed prepared benchmark case.
pub struct NativeBenchmarkAdapter {
    prepared: PreparedNativeBenchmark,
    reset_epoch: u64,
}

impl NativeBenchmarkAdapter {
    /// Creates an adapter whose complete per-request reset epochs begin at one.
    #[must_use]
    pub const fn new(prepared: PreparedNativeBenchmark) -> Self {
        Self {
            prepared,
            reset_epoch: 0,
        }
    }
}

impl PairedBenchmarkAdapter for NativeBenchmarkAdapter {
    fn engine_role(&self) -> PerformanceEngineRole {
        PerformanceEngineRole::NativeRust
    }

    fn execute(
        &mut self,
        request: &BenchmarkRunRequest,
        _baseline_run: u8,
    ) -> Result<BenchmarkAdapterOutput, BenchmarkHarnessFailureKind> {
        let measurement = self
            .prepared
            .measure_sample_for_request(request)
            .map_err(|error| match error.kind() {
                PerformanceExecutionErrorKind::ResolvedIdentity
                | PerformanceExecutionErrorKind::HorizonMismatch => {
                    BenchmarkHarnessFailureKind::IdentityMismatch
                }
                PerformanceExecutionErrorKind::ResourceLimit
                | PerformanceExecutionErrorKind::NativeExecution
                | PerformanceExecutionErrorKind::CheckpointMismatch
                | PerformanceExecutionErrorKind::DurationOverflow => {
                    BenchmarkHarnessFailureKind::AdapterFailure
                }
            })?;
        let nanoseconds = u64::try_from(measurement.elapsed().as_nanos())
            .map_err(|_error| BenchmarkHarnessFailureKind::AdapterFailure)?;
        let performance = BenchmarkPerformanceResult::new(
            nanoseconds,
            None,
            measurement.semantic_checkpoint_identity().clone(),
        )
        .map_err(|_error| BenchmarkHarnessFailureKind::AdapterFailure)?;
        let reset_epoch = self
            .reset_epoch
            .checked_add(1)
            .ok_or(BenchmarkHarnessFailureKind::AdapterResetFailure)?;
        let result = BenchmarkRunResult::new(
            request.identity().clone(),
            PerformanceEngineRole::NativeRust,
            reset_epoch,
            BenchmarkRunOutcome::Performance(performance),
        )
        .map_err(|_error| BenchmarkHarnessFailureKind::IdentityMismatch)?;
        self.reset_epoch = reset_epoch;
        BenchmarkAdapterOutput::new_with_process_generation(result, Vec::new(), 1)
    }
}

/// Concrete long-lived pinned C++ adapter using the repository's bounded synchronous supervisor.
pub struct OracleBenchmarkAdapter {
    supervisor: CatalogOracleSupervisor,
}

impl OracleBenchmarkAdapter {
    /// Resolves the reviewed release oracle and creates one reusable same-host session.
    ///
    /// # Errors
    ///
    /// Returns [`OracleExecutableError`] when the confined release executable is absent, linked,
    /// non-executable, or outside its reviewed preset directory.
    pub fn new(
        repository_root: &Path,
        expected_oracle_revision: &str,
    ) -> Result<Self, OracleExecutableError> {
        let executable = OracleExecutable::resolve(repository_root, OraclePreset::Release)?;
        Ok(Self {
            supervisor: CatalogOracleSupervisor::new(
                executable,
                SessionProfile::Reuse,
                expected_oracle_revision,
            ),
        })
    }
}

impl PairedBenchmarkAdapter for OracleBenchmarkAdapter {
    fn engine_role(&self) -> PerformanceEngineRole {
        PerformanceEngineRole::PinnedCppOracle
    }

    fn execute(
        &mut self,
        request: &BenchmarkRunRequest,
        _baseline_run: u8,
    ) -> Result<BenchmarkAdapterOutput, BenchmarkHarnessFailureKind> {
        let result = self.supervisor.execute_benchmark(request)?;
        BenchmarkAdapterOutput::new_with_process_generation(
            result,
            Vec::new(),
            self.supervisor.process_generation(),
        )
    }
}
