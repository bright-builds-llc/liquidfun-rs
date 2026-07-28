use super::{
    HandleIdentity, IdentityState, OwnedLaneBundle, ParticleBufferBundle, ParticleBufferLanes,
    ParticleBufferMode, ParticleColor, ParticleFlags, ParticleGroupId, ParticleId, ParticleIndex,
    ParticleInput, ParticleProxy, ParticleSnapshot, ParticleStorage, ParticleStorageError, Vec2,
    mutation, push_expiration_order, push_optional, push_optional_stuck, validate_groups,
    validate_reference_sets, validate_references,
};

impl ParticleStorage {
    pub(crate) fn lifetime_tracking_enabled(&self) -> bool {
        self.maybe_expiration_times.is_some()
    }

    pub(in crate::particle) fn enable_lifetime_tracking(&mut self) {
        if self.maybe_expiration_times.is_none() {
            self.maybe_expiration_times = Some(vec![0; self.len()]);
            self.maybe_expiration_order = Some((0..self.len()).map(ParticleIndex).collect());
        }
    }

    pub(in crate::particle) fn expiration_time(
        &self,
        id: ParticleId,
    ) -> Result<i32, ParticleStorageError> {
        let dense = self.resolve_live(id)?;
        self.maybe_expiration_times
            .as_ref()
            .map(|times| times[dense.0])
            .ok_or(ParticleStorageError::InvalidLaneBundle)
    }

    pub(in crate::particle) fn set_expiration_time(
        &mut self,
        id: ParticleId,
        expiration: i32,
    ) -> Result<bool, ParticleStorageError> {
        let dense = self.resolve_live(id)?;
        self.enable_lifetime_tracking();
        let times = self
            .maybe_expiration_times
            .as_mut()
            .expect("lifetime tracking was enabled before mutation");
        let changed = times[dense.0] != expiration;
        times[dense.0] = expiration;
        Ok(changed)
    }

    pub(in crate::particle) fn expiration_entries(&self) -> Vec<(ParticleId, i32)> {
        let Some(times) = self.maybe_expiration_times.as_ref() else {
            return Vec::new();
        };
        let order = self
            .maybe_expiration_order
            .as_ref()
            .expect("expiration times and order are allocated together");
        order
            .iter()
            .map(|index| (self.particle_id_at(*index), times[index.0]))
            .collect()
    }

    pub(in crate::particle) fn replace_expiration_order(
        &mut self,
        ordered_ids: &[ParticleId],
    ) -> Result<(), ParticleStorageError> {
        let order = ordered_ids
            .iter()
            .map(|id| self.resolve_present(*id))
            .collect::<Result<Vec<_>, _>>()?;
        if order.len() != self.len() {
            return Err(ParticleStorageError::InvalidDerivedReference);
        }
        self.maybe_expiration_order = Some(order);
        Ok(())
    }

    pub(in crate::particle) fn is_pending(
        &self,
        id: ParticleId,
    ) -> Result<bool, ParticleStorageError> {
        let local_slot = self.local_slot(id)?;
        let entry = self
            .identities
            .get(local_slot)
            .ok_or(ParticleStorageError::StaleOrDestroyed)?;
        if entry.generation != id.identity().generation() {
            return Err(ParticleStorageError::StaleOrDestroyed);
        }
        Ok(matches!(entry.state, IdentityState::PendingDelete { .. }))
    }

    pub(in crate::particle) fn pending_snapshot(
        &self,
        id: ParticleId,
    ) -> Result<ParticleSnapshot, ParticleStorageError> {
        let local_slot = self.local_slot(id)?;
        let entry = self
            .identities
            .get(local_slot)
            .ok_or(ParticleStorageError::StaleOrDestroyed)?;
        if entry.generation != id.identity().generation() {
            return Err(ParticleStorageError::StaleOrDestroyed);
        }
        match entry.state {
            IdentityState::PendingDelete { snapshot, .. } => Ok(snapshot),
            IdentityState::Live(_) | IdentityState::Vacant | IdentityState::Retired => {
                Err(ParticleStorageError::InvalidPermutation)
            }
        }
    }

