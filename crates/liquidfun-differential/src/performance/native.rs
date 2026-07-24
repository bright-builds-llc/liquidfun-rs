//! Native measured-region executor with injectable timing boundaries.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use liquidfun_test_protocol::{
    BenchmarkRunRequest, CanonicalCheckpoint, HarnessLimits, RequestId, ResolvedScenario,
    SemanticCheckpointIdentity, Sha256Hex, decode_resolved_scenario,
    encode_canonical_checkpoint_jsonl,
};
use sha2::{Digest, Sha256};

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

    /// Constructs every fresh per-unit session from already validated resolved bytes.
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
    measure_native_actions_with_checkpoint(driver, clock, actions)
        .map(|(elapsed, _checkpoint)| elapsed)
}

fn measure_native_actions_with_checkpoint<D, C>(
    driver: &mut D,
    clock: &mut C,
    actions: u32,
) -> Result<(Duration, D::Checkpoint), PerformanceExecutionError>
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
    let checkpoint = driver.capture();
    let validation = checkpoint
        .as_ref()
        .map_err(|error| *error)
        .and_then(|checkpoint| driver.validate(checkpoint));
    driver.teardown();
    validation?;
    Ok((elapsed, checkpoint?))
}

/// One authoritative native duration and its canonical semantic checkpoint identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeBenchmarkMeasurement {
    elapsed: Duration,
    semantic_checkpoint_identity: SemanticCheckpointIdentity,
}

impl NativeBenchmarkMeasurement {
    /// Returns the authoritative unprofiled duration.
    #[must_use]
    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Returns the checkpoint identity protecting the duration.
    #[must_use]
    pub const fn semantic_checkpoint_identity(&self) -> &SemanticCheckpointIdentity {
        &self.semantic_checkpoint_identity
    }
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
        let expected_checkpoint = run_untimed(&resolved, logical_horizon, None)?;
        for _ in 0..warmup_runs {
            let candidate = run_untimed(&resolved, logical_horizon, None)?;
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
        let mut driver = CatalogNativeDriver::new(
            &self.resolved,
            &self.expected_checkpoint,
            None,
            self.logical_horizon,
            1,
        );
        let mut clock = InstantClock;
        measure_native_actions(&mut driver, &mut clock, self.logical_horizon)
    }

    /// Measures one exact paired request and returns its canonical checkpoint identity.
    ///
    /// Authority setup and warm-ups run before the timer. The returned duration covers only the
    /// declared logical actions, while capture, semantic validation, hashing, and teardown remain
    /// outside the measured interval.
    ///
    /// # Errors
    ///
    /// Returns a bounded identity, execution, resource, horizon, or checkpoint failure before a
    /// duration can be accepted.
    pub fn measure_sample_for_request(
        &self,
        request: &BenchmarkRunRequest,
    ) -> Result<NativeBenchmarkMeasurement, PerformanceExecutionError> {
        let _execution_guard = NATIVE_BENCHMARK_LOCK.lock().map_err(|_error| {
            PerformanceExecutionError::new(PerformanceExecutionErrorKind::NativeExecution)
        })?;
        validate_request(self, request)?;
        let request_id = request.identity().request_id();
        let execution_units = request.identity().size_point().execution_units();
        let measured_actions = self
            .logical_horizon
            .checked_mul(execution_units)
            .ok_or_else(|| {
                PerformanceExecutionError::new(PerformanceExecutionErrorKind::ResourceLimit)
            })?;
        let expected_checkpoint = run_repeated_untimed(
            &self.resolved,
            self.logical_horizon,
            execution_units,
            Some(request_id),
        )?;
        if !benchmark_semantics_match_except_request_id(
            &self.expected_checkpoint,
            &expected_checkpoint,
        ) {
            return Err(PerformanceExecutionError::new(
                PerformanceExecutionErrorKind::CheckpointMismatch,
            ));
        }
        for _ in 0..request.identity().warmup_count() {
            let candidate = run_repeated_untimed(
                &self.resolved,
                self.logical_horizon,
                execution_units,
                Some(request_id),
            )?;
            if !benchmark_semantics_match(&expected_checkpoint, &candidate) {
                return Err(PerformanceExecutionError::new(
                    PerformanceExecutionErrorKind::CheckpointMismatch,
                ));
            }
        }
        let mut driver = CatalogNativeDriver::new(
            &self.resolved,
            &expected_checkpoint,
            Some(request_id),
            self.logical_horizon,
            execution_units,
        );
        let mut clock = InstantClock;
        let (elapsed, checkpoint) =
            measure_native_actions_with_checkpoint(&mut driver, &mut clock, measured_actions)?;
        let semantic_checkpoint_identity = semantic_checkpoint_identity(&checkpoint)?;
        Ok(NativeBenchmarkMeasurement {
            elapsed,
            semantic_checkpoint_identity,
        })
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
    resolved: &'a ResolvedScenario,
    expected_checkpoint: &'a CanonicalCheckpoint,
    maybe_request_id: Option<&'a RequestId>,
    logical_horizon: u32,
    execution_units: u32,
    executed_actions: u32,
    controllers: Vec<SessionController<NativeCatalogBackend>>,
}

impl<'a> CatalogNativeDriver<'a> {
    fn new(
        resolved: &'a ResolvedScenario,
        expected_checkpoint: &'a CanonicalCheckpoint,
        maybe_request_id: Option<&'a RequestId>,
        logical_horizon: u32,
        execution_units: u32,
    ) -> Self {
        Self {
            resolved,
            expected_checkpoint,
            maybe_request_id,
            logical_horizon,
            execution_units,
            executed_actions: 0,
            controllers: Vec::new(),
        }
    }

