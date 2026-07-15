use crate::identity::{BodyId, FixtureId, ParticleGroupId};
use crate::math::Vec2;
use crate::particle::{ParticleColor, ParticleFlags};

use super::{ParticleIndex, ParticleStorageError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UserAssociationKey(u64);

impl UserAssociationKey {
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct GroupRange {
    pub(super) maybe_group: Option<ParticleGroupId>,
    pub(super) start: ParticleIndex,
    pub(super) end: ParticleIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ParticleProxy {
    pub(super) index: ParticleIndex,
    pub(super) tag: u32,
}

impl ParticleProxy {
    pub(super) const fn new(index: ParticleIndex) -> Self {
        Self { index, tag: 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::particle) struct ParticleContact {
    pub(in crate::particle) indices: [ParticleIndex; 2],
    pub(in crate::particle) flags: ParticleFlags,
    pub(in crate::particle) weight: f32,
    pub(in crate::particle) normal: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::particle) struct ParticleBodyContact {
    pub(in crate::particle) index: ParticleIndex,
    pub(in crate::particle) body: BodyId,
    pub(in crate::particle) fixture: FixtureId,
    pub(in crate::particle) weight: f32,
    pub(in crate::particle) normal: Vec2,
    pub(in crate::particle) mass: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::particle) struct ParticlePair {
    pub(in crate::particle) indices: [ParticleIndex; 2],
    pub(in crate::particle) flags: ParticleFlags,
    pub(in crate::particle) strength: f32,
    pub(in crate::particle) distance: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::particle) struct ParticleTriad {
    pub(in crate::particle) indices: [ParticleIndex; 3],
    pub(in crate::particle) flags: ParticleFlags,
    pub(in crate::particle) strength: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StuckLanes {
    pub(super) last_body_contact_steps: Vec<u32>,
    pub(super) body_contact_counts: Vec<u32>,
    pub(super) consecutive_contact_steps: Vec<u32>,
    pub(super) candidates: Vec<ParticleIndex>,
}

impl StuckLanes {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            last_body_contact_steps: Vec::with_capacity(capacity),
            body_contact_counts: Vec::with_capacity(capacity),
            consecutive_contact_steps: Vec::with_capacity(capacity),
            candidates: Vec::new(),
        }
    }

    fn validate_empty(&self, declared_capacity: usize) -> bool {
        self.last_body_contact_steps.is_empty()
            && self.body_contact_counts.is_empty()
            && self.consecutive_contact_steps.is_empty()
            && self.candidates.is_empty()
            && self.last_body_contact_steps.capacity() >= declared_capacity
            && self.body_contact_counts.capacity() >= declared_capacity
            && self.consecutive_contact_steps.capacity() >= declared_capacity
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct OwnedLaneBundle {
    pub(crate) positions: Vec<Vec2>,
    pub(crate) velocities: Vec<Vec2>,
    pub(crate) flags: Vec<ParticleFlags>,
    pub(crate) groups: Vec<Option<ParticleGroupId>>,
    pub(crate) weights: Vec<f32>,
    pub(crate) forces: Vec<Vec2>,
    pub(crate) maybe_colors: Option<Vec<ParticleColor>>,
    pub(super) maybe_user_associations: Option<Vec<Option<UserAssociationKey>>>,
    pub(super) maybe_stuck: Option<StuckLanes>,
    pub(crate) maybe_expiration_times: Option<Vec<i32>>,
    pub(super) maybe_expiration_order: Option<Vec<ParticleIndex>>,
}

impl OwnedLaneBundle {
    pub(crate) fn with_capacity(capacity: usize, optional: bool) -> Self {
        Self {
            positions: Vec::with_capacity(capacity),
            velocities: Vec::with_capacity(capacity),
            flags: Vec::with_capacity(capacity),
            groups: Vec::with_capacity(capacity),
            weights: Vec::with_capacity(capacity),
            forces: Vec::with_capacity(capacity),
            maybe_colors: optional.then(|| Vec::with_capacity(capacity)),
            maybe_user_associations: optional.then(|| Vec::with_capacity(capacity)),
            maybe_stuck: optional.then(|| StuckLanes::with_capacity(capacity)),
            maybe_expiration_times: optional.then(|| Vec::with_capacity(capacity)),
            maybe_expiration_order: optional.then(|| Vec::with_capacity(capacity)),
        }
    }

    pub(super) fn validate_empty(
        &self,
        declared_capacity: usize,
    ) -> Result<(), ParticleStorageError> {
        let required_valid = self.positions.is_empty()
            && self.velocities.is_empty()
            && self.flags.is_empty()
            && self.groups.is_empty()
            && self.weights.is_empty()
            && self.forces.is_empty()
            && self.positions.capacity() >= declared_capacity
            && self.velocities.capacity() >= declared_capacity
            && self.flags.capacity() >= declared_capacity
            && self.groups.capacity() >= declared_capacity
            && self.weights.capacity() >= declared_capacity
            && self.forces.capacity() >= declared_capacity;
        let optional_valid = optional_lane_is_empty(self.maybe_colors.as_ref(), declared_capacity)
            && optional_lane_is_empty(self.maybe_user_associations.as_ref(), declared_capacity)
            && optional_lane_is_empty(self.maybe_expiration_times.as_ref(), declared_capacity)
            && optional_lane_is_empty(self.maybe_expiration_order.as_ref(), declared_capacity)
            && self
                .maybe_stuck
                .as_ref()
                .is_none_or(|lanes| lanes.validate_empty(declared_capacity));
        if !required_valid || !optional_valid {
            return Err(ParticleStorageError::InvalidLaneBundle);
        }
        Ok(())
    }
}

fn optional_lane_is_empty<T>(lane: Option<&Vec<T>>, declared_capacity: usize) -> bool {
    lane.is_none_or(|values| values.is_empty() && values.capacity() >= declared_capacity)
}

#[cfg(test)]
mod tests {
    use crate::identity::{HandleIdentity, Identity, ParticleGroupId};
    use crate::math::Vec2;
    use crate::particle::{ParticleColor, ParticleFlags};

    use super::super::*;

    fn storage() -> ParticleStorage {
        let world = WorldKey::fresh().expect("test world key remains available");
        let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
        ParticleStorage::new(world, system, 0, 4, 4).expect("test storage contract is valid")
    }

    fn input(value: f32) -> ParticleInput {
        ParticleInput {
            position: Vec2::new(value, -value),
            velocity: Vec2::new(value + 1.0, value + 2.0),
            flags: ParticleFlags::WALL | ParticleFlags::from_bits_retain(1 << 31),
            maybe_group: None,
            maybe_color: None,
            maybe_user_association: None,
            maybe_expiration_time: None,
        }
    }

    #[test]
    fn required_production_lanes_initialize_in_one_row_commit() {
        // Arrange
        let mut storage = storage();

        // Act
        let id = storage.create(input(3.0)).expect("particle fits");

        // Assert
        assert_eq!(storage.input(id), Ok(input(3.0)));
        assert_eq!(storage.positions, vec![Vec2::new(3.0, -3.0)]);
        assert_eq!(storage.velocities, vec![Vec2::new(4.0, 5.0)]);
        assert_eq!(storage.weights, vec![0.0]);
        assert_eq!(storage.forces, vec![Vec2::ZERO]);
        assert_eq!(storage.proxies, vec![ParticleProxy::new(ParticleIndex(0))]);
        assert!(storage.particle_contacts.is_empty());
        assert!(storage.body_contacts.is_empty());
        assert!(storage.pairs.is_empty());
        assert!(storage.triads.is_empty());
        assert!(storage.maybe_colors.is_none());
        assert!(storage.maybe_user_associations.is_none());
        assert!(storage.maybe_stuck.is_none());
        assert!(storage.maybe_expiration_times.is_none());
        assert!(storage.maybe_expiration_order.is_none());
    }

    #[test]
    fn source_optional_lanes_allocate_lazily_and_backfill_defaults() {
        // Arrange
        let mut storage = storage();
        storage.create(input(1.0)).expect("first particle fits");
        let mut optional = input(2.0);
        optional.maybe_color = Some(ParticleColor::new(1, 2, 3, 4));
        optional.maybe_user_association = Some(UserAssociationKey::new(9));
        optional.maybe_expiration_time = Some(17);

        // Act
        storage.create(optional).expect("optional particle fits");

        // Assert
        assert_eq!(
            storage.maybe_colors,
            Some(vec![ParticleColor::ZERO, ParticleColor::new(1, 2, 3, 4)])
        );
        assert_eq!(
            storage.maybe_user_associations,
            Some(vec![None, Some(UserAssociationKey::new(9))])
        );
        assert_eq!(storage.maybe_expiration_times, Some(vec![0, 17]));
        assert_eq!(
            storage.maybe_expiration_order,
            Some(vec![ParticleIndex(0), ParticleIndex(1)])
        );
    }

    #[test]
    fn rejected_creation_leaves_identities_and_every_lane_unchanged() {
        // Arrange
        let mut storage = storage();
        storage.create(input(1.0)).expect("first particle fits");
        let before = storage.clone();
        let mut invalid = input(2.0);
        invalid.maybe_group = Some(ParticleGroupId::from_identity(Identity::new(
            storage.world,
            77,
            0,
        )));
        storage
            .create(invalid)
            .expect("first grouped particle fits");
        storage.create(input(3.0)).expect("ungrouped particle fits");
        let before_rejection = storage.clone();
        let result = storage.create(invalid);

        // Act / Assert
        assert!(storage != before);
        assert_eq!(result, Err(ParticleStorageError::InvalidGroupRange));
        assert!(storage == before_rejection);
    }
}
