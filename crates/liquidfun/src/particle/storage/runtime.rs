use super::{
    AggregateGroupFlags, GroupRecord, ParticleBodyContact, ParticleColor, ParticleContact,
    ParticleFlags, ParticleGroupFlags, ParticleGroupId, ParticleId, ParticleIndex, ParticlePair,
    ParticleStorage, ParticleStorageError, ParticleTriad, Range, SemanticBodyContact,
    SemanticParticleContact, StuckLanes, Vec2, group, validate_groups,
};

impl ParticleStorage {
    pub(crate) fn positions(&self) -> &[Vec2] {
        &self.positions
    }

    pub(crate) fn velocities(&self) -> &[Vec2] {
        &self.velocities
    }

    pub(in crate::particle) fn replace_solver_velocities(
        &mut self,
        candidate: Vec<Vec2>,
    ) -> Result<(), ParticleStorageError> {
        if candidate.len() != self.len() || candidate.iter().any(|velocity| !velocity.is_valid()) {
            return Err(ParticleStorageError::InvalidLaneBundle);
        }
        let changed = self
            .velocities
            .iter()
            .zip(&candidate)
            .enumerate()
            .filter_map(|(index, (current, next))| {
                (current != next).then_some(ParticleIndex(index))
            })
            .collect::<Vec<_>>();
        self.velocities = candidate;
        for dense in changed {
            self.invalidate_group_statistics_at(dense);
        }
        Ok(())
    }

    pub(crate) fn flags(&self) -> &[ParticleFlags] {
        &self.flags
    }

    pub(crate) fn groups(&self) -> &[Option<ParticleGroupId>] {
        &self.groups
    }

