use std::ops::Range;

use crate::identity::{HandleIdentity, ParticleGroupId, ParticleSystemId};
use crate::math::{Transform, Vec2};
use crate::particle::ParticleGroupFlags;

use super::{ParticleStorageError, UserAssociationKey};

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

#[cfg(test)]
mod tests {
    use crate::identity::{HandleIdentity, Identity, WorldKey};
    use crate::math::Rotation;
    use crate::particle::ParticleFlags;

    use super::*;
    use crate::particle::storage::{ParticleInput, ParticleStorage};

    fn identities() -> (ParticleSystemId, ParticleGroupId) {
        let world = WorldKey::fresh().expect("test world key remains available");
        (
            ParticleSystemId::from_identity(Identity::new(world, 0, 0)),
            ParticleGroupId::from_identity(Identity::new(world, 1, 0)),
        )
    }

    #[test]
    fn cache_invalidation_clears_only_the_source_timestamp() {
        // Arrange
        let mut cache = GroupStatisticsCache {
            maybe_source_timestamp: Some(7),
            mass: 2.0,
            center: Vec2::new(1.0, 2.0),
            linear_velocity: Vec2::new(3.0, 4.0),
            inertia: 5.0,
            angular_velocity: 6.0,
        };

        // Act
        cache.invalidate();

        // Assert
        assert_eq!(cache.maybe_source_timestamp, None);
        assert_eq!(cache.mass.to_bits(), 2.0_f32.to_bits());
        assert_eq!(cache.center, Vec2::new(1.0, 2.0));
    }

    #[test]
    fn empty_transition_resets_every_aggregate_to_positive_zero() {
        // Arrange
        let (system, group) = identities();
        let mut record = GroupRecord::new(group, system, 2..4);
        record.statistics = GroupStatisticsCache {
            maybe_source_timestamp: Some(9),
            mass: 2.0,
            center: Vec2::new(1.0, 2.0),
            linear_velocity: Vec2::new(3.0, 4.0),
            inertia: 5.0,
            angular_velocity: 6.0,
        };

        // Act
        record.retain_empty_after_member_removal();

        // Assert
        assert_eq!(record.range(), 0..0);
        assert_eq!(record.statistics, GroupStatisticsCache::INVALIDATED_ZERO);
        assert!(
            record
                .internal_flags
                .contains(InternalGroupFlags::WILL_BE_DESTROYED)
        );
        assert_eq!(record.validate(system, 4), Ok(()));
    }

    #[test]
    fn non_finite_transform_or_statistics_are_rejected() {
        // Arrange
        let (system, group) = identities();
        let mut invalid_transform = GroupRecord::new(group, system, 0..1);
        invalid_transform.transform = Transform::new(Vec2::ZERO, Rotation::from_angle(f32::NAN));
        let mut invalid_cache = GroupRecord::new(group, system, 0..1);
        invalid_cache.statistics.mass = f32::INFINITY;

        // Act
        let transform_result = invalid_transform.validate(system, 1);
        let cache_result = invalid_cache.validate(system, 1);

        // Assert
        assert_eq!(
            transform_result,
            Err(ParticleStorageError::InvalidGroupRange)
        );
        assert_eq!(cache_result, Err(ParticleStorageError::InvalidGroupRange));
    }

    #[test]
    fn unknown_internal_state_cannot_enter_the_authoritative_table() {
        // Arrange
        let (system, group) = identities();
        let mut record = GroupRecord::new(group, system, 0..1);
        record.internal_flags = InternalGroupFlags(0b0000_0100);

        // Act
        let result = record.validate(system, 1);

        // Assert
        assert_eq!(result, Err(ParticleStorageError::InvalidGroupRange));
    }

    #[test]
    fn storage_exposes_individual_public_flags_in_stable_record_order() {
        // Arrange
        let (system, first_group) = identities();
        let second_group =
            ParticleGroupId::from_identity(Identity::new(system.identity().world(), 2, 0));
        let mut storage = ParticleStorage::new(system.identity().world(), system, 0, 4, 4)
            .expect("test storage contract is valid");
        for group in [first_group, second_group] {
            storage
                .create(ParticleInput {
                    position: Vec2::ZERO,
                    velocity: Vec2::ZERO,
                    flags: ParticleFlags::WATER,
                    maybe_group: Some(group),
                    maybe_color: None,
                    maybe_user_association: None,
                    maybe_expiration_time: None,
                })
                .expect("source-ordered grouped particle fits");
        }
        storage.group_records[0].flags = ParticleGroupFlags::RIGID;
        storage.group_records[1].flags = ParticleGroupFlags::SOLID;

        // Act
        let first_scan = storage.group_flags().collect::<Vec<_>>();
        let second_scan = storage.group_flags().collect::<Vec<_>>();

        // Assert
        assert_eq!(
            first_scan,
            vec![ParticleGroupFlags::RIGID, ParticleGroupFlags::SOLID]
        );
        assert_eq!(second_scan, first_scan);
    }
}
