use super::{
    CreateObjectError, DestroyedId, DestructionCause, DestructionRecord, HandleError,
    HandleIdentity, ObjectSnapshot, ParticleCreationReceipt, ParticleDef, ParticleGroupId,
    ParticleId, ParticleInput, ParticleSnapshot, ParticleSystemId, StorageParticleSnapshot, World,
    particle_input, particle_lifecycle_creation_error, public_particle_snapshot, snapshot_system,
    storage_handle_error, storage_object_creation_error,
};

impl World {
    /// Creates one stable particle from a checked definition.
    ///
    /// Application association values remain application-owned and can be
    /// paired with the returned ID in [`crate::AssociationMap`].
    ///
    /// # Errors
    ///
    /// Returns a scoped owner error, capacity error, or identity exhaustion
    /// before any particle row is committed.
    pub fn create_particle_with_def<UserAssociation>(
        &mut self,
        system: ParticleSystemId,
        maybe_group: Option<ParticleGroupId>,
        definition: &ParticleDef<UserAssociation>,
    ) -> Result<ParticleCreationReceipt, CreateObjectError> {
        self.ensure_not_poisoned_for_handle()?;
        self.particle_systems.get(system)?;
        if let Some(group) = maybe_group {
            let group_record = self.particle_groups.get(group)?;
            if group_record.system != system {
                return Err(CreateObjectError::InvalidHandle(
                    HandleError::WrongParticleSystem,
                ));
            }
        }
        let input = particle_input(definition, maybe_group);
        let mut preflight = self.particle_systems.get(system)?.clone();
        preflight
            .lifetime
            .prepare_capacity_for_creation(&mut preflight.storage)
            .map_err(particle_lifecycle_creation_error)?;
        preflight
            .storage
            .validate_create(input)
            .map_err(storage_object_creation_error)?;
        preflight
            .lifetime
            .validate_created_lifetime(&preflight.storage, definition.lifetime())?;
        let diagnostic_id = self.allocate_diagnostic_id()?;
        Ok(self.commit_preflighted_particle(system, input, definition.lifetime(), diagnostic_id))
    }

    pub(super) fn commit_preflighted_particle(
        &mut self,
        system: ParticleSystemId,
        input: ParticleInput,
        lifetime: f32,
        diagnostic_id: u64,
    ) -> ParticleCreationReceipt {
        let record = self.system_mut_after_validation(system);
        let maybe_compaction = record
            .lifetime
            .prepare_capacity_for_creation(&mut record.storage)
            .expect("preflighted capacity decision remains valid until immediate commit");
        let particle = record
            .storage
            .create_with_diagnostic(input, diagnostic_id)
            .expect("preflighted particle candidate remains valid until immediate commit");
        record
            .lifetime
            .initialize_created_particle(&mut record.storage, particle, lifetime)
            .expect("preflighted lifetime remains valid until immediate commit");
        ParticleCreationReceipt {
            created_particle: particle,
            destruction_occurrences: maybe_compaction
                .map_or_else(Vec::new, |outcome| outcome.requested_listener_occurrences),
        }
    }

    /// Returns owned semantic state after validating a particle's embedded owner.
    ///
    /// # Errors
    ///
    /// Wrong-world, stale-owner, pending-delete, and stale particle states are
    /// reported distinctly.
    pub fn particle_snapshot(&self, particle: ParticleId) -> Result<ParticleSnapshot, HandleError> {
        let system = self.particle_system_id_for_particle(particle)?;
        self.particle_snapshot_in_system(system, particle)
    }

    /// Returns owned semantic state while requiring one explicit owning system.
    ///
    /// # Errors
    ///
    /// In addition to ordinary scoped failures, returns
    /// [`HandleError::WrongParticleSystem`] when the particle belongs elsewhere.
    pub fn particle_snapshot_in_system(
        &self,
        system: ParticleSystemId,
        particle: ParticleId,
    ) -> Result<ParticleSnapshot, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let record = self.particle_systems.get(system)?;
        record
            .storage
            .snapshot(particle)
            .map(public_particle_snapshot)
            .map_err(storage_handle_error)
    }

    /// Marks a live particle pending-delete while preserving an owned snapshot.
    ///
    /// # Errors
    ///
    /// Returns a distinct pending-delete error for a repeated mark.
    pub fn mark_particle_for_destruction(
        &mut self,
        particle: ParticleId,
    ) -> Result<ParticleSnapshot, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let system = self.particle_system_id_for_particle(particle)?;
        self.system_mut_after_validation(system)
            .storage
            .mark_delete(particle)
            .map(public_particle_snapshot)
            .map_err(storage_handle_error)
    }

    pub(in crate::world) fn destroy_particle_now(
        &mut self,
        particle: ParticleId,
    ) -> Result<DestructionRecord, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let system = self.particle_system_id_for_particle(particle)?;
        let storage = &mut self.system_mut_after_validation(system).storage;
        storage
            .mark_delete(particle)
            .map_err(storage_handle_error)?;
        let snapshot = storage
            .compact_particle(particle)
            .expect("the particle marked by this operation remains pending until commit");
        Ok(Self::particle_destruction_record(
            snapshot,
            DestructionCause::Explicit,
        ))
    }

    pub(in crate::world) fn particle_destruction_record(
        snapshot: StorageParticleSnapshot,
        cause: DestructionCause,
    ) -> DestructionRecord {
        DestructionRecord {
            destroyed: DestroyedId::Particle(snapshot.id),
            diagnostic_id: snapshot.diagnostic_id,
            cause,
            snapshot: ObjectSnapshot::Particle {
                system: snapshot_system(snapshot),
                maybe_group: snapshot.input.maybe_group,
            },
        }
    }

    pub(super) fn particle_system_id_for_particle(
        &self,
        particle: ParticleId,
    ) -> Result<ParticleSystemId, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let identity = particle.identity();
        if identity.world() != self.scope_key {
            return Err(HandleError::WrongWorld);
        }
        let Some(system_scope) = identity.maybe_particle_system() else {
            return Err(HandleError::WrongParticleSystem);
        };
        self.particle_system_order
            .iter()
            .copied()
            .find(|system| system.identity().scope() == system_scope)
            .ok_or(HandleError::StaleOrDestroyed)
    }

    pub(super) fn debug_assert_particle_system_order_invariant(&self) {
        debug_assert_eq!(
            self.particle_system_order.len(),
            self.particle_systems.iter().count()
        );
        debug_assert!(
            self.particle_system_order
                .iter()
                .all(|system| self.particle_systems.get(*system).is_ok())
        );
    }
}
