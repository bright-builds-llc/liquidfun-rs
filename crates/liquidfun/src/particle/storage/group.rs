use std::ops::Range;

use crate::identity::{HandleIdentity, ParticleGroupId, ParticleSystemId};
use crate::math::{Transform, Vec2};
use crate::particle::topology::VoronoiLimits;
use crate::particle::topology::constraints::{TopologyGroup, TopologyInput};
use crate::particle::{ParticleFlags, ParticleGroupFlags};

use super::mutation::MutationCandidate;
use super::{ParticleStorage, ParticleStorageError, UserAssociationKey};

mod depth;
mod statistics;

const INTERNAL_GROUP_FLAG_MASK: u8 = 0b0000_0011;
const UPSTREAM_INTERNAL_GROUP_FLAG_MASK: u32 = 0x0018;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(in crate::particle) struct InternalGroupFlags(u8);

impl InternalGroupFlags {
    pub(in crate::particle) const WILL_BE_DESTROYED: Self = Self(0b0000_0001);
    pub(in crate::particle) const NEEDS_UPDATE_DEPTH: Self = Self(0b0000_0010);

    pub(in crate::particle) const fn empty() -> Self {
        Self(0)
    }

    pub(in crate::particle) const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub(in crate::particle) fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub(in crate::particle) fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }

    fn is_valid(self) -> bool {
        self.0 & !INTERNAL_GROUP_FLAG_MASK == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::particle) struct GroupStatisticsCache {
    pub(in crate::particle) maybe_source_timestamp: Option<u32>,
    pub(in crate::particle) mass: f32,
    pub(in crate::particle) center: Vec2,
    pub(in crate::particle) linear_velocity: Vec2,
    pub(in crate::particle) inertia: f32,
    pub(in crate::particle) angular_velocity: f32,
}

impl GroupStatisticsCache {
    pub(in crate::particle) const INVALIDATED_ZERO: Self = Self {
        maybe_source_timestamp: None,
        mass: 0.0,
        center: Vec2::ZERO,
        linear_velocity: Vec2::ZERO,
        inertia: 0.0,
        angular_velocity: 0.0,
    };

    pub(in crate::particle) fn invalidate(&mut self) {
        self.maybe_source_timestamp = None;
    }

    pub(in crate::particle) fn reset_empty(&mut self) {
        *self = Self::INVALIDATED_ZERO;
    }

    fn is_finite(self) -> bool {
        self.mass.is_finite()
            && self.center.is_valid()
            && self.linear_velocity.is_valid()
            && self.inertia.is_finite()
            && self.angular_velocity.is_finite()
    }

    fn is_exact_zero(self) -> bool {
        self.mass.to_bits() == 0
            && self.center.x.to_bits() == 0
            && self.center.y.to_bits() == 0
            && self.linear_velocity.x.to_bits() == 0
            && self.linear_velocity.y.to_bits() == 0
            && self.inertia.to_bits() == 0
            && self.angular_velocity.to_bits() == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::particle) struct GroupRecord {
    pub(in crate::particle) id: ParticleGroupId,
    pub(in crate::particle) system: ParticleSystemId,
    pub(in crate::particle) flags: ParticleGroupFlags,
    pub(in crate::particle) internal_flags: InternalGroupFlags,
    pub(in crate::particle) first: usize,
    pub(in crate::particle) last: usize,
    pub(in crate::particle) strength: f32,
    pub(in crate::particle) transform: Transform,
    pub(in crate::particle) maybe_user_association: Option<UserAssociationKey>,
    pub(in crate::particle) statistics: GroupStatisticsCache,
}

impl GroupRecord {
    pub(in crate::particle) const fn new(
        id: ParticleGroupId,
        system: ParticleSystemId,
        range: Range<usize>,
    ) -> Self {
        Self {
            id,
            system,
            flags: ParticleGroupFlags::empty(),
            internal_flags: InternalGroupFlags::empty(),
            first: range.start,
            last: range.end,
            strength: 1.0,
            transform: Transform::IDENTITY,
            maybe_user_association: None,
            statistics: GroupStatisticsCache::INVALIDATED_ZERO,
        }
    }

    pub(in crate::particle) const fn range(self) -> Range<usize> {
        self.first..self.last
    }

    pub(in crate::particle) fn set_range(&mut self, range: Range<usize>) {
        if self.range() == range {
            return;
        }
        self.first = range.start;
        self.last = range.end;
        self.statistics.invalidate();
        if self.first == self.last {
            self.statistics.reset_empty();
        }
    }

    pub(in crate::particle) fn retain_empty_after_member_removal(&mut self) {
        self.first = 0;
        self.last = 0;
        if !self.flags.contains(ParticleGroupFlags::CAN_BE_EMPTY) {
            self.internal_flags
                .insert(InternalGroupFlags::WILL_BE_DESTROYED);
        }
        self.statistics.reset_empty();
    }

    pub(in crate::particle) fn invalidate_statistics(&mut self) {
        self.statistics.invalidate();
    }

    pub(in crate::particle) fn set_public_flags(&mut self, flags: ParticleGroupFlags) {
        if self.flags == flags {
            return;
        }
        if (self.flags.bits() ^ flags.bits()) & ParticleGroupFlags::SOLID.bits() != 0 {
            self.internal_flags
                .insert(InternalGroupFlags::NEEDS_UPDATE_DEPTH);
        }
        self.flags = flags;
        self.statistics.invalidate();
    }