    pub(in crate::particle) fn particle_id_at(&self, index: ParticleIndex) -> ParticleId {
        self.dense_to_id[index.0]
    }

    pub(crate) fn into_owned_lanes(self) -> OwnedLaneBundle {
        OwnedLaneBundle {
            positions: self.positions,
            velocities: self.velocities,
            flags: self.flags,
            groups: self.groups,
            weights: self.weights,
            forces: self.forces,
            maybe_colors: self.maybe_colors,
            maybe_user_associations: self.maybe_user_associations,
            maybe_stuck: self.maybe_stuck,
            maybe_expiration_times: self.maybe_expiration_times,
            maybe_expiration_order: self.maybe_expiration_order,
        }
    }

    pub(crate) fn into_buffer_bundle(self, mode: ParticleBufferMode) -> ParticleBufferBundle {
        ParticleBufferBundle::from_storage(
            mode,
            ParticleBufferLanes::new(
                self.positions,
                self.velocities,
                self.flags,
                self.maybe_colors,
            ),
        )
    }

    pub(super) fn identity_slot_candidate(
        &self,
    ) -> Result<(usize, u64, bool), ParticleStorageError> {
        if let Some(local_slot) = self.free_identity_slots.last().copied() {
            let entry = self
                .identities
                .get(local_slot)
                .expect("free identity slots always refer to existing entries");
            debug_assert_eq!(entry.state, IdentityState::Vacant);
            return Ok((local_slot, entry.generation, false));
        }
        if self.identities.len() >= self.identity_capacity {
            if self.retired_identity_slots > 0 {
                return Err(ParticleStorageError::IdentityExhausted);
            }
            return Err(ParticleStorageError::CapacityExceeded {
                limit: self.identity_capacity,
            });
        }

        let local_slot = self.identities.len();
        Ok((local_slot, 0, true))
    }

    pub(super) fn push_row(&mut self, id: ParticleId, input: ParticleInput) {
        let previous_len = self.dense_to_id.len();
        self.dense_to_id.push(id);
        self.positions.push(input.position);
        self.velocities.push(input.velocity);
        self.flags.push(input.flags);
        self.groups.push(input.maybe_group);
        self.weights.push(0.0);
        self.forces.push(Vec2::ZERO);
        let dense = ParticleIndex(previous_len);
        self.proxies.push(ParticleProxy::new(dense));
        push_optional(
            &mut self.maybe_colors,
            input.maybe_color,
            ParticleColor::ZERO,
            previous_len,
        );
        push_optional(
            &mut self.maybe_user_associations,
            input.maybe_user_association.map(Some),
            None,
            previous_len,
        );
        push_optional_stuck(&mut self.maybe_stuck);
        push_optional(
            &mut self.maybe_expiration_times,
            input.maybe_expiration_time,
            0,
            previous_len,
        );
        push_expiration_order(
            &mut self.maybe_expiration_order,
            input.maybe_expiration_time.is_some(),
            dense,
        );
    }

    pub(crate) fn input(&self, id: ParticleId) -> Result<ParticleInput, ParticleStorageError> {
        let dense = self.resolve_live(id)?;
        Ok(self.input_at(dense))
    }

    pub(crate) fn set_position(
        &mut self,
        id: ParticleId,
        position: Vec2,
    ) -> Result<(), ParticleStorageError> {
        let dense = self.resolve_live(id)?;
        self.positions[dense.0] = position;
        self.invalidate_group_statistics_at(dense);
        Ok(())
    }

    pub(crate) fn commit_kinematic_edit(
        &mut self,
        id: ParticleId,
        position: Vec2,
        velocity: Vec2,
    ) -> Result<(), ParticleStorageError> {
        let dense = self.resolve_live(id)?;
        let position_changed = self.positions[dense.0] != position;
        self.positions[dense.0] = position;
        self.velocities[dense.0] = velocity;
        self.invalidate_group_statistics_at(dense);
        if position_changed {
            self.repair_spatial_state();
        }
        debug_assert_eq!(self.check_invariants(), Ok(()));
        Ok(())
    }

