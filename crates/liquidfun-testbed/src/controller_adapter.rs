//! Typed visual-adapter boundary for the renderer-neutral session controller.

#![allow(
    missing_docs,
    reason = "closed controller-action and enabledness fields mirror the UI contract"
)]

use liquidfun_differential::{SessionCommand, SessionCommandKind, SessionState, transition};
use liquidfun_test_protocol::{CheckpointId, ResolvedScenario, ScenarioActionId};

pub const MODULE_NAME: &str = "controller_adapter";
pub const SESSION_PAUSED_LABEL: &str = "Session paused";
pub const PARTICLE_PAUSE_ACTION_LABEL: &str = "Particle system pause action";

/// UI-owned request whose payload is already a validated stable identity.
#[derive(Debug, Clone)]
pub enum ControllerAction {
    Select(ResolvedScenario),
    Run,
    Pause,
    StepOnce,
    Restart,
    CaptureCheckpoint(CheckpointId),
    ApplySettingsAndRestart {
        settings: liquidfun_differential::RunSettingsInput,
        resolved: ResolvedScenario,
    },
    ApplyScenarioAction(ScenarioActionId),
}

impl ControllerAction {
    const fn kind(&self) -> SessionCommandKind {
        match self {
            Self::Select(_) => SessionCommandKind::Select,
            Self::Run => SessionCommandKind::Run,
            Self::Pause => SessionCommandKind::Pause,
            Self::StepOnce => SessionCommandKind::StepOnce,
            Self::Restart => SessionCommandKind::Restart,
            Self::CaptureCheckpoint(_) => SessionCommandKind::CaptureCheckpoint,
            Self::ApplySettingsAndRestart { .. } => SessionCommandKind::ApplySettingsAndRestart,
            Self::ApplyScenarioAction(_) => SessionCommandKind::ApplyScenarioAction,
        }
    }

    fn into_command(self) -> SessionCommand {
        match self {
            Self::Select(resolved) => SessionCommand::Select { resolved },
            Self::Run => SessionCommand::Run,
            Self::Pause => SessionCommand::Pause,
            Self::StepOnce => SessionCommand::StepOnce,
            Self::Restart => SessionCommand::Restart,
            Self::CaptureCheckpoint(checkpoint_id) => {
                SessionCommand::CaptureCheckpoint { checkpoint_id }
            }
            Self::ApplySettingsAndRestart { settings, resolved } => {
                SessionCommand::ApplySettingsAndRestart { settings, resolved }
            }
            Self::ApplyScenarioAction(action_id) => {
                SessionCommand::ApplyScenarioAction { action_id }
            }
        }
    }
}

/// Pure enabledness projection over the controller-owned closed state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControllerProjection {
    enabled: [bool; 8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlCapability {
    SelectScenario,
    Run,
    Pause,
    StepOnce,
    Restart,
    Capture,
    ApplySettings,
    ApplyScenarioAction,
}

impl ControlCapability {
    const fn index(self) -> usize {
        match self {
            Self::SelectScenario => 0,
            Self::Run => 1,
            Self::Pause => 2,
            Self::StepOnce => 3,
            Self::Restart => 4,
            Self::Capture => 5,
            Self::ApplySettings => 6,
            Self::ApplyScenarioAction => 7,
        }
    }
}

impl ControllerProjection {
    #[must_use]
    pub const fn from_state(state: SessionState) -> Self {
        let ready = matches!(state, SessionState::ReadyPaused);
        let running = matches!(state, SessionState::Running);
        let settled = matches!(
            state,
            SessionState::ReadyPaused
                | SessionState::Running
                | SessionState::Completed
                | SessionState::RecoverableError
                | SessionState::HarnessFailure
        );
        Self {
            enabled: [
                matches!(
                    state,
                    SessionState::NoSelection
                        | SessionState::ReadyPaused
                        | SessionState::Completed
                        | SessionState::RecoverableError
                        | SessionState::HarnessFailure
                ),
                ready,
                running,
                ready || running,
                settled,
                matches!(
                    state,
                    SessionState::ReadyPaused | SessionState::Running | SessionState::Completed
                ),
                settled,
                ready || running,
            ],
        }
    }

    #[must_use]
    pub const fn enabled(self, capability: ControlCapability) -> bool {
        self.enabled[capability.index()]
    }
}

/// Stable adapter rejection categories safe for inline UI diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ControllerAdapterError {
    #[error("a controller command is already in flight")]
    CommandInFlight,
    #[error("the controller command is unavailable in the current state")]
    InvalidTransition,
}

/// Single-flight command mapper. It never executes simulation work itself.
#[derive(Debug, Default)]
pub struct ControllerAdapter {
    command_in_flight: bool,
}

impl ControllerAdapter {
    #[must_use]
    pub const fn command_in_flight(&self) -> bool {
        self.command_in_flight
    }

    /// Admits one typed command for submission by the external controller owner.
    ///
    /// # Errors
    ///
    /// Returns a bounded error for duplicate in-flight work or a closed-state rejection.
    pub fn begin(
        &mut self,
        state: SessionState,
        action: ControllerAction,
    ) -> Result<SessionCommand, ControllerAdapterError> {
        if self.command_in_flight {
            return Err(ControllerAdapterError::CommandInFlight);
        }
        transition(state, action.kind()).map_err(|_| ControllerAdapterError::InvalidTransition)?;
        self.command_in_flight = true;
        Ok(action.into_command())
    }

    /// Clears only adapter admission state after the controller returns an outcome.
    pub const fn complete(&mut self) {
        self.command_in_flight = false;
    }
}
