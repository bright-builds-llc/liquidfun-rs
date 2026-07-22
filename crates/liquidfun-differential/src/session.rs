//! Renderer-neutral command and backend boundary for one resolved run session.

use liquidfun_test_protocol::{
    ActionSchedule, CheckpointDeclaration, CheckpointId, FloatBits, ResolvedScenario, RunSettings,
    ScenarioActionId, ScheduledAction,
};

mod backend;
mod state;
pub use state::*;

/// Monotonic identity of one frontend-submitted command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionCommandId(u64);

impl SessionCommandId {
    /// Creates a nonzero command identity.
    ///
    /// # Errors
    ///
    /// Returns [`SessionCommandIdError`] for zero, which is reserved as an invalid sentinel.
    pub const fn new(value: u64) -> Result<Self, SessionCommandIdError> {
        if value == 0 {
            return Err(SessionCommandIdError);
        }
        Ok(Self(value))
    }

    /// Returns the validated ordinal.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Rejection of the reserved zero command identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("session command identity must be nonzero")]
pub struct SessionCommandIdError;

/// Raw exact-bit settings parsed by a frontend before controller validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunSettingsInput {
    timestep_bits: FloatBits,
    velocity_iterations: u32,
    position_iterations: u32,
    particle_iterations: u32,
}

impl RunSettingsInput {
    /// Creates an unvalidated settings candidate. Validation happens before backend effects.
    #[must_use]
    pub const fn new(
        timestep_bits: FloatBits,
        velocity_iterations: u32,
        position_iterations: u32,
        particle_iterations: u32,
    ) -> Self {
        Self {
            timestep_bits,
            velocity_iterations,
            position_iterations,
            particle_iterations,
        }
    }

    fn validate(self) -> Result<RunSettings, SessionControllerError> {
        RunSettings::new(
            self.timestep_bits,
            self.velocity_iterations,
            self.position_iterations,
            self.particle_iterations,
        )
        .map_err(|_| SessionControllerError::new(SessionControllerErrorKind::InvalidRunSettings))
    }
}

/// Closed commands accepted from headless or visual frontend adapters.
#[derive(Debug, Clone)]
pub enum SessionCommand {
    /// Select one already validated, immutable resolved plan.
    Select {
        /// Exact engine-neutral run input.
        resolved: ResolvedScenario,
    },
    /// Enter running state without tying logical work to a render frame.
    Run,
    /// Pause the session without a backend tick or checkpoint.
    Pause,
    /// Pause if necessary, execute exactly one logical action, and stay paused.
    StepOnce,
    /// Destroy and reconstruct the selected session from identical resolved bytes.
    Restart,
    /// Validate settings and reconstruct from a matching newly resolved plan.
    ApplySettingsAndRestart {
        /// Raw settings candidate that must validate before any backend effect.
        settings: RunSettingsInput,
        /// Canonical replacement whose identity must differ only by settings-derived content.
        resolved: ResolvedScenario,
    },
    /// Apply one declared typed scenario action by stable action identity.
    ApplyScenarioAction {
        /// Stable action identity from the selected resolved plan.
        action_id: ScenarioActionId,
    },
    /// Capture one currently reachable declared checkpoint.
    CaptureCheckpoint {
        /// Stable checkpoint identity from the selected resolved plan.
        checkpoint_id: CheckpointId,
    },
}

impl SessionCommand {
    const fn kind(&self) -> SessionCommandKind {
        match self {
            Self::Select { .. } => SessionCommandKind::Select,
            Self::Run => SessionCommandKind::Run,
            Self::Pause => SessionCommandKind::Pause,
            Self::StepOnce => SessionCommandKind::StepOnce,
            Self::Restart => SessionCommandKind::Restart,
            Self::ApplySettingsAndRestart { .. } => SessionCommandKind::ApplySettingsAndRestart,
            Self::ApplyScenarioAction { .. } => SessionCommandKind::ApplyScenarioAction,
            Self::CaptureCheckpoint { .. } => SessionCommandKind::CaptureCheckpoint,
        }
    }
}

