//! Native measured-region executor with injectable timing boundaries.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use liquidfun_test_protocol::{
    CanonicalCheckpoint, ResolvedScenario, Sha256Hex, decode_resolved_scenario,
};

use crate::{NativeCatalogBackend, SessionCommand, SessionController, SessionControllerError};

/// Maximum warm-up runs accepted during one case preparation.
pub const MAXIMUM_BENCHMARK_WARMUPS: u32 = 100;
/// Maximum timing samples accepted for one engine.
pub const MAXIMUM_BENCHMARK_SAMPLES: u64 = 10_000;
/// Maximum logical actions accepted in one measured sample.
pub const MAXIMUM_BENCHMARK_ACTIONS: u32 = 1_000_000;
static NATIVE_BENCHMARK_LOCK: Mutex<()> = Mutex::new(());

/// Stable native benchmark failure categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceExecutionErrorKind {
    /// Exact resolved bytes or their asserted digest did not validate.
    ResolvedIdentity,
    /// A requested warm-up, sample, or action count exceeded its closed bound.
    ResourceLimit,
    /// Native session construction or logical execution failed.
    NativeExecution,
    /// The measured horizon disagreed with the resolved checkpoint schedule.
    HorizonMismatch,
    /// Post-timer semantic state disagreed with the prepared authority.
    CheckpointMismatch,
    /// Accumulating sample durations exceeded [`Duration`].
    DurationOverflow,
}

/// Redacted native benchmark failure without timing or engine-private state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("native performance execution failed: {kind:?}")]
pub struct PerformanceExecutionError {
    kind: PerformanceExecutionErrorKind,
}

impl PerformanceExecutionError {
    const fn new(kind: PerformanceExecutionErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the stable failure category.
    #[must_use]
    pub const fn kind(self) -> PerformanceExecutionErrorKind {
        self.kind
    }
}

/// Injectable clock used to prove the exact measured boundary.
pub trait NativeBenchmarkClock {
    /// Opaque start marker retained only until this sample stops.
    type Stamp;

    /// Starts the authoritative measured interval.
    fn start(&mut self) -> Self::Stamp;

    /// Stops the interval and returns only its elapsed duration.
    fn elapsed(&mut self, stamp: Self::Stamp) -> Duration;
}

/// Injectable native lifecycle used to prove timed and excluded operations.
pub trait NativeBenchmarkDriver {
    /// Owned semantic checkpoint captured after the timer stops.
    type Checkpoint;

    /// Constructs a fresh session from already validated resolved bytes.
    ///
    /// # Errors
    ///
    /// Returns a bounded native execution failure.
    fn restart(&mut self) -> Result<(), PerformanceExecutionError>;

    /// Executes exactly one declared logical action.
    ///
    /// # Errors
    ///
    /// Returns a bounded native execution failure.
    fn execute_action(&mut self) -> Result<(), PerformanceExecutionError>;

    /// Captures final semantic state after timing stops.
    ///
    /// # Errors
    ///
    /// Returns a bounded capture or execution failure.
    fn capture(&mut self) -> Result<Self::Checkpoint, PerformanceExecutionError>;

    /// Validates captured semantic state before the duration is accepted.
    ///
    /// # Errors
    ///
    /// Returns a bounded semantic mismatch.
    fn validate(&mut self, checkpoint: &Self::Checkpoint) -> Result<(), PerformanceExecutionError>;