    pub(in crate::particle) fn group_flags(
        &self,
    ) -> impl ExactSizeIterator<Item = ParticleGroupFlags> + '_ {
        self.group_records.iter().map(|record| record.flags)
    }

    pub(crate) fn group_records(&self) -> &[GroupRecord] {
        &self.group_records
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the solver transaction validates every authoritative candidate lane"
    )]
    pub(crate) fn replace_solver_candidate(
        &mut self,
        particle_ids: &[ParticleId],
        positions: Vec<Vec2>,
        velocities: Vec<Vec2>,
        forces: Vec<Vec2>,
        group_records: Vec<GroupRecord>,
        pending_system_force: bool,
    ) -> Result<(), ParticleStorageError> {
        let count = self.len();
        if particle_ids != self.particle_ids()
            || positions.len() != count
            || velocities.len() != count
            || forces.len() != count
            || positions.iter().any(|position| !position.is_valid())
            || velocities.iter().any(|velocity| !velocity.is_valid())
            || forces.iter().any(|force| !force.is_valid())
        {
            return Err(ParticleStorageError::InvalidLaneBundle);
        }
        validate_groups(self.system, &self.groups, &group_records)?;
        let mut candidate = self.clone();
        candidate.positions = positions;
        candidate.velocities = velocities;
        candidate.forces = forces;
        candidate.group_records = group_records;
        candidate
            .solver_state
            .set_pending_system_force(pending_system_force);
        candidate
            .solver_state
            .refresh_group_flags(&candidate.group_records);
        candidate.check_invariants()?;
        *self = candidate;
        Ok(())
    }

    pub(crate) fn aggregate_particle_flags(&mut self) -> ParticleFlags {
        self.solver_state.refresh_particle_flags(&self.flags);
        self.solver_state.aggregate_particle_flags()
    }

    pub(crate) fn aggregate_group_flags(&mut self) -> AggregateGroupFlags {
        self.solver_state.refresh_group_flags(&self.group_records);
        self.solver_state.aggregate_group_flags()
    }

    pub(in crate::particle) fn ensure_static_pressures(
        &mut self,
    ) -> Result<(), ParticleStorageError> {
        let aggregate = self.aggregate_particle_flags();
        if !aggregate.contains(ParticleFlags::STATIC_PRESSURE) {
            return Err(ParticleStorageError::InvalidLaneBundle);
        }
        let particle_count = self.len();
        self.solver_state
            .ensure_static_pressures(particle_count, self.declared_capacity)
    }

    pub(in crate::particle) fn ensure_tensile_accumulations(
        &mut self,
    ) -> Result<(), ParticleStorageError> {
        let aggregate = self.aggregate_particle_flags();
        if !aggregate.contains(ParticleFlags::TENSILE) {
            return Err(ParticleStorageError::InvalidLaneBundle);
        }
        let particle_count = self.len();
        self.solver_state
            .ensure_tensile_accumulations(particle_count, self.declared_capacity)
    }

    pub(in crate::particle) fn ensure_depths(&mut self) -> Result<(), ParticleStorageError> {
        let aggregate = self.aggregate_group_flags();
        let requires_depth = aggregate.public.contains(ParticleGroupFlags::SOLID)
            || aggregate
                .internal
                .contains(group::InternalGroupFlags::NEEDS_UPDATE_DEPTH);
        if !requires_depth {
            return Err(ParticleStorageError::InvalidLaneBundle);
        }
        let particle_count = self.len();
        self.solver_state
            .ensure_depths(particle_count, self.declared_capacity)
    }

    pub(in crate::particle) fn replace_static_pressures(
        &mut self,
        candidate: Vec<f32>,
    ) -> Result<(), ParticleStorageError> {
        let particle_count = self.len();
        self.solver_state
            .replace_static_pressures(candidate, particle_count)
    }

    pub(in crate::particle) fn replace_tensile_accumulations(
        &mut self,
        candidate: Vec<Vec2>,
    ) -> Result<(), ParticleStorageError> {
        let particle_count = self.len();
        self.solver_state
            .replace_tensile_accumulations(candidate, particle_count)
    }

    pub(in crate::particle) fn replace_depths(
        &mut self,
        candidate: Vec<f32>,
    ) -> Result<(), ParticleStorageError> {
        let particle_count = self.len();
        self.solver_state.replace_depths(candidate, particle_count)
    }

    pub(in crate::particle) fn maybe_depths(&self) -> Option<&[f32]> {
        self.solver_state.maybe_depths()
    }

    pub(in crate::particle) fn maybe_static_pressures(&self) -> Option<&[f32]> {
        self.solver_state.maybe_static_pressures()
    }

    pub(in crate::particle) fn maybe_tensile_accumulations(&self) -> Option<&[Vec2]> {
        self.solver_state.maybe_tensile_accumulations()
    }

    pub(crate) const fn has_pending_system_force(&self) -> bool {
        self.solver_state.has_pending_system_force()
    }

    pub(in crate::particle) fn clear_pending_system_force(&mut self) {
        self.solver_state.clear_pending_system_force();
    }

    pub(in crate::particle) fn weights(&self) -> &[f32] {
        &self.weights
    }

    pub(crate) fn forces(&self) -> &[Vec2] {
        &self.forces
    }

    pub(in crate::particle) fn maybe_colors(&self) -> Option<&[ParticleColor]> {
        self.maybe_colors.as_deref()
    }

    pub(in crate::particle) fn replace_solver_colors(
        &mut self,
        candidate: Vec<ParticleColor>,
    ) -> Result<(), ParticleStorageError> {
        if candidate.len() != self.len() {
            return Err(ParticleStorageError::LaneLengthMismatch);
        }
        let Some(colors) = self.maybe_colors.as_mut() else {
            return Err(ParticleStorageError::InvalidLaneBundle);
        };
        *colors = candidate;
        Ok(())
    }

    pub(crate) fn particle_contacts(&self) -> &[ParticleContact] {
        &self.particle_contacts
    }

    pub(crate) fn semantic_particle_contacts(&self) -> Vec<SemanticParticleContact> {
        self.particle_contacts
            .iter()
            .map(|contact| {
                SemanticParticleContact::new_internal(
                    contact.indices.map(|index| self.particle_id_at(index)),
                    contact.flags,
                    contact.weight,
                    contact.normal,
                )
            })
            .collect()
    }

    pub(crate) fn replace_particle_contacts(
        &mut self,
        contacts: &[SemanticParticleContact],
    ) -> Result<(), ParticleStorageError> {
        let particle_contacts = contacts
            .iter()
            .map(|contact| {
                let [first, second] = contact.particles();
                Ok(ParticleContact {
                    indices: [self.resolve_live(first)?, self.resolve_live(second)?],
                    flags: contact.flags(),
                    weight: contact.weight(),
                    normal: contact.normal(),
                })
            })
            .collect::<Result<Vec<_>, ParticleStorageError>>()?;
        self.particle_contacts = particle_contacts;
        self.recompute_weights();
        debug_assert_eq!(self.check_invariants(), Ok(()));
        Ok(())
    }

    pub(in crate::particle) fn body_contacts(&self) -> &[ParticleBodyContact] {
        &self.body_contacts
    }

    pub(crate) fn semantic_body_contacts(&self) -> Vec<SemanticBodyContact> {
        self.body_contacts
            .iter()
            .map(|contact| {
                SemanticBodyContact::new_internal(
                    self.particle_id_at(contact.index),
                    contact.body,
                    contact.fixture,
                    contact.weight,
                    contact.normal,
                    contact.mass,
                )
            })
            .collect()
    }

    pub(crate) fn replace_body_contacts(
        &mut self,
        contacts: &[SemanticBodyContact],
    ) -> Result<(), ParticleStorageError> {
        let body_contacts = contacts
            .iter()
            .map(|contact| {
                Ok(ParticleBodyContact {
                    index: self.resolve_live(contact.particle())?,
                    body: contact.body(),
                    fixture: contact.fixture(),
                    weight: contact.weight(),
                    normal: contact.normal(),
                    mass: contact.mass(),
                })
            })
            .collect::<Result<Vec<_>, ParticleStorageError>>()?;
        self.body_contacts = body_contacts;
        self.recompute_weights();
        debug_assert_eq!(self.check_invariants(), Ok(()));
        Ok(())
    }

    pub(crate) fn update_stuck_candidates(&mut self, timestamp: u32, threshold: u32) {
        if threshold == 0 {
            return;
        }
        let particle_count = self.len();
        if self.maybe_stuck.is_none() {
            self.maybe_stuck = Some(StuckLanes {
                last_body_contact_steps: vec![0; particle_count],
                body_contact_counts: vec![0; particle_count],
                consecutive_contact_steps: vec![0; particle_count],
                candidates: Vec::new(),
            });
        }
        let stuck = self
            .maybe_stuck
            .as_mut()
            .expect("stuck lanes were allocated before update");
        stuck.body_contact_counts.fill(0);
        stuck.candidates.clear();
        for row in 0..particle_count {
            if timestamp > stuck.last_body_contact_steps[row].saturating_add(1) {
                stuck.consecutive_contact_steps[row] = 0;
            }
        }
        for contact in &self.body_contacts {
            let row = contact.index.0;
            stuck.body_contact_counts[row] += 1;
            if stuck.body_contact_counts[row] == 2 {
                stuck.consecutive_contact_steps[row] += 1;
                if stuck.consecutive_contact_steps[row] > threshold {
                    stuck.candidates.push(contact.index);
                }
            }
            stuck.last_body_contact_steps[row] = timestamp;
        }
    }

    pub(crate) fn particle_velocity(
        &self,
        particle: ParticleId,
    ) -> Result<Vec2, ParticleStorageError> {
        let index = self.resolve_live(particle)?;
        Ok(self.velocities[index.0])
    }

    pub(crate) fn particle_weight(
        &self,
        particle: ParticleId,
    ) -> Result<f32, ParticleStorageError> {
        let index = self.resolve_live(particle)?;
        Ok(self.weights[index.0])
    }

    pub(crate) fn set_particle_velocity_internal(
        &mut self,
        particle: ParticleId,
        velocity: Vec2,
    ) -> Result<(), ParticleStorageError> {
        let index = self.resolve_live(particle)?;
        self.velocities[index.0] = velocity;
        self.invalidate_group_statistics_at(index);
        Ok(())
    }

    pub(in crate::particle) fn resolve_contiguous_live_range(
        &self,
        particles: &[ParticleId],
    ) -> Result<Range<usize>, ParticleStorageError> {
        let indices = particles
            .iter()
            .copied()
            .map(|particle| self.resolve_live(particle))
            .collect::<Result<Vec<_>, _>>()?;
        let Some(first) = indices.first().copied() else {
            return Err(ParticleStorageError::InvalidGroupRange);
        };
        if indices.windows(2).any(|pair| pair[1].0 != pair[0].0 + 1) {
            return Err(ParticleStorageError::InvalidGroupRange);
        }
        Ok(first.0..first.0 + indices.len())
    }

    pub(in crate::particle) fn range_contains_wall(&self, range: Range<usize>) -> bool {
        self.flags[range]
            .iter()
            .any(|flags| flags.contains(ParticleFlags::WALL))
    }

    pub(in crate::particle) fn force_range(&self, range: Range<usize>) -> &[Vec2] {
        &self.forces[range]
    }

    pub(in crate::particle) fn velocity_range(&self, range: Range<usize>) -> &[Vec2] {
        &self.velocities[range]
    }

    pub(in crate::particle) fn replace_force_range(
        &mut self,
        range: Range<usize>,
        forces: &[Vec2],
    ) {
        self.forces[range].copy_from_slice(forces);
        if forces.iter().any(|force| *force != Vec2::ZERO) {
            self.solver_state.mark_pending_system_force();
        }
    }

    pub(in crate::particle) fn replace_velocity_range(
        &mut self,
        range: Range<usize>,
        velocities: &[Vec2],
    ) {
        self.velocities[range].copy_from_slice(velocities);
        for record in &mut self.group_records {
            record.invalidate_statistics();
        }
    }

    pub(in crate::particle) fn stuck_candidates(
        &self,
    ) -> impl ExactSizeIterator<Item = ParticleId> + '_ {
        self.maybe_stuck
            .as_ref()
            .map_or(&[] as &[ParticleIndex], |stuck| stuck.candidates.as_slice())
            .iter()
            .copied()
            .map(|index| self.particle_id_at(index))
    }

    fn recompute_weights(&mut self) {
        Self::recompute_contact_weights(
            &mut self.weights,
            &self.body_contacts,
            &self.particle_contacts,
        );
    }

    pub(in crate::particle) fn refresh_solver_weights(&mut self) {
        self.recompute_weights();
    }

    pub(super) fn recompute_contact_weights(
        weights: &mut [f32],
        body_contacts: &[ParticleBodyContact],
        particle_contacts: &[ParticleContact],
    ) {
        weights.fill(0.0);
        for contact in body_contacts {
            weights[contact.index.0] += contact.weight;
        }
        for contact in particle_contacts {
            for index in contact.indices {
                weights[index.0] += contact.weight;
            }
        }
    }

    pub(crate) fn pairs(&self) -> &[ParticlePair] {
        &self.pairs
    }

    pub(in crate::particle) fn triads(&self) -> &[ParticleTriad] {
        &self.triads
    }

    pub(in crate::particle) fn maybe_expiration_order(&self) -> Option<&[ParticleIndex]> {
        self.maybe_expiration_order.as_deref()
    }
}