/// Backend failure severity used to select the controller error state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionBackendErrorKind {
    /// The selected plan remains available for an explicit retry or restart.
    Recoverable,
    /// Protocol, provenance, or invariant failure makes the run inadmissible.
    Harness,
}

/// Bounded backend operation categories suitable for frontend diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionBackendErrorCategory {
    /// Session construction failed.
    Create,
    /// A typed action failed.
    Action,
    /// Deterministic checkpoint capture failed.
    Capture,
    /// A protocol or provenance invariant failed.
    Protocol,
    /// A reviewed work or collection bound was exceeded.
    ResourceLimit,
}

/// Bounded classified backend failure without raw records or unbounded diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("session backend failure: {kind:?}/{category:?}")]
pub struct SessionBackendError {
    kind: SessionBackendErrorKind,
    category: SessionBackendErrorCategory,
}

impl SessionBackendError {
    /// Creates a recoverable backend failure.
    #[must_use]
    pub const fn recoverable(category: SessionBackendErrorCategory) -> Self {
        Self {
            kind: SessionBackendErrorKind::Recoverable,
            category,
        }
    }

    /// Creates a harness backend failure.
    #[must_use]
    pub const fn harness(category: SessionBackendErrorCategory) -> Self {
        Self {
            kind: SessionBackendErrorKind::Harness,
            category,
        }
    }

    /// Returns the failure severity.
    #[must_use]
    pub const fn kind(self) -> SessionBackendErrorKind {
        self.kind
    }

    /// Returns the bounded operation category.
    #[must_use]
    pub const fn category(self) -> SessionBackendErrorCategory {
        self.category
    }
}

/// Narrow effect boundary implemented by native and supervised oracle adapters.
pub trait SessionBackend {
    /// Owned semantic checkpoint returned to headless and visual consumers.
    type Checkpoint;

    /// Constructs a fresh backend session from exact validated resolved input.
    ///
    /// Implementations execute setup actions here and must leave no partial live session on error.
    ///
    /// # Errors
    ///
    /// Returns a bounded classified backend failure.
    fn create_session(&mut self, resolved: &ResolvedScenario) -> Result<(), SessionBackendError>;

    /// Destroys the complete current backend session. This operation is intentionally infallible.
    fn destroy_session(&mut self);

    /// Executes one declared action transactionally.
    ///
    /// # Errors
    ///
    /// Returns a bounded classified backend failure without committing a partial action.
    fn execute_action(&mut self, action: &ScheduledAction) -> Result<(), SessionBackendError>;

    /// Captures semantic state without advancing simulation.
    ///
    /// # Errors
    ///
    /// Returns a bounded classified backend failure.
    fn capture_checkpoint(
        &mut self,
        checkpoint: &SessionCheckpointIdentity,
    ) -> Result<Self::Checkpoint, SessionBackendError>;
}

/// Stable checkpoint identity bound to both action and logical-step ordinals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCheckpointIdentity {
    checkpoint_id: CheckpointId,
    after_action_id: ScenarioActionId,
    logical_step: u32,
}

impl SessionCheckpointIdentity {
    fn from_declaration(declaration: &CheckpointDeclaration) -> Self {
        Self {
            checkpoint_id: declaration.checkpoint_id().clone(),
            after_action_id: declaration.after_action_id().clone(),
            logical_step: declaration.logical_step(),
        }
    }

    /// Returns the stable checkpoint ID.
    #[must_use]
    pub const fn checkpoint_id(&self) -> &CheckpointId {
        &self.checkpoint_id
    }

    /// Returns the stable action boundary captured by this checkpoint.
    #[must_use]
    pub const fn after_action_id(&self) -> &ScenarioActionId {
        &self.after_action_id
    }

    /// Returns the explicit one-based logical-step ordinal.
    #[must_use]
    pub const fn logical_step(&self) -> u32 {
        self.logical_step
    }
}

