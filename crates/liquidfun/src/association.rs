//! Application-owned typed associations for world identities.

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

use crate::{
    BodyId, DestroyedId, DestructionRecord, FixtureId, JointId, ParticleGroupId, ParticleId,
    ParticleSystemId,
};

mod sealed {
    pub trait Sealed {}
}

/// A stable typed handle that can key an [`AssociationMap`].
///
/// This trait is sealed: only the object identity types supplied by this crate implement it.
pub trait AssociationId: sealed::Sealed + Copy + Eq + Hash {
    #[doc(hidden)]
    fn from_destroyed(destroyed: DestroyedId) -> Option<Self>;
}

macro_rules! impl_association_id {
    ($id:ty, $variant:ident) => {
        impl sealed::Sealed for $id {}

        impl AssociationId for $id {
            fn from_destroyed(destroyed: DestroyedId) -> Option<Self> {
                let DestroyedId::$variant(id) = destroyed else {
                    return None;
                };
                Some(id)
            }
        }
    };
}

impl_association_id!(BodyId, Body);
impl_association_id!(FixtureId, Fixture);
impl_association_id!(JointId, Joint);
impl_association_id!(ParticleSystemId, ParticleSystem);
impl_association_id!(ParticleGroupId, ParticleGroup);
impl_association_id!(ParticleId, Particle);

/// An application-owned side table keyed by one exact world-object handle type.
///
/// Values are not stored in [`crate::World`] and have no lifetime coupling to it. Destroying an
/// object invalidates its handle but cannot implicitly mutate application state; pass the returned
/// [`DestructionRecord`] values to [`Self::cleanup`] when explicit cleanup is desired.
///
/// Handle kinds cannot be mixed:
///
/// ```compile_fail
/// use liquidfun::collision::{FilterData, Shape};
/// use liquidfun::collision::shape::CircleShape;
/// use liquidfun::math::Vec2;
/// use liquidfun::{AssociationMap, BodyDef, BodyId, FixtureDef, World};
///
/// let mut world = World::new().expect("world key should remain available");
/// let body = world
///     .create_body(&BodyDef::default())
///     .expect("body should fit");
/// let shape = Shape::from(CircleShape::new(Vec2::ZERO, 0.5).expect("valid circle"));
/// let definition = FixtureDef::new(shape, 0.0, 0.2, 0.0, false, FilterData::default())
///     .expect("valid fixture definition");
/// let fixture = world
///     .create_fixture(body, &definition)
///     .expect("fixture should fit");
/// let mut body_names = AssociationMap::<BodyId, _>::new();
/// body_names.insert(fixture, "wrong kind");
/// ```
#[derive(Debug, Clone)]
pub struct AssociationMap<Id, T> {
    values: HashMap<Id, T>,
}

impl<Id: Eq + Hash, T> AssociationMap<Id, T> {
    /// Creates an empty application-owned association table.
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Returns the number of associated identities.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns whether the table has no associations.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Inserts an owned value, returning the previous value for the same complete identity.
    pub fn insert(&mut self, id: Id, value: T) -> Option<T> {
        self.values.insert(id, value)
    }

    /// Looks up an association by its complete typed identity.
    #[must_use]
    pub fn get(&self, id: &Id) -> Option<&T> {
        self.values.get(id)
    }

    /// Removes and returns one association.
    pub fn remove(&mut self, id: &Id) -> Option<T> {
        self.values.remove(id)
    }
}

impl<Id: Eq + Hash, T> Default for AssociationMap<Id, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Id: AssociationId, T> AssociationMap<Id, T> {
    /// Removes the association for one matching destruction record.
    ///
    /// Records for another object kind and records without a matching value are no-ops.
    pub fn cleanup_record(&mut self, record: &DestructionRecord) -> Option<T> {
        let id = Id::from_destroyed(record.destroyed())?;
        self.remove(&id)
    }

