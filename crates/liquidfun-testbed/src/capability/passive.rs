//! Passive semantic inputs consumed by the renderer adapter.

use liquidfun_differential::{
    ComparisonLimits, ComparisonModel, SessionBackend, SessionBackendError, SessionController,
    SessionState, compare_canonical_checkpoints,
};
use liquidfun_test_protocol::{
    CanonicalCheckpoint, CheckpointId, CheckpointPosition, FloatBits, Phase4PolicyProfile,
    RequestId, ResolvedScenario, ScheduledAction, Sha256Hex,
};
use serde::Serialize;

use super::CapabilityError;
use super::fixture::FixtureSnapshot;

/// Effect-free backend used to expose a real controller input to the adapter gate.
pub(super) struct CapabilityBackend;

impl SessionBackend for CapabilityBackend {
    type Checkpoint = ();

    fn create_session(&mut self, _resolved: &ResolvedScenario) -> Result<(), SessionBackendError> {
        Ok(())
    }

    fn destroy_session(&mut self) {}

    fn execute_action(&mut self, _action: &ScheduledAction) -> Result<(), SessionBackendError> {
        Ok(())
    }

    fn capture_checkpoint(
        &mut self,
        _checkpoint: &liquidfun_differential::SessionCheckpointIdentity,
    ) -> Result<Self::Checkpoint, SessionBackendError> {
        Ok(())
    }
}

/// Immutable presentation-only projection of controller and comparison state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct PassiveInputSnapshot {
    pub(super) session_state: &'static str,
    pub(super) logical_steps: u32,
    pub(super) captures: usize,
    pub(super) comparison_state: &'static str,
    pub(super) comparison_entries: usize,
}

pub(super) fn build_passive_inputs(
    fixture: &FixtureSnapshot,
) -> Result<(SessionController<CapabilityBackend>, ComparisonModel), CapabilityError> {
    let controller = SessionController::new(CapabilityBackend);
    let rust = empty_checkpoint(&fixture.sha256)?;
    let oracle = empty_checkpoint(&fixture.sha256)?;
    let policy = Phase4PolicyProfile::parse_toml(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../protocol/tolerances/phase4-v1.toml"
    )))
    .map_err(|_| CapabilityError::InvalidComparison)?;
    let comparison =
        compare_canonical_checkpoints(&rust, &oracle, &policy, ComparisonLimits::phase11_default())
            .map_err(|_| CapabilityError::InvalidComparison)?;
    Ok((controller, comparison))
}

pub(super) fn observe_passive_inputs(
    controller: &SessionController<CapabilityBackend>,
    comparison: &ComparisonModel,
) -> PassiveInputSnapshot {
    PassiveInputSnapshot {
        session_state: session_state_name(controller.state()),
        logical_steps: controller.completed_logical_steps(),
        captures: controller.captures().len(),
        comparison_state: comparison_state_name(comparison.state()),
        comparison_entries: comparison.entries().len(),
    }
}

fn empty_checkpoint(resolved_sha256: &str) -> Result<CanonicalCheckpoint, CapabilityError> {
    CanonicalCheckpoint::new(
        RequestId::new("testbed-capability").map_err(|_| CapabilityError::InvalidComparison)?,
        Sha256Hex::new(resolved_sha256.to_owned())
            .map_err(|_| CapabilityError::InvalidComparison)?,
        CheckpointId::new("checkpoint-0001").map_err(|_| CapabilityError::InvalidComparison)?,
        CheckpointPosition::LogicalStep { ordinal: 1 },
        FloatBits::from_f32(1.0 / 60.0),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .map_err(|_| CapabilityError::InvalidComparison)
}

const fn session_state_name(state: SessionState) -> &'static str {
    match state {
        SessionState::NoSelection => "no_selection",
        SessionState::Resolving => "resolving",
        SessionState::ReadyPaused => "ready_paused",
        SessionState::Running => "running",
        SessionState::Stepping => "stepping",
        SessionState::Comparing => "comparing",
        SessionState::Completed => "completed",
        SessionState::RecoverableError => "recoverable_error",
        SessionState::HarnessFailure => "harness_failure",
    }
}

const fn comparison_state_name(state: liquidfun_differential::ComparisonState) -> &'static str {
    match state {
        liquidfun_differential::ComparisonState::ExactMatch => "exact_match",
        liquidfun_differential::ComparisonState::WithinPolicy => "within_policy",
        liquidfun_differential::ComparisonState::PhysicsMismatch => "physics_mismatch",
        liquidfun_differential::ComparisonState::RustOnly => "rust_only",
        liquidfun_differential::ComparisonState::OracleOnly => "oracle_only",
    }
}
