use crate::particle::storage::ParticleStorageError;
use crate::{
    CreateObjectError, DestroyedId, DestructionCause, DestructionRecord, DestructionReport,
    HandleError, LifecycleEvent, MutationReport, ObjectSnapshot, ParticleGroupId, ParticleSystemId,
};

use super::{ParticleSystem, World, storage_object_creation_error};

impl World {
    /// Marks every current member of a particle group for deferred destruction.
    ///
    /// Members remain inspectable through the group view until the next
    /// positive-duration lifecycle step or an explicit pending-particle
    /// compaction. When `call_listener` is true, each member requests one
    /// destruction-listener occurrence in ascending source order.
    ///
    /// # Errors
    ///
    /// Returns a typed no-effect error when the world is locked or poisoned,
    /// or when `group` is foreign, stale, or internally inconsistent.
    pub fn destroy_particle_group_particles(
        &mut self,
        group: ParticleGroupId,
        call_listener: bool,
    ) -> Result<(), CreateObjectError> {
        self.ensure_not_poisoned_for_handle()?;
        if self.step_state.is_locked() {
            return Err(CreateObjectError::WorldLocked);
        }
        let shell = self.particle_groups.get(group)?;
        let system = shell.system;
        let mut candidate = self.particle_systems.get(system)?.clone();
        let particle_ids = candidate
            .storage
            .group_view(group, 0.0)
            .map_err(storage_object_creation_error)?
            .member_ids()
            .to_vec();
        for particle in particle_ids {
            candidate
                .storage
                .mark_delete_for_group_lifecycle(particle, call_listener)
                .map_err(storage_object_creation_error)?;
        }
        *self.system_mut_after_validation(system) = candidate;
        Ok(())
    }

    /// Invalidates every pending identity in ascending old-row order.
    ///
    /// # Errors
    ///
    /// Returns a scoped system error without mutation when `system` is invalid.
    ///
    /// # Panics
    ///
    /// Panics only if an internal storage invariant was violated before this
    /// call; public operations cannot construct such a state.
    pub fn compact_pending_particles(
        &mut self,
        system: ParticleSystemId,
    ) -> Result<DestructionReport, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        self.particle_systems.get(system)?;
        let mut preview = self
            .particle_systems
            .get(system)
            .expect("validated particle system remains live")
            .clone();
        crate::particle::lifetime::compact_pending_with_occurrences(&mut preview.storage)
            .expect("validated authoritative storage previews compaction transactionally");
        let empty_groups = Self::empty_particle_groups(&preview);
        let group_records = self
            .prepare_empty_particle_group_records(system, &empty_groups)
            .expect("validated empty group destruction records preflight successfully");
        let outcome = crate::particle::lifetime::compact_pending_with_occurrences(
            &mut self.system_mut_after_validation(system).storage,
        )
        .expect("validated authoritative storage compacts transactionally");
        let mut records = outcome
            .destroyed
            .into_iter()
            .map(|snapshot| Self::particle_destruction_record(snapshot, DestructionCause::Explicit))
            .collect::<Vec<_>>();
        let mut lifecycle = outcome
            .requested_listener_occurrences
            .into_iter()
            .map(|occurrence| {
                let particle = occurrence.particle();
                let record = records
                    .iter()
                    .find(|record| record.destroyed() == DestroyedId::Particle(particle))
                    .cloned()
                    .expect("a requested occurrence always names one compacted particle");
                LifecycleEvent::ParticleDestruction(record)
            })
            .collect::<Vec<_>>();
        Self::remove_empty_particle_group_records(
            self.system_mut_after_validation(system),
            &empty_groups,
        )
        .expect("preflighted empty group records remain removable until immediate commit");
        for record in &group_records {
            self.remove_particle_group_shell_after_compaction(record);
        }
        lifecycle.extend(
            group_records
                .iter()
                .cloned()
                .map(LifecycleEvent::Destruction),
        );
        records.extend(group_records);
        Ok(MutationReport::new(records, lifecycle))
    }

    pub(in crate::world) fn prepare_empty_particle_group_destructions(
        &self,
        owner: ParticleSystemId,
        system: &mut ParticleSystem,
    ) -> Result<Vec<DestructionRecord>, ParticleStorageError> {
        let groups = Self::empty_particle_groups(system);
        let records = self.prepare_empty_particle_group_records(owner, &groups)?;
        Self::remove_empty_particle_group_records(system, &groups)?;
        Ok(records)
    }

    fn empty_particle_groups(system: &ParticleSystem) -> Vec<ParticleGroupId> {
        system
            .groups
            .iter()
            .copied()
            .filter(|group| system.storage.group_will_be_destroyed(*group))
            .collect()
    }

    fn prepare_empty_particle_group_records(
        &self,
        owner: ParticleSystemId,
        groups: &[ParticleGroupId],
    ) -> Result<Vec<DestructionRecord>, ParticleStorageError> {
        let mut records = Vec::new();
        records
            .try_reserve_exact(groups.len())
            .map_err(|_error| ParticleStorageError::InvalidLaneBundle)?;
        for group in groups {
            let shell = self
                .particle_groups
                .get(*group)
                .map_err(handle_to_storage_error)?;
            if shell.system != owner {
                return Err(ParticleStorageError::WrongParticleSystem);
            }
            records.push(DestructionRecord {
                destroyed: DestroyedId::ParticleGroup(*group),
                diagnostic_id: shell.diagnostic_id,
                cause: DestructionCause::Explicit,
                snapshot: ObjectSnapshot::ParticleGroup {
                    system: shell.system,
                    particles: Vec::new(),
                },
            });
        }
        Ok(records)
    }

    fn remove_empty_particle_group_records(
        system: &mut ParticleSystem,
        groups: &[ParticleGroupId],
    ) -> Result<(), ParticleStorageError> {
        for group in groups {
            let removed = system.storage.clear_group(*group)?;
            debug_assert!(removed.is_empty());
        }
        system.groups.retain(|group| !groups.contains(group));
        Ok(())
    }

    pub(in crate::world) fn remove_particle_group_shell_after_compaction(
        &mut self,
        record: &DestructionRecord,
    ) {
        let DestroyedId::ParticleGroup(group) = record.destroyed() else {
            unreachable!("empty-group cleanup records only particle groups");
        };
        self.particle_groups
            .remove(group)
            .expect("preflighted empty particle-group shell remains live until commit");
    }
}

fn handle_to_storage_error(error: HandleError) -> ParticleStorageError {
    match error {
        HandleError::WrongWorld => ParticleStorageError::WrongWorld,
        HandleError::WrongParticleSystem => ParticleStorageError::WrongParticleSystem,
        HandleError::StaleOrDestroyed => ParticleStorageError::StaleOrDestroyed,
        HandleError::PendingDelete => ParticleStorageError::PendingDelete,
        HandleError::WorldPoisoned | HandleError::WrongKind { .. } => {
            ParticleStorageError::InvalidLaneBundle
        }
    }
}
