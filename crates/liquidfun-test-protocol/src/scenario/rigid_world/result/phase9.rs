use std::collections::{HashMap, HashSet};

use super::{
    Phase9ParticleObservation, RigidCheckpointLiveIdentities, RigidWorldDecodeError,
    RigidWorldErrorKind, RigidWorldObservation, validation,
};
use crate::{
    Phase9BodyContactObservation, Phase9Occurrence, Phase9OccurrenceKind, Phase9ParticleAction,
    Phase9ParticleContactObservation, Phase9ParticleSystemDeclaration, Phase9QueryControl,
    Phase9RayControl, RigidWorldTimeline, ScenarioId,
};

pub(super) struct Phase9ResultState<'a> {
    timeline: &'a RigidWorldTimeline,
    particle_owners: HashMap<ScenarioId, ScenarioId>,
    live_systems: HashSet<ScenarioId>,
    live_particles: HashSet<ScenarioId>,
    pending_particles: HashSet<ScenarioId>,
    maybe_expected_occurrence: Option<Phase9Occurrence>,
    next_occurrence_ordinal: u32,
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
            maybe_expected_occurrence: None,
            next_occurrence_ordinal: 0,
        }
    }

    pub(super) fn apply(&mut self, action: &Phase9ParticleAction) {
        self.maybe_expected_occurrence = None;
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
                self.set_expected_occurrence(
                    Phase9OccurrenceKind::SystemDestroyed,
                    system_id.clone(),
                    None,
                );
            }
            Phase9ParticleAction::CreateParticle { particle_id } => {
                if let Some(system_id) = self.particle_owners.get(particle_id).cloned()
                    && self.visible_particle_ids_for_system(&system_id).len()
                        >= self.system_capacity(&system_id)
                    && let Some(victim) = self.capacity_victim(&system_id)
                {
                    if self
                        .particle_declaration(&victim)
                        .is_some_and(|declaration| declaration.flags_bits & (1 << 9) != 0)
                    {
                        self.set_expected_occurrence(
                            Phase9OccurrenceKind::ParticleDestroyed,
                            system_id,
                            Some(victim.clone()),
                        );
                    }
                    self.live_particles.remove(&victim);
                    self.pending_particles.remove(&victim);
                }
                self.live_particles.insert(particle_id.clone());
            }
            Phase9ParticleAction::MarkForDestruction { particle_id } => {
                self.live_particles.remove(particle_id);
                self.pending_particles.insert(particle_id.clone());
            }
            Phase9ParticleAction::Compact { system_id } => {
                let maybe_requested = self
                    .timeline
                    .particles()
                    .iter()
                    .find(|declaration| {
                        &declaration.system_id == system_id
                            && self.pending_particles.contains(&declaration.particle_id)
                            && declaration.flags_bits & (1 << 9) != 0
                    })
                    .map(|declaration| declaration.particle_id.clone());
                if let Some(particle_id) = maybe_requested {
                    self.set_expected_occurrence(
                        Phase9OccurrenceKind::ParticleDestroyed,
                        system_id.clone(),
                        Some(particle_id),
                    );
                }
                let owners = &self.particle_owners;
                self.pending_particles
                    .retain(|particle_id| owners.get(particle_id) != Some(system_id));
            }
            Phase9ParticleAction::InspectSystem { .. }
            | Phase9ParticleAction::InspectParticle { .. }
            | Phase9ParticleAction::InspectParticleContact { .. }
            | Phase9ParticleAction::InspectBodyContact { .. }
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
                Phase9ParticleAction::InspectSystem { system_id },
                Phase9ParticleObservation::System {
                    system_id: observed_system_id,
                    paused,
                    particle_ids,
                },
            ) => {
                observed_system_id == system_id
                    && self
                        .system_declaration(system_id)
                        .is_some_and(|declaration| declaration.paused == *paused)
                    && particle_ids.as_ref()
                        == self.visible_particle_ids_for_system(system_id).as_slice()
            }
            (
                Phase9ParticleAction::InspectParticle { particle_id },
                Phase9ParticleObservation::Particle { snapshot },
            ) => {
                snapshot.particle_id == *particle_id
                    && self.owner(particle_id) == Some(&snapshot.system_id)
                    && self.live_particles.contains(particle_id)
                    && !snapshot.pending_destruction
                    && snapshot.position.x_bits.to_f32().is_finite()
                    && snapshot.position.y_bits.to_f32().is_finite()
                    && snapshot.velocity.x_bits.to_f32().is_finite()
                    && snapshot.velocity.y_bits.to_f32().is_finite()
                    && snapshot.weight_bits.to_f32().is_finite()
                    && snapshot.force.x_bits.to_f32().is_finite()
                    && snapshot.force.y_bits.to_f32().is_finite()
            }
            (
                Phase9ParticleAction::InspectParticleContact { system_id, .. },
                Phase9ParticleObservation::ParticleContact { contact },
            ) => self.particle_contact_matches(system_id, contact),
            (
                Phase9ParticleAction::InspectBodyContact { system_id, .. },
                Phase9ParticleObservation::BodyContact { contact },
            ) => self.body_contact_matches(system_id, live_rigid, contact),
            (
                Phase9ParticleAction::RequestStatistics { system_id },
                Phase9ParticleObservation::Statistics { statistics },
            ) => self.statistics_match(system_id, statistics),
            (
                Phase9ParticleAction::QueryAabb {
                    system_id, control, ..
                },
                Phase9ParticleObservation::Query {
                    terminated,
                    particle_ids,
                },
            ) => self.query_matches(system_id.as_ref(), *control, *terminated, particle_ids),
            (
                Phase9ParticleAction::RayCast {
                    system_id, control, ..
                },
                Phase9ParticleObservation::RayCast {
                    terminated,
                    particle_ids,
                    fractions_bits,
                },
            ) => self.ray_cast_matches(
                system_id.as_ref(),
                *control,
                *terminated,
                particle_ids,
                fractions_bits,
            ),
            (
                Phase9ParticleAction::DestroySystem { .. }
                | Phase9ParticleAction::CreateParticle { .. }
                | Phase9ParticleAction::Compact { .. },
                Phase9ParticleObservation::Lifecycle { occurrence },
            ) if self.maybe_expected_occurrence.as_ref() == Some(occurrence) => true,
            (
                Phase9ParticleAction::CreateSystem { .. }
                | Phase9ParticleAction::CreateParticle { .. }
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
            ) if self.maybe_expected_occurrence.is_none() => {
                self.mixed_state_matches(live_rigid, body_ids, particle_ids)
            }
            _ => false,
        };
        if matches { Ok(()) } else { Err(mismatch()) }
    }

    fn ray_cast_matches(
        &self,
        maybe_system_id: Option<&ScenarioId>,
        control: Phase9RayControl,
        terminated: bool,
        particle_ids: &[ScenarioId],
        fractions_bits: &[crate::FloatBits],
    ) -> bool {
        control.termination_matches(terminated, particle_ids)
            && particle_ids.len() == fractions_bits.len()
            && self.selection_matches(maybe_system_id, particle_ids)
            && fractions_bits.iter().all(|bits| {
                let fraction = bits.to_f32();
                fraction.is_finite() && (0.0..=1.0).contains(&fraction)
            })
    }

    fn query_matches(
        &self,
        maybe_system_id: Option<&ScenarioId>,
        control: Phase9QueryControl,
        terminated: bool,
        particle_ids: &[ScenarioId],
    ) -> bool {
        control.termination_matches(terminated, particle_ids)
            && self.selection_matches(maybe_system_id, particle_ids)
    }

    fn mixed_state_matches(
        &self,
        live_rigid: &RigidCheckpointLiveIdentities<'_>,
        body_ids: &[ScenarioId],
        particle_ids: &[ScenarioId],
    ) -> bool {
        body_ids
            == live_rigid
                .body_ids
                .iter()
                .map(|id| (*id).clone())
                .collect::<Vec<_>>()
                .as_slice()
            && particle_ids == self.visible_particle_ids().as_slice()
    }

    fn set_expected_occurrence(
        &mut self,
        kind: Phase9OccurrenceKind,
        system_id: ScenarioId,
        maybe_particle_id: Option<ScenarioId>,
    ) {
        let ordinal = self.next_occurrence_ordinal;
        self.next_occurrence_ordinal = self
            .next_occurrence_ordinal
            .checked_add(1)
            .expect("bounded Phase 9 action count cannot overflow u32");
        self.maybe_expected_occurrence = Some(Phase9Occurrence {
            ordinal,
            kind,
            system_id,
            maybe_particle_id,
            maybe_other_particle_id: None,
            maybe_fixture_id: None,
        });
    }

    fn particle_contact_matches(
        &self,
        system_id: &ScenarioId,
        contact: &Phase9ParticleContactObservation,
    ) -> bool {
        contact.system_id == *system_id
            && self.owner(&contact.particle_a_id) == Some(system_id)
            && self.owner(&contact.particle_b_id) == Some(system_id)
            && self.live_particles.contains(&contact.particle_a_id)
            && self.live_particles.contains(&contact.particle_b_id)
            && contact.weight_bits.to_f32().is_finite()
            && contact.normal.x_bits.to_f32().is_finite()
            && contact.normal.y_bits.to_f32().is_finite()
    }

    fn body_contact_matches(
        &self,
        system_id: &ScenarioId,
        live_rigid: &RigidCheckpointLiveIdentities<'_>,
        contact: &Phase9BodyContactObservation,
    ) -> bool {
        contact.system_id == *system_id
            && self.owner(&contact.particle_id) == Some(system_id)
            && self.live_particles.contains(&contact.particle_id)
            && live_rigid.body_ids.contains(&&contact.body_id)
            && live_rigid.fixture_ids.contains(&&contact.fixture_id)
            && contact.weight_bits.to_f32().is_finite()
            && contact.normal.x_bits.to_f32().is_finite()
            && contact.normal.y_bits.to_f32().is_finite()
            && contact.mass_bits.to_f32().is_finite()
    }

    fn particle_declaration(
        &self,
        particle_id: &ScenarioId,
    ) -> Option<&crate::Phase9ParticleDeclaration> {
        self.timeline
            .particles()
            .iter()
            .find(|declaration| &declaration.particle_id == particle_id)
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

    fn visible_particle_ids_for_system(&self, system_id: &ScenarioId) -> Vec<ScenarioId> {
        self.visible_particle_ids()
            .into_iter()
            .filter(|particle_id| self.owner(particle_id) == Some(system_id))
            .collect()
    }

    fn system_capacity(&self, system_id: &ScenarioId) -> usize {
        self.system_declaration(system_id)
            .map_or(usize::MAX, |declaration| {
                declaration.maximum_count.map_or_else(
                    || declared_capacity(declaration),
                    |maximum| maximum.min(declared_capacity(declaration)),
                )
            })
    }

    fn capacity_victim(&self, system_id: &ScenarioId) -> Option<ScenarioId> {
        self.timeline
            .particles()
            .iter()
            .filter(|declaration| {
                &declaration.system_id == system_id
                    && (self.live_particles.contains(&declaration.particle_id)
                        || self.pending_particles.contains(&declaration.particle_id))
                    && declaration.lifetime_bits.to_f32() > 0.0
            })
            .min_by_key(|declaration| declaration.lifetime_bits.bits())
            .map(|declaration| declaration.particle_id.clone())
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

trait Phase9TerminationControl {
    fn termination_matches(&self, terminated: bool, particle_ids: &[ScenarioId]) -> bool;
}

impl Phase9TerminationControl for Phase9QueryControl {
    fn termination_matches(&self, terminated: bool, particle_ids: &[ScenarioId]) -> bool {
        match self {
            Self::Continue => !terminated,
            Self::Terminate => terminated != particle_ids.is_empty(),
        }
    }
}

impl Phase9TerminationControl for Phase9RayControl {
    fn termination_matches(&self, terminated: bool, particle_ids: &[ScenarioId]) -> bool {
        match self {
            Self::Ignore | Self::Continue | Self::Clip => !terminated,
            Self::Terminate => terminated != particle_ids.is_empty(),
        }
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