    /// Tears down the complete sample session outside the measured interval.
    fn teardown(&mut self);
}

/// Runs one fresh sample with restart, capture, validation, and teardown outside its timer.
///
/// # Errors
///
/// Returns a bounded failure for excessive actions, native execution, or semantic drift.
pub fn measure_native_actions<D, C>(
    driver: &mut D,
    clock: &mut C,
    actions: u32,
) -> Result<Duration, PerformanceExecutionError>
where
    D: NativeBenchmarkDriver,
    C: NativeBenchmarkClock,
{
    if actions == 0 || actions > MAXIMUM_BENCHMARK_ACTIONS {
        return Err(PerformanceExecutionError::new(
            PerformanceExecutionErrorKind::ResourceLimit,
        ));
    }
    if let Err(error) = driver.restart() {
        driver.teardown();
        return Err(error);
    }
    let stamp = clock.start();
    for _ in 0..actions {
        if let Err(error) = driver.execute_action() {
            let _discarded_elapsed = clock.elapsed(stamp);
            driver.teardown();
            return Err(error);
        }
    }
    let elapsed = clock.elapsed(stamp);
    let validation = driver
        .capture()
        .and_then(|checkpoint| driver.validate(&checkpoint));
    driver.teardown();
    validation?;
    Ok(elapsed)
}

/// One sealed native case prepared before any authoritative timer starts.
#[derive(Clone)]
pub struct PreparedNativeBenchmark {
    resolved: ResolvedScenario,
    expected_checkpoint: CanonicalCheckpoint,
    logical_horizon: u32,
}

impl PreparedNativeBenchmark {
    /// Validates canonical bytes/hash, prepares semantic authority, and completes untimed warmups.
    ///
    /// # Errors
    ///
    /// Returns a bounded failure for identity, horizon, resource, execution, or semantic drift.
    pub fn new(
        resolved: ResolvedScenario,
        asserted_sha256: &Sha256Hex,
        logical_horizon: u32,
        warmup_runs: u32,
    ) -> Result<Self, PerformanceExecutionError> {
        let _execution_guard = NATIVE_BENCHMARK_LOCK.lock().map_err(|_error| {
            PerformanceExecutionError::new(PerformanceExecutionErrorKind::NativeExecution)
        })?;
        if warmup_runs > MAXIMUM_BENCHMARK_WARMUPS
            || logical_horizon == 0
            || logical_horizon > MAXIMUM_BENCHMARK_ACTIONS
        {
            return Err(PerformanceExecutionError::new(
                PerformanceExecutionErrorKind::ResourceLimit,
            ));
        }
        let decoded = decode_resolved_scenario(resolved.canonical_bytes(), asserted_sha256)
            .map_err(|_error| {
                PerformanceExecutionError::new(PerformanceExecutionErrorKind::ResolvedIdentity)
            })?;
        if decoded != resolved || resolved.identity().content_sha256() != asserted_sha256 {
            return Err(PerformanceExecutionError::new(
                PerformanceExecutionErrorKind::ResolvedIdentity,
            ));
        }
        let checkpoint_count = u32::try_from(resolved.checkpoints().len()).map_err(|_| {
            PerformanceExecutionError::new(PerformanceExecutionErrorKind::HorizonMismatch)
        })?;
        if checkpoint_count != logical_horizon {
            return Err(PerformanceExecutionError::new(
                PerformanceExecutionErrorKind::HorizonMismatch,
            ));
        }
        let expected_checkpoint = run_untimed(&resolved, logical_horizon)?;
        for _ in 0..warmup_runs {
            let candidate = run_untimed(&resolved, logical_horizon)?;
            if !benchmark_semantics_match(&expected_checkpoint, &candidate) {
                return Err(PerformanceExecutionError::new(
                    PerformanceExecutionErrorKind::CheckpointMismatch,
                ));
            }
        }
        Ok(Self {
            resolved,
            expected_checkpoint,
            logical_horizon,
        })
    }

    /// Returns the exact decoded resolved input.
    #[must_use]
    pub const fn resolved(&self) -> &ResolvedScenario {
        &self.resolved
    }

    /// Returns the pre-timing semantic authority.
    #[must_use]
    pub const fn expected_checkpoint(&self) -> &CanonicalCheckpoint {
        &self.expected_checkpoint
    }

