//! Transactional session construction and fail-closed replay.

mod identity;
mod objects;
mod particles;
mod rigid_actions;

use std::panic::{AssertUnwindSafe, catch_unwind};

use liquidfun::math::Vec2;
use liquidfun::rope::Rope;
use liquidfun::{
    BodyId, FixtureId, JointId, NoDecisionHook, ParticleGroupId, ParticleId, ParticleSystemId,
    StepConfiguration, StepLimits, World,
};
use liquidfun_test_protocol::{
    ResolvedScenario, RigidWorldAction, ScenarioId, ScheduledAction, Sha256Hex,
    decode_resolved_scenario,
};

use crate::session::SessionCheckpointIdentity;
use crate::{SessionBackendError, SessionBackendErrorCategory};

use super::capture::capture_checkpoint;

const MAXIMUM_REPLAY_ACTIONS: usize = 128;

/// Transactional native executor for one exact resolved catalog plan.
#[derive(Default)]
pub struct NativeCatalogBackend {
    maybe_resolved: Option<ResolvedScenario>,
    completed_logical_actions: Vec<ScheduledAction>,
    maybe_session: Option<NativeSession>,
}

impl NativeCatalogBackend {
    /// Creates an empty backend without allocating a world.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            maybe_resolved: None,
            completed_logical_actions: Vec::new(),
            maybe_session: None,
        }
    }

    /// Returns whether one fully constructed native session is live.
    #[must_use]
    pub const fn is_session_active(&self) -> bool {
        self.maybe_session.is_some()
    }

    /// Strictly decodes asserted canonical bytes and constructs a session only after validation.
    ///
    /// # Errors
    ///
    /// Returns a bounded protocol or action failure and leaves no session alive.
    pub fn create_canonical(
        &mut self,
        canonical_bytes: &[u8],
        asserted_sha256: &Sha256Hex,
    ) -> Result<(), SessionBackendError> {
        self.destroy();
        let decoded = decode_resolved_scenario(canonical_bytes, asserted_sha256)
            .map_err(|_error| harness(SessionBackendErrorCategory::Protocol))?;
        let session = build_session(&decoded, &[])?;
        self.maybe_resolved = Some(decoded);
        self.maybe_session = Some(session);
        Ok(())
    }

    pub(crate) fn create(
        &mut self,
        resolved: &ResolvedScenario,
    ) -> Result<(), SessionBackendError> {
        self.create_canonical(
            resolved.canonical_bytes(),
            resolved.identity().content_sha256(),
        )?;
        if self.maybe_resolved.as_ref() != Some(resolved) {
            self.destroy();
            return Err(protocol_failure());
        }
        Ok(())
    }

    pub(crate) fn destroy(&mut self) {
        self.maybe_session = None;
        self.maybe_resolved = None;
        self.completed_logical_actions.clear();
    }

    pub(crate) fn execute(&mut self, action: &ScheduledAction) -> Result<(), SessionBackendError> {
        let Some(resolved) = self.maybe_resolved.as_ref() else {
            return Err(protocol_failure());
        };
        let is_exact_action = resolved
            .actions()
            .iter()
            .any(|candidate| candidate == action);
        if !is_exact_action || self.completed_logical_actions.len() >= MAXIMUM_REPLAY_ACTIONS {
            self.destroy();
            return Err(harness(if is_exact_action {
                SessionBackendErrorCategory::ResourceLimit
            } else {
                SessionBackendErrorCategory::Protocol
            }));
        }
        let mut completed = self.completed_logical_actions.clone();
        completed.push(action.clone());
        match build_session(resolved, &completed) {
            Ok(session) => {
                self.completed_logical_actions = completed;
                self.maybe_session = Some(session);
                Ok(())
            }
            Err(error) => {
                self.destroy();
                Err(error)
            }
        }
    }

    pub(crate) fn capture(
        &mut self,
        checkpoint: &SessionCheckpointIdentity,
    ) -> Result<liquidfun_test_protocol::CanonicalCheckpoint, SessionBackendError> {
        let Some(resolved) = self.maybe_resolved.as_ref() else {
            return Err(protocol_failure());
        };
        let Some(session) = self.maybe_session.as_ref() else {
            return Err(harness(SessionBackendErrorCategory::Capture));
        };
        let captured = catch_unwind(AssertUnwindSafe(|| {
            capture_checkpoint(resolved, session, checkpoint)
        }));
        match captured {
            Ok(result) => result,
            Err(_payload) => {
                self.destroy();
                Err(harness(SessionBackendErrorCategory::Capture))
            }
        }
    }
}