    fn repair_spatial_state(&mut self) {
        self.weights.fill(0.0);
        if let Some(stuck) = &mut self.maybe_stuck {
            stuck.last_body_contact_steps.fill(0);
            stuck.body_contact_counts.fill(0);
            stuck.consecutive_contact_steps.fill(0);
            stuck.candidates.clear();
        }
        self.proxies = (0..self.len())
            .map(|index| ParticleProxy::new(ParticleIndex(index)))
            .collect();
        self.particle_contacts.clear();
        self.body_contacts.clear();
        self.pairs.clear();
        self.triads.clear();
    }

    pub(super) fn invalidate_group_statistics_at(&mut self, dense: ParticleIndex) {
        let maybe_group = self.groups[dense.0];
        let Some(group) = maybe_group else {
            return;
        };
        if let Some(record) = self
            .group_records
            .iter_mut()
            .find(|record| record.id == group)
        {
            record.invalidate_statistics();
        }
    }

    pub(crate) fn mark_delete(
        &mut self,
        id: ParticleId,
    ) -> Result<ParticleSnapshot, ParticleStorageError> {
        self.transition_to_pending(id, false)
    }

    fn transition_to_pending(
        &mut self,
        id: ParticleId,
        request_listener: bool,
    ) -> Result<ParticleSnapshot, ParticleStorageError> {
        let dense = self.resolve_live(id)?;
        let local_slot = self.local_slot(id)?;
        self.flags[dense.0].insert(ParticleFlags::ZOMBIE);
        if request_listener {
            self.flags[dense.0].insert(ParticleFlags::DESTRUCTION_LISTENER);
        }
        self.solver_state.mark_particle_flags_dirty();
        self.invalidate_group_statistics_at(dense);
        let snapshot = ParticleSnapshot {
            id,
            diagnostic_id: self.identities[local_slot]
                .diagnostic_id
                .expect("live particles always retain a diagnostic identity"),
            input: self.input_at(dense),
        };
        self.identities[local_slot].state = IdentityState::PendingDelete { dense, snapshot };
        Ok(snapshot)
    }

    pub(in crate::particle) fn mark_delete_for_lifecycle(
        &mut self,
        id: ParticleId,
        request_listener: bool,
    ) -> Result<ParticleSnapshot, ParticleStorageError> {
        self.transition_to_pending(id, request_listener)
    }

    pub(crate) fn mark_delete_for_group_lifecycle(
        &mut self,
        id: ParticleId,
        request_listener: bool,
    ) -> Result<ParticleSnapshot, ParticleStorageError> {
        match self.transition_to_pending(id, request_listener) {
            Ok(snapshot) => Ok(snapshot),
            Err(ParticleStorageError::PendingDelete) => {
                let dense = self.resolve_present(id)?;
                let local_slot = self.local_slot(id)?;
                let IdentityState::PendingDelete { snapshot, .. } =
                    &mut self.identities[local_slot].state
                else {
                    return Err(ParticleStorageError::InvalidPermutation);
                };
                if request_listener {
                    self.flags[dense.0].insert(ParticleFlags::DESTRUCTION_LISTENER);
                    snapshot
                        .input
                        .flags
                        .insert(ParticleFlags::DESTRUCTION_LISTENER);
                }
                Ok(*snapshot)
            }
            Err(error) => Err(error),
        }
    }

    pub(crate) fn synchronize_zombie_flags(&mut self) -> Result<(), ParticleStorageError> {
        for row in 0..self.len() {
            let particle = self.dense_to_id[row];
            if self.is_pending(particle)? {
                self.flags[row].insert(ParticleFlags::ZOMBIE);
                continue;
            }
            if self.flags[row].contains(ParticleFlags::ZOMBIE) {
                self.transition_to_pending(particle, false)?;
            }
        }
        Ok(())
    }

    pub(crate) fn compact_pending(
        &mut self,
    ) -> Result<Vec<ParticleSnapshot>, ParticleStorageError> {
        crate::particle::lifetime::compact_pending_with_occurrences(self)
            .map(|outcome| outcome.destroyed)
    }