    /// Measures one validated native sample.
    ///
    /// # Errors
    ///
    /// Returns a bounded execution or semantic failure before a duration can be accepted.
    pub fn measure_sample(&self) -> Result<Duration, PerformanceExecutionError> {
        let _execution_guard = NATIVE_BENCHMARK_LOCK.lock().map_err(|_error| {
            PerformanceExecutionError::new(PerformanceExecutionErrorKind::NativeExecution)
        })?;
        let mut driver = CatalogNativeDriver::new(self);
        let mut clock = InstantClock;
        measure_native_actions(&mut driver, &mut clock, self.logical_horizon)
    }
}

struct InstantClock;

impl NativeBenchmarkClock for InstantClock {
    type Stamp = Instant;

    fn start(&mut self) -> Self::Stamp {
        Instant::now()
    }

    fn elapsed(&mut self, stamp: Self::Stamp) -> Duration {
        stamp.elapsed()
    }
}

struct CatalogNativeDriver<'a> {
    prepared: &'a PreparedNativeBenchmark,
    maybe_controller: Option<SessionController<NativeCatalogBackend>>,
}

impl<'a> CatalogNativeDriver<'a> {
    const fn new(prepared: &'a PreparedNativeBenchmark) -> Self {
        Self {
            prepared,
            maybe_controller: None,
        }
    }

    fn controller(
        &mut self,
    ) -> Result<&mut SessionController<NativeCatalogBackend>, PerformanceExecutionError> {
        self.maybe_controller.as_mut().ok_or_else(|| {
            PerformanceExecutionError::new(PerformanceExecutionErrorKind::NativeExecution)
        })
    }
}

impl NativeBenchmarkDriver for CatalogNativeDriver<'_> {
    type Checkpoint = CanonicalCheckpoint;

    fn restart(&mut self) -> Result<(), PerformanceExecutionError> {
        let mut controller = SessionController::new(NativeCatalogBackend::new());
        submit(
            &mut controller,
            SessionCommand::Select {
                resolved: self.prepared.resolved.clone(),
            },
        )?;
        self.maybe_controller = Some(controller);
        Ok(())
    }

    fn execute_action(&mut self) -> Result<(), PerformanceExecutionError> {
        submit(self.controller()?, SessionCommand::StepOnce)
    }

    fn capture(&mut self) -> Result<Self::Checkpoint, PerformanceExecutionError> {
        let checkpoint_id = self
            .prepared
            .resolved
            .checkpoints()
            .last()
            .ok_or_else(|| {
                PerformanceExecutionError::new(PerformanceExecutionErrorKind::HorizonMismatch)
            })?
            .checkpoint_id()
            .clone();
        let controller = self.controller()?;
        submit(
            controller,
            SessionCommand::CaptureCheckpoint { checkpoint_id },
        )?;
        controller
            .captures()
            .last()
            .map(|capture| capture.value().clone())
            .ok_or_else(|| {
                PerformanceExecutionError::new(PerformanceExecutionErrorKind::CheckpointMismatch)
            })
    }

    fn validate(&mut self, checkpoint: &Self::Checkpoint) -> Result<(), PerformanceExecutionError> {
        if !benchmark_semantics_match(&self.prepared.expected_checkpoint, checkpoint) {
            return Err(PerformanceExecutionError::new(
                PerformanceExecutionErrorKind::CheckpointMismatch,
            ));
        }
        Ok(())
    }

    fn teardown(&mut self) {
        self.maybe_controller = None;
    }
}

fn run_untimed(
    resolved: &ResolvedScenario,
    logical_horizon: u32,
) -> Result<CanonicalCheckpoint, PerformanceExecutionError> {
    let mut driver = UntimedCatalogDriver::new(resolved);
    driver.restart()?;
    for _ in 0..logical_horizon {
        driver.execute_action()?;
    }
    let checkpoint = driver.capture()?;
    driver.teardown();
    Ok(checkpoint)
}

struct UntimedCatalogDriver<'a> {
    resolved: &'a ResolvedScenario,
    maybe_controller: Option<SessionController<NativeCatalogBackend>>,
}