/// One successful owned semantic capture and its controller-bound identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCapture<T> {
    identity: SessionCheckpointIdentity,
    value: T,
}

impl<T> SessionCapture<T> {
    /// Returns the stable deterministic capture identity.
    #[must_use]
    pub const fn identity(&self) -> &SessionCheckpointIdentity {
        &self.identity
    }

    /// Returns the backend-owned semantic checkpoint value.
    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }
}

/// Stable controller rejection categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionControllerErrorKind {
    /// A command was submitted while another effect was in flight.
    ReentrantCommand,
    /// A previously consumed command identity was replayed.
    StaleCommand,
    /// A command skipped the next expected identity.
    FutureCommand,
    /// Every representable command identity has been consumed.
    CommandCounterExhausted,
    /// The command is unavailable from the current state.
    InvalidTransition,
    /// Timestep or iteration settings are invalid.
    InvalidRunSettings,
    /// Replacement bytes identify a different scenario or do not carry the validated settings.
    ReplacementRunMismatch,
    /// No resolved scenario is available for the requested effect.
    MissingSelection,
    /// The requested stable scenario action is absent or not currently reachable.
    InvalidScenarioAction,
    /// The checkpoint is absent, not current, or was already captured.
    InvalidCheckpoint,
    /// A checked logical ordinal could not advance.
    LogicalCounterExhausted,
    /// The backend returned a classified failure.
    Backend,
}

/// Bounded controller failure with an optional classified backend cause.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("session controller failure: {kind:?}")]
pub struct SessionControllerError {
    kind: SessionControllerErrorKind,
    maybe_backend: Option<SessionBackendError>,
}

impl SessionControllerError {
    const fn new(kind: SessionControllerErrorKind) -> Self {
        Self {
            kind,
            maybe_backend: None,
        }
    }

    const fn backend(error: SessionBackendError) -> Self {
        Self {
            kind: SessionControllerErrorKind::Backend,
            maybe_backend: Some(error),
        }
    }

    /// Returns the stable controller failure category.
    #[must_use]
    pub const fn kind(self) -> SessionControllerErrorKind {
        self.kind
    }

    /// Returns the classified backend cause when one exists.
    #[must_use]
    pub const fn maybe_backend(self) -> Option<SessionBackendError> {
        self.maybe_backend
    }
}

/// State returned after one accepted command settles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionCommandOutcome {
    state: SessionState,
}

impl SessionCommandOutcome {
    /// Returns the settled controller state.
    #[must_use]
    pub const fn state(self) -> SessionState {
        self.state
    }
}

/// Thin imperative shell over the pure state machine and one backend implementation.
pub struct SessionController<B: SessionBackend> {
    backend: B,
    state: SessionState,
    maybe_selected: Option<ResolvedScenario>,
    next_logical_step: u32,
    maybe_next_command_id: Option<SessionCommandId>,
    command_in_flight: bool,
    captures: Vec<SessionCapture<B::Checkpoint>>,
}

impl<B: SessionBackend> SessionController<B> {
    /// Creates an empty controller that accepts command identity 1 first.
    #[must_use]
    pub const fn new(backend: B) -> Self {
        Self {
            backend,
            state: SessionState::NoSelection,
            maybe_selected: None,
            next_logical_step: 1,
            maybe_next_command_id: Some(SessionCommandId(1)),
            command_in_flight: false,
            captures: Vec::new(),
        }
    }

    /// Returns the current closed state.
    #[must_use]
    pub const fn state(&self) -> SessionState {
        self.state
    }

    /// Returns the next exact frontend command identity, or `None` after exhaustion.
    #[must_use]
    pub const fn next_command_id(&self) -> Option<SessionCommandId> {
        self.maybe_next_command_id
    }

    /// Returns the selected exact resolved plan.
    #[must_use]
    pub const fn selected(&self) -> Option<&ResolvedScenario> {
        self.maybe_selected.as_ref()
    }

