use super::{
    ArenaInsertError, HandleError, MAX_PARTICLE_COUNT, ParticleBufferAdoptionError,
    ParticleBufferAdoptionErrorKind, ParticleBufferBundle, ParticleCapacity, ParticleEditError,
    ParticleEditor, ParticleId, ParticleLifetimeState, ParticleStorage, ParticleSystem,
    ParticleSystemDef, ParticleSystemId, ParticleSystemSnapshot, ParticleSystemStatistics,
    ParticleSystemView, ParticleWorldStatistics, Vec2, World, force, storage_creation_error,
    storage_handle_error,
};

impl World {
    /// Creates a particle system that uniquely owns a validated consumer lane bundle.
    ///
    /// The bundle's fixed or growable mode replaces the definition's ordinary
    /// capacity policy. A configured maximum must still fit a fixed bundle.
    ///
    /// # Errors
    ///
    /// Returns the complete bundle with a typed definition or world allocation
    /// failure. No system, diagnostic identity, or lane ownership is committed.
    pub fn create_particle_system_with_buffers(
        &mut self,
        definition: &ParticleSystemDef,
        bundle: ParticleBufferBundle,
    ) -> Result<ParticleSystemId, ParticleBufferAdoptionError> {
        let mode = bundle.mode();
        let capacity = ParticleCapacity::from_buffer_mode(mode);
        let definition = match definition.with_capacity(capacity) {
            Ok(definition) => definition,
            Err(error) => {
                return Err(ParticleBufferAdoptionError::new(
                    ParticleBufferAdoptionErrorKind::Definition(error),
                    bundle,
                ));
            }
        };
        if let Err(error) = self.ensure_not_poisoned_for_insert() {
            return Err(ParticleBufferAdoptionError::new(
                ParticleBufferAdoptionErrorKind::World(error),
                bundle,
            ));
        }
        let system = match self.particle_systems.next_handle() {
            Ok(system) => system,
            Err(error) => {
                return Err(ParticleBufferAdoptionError::new(
                    ParticleBufferAdoptionErrorKind::World(error),
                    bundle,
                ));
            }
        };
        let declared_capacity = if capacity.is_fixed() {
            capacity.count()
        } else {
            definition.maximum_count().unwrap_or(MAX_PARTICLE_COUNT)
        };
        let mut storage = ParticleStorage::from_buffer_bundle(
            self.scope_key,
            system,
            0,
            declared_capacity,
            declared_capacity,
            bundle,
        );
        let lifetime = ParticleLifetimeState::new(definition, &mut storage);
        let diagnostic_id = match self.allocate_diagnostic_id() {
            Ok(diagnostic_id) => diagnostic_id,
            Err(error) => {
                return Err(ParticleBufferAdoptionError::new(
                    ParticleBufferAdoptionErrorKind::World(error),
                    storage.into_buffer_bundle(mode),
                ));
            }
        };
        self.insert_particle_system_after_preflight(
            system,
            ParticleSystem {
                diagnostic_id,
                definition,
                groups: Vec::new(),
                storage,
                lifetime,
                timestamp: 0,
            },
        );
        self.particle_system_order.insert(0, system);
        self.debug_assert_particle_system_order_invariant();
        Ok(system)
    }

    pub(super) fn insert_particle_system_after_preflight(
        &mut self,
        expected: ParticleSystemId,
        record: ParticleSystem,
    ) {
        let inserted = self
            .particle_systems
            .insert(record)
            .expect("preflighted particle-system slot remains available until insertion");
        debug_assert_eq!(inserted, expected);
    }

    /// Creates a particle system from a checked reusable definition.
    ///
    /// # Errors
    ///
    /// Returns an allocation error before insertion when world identities or
    /// particle-system storage are exhausted.
    pub fn create_particle_system_with_def(
        &mut self,
        definition: &ParticleSystemDef,
    ) -> Result<ParticleSystemId, ArenaInsertError> {
        self.ensure_not_poisoned_for_insert()?;
        let system = self.particle_systems.next_handle()?;
        let capacity = definition.capacity();
        let declared_capacity = if capacity.is_fixed() {
            capacity.count()
        } else {
            definition.maximum_count().unwrap_or(MAX_PARTICLE_COUNT)
        };
        let mut storage = ParticleStorage::with_initial_capacity(
            self.scope_key,
            system,
            0,
            declared_capacity,
            capacity.count(),
            declared_capacity,
        )
        .map_err(storage_creation_error)?;
        let lifetime = ParticleLifetimeState::new(*definition, &mut storage);
        let diagnostic_id = self.allocate_diagnostic_id()?;
        let inserted = self.particle_systems.insert(ParticleSystem {
            diagnostic_id,
            definition: *definition,
            groups: Vec::new(),
            storage,
            lifetime,
            timestamp: 0,
        })?;
        debug_assert_eq!(inserted, system);
        self.particle_system_order.insert(0, system);
        self.debug_assert_particle_system_order_invariant();
        Ok(system)
    }