    /// Removes associations identified by destruction records in occurrence order.
    ///
    /// The returned values have the same order as their matching records. Input is not deduplicated;
    /// repeated records simply observe that the association was removed by the first occurrence.
    pub fn cleanup<I, R>(&mut self, records: I) -> Vec<T>
    where
        I: IntoIterator<Item = R>,
        R: Borrow<DestructionRecord>,
    {
        records
            .into_iter()
            .filter_map(|record| self.cleanup_record(record.borrow()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collision::FilterData;
    use crate::collision::shape::{CircleShape, Shape};
    use crate::math::Vec2;
    use crate::{BodyDef, FixtureDef, World};

    fn fixture_definition() -> FixtureDef {
        let shape = Shape::from(
            CircleShape::new(Vec2::ZERO, 0.5).expect("test circle should remain valid"),
        );
        FixtureDef::new(shape, 0.0, 0.2, 0.0, false, FilterData::default())
            .expect("test fixture definition should remain valid")
    }

    fn test_world() -> World {
        World::new().expect("test world key should remain available")
    }

    #[test]
    fn body_cascade_cleanup_removes_exact_typed_identities_and_preserves_survivors() {
        // Arrange
        let mut world = test_world();
        let destroyed_body = world
            .create_body(&BodyDef::default())
            .expect("body should fit");
        let surviving_body = world
            .create_body(&BodyDef::default())
            .expect("body should fit");
        let fixture = world
            .create_fixture(destroyed_body, &fixture_definition())
            .expect("fixture should fit");
        let joint = world
            .create_joint(
                crate::RevoluteJointDef::new(destroyed_body, surviving_body)
                    .expect("distinct bodies form a valid joint")
                    .into(),
            )
            .expect("joint should fit");
        let mut bodies = AssociationMap::new();
        let mut fixtures = AssociationMap::new();
        let mut joints = AssociationMap::new();
        bodies.insert(destroyed_body, "destroyed body");
        bodies.insert(surviving_body, "surviving body");
        fixtures.insert(fixture, "fixture");
        joints.insert(joint, "joint");
        let records = world
            .destroy_body(destroyed_body)
            .expect("body should be live");

        // Act
        let removed_joints = joints.cleanup(&records);
        let removed_fixtures = fixtures.cleanup(&records);
        let removed_bodies = bodies.cleanup(&records);

        // Assert
        assert_eq!(removed_joints, vec!["joint"]);
        assert_eq!(removed_fixtures, vec!["fixture"]);
        assert_eq!(removed_bodies, vec!["destroyed body"]);
        assert_eq!(bodies.get(&surviving_body), Some(&"surviving body"));
        assert_eq!(bodies.len(), 1);
        assert!(fixtures.is_empty());
        assert!(joints.is_empty());
    }

    #[test]
    fn particle_system_cleanup_preserves_record_order_and_unrelated_system() {
        // Arrange
        let mut world = test_world();
        let destroyed_system = world
            .create_particle_system()
            .expect("particle system should fit");
        let surviving_system = world
            .create_particle_system()
            .expect("particle system should fit");
        let first = world
            .create_particle(destroyed_system, None)
            .expect("particle should fit");
        let second = world
            .create_particle(destroyed_system, None)
            .expect("particle should fit");
        let survivor = world
            .create_particle(surviving_system, None)
            .expect("particle should fit");
        let mut particles = AssociationMap::new();
        particles.insert(first, "first");
        particles.insert(second, "second");
        particles.insert(survivor, "survivor");
        let records = world
            .destroy_particle_system(destroyed_system)
            .expect("system should be live");

        // Act
        let removed = particles.cleanup(records.iter());

        // Assert
        assert_eq!(removed, vec!["first", "second"]);
        assert_eq!(particles.get(&survivor), Some(&"survivor"));
        assert_eq!(particles.len(), 1);
    }

    #[test]
    fn cleanup_record_for_other_kind_or_missing_identity_is_a_no_op() {
        // Arrange
        let mut world = test_world();
        let body = world
            .create_body(&BodyDef::default())
            .expect("body should fit");
        let fixture = world
            .create_fixture(body, &fixture_definition())
            .expect("fixture should fit");
        let fixture_record = world
            .destroy_fixture(fixture)
            .expect("fixture should be live");
        let mut bodies = AssociationMap::new();
        bodies.insert(body, "body");

        // Act
        let first = bodies.cleanup_record(&fixture_record);
        let second = bodies.cleanup_record(&fixture_record);

        // Assert
        assert_eq!(first, None);
        assert_eq!(second, None);
        assert_eq!(bodies.get(&body), Some(&"body"));
    }
}
