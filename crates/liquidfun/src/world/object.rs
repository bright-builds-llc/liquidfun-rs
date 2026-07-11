use std::error::Error;
use std::fmt;

use crate::arena::Arena;
use crate::identity::{HandleIdentity, WorldKey};
use crate::{
    ArenaInsertError, BodyId, FixtureId, HandleError, JointId, ObjectKind, ParticleGroupId,
    ParticleId, ParticleSystemId, WorldKeyError,
};

use super::step::StepState;

#[derive(Debug)]
struct Body {
    diagnostic_id: u64,
    fixtures: Vec<FixtureId>,
    joints: Vec<JointId>,
}

#[derive(Debug)]
struct Fixture {
    diagnostic_id: u64,
    body: BodyId,
}

#[derive(Debug)]
struct Joint {
    diagnostic_id: u64,
    bodies: [BodyId; 2],
}

#[derive(Debug)]
struct ParticleSystem {
    diagnostic_id: u64,
    groups: Vec<ParticleGroupId>,
    particles: Vec<ParticleId>,
}

#[derive(Debug)]
struct ParticleGroup {
    diagnostic_id: u64,
    system: ParticleSystemId,
    particles: Vec<ParticleId>,
}

#[derive(Debug)]
struct Particle {
    diagnostic_id: u64,
    system: ParticleSystemId,
    maybe_group: Option<ParticleGroupId>,
}

/// A failure while creating a world-owned object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CreateObjectError {
    /// A referenced owner or endpoint does not belong to this world or is no longer live.
    InvalidHandle(HandleError),
    /// The arena for the new object cannot accept another entry.
    Arena(ArenaInsertError),
}

impl fmt::Display for CreateObjectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHandle(error) => write!(formatter, "invalid related handle: {error}"),
            Self::Arena(error) => write!(formatter, "could not store object: {error}"),
        }
    }
}

impl Error for CreateObjectError {}

impl From<HandleError> for CreateObjectError {
    fn from(error: HandleError) -> Self {
        Self::InvalidHandle(error)
    }
}

impl From<ArenaInsertError> for CreateObjectError {
    fn from(error: ArenaInsertError) -> Self {
        Self::Arena(error)
    }
}

/// The typed identity invalidated by a destruction record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DestroyedId {
    /// A body identity.
    Body(BodyId),
    /// A fixture identity.
    Fixture(FixtureId),
    /// A joint identity.
    Joint(JointId),
    /// A particle-system identity.
    ParticleSystem(ParticleSystemId),
    /// A particle-group identity.
    ParticleGroup(ParticleGroupId),
    /// A particle identity.
    Particle(ParticleId),
}

impl DestroyedId {
    /// Returns the kind of object that was invalidated.
    #[must_use]
    pub const fn kind(self) -> ObjectKind {
        match self {
            Self::Body(_) => ObjectKind::Body,
            Self::Fixture(_) => ObjectKind::Fixture,
            Self::Joint(_) => ObjectKind::Joint,
            Self::ParticleSystem(_) => ObjectKind::ParticleSystem,
            Self::ParticleGroup(_) => ObjectKind::ParticleGroup,
            Self::Particle(_) => ObjectKind::Particle,
        }
    }
}

/// Why an object was destroyed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DestructionCause {
    /// The object was the root passed to a public destruction method.
    Explicit,
    /// A body destruction invalidated an attached object.
    BodyCascade {
        /// Body whose destruction caused this invalidation.
        body: BodyId,
    },
    /// A particle-system destruction invalidated a contained object.
    ParticleSystemCascade {
        /// Particle system whose destruction caused this invalidation.
        system: ParticleSystemId,
    },
}

