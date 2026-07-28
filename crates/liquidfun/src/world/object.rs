use std::error::Error;
use std::fmt;

use super::joint::JointRecord;
use crate::arena::Arena;
use crate::identity::WorldKey;
use crate::{
    ArenaInsertError, BodyId, FixtureId, HandleError, JointId, ObjectKind, ParticleGroupId,
    ParticleId, ParticleSystemId, WorldKeyError,
};

use super::body::BodyActivationError;
use super::body::{
    AggregateMassError, BodyControlError, BodyDef, BodyMassData, BodyMassMutationError,
    BodyMassResetError, BodySnapshot, BodyState, BodyTransformError, BodyType, BodyTypeChangeError,
    WakePolicy,
};
use super::config::{StepTiming, WorldConfiguration, WorldConfigurationError};
#[cfg(feature = "differential-internals")]
use super::contact::ContactTransition;
use super::contact_manager::ContactManager;
use super::contact_solver::{ContactImpulseSolution, ContactSolve, ContactSolveFailure};
use super::continuous::ContinuousStepState;
use super::fixture::{
    FixtureBoundsError, FixtureDef, FixtureDestructionError, FixtureMutationError,
    WorldFixtureSnapshot,
};
use super::island::{
    IslandBuildError, IslandLimits, IslandSolution, IslandSolveParameters, SolveFailureInjection,
    build_islands, solve_islands,
};
use super::joint::solver::JointImpulseSolution;
use super::proxy::{FixtureProxies, FixtureProxy, PreparedFixtureBounds, PreparedSynchronization};
use super::step::{CollisionDecisionHook, ContactHookRun, LifecycleEvent, StepError, StepState};
use crate::collision::{
    BroadPhase, ChildIndex, CollisionError, FilterData, MassData, RayCastHit, RayCastInput,
};
use crate::math::Vec2;
use crate::particle::storage::{ParticleSnapshot as StorageParticleSnapshot, ParticleStorage};
use crate::particle::{ParticleBufferMode, ParticleBufferTeardown, ParticleSystemDef};

mod body_object;
mod fixture_object;
mod lifecycle;
mod report;
mod solver;
#[cfg(test)]
mod tests;
pub use report::{DestructionReport, MutationReport};

#[cfg(test)]
use super::fixture::test_fixture_definition;

#[derive(Debug, Clone)]
pub(super) struct Body {
    pub(super) diagnostic_id: u64,
    pub(super) state: BodyState,
    pub(super) fixtures: Vec<FixtureId>,
    pub(super) joints: Vec<JointId>,
    pub(super) contacts: Vec<u64>,
    pub(super) pending_contact_destruction: bool,
    pub(super) pending_wake: bool,
}

#[derive(Debug, Clone)]
pub(super) struct Fixture {
    pub(super) diagnostic_id: u64,
    pub(super) body: BodyId,
    pub(super) definition: FixtureDef,
    pub(super) proxies: FixtureProxies,
    pub(super) contacts: Vec<u64>,
    pub(super) pending_refilter: bool,
}

#[derive(Clone)]
pub(super) struct ParticleSystem {
    pub(super) diagnostic_id: u64,
    pub(super) definition: ParticleSystemDef,
    pub(super) groups: Vec<ParticleGroupId>,
    pub(super) storage: ParticleStorage,
    pub(super) lifetime: crate::particle::lifetime::ParticleLifetimeState,
    pub(super) timestamp: u32,
}

#[derive(Debug, Clone)]
pub(super) struct ParticleGroup {
    pub(super) diagnostic_id: u64,
    pub(super) system: ParticleSystemId,
}

struct WorldStepCandidate {
    body_states: Vec<(BodyId, BodyState)>,
    contact_impulses: Vec<ContactImpulseSolution>,
    joint_impulses: Vec<JointImpulseSolution>,
    contact_solves: Vec<ContactSolve>,
    synchronizations: Vec<(FixtureId, PreparedSynchronization)>,
    timing: StepTiming,
}

fn contact_solve_build_error(error: IslandBuildError) -> ContactSolveFailure {
    match error {
        IslandBuildError::CapacityExceeded { resource, limit } => {
            ContactSolveFailure::CapacityExceeded { resource, limit }
        }
        IslandBuildError::InvalidGraph => ContactSolveFailure::UnsupportedTopology,
    }
}

fn maybe_solution_body_state(solutions: &[IslandSolution], body_id: BodyId) -> Option<BodyState> {
    for solution in solutions {
        let maybe_index = solution
            .body_ids
            .iter()
            .position(|candidate| *candidate == body_id);
        if let Some(index) = maybe_index {
            return solution.body_states.get(index).copied();
        }
    }
    None
}

#[derive(Debug)]
struct ParticleSystemDestructionTransaction {
    groups: Vec<ParticleGroupId>,
    particles: Vec<StorageParticleSnapshot>,
    root_snapshot: ObjectSnapshot,
}