fn build_session(
    resolved: &ResolvedScenario,
    completed: &[ScheduledAction],
) -> Result<NativeSession, SessionBackendError> {
    let replay = catch_unwind(AssertUnwindSafe(|| {
        let mut session = NativeSession::new()?;
        for action in resolved.actions().iter().filter(|action| {
            matches!(
                action.schedule(),
                liquidfun_test_protocol::ActionSchedule::Setup { .. }
            )
        }) {
            session.execute(resolved, action)?;
        }
        for action in completed {
            session.execute(resolved, action)?;
        }
        Ok(session)
    }));
    replay.unwrap_or_else(|_payload| Err(action_failure()))
}

pub(super) struct NativeSession {
    pub(super) world: World,
    pub(super) bodies: Vec<(ScenarioId, BodyId)>,
    pub(super) fixtures: Vec<(ScenarioId, FixtureId)>,
    pub(super) joints: Vec<(ScenarioId, JointId)>,
    pub(super) ropes: Vec<(ScenarioId, Rope)>,
    pub(super) systems: Vec<(ScenarioId, ParticleSystemId)>,
    pub(super) particles: Vec<(ScenarioId, ParticleSystemId, ParticleId)>,
    pub(super) groups: Vec<(ScenarioId, ParticleSystemId, ParticleGroupId)>,
    pub(super) simulation_time: f32,
}

impl NativeSession {
    fn new() -> Result<Self, SessionBackendError> {
        Ok(Self {
            world: World::new().map_err(|_error| action_failure())?,
            bodies: Vec::new(),
            fixtures: Vec::new(),
            joints: Vec::new(),
            ropes: Vec::new(),
            systems: Vec::new(),
            particles: Vec::new(),
            groups: Vec::new(),
            simulation_time: 0.0,
        })
    }

    fn execute(
        &mut self,
        resolved: &ResolvedScenario,
        scheduled: &ScheduledAction,
    ) -> Result<(), SessionBackendError> {
        match scheduled.action() {
            RigidWorldAction::Particle { action } => self.execute_particle(action),
            RigidWorldAction::ParticleGroup { operation } => self.execute_group(operation),
            action => self.execute_rigid(resolved, action),
        }
    }

    pub(super) fn step(
        &mut self,
        timestep: f32,
        velocity_iterations: u32,
        position_iterations: u32,
        particle_iterations: u32,
    ) -> Result<(), SessionBackendError> {
        let configuration =
            StepConfiguration::new(timestep, velocity_iterations, position_iterations)
                .and_then(|value| value.with_particle_iterations(particle_iterations))
                .map_err(|_error| action_failure())?;
        self.world
            .step(configuration, &mut NoDecisionHook, StepLimits::default())
            .map_err(|_error| action_failure())?;
        self.simulation_time += timestep;
        self.particles
            .retain(|(_, _, particle)| self.world.contains_particle(*particle));
        Ok(())
    }
}

pub(super) fn vec2(value: liquidfun_test_protocol::Vec2Bits) -> Vec2 {
    Vec2::new(value.x_bits.to_f32(), value.y_bits.to_f32())
}

pub(super) const fn harness(category: SessionBackendErrorCategory) -> SessionBackendError {
    SessionBackendError::harness(category)
}

pub(super) const fn action_failure() -> SessionBackendError {
    harness(SessionBackendErrorCategory::Action)
}

pub(super) const fn protocol_failure() -> SessionBackendError {
    harness(SessionBackendErrorCategory::Protocol)
}

pub(super) const fn resource_failure() -> SessionBackendError {
    harness(SessionBackendErrorCategory::ResourceLimit)
}

#[cfg(test)]
mod tests {
    use super::{SessionBackendErrorCategory, action_failure};

    #[test]
    fn panic_boundary_returns_one_bounded_action_failure() {
        // Arrange / Act
        let result = std::panic::catch_unwind(|| -> Result<(), crate::SessionBackendError> {
            panic!("private panic payload must not escape")
        })
        .unwrap_or_else(|_payload| Err(action_failure()));

        // Assert
        assert_eq!(
            result.expect_err("panic must be contained").category(),
            SessionBackendErrorCategory::Action
        );
    }
}
