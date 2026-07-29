use super::{
    CreateCandidate, HandleIdentity, Identity, IdentityEntry, IdentityState, OwnedLaneBundle,
    ParticleBufferBundle, ParticleGroupId, ParticleGroupView, ParticleGroupViewState, ParticleId,
    ParticleIndex, ParticleInput, ParticleSnapshot, ParticleStorage, ParticleStorageError,
    ParticleSystemId, SolverState, Vec2, WorldKey, rebuild_group_records_for_system,
    validate_groups,
};

impl ParticleStorage {
    pub(crate) fn new(
        world: WorldKey,
        system: ParticleSystemId,
        identity_slot_base: usize,
        identity_capacity: usize,
        declared_capacity: usize,
    ) -> Result<Self, ParticleStorageError> {
        Self::with_initial_capacity(
            world,
            system,
            identity_slot_base,
            identity_capacity,
            declared_capacity,
            declared_capacity,
        )
    }

    pub(crate) fn with_initial_capacity(
        world: WorldKey,
        system: ParticleSystemId,
        identity_slot_base: usize,
        identity_capacity: usize,
        initial_capacity: usize,
        declared_capacity: usize,
    ) -> Result<Self, ParticleStorageError> {
        if initial_capacity > declared_capacity {
            return Err(ParticleStorageError::InvalidLaneBundle);
        }
        let lanes = OwnedLaneBundle::with_capacity(initial_capacity, false);
        Self::from_validated_lanes(
            world,
            system,
            identity_slot_base,
            identity_capacity,
            declared_capacity,
            lanes,
            initial_capacity,
        )
    }

    pub(crate) fn from_buffer_bundle(
        world: WorldKey,
        system: ParticleSystemId,
        identity_slot_base: usize,
        identity_capacity: usize,
        declared_capacity: usize,
        bundle: ParticleBufferBundle,
    ) -> Self {
        let (mode, supplied) = bundle.into_parts();
        let initial_capacity = mode.declared_count();
        let lanes = OwnedLaneBundle {
            positions: supplied.positions,
            velocities: supplied.velocities,
            flags: supplied.flags,
            groups: Vec::with_capacity(initial_capacity),
            weights: Vec::with_capacity(initial_capacity),
            forces: Vec::with_capacity(initial_capacity),
            maybe_colors: supplied.maybe_colors,
            maybe_user_associations: None,
            maybe_stuck: None,
            maybe_expiration_times: None,
            maybe_expiration_order: None,
        };
        Self::from_validated_lanes(
            world,
            system,
            identity_slot_base,
            identity_capacity,
            declared_capacity,
            lanes,
            initial_capacity,
        )
        .expect("validated owned lanes and world-scoped system form valid storage")
    }

    fn from_validated_lanes(
        world: WorldKey,
        system: ParticleSystemId,
        identity_slot_base: usize,
        identity_capacity: usize,
        declared_capacity: usize,
        lanes: OwnedLaneBundle,
        minimum_lane_capacity: usize,
    ) -> Result<Self, ParticleStorageError> {
        if system.identity().world() != world {
            return Err(ParticleStorageError::WrongWorld);
        }
        if identity_slot_base.checked_add(identity_capacity).is_none() {
            return Err(ParticleStorageError::IdentityExhausted);
        }
        lanes.validate_empty(minimum_lane_capacity)?;

        Ok(Self {
            world,
            system,
            identity_slot_base,
            identity_capacity,
            declared_capacity,
            identities: Vec::new(),
            free_identity_slots: Vec::new(),
            retired_identity_slots: 0,
            dense_to_id: Vec::with_capacity(minimum_lane_capacity),
            positions: lanes.positions,
            velocities: lanes.velocities,
            flags: lanes.flags,
            groups: lanes.groups,
            weights: lanes.weights,
            forces: lanes.forces,
            maybe_colors: lanes.maybe_colors,
            maybe_user_associations: lanes.maybe_user_associations,
            maybe_stuck: lanes.maybe_stuck,
            maybe_expiration_times: lanes.maybe_expiration_times,
            maybe_expiration_order: lanes.maybe_expiration_order,
            proxies: Vec::with_capacity(minimum_lane_capacity),
            particle_contacts: Vec::new(),
            body_contacts: Vec::new(),
            pairs: Vec::new(),
            triads: Vec::new(),
            group_records: Vec::new(),
            solver_state: SolverState::new(),
        })
    }

