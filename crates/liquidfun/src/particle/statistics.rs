//! Owned semantic particle statistics over stable identities.

use crate::math::settings;
use crate::particle::storage::ParticleStorage;
use crate::{ParticleId, ParticleSystemDef, ParticleSystemId};

/// Owned statistics for one live particle system.
#[derive(Debug, Clone, PartialEq)]
pub struct ParticleSystemStatistics {
    system: ParticleSystemId,
    particle_ids: Vec<ParticleId>,
    pending_particle_count: usize,
    group_count: usize,
    particle_contact_count: usize,
    body_contact_count: usize,
    stuck_candidates: Vec<ParticleId>,
    collision_energy: f32,
    paused: bool,
    declared_capacity: usize,
    effective_capacity: usize,
    configured_maximum: Option<usize>,
}

impl ParticleSystemStatistics {
    pub(crate) fn from_storage(
        storage: &ParticleStorage,
        definition: ParticleSystemDef,
        group_count: usize,
    ) -> Self {
        let declared_capacity = storage.declared_capacity();
        let configured_maximum = definition.maximum_count();
        let effective_capacity =
            configured_maximum.map_or(declared_capacity, |maximum| maximum.min(declared_capacity));
        Self {
            system: storage.system(),
            particle_ids: storage.particle_ids().to_vec(),
            pending_particle_count: storage.pending_count(),
            group_count,
            particle_contact_count: storage.particle_contacts().len(),
            body_contact_count: storage.body_contacts().len(),
            stuck_candidates: storage.stuck_candidates().collect(),
            collision_energy: collision_energy(storage, definition),
            paused: definition.is_paused(),
            declared_capacity,
            effective_capacity,
            configured_maximum,
        }
    }

    /// Returns the owning particle-system identity.
    #[must_use]
    pub const fn system(&self) -> ParticleSystemId {
        self.system
    }

    /// Returns stable particle identities in current source order.
    #[must_use]
    pub fn particle_ids(&self) -> &[ParticleId] {
        &self.particle_ids
    }

    /// Returns the live plus pending particle count.
    #[must_use]
    pub fn particle_count(&self) -> usize {
        self.particle_ids.len()
    }

    /// Returns the number of particles awaiting compaction.
    #[must_use]
    pub const fn pending_particle_count(&self) -> usize {
        self.pending_particle_count
    }

    /// Returns the number of live particle groups owned by the system.
    #[must_use]
    pub const fn group_count(&self) -> usize {
        self.group_count
    }

    /// Returns the current particle-particle contact occurrence count.
    #[must_use]
    pub const fn particle_contact_count(&self) -> usize {
        self.particle_contact_count
    }

    /// Returns the current fixture/body contact occurrence count.
    #[must_use]
    pub const fn body_contact_count(&self) -> usize {
        self.body_contact_count
    }

    /// Returns possible stuck particles as stable identities.
    #[must_use]
    pub fn stuck_candidates(&self) -> &[ParticleId] {
        &self.stuck_candidates
    }

    /// Returns source-ordered kinetic energy available to contact damping.
    #[must_use]
    pub const fn collision_energy(&self) -> f32 {
        self.collision_energy
    }

    /// Returns whether world stepping currently skips this system.
    #[must_use]
    pub const fn is_paused(&self) -> bool {
        self.paused
    }

    /// Returns the explicit storage limit, never an allocator capacity.
    #[must_use]
    pub const fn declared_capacity(&self) -> usize {
        self.declared_capacity
    }

    /// Returns the storage limit after applying the configured maximum.
    #[must_use]
    pub const fn effective_capacity(&self) -> usize {
        self.effective_capacity
    }

    /// Returns the configured maximum, or `None` for the pinned unlimited value.
    #[must_use]
    pub const fn configured_maximum(&self) -> Option<usize> {
        self.configured_maximum
    }
}

/// Owned aggregate counts across all current particle systems.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ParticleWorldStatistics {
    system_count: usize,
    particle_count: usize,
    pending_particle_count: usize,
    group_count: usize,
    particle_contact_count: usize,
    body_contact_count: usize,
    stuck_candidate_count: usize,
    collision_energy: f32,
}

impl ParticleWorldStatistics {
    pub(crate) fn include(&mut self, statistics: &ParticleSystemStatistics) {
        self.system_count += 1;
        self.particle_count += statistics.particle_count();
        self.pending_particle_count += statistics.pending_particle_count();
        self.group_count += statistics.group_count();
        self.particle_contact_count += statistics.particle_contact_count();
        self.body_contact_count += statistics.body_contact_count();
        self.stuck_candidate_count += statistics.stuck_candidates().len();
        self.collision_energy += statistics.collision_energy();
    }

    /// Returns the number of current particle systems.
    #[must_use]
    pub const fn system_count(self) -> usize {
        self.system_count
    }

    /// Returns the live plus pending particle count across all systems.
    #[must_use]
    pub const fn particle_count(self) -> usize {
        self.particle_count
    }

    /// Returns the pending particle count across all systems.
    #[must_use]
    pub const fn pending_particle_count(self) -> usize {
        self.pending_particle_count
    }

    /// Returns the current particle-group count across all systems.
    #[must_use]
    pub const fn group_count(self) -> usize {
        self.group_count
    }

    /// Returns current particle-particle contact occurrences across all systems.
    #[must_use]
    pub const fn particle_contact_count(self) -> usize {
        self.particle_contact_count
    }

    /// Returns current fixture/body contact occurrences across all systems.
    #[must_use]
    pub const fn body_contact_count(self) -> usize {
        self.body_contact_count
    }

    /// Returns possible stuck-particle occurrences across all systems.
    #[must_use]
    pub const fn stuck_candidate_count(self) -> usize {
        self.stuck_candidate_count
    }

    /// Returns source-ordered collision energy summed in system traversal order.
    #[must_use]
    pub const fn collision_energy(self) -> f32 {
        self.collision_energy
    }
}

fn collision_energy(storage: &ParticleStorage, definition: ParticleSystemDef) -> f32 {
    let velocities = storage.velocities();
    let mut sum_velocity_squared = 0.0;
    for contact in storage.particle_contacts() {
        let [first, second] = contact.indices;
        let relative_velocity = velocities[second.0] - velocities[first.0];
        let normal_velocity = relative_velocity.dot(contact.normal);
        if normal_velocity < 0.0 {
            sum_velocity_squared += normal_velocity * normal_velocity;
        }
    }
    if sum_velocity_squared == 0.0 {
        return 0.0;
    }
    let diameter = 2.0 * definition.radius();
    let stride = settings::PARTICLE_STRIDE * diameter;
    let particle_mass = definition.density() * stride * stride;
    0.5 * particle_mass * sum_velocity_squared
}