/// A failure while creating a world-owned object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CreateObjectError {
    /// The world is currently inside its locked step transaction.
    WorldLocked,
    /// A referenced owner or endpoint does not belong to this world or is no longer live.
    InvalidHandle(HandleError),
    /// The arena for the new object cannot accept another entry.
    Arena(ArenaInsertError),
    /// A fixture child cannot be represented in broad-phase coordinates.
    InvalidFixtureBounds(FixtureBoundsError),
    /// Fixture density produces non-finite shape mass properties.
    InvalidFixtureMass,
    /// The complete prospective fixture aggregate is invalid.
    InvalidAggregateMass(AggregateMassError),
    /// Particle lifetime quantization cannot represent the requested value.
    InvalidParticleLifetime(crate::ParticleLifetimeError),
    /// Bounded source sampling could not produce a valid complete particle set.
    InvalidParticleGroupSampling,
    /// Pair or triad generation could not produce a valid complete topology.
    InvalidParticleGroupTopology,
    /// The application-owned association table could not reserve its commit slot.
    AssociationCapacityExceeded,
}

impl fmt::Display for CreateObjectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorldLocked => formatter.write_str("world is locked by an active step"),
            Self::InvalidHandle(error) => write!(formatter, "invalid related handle: {error}"),
            Self::Arena(error) => write!(formatter, "could not store object: {error}"),
            Self::InvalidFixtureBounds(error) => {
                write!(formatter, "invalid fixture bounds: {error}")
            }
            Self::InvalidFixtureMass => {
                formatter.write_str("fixture density produces invalid mass properties")
            }
            Self::InvalidAggregateMass(error) => {
                write!(formatter, "invalid aggregate body mass: {error}")
            }
            Self::InvalidParticleLifetime(error) => {
                write!(formatter, "invalid particle lifetime: {error}")
            }
            Self::InvalidParticleGroupSampling => {
                formatter.write_str("particle-group sampling failed")
            }
            Self::InvalidParticleGroupTopology => {
                formatter.write_str("particle-group topology generation failed")
            }
            Self::AssociationCapacityExceeded => {
                formatter.write_str("particle-group association capacity is exhausted")
            }
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

impl From<FixtureBoundsError> for CreateObjectError {
    fn from(error: FixtureBoundsError) -> Self {
        Self::InvalidFixtureBounds(error)
    }
}

impl From<AggregateMassError> for CreateObjectError {
    fn from(error: AggregateMassError) -> Self {
        Self::InvalidAggregateMass(error)
    }
}

impl From<crate::ParticleLifetimeError> for CreateObjectError {
    fn from(error: crate::ParticleLifetimeError) -> Self {
        Self::InvalidParticleLifetime(error)
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
    /// A finite particle lifetime reached its quantized expiration.
    ParticleExpiration,
    /// A body destruction invalidated an attached object.
    BodyCascade {
        /// Body whose destruction caused this invalidation.
        body: BodyId,
    },
    /// Destruction of a source joint invalidated a dependent gear first.
    GearDependencyCascade {
        /// Source joint whose destruction caused the gear invalidation.
        source: JointId,
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
        /// Checked semantic body state captured before invalidation.
        state: BodySnapshot,
        /// Attached fixtures in occurrence order.
        fixtures: Vec<FixtureId>,
        /// Attached joints in occurrence order.
        joints: Vec<JointId>,
    },
    /// The body that owned a fixture.
    Fixture {
        /// Owning body at destruction time.
        body: BodyId,
        /// Checked semantic fixture state captured before invalidation.
        state: WorldFixtureSnapshot,
    },
    /// The two body endpoints of a joint.
    Joint {
        /// Joint endpoints at destruction time.
        bodies: [BodyId; 2],
        /// Gear source identities, when the destroyed joint was a gear.
        maybe_gear_dependencies: Option<[JointId; 2]>,
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
    pub(super) destroyed: DestroyedId,
    pub(super) diagnostic_id: u64,
    pub(super) cause: DestructionCause,
    pub(super) snapshot: ObjectSnapshot,
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
    pub(super) scope_key: WorldKey,
    pub(super) bodies: Arena<Body, BodyId>,
    pub(super) body_order: Vec<BodyId>,
    pub(super) fixtures: Arena<Fixture, FixtureId>,
    pub(super) joints: Arena<JointRecord, JointId>,
    pub(super) particle_systems: Arena<ParticleSystem, ParticleSystemId>,
    pub(super) particle_system_order: Vec<ParticleSystemId>,
    pub(super) particle_groups: Arena<ParticleGroup, ParticleGroupId>,
    pub(super) broad_phase: BroadPhase<FixtureProxy>,
    pub(super) contact_manager: ContactManager,
    pub(super) continuous_step_state: ContinuousStepState,
    next_diagnostic_id: Option<u64>,
    pub(super) step_state: StepState,
    pub(super) configuration: WorldConfiguration,
}

fn new_world_broad_phase() -> BroadPhase<FixtureProxy> {
    BroadPhase::new().expect("a process cannot exhaust broad-phase identities after one world key")
}

fn remove_occurrence<T: PartialEq>(items: &mut Vec<T>, target: &T) {
    let position = items
        .iter()
        .position(|item| item == target)
        .expect("bidirectional adjacency contains matching occurrence");
    items.remove(position);
}
