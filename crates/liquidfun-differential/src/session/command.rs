use liquidfun_test_protocol::{
    CheckpointId, FloatBits, ResolvedScenario, RunSettings, ScenarioActionId,
};

use super::{SessionCommandKind, SessionControllerError, SessionControllerErrorKind};

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

    pub(super) fn validate(self) -> Result<RunSettings, SessionControllerError> {
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
    pub(super) const fn kind(&self) -> SessionCommandKind {
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

pub(super) fn validate_resolved_settings(
    resolved: &ResolvedScenario,
) -> Result<(), SessionControllerError> {
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