    fn controller_for_next_action(
        &mut self,
    ) -> Result<&mut SessionController<NativeCatalogBackend>, PerformanceExecutionError> {
        let unit_index =
            usize::try_from(self.executed_actions / self.logical_horizon).map_err(|_error| {
                PerformanceExecutionError::new(PerformanceExecutionErrorKind::ResourceLimit)
            })?;
        self.controllers.get_mut(unit_index).ok_or_else(|| {
            PerformanceExecutionError::new(PerformanceExecutionErrorKind::NativeExecution)
        })
    }
}

impl NativeBenchmarkDriver for CatalogNativeDriver<'_> {
    type Checkpoint = CanonicalCheckpoint;

    fn restart(&mut self) -> Result<(), PerformanceExecutionError> {
        self.controllers.clear();
        self.executed_actions = 0;
        let execution_units = usize::try_from(self.execution_units).map_err(|_error| {
            PerformanceExecutionError::new(PerformanceExecutionErrorKind::ResourceLimit)
        })?;
        self.controllers
            .try_reserve_exact(execution_units)
            .map_err(|_error| {
                PerformanceExecutionError::new(PerformanceExecutionErrorKind::ResourceLimit)
            })?;
        for _ in 0..execution_units {
            let mut backend = NativeCatalogBackend::new();
            if let Some(request_id) = self.maybe_request_id {
                backend.set_request_id(request_id.clone());
            }
            let mut controller = SessionController::new(backend);
            submit(
                &mut controller,
                SessionCommand::Select {
                    resolved: self.resolved.clone(),
                },
            )?;
            self.controllers.push(controller);
        }
        Ok(())
    }

