//! Pure run-session state transitions.

/// Closed renderer-neutral lifecycle for one run session.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum SessionState {
    /// No resolved scenario has been selected.
    #[default]
    NoSelection,
    /// A validated selection or restart is being constructed.
    Resolving,
    /// A selected session is ready and does not advance automatically.
    ReadyPaused,
    /// The controller may advance through explicit driver calls.
    Running,
    /// Exactly one logical action is in flight.
    Stepping,
    /// One deterministic checkpoint capture is in flight.
    Comparing,
    /// Every logical action completed successfully.
    Completed,
    /// A backend rejected an action through a recoverable category.
    RecoverableError,
    /// A backend invariant, protocol, or provenance failure stopped the session.
    HarnessFailure,
}

/// Payload-independent command identity used by the pure transition core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionCommandKind {
    /// Select one resolved scenario.
    Select,
    /// Enter running state without executing an implicit tick.
    Run,
    /// Stop automatic advancement without a backend effect.
    Pause,
    /// Execute exactly one logical action and settle paused.
    StepOnce,
    /// Reconstruct the selected scenario from its exact bytes.
    Restart,
    /// Replace settings through a newly validated resolved plan.
    ApplySettingsAndRestart,
    /// Apply one declared typed scenario action.
    ApplyScenarioAction,
    /// Capture one declared deterministic checkpoint.
    CaptureCheckpoint,
}

/// Pure transient and settled states for one admitted command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionTransition {
    transient: SessionState,
    settled: SessionState,
}

impl SessionTransition {
    const fn new(transient: SessionState, settled: SessionState) -> Self {
        Self { transient, settled }
    }

    /// Returns the state visible while the command effect is in flight.
    #[must_use]
    pub const fn transient(self) -> SessionState {
        self.transient
    }

    /// Returns the state after the command effect succeeds.
    #[must_use]
    pub const fn settled(self) -> SessionState {
        self.settled
    }
}

/// Stable pure-transition failure categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionTransitionErrorKind {
    /// The command is unavailable from the current closed state.
    InvalidTransition,
}

/// Bounded pure-transition failure without command payload disclosure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("invalid run-session state transition")]
pub struct SessionTransitionError {
    kind: SessionTransitionErrorKind,
}

impl SessionTransitionError {
    /// Returns the stable failure category.
    #[must_use]
    pub const fn kind(self) -> SessionTransitionErrorKind {
        self.kind
    }
}

/// Computes one closed controller transition without backend effects.
///
/// # Errors
///
/// Returns [`SessionTransitionError`] when the command is unavailable from `state`.
pub const fn transition(
    state: SessionState,
    command: SessionCommandKind,
) -> Result<SessionTransition, SessionTransitionError> {
    let transition = match (state, command) {
        (SessionState::NoSelection, SessionCommandKind::Select) => {
            SessionTransition::new(SessionState::Resolving, SessionState::ReadyPaused)
        }
        (
            SessionState::ReadyPaused
            | SessionState::Completed
            | SessionState::RecoverableError
            | SessionState::HarnessFailure,
            SessionCommandKind::Select,
        )
        | (
            SessionState::ReadyPaused
            | SessionState::Running
            | SessionState::Completed
            | SessionState::RecoverableError
            | SessionState::HarnessFailure,
            SessionCommandKind::Restart | SessionCommandKind::ApplySettingsAndRestart,
        ) => SessionTransition::new(SessionState::Resolving, SessionState::ReadyPaused),
        (SessionState::ReadyPaused, SessionCommandKind::Run) => {
            SessionTransition::new(SessionState::Running, SessionState::Running)
        }
        (SessionState::Running, SessionCommandKind::Pause) => {
            SessionTransition::new(SessionState::ReadyPaused, SessionState::ReadyPaused)
        }
        (SessionState::ReadyPaused | SessionState::Running, SessionCommandKind::StepOnce) => {
            SessionTransition::new(SessionState::Stepping, SessionState::ReadyPaused)
        }
        (
            SessionState::ReadyPaused | SessionState::Running,
            SessionCommandKind::ApplyScenarioAction,
        ) => SessionTransition::new(state, state),
        (
            SessionState::ReadyPaused | SessionState::Running | SessionState::Completed,
            SessionCommandKind::CaptureCheckpoint,
        ) => SessionTransition::new(SessionState::Comparing, state),
        _ => {
            return Err(SessionTransitionError {
                kind: SessionTransitionErrorKind::InvalidTransition,
            });
        }
    };
    Ok(transition)
}
