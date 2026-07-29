use super::{
    Aabb, BodyId, BodySnapshot, ChildIndex, ContactPointSnapshot, FixtureId, FixtureSnapshot,
    JointId, JointSnapshot, ManagedContactSnapshot, Manifold, ParticleColor, ParticleFlags,
    ParticleId, ParticleSystemId,
};

/// One owned body observation identified by its stable public handle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyObservation {
    pub(super) id: BodyId,
    pub(super) snapshot: BodySnapshot,
}

impl BodyObservation {
    /// Returns the stable body identity.
    #[must_use]
    pub const fn id(self) -> BodyId {
        self.id
    }

    /// Returns the owned semantic body state.
    #[must_use]
    pub const fn snapshot(self) -> BodySnapshot {
        self.snapshot
    }
}

/// One owned fixture observation with semantic owner and immutable geometry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureObservation {
    pub(super) id: FixtureId,
    pub(super) body: BodyId,
    pub(super) snapshot: FixtureSnapshot,
}

impl FixtureObservation {
    /// Returns the stable fixture identity.
    #[must_use]
    pub const fn id(&self) -> FixtureId {
        self.id
    }

    /// Returns the stable owning body identity.
    #[must_use]
    pub const fn body(&self) -> BodyId {
        self.body
    }

    /// Returns the owned immutable fixture state.
    #[must_use]
    pub const fn snapshot(&self) -> &FixtureSnapshot {
        &self.snapshot
    }
}

/// One owned joint observation identified by its stable public handle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JointObservation {
    pub(super) id: JointId,
    pub(super) snapshot: JointSnapshot,
}

impl JointObservation {
    /// Returns the stable joint identity.
    #[must_use]
    pub const fn id(self) -> JointId {
        self.id
    }

    /// Returns the owned semantic joint state.
    #[must_use]
    pub const fn snapshot(self) -> JointSnapshot {
        self.snapshot
    }
}

/// One owned particle observation with no dense row coordinate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleObservation {
    pub(super) system: ParticleSystemId,
    pub(super) particle: ParticleId,
    pub(super) position: crate::math::Vec2,
    pub(super) radius: f32,
    pub(super) color: ParticleColor,
}

impl ParticleObservation {
    /// Returns the stable owning system identity.
    #[must_use]
    pub const fn system(self) -> ParticleSystemId {
        self.system
    }

    /// Returns the stable particle identity.
    #[must_use]
    pub const fn particle(self) -> ParticleId {
        self.particle
    }

    /// Returns the current world-space position in meters.
    #[must_use]
    pub const fn position(self) -> crate::math::Vec2 {
        self.position
    }

    /// Returns the owning system's particle radius in meters.
    #[must_use]
    pub const fn radius(self) -> f32 {
        self.radius
    }

    /// Returns the exact particle color.
    #[must_use]
    pub const fn color(self) -> ParticleColor {
        self.color
    }
}

/// One owned rigid contact without a contact-manager index or reusable contact identity.
#[derive(Debug, Clone, PartialEq)]
pub struct ContactObservation {
    fixtures: [FixtureId; 2],
    bodies: [BodyId; 2],
    child_indices: [ChildIndex; 2],
    touching: bool,
    enabled: bool,
    sensor: bool,
    maybe_manifold: Option<Manifold>,
    points: Vec<ContactPointSnapshot>,
    friction: f32,
    restitution: f32,
    tangent_speed: f32,
}

impl ContactObservation {
    pub(super) fn from_snapshot(snapshot: &ManagedContactSnapshot) -> Self {
        Self {
            fixtures: snapshot.fixtures(),
            bodies: snapshot.bodies(),
            child_indices: snapshot.child_indices(),
            touching: snapshot.is_touching(),
            enabled: snapshot.is_enabled(),
            sensor: snapshot.is_sensor(),
            maybe_manifold: snapshot.maybe_manifold().cloned(),
            points: snapshot.points().to_vec(),
            friction: snapshot.friction(),
            restitution: snapshot.restitution(),
            tangent_speed: snapshot.tangent_speed(),
        }
    }

    /// Returns stable fixture identities in oriented source order.
    #[must_use]
    pub const fn fixtures(&self) -> [FixtureId; 2] {
        self.fixtures
    }

    /// Returns stable body identities in oriented source order.
    #[must_use]
    pub const fn bodies(&self) -> [BodyId; 2] {
        self.bodies
    }

    /// Returns public shape-child coordinates in oriented source order.
    #[must_use]
    pub const fn child_indices(&self) -> [ChildIndex; 2] {
        self.child_indices
    }

    /// Returns whether this occurrence is currently touching.
    #[must_use]
    pub const fn is_touching(&self) -> bool {
        self.touching
    }