impl<'a> UntimedCatalogDriver<'a> {
    const fn new(resolved: &'a ResolvedScenario) -> Self {
        Self {
            resolved,
            maybe_controller: None,
        }
    }
}

impl NativeBenchmarkDriver for UntimedCatalogDriver<'_> {
    type Checkpoint = CanonicalCheckpoint;

    fn restart(&mut self) -> Result<(), PerformanceExecutionError> {
        let mut controller = SessionController::new(NativeCatalogBackend::new());
        submit(
            &mut controller,
            SessionCommand::Select {
                resolved: self.resolved.clone(),
            },
        )?;
        self.maybe_controller = Some(controller);
        Ok(())
    }

    fn execute_action(&mut self) -> Result<(), PerformanceExecutionError> {
        let controller = self.maybe_controller.as_mut().ok_or_else(|| {
            PerformanceExecutionError::new(PerformanceExecutionErrorKind::NativeExecution)
        })?;
        submit(controller, SessionCommand::StepOnce)
    }

    fn capture(&mut self) -> Result<Self::Checkpoint, PerformanceExecutionError> {
        let checkpoint_id = self
            .resolved
            .checkpoints()
            .last()
            .ok_or_else(|| {
                PerformanceExecutionError::new(PerformanceExecutionErrorKind::HorizonMismatch)
            })?
            .checkpoint_id()
            .clone();
        let controller = self.maybe_controller.as_mut().ok_or_else(|| {
            PerformanceExecutionError::new(PerformanceExecutionErrorKind::NativeExecution)
        })?;
        submit(
            controller,
            SessionCommand::CaptureCheckpoint { checkpoint_id },
        )?;
        controller
            .captures()
            .last()
            .map(|capture| capture.value().clone())
            .ok_or_else(|| {
                PerformanceExecutionError::new(PerformanceExecutionErrorKind::CheckpointMismatch)
            })
    }

    fn validate(
        &mut self,
        _checkpoint: &Self::Checkpoint,
    ) -> Result<(), PerformanceExecutionError> {
        Ok(())
    }

    fn teardown(&mut self) {
        self.maybe_controller = None;
    }
}

fn submit(
    controller: &mut SessionController<NativeCatalogBackend>,
    command: SessionCommand,
) -> Result<(), PerformanceExecutionError> {
    let command_id = controller.next_command_id().ok_or_else(|| {
        PerformanceExecutionError::new(PerformanceExecutionErrorKind::ResourceLimit)
    })?;
    controller
        .submit(command_id, command)
        .map(|_outcome| ())
        .map_err(map_controller_error)
}

const fn map_controller_error(_error: SessionControllerError) -> PerformanceExecutionError {
    PerformanceExecutionError::new(PerformanceExecutionErrorKind::NativeExecution)
}

/// Compares only authoritative non-visual checkpoint identity and semantic observation lanes.
///
/// Renderer debug primitives and diagnostic profile names cannot accept or reject a physics timing
/// sample. Exact resolved identity, checkpoint position/time, structural and numeric observations,
/// ordered occurrences, and unordered sets remain mandatory.
#[must_use]
pub fn benchmark_semantics_match(
    expected: &CanonicalCheckpoint,
    candidate: &CanonicalCheckpoint,
) -> bool {
    expected.protocol_version() == candidate.protocol_version()
        && expected.schema_version() == candidate.schema_version()
        && expected.record_kind() == candidate.record_kind()
        && expected.request_id() == candidate.request_id()
        && expected.resolved_sha256() == candidate.resolved_sha256()
        && expected.checkpoint_id() == candidate.checkpoint_id()
        && expected.position() == candidate.position()
        && expected.simulation_time_bits() == candidate.simulation_time_bits()
        && expected.observations() == candidate.observations()
        && expected.numeric_observations() == candidate.numeric_observations()
        && expected.ordered_occurrences() == candidate.ordered_occurrences()
        && expected.unordered_sets() == candidate.unordered_sets()
}
