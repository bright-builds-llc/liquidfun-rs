//! Bounded renderer-neutral observations over stable public identities.

use std::error::Error;
use std::fmt;

use crate::collision::{Aabb, ChildIndex, Manifold};
use crate::particle::{
    ParticleColor, ParticleSystemStatistics, ParticleSystemView, ParticleWorldStatistics,
};
use crate::{
    BodyId, BodySnapshot, ContactPointSnapshot, FixtureId, FixtureSnapshot, JointId, JointSnapshot,
    ManagedContactSnapshot, ParticleFlags, ParticleId, ParticleSystemId,
};

use super::WorldDiagnostics;
use super::object::World;

mod collection;
mod profile;
mod records;
use collection::check_requested_limit;
pub(in crate::world) use profile::DiagnosticStepProfiler;
pub use profile::{
    DiagnosticProfileChild, DiagnosticProfileParent, DiagnosticProfileSchema, DiagnosticStepPhase,
    DiagnosticStepPhaseTiming, DiagnosticStepProfile,
};
pub use records::{
    BodyObservation, BroadPhaseObservation, ContactObservation, FixtureObservation,
    JointObservation, ParticleBodyContactObservation, ParticleContactObservation,
    ParticleObservation,
};

const REVIEWED_MAX_CONTACTS: usize = 4_096;
const REVIEWED_MAX_PARTICLE_CONTACTS: usize = 65_536;
const REVIEWED_MAX_PARTICLE_BODY_CONTACTS: usize = 65_536;
const REVIEWED_MAX_BROAD_PHASE_OBSERVATIONS: usize = 32_768;
const REVIEWED_MAX_PARTICLE_SYSTEMS: usize = 1_024;
const REVIEWED_MAX_PARTICLES: usize = 1_048_576;
const REVIEWED_MAX_BODIES: usize = 4_096;
const REVIEWED_MAX_FIXTURES: usize = 8_192;
const REVIEWED_MAX_JOINTS: usize = 8_192;

/// A bounded observation collection whose order follows the engine's semantic traversal order.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldObservation {
    diagnostics: WorldDiagnostics,
    bodies: Vec<BodyObservation>,
    fixtures: Vec<FixtureObservation>,
    joints: Vec<JointObservation>,
    particles: Vec<ParticleObservation>,
    contacts: Vec<ContactObservation>,
    particle_contacts: Vec<ParticleContactObservation>,
    particle_body_contacts: Vec<ParticleBodyContactObservation>,
    broad_phase_observations: Vec<BroadPhaseObservation>,
    particle_statistics: Vec<ParticleSystemStatistics>,
    particle_world_statistics: ParticleWorldStatistics,
}

impl WorldObservation {
    /// Returns exact world counts and dynamic-tree metrics.
    #[must_use]
    pub const fn diagnostics(&self) -> WorldDiagnostics {
        self.diagnostics
    }

    /// Returns live bodies in newest-first world order.
    #[must_use]
    pub fn bodies(&self) -> &[BodyObservation] {
        &self.bodies
    }

    /// Returns live fixtures in newest-first body and fixture order.
    #[must_use]
    pub fn fixtures(&self) -> &[FixtureObservation] {
        &self.fixtures
    }

    /// Returns live joints in newest-first world order.
    #[must_use]
    pub fn joints(&self) -> &[JointObservation] {
        &self.joints
    }

    /// Returns live particles in newest-first system and stored particle order.
    #[must_use]
    pub fn particles(&self) -> &[ParticleObservation] {
        &self.particles
    }

    /// Returns current rigid contacts in source-significant manager order.
    #[must_use]
    pub fn contacts(&self) -> &[ContactObservation] {
        &self.contacts
    }

    /// Returns particle-pair contacts in newest-first system order and stored contact order.
    #[must_use]
    pub fn particle_contacts(&self) -> &[ParticleContactObservation] {
        &self.particle_contacts
    }

    /// Returns particle-to-fixture contacts in newest-first system order and stored contact order.
    #[must_use]
    pub fn particle_body_contacts(&self) -> &[ParticleBodyContactObservation] {
        &self.particle_body_contacts
    }

    /// Returns current fixture-child AABBs in newest-first body and fixture order.
    #[must_use]
    pub fn broad_phase_observations(&self) -> &[BroadPhaseObservation] {
        &self.broad_phase_observations
    }

    /// Returns one owned semantic statistics record per system in newest-first order.
    #[must_use]
    pub fn particle_statistics(&self) -> &[ParticleSystemStatistics] {
        &self.particle_statistics
    }

    /// Returns aggregate particle statistics summed in newest-first system order.
    #[must_use]
    pub const fn particle_world_statistics(&self) -> ParticleWorldStatistics {
        self.particle_world_statistics
    }
}

/// Reviewed finite capacities for one owned world observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldObservationLimits {
    contacts: usize,
    particle_contacts: usize,
    particle_body_contacts: usize,
    broad_phase_observations: usize,
    particle_systems: usize,
    particles: usize,
}

impl WorldObservationLimits {
    /// Returns the repository-reviewed production limits.
    #[must_use]
    pub const fn reviewed() -> Self {
        Self {
            contacts: REVIEWED_MAX_CONTACTS,
            particle_contacts: REVIEWED_MAX_PARTICLE_CONTACTS,
            particle_body_contacts: REVIEWED_MAX_PARTICLE_BODY_CONTACTS,
            broad_phase_observations: REVIEWED_MAX_BROAD_PHASE_OBSERVATIONS,
            particle_systems: REVIEWED_MAX_PARTICLE_SYSTEMS,
            particles: REVIEWED_MAX_PARTICLES,
        }
    }