    fn execute_action(&mut self) -> Result<(), PerformanceExecutionError> {
        submit(self.controller_for_next_action()?, SessionCommand::StepOnce)?;
        self.executed_actions = self.executed_actions.checked_add(1).ok_or_else(|| {
            PerformanceExecutionError::new(PerformanceExecutionErrorKind::ResourceLimit)
        })?;
        Ok(())
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
        let controller = self.controllers.last_mut().ok_or_else(|| {
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

    fn validate(&mut self, checkpoint: &Self::Checkpoint) -> Result<(), PerformanceExecutionError> {
        if !benchmark_semantics_match(self.expected_checkpoint, checkpoint) {
            return Err(PerformanceExecutionError::new(
                PerformanceExecutionErrorKind::CheckpointMismatch,
            ));
        }
        Ok(())
    }

    fn teardown(&mut self) {
        self.controllers.clear();
        self.executed_actions = 0;
    }
}

fn run_repeated_untimed(
    resolved: &ResolvedScenario,
    logical_horizon: u32,
    execution_units: u32,
    maybe_request_id: Option<&RequestId>,
) -> Result<CanonicalCheckpoint, PerformanceExecutionError> {
    let mut maybe_checkpoint = None;
    for _ in 0..execution_units {
        maybe_checkpoint = Some(run_untimed(resolved, logical_horizon, maybe_request_id)?);
    }
    maybe_checkpoint
        .ok_or_else(|| PerformanceExecutionError::new(PerformanceExecutionErrorKind::ResourceLimit))
}

fn run_untimed(
    resolved: &ResolvedScenario,
    logical_horizon: u32,
    maybe_request_id: Option<&RequestId>,
) -> Result<CanonicalCheckpoint, PerformanceExecutionError> {
    let mut driver = UntimedCatalogDriver::new(resolved, maybe_request_id);
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
    maybe_request_id: Option<&'a RequestId>,
    maybe_controller: Option<SessionController<NativeCatalogBackend>>,
}

impl<'a> UntimedCatalogDriver<'a> {
    const fn new(resolved: &'a ResolvedScenario, maybe_request_id: Option<&'a RequestId>) -> Self {
        Self {
            resolved,
            maybe_request_id,
            maybe_controller: None,
        }
    }
}

impl NativeBenchmarkDriver for UntimedCatalogDriver<'_> {
    type Checkpoint = CanonicalCheckpoint;

    fn restart(&mut self) -> Result<(), PerformanceExecutionError> {
        let mut backend = NativeCatalogBackend::new();
        if let Some(request_id) = self.maybe_request_id {
            backend.set_request_id(request_id.clone());
        }
        let mut controller = SessionController::new(backend);
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

fn validate_request(
    prepared: &PreparedNativeBenchmark,
    request: &BenchmarkRunRequest,
) -> Result<(), PerformanceExecutionError> {
    let identity = request.identity();
    if request.resolved_bytes() != prepared.resolved.canonical_bytes()
        || identity.resolved_sha256() != prepared.resolved.identity().content_sha256()
        || identity.settings() != prepared.resolved.identity().settings()
    {
        return Err(PerformanceExecutionError::new(
            PerformanceExecutionErrorKind::ResolvedIdentity,
        ));
    }
    if identity.measured_horizon() != prepared.logical_horizon {
        return Err(PerformanceExecutionError::new(
            PerformanceExecutionErrorKind::HorizonMismatch,
        ));
    }
    Ok(())
}

fn semantic_checkpoint_identity(
    checkpoint: &CanonicalCheckpoint,
) -> Result<SemanticCheckpointIdentity, PerformanceExecutionError> {
    let mut bytes =
        encode_canonical_checkpoint_jsonl(checkpoint, &HarnessLimits::phase2_default_v1())
            .map_err(|_error| {
                PerformanceExecutionError::new(PerformanceExecutionErrorKind::CheckpointMismatch)
            })?;
    if bytes.pop() != Some(b'\n') {
        return Err(PerformanceExecutionError::new(
            PerformanceExecutionErrorKind::CheckpointMismatch,
        ));
    }
    Ok(SemanticCheckpointIdentity::new(
        checkpoint.request_id().clone(),
        checkpoint.resolved_sha256().clone(),
        checkpoint.checkpoint_id().clone(),
        Sha256Hex::from_digest(Sha256::digest(bytes).into()),
    ))
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

fn benchmark_semantics_match_except_request_id(
    expected: &CanonicalCheckpoint,
    candidate: &CanonicalCheckpoint,
) -> bool {
    expected.protocol_version() == candidate.protocol_version()
        && expected.schema_version() == candidate.schema_version()
        && expected.record_kind() == candidate.record_kind()
        && expected.resolved_sha256() == candidate.resolved_sha256()
        && expected.checkpoint_id() == candidate.checkpoint_id()
        && expected.position() == candidate.position()
        && expected.simulation_time_bits() == candidate.simulation_time_bits()
        && expected.observations() == candidate.observations()
        && expected.numeric_observations() == candidate.numeric_observations()
        && expected.ordered_occurrences() == candidate.ordered_occurrences()
        && expected.unordered_sets() == candidate.unordered_sets()
}
