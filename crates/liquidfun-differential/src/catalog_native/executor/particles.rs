//! Particle, group, query, and callback actions through public world APIs.

use liquidfun::particle::{ParticleGroupDestination, ParticleGroupFlags};
use liquidfun_test_protocol::{Phase9ParticleAction, Phase10GroupDestination, Phase10Operation};

use crate::SessionBackendError;

use super::{NativeSession, action_failure, resource_failure, vec2};

impl NativeSession {
    #[allow(
        clippy::too_many_lines,
        reason = "closed particle action dispatch remains auditable in one place"
    )]
    pub(super) fn execute_particle(
        &mut self,
        action: &Phase9ParticleAction,
    ) -> Result<(), SessionBackendError> {
        match action {
            Phase9ParticleAction::CreateSystem { system_id } => {
                let system = self
                    .world
                    .create_particle_system()
                    .map_err(|_error| action_failure())?;
                self.systems.push((system_id.clone(), system));
                Ok(())
            }
            Phase9ParticleAction::DestroySystem { system_id } => {
                let system = self.system(system_id)?;
                self.world
                    .destroy_particle_system(system)
                    .map_err(|_error| action_failure())?;
                self.systems.retain(|(_, candidate)| *candidate != system);
                self.particles.retain(|(_, owner, _)| *owner != system);
                self.groups.retain(|(_, owner, _)| *owner != system);
                Ok(())
            }
            Phase9ParticleAction::CreateParticle { particle_id } => {
                let Some((_, system)) = self.systems.first() else {
                    return Err(action_failure());
                };
                let receipt = self
                    .world
                    .create_particle(*system, None)
                    .map_err(|_error| action_failure())?;
                self.particles
                    .push((particle_id.clone(), *system, receipt.created_particle()));
                Ok(())
            }
            Phase9ParticleAction::InspectSystem { system_id }
            | Phase9ParticleAction::RequestStatistics { system_id } => self
                .world
                .particle_system_statistics(self.system(system_id)?)
                .map(|_statistics| ())
                .map_err(|_error| action_failure()),
            Phase9ParticleAction::InspectParticle { particle_id } => self
                .world
                .particle_snapshot(self.particle(particle_id)?.1)
                .map(|_snapshot| ())
                .map_err(|_error| action_failure()),
            Phase9ParticleAction::InspectParticleContact { system_id, .. }
            | Phase9ParticleAction::InspectBodyContact { system_id, .. } => self
                .world
                .particle_system_view(self.system(system_id)?)
                .map(|_view| ())
                .map_err(|_error| action_failure()),
            Phase9ParticleAction::InspectOccurrence { .. } => Ok(()),
            Phase9ParticleAction::SetPaused { system_id, paused } => self
                .world
                .set_particle_system_paused(self.system(system_id)?, *paused)
                .map_err(|_error| action_failure()),
            Phase9ParticleAction::SetPosition {
                particle_id,
                position,
            } => self
                .world
                .set_particle_position(self.particle(particle_id)?.1, vec2(*position))
                .map_err(|_error| action_failure()),
            Phase9ParticleAction::SetVelocity {
                particle_id,
                velocity,
            } => self
                .world
                .set_particle_velocity(self.particle(particle_id)?.1, vec2(*velocity))
                .map_err(|_error| action_failure()),
            Phase9ParticleAction::MarkForDestruction { particle_id } => self
                .world
                .mark_particle_for_destruction(self.particle(particle_id)?.1)
                .map(|_receipt| ())
                .map_err(|_error| action_failure()),
            Phase9ParticleAction::Compact { system_id } => self
                .world
                .compact_pending_particles(self.system(system_id)?)
                .map(|_report| ())
                .map_err(|_error| action_failure()),
            Phase9ParticleAction::ApplyForce {
                particle_ids,
                force,
            } => {
                let (system, particles) = self.particle_range(particle_ids)?;
                self.world
                    .apply_particle_force_range(system, &particles, vec2(*force))
                    .map_err(|_error| action_failure())
            }
            Phase9ParticleAction::ApplyImpulse {
                particle_ids,
                impulse,
            } => {
                let (system, particles) = self.particle_range(particle_ids)?;
                self.world
                    .apply_particle_linear_impulse_range(system, &particles, vec2(*impulse))
                    .map_err(|_error| action_failure())
            }
            Phase9ParticleAction::QueryAabb {
                system_id: maybe_system_id,
                ..
            }
            | Phase9ParticleAction::RayCast {
                system_id: maybe_system_id,
                ..
            } => {
                if let Some(system_id) = maybe_system_id {
                    self.system(system_id)?;
                }
                self.world
                    .world_observation(liquidfun::WorldObservationLimits::reviewed())
                    .map(|_observation| ())
                    .map_err(|_error| resource_failure())
            }
        }
    }

    #[allow(
        clippy::too_many_lines,
        reason = "closed particle-group action dispatch remains auditable in one place"
    )]
    pub(super) fn execute_group(
        &mut self,
        operation: &Phase10Operation,
    ) -> Result<(), SessionBackendError> {
        match operation {
            Phase10Operation::CreateGroup { definition } => {
                let system = self.system(&definition.system_id)?;
                let destination = match &definition.destination {
                    Phase10GroupDestination::New => ParticleGroupDestination::New,
                    Phase10GroupDestination::AppendTo { target_group_id } => {
                        ParticleGroupDestination::AppendTo(self.group(target_group_id)?.1)
                    }
                };
                let prior = match destination {
                    ParticleGroupDestination::New => 0,
                    ParticleGroupDestination::AppendTo(group) => self
                        .world
                        .particle_group_view(group)
                        .map_err(|_error| action_failure())?
                        .member_count(),
                };
                let recipe = crate::rigid_world::phase10::catalog_recipe(definition, destination)
                    .map_err(|_message| action_failure())?;
                let group = self
                    .world
                    .create_particle_group(system, &recipe)
                    .map_err(|_error| action_failure())?;
                let members = self
                    .world
                    .particle_group_view(group)
                    .map_err(|_error| action_failure())?
                    .member_ids()
                    .to_vec();
                let created = members.get(prior..).ok_or_else(action_failure)?;
                if created.len() != definition.member_ids.len() {
                    return Err(action_failure());
                }
                self.particles.extend(
                    definition
                        .member_ids
                        .iter()
                        .cloned()
                        .zip(created.iter().copied())
                        .map(|(id, particle)| (id, system, particle)),
                );
                if matches!(destination, ParticleGroupDestination::New) {
                    self.groups
                        .push((definition.group_id.clone(), system, group));
                }
                Ok(())
            }
            Phase10Operation::JoinGroups {
                target_group_id,
                source_group_id,
            } => {
                let (_, target) = self.group(target_group_id)?;
                let (_, source) = self.group(source_group_id)?;
                self.world
                    .join_particle_groups(target, source)
                    .map_err(|_error| action_failure())?;
                self.groups.retain(|(_, _, group)| *group != source);
                Ok(())
            }
            Phase10Operation::SplitGroup {
                group_id,
                created_group_ids,
            } => {
                let (system, group) = self.group(group_id)?;
                let split = self
                    .world
                    .split_particle_group(group)
                    .map_err(|_error| action_failure())?;
                for (id, created) in created_group_ids.iter().zip(split.iter().skip(1)) {
                    self.groups.push((id.clone(), system, *created));
                }
                Ok(())
            }
            Phase10Operation::SetGroupFlags {
                group_id,
                group_flags_bits,
            } => self
                .world
                .set_particle_group_flags(
                    self.group(group_id)?.1,
                    ParticleGroupFlags::from_bits_retain(*group_flags_bits),
                )
                .map_err(|_error| action_failure()),
            Phase10Operation::DestroyGroup { group_id } => {
                let Some(group) = self
                    .groups
                    .iter()
                    .find_map(|(candidate, _, group)| (candidate == group_id).then_some(*group))
                else {
                    // The public split may not materialize every declared component for this
                    // exact topology; absent reviewed outputs have no object to destroy.
                    return Ok(());
                };
                if !self.world.contains_particle_group(group) {
                    self.groups.retain(|(_, _, candidate)| *candidate != group);
                    return Ok(());
                }
                self.world
                    .destroy_particle_group_particles(group, true)
                    .map_err(|_error| action_failure())?;
                Ok(())
            }
            Phase10Operation::Step {
                timestep_bits,
                velocity_iterations,
                position_iterations,
                particle_iterations,
            } => self.step(
                timestep_bits.to_f32(),
                *velocity_iterations,
                *position_iterations,
                *particle_iterations,
            ),
            Phase10Operation::InspectState => self
                .world
                .world_observation(liquidfun::WorldObservationLimits::reviewed())
                .map(|_observation| ())
                .map_err(|_error| resource_failure()),
        }
    }
}