    /// Creates per-collection limits no larger than the reviewed maxima.
    ///
    /// # Errors
    ///
    /// Returns a typed error naming the first requested limit above its hard maximum.
    pub fn new(
        contacts: usize,
        particle_contacts: usize,
        particle_body_contacts: usize,
        broad_phase_observations: usize,
        particle_systems: usize,
        particles: usize,
    ) -> Result<Self, WorldObservationLimitError> {
        check_requested_limit(
            WorldObservationResource::Contacts,
            contacts,
            REVIEWED_MAX_CONTACTS,
        )?;
        check_requested_limit(
            WorldObservationResource::ParticleContacts,
            particle_contacts,
            REVIEWED_MAX_PARTICLE_CONTACTS,
        )?;
        check_requested_limit(
            WorldObservationResource::ParticleBodyContacts,
            particle_body_contacts,
            REVIEWED_MAX_PARTICLE_BODY_CONTACTS,
        )?;
        check_requested_limit(
            WorldObservationResource::BroadPhaseObservations,
            broad_phase_observations,
            REVIEWED_MAX_BROAD_PHASE_OBSERVATIONS,
        )?;
        check_requested_limit(
            WorldObservationResource::ParticleSystems,
            particle_systems,
            REVIEWED_MAX_PARTICLE_SYSTEMS,
        )?;
        check_requested_limit(
            WorldObservationResource::Particles,
            particles,
            REVIEWED_MAX_PARTICLES,
        )?;
        Ok(Self {
            contacts,
            particle_contacts,
            particle_body_contacts,
            broad_phase_observations,
            particle_systems,
            particles,
        })
    }

    /// Returns the maximum rigid-contact record count.
    #[must_use]
    pub const fn max_contacts(self) -> usize {
        self.contacts
    }

    /// Returns the maximum particle-pair contact record count.
    #[must_use]
    pub const fn max_particle_contacts(self) -> usize {
        self.particle_contacts
    }

    /// Returns the maximum particle-to-fixture contact record count.
    #[must_use]
    pub const fn max_particle_body_contacts(self) -> usize {
        self.particle_body_contacts
    }

    /// Returns the maximum fixture-child AABB record count.
    #[must_use]
    pub const fn max_broad_phase_observations(self) -> usize {
        self.broad_phase_observations
    }

    /// Returns the maximum particle-system statistics record count.
    #[must_use]
    pub const fn max_particle_systems(self) -> usize {
        self.particle_systems
    }

    /// Returns the maximum total stable particle identity count.
    #[must_use]
    pub const fn max_particles(self) -> usize {
        self.particles
    }
}

/// A semantic collection category used by bounded observation errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WorldObservationResource {
    /// Current live bodies.
    Bodies,
    /// Current live fixtures.
    Fixtures,
    /// Current live joints.
    Joints,
    /// Current rigid contacts.
    Contacts,
    /// Current particle-pair contacts.
    ParticleContacts,
    /// Current particle-to-fixture contacts.
    ParticleBodyContacts,
    /// Current fixture-child AABBs.
    BroadPhaseObservations,
    /// Current particle systems.
    ParticleSystems,
    /// Stable particle identities copied into statistics.
    Particles,
}

impl fmt::Display for WorldObservationResource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Bodies => "bodies",
            Self::Fixtures => "fixtures",
            Self::Joints => "joints",
            Self::Contacts => "contacts",
            Self::ParticleContacts => "particle contacts",
            Self::ParticleBodyContacts => "particle body contacts",
            Self::BroadPhaseObservations => "broad-phase observations",
            Self::ParticleSystems => "particle systems",
            Self::Particles => "particles",
        };
        formatter.write_str(name)
    }
}

/// A requested observation limit exceeded a reviewed hard maximum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldObservationLimitError {
    resource: WorldObservationResource,
    requested: usize,
    maximum: usize,
}

impl WorldObservationLimitError {
    /// Returns the rejected semantic collection.
    #[must_use]
    pub const fn resource(self) -> WorldObservationResource {
        self.resource
    }

    /// Returns the rejected requested limit.
    #[must_use]
    pub const fn requested(self) -> usize {
        self.requested
    }

    /// Returns the reviewed hard maximum.
    #[must_use]
    pub const fn maximum(self) -> usize {
        self.maximum
    }
}

impl fmt::Display for WorldObservationLimitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "requested {} limit {} exceeds reviewed maximum {}",
            self.resource, self.requested, self.maximum
        )
    }
}

impl Error for WorldObservationLimitError {}

/// A bounded failure while collecting owned semantic observations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WorldObservationError {
    /// A live collection exceeds its configured finite capacity.
    CapacityExceeded {
        /// Stable semantic collection category.
        resource: WorldObservationResource,
        /// Configured finite limit.
        limit: usize,
    },
    /// Private world state could not be translated into complete semantic records.
    InvalidState {
        /// Stable bounded category, never a private coordinate.
        resource: WorldObservationResource,
    },
}

impl fmt::Display for WorldObservationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapacityExceeded { resource, limit } => {
                write!(formatter, "{resource} exceed observation limit {limit}")
            }
            Self::InvalidState { resource } => {
                write!(formatter, "{resource} could not be translated semantically")
            }
        }
    }
}

impl Error for WorldObservationError {}