    /// Enumerates live systems in pinned newest-first world order.
    #[must_use]
    pub fn particle_system_ids(
        &self,
    ) -> impl ExactSizeIterator<Item = ParticleSystemId> + DoubleEndedIterator + '_ {
        self.particle_system_order.iter().copied()
    }

    /// Returns owned configuration and membership state for a live system.
    ///
    /// # Errors
    ///
    /// Returns a scoped handle error when `system` is foreign or stale.
    pub fn particle_system_snapshot(
        &self,
        system: ParticleSystemId,
    ) -> Result<ParticleSystemSnapshot, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let record = self.particle_systems.get(system)?;
        Ok(ParticleSystemSnapshot {
            definition: record.definition,
            particle_count: record.storage.len(),
            pending_particle_count: record.storage.pending_count(),
        })
    }

    /// Returns an owned semantic statistics snapshot for one live system.
    ///
    /// Stable identities replace dense rows, and capacity values come only
    /// from explicit contracts rather than allocator internals.
    ///
    /// # Errors
    ///
    /// Returns a scoped handle error when `system` is foreign or stale.
    pub fn particle_system_statistics(
        &self,
        system: ParticleSystemId,
    ) -> Result<ParticleSystemStatistics, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let record = self.particle_systems.get(system)?;
        Ok(ParticleSystemStatistics::from_storage(
            &record.storage,
            record.definition,
            record.groups.len(),
        ))
    }

    /// Returns owned aggregate semantic counts in system traversal order.
    ///
    /// # Panics
    ///
    /// Panics only if an internal particle-system order invariant was already
    /// violated; public operations maintain the bidirectional owner list.
    #[must_use]
    pub fn particle_world_statistics(&self) -> ParticleWorldStatistics {
        let mut statistics = ParticleWorldStatistics::default();
        for system in &self.particle_system_order {
            let record = self
                .particle_systems
                .get(*system)
                .expect("particle-system order contains only live systems");
            statistics.include(&ParticleSystemStatistics::from_storage(
                &record.storage,
                record.definition,
                record.groups.len(),
            ));
        }
        statistics
    }

    /// Borrows every supported semantic lane for one live particle system.
    ///
    /// The returned view keeps this world immutably borrowed, preventing
    /// structural particle mutation while any lane or derived record is in use.
    /// Dense rows, capacities, scratch state, and mutable slices remain private.
    ///
    /// # Errors
    ///
    /// Returns a scoped handle error when `system` is foreign or stale.
    pub fn particle_system_view(
        &self,
        system: ParticleSystemId,
    ) -> Result<ParticleSystemView<'_>, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let record = self.particle_systems.get(system)?;
        Ok(ParticleSystemView::new(&record.storage))
    }

    /// Applies one closure-scoped edit after validating the complete candidate.
    ///
    /// Position changes synchronously rebuild proxies and clear contact- and
    /// spatially-derived records. A closure panic occurs before storage mutation.
    ///
    /// # Errors
    ///
    /// Returns a scoped handle or non-finite candidate error without mutation.
    pub fn edit_particle<R>(
        &mut self,
        particle: ParticleId,
        edit: impl for<'edit> FnOnce(&mut ParticleEditor<'edit>) -> Result<R, ParticleEditError>,
    ) -> Result<R, ParticleEditError> {
        self.ensure_not_poisoned_for_handle()?;
        let system = self.particle_system_id_for_particle(particle)?;
        let input = self
            .particle_systems
            .get(system)?
            .storage
            .input(particle)
            .map_err(storage_handle_error)?;
        let mut editor = ParticleEditor::new(input.position, input.velocity);
        let output = edit(&mut editor)?;
        let (position, velocity) = editor.into_parts();
        self.system_mut_after_validation(system)
            .storage
            .commit_kinematic_edit(particle, position, velocity)
            .map_err(storage_handle_error)?;
        Ok(output)
    }

    /// Sets a stable particle position and repairs spatially derived state.
    ///
    /// # Errors
    ///
    /// Returns a scoped handle or non-finite position error without mutation.
    pub fn set_particle_position(
        &mut self,
        particle: ParticleId,
        position: Vec2,
    ) -> Result<(), ParticleEditError> {
        self.edit_particle(particle, |editor| editor.set_position(position))
    }

    /// Sets a stable particle velocity without exposing a mutable lane.
    ///
    /// # Errors
    ///
    /// Returns a scoped handle or non-finite velocity error without mutation.
    pub fn set_particle_velocity(
        &mut self,
        particle: ParticleId,
        velocity: Vec2,
    ) -> Result<(), ParticleEditError> {
        self.edit_particle(particle, |editor| editor.set_velocity(velocity))
    }

    /// Accumulates one checked world-space force for a stable particle.
    ///
    /// # Errors
    ///
    /// Returns a scoped handle, non-finite vector, wall-particle, or derived
    /// distribution error without mutation.
    pub fn apply_particle_force(
        &mut self,
        particle: ParticleId,
        force: Vec2,
    ) -> Result<(), crate::ParticleForceError> {
        let system = self.particle_system_id_for_particle(particle)?;
        self.apply_particle_force_range(system, &[particle], force)
    }

    /// Distributes one checked force over contiguous stable identities.
    ///
    /// The identities must name every particle in one current source-ordered
    /// contiguous range. Validation completes before any force lane changes.
    ///
    /// # Errors
    ///
    /// Returns a scoped handle, empty/non-contiguous range, non-finite vector,
    /// wall-particle, or derived distribution error without mutation.
    pub fn apply_particle_force_range(
        &mut self,
        system: ParticleSystemId,
        particles: &[ParticleId],
        force: Vec2,
    ) -> Result<(), crate::ParticleForceError> {
        self.ensure_not_poisoned_for_handle()?;
        let prepared = {
            let record = self.particle_systems.get(system)?;
            force::prepare_force(&record.storage, particles, force)?
        };
        force::apply_force(
            &mut self.system_mut_after_validation(system).storage,
            prepared,
        );
        Ok(())
    }

    /// Applies one checked world-space linear impulse to a stable particle.
    ///
    /// # Errors
    ///
    /// Returns a scoped handle, non-finite vector, wall-particle, or derived
    /// mass/distribution error without mutation.
    pub fn apply_particle_linear_impulse(
        &mut self,
        particle: ParticleId,
        impulse: Vec2,
    ) -> Result<(), crate::ParticleForceError> {
        let system = self.particle_system_id_for_particle(particle)?;
        self.apply_particle_linear_impulse_range(system, &[particle], impulse)
    }

    /// Distributes one checked linear impulse over contiguous stable identities.
    ///
    /// The impulse targets the selected particles' total source-derived mass,
    /// so each selected velocity receives the same delta.
    ///
    /// # Errors
    ///
    /// Returns a scoped handle, empty/non-contiguous range, non-finite vector,
    /// wall-particle, or derived mass/distribution error without mutation.
    pub fn apply_particle_linear_impulse_range(
        &mut self,
        system: ParticleSystemId,
        particles: &[ParticleId],
        impulse: Vec2,
    ) -> Result<(), crate::ParticleForceError> {
        self.ensure_not_poisoned_for_handle()?;
        let prepared = {
            let record = self.particle_systems.get(system)?;
            force::prepare_impulse(&record.storage, record.definition, particles, impulse)?
        };
        force::apply_impulse(
            &mut self.system_mut_after_validation(system).storage,
            prepared,
        );
        Ok(())
    }

    /// Changes only the paused state of a live system.
    ///
    /// # Errors
    ///
    /// Returns a scoped handle error without mutation for a foreign or stale system.
    pub fn set_particle_system_paused(
        &mut self,
        system: ParticleSystemId,
        paused: bool,
    ) -> Result<(), HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let definition = self
            .particle_systems
            .get(system)?
            .definition
            .with_paused(paused);
        self.system_mut_after_validation(system).definition = definition;
        Ok(())
    }
}