/// Owned semantic state retained after an object has been invalidated.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ObjectSnapshot {
    /// Body adjacency at the start of its destruction transaction.
    Body {
        /// Attached fixtures in occurrence order.
        fixtures: Vec<FixtureId>,
        /// Attached joints in occurrence order.
        joints: Vec<JointId>,
    },
    /// The body that owned a fixture.
    Fixture {
        /// Owning body at destruction time.
        body: BodyId,
    },
    /// The two body endpoints of a joint.
    Joint {
        /// Joint endpoints at destruction time.
        bodies: [BodyId; 2],
    },
    /// Particle-system membership at the start of its destruction transaction.
    ParticleSystem {
        /// Groups in occurrence order.
        groups: Vec<ParticleGroupId>,
        /// Particles in occurrence order.
        particles: Vec<ParticleId>,
    },
    /// The system and particles associated with a group.
    ParticleGroup {
        /// Owning particle system.
        system: ParticleSystemId,
        /// Member particles in occurrence order.
        particles: Vec<ParticleId>,
    },
    /// The system and optional group associated with a particle.
    Particle {
        /// Owning particle system.
        system: ParticleSystemId,
        /// Group membership at destruction time.
        maybe_group: Option<ParticleGroupId>,
    },
}

/// Owned evidence describing one invalidated world object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestructionRecord {
    destroyed: DestroyedId,
    diagnostic_id: u64,
    cause: DestructionCause,
    snapshot: ObjectSnapshot,
}

impl DestructionRecord {
    /// Returns the exact typed identity that is no longer valid.
    #[must_use]
    pub const fn destroyed(&self) -> DestroyedId {
        self.destroyed
    }

    /// Returns the object's stable, world-local semantic diagnostic identity.
    #[must_use]
    pub const fn diagnostic_id(&self) -> u64 {
        self.diagnostic_id
    }

    /// Returns why the object was invalidated.
    #[must_use]
    pub const fn cause(&self) -> DestructionCause {
        self.cause
    }

    /// Returns semantic state captured before invalidation.
    #[must_use]
    pub const fn snapshot(&self) -> &ObjectSnapshot {
        &self.snapshot
    }
}

/// A minimal owner of typed object arenas and their adjacency.
///
/// This type intentionally contains no stepping, collision, or solver behavior. Destruction is
/// transactional with respect to public input: the root handle is validated before any mutation.
/// Body cascades emit joints, then fixtures, then the body. Particle-system cascades emit groups,
/// then particles, then the system. Body fixture and joint categories use the pinned upstream
/// newest-first list order; particle-system categories preserve creation/occurrence order.
pub struct World {
    bodies: Arena<Body, BodyId>,
    fixtures: Arena<Fixture, FixtureId>,
    joints: Arena<Joint, JointId>,
    particle_systems: Arena<ParticleSystem, ParticleSystemId>,
    particle_groups: Arena<ParticleGroup, ParticleGroupId>,
    particles: Arena<Particle, ParticleId>,
    next_diagnostic_id: u64,
    pub(super) step_state: StepState,
}

impl World {
    /// Creates an empty world with a process-unique identity scope.
    ///
    /// # Errors
    ///
    /// Returns [`WorldKeyError::Exhausted`] if process-unique world identities are exhausted.
    pub fn new() -> Result<Self, WorldKeyError> {
        let world = WorldKey::fresh()?;
        Ok(Self {
            bodies: Arena::new(world, usize::MAX),
            fixtures: Arena::new(world, usize::MAX),
            joints: Arena::new(world, usize::MAX),
            particle_systems: Arena::new(world, usize::MAX),
            particle_groups: Arena::new(world, usize::MAX),
            particles: Arena::new(world, usize::MAX),
            next_diagnostic_id: 1,
            step_state: StepState::new(),
        })
    }

    pub(super) fn validate_fixture(&self, fixture: FixtureId) -> Result<(), HandleError> {
        self.fixtures.get(fixture).map(|_fixture| ())
    }

    fn allocate_diagnostic_id(&mut self) -> u64 {
        let id = self.next_diagnostic_id;
        self.next_diagnostic_id = self.next_diagnostic_id.saturating_add(1);
        id
    }

    /// Creates a body.
    ///
    /// # Errors
    ///
    /// Returns an arena error if body storage is exhausted.
    pub fn create_body(&mut self) -> Result<BodyId, ArenaInsertError> {
        self.ensure_not_poisoned_for_insert()?;
        let diagnostic_id = self.allocate_diagnostic_id();
        self.bodies.insert(Body {
            diagnostic_id,
            fixtures: Vec::new(),
            joints: Vec::new(),
        })
    }

