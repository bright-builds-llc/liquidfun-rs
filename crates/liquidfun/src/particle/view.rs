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
/// world
///     .create_particle(system, None)
///     .expect("particle should fit")
///     .created_particle();
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

    /// Returns accumulated forces in newtons, aligned with particle identities.
    #[must_use]
    pub fn forces(&self) -> &[Vec2] {
        self.storage.forces()
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
            pa: triad.pa,
            pb: triad.pb,
            pc: triad.pc,
            ka: triad.ka,
            kb: triad.kb,
            kc: triad.kc,
            s: triad.s,
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
    pa: Vec2,
    pb: Vec2,
    pc: Vec2,
    ka: f32,
    kb: f32,
    kc: f32,
    s: f32,
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

    /// Returns the first centroid-relative rest offset in meters.
    #[must_use]
    pub const fn pa(self) -> Vec2 {
        self.pa
    }

    /// Returns the second centroid-relative rest offset in meters.
    #[must_use]
    pub const fn pb(self) -> Vec2 {
        self.pb
    }

    /// Returns the third centroid-relative rest offset in meters.
    #[must_use]
    pub const fn pc(self) -> Vec2 {
        self.pc
    }

    /// Returns the signed rest coefficient opposite the first particle.
    #[must_use]
    pub const fn ka(self) -> f32 {
        self.ka
    }

    /// Returns the signed rest coefficient opposite the second particle.
    #[must_use]
    pub const fn kb(self) -> f32 {
        self.kb
    }

    /// Returns the signed rest coefficient opposite the third particle.
    #[must_use]
    pub const fn kc(self) -> f32 {
        self.kc
    }

    /// Returns the signed doubled rest area.
    #[must_use]
    pub const fn s(self) -> f32 {
        self.s
    }
}

#[cfg(test)]
mod tests {
    use crate::identity::{HandleIdentity, Identity, ParticleSystemId, WorldKey};
    use crate::particle::storage::{ParticleInput, ParticleStorage};

    use super::*;

    fn input(position: Vec2) -> ParticleInput {
        ParticleInput {
            position,
            velocity: Vec2::ZERO,
            flags: ParticleFlags::ELASTIC,
            maybe_group: None,
            maybe_color: None,
            maybe_user_association: None,
            maybe_expiration_time: None,
        }
    }

    #[test]
    fn triad_view_translates_oriented_rest_state_to_stable_particle_ids() {
        // Arrange
        let world = WorldKey::fresh().expect("test world key remains available");
        let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
        let mut storage =
            ParticleStorage::new(world, system, 0, 3, 3).expect("test storage contract is valid");
        let particles = [
            storage
                .create(input(Vec2::new(1.0, 0.0)))
                .expect("first particle fits"),
            storage
                .create(input(Vec2::new(0.0, 1.0)))
                .expect("second particle fits"),
            storage
                .create(input(Vec2::new(-1.0, -1.0)))
                .expect("third particle fits"),
        ];
        let triad = ParticleTriad {
            indices: [ParticleIndex(2), ParticleIndex(0), ParticleIndex(1)],
            flags: ParticleFlags::ELASTIC,
            strength: -0.5,
            pa: Vec2::new(1.0, -2.0),
            pb: Vec2::new(-3.0, 4.0),
            pc: Vec2::new(5.0, -6.0),
            ka: -7.0,
            kb: 8.0,
            kc: -9.0,
            s: -10.0,
        };
        let view = ParticleSystemView::new(&storage);

        // Act
        let triad_view = view.triad_view(triad);

        // Assert
        assert_eq!(
            triad_view.particles(),
            [particles[2], particles[0], particles[1]]
        );
        assert_eq!(triad_view.flags(), ParticleFlags::ELASTIC);
        assert_eq!(triad_view.strength().to_bits(), (-0.5_f32).to_bits());
        assert_eq!(triad_view.pa(), Vec2::new(1.0, -2.0));
        assert_eq!(triad_view.pb(), Vec2::new(-3.0, 4.0));
        assert_eq!(triad_view.pc(), Vec2::new(5.0, -6.0));
        assert_eq!(triad_view.ka().to_bits(), (-7.0_f32).to_bits());
        assert_eq!(triad_view.kb().to_bits(), 8.0_f32.to_bits());
        assert_eq!(triad_view.kc().to_bits(), (-9.0_f32).to_bits());
        assert_eq!(triad_view.s().to_bits(), (-10.0_f32).to_bits());
    }
}