    pub(crate) fn compact_particle(
        &mut self,
        id: ParticleId,
    ) -> Result<ParticleSnapshot, ParticleStorageError> {
        let local_slot = self.local_slot(id)?;
        let entry = self
            .identities
            .get(local_slot)
            .ok_or(ParticleStorageError::StaleOrDestroyed)?;
        if entry.generation != id.identity().generation() {
            return Err(ParticleStorageError::StaleOrDestroyed);
        }
        if !matches!(entry.state, IdentityState::PendingDelete { .. }) {
            return Err(ParticleStorageError::InvalidPermutation);
        }

        let mut next = 0;
        let old_to_new = self
            .dense_to_id
            .iter()
            .map(|candidate| {
                if *candidate == id {
                    return None;
                }
                let destination = next;
                next += 1;
                Some(destination)
            })
            .collect::<Vec<_>>();
        let candidate = mutation::MutationCandidate::prepare_zombie_compaction(self, &old_to_new)?;
        let mut destroyed = candidate.commit(self).destroyed;
        let Some(snapshot) = destroyed.pop() else {
            return Err(ParticleStorageError::InvalidPermutation);
        };
        debug_assert!(destroyed.is_empty());
        Ok(snapshot)
    }

    pub(crate) fn rotate_rows(
        &mut self,
        start: usize,
        middle: usize,
        end: usize,
    ) -> Result<(), ParticleStorageError> {
        let candidate =
            mutation::MutationCandidate::prepare_ordinary_rotation(self, start, middle, end)?;
        candidate.commit(self);
        Ok(())
    }

    pub(super) fn resolve_live(
        &self,
        id: ParticleId,
    ) -> Result<ParticleIndex, ParticleStorageError> {
        let local_slot = self.local_slot(id)?;
        let entry = self
            .identities
            .get(local_slot)
            .ok_or(ParticleStorageError::StaleOrDestroyed)?;
        if entry.generation != id.identity().generation() {
            return Err(ParticleStorageError::StaleOrDestroyed);
        }
        match entry.state {
            IdentityState::Live(dense) => Ok(dense),
            IdentityState::PendingDelete { .. } => Err(ParticleStorageError::PendingDelete),
            IdentityState::Vacant | IdentityState::Retired => {
                Err(ParticleStorageError::StaleOrDestroyed)
            }
        }
    }

    pub(super) fn resolve_present(
        &self,
        id: ParticleId,
    ) -> Result<ParticleIndex, ParticleStorageError> {
        let local_slot = self.local_slot(id)?;
        let entry = self
            .identities
            .get(local_slot)
            .ok_or(ParticleStorageError::StaleOrDestroyed)?;
        if entry.generation != id.identity().generation() {
            return Err(ParticleStorageError::StaleOrDestroyed);
        }
        match entry.state {
            IdentityState::Live(dense) | IdentityState::PendingDelete { dense, .. } => Ok(dense),
            IdentityState::Vacant | IdentityState::Retired => {
                Err(ParticleStorageError::StaleOrDestroyed)
            }
        }
    }

    pub(super) fn local_slot(&self, id: ParticleId) -> Result<usize, ParticleStorageError> {
        let identity = id.identity();
        if identity.world() != self.world {
            return Err(ParticleStorageError::WrongWorld);
        }
        if identity.maybe_particle_system() != Some(self.system.identity().scope()) {
            return Err(ParticleStorageError::WrongParticleSystem);
        }
        let Some(local_slot) = identity.slot().checked_sub(self.identity_slot_base) else {
            return Err(ParticleStorageError::WrongParticleSystem);
        };
        if local_slot >= self.identity_capacity {
            return Err(ParticleStorageError::WrongParticleSystem);
        }
        Ok(local_slot)
    }

    pub(super) fn input_at(&self, dense: ParticleIndex) -> ParticleInput {
        ParticleInput {
            position: self.positions[dense.0],
            velocity: self.velocities[dense.0],
            flags: self.flags[dense.0],
            maybe_group: self.groups[dense.0],
            maybe_color: self.maybe_colors.as_ref().map(|lane| lane[dense.0]),
            maybe_user_association: self
                .maybe_user_associations
                .as_ref()
                .and_then(|lane| lane[dense.0]),
            maybe_expiration_time: self
                .maybe_expiration_times
                .as_ref()
                .map(|lane| lane[dense.0]),
        }
    }

