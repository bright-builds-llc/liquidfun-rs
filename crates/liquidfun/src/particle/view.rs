//! Borrow-scoped semantic inspection of one particle system.

use crate::math::Vec2;
use crate::particle::storage::lanes::{
    ParticleBodyContact, ParticleContact, ParticlePair, ParticleTriad,
};
use crate::particle::storage::{ParticleIndex, ParticleStorage};
use crate::{
    AssociationMap, BodyId, FixtureId, ParticleColor, ParticleFlags, ParticleGroupId, ParticleId,
    ParticleSystemId,
};

/// Borrow-scoped semantic lanes for one live particle system.
///
/// Row coordinates, allocation capacity, scratch storage, and mutable lane
/// access remain private. Stable particle identities align the row-owned
/// slices, while derived records translate every private row reference before
/// crossing this boundary.
///
/// ```compile_fail
/// use liquidfun::World;
/// let mut world = World::new().expect("world key should remain available");
/// let system = world.create_particle_system().expect("system should fit");
/// let view = world.particle_system_view(system).expect("system should be live");
/// world.create_particle(system, None).expect("particle should fit");
/// let _positions = view.positions();
/// ```
pub struct ParticleSystemView<'a> {
    storage: &'a ParticleStorage,
}

impl<'a> ParticleSystemView<'a> {
    pub(crate) const fn new(storage: &'a ParticleStorage) -> Self {
        Self { storage }
    }

    /// Returns the owning particle system.
    #[must_use]
    pub fn system(&self) -> ParticleSystemId {
        self.storage.system()
    }

    /// Returns stable identities aligned with the row-owned property slices.
    #[must_use]
    pub fn particle_ids(&self) -> &[ParticleId] {
        self.storage.particle_ids()
    }

    /// Returns positions in meters, aligned with [`Self::particle_ids`].
    #[must_use]
    pub fn positions(&self) -> &[Vec2] {
        self.storage.positions()
    }

    /// Returns velocities in meters per second, aligned with particle identities.
    #[must_use]
    pub fn velocities(&self) -> &[Vec2] {
        self.storage.velocities()
    }

    /// Returns exact retained flag bits, aligned with particle identities.
    #[must_use]
    pub fn flags(&self) -> &[ParticleFlags] {
        self.storage.flags()
    }

    /// Returns stable group identities, aligned with particle identities.
    #[must_use]
    pub fn group_ids(&self) -> &[Option<ParticleGroupId>] {
        self.storage.groups()
    }

    /// Returns current derived particle weights, aligned with particle identities.
    #[must_use]
    pub fn weights(&self) -> &[f32] {
        self.storage.weights()
    }