    /// Returns the number of successfully completed logical actions.
    #[must_use]
    pub const fn completed_logical_steps(&self) -> u32 {
        match self.next_logical_step.checked_sub(1) {
            Some(completed) => completed,
            None => 0,
        }
    }

    /// Returns successful captures in command order.
    #[must_use]
    pub fn captures(&self) -> &[SessionCapture<B::Checkpoint>] {
        &self.captures
    }

    /// Returns a shared backend reference for adapter-specific inspection.
    #[must_use]
    pub const fn backend(&self) -> &B {
        &self.backend
    }

    /// Returns an exclusive backend reference for adapter-specific preparation.
    #[must_use]
    pub const fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    /// Submits one typed command under exact monotonic admission.
    ///
    /// Command identities are consumed once admitted even when command validation or a backend
    /// effect fails, preventing a stale retry from becoming valid after state changes.
    ///
    /// # Errors
    ///
    /// Returns a bounded admission, transition, validation, identity, or backend failure.
    pub fn submit(
        &mut self,
        command_id: SessionCommandId,
        command: SessionCommand,
    ) -> Result<SessionCommandOutcome, SessionControllerError> {
        if self.command_in_flight {
            return Err(SessionControllerError::new(
                SessionControllerErrorKind::ReentrantCommand,
            ));
        }
        let Some(expected) = self.maybe_next_command_id else {
            return Err(SessionControllerError::new(
                SessionControllerErrorKind::CommandCounterExhausted,
            ));
        };
        if command_id < expected {
            return Err(SessionControllerError::new(
                SessionControllerErrorKind::StaleCommand,
            ));
        }
        if command_id > expected {
            return Err(SessionControllerError::new(
                SessionControllerErrorKind::FutureCommand,
            ));
        }
        self.maybe_next_command_id = command_id.get().checked_add(1).map(SessionCommandId);
        self.command_in_flight = true;
        let result = self.dispatch(command);
        self.command_in_flight = false;
        result
    }

    /// Advances one logical action only while the controller is explicitly running.
    ///
    /// # Errors
    ///
    /// Returns a bounded transition, counter, or backend failure.
    pub fn advance_running(&mut self) -> Result<SessionCommandOutcome, SessionControllerError> {
        if self.command_in_flight {
            return Err(SessionControllerError::new(
                SessionControllerErrorKind::ReentrantCommand,
            ));
        }
        if self.state != SessionState::Running {
            return Err(SessionControllerError::new(
                SessionControllerErrorKind::InvalidTransition,
            ));
        }
        self.command_in_flight = true;
        let result = self.execute_next_logical(SessionState::Running);
        self.command_in_flight = false;
        result.map(|()| SessionCommandOutcome { state: self.state })
    }

    fn dispatch(
        &mut self,
        command: SessionCommand,
    ) -> Result<SessionCommandOutcome, SessionControllerError> {
        let planned = transition(self.state, command.kind()).map_err(|_| {
            SessionControllerError::new(SessionControllerErrorKind::InvalidTransition)
        })?;
        match command {
            SessionCommand::Select { resolved } => self.select(resolved, planned)?,
            SessionCommand::Run | SessionCommand::Pause => self.state = planned.settled(),
            SessionCommand::StepOnce => {
                if self.state == SessionState::Running {
                    self.state = SessionState::ReadyPaused;
                }
                self.state = planned.transient();
                self.execute_next_logical(planned.settled())?;
            }
            SessionCommand::Restart => self.restart(None, planned)?,
            SessionCommand::ApplySettingsAndRestart { settings, resolved } => {
                let validated = settings.validate()?;
                validate_resolved_settings(&resolved)?;
                if resolved.identity().settings() != validated
                    || !same_run_except_settings(self.maybe_selected.as_ref(), Some(&resolved))
                {
                    return Err(SessionControllerError::new(
                        SessionControllerErrorKind::ReplacementRunMismatch,
                    ));
                }
                self.restart(Some(resolved), planned)?;
            }
            SessionCommand::ApplyScenarioAction { action_id } => {
                self.apply_scenario_action(&action_id, planned)?;
            }
            SessionCommand::CaptureCheckpoint { checkpoint_id } => {
                self.capture_checkpoint(&checkpoint_id, planned)?;
            }
        }
        Ok(SessionCommandOutcome { state: self.state })
    }

