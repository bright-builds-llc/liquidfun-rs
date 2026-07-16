use std::collections::{HashMap, HashSet};

use super::{
    Phase9ParticleObservation, RigidCheckpointLiveIdentities, RigidWorldDecodeError,
    RigidWorldErrorKind, RigidWorldObservation, validation,
};
use crate::{
    Phase9ParticleAction, Phase9ParticleSystemDeclaration, RigidWorldTimeline, ScenarioId,
};

pub(super) struct Phase9ResultState<'a> {
    timeline: &'a RigidWorldTimeline,
    particle_owners: HashMap<ScenarioId, ScenarioId>,
    live_systems: HashSet<ScenarioId>,
    live_particles: HashSet<ScenarioId>,
    pending_particles: HashSet<ScenarioId>,
}

impl<'a> Phase9ResultState<'a> {
    pub(super) fn new(timeline: &'a RigidWorldTimeline) -> Self {
        Self {
            timeline,
            particle_owners: timeline
                .particles()
                .iter()
                .map(|particle| (particle.particle_id.clone(), particle.system_id.clone()))
                .collect(),
            live_systems: HashSet::new(),
            live_particles: HashSet::new(),
            pending_particles: HashSet::new(),
        }
    }

    pub(super) fn apply(&mut self, action: &Phase9ParticleAction) {
        match action {
            Phase9ParticleAction::CreateSystem { system_id } => {
                self.live_systems.insert(system_id.clone());
            }
            Phase9ParticleAction::DestroySystem { system_id } => {
                self.live_systems.remove(system_id);
                let owners = &self.particle_owners;
                self.live_particles
                    .retain(|particle_id| owners.get(particle_id) != Some(system_id));
                self.pending_particles
                    .retain(|particle_id| owners.get(particle_id) != Some(system_id));
            }
            Phase9ParticleAction::CreateParticle { particle_id } => {
                self.live_particles.insert(particle_id.clone());
            }
            Phase9ParticleAction::MarkForDestruction { particle_id } => {
                self.live_particles.remove(particle_id);
                self.pending_particles.insert(particle_id.clone());
            }
            Phase9ParticleAction::Compact { system_id } => {
                let owners = &self.particle_owners;
                self.pending_particles
                    .retain(|particle_id| owners.get(particle_id) != Some(system_id));
            }
            Phase9ParticleAction::InspectSystem { .. }
            | Phase9ParticleAction::InspectParticle { .. }
            | Phase9ParticleAction::SetPaused { .. }
            | Phase9ParticleAction::SetPosition { .. }
            | Phase9ParticleAction::SetVelocity { .. }
            | Phase9ParticleAction::ApplyForce { .. }
            | Phase9ParticleAction::ApplyImpulse { .. }
            | Phase9ParticleAction::RequestStatistics { .. }
            | Phase9ParticleAction::QueryAabb { .. }
            | Phase9ParticleAction::RayCast { .. } => {}
        }
    }

    pub(super) fn validate(
        &self,
        action: &Phase9ParticleAction,
        live_rigid: &RigidCheckpointLiveIdentities<'_>,
        actual: &RigidWorldObservation,
    ) -> Result<(), RigidWorldDecodeError> {
        let RigidWorldObservation::Particle { observation } = actual else {
            return Err(mismatch());
        };
        let matches = match (action, observation) {
            (
                Phase9ParticleAction::RequestStatistics { system_id },
                Phase9ParticleObservation::Statistics { statistics },
            ) => self.statistics_match(system_id, statistics),
            (
                Phase9ParticleAction::QueryAabb { system_id, .. },
                Phase9ParticleObservation::Query {
                    terminated,
                    particle_ids,
                },
            ) => !terminated && self.selection_matches(system_id.as_ref(), particle_ids),
            (
                Phase9ParticleAction::RayCast { system_id, .. },
                Phase9ParticleObservation::RayCast {
                    terminated,
                    particle_ids,
                    fractions_bits,
                },
            ) => {
                !terminated
                    && particle_ids.len() == fractions_bits.len()
                    && self.selection_matches(system_id.as_ref(), particle_ids)
                    && fractions_bits.iter().all(|bits| {
                        let fraction = bits.to_f32();
                        fraction.is_finite() && (0.0..=1.0).contains(&fraction)
                    })
            }
            (
                Phase9ParticleAction::CreateSystem { .. }
                | Phase9ParticleAction::DestroySystem { .. }
                | Phase9ParticleAction::CreateParticle { .. }
                | Phase9ParticleAction::InspectSystem { .. }
                | Phase9ParticleAction::InspectParticle { .. }
                | Phase9ParticleAction::SetPaused { .. }
                | Phase9ParticleAction::SetPosition { .. }
                | Phase9ParticleAction::SetVelocity { .. }
                | Phase9ParticleAction::MarkForDestruction { .. }
                | Phase9ParticleAction::Compact { .. }
                | Phase9ParticleAction::ApplyForce { .. }
                | Phase9ParticleAction::ApplyImpulse { .. },
                Phase9ParticleObservation::MixedState {
                    body_ids,
                    particle_ids,
                },
            ) => {
                body_ids.as_ref()
                    == live_rigid
                        .body_ids
                        .iter()
                        .map(|id| (*id).clone())
                        .collect::<Vec<_>>()
                        .as_slice()
                    && particle_ids.as_ref() == self.visible_particle_ids().as_slice()
            }
            _ => false,
        };
        if matches { Ok(()) } else { Err(mismatch()) }
    }