    /// Returns whether this occurrence is enabled for solving.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns whether either fixture is a sensor.
    #[must_use]
    pub const fn is_sensor(&self) -> bool {
        self.sensor
    }

    /// Returns the current semantic manifold, absent for sensors and separation.
    #[must_use]
    pub const fn maybe_manifold(&self) -> Option<&Manifold> {
        self.maybe_manifold.as_ref()
    }

    /// Returns manifold points in canonical point order.
    #[must_use]
    pub fn points(&self) -> &[ContactPointSnapshot] {
        &self.points
    }

    /// Returns the mixed dimensionless friction coefficient.
    #[must_use]
    pub const fn friction(&self) -> f32 {
        self.friction
    }

    /// Returns the mixed dimensionless restitution coefficient.
    #[must_use]
    pub const fn restitution(&self) -> f32 {
        self.restitution
    }

    /// Returns the configured surface tangent speed in meters per second.
    #[must_use]
    pub const fn tangent_speed(&self) -> f32 {
        self.tangent_speed
    }
}

/// One owned particle-pair contact translated from private dense rows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleContactObservation {
    pub(super) system: ParticleSystemId,
    pub(super) particles: [ParticleId; 2],
    pub(super) flags: ParticleFlags,
    pub(super) weight: f32,
    pub(super) normal: crate::math::Vec2,
}

impl ParticleContactObservation {
    /// Returns the owning particle system.
    #[must_use]
    pub const fn system(self) -> ParticleSystemId {
        self.system
    }

    /// Returns both stable particle identities in stored contact order.
    #[must_use]
    pub const fn particles(self) -> [ParticleId; 2] {
        self.particles
    }

    /// Returns exact combined particle flags.
    #[must_use]
    pub const fn flags(self) -> ParticleFlags {
        self.flags
    }

    /// Returns the dimensionless contact weight.
    #[must_use]
    pub const fn weight(self) -> f32 {
        self.weight
    }

    /// Returns the contact normal from the first particle toward the second.
    #[must_use]
    pub const fn normal(self) -> crate::math::Vec2 {
        self.normal
    }
}

/// One owned particle-to-fixture contact translated from a private dense row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleBodyContactObservation {
    pub(super) system: ParticleSystemId,
    pub(super) particle: ParticleId,
    pub(super) body: BodyId,
    pub(super) fixture: FixtureId,
    pub(super) weight: f32,
    pub(super) normal: crate::math::Vec2,
    pub(super) mass: f32,
}

impl ParticleBodyContactObservation {
    /// Returns the owning particle system.
    #[must_use]
    pub const fn system(self) -> ParticleSystemId {
        self.system
    }

    /// Returns the stable particle identity.
    #[must_use]
    pub const fn particle(self) -> ParticleId {
        self.particle
    }

    /// Returns the stable contacted body identity.
    #[must_use]
    pub const fn body(self) -> BodyId {
        self.body
    }

    /// Returns the stable contacted fixture identity.
    #[must_use]
    pub const fn fixture(self) -> FixtureId {
        self.fixture
    }

    /// Returns the dimensionless contact weight.
    #[must_use]
    pub const fn weight(self) -> f32 {
        self.weight
    }

    /// Returns the contact normal directed toward the particle.
    #[must_use]
    pub const fn normal(self) -> crate::math::Vec2 {
        self.normal
    }

    /// Returns effective contact mass in kilograms.
    #[must_use]
    pub const fn mass(self) -> f32 {
        self.mass
    }
}

/// One current fixture-child AABB identified only by stable semantic owners.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BroadPhaseObservation {
    pub(super) body: BodyId,
    pub(super) fixture: FixtureId,
    pub(super) child_index: ChildIndex,
    pub(super) aabb: Aabb,
}

impl BroadPhaseObservation {
    /// Returns the stable owning body identity.
    #[must_use]
    pub const fn body(self) -> BodyId {
        self.body
    }

    /// Returns the stable fixture identity.
    #[must_use]
    pub const fn fixture(self) -> FixtureId {
        self.fixture
    }

    /// Returns the public shape-child coordinate.
    #[must_use]
    pub const fn child_index(self) -> ChildIndex {
        self.child_index
    }

    /// Returns the tight current world-space bounds in meters.
    ///
    /// Private fattened tree bounds and proxy identities are intentionally not exposed.
    #[must_use]
    pub const fn aabb(self) -> Aabb {
        self.aabb
    }

    /// Applies the same inclusive AABB overlap semantics as public world queries.
    #[must_use]
    pub fn overlaps(self, query: Aabb) -> bool {
        self.aabb.overlaps(query)
    }
}