    pub(super) fn validate_appended_group(
        &self,
        maybe_group: Option<ParticleGroupId>,
    ) -> Result<(), ParticleStorageError> {
        let Some(group) = maybe_group else {
            return Ok(());
        };
        let Some(last) = self.groups.last() else {
            return Ok(());
        };
        if *last == Some(group) || !self.groups.contains(&Some(group)) {
            return Ok(());
        }
        Err(ParticleStorageError::InvalidGroupRange)
    }

    pub(super) fn check_invariants(&self) -> Result<(), ParticleStorageError> {
        let count = self.dense_to_id.len();
        self.check_lane_lengths(count)?;
        self.check_identity_map()?;
        self.check_derived_references(count)?;
        validate_groups(self.system, &self.groups, &self.group_records)?;
        Ok(())
    }

    fn check_lane_lengths(&self, count: usize) -> Result<(), ParticleStorageError> {
        if self.positions.len() != count
            || self.velocities.len() != count
            || self.flags.len() != count
            || self.groups.len() != count
            || self.weights.len() != count
            || self.forces.len() != count
            || self
                .maybe_colors
                .as_ref()
                .is_some_and(|lane| lane.len() != count)
            || self
                .maybe_user_associations
                .as_ref()
                .is_some_and(|lane| lane.len() != count)
            || self.maybe_stuck.as_ref().is_some_and(|lanes| {
                lanes.last_body_contact_steps.len() != count
                    || lanes.body_contact_counts.len() != count
                    || lanes.consecutive_contact_steps.len() != count
            })
            || self
                .maybe_expiration_times
                .as_ref()
                .is_some_and(|lane| lane.len() != count)
            || self
                .maybe_expiration_order
                .as_ref()
                .is_some_and(|lane| lane.len() != count)
        {
            return Err(ParticleStorageError::LaneLengthMismatch);
        }
        self.solver_state
            .validate(count, &self.flags, &self.group_records)?;
        Ok(())
    }

    fn check_identity_map(&self) -> Result<(), ParticleStorageError> {
        for (dense, id) in self.dense_to_id.iter().copied().enumerate() {
            let local_slot = self.local_slot(id)?;
            let entry = self
                .identities
                .get(local_slot)
                .ok_or(ParticleStorageError::StaleOrDestroyed)?;
            if entry.generation != id.identity().generation()
                || !matches!(
                    entry.state,
                    IdentityState::Live(ParticleIndex(index))
                        | IdentityState::PendingDelete {
                            dense: ParticleIndex(index),
                            ..
                        } if index == dense
                )
            {
                return Err(ParticleStorageError::StaleOrDestroyed);
            }
            if self.dense_to_id[..dense].contains(&id) {
                return Err(ParticleStorageError::StaleOrDestroyed);
            }
        }
        Ok(())
    }

    fn check_derived_references(&self, count: usize) -> Result<(), ParticleStorageError> {
        validate_references(
            &self
                .proxies
                .iter()
                .map(|proxy| proxy.index)
                .collect::<Vec<_>>(),
            count,
        )?;
        validate_reference_sets(
            &self
                .particle_contacts
                .iter()
                .map(|contact| contact.indices)
                .collect::<Vec<_>>(),
            count,
        )?;
        validate_references(
            &self
                .body_contacts
                .iter()
                .map(|contact| contact.index)
                .collect::<Vec<_>>(),
            count,
        )?;
        validate_reference_sets(
            &self
                .pairs
                .iter()
                .map(|pair| pair.indices)
                .collect::<Vec<_>>(),
            count,
        )?;
        validate_reference_sets(
            &self
                .triads
                .iter()
                .map(|triad| triad.indices)
                .collect::<Vec<_>>(),
            count,
        )?;
        if let Some(order) = &self.maybe_expiration_order {
            validate_references(order, count)?;
        }
        if let Some(stuck) = &self.maybe_stuck {
            validate_references(&stuck.candidates, count)?;
        }
        Ok(())
    }
}