    /// Creates a fixture attached to `body`.
    ///
    /// # Errors
    ///
    /// Returns an error if `body` is invalid or fixture storage is exhausted.
    pub fn create_fixture(&mut self, body: BodyId) -> Result<FixtureId, CreateObjectError> {
        self.ensure_not_poisoned_for_handle()?;
        self.bodies.get(body)?;
        let diagnostic_id = self.allocate_diagnostic_id();
        let fixture = self.fixtures.insert(Fixture {
            diagnostic_id,
            body,
        })?;
        self.body_mut_after_validation(body)
            .fixtures
            .insert(0, fixture);
        Ok(fixture)
    }

    /// Creates a joint between two live bodies.
    ///
    /// # Errors
    ///
    /// Returns an error if either body is invalid or joint storage is exhausted.
    pub fn create_joint(
        &mut self,
        first: BodyId,
        second: BodyId,
    ) -> Result<JointId, CreateObjectError> {
        self.ensure_not_poisoned_for_handle()?;
        self.bodies.get(first)?;
        self.bodies.get(second)?;
        let diagnostic_id = self.allocate_diagnostic_id();
        let joint = self.joints.insert(Joint {
            diagnostic_id,
            bodies: [first, second],
        })?;
        self.body_mut_after_validation(first)
            .joints
            .insert(0, joint);
        if second != first {
            self.body_mut_after_validation(second)
                .joints
                .insert(0, joint);
        }
        Ok(joint)
    }

    /// Creates a particle system.
    ///
    /// # Errors
    ///
    /// Returns an arena error if particle-system storage is exhausted.
    pub fn create_particle_system(&mut self) -> Result<ParticleSystemId, ArenaInsertError> {
        self.ensure_not_poisoned_for_insert()?;
        let diagnostic_id = self.allocate_diagnostic_id();
        self.particle_systems.insert(ParticleSystem {
            diagnostic_id,
            groups: Vec::new(),
            particles: Vec::new(),
        })
    }

    /// Creates a particle group in `system`.
    ///
    /// # Errors
    ///
    /// Returns an error if `system` is invalid or particle-group storage is exhausted.
    pub fn create_particle_group(
        &mut self,
        system: ParticleSystemId,
    ) -> Result<ParticleGroupId, CreateObjectError> {
        self.ensure_not_poisoned_for_handle()?;
        self.particle_systems.get(system)?;
        let diagnostic_id = self.allocate_diagnostic_id();
        let group = self.particle_groups.insert(ParticleGroup {
            diagnostic_id,
            system,
            particles: Vec::new(),
        })?;
        self.system_mut_after_validation(system).groups.push(group);
        Ok(group)
    }

    /// Creates a stable particle identity in `system` and optionally associates it with `group`.
    ///
    /// # Errors
    ///
    /// Returns an error if an owner is invalid, the group belongs to another system, or particle
    /// storage is exhausted.
    pub fn create_particle(
        &mut self,
        system: ParticleSystemId,
        maybe_group: Option<ParticleGroupId>,
    ) -> Result<ParticleId, CreateObjectError> {
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
        let diagnostic_id = self.allocate_diagnostic_id();
        let particle = self.particles.insert_particle(
            Particle {
                diagnostic_id,
                system,
                maybe_group,
            },
            system.identity(),
        )?;
        self.system_mut_after_validation(system)
            .particles
            .push(particle);
        if let Some(group) = maybe_group {
            self.group_mut_after_validation(group)
                .particles
                .push(particle);
        }
        Ok(particle)
    }

    /// Returns whether a body handle resolves in this world.
    #[must_use]
    pub fn contains_body(&self, body: BodyId) -> bool {
        self.bodies.get(body).is_ok()
    }

    /// Returns whether a fixture handle resolves in this world.
    #[must_use]
    pub fn contains_fixture(&self, fixture: FixtureId) -> bool {
        self.fixtures.get(fixture).is_ok()
    }

    /// Returns whether a joint handle resolves in this world.
    #[must_use]
    pub fn contains_joint(&self, joint: JointId) -> bool {
        self.joints.get(joint).is_ok()
    }

    /// Returns whether a particle-system handle resolves in this world.
    #[must_use]
    pub fn contains_particle_system(&self, system: ParticleSystemId) -> bool {
        self.particle_systems.get(system).is_ok()
    }