    fn select(
        &mut self,
        resolved: ResolvedScenario,
        planned: SessionTransition,
    ) -> Result<(), SessionControllerError> {
        validate_resolved_settings(&resolved)?;
        self.state = planned.transient();
        if self.maybe_selected.is_some() {
            self.backend.destroy_session();
        }
        if let Err(error) = self.backend.create_session(&resolved) {
            return Err(self.record_backend_failure(error));
        }
        self.maybe_selected = Some(resolved);
        self.next_logical_step = 1;
        self.captures.clear();
        self.state = planned.settled();
        Ok(())
    }

    fn restart(
        &mut self,
        replacement: Option<ResolvedScenario>,
        planned: SessionTransition,
    ) -> Result<(), SessionControllerError> {
        if replacement.is_none() && self.maybe_selected.is_none() {
            return Err(SessionControllerError::new(
                SessionControllerErrorKind::MissingSelection,
            ));
        }
        self.state = planned.transient();
        self.backend.destroy_session();
        if let Some(resolved) = replacement {
            if let Err(error) = self.backend.create_session(&resolved) {
                return Err(self.record_backend_failure(error));
            }
            self.maybe_selected = Some(resolved);
        } else {
            let Some(resolved) = self.maybe_selected.as_ref() else {
                return Err(SessionControllerError::new(
                    SessionControllerErrorKind::MissingSelection,
                ));
            };
            if let Err(error) = self.backend.create_session(resolved) {
                return Err(self.record_backend_failure(error));
            }
        }
        self.next_logical_step = 1;
        self.captures.clear();
        self.state = planned.settled();
        Ok(())
    }

    fn apply_scenario_action(
        &mut self,
        action_id: &ScenarioActionId,
        planned: SessionTransition,
    ) -> Result<(), SessionControllerError> {
        let Some(resolved) = self.maybe_selected.as_ref() else {
            return Err(SessionControllerError::new(
                SessionControllerErrorKind::MissingSelection,
            ));
        };
        let Some(action) = resolved
            .actions()
            .iter()
            .find(|action| action.action_id() == action_id)
            .cloned()
        else {
            return Err(SessionControllerError::new(
                SessionControllerErrorKind::InvalidScenarioAction,
            ));
        };
        let ActionSchedule::LogicalStep { ordinal } = action.schedule() else {
            return Err(SessionControllerError::new(
                SessionControllerErrorKind::InvalidScenarioAction,
            ));
        };
        if ordinal != self.next_logical_step {
            return Err(SessionControllerError::new(
                SessionControllerErrorKind::InvalidScenarioAction,
            ));
        }
        self.state = planned.transient();
        if let Err(error) = self.backend.execute_action(&action) {
            return Err(self.record_backend_failure(error));
        }
        self.advance_logical_counter()?;
        self.state = if self.has_next_logical_action() {
            planned.settled()
        } else {
            SessionState::Completed
        };
        Ok(())
    }

    fn capture_checkpoint(
        &mut self,
        checkpoint_id: &CheckpointId,
        planned: SessionTransition,
    ) -> Result<(), SessionControllerError> {
        let Some(resolved) = self.maybe_selected.as_ref() else {
            return Err(SessionControllerError::new(
                SessionControllerErrorKind::MissingSelection,
            ));
        };
        let completed = self.next_logical_step.checked_sub(1).ok_or_else(|| {
            SessionControllerError::new(SessionControllerErrorKind::LogicalCounterExhausted)
        })?;
        let Some(declaration) = resolved
            .checkpoints()
            .iter()
            .find(|checkpoint| checkpoint.checkpoint_id() == checkpoint_id)
        else {
            return Err(SessionControllerError::new(
                SessionControllerErrorKind::InvalidCheckpoint,
            ));
        };
        let identity = SessionCheckpointIdentity::from_declaration(declaration);
        if identity.logical_step() != completed
            || self
                .captures
                .iter()
                .any(|capture| capture.identity() == &identity)
        {
            return Err(SessionControllerError::new(
                SessionControllerErrorKind::InvalidCheckpoint,
            ));
        }
        self.state = planned.transient();
        let value = match self.backend.capture_checkpoint(&identity) {
            Ok(value) => value,
            Err(error) => return Err(self.record_backend_failure(error)),
        };
        self.captures.push(SessionCapture { identity, value });
        self.state = planned.settled();
        Ok(())
    }