    /// Iterates possible stuck particles from the latest contact sub-iteration.
    #[must_use]
    pub fn stuck_candidates(&self) -> impl ExactSizeIterator<Item = ParticleId> + '_ {
        self.storage.stuck_candidates()
    }

    /// Returns the lazily allocated color lane when any particle requires it.
    #[must_use]
    pub fn maybe_colors(&self) -> Option<&[ParticleColor]> {
        self.storage.maybe_colors()
    }

    /// Resolves application-owned user associations in stable particle order.
    ///
    /// User values remain in the caller's safe typed side table; the world does
    /// not duplicate or type-erase them inside particle storage.
    #[must_use]
    pub fn user_associations<'view, T>(
        &'view self,
        associations: &'view AssociationMap<ParticleId, T>,
    ) -> impl ExactSizeIterator<Item = (ParticleId, Option<&'view T>)> + 'view {
        self.particle_ids()
            .iter()
            .copied()
            .map(|id| (id, associations.get(&id)))
    }

    /// Iterates particle contacts with stable particle identities.
    #[must_use]
    pub fn particle_contacts(&self) -> impl ExactSizeIterator<Item = ParticleContactView> + '_ {
        self.storage
            .particle_contacts()
            .iter()
            .map(|contact| self.particle_contact_view(*contact))
    }

    /// Iterates fixture/body contacts with stable semantic identities.
    #[must_use]
    pub fn body_contacts(&self) -> impl ExactSizeIterator<Item = ParticleBodyContactView> + '_ {
        self.storage
            .body_contacts()
            .iter()
            .map(|contact| self.body_contact_view(*contact))
    }

    /// Iterates deferred pair records with stable particle identities.
    #[must_use]
    pub fn pairs(&self) -> impl ExactSizeIterator<Item = ParticlePairView> + '_ {
        self.storage
            .pairs()
            .iter()
            .map(|pair| self.pair_view(*pair))
    }

    /// Iterates deferred triad records with stable particle identities.
    #[must_use]
    pub fn triads(&self) -> impl ExactSizeIterator<Item = ParticleTriadView> + '_ {
        self.storage
            .triads()
            .iter()
            .map(|triad| self.triad_view(*triad))
    }

    /// Returns the lazily allocated expiration order as stable identities.
    #[must_use]
    pub fn maybe_expiration_order(&self) -> Option<impl ExactSizeIterator<Item = ParticleId> + '_> {
        self.storage.maybe_expiration_order().map(|order| {
            order
                .iter()
                .copied()
                .map(|index| self.storage.particle_id_at(index))
        })
    }

    fn particle_contact_view(&self, contact: ParticleContact) -> ParticleContactView {
        ParticleContactView {
            particles: self.particle_ids_for(contact.indices),
            flags: contact.flags,
            weight: contact.weight,
            normal: contact.normal,
        }
    }

    fn body_contact_view(&self, contact: ParticleBodyContact) -> ParticleBodyContactView {
        ParticleBodyContactView {
            particle: self.storage.particle_id_at(contact.index),
            body: contact.body,
            fixture: contact.fixture,
            weight: contact.weight,
            normal: contact.normal,
            mass: contact.mass,
        }
    }

    fn pair_view(&self, pair: ParticlePair) -> ParticlePairView {
        ParticlePairView {
            particles: self.particle_ids_for(pair.indices),
            flags: pair.flags,
            strength: pair.strength,
            distance: pair.distance,
        }
    }

    fn triad_view(&self, triad: ParticleTriad) -> ParticleTriadView {
        ParticleTriadView {
            particles: triad
                .indices
                .map(|index| self.storage.particle_id_at(index)),
            flags: triad.flags,
            strength: triad.strength,
        }
    }

    fn particle_ids_for(&self, indices: [ParticleIndex; 2]) -> [ParticleId; 2] {
        indices.map(|index| self.storage.particle_id_at(index))
    }
}

/// One semantic particle-particle contact occurrence.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleContactView {
    particles: [ParticleId; 2],
    flags: ParticleFlags,
    weight: f32,
    normal: Vec2,
}

impl ParticleContactView {
    /// Returns the two stable particle identities in stored contact order.
    #[must_use]
    pub const fn particles(self) -> [ParticleId; 2] {
        self.particles
    }

    /// Returns the exact combined contact flags.
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
    pub const fn normal(self) -> Vec2 {
        self.normal
    }
}

/// One semantic particle-to-fixture/body contact occurrence.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleBodyContactView {
    particle: ParticleId,
    body: BodyId,
    fixture: FixtureId,
    weight: f32,
    normal: Vec2,
    mass: f32,
}

impl ParticleBodyContactView {
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
    pub const fn normal(self) -> Vec2 {
        self.normal
    }

    /// Returns the effective contact mass in kilograms.
    #[must_use]
    pub const fn mass(self) -> f32 {
        self.mass
    }
}

/// One semantic particle-pair topology record.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticlePairView {
    particles: [ParticleId; 2],
    flags: ParticleFlags,
    strength: f32,
    distance: f32,
}

impl ParticlePairView {
    /// Returns stable pair identities in stored order.
    #[must_use]
    pub const fn particles(self) -> [ParticleId; 2] {
        self.particles
    }

    /// Returns exact combined pair flags.
    #[must_use]
    pub const fn flags(self) -> ParticleFlags {
        self.flags
    }

    /// Returns pair strength.
    #[must_use]
    pub const fn strength(self) -> f32 {
        self.strength
    }

    /// Returns pair rest distance in meters.
    #[must_use]
    pub const fn distance(self) -> f32 {
        self.distance
    }
}

/// One semantic particle-triad topology record.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleTriadView {
    particles: [ParticleId; 3],
    flags: ParticleFlags,
    strength: f32,
}

impl ParticleTriadView {
    /// Returns stable triad identities in stored order.
    #[must_use]
    pub const fn particles(self) -> [ParticleId; 3] {
        self.particles
    }

    /// Returns exact combined triad flags.
    #[must_use]
    pub const fn flags(self) -> ParticleFlags {
        self.flags
    }

    /// Returns triad strength.
    #[must_use]
    pub const fn strength(self) -> f32 {
        self.strength
    }
}
