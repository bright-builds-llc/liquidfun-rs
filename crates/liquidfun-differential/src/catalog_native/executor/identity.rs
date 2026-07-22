//! Typed semantic-ID resolution without exposing storage coordinates.

use liquidfun::rope::Rope;
use liquidfun::{BodyId, FixtureId, JointId, ParticleGroupId, ParticleId, ParticleSystemId};
use liquidfun_test_protocol::ScenarioId;

use crate::SessionBackendError;

use super::{NativeSession, action_failure};

impl NativeSession {
    pub(super) fn body(&self, id: &ScenarioId) -> Result<BodyId, SessionBackendError> {
        self.bodies
            .iter()
            .find_map(|(candidate, body)| (candidate == id).then_some(*body))
            .ok_or_else(action_failure)
    }

    pub(super) fn fixture(&self, id: &ScenarioId) -> Result<FixtureId, SessionBackendError> {
        self.fixtures
            .iter()
            .find_map(|(candidate, fixture)| (candidate == id).then_some(*fixture))
            .ok_or_else(action_failure)
    }

    pub(super) fn joint(&self, id: &ScenarioId) -> Result<JointId, SessionBackendError> {
        self.joints
            .iter()
            .find_map(|(candidate, joint)| (candidate == id).then_some(*joint))
            .ok_or_else(action_failure)
    }

    pub(super) fn rope_mut(&mut self, id: &ScenarioId) -> Result<&mut Rope, SessionBackendError> {
        self.ropes
            .iter_mut()
            .find_map(|(candidate, rope)| (candidate == id).then_some(rope))
            .ok_or_else(action_failure)
    }

    pub(super) fn system(&self, id: &ScenarioId) -> Result<ParticleSystemId, SessionBackendError> {
        self.systems
            .iter()
            .find_map(|(candidate, system)| (candidate == id).then_some(*system))
            .ok_or_else(action_failure)
    }

    pub(super) fn particle(
        &self,
        id: &ScenarioId,
    ) -> Result<(ParticleSystemId, ParticleId), SessionBackendError> {
        self.particles
            .iter()
            .find_map(|(candidate, system, particle)| {
                (candidate == id).then_some((*system, *particle))
            })
            .ok_or_else(action_failure)
    }

    pub(super) fn particle_range(
        &self,
        ids: &[ScenarioId],
    ) -> Result<(ParticleSystemId, Vec<ParticleId>), SessionBackendError> {
        let particles = ids
            .iter()
            .map(|id| self.particle(id))
            .collect::<Result<Vec<_>, _>>()?;
        let Some((system, _)) = particles.first() else {
            return Err(action_failure());
        };
        if particles.iter().any(|(candidate, _)| candidate != system) {
            return Err(action_failure());
        }
        Ok((*system, particles.into_iter().map(|(_, id)| id).collect()))
    }

    pub(super) fn group(
        &self,
        id: &ScenarioId,
    ) -> Result<(ParticleSystemId, ParticleGroupId), SessionBackendError> {
        self.groups
            .iter()
            .find_map(|(candidate, system, group)| (candidate == id).then_some((*system, *group)))
            .ok_or_else(action_failure)
    }
}