    pub(crate) fn system(&self) -> ParticleSystemId {
        self.system
    }

    pub(crate) fn len(&self) -> usize {
        self.dense_to_id.len()
    }

    pub(crate) fn pending_count(&self) -> usize {
        self.identities
            .iter()
            .filter(|entry| matches!(entry.state, IdentityState::PendingDelete { .. }))
            .count()
    }

    pub(crate) const fn declared_capacity(&self) -> usize {
        self.declared_capacity
    }

    pub(crate) fn snapshots(&self) -> Vec<ParticleSnapshot> {
        self.dense_to_id
            .iter()
            .copied()
            .map(|id| self.snapshot_for_teardown(id))
            .collect()
    }

    pub(crate) fn particle_ids(&self) -> &[ParticleId] {
        &self.dense_to_id
    }

    pub(crate) fn group_view(
        &self,
        group: ParticleGroupId,
        particle_mass: f32,
    ) -> Result<ParticleGroupView<'_>, ParticleStorageError> {
        let record = self
            .group_records
            .iter()
            .find(|record| record.id == group && record.system == self.system)
            .copied()
            .ok_or(ParticleStorageError::StaleOrDestroyed)?;
        let range = record.range();
        let count = range.len();
        #[allow(
            clippy::cast_precision_loss,
            reason = "the pinned C++ statistics path converts the bounded particle count to float32"
        )]
        let mass = particle_mass * count as f32;
        let inverse_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };
        let center = self.positions[range.clone()]
            .iter()
            .copied()
            .fold(Vec2::ZERO, |sum, position| sum + particle_mass * position)
            * inverse_mass;
        let linear_velocity = self.velocities[range.clone()]
            .iter()
            .copied()
            .fold(Vec2::ZERO, |sum, velocity| sum + particle_mass * velocity)
            * inverse_mass;
        let (inertia, angular_momentum) = self.positions[range.clone()]
            .iter()
            .copied()
            .zip(self.velocities[range.clone()].iter().copied())
            .fold((0.0, 0.0), |(inertia, angular), (position, velocity)| {
                let relative_position = position - center;
                let relative_velocity = velocity - linear_velocity;
                (
                    inertia + particle_mass * relative_position.dot(relative_position),
                    angular + particle_mass * relative_position.cross(relative_velocity),
                )
            });
        let angular_velocity = if inertia > 0.0 {
            angular_momentum / inertia
        } else {
            0.0
        };
        let maybe_depths = self
            .solver_state
            .maybe_depths()
            .map(|depths| &depths[range.clone()]);
        ParticleGroupView::new(
            ParticleGroupViewState {
                id: group,
                flags: record.flags,
                transform: record.transform,
                center,
                linear_velocity,
                angular_velocity,
                mass,
                inertia,
            },
            &self.dense_to_id[range],
            maybe_depths,
        )
        .map_err(|_error| ParticleStorageError::LaneLengthMismatch)
    }

    pub(crate) fn clear_group(
        &mut self,
        group: ParticleGroupId,
    ) -> Result<Vec<ParticleId>, ParticleStorageError> {
        self.check_invariants()?;
        if !self
            .group_records
            .iter()
            .any(|record| record.id == group && record.system == self.system)
        {
            return Err(ParticleStorageError::InvalidGroupRange);
        }
        let particles = self
            .dense_to_id
            .iter()
            .copied()
            .zip(self.groups.iter().copied())
            .filter_map(|(id, maybe_group)| (maybe_group == Some(group)).then_some(id))
            .collect::<Vec<_>>();
        let mut groups = self.groups.clone();
        for maybe_group in &mut groups {
            if *maybe_group == Some(group) {
                *maybe_group = None;
            }
        }
        let group_records = self
            .group_records
            .iter()
            .copied()
            .filter(|record| record.id != group)
            .collect::<Vec<_>>();
        validate_groups(self.system, &groups, &group_records)?;
        self.groups = groups;
        self.group_records = group_records;
        self.solver_state.refresh_group_flags(&self.group_records);
        debug_assert_eq!(self.check_invariants(), Ok(()));
        Ok(particles)
    }

    pub(crate) fn snapshot(
        &self,
        id: ParticleId,
    ) -> Result<ParticleSnapshot, ParticleStorageError> {
        let dense = self.resolve_live(id)?;
        let local_slot = self.local_slot(id)?;
        Ok(ParticleSnapshot {
            id,
            diagnostic_id: self.identities[local_slot]
                .diagnostic_id
                .expect("live particles always retain a diagnostic identity"),
            input: self.input_at(dense),
        })
    }

    fn snapshot_for_teardown(&self, id: ParticleId) -> ParticleSnapshot {
        let local_slot = self
            .local_slot(id)
            .expect("dense identities remain scoped to their owning storage");
        let entry = &self.identities[local_slot];
        match entry.state {
            IdentityState::Live(dense) => ParticleSnapshot {
                id,
                diagnostic_id: entry
                    .diagnostic_id
                    .expect("live particles always retain a diagnostic identity"),
                input: self.input_at(dense),
            },
            IdentityState::PendingDelete { snapshot, .. } => snapshot,
            IdentityState::Vacant | IdentityState::Retired => {
                unreachable!("dense rows cannot refer to vacant or retired identities")
            }
        }
    }

    pub(crate) fn create_with_diagnostic(
        &mut self,
        input: ParticleInput,
        diagnostic_id: u64,
    ) -> Result<ParticleId, ParticleStorageError> {
        let candidate = self.prepare_create(input, diagnostic_id)?;
        Ok(self.commit_create(candidate))
    }

    pub(crate) fn validate_create(&self, input: ParticleInput) -> Result<(), ParticleStorageError> {
        self.prepare_create(input, 0).map(|_candidate| ())
    }

    pub(crate) fn create(
        &mut self,
        input: ParticleInput,
    ) -> Result<ParticleId, ParticleStorageError> {
        self.create_with_diagnostic(input, 0)
    }

    fn prepare_create(
        &self,
        input: ParticleInput,
        diagnostic_id: u64,
    ) -> Result<CreateCandidate, ParticleStorageError> {
        if self.dense_to_id.len() >= self.declared_capacity {
            return Err(ParticleStorageError::CapacityExceeded {
                limit: self.declared_capacity,
            });
        }
        self.validate_appended_group(input.maybe_group)?;
        let (local_slot, generation, append_identity) = self.identity_slot_candidate()?;
        let particle_slot = self
            .identity_slot_base
            .checked_add(local_slot)
            .ok_or(ParticleStorageError::IdentityExhausted)?;
        let id = ParticleId::from_identity(Identity::new_particle(
            self.world,
            particle_slot,
            generation,
            self.system.identity(),
        ));
        let dense = ParticleIndex(self.dense_to_id.len());
        let mut groups = self.groups.clone();
        groups.push(input.maybe_group);
        let group_records =
            rebuild_group_records_for_system(&self.group_records, &groups, self.system)?;
        let solver_state = self.solver_state.prepare_append(
            &self.flags,
            input.flags,
            &group_records,
            self.declared_capacity,
        )?;
        Ok(CreateCandidate {
            input,
            diagnostic_id,
            id,
            local_slot,
            generation,
            append_identity,
            dense,
            group_records,
            solver_state,
        })
    }

    fn commit_create(&mut self, candidate: CreateCandidate) -> ParticleId {
        if candidate.append_identity {
            self.identities.push(IdentityEntry {
                generation: candidate.generation,
                diagnostic_id: None,
                state: IdentityState::Vacant,
            });
        } else {
            let reused = self
                .free_identity_slots
                .pop()
                .expect("prepared reused identity remains available until commit");
            debug_assert_eq!(reused, candidate.local_slot);
        }
        self.identities[candidate.local_slot].diagnostic_id = Some(candidate.diagnostic_id);
        self.identities[candidate.local_slot].state = IdentityState::Live(candidate.dense);
        self.push_row(candidate.id, candidate.input);
        self.group_records = candidate.group_records;
        self.solver_state = candidate.solver_state;
        debug_assert_eq!(self.check_invariants(), Ok(()));
        candidate.id
    }

    pub(crate) fn from_owned_lanes(
        world: WorldKey,
        system: ParticleSystemId,
        identity_slot_base: usize,
        identity_capacity: usize,
        declared_capacity: usize,
        lanes: OwnedLaneBundle,
    ) -> Result<Self, ParticleStorageError> {
        Self::from_validated_lanes(
            world,
            system,
            identity_slot_base,
            identity_capacity,
            declared_capacity,
            lanes,
            declared_capacity,
        )
    }
}