    fn owner(&self, particle_id: &ScenarioId) -> Option<&ScenarioId> {
        self.particle_owners.get(particle_id)
    }

    fn visible_particle_ids(&self) -> Vec<ScenarioId> {
        self.timeline
            .particles()
            .iter()
            .map(|particle| &particle.particle_id)
            .filter(|particle_id| {
                self.live_particles.contains(*particle_id)
                    || self.pending_particles.contains(*particle_id)
            })
            .cloned()
            .collect()
    }

    fn selection_matches(
        &self,
        maybe_system_id: Option<&ScenarioId>,
        particle_ids: &[ScenarioId],
    ) -> bool {
        let mut unique = HashSet::new();
        particle_ids.iter().all(|particle_id| {
            unique.insert(particle_id)
                && (self.live_particles.contains(particle_id)
                    || self.pending_particles.contains(particle_id))
                && maybe_system_id
                    .is_none_or(|system_id| self.owner(particle_id) == Some(system_id))
        })
    }

    fn statistics_match(
        &self,
        system_id: &ScenarioId,
        statistics: &crate::Phase9StatisticsObservation,
    ) -> bool {
        let Some(declaration) = self.system_declaration(system_id) else {
            return false;
        };
        let visible_count = self
            .visible_particle_ids()
            .iter()
            .filter(|particle_id| self.owner(particle_id) == Some(system_id))
            .count();
        let pending_count = self
            .pending_particles
            .iter()
            .filter(|particle_id| self.owner(particle_id) == Some(system_id))
            .count();
        let stuck_match =
            self.selection_matches(Some(system_id), statistics.stuck_particle_ids.as_ref());
        statistics.maybe_system_id.as_ref() == Some(system_id)
            && usize::try_from(statistics.system_count).ok() == Some(self.live_systems.len())
            && usize::try_from(statistics.particle_count).ok() == Some(visible_count)
            && usize::try_from(statistics.pending_particle_count).ok() == Some(pending_count)
            && stuck_match
            && statistics.collision_energy_bits.to_f32().is_finite()
            && usize::try_from(statistics.declared_capacity).ok()
                == Some(declared_capacity(declaration))
            && statistics.effective_capacity == statistics.declared_capacity
    }

    fn system_declaration(
        &self,
        system_id: &ScenarioId,
    ) -> Option<&Phase9ParticleSystemDeclaration> {
        self.timeline
            .particle_systems()
            .iter()
            .find(|declaration| &declaration.system_id == system_id)
    }
}

fn declared_capacity(declaration: &Phase9ParticleSystemDeclaration) -> usize {
    match declaration.buffer_mode {
        crate::Phase9ParticleBufferMode::Fixed { capacity } => capacity,
        crate::Phase9ParticleBufferMode::Growable { .. } => {
            declaration.maximum_count.unwrap_or(i32::MAX as usize)
        }
    }
}

fn mismatch() -> RigidWorldDecodeError {
    validation(RigidWorldErrorKind::ResultObservationMismatch)
}