    /// Returns whether a particle-group handle resolves in this world.
    #[must_use]
    pub fn contains_particle_group(&self, group: ParticleGroupId) -> bool {
        self.particle_groups.get(group).is_ok()
    }

    /// Returns whether a particle handle resolves in this world.
    #[must_use]
    pub fn contains_particle(&self, particle: ParticleId) -> bool {
        self.particles.get(particle).is_ok()
    }

    /// Destroys a body and all attached joints and fixtures.
    ///
    /// # Errors
    ///
    /// Returns a handle error without mutation when `body` is foreign, stale, or destroyed.
    pub fn destroy_body(&mut self, body: BodyId) -> Result<Vec<DestructionRecord>, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let root = self.bodies.get(body)?;
        let joints = root.joints.clone();
        let fixtures = root.fixtures.clone();
        let root_snapshot = ObjectSnapshot::Body {
            fixtures: fixtures.clone(),
            joints: joints.clone(),
        };
        let mut records = Vec::with_capacity(joints.len() + fixtures.len() + 1);

        for joint in joints {
            records.push(self.remove_joint(joint, DestructionCause::BodyCascade { body }));
        }
        for fixture in fixtures {
            records.push(self.remove_fixture(fixture, DestructionCause::BodyCascade { body }));
        }
        records.push(self.remove_body(body, DestructionCause::Explicit, root_snapshot));
        Ok(records)
    }

    /// Destroys one fixture after validating it before mutation.
    ///
    /// # Errors
    ///
    /// Returns a handle error without mutation when `fixture` is foreign, stale, or destroyed.
    pub fn destroy_fixture(
        &mut self,
        fixture: FixtureId,
    ) -> Result<DestructionRecord, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        self.fixtures.get(fixture)?;
        Ok(self.remove_fixture(fixture, DestructionCause::Explicit))
    }

    /// Destroys one joint after validating it before mutation.
    ///
    /// # Errors
    ///
    /// Returns a handle error without mutation when `joint` is foreign, stale, or destroyed.
    pub fn destroy_joint(&mut self, joint: JointId) -> Result<DestructionRecord, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        self.joints.get(joint)?;
        Ok(self.remove_joint(joint, DestructionCause::Explicit))
    }

    /// Destroys a particle system and all its groups and particles.
    ///
    /// # Errors
    ///
    /// Returns a handle error without mutation when `system` is foreign, stale, or destroyed.
    pub fn destroy_particle_system(
        &mut self,
        system: ParticleSystemId,
    ) -> Result<Vec<DestructionRecord>, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let root = self.particle_systems.get(system)?;
        let groups = root.groups.clone();
        let particles = root.particles.clone();
        let mut records = Vec::with_capacity(groups.len() + particles.len() + 1);

        for group in groups {
            records.push(
                self.remove_particle_group(
                    group,
                    DestructionCause::ParticleSystemCascade { system },
                ),
            );
        }
        for particle in particles {
            records.push(
                self.remove_particle(particle, DestructionCause::ParticleSystemCascade { system }),
            );
        }
        records.push(self.remove_particle_system(system, DestructionCause::Explicit));
        Ok(records)
    }

    /// Destroys a particle group without destroying its particles.
    ///
    /// # Errors
    ///
    /// Returns a handle error without mutation when `group` is foreign, stale, or destroyed.
    pub fn destroy_particle_group(
        &mut self,
        group: ParticleGroupId,
    ) -> Result<DestructionRecord, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        self.particle_groups.get(group)?;
        Ok(self.remove_particle_group(group, DestructionCause::Explicit))
    }

    /// Destroys one stable particle identity.
    ///
    /// # Errors
    ///
    /// Returns a handle error without mutation when `particle` is foreign, stale, or destroyed.
    pub fn destroy_particle(
        &mut self,
        particle: ParticleId,
    ) -> Result<DestructionRecord, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        self.particles.get(particle)?;
        Ok(self.remove_particle(particle, DestructionCause::Explicit))
    }

    fn ensure_not_poisoned_for_handle(&self) -> Result<(), HandleError> {
        if self.step_state.is_poisoned() {
            return Err(HandleError::WorldPoisoned);
        }
        Ok(())
    }

    fn ensure_not_poisoned_for_insert(&self) -> Result<(), ArenaInsertError> {
        if self.step_state.is_poisoned() {
            return Err(ArenaInsertError::WorldPoisoned);
        }
        Ok(())
    }

    fn remove_body(
        &mut self,
        body: BodyId,
        cause: DestructionCause,
        snapshot: ObjectSnapshot,
    ) -> DestructionRecord {
        let removed = self
            .bodies
            .remove(body)
            .expect("validated destruction root and adjacency remain live");
        DestructionRecord {
            destroyed: DestroyedId::Body(body),
            diagnostic_id: removed.diagnostic_id,
            cause,
            snapshot,
        }
    }

    fn remove_fixture(&mut self, fixture: FixtureId, cause: DestructionCause) -> DestructionRecord {
        let removed = self
            .fixtures
            .remove(fixture)
            .expect("validated fixture adjacency remains live");
        remove_occurrence(
            &mut self.body_mut_after_validation(removed.body).fixtures,
            &fixture,
        );
        DestructionRecord {
            destroyed: DestroyedId::Fixture(fixture),
            diagnostic_id: removed.diagnostic_id,
            cause,
            snapshot: ObjectSnapshot::Fixture { body: removed.body },
        }
    }

    fn remove_joint(&mut self, joint: JointId, cause: DestructionCause) -> DestructionRecord {
        let removed = self
            .joints
            .remove(joint)
            .expect("validated joint adjacency remains live");
        remove_occurrence(
            &mut self.body_mut_after_validation(removed.bodies[0]).joints,
            &joint,
        );
        if removed.bodies[1] != removed.bodies[0] {
            remove_occurrence(
                &mut self.body_mut_after_validation(removed.bodies[1]).joints,
                &joint,
            );
        }
        DestructionRecord {
            destroyed: DestroyedId::Joint(joint),
            diagnostic_id: removed.diagnostic_id,
            cause,
            snapshot: ObjectSnapshot::Joint {
                bodies: removed.bodies,
            },
        }
    }

    fn remove_particle_system(
        &mut self,
        system: ParticleSystemId,
        cause: DestructionCause,
    ) -> DestructionRecord {
        let removed = self
            .particle_systems
            .remove(system)
            .expect("validated destruction root and adjacency remain live");
        DestructionRecord {
            destroyed: DestroyedId::ParticleSystem(system),
            diagnostic_id: removed.diagnostic_id,
            cause,
            snapshot: ObjectSnapshot::ParticleSystem {
                groups: removed.groups,
                particles: removed.particles,
            },
        }
    }

    fn remove_particle_group(
        &mut self,
        group: ParticleGroupId,
        cause: DestructionCause,
    ) -> DestructionRecord {
        let removed = self
            .particle_groups
            .remove(group)
            .expect("validated particle-group adjacency remains live");
        remove_occurrence(
            &mut self.system_mut_after_validation(removed.system).groups,
            &group,
        );
        for particle in &removed.particles {
            let particle_record = self
                .particles
                .get(*particle)
                .expect("group membership contains live particles");
            debug_assert_eq!(particle_record.maybe_group, Some(group));
        }
        for particle in &removed.particles {
            self.particle_mut_after_validation(*particle).maybe_group = None;
        }
        DestructionRecord {
            destroyed: DestroyedId::ParticleGroup(group),
            diagnostic_id: removed.diagnostic_id,
            cause,
            snapshot: ObjectSnapshot::ParticleGroup {
                system: removed.system,
                particles: removed.particles,
            },
        }
    }

    fn remove_particle(
        &mut self,
        particle: ParticleId,
        cause: DestructionCause,
    ) -> DestructionRecord {
        let removed = self
            .particles
            .remove(particle)
            .expect("validated particle adjacency remains live");
        remove_occurrence(
            &mut self.system_mut_after_validation(removed.system).particles,
            &particle,
        );
        if let Some(group) = removed.maybe_group {
            remove_occurrence(
                &mut self.group_mut_after_validation(group).particles,
                &particle,
            );
        }
        DestructionRecord {
            destroyed: DestroyedId::Particle(particle),
            diagnostic_id: removed.diagnostic_id,
            cause,
            snapshot: ObjectSnapshot::Particle {
                system: removed.system,
                maybe_group: removed.maybe_group,
            },
        }
    }

    fn body_mut_after_validation(&mut self, body: BodyId) -> &mut Body {
        self.bodies
            .get_mut(body)
            .expect("validated body remains live during one operation")
    }

    fn system_mut_after_validation(&mut self, system: ParticleSystemId) -> &mut ParticleSystem {
        self.particle_systems
            .get_mut(system)
            .expect("validated particle system remains live during one operation")
    }

    fn group_mut_after_validation(&mut self, group: ParticleGroupId) -> &mut ParticleGroup {
        self.particle_groups
            .get_mut(group)
            .expect("validated particle group remains live during one operation")
    }

    fn particle_mut_after_validation(&mut self, particle: ParticleId) -> &mut Particle {
        self.particles
            .get_mut(particle)
            .expect("validated particle remains live during one operation")
    }
}