    pub(in crate::particle) fn validate(
        self,
        owner: ParticleSystemId,
        particle_count: usize,
    ) -> Result<(), ParticleStorageError> {
        if self.system != owner
            || self.id.identity().world() != owner.identity().world()
            || self.first > self.last
            || self.last > particle_count
            || self.flags.bits() & UPSTREAM_INTERNAL_GROUP_FLAG_MASK != 0
            || !self.internal_flags.is_valid()
            || !self.strength.is_finite()
            || !transform_is_finite(self.transform)
            || !self.statistics.is_finite()
        {
            return Err(ParticleStorageError::InvalidGroupRange);
        }
        if self.first != self.last {
            return Ok(());
        }
        let retained = self.flags.contains(ParticleGroupFlags::CAN_BE_EMPTY)
            || self
                .internal_flags
                .contains(InternalGroupFlags::WILL_BE_DESTROYED);
        if !retained || !self.statistics.is_exact_zero() {
            return Err(ParticleStorageError::InvalidGroupRange);
        }
        Ok(())
    }
}

fn transform_is_finite(transform: Transform) -> bool {
    transform.position().is_valid()
        && transform.rotation().sine().is_finite()
        && transform.rotation().cosine().is_finite()
}

impl ParticleStorage {
    pub(crate) fn group_will_be_destroyed(&self, group: ParticleGroupId) -> bool {
        self.group_records.iter().any(|record| {
            record.id == group
                && record
                    .internal_flags
                    .contains(InternalGroupFlags::WILL_BE_DESTROYED)
        })
    }

    pub(in crate::particle) fn regenerate_reactive_topology(
        &mut self,
        particle_diameter: f32,
        voronoi_limits: VoronoiLimits,
    ) -> Result<(), ParticleStorageError> {
        self.check_invariants()?;
        if !self
            .flags
            .iter()
            .any(|flags| flags.contains(ParticleFlags::REACTIVE))
        {
            return Ok(());
        }
        let groups = self.topology_groups()?;
        let generated =
            crate::particle::topology::generate_reactive_pairs_and_triads(&TopologyInput {
                owner: self.system,
                positions: &self.positions,
                flags: &self.flags,
                groups: &groups,
                contacts: &self.particle_contacts,
                range: 0..self.len(),
                particle_diameter,
                voronoi_limits,
            })
            .map_err(|_error| ParticleStorageError::InvalidLaneBundle)?;
        let mut candidate = self.clone();
        let mutation = MutationCandidate::prepare_reactive_regeneration(
            &candidate,
            generated.pairs,
            generated.triads,
        )?;
        mutation.commit(&mut candidate);
        for dense in 0..candidate.len() {
            if !candidate.flags[dense].contains(ParticleFlags::REACTIVE) {
                continue;
            }
            candidate.flags[dense].remove(ParticleFlags::REACTIVE);
            candidate.invalidate_group_statistics_at(super::ParticleIndex(dense));
        }
        candidate
            .solver_state
            .refresh_particle_flags(&candidate.flags);
        candidate.check_invariants()?;
        *self = candidate;
        Ok(())
    }

    fn topology_groups(&self) -> Result<Vec<Option<TopologyGroup>>, ParticleStorageError> {
        let mut groups = Vec::new();
        groups
            .try_reserve_exact(self.len())
            .map_err(|_error| ParticleStorageError::InvalidLaneBundle)?;
        for maybe_group in &self.groups {
            let maybe_topology_group = maybe_group
                .map(|group| {
                    self.group_records
                        .iter()
                        .find(|record| record.id == group)
                        .copied()
                        .map(TopologyGroup::from_record)
                        .ok_or(ParticleStorageError::InvalidGroupRange)
                })
                .transpose()?;
            groups.push(maybe_topology_group);
        }
        Ok(groups)
    }

    pub(in crate::particle) fn set_group_flags_internal(
        &mut self,
        group: ParticleGroupId,
        flags: ParticleGroupFlags,
    ) -> Result<(), ParticleStorageError> {
        self.check_invariants()?;
        let mut candidate = self.clone();
        let record = candidate
            .group_records
            .iter_mut()
            .find(|record| record.id == group && record.system == candidate.system)
            .ok_or(ParticleStorageError::StaleOrDestroyed)?;
        record.set_public_flags(flags);
        let needs_update_depth = record
            .internal_flags
            .contains(InternalGroupFlags::NEEDS_UPDATE_DEPTH);
        candidate.solver_state.mark_group_flags_dirty();
        let mutation = MutationCandidate::prepare_group_flag_change(&candidate)?;
        mutation.commit(&mut candidate);
        candidate
            .solver_state
            .refresh_group_flags(&candidate.group_records);
        if needs_update_depth {
            candidate.ensure_depths()?;
        }
        candidate.check_invariants()?;
        *self = candidate;
        Ok(())
    }

    pub(in crate::particle) fn set_particle_flags_internal(
        &mut self,
        particle: crate::identity::ParticleId,
        flags: ParticleFlags,
    ) -> Result<(), ParticleStorageError> {
        let dense = self.resolve_live(particle)?;
        if self.flags[dense.0] == flags {
            return Ok(());
        }
        self.flags[dense.0] = flags;
        self.solver_state.mark_particle_flags_dirty();
        self.invalidate_group_statistics_at(dense);
        Ok(())
    }
}

#[cfg(test)]
#[path = "group/tests.rs"]
mod tests;
