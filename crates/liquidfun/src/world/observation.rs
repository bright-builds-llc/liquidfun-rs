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

mod profile;
pub(in crate::world) use profile::DiagnosticStepProfiler;
pub use profile::{DiagnosticStepPhase, DiagnosticStepPhaseTiming, DiagnosticStepProfile};

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

/// One owned body observation identified by its stable public handle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyObservation {
    id: BodyId,
    snapshot: BodySnapshot,
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
    id: FixtureId,
    body: BodyId,
    snapshot: FixtureSnapshot,
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
    id: JointId,
    snapshot: JointSnapshot,
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
    system: ParticleSystemId,
    particle: ParticleId,
    position: crate::math::Vec2,
    radius: f32,
    color: ParticleColor,
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
    fn from_snapshot(snapshot: &ManagedContactSnapshot) -> Self {
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
    system: ParticleSystemId,
    particles: [ParticleId; 2],
    flags: ParticleFlags,
    weight: f32,
    normal: crate::math::Vec2,
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
    system: ParticleSystemId,
    particle: ParticleId,
    body: BodyId,
    fixture: FixtureId,
    weight: f32,
    normal: crate::math::Vec2,
    mass: f32,
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
    body: BodyId,
    fixture: FixtureId,
    child_index: ChildIndex,
    aabb: Aabb,
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

struct ObservationCounts {
    particle_contacts: usize,
    particle_body_contacts: usize,
    broad_phase_observations: usize,
}

struct CollectedParticleObservations {
    particles: Vec<ParticleObservation>,
    contacts: Vec<ParticleContactObservation>,
    body_contacts: Vec<ParticleBodyContactObservation>,
    statistics: Vec<ParticleSystemStatistics>,
    world_statistics: ParticleWorldStatistics,
}

impl World {
    /// Collects one owned, bounded renderer-neutral observation.
    ///
    /// Counts and tree metrics are exact. Current rigid contacts preserve
    /// manager order; particle records preserve newest-first system order and
    /// each system's stored contact order; fixture-child AABBs preserve
    /// newest-first body and fixture order. Every storage coordinate is
    /// translated to a stable public identity before it crosses this boundary.
    ///
    /// # Errors
    ///
    /// Returns a typed capacity error before output allocation when any
    /// reviewed collection limit would be exceeded. An invariant error names
    /// only the bounded semantic category that could not be translated.
    pub fn world_observation(
        &self,
        limits: WorldObservationLimits,
    ) -> Result<WorldObservation, WorldObservationError> {
        let diagnostics = self.world_diagnostics();
        let counts = self.preflight_observation(diagnostics, limits)?;
        let (bodies, fixtures) = self.collect_body_fixture_observations()?;
        let joints = self.collect_joint_observations()?;
        let contacts = self
            .contact_manager
            .contacts()
            .iter()
            .map(|contact| ContactObservation::from_snapshot(&contact.snapshot()))
            .collect();
        let particles = self.collect_particle_observations(&counts)?;
        let broad_phase_observations =
            self.collect_broad_phase_observations(counts.broad_phase_observations)?;

        Ok(WorldObservation {
            diagnostics,
            bodies,
            fixtures,
            joints,
            particles: particles.particles,
            contacts,
            particle_contacts: particles.contacts,
            particle_body_contacts: particles.body_contacts,
            broad_phase_observations,
            particle_statistics: particles.statistics,
            particle_world_statistics: particles.world_statistics,
        })
    }

    fn preflight_observation(
        &self,
        diagnostics: WorldDiagnostics,
        limits: WorldObservationLimits,
    ) -> Result<ObservationCounts, WorldObservationError> {
        check_collection_bound(
            WorldObservationResource::Bodies,
            diagnostics.body_count(),
            REVIEWED_MAX_BODIES,
        )?;
        check_collection_bound(
            WorldObservationResource::Fixtures,
            diagnostics.fixture_count(),
            REVIEWED_MAX_FIXTURES,
        )?;
        check_collection_bound(
            WorldObservationResource::Joints,
            diagnostics.joint_count(),
            REVIEWED_MAX_JOINTS,
        )?;
        check_collection_bound(
            WorldObservationResource::Contacts,
            diagnostics.contact_count(),
            limits.contacts,
        )?;
        check_collection_bound(
            WorldObservationResource::BroadPhaseObservations,
            diagnostics.proxy_count(),
            limits.broad_phase_observations,
        )?;
        check_collection_bound(
            WorldObservationResource::ParticleSystems,
            self.particle_system_order.len(),
            limits.particle_systems,
        )?;

        let mut particle_count = 0_usize;
        let mut particle_contact_count = 0_usize;
        let mut particle_body_contact_count = 0_usize;
        for system in &self.particle_system_order {
            let record = self.particle_systems.get(*system).map_err(|_error| {
                WorldObservationError::InvalidState {
                    resource: WorldObservationResource::ParticleSystems,
                }
            })?;
            particle_count = checked_add_resource(
                WorldObservationResource::Particles,
                particle_count,
                record.storage.len(),
            )?;
            particle_contact_count = checked_add_resource(
                WorldObservationResource::ParticleContacts,
                particle_contact_count,
                record.storage.particle_contacts().len(),
            )?;
            let view = ParticleSystemView::new(&record.storage);
            particle_body_contact_count = checked_add_resource(
                WorldObservationResource::ParticleBodyContacts,
                particle_body_contact_count,
                view.body_contacts().len(),
            )?;
        }
        check_collection_bound(
            WorldObservationResource::Particles,
            particle_count,
            limits.particles,
        )?;
        check_collection_bound(
            WorldObservationResource::ParticleContacts,
            particle_contact_count,
            limits.particle_contacts,
        )?;
        check_collection_bound(
            WorldObservationResource::ParticleBodyContacts,
            particle_body_contact_count,
            limits.particle_body_contacts,
        )?;

        Ok(ObservationCounts {
            particle_contacts: particle_contact_count,
            particle_body_contacts: particle_body_contact_count,
            broad_phase_observations: diagnostics.proxy_count(),
        })
    }

    fn collect_particle_observations(
        &self,
        counts: &ObservationCounts,
    ) -> Result<CollectedParticleObservations, WorldObservationError> {
        let mut contacts = Vec::with_capacity(counts.particle_contacts);
        let mut body_contacts = Vec::with_capacity(counts.particle_body_contacts);
        let mut particles = Vec::new();
        let mut particle_statistics = Vec::with_capacity(self.particle_system_order.len());
        let mut particle_world_statistics = ParticleWorldStatistics::default();
        for system in &self.particle_system_order {
            let record = self.particle_systems.get(*system).map_err(|_error| {
                WorldObservationError::InvalidState {
                    resource: WorldObservationResource::ParticleSystems,
                }
            })?;
            let view = ParticleSystemView::new(&record.storage);
            let maybe_colors = view.maybe_colors();
            particles.extend(
                view.particle_ids()
                    .iter()
                    .enumerate()
                    .map(|(index, particle)| ParticleObservation {
                        system: *system,
                        particle: *particle,
                        position: view.positions()[index],
                        radius: record.definition.radius(),
                        color: maybe_colors
                            .map_or_else(ParticleColor::default, |colors| colors[index]),
                    }),
            );
            contacts.extend(
                view.particle_contacts()
                    .map(|contact| ParticleContactObservation {
                        system: *system,
                        particles: contact.particles(),
                        flags: contact.flags(),
                        weight: contact.weight(),
                        normal: contact.normal(),
                    }),
            );
            body_contacts.extend(view.body_contacts().map(|contact| {
                ParticleBodyContactObservation {
                    system: *system,
                    particle: contact.particle(),
                    body: contact.body(),
                    fixture: contact.fixture(),
                    weight: contact.weight(),
                    normal: contact.normal(),
                    mass: contact.mass(),
                }
            }));
            let statistics = ParticleSystemStatistics::from_storage(
                &record.storage,
                record.definition,
                record.groups.len(),
            );
            particle_world_statistics.include(&statistics);
            particle_statistics.push(statistics);
        }

        Ok(CollectedParticleObservations {
            particles,
            contacts,
            body_contacts,
            statistics: particle_statistics,
            world_statistics: particle_world_statistics,
        })
    }

    fn collect_body_fixture_observations(
        &self,
    ) -> Result<(Vec<BodyObservation>, Vec<FixtureObservation>), WorldObservationError> {
        let mut bodies = Vec::with_capacity(self.body_order.len());
        let mut fixtures = Vec::new();
        for body_id in &self.body_order {
            let body = self.bodies.get(*body_id).map_err(|_error| {
                WorldObservationError::InvalidState {
                    resource: WorldObservationResource::Bodies,
                }
            })?;
            bodies.push(BodyObservation {
                id: *body_id,
                snapshot: body.state.snapshot(),
            });
            for fixture_id in &body.fixtures {
                let fixture = self.fixtures.get(*fixture_id).map_err(|_error| {
                    WorldObservationError::InvalidState {
                        resource: WorldObservationResource::Fixtures,
                    }
                })?;
                fixtures.push(FixtureObservation {
                    id: *fixture_id,
                    body: *body_id,
                    snapshot: fixture.definition.snapshot(),
                });
            }
        }
        Ok((bodies, fixtures))
    }

    fn collect_joint_observations(&self) -> Result<Vec<JointObservation>, WorldObservationError> {
        let mut ordered = self.joints.iter().collect::<Vec<_>>();
        ordered.sort_by_key(|(_id, joint)| std::cmp::Reverse(joint.diagnostic_id));
        ordered
            .into_iter()
            .map(|(id, _record)| {
                self.joint_snapshot(id)
                    .map(|snapshot| JointObservation { id, snapshot })
                    .map_err(|_error| WorldObservationError::InvalidState {
                        resource: WorldObservationResource::Joints,
                    })
            })
            .collect()
    }

    fn collect_broad_phase_observations(
        &self,
        expected_count: usize,
    ) -> Result<Vec<BroadPhaseObservation>, WorldObservationError> {
        let mut broad_phase_observations = Vec::with_capacity(expected_count);
        for body_id in &self.body_order {
            let body = self.bodies.get(*body_id).map_err(|_error| {
                WorldObservationError::InvalidState {
                    resource: WorldObservationResource::BroadPhaseObservations,
                }
            })?;
            if !body.state.snapshot().is_active() {
                continue;
            }
            let transform = body.state.transform();
            for fixture_id in &body.fixtures {
                let fixture = self.fixtures.get(*fixture_id).map_err(|_error| {
                    WorldObservationError::InvalidState {
                        resource: WorldObservationResource::BroadPhaseObservations,
                    }
                })?;
                let shape = fixture.definition.shape();
                for requested_child in 0..shape.child_count() {
                    let child_index = shape.child_index(requested_child).map_err(|_error| {
                        WorldObservationError::InvalidState {
                            resource: WorldObservationResource::BroadPhaseObservations,
                        }
                    })?;
                    let aabb = shape
                        .compute_aabb(transform, child_index)
                        .map_err(|_error| WorldObservationError::InvalidState {
                            resource: WorldObservationResource::BroadPhaseObservations,
                        })?;
                    broad_phase_observations.push(BroadPhaseObservation {
                        body: *body_id,
                        fixture: *fixture_id,
                        child_index,
                        aabb,
                    });
                }
            }
        }
        if broad_phase_observations.len() != expected_count {
            return Err(WorldObservationError::InvalidState {
                resource: WorldObservationResource::BroadPhaseObservations,
            });
        }
        Ok(broad_phase_observations)
    }
}

fn check_requested_limit(
    resource: WorldObservationResource,
    requested: usize,
    maximum: usize,
) -> Result<(), WorldObservationLimitError> {
    if requested > maximum {
        return Err(WorldObservationLimitError {
            resource,
            requested,
            maximum,
        });
    }
    Ok(())
}

fn check_collection_bound(
    resource: WorldObservationResource,
    count: usize,
    limit: usize,
) -> Result<(), WorldObservationError> {
    if count > limit {
        return Err(WorldObservationError::CapacityExceeded { resource, limit });
    }
    Ok(())
}

fn checked_add_resource(
    resource: WorldObservationResource,
    current: usize,
    additional: usize,
) -> Result<usize, WorldObservationError> {
    current
        .checked_add(additional)
        .ok_or(WorldObservationError::InvalidState { resource })
}
