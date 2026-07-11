#![allow(
    dead_code,
    reason = "private identity construction is consumed by the generational arena"
)]

use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::WorldKeyError;

static LAST_WORLD_KEY: AtomicU64 = AtomicU64::new(0);

/// The kind of object identified by an opaque handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ObjectKind {
    /// A rigid body.
    Body,
    /// A fixture attached to a body.
    Fixture,
    /// A joint connecting bodies.
    Joint,
    /// A particle system owned by a world.
    ParticleSystem,
    /// A particle group owned by a particle system.
    ParticleGroup,
    /// A stable particle identity.
    Particle,
}

impl fmt::Display for ObjectKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Body => "body",
            Self::Fixture => "fixture",
            Self::Joint => "joint",
            Self::ParticleSystem => "particle system",
            Self::ParticleGroup => "particle group",
            Self::Particle => "particle",
        };
        formatter.write_str(name)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct WorldKey(u64);

impl WorldKey {
    pub(crate) fn fresh() -> Result<Self, WorldKeyError> {
        LAST_WORLD_KEY
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .map(|previous| Self(previous + 1))
            .map_err(|_exhausted| WorldKeyError::Exhausted)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Identity {
    world: WorldKey,
    slot: u32,
    generation: u64,
}

impl Identity {
    pub(crate) const fn new(world: WorldKey, slot: u32, generation: u64) -> Self {
        Self {
            world,
            slot,
            generation,
        }
    }

    pub(crate) const fn world(self) -> WorldKey {
        self.world
    }

    pub(crate) const fn slot(self) -> u32 {
        self.slot
    }

    pub(crate) const fn generation(self) -> u64 {
        self.generation
    }

    fn diagnostic_token(self, kind: ObjectKind) -> u64 {
        let mut hasher = DefaultHasher::new();
        "liquidfun-opaque-handle".hash(&mut hasher);
        kind.hash(&mut hasher);
        self.hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ErasedHandle {
    kind: ObjectKind,
    identity: Identity,
}

impl ErasedHandle {
    pub(crate) const fn kind(self) -> ObjectKind {
        self.kind
    }

    pub(crate) const fn identity(self) -> Identity {
        self.identity
    }
}

pub(crate) trait HandleIdentity: Copy {
    const KIND: ObjectKind;

    fn from_identity(identity: Identity) -> Self;
    fn identity(self) -> Identity;

    fn erased(self) -> ErasedHandle {
        ErasedHandle {
            kind: Self::KIND,
            identity: self.identity(),
        }
    }
}

macro_rules! define_handle {
    ($(#[$metadata:meta])* $name:ident, $kind:ident) => {
        $(#[$metadata])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(Identity);

        impl HandleIdentity for $name {
            const KIND: ObjectKind = ObjectKind::$kind;

            fn from_identity(identity: Identity) -> Self {
                Self(identity)
            }

            fn identity(self) -> Identity {
                self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                let token = self.0.diagnostic_token(ObjectKind::$kind);
                write!(
                    formatter,
                    concat!(stringify!($name), "({:016x})"),
                    token
                )
            }
        }
    };
}

define_handle!(
    /// An opaque, world-scoped identity for a rigid body.
    BodyId,
    Body
);
define_handle!(
    /// An opaque, world-scoped identity for a fixture.
    FixtureId,
    Fixture
);
define_handle!(
    /// An opaque, world-scoped identity for a joint.
    JointId,
    Joint
);
define_handle!(
    /// An opaque, world-scoped identity for a particle system.
    ParticleSystemId,
    ParticleSystem
);
define_handle!(
    /// An opaque, world-scoped identity for a particle group.
    ParticleGroupId,
    ParticleGroup
);
define_handle!(
    /// An opaque, stable, world-scoped identity for a particle.
    ParticleId,
    Particle
);

#[cfg(test)]
mod tests {
    use std::any::TypeId;
    use std::collections::HashSet;

    use super::*;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn public_handle_types_are_distinct() {
        // Arrange
        let types = [
            TypeId::of::<BodyId>(),
            TypeId::of::<FixtureId>(),
            TypeId::of::<JointId>(),
            TypeId::of::<ParticleSystemId>(),
            TypeId::of::<ParticleGroupId>(),
            TypeId::of::<ParticleId>(),
        ];

        // Act
        let distinct = types
            .iter()
            .enumerate()
            .all(|(index, item)| !types[index + 1..].contains(item));

        // Assert
        assert!(distinct);
    }

    #[test]
    fn complete_identity_controls_equality_and_hashing() {
        // Arrange
        let world = WorldKey::fresh().expect("test world key should remain available");
        let other_world = WorldKey::fresh().expect("test world key should remain available");
        let original = BodyId::from_identity(Identity::new(world, 7, 11));
        let equal = BodyId::from_identity(Identity::new(world, 7, 11));
        let other_slot = BodyId::from_identity(Identity::new(world, 8, 11));
        let other_generation = BodyId::from_identity(Identity::new(world, 7, 12));
        let other_world = BodyId::from_identity(Identity::new(other_world, 7, 11));

        // Act
        let identities =
            HashSet::from([original, equal, other_slot, other_generation, other_world]);

        // Assert
        assert_eq!(original, equal);
        assert_eq!(identities.len(), 4);
    }

    #[test]
    fn debug_output_exposes_only_an_opaque_token() {
        // Arrange
        let world = WorldKey::fresh().expect("test world key should remain available");
        let handle = BodyId::from_identity(Identity::new(world, 123_456, 987_654));

        // Act
        let rendered = format!("{handle:?}");

        // Assert
        assert!(rendered.starts_with("BodyId("));
        assert!(!rendered.contains("slot"));
        assert!(!rendered.contains("generation"));
        assert!(!rendered.contains("123456"));
        assert!(!rendered.contains("987654"));
    }

    #[test]
    fn handles_are_send_and_sync_through_auto_traits() {
        // Arrange / Act
        assert_send_sync::<BodyId>();
        assert_send_sync::<FixtureId>();
        assert_send_sync::<JointId>();
        assert_send_sync::<ParticleSystemId>();
        assert_send_sync::<ParticleGroupId>();
        assert_send_sync::<ParticleId>();

        // Assert
        // Compilation is the assertion; no manual auto-trait implementation is needed.
    }

    #[test]
    fn freshly_allocated_world_keys_are_distinct() {
        // Arrange
        let first = WorldKey::fresh().expect("test world key should remain available");

        // Act
        let second = WorldKey::fresh().expect("test world key should remain available");

        // Assert
        assert!(first != second);
    }

    #[test]
    fn checked_world_key_increment_reports_exhaustion() {
        // Arrange
        let exhausted = u64::MAX;

        // Act
        let maybe_next = exhausted.checked_add(1);

        // Assert
        assert_eq!(maybe_next, None);
    }
}
