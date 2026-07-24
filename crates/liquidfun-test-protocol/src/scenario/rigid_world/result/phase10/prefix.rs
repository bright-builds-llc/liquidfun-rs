use std::collections::{HashMap, HashSet};

use super::{
    Phase10Event, Phase10EventKind, Phase10GroupSnapshot, Phase10ParticleSnapshot,
    Phase10StateObservation,
};
use crate::ScenarioId;

use super::super::super::{Phase10GroupDestination, Phase10Operation, Phase10ValidationKind};

pub(crate) fn validate_event_shapes(events: &[Phase10Event]) -> Result<(), Phase10ValidationKind> {
    for event in events {
        let shape_is_valid = match event.kind {
            Phase10EventKind::GroupCreated
            | Phase10EventKind::GroupsJoined
            | Phase10EventKind::GroupSplit
            | Phase10EventKind::GroupDestroyed => {
                event.maybe_group_id.is_some()
                    && event.maybe_particle_id.is_none()
                    && event.maybe_other_particle_id.is_none()
                    && event.maybe_body_id.is_none()
            }
            Phase10EventKind::ParticleDestroyed | Phase10EventKind::BodyContactEnd => {
                event.maybe_group_id.is_none()
                    && event.maybe_particle_id.is_some()
                    && event.maybe_other_particle_id.is_none()
                    && event.maybe_body_id.is_none()
            }
            Phase10EventKind::ParticleContactBegin | Phase10EventKind::ParticleContactEnd => {
                event.maybe_group_id.is_none()
                    && event.maybe_particle_id.is_some()
                    && event.maybe_other_particle_id.is_some()
                    && event.maybe_body_id.is_none()
            }
            Phase10EventKind::BodyContactBegin => {
                event.maybe_group_id.is_none()
                    && event.maybe_particle_id.is_some()
                    && event.maybe_other_particle_id.is_none()
                    && event.maybe_body_id.is_some()
            }
        };
        if !shape_is_valid {
            return Err(Phase10ValidationKind::InvalidTopology);
        }
    }
    Ok(())
}

#[derive(Default)]
pub(crate) struct Phase10ResultState {
    maybe_provenance: Option<super::super::super::Phase10Provenance>,
    historical_group_owners: HashMap<ScenarioId, ScenarioId>,
    live_group_owners: HashMap<ScenarioId, ScenarioId>,
    particle_owners: HashMap<ScenarioId, ScenarioId>,
    possible_particle_groups: HashMap<ScenarioId, HashSet<ScenarioId>>,
}

impl Phase10ResultState {
    pub(crate) fn apply(&mut self, operation: &Phase10Operation) {
        match operation {
            Phase10Operation::CreateGroup { definition } => {
                self.maybe_provenance
                    .get_or_insert_with(|| definition.provenance.clone());
                let group_id = match &definition.destination {
                    Phase10GroupDestination::New => {
                        self.historical_group_owners
                            .insert(definition.group_id.clone(), definition.system_id.clone());
                        self.live_group_owners
                            .insert(definition.group_id.clone(), definition.system_id.clone());
                        &definition.group_id
                    }
                    Phase10GroupDestination::AppendTo { target_group_id } => target_group_id,
                };
                for particle_id in &definition.member_ids {
                    self.particle_owners
                        .insert(particle_id.clone(), definition.system_id.clone());
                    self.possible_particle_groups
                        .insert(particle_id.clone(), HashSet::from([group_id.clone()]));
                }
            }
            Phase10Operation::JoinGroups {
                target_group_id,
                source_group_id,
            } => {
                self.live_group_owners.remove(source_group_id);
                for possible_groups in self.possible_particle_groups.values_mut() {
                    if possible_groups.remove(source_group_id) {
                        possible_groups.insert(target_group_id.clone());
                    }
                }
            }
            Phase10Operation::SplitGroup {
                group_id,
                created_group_ids,
            } => {
                let Some(owner) = self.live_group_owners.get(group_id).cloned() else {
                    return;
                };
                for created_group_id in created_group_ids {
                    self.historical_group_owners
                        .insert(created_group_id.clone(), owner.clone());
                    self.live_group_owners
                        .insert(created_group_id.clone(), owner.clone());
                }
                for possible_groups in self.possible_particle_groups.values_mut() {
                    if possible_groups.contains(group_id) {
                        possible_groups.extend(created_group_ids.iter().cloned());
                    }
                }
            }
            Phase10Operation::DestroyGroup { group_id } => {
                self.live_group_owners.remove(group_id);
                for possible_groups in self.possible_particle_groups.values_mut() {
                    possible_groups.remove(group_id);
                }
            }
            Phase10Operation::SetGroupFlags { .. }
            | Phase10Operation::Step { .. }
            | Phase10Operation::InspectState => {}
        }
    }

    pub(crate) fn validate(
        &self,
        state: &Phase10StateObservation,
        created_body_ids: &HashSet<ScenarioId>,
    ) -> Result<(), Phase10ValidationKind> {
        if self.maybe_provenance.as_ref() != Some(&state.provenance) {
            return Err(Phase10ValidationKind::InvalidProvenance);
        }
        for group in &state.groups {
            self.validate_group(group)?;
        }
        for particle in &state.particles {
            self.validate_particle(particle)?;
        }
        for event in &state.events {
            self.validate_event(event, created_body_ids)?;
        }
        Ok(())
    }

    fn validate_group(&self, group: &Phase10GroupSnapshot) -> Result<(), Phase10ValidationKind> {
        if self.live_group_owners.get(&group.group_id) != Some(&group.system_id) {
            return Err(Phase10ValidationKind::InvalidOwnership);
        }
        Ok(())
    }

    fn validate_particle(
        &self,
        particle: &Phase10ParticleSnapshot,
    ) -> Result<(), Phase10ValidationKind> {
        if self.particle_owners.get(&particle.particle_id) != Some(&particle.system_id)
            || self.live_group_owners.get(&particle.group_id) != Some(&particle.system_id)
            || self
                .possible_particle_groups
                .get(&particle.particle_id)
                .is_none_or(|groups| !groups.contains(&particle.group_id))
        {
            return Err(Phase10ValidationKind::InvalidOwnership);
        }
        Ok(())
    }

    fn validate_event(
        &self,
        event: &Phase10Event,
        created_body_ids: &HashSet<ScenarioId>,
    ) -> Result<(), Phase10ValidationKind> {
        if let Some(group_id) = &event.maybe_group_id
            && self.historical_group_owners.get(group_id) != Some(&event.system_id)
        {
            return Err(Phase10ValidationKind::InvalidOwnership);
        }
        for particle_id in [
            event.maybe_particle_id.as_ref(),
            event.maybe_other_particle_id.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            if self.particle_owners.get(particle_id) != Some(&event.system_id) {
                return Err(Phase10ValidationKind::InvalidOwnership);
            }
        }
        if matches!(event.kind, Phase10EventKind::BodyContactBegin)
            && event
                .maybe_body_id
                .as_ref()
                .is_none_or(|body_id| !created_body_ids.contains(body_id))
        {
            return Err(Phase10ValidationKind::UnknownSemanticId);
        }
        Ok(())
    }
}