    fn execute_next_logical(
        &mut self,
        success_state: SessionState,
    ) -> Result<(), SessionControllerError> {
        let Some(resolved) = self.maybe_selected.as_ref() else {
            return Err(SessionControllerError::new(
                SessionControllerErrorKind::MissingSelection,
            ));
        };
        let Some(action) = resolved
            .actions()
            .iter()
            .find(|action| {
                matches!(
                    action.schedule(),
                    ActionSchedule::LogicalStep { ordinal }
                        if ordinal == self.next_logical_step
                )
            })
            .cloned()
        else {
            self.state = SessionState::Completed;
            return Ok(());
        };
        if let Err(error) = self.backend.execute_action(&action) {
            return Err(self.record_backend_failure(error));
        }
        self.advance_logical_counter()?;
        self.state = if self.has_next_logical_action() {
            success_state
        } else {
            SessionState::Completed
        };
        Ok(())
    }

    fn advance_logical_counter(&mut self) -> Result<(), SessionControllerError> {
        self.next_logical_step = self.next_logical_step.checked_add(1).ok_or_else(|| {
            SessionControllerError::new(SessionControllerErrorKind::LogicalCounterExhausted)
        })?;
        Ok(())
    }

    fn has_next_logical_action(&self) -> bool {
        self.maybe_selected.as_ref().is_some_and(|resolved| {
            resolved.actions().iter().any(|action| {
                matches!(
                    action.schedule(),
                    ActionSchedule::LogicalStep { ordinal }
                        if ordinal == self.next_logical_step
                )
            })
        })
    }

    fn record_backend_failure(&mut self, error: SessionBackendError) -> SessionControllerError {
        self.state = match error.kind() {
            SessionBackendErrorKind::Recoverable => SessionState::RecoverableError,
            SessionBackendErrorKind::Harness => SessionState::HarnessFailure,
        };
        SessionControllerError::backend(error)
    }
}

fn validate_resolved_settings(resolved: &ResolvedScenario) -> Result<(), SessionControllerError> {
    let settings = resolved.identity().settings();
    RunSettings::new(
        settings.timestep_bits(),
        settings.velocity_iterations(),
        settings.position_iterations(),
        settings.particle_iterations(),
    )
    .map(|_| ())
    .map_err(|_| SessionControllerError::new(SessionControllerErrorKind::InvalidRunSettings))
}

fn same_run_except_settings(
    maybe_current: Option<&ResolvedScenario>,
    maybe_replacement: Option<&ResolvedScenario>,
) -> bool {
    let (Some(current_scenario), Some(replacement_scenario)) = (maybe_current, maybe_replacement)
    else {
        return false;
    };
    let current = current_scenario.identity();
    let replacement = replacement_scenario.identity();
    current.catalog_schema_version() == replacement.catalog_schema_version()
        && current.slug() == replacement.slug()
        && current.scenario_version() == replacement.scenario_version()
        && current.generator_id() == replacement.generator_id()
        && current.generator_version() == replacement.generator_version()
        && current.maybe_seed() == replacement.maybe_seed()
        && current_scenario.entities() == replacement_scenario.entities()
        && current_scenario.actions() == replacement_scenario.actions()
        && current_scenario.checkpoints() == replacement_scenario.checkpoints()
}

#[cfg(test)]
mod tests;