fn remove_occurrence<T: PartialEq>(items: &mut Vec<T>, target: &T) {
    let position = items
        .iter()
        .position(|item| item == target)
        .expect("bidirectional adjacency contains matching occurrence");
    items.remove(position);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_world() -> World {
        World::new().expect("test world key should remain available")
    }

    #[test]
    fn body_destruction_cascades_joints_then_fixtures_and_preserves_other_body() {
        // Arrange
        let mut world = test_world();
        let root = world.create_body().expect("body should fit");
        let survivor = world.create_body().expect("body should fit");
        let first_fixture = world.create_fixture(root).expect("fixture should fit");
        let second_fixture = world.create_fixture(root).expect("fixture should fit");
        let first_joint = world
            .create_joint(root, survivor)
            .expect("joint should fit");
        let second_joint = world
            .create_joint(root, survivor)
            .expect("joint should fit");

        // Act
        let records = world.destroy_body(root).expect("root should be live");

        // Assert
        assert_eq!(
            records
                .iter()
                .map(DestructionRecord::destroyed)
                .collect::<Vec<_>>(),
            vec![
                DestroyedId::Joint(second_joint),
                DestroyedId::Joint(first_joint),
                DestroyedId::Fixture(second_fixture),
                DestroyedId::Fixture(first_fixture),
                DestroyedId::Body(root),
            ]
        );
        assert!(!world.contains_body(root));
        assert!(!world.contains_joint(first_joint));
        assert!(!world.contains_joint(second_joint));
        assert!(!world.contains_fixture(first_fixture));
        assert!(!world.contains_fixture(second_fixture));
        assert!(world.contains_body(survivor));
        assert!(
            world
                .bodies
                .get(survivor)
                .expect("survivor remains live")
                .joints
                .is_empty()
        );
        assert!(matches!(
            records.last().map(DestructionRecord::snapshot),
            Some(ObjectSnapshot::Body { fixtures, joints })
                if fixtures == &[second_fixture, first_fixture]
                    && joints == &[second_joint, first_joint]
        ));
    }

    #[test]
    fn invalid_body_destruction_is_state_preserving() {
        // Arrange
        let mut world = test_world();
        let body = world.create_body().expect("body should fit");
        let fixture = world.create_fixture(body).expect("fixture should fit");
        let mut other = test_world();
        let foreign = other.create_body().expect("body should fit");

        // Act
        let result = world.destroy_body(foreign);

        // Assert
        assert_eq!(result, Err(HandleError::WrongWorld));
        assert!(world.contains_body(body));
        assert!(world.contains_fixture(fixture));
        assert_eq!(world.bodies.iter().count(), 1);
        assert_eq!(world.fixtures.iter().count(), 1);
    }

    #[test]
    fn stale_body_destruction_is_state_preserving() {
        // Arrange
        let mut world = test_world();
        let stale = world.create_body().expect("body should fit");
        world.destroy_body(stale).expect("body should be live");
        let survivor = world.create_body().expect("body should fit");

        // Act
        let result = world.destroy_body(stale);

        // Assert
        assert_eq!(result, Err(HandleError::StaleOrDestroyed));
        assert!(world.contains_body(survivor));
        assert_eq!(world.bodies.iter().count(), 1);
    }

    #[test]
    fn particle_system_destruction_cascades_groups_then_particles() {
        // Arrange
        let mut world = test_world();
        let system = world
            .create_particle_system()
            .expect("particle system should fit");
        let group = world
            .create_particle_group(system)
            .expect("particle group should fit");
        let grouped = world
            .create_particle(system, Some(group))
            .expect("particle should fit");
        let ungrouped = world
            .create_particle(system, None)
            .expect("particle should fit");

        // Act
        let records = world
            .destroy_particle_system(system)
            .expect("system should be live");

        // Assert
        assert_eq!(
            records
                .iter()
                .map(DestructionRecord::destroyed)
                .collect::<Vec<_>>(),
            vec![
                DestroyedId::ParticleGroup(group),
                DestroyedId::Particle(grouped),
                DestroyedId::Particle(ungrouped),
                DestroyedId::ParticleSystem(system),
            ]
        );
        assert!(!world.contains_particle_system(system));
        assert!(!world.contains_particle_group(group));
        assert!(!world.contains_particle(grouped));
        assert!(!world.contains_particle(ungrouped));
        assert!(matches!(
            records.first().map(DestructionRecord::snapshot),
            Some(ObjectSnapshot::ParticleGroup { particles, .. }) if particles == &[grouped]
        ));
    }

    #[test]
    fn invalid_particle_system_destruction_is_state_preserving() {
        // Arrange
        let mut world = test_world();
        let system = world
            .create_particle_system()
            .expect("particle system should fit");
        let particle = world
            .create_particle(system, None)
            .expect("particle should fit");
        let mut other = test_world();
        let foreign = other
            .create_particle_system()
            .expect("particle system should fit");

        // Act
        let result = world.destroy_particle_system(foreign);

        // Assert
        assert_eq!(result, Err(HandleError::WrongWorld));
        assert!(world.contains_particle_system(system));
        assert!(world.contains_particle(particle));
        assert_eq!(world.particle_systems.iter().count(), 1);
        assert_eq!(world.particles.iter().count(), 1);
    }

    #[test]
    fn direct_dependent_destruction_updates_all_adjacency() {
        // Arrange
        let mut world = test_world();
        let first = world.create_body().expect("body should fit");
        let second = world.create_body().expect("body should fit");
        let fixture = world.create_fixture(first).expect("fixture should fit");
        let joint = world.create_joint(first, second).expect("joint should fit");
        let system = world
            .create_particle_system()
            .expect("particle system should fit");
        let group = world
            .create_particle_group(system)
            .expect("particle group should fit");
        let particle = world
            .create_particle(system, Some(group))
            .expect("particle should fit");

        // Act
        world
            .destroy_fixture(fixture)
            .expect("fixture should be live");
        world.destroy_joint(joint).expect("joint should be live");
        world
            .destroy_particle(particle)
            .expect("particle should be live");
        world
            .destroy_particle_group(group)
            .expect("group should be live");

        // Assert
        assert!(
            world
                .bodies
                .get(first)
                .expect("body remains live")
                .fixtures
                .is_empty()
        );
        assert!(
            world
                .bodies
                .get(first)
                .expect("body remains live")
                .joints
                .is_empty()
        );
        assert!(
            world
                .bodies
                .get(second)
                .expect("body remains live")
                .joints
                .is_empty()
        );
        let system = world
            .particle_systems
            .get(system)
            .expect("system remains live");
        assert!(system.groups.is_empty());
        assert!(system.particles.is_empty());
    }

    #[test]
    fn owned_records_remain_usable_after_slot_reuse() {
        // Arrange
        let mut world = test_world();
        let body = world.create_body().expect("body should fit");
        let fixture = world.create_fixture(body).expect("fixture should fit");

        // Act
        let records = world.destroy_body(body).expect("body should be live");
        let replacement = world.create_body().expect("reused slot should fit");

        // Assert
        assert_ne!(body, replacement);
        assert_eq!(records[0].destroyed(), DestroyedId::Fixture(fixture));
        assert_eq!(records[1].destroyed(), DestroyedId::Body(body));
        assert!(matches!(
            records[0].snapshot(),
            ObjectSnapshot::Fixture { body: snapshot_body } if *snapshot_body == body
        ));
    }
}
