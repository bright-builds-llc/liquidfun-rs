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
#[cfg(test)]
use super::step::{NoDecisionHook, StepLimits};
use crate::collision::{
    BroadPhase, ChildIndex, CollisionError, FilterData, MassData, RayCastHit, RayCastInput,
};
use crate::math::Vec2;
use crate::particle::storage::{ParticleSnapshot as StorageParticleSnapshot, ParticleStorage};
use crate::particle::{ParticleBufferMode, ParticleBufferTeardown, ParticleSystemDef};

mod report;
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

#[derive(Debug)]
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
}

impl fmt::Display for CreateObjectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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

impl World {
    /// Creates an empty world with a process-unique identity scope.
    ///
    /// # Errors
    ///
    /// Returns [`WorldKeyError::Exhausted`] if process-unique world identities are exhausted.
    pub fn new() -> Result<Self, WorldKeyError> {
        let world = WorldKey::fresh()?;
        Ok(Self {
            scope_key: world,
            bodies: Arena::new(world, usize::MAX),
            body_order: Vec::new(),
            fixtures: Arena::new(world, usize::MAX),
            joints: Arena::new(world, usize::MAX),
            particle_systems: Arena::new(world, usize::MAX),
            particle_system_order: Vec::new(),
            particle_groups: Arena::new(world, usize::MAX),
            broad_phase: new_world_broad_phase(),
            contact_manager: ContactManager::new(),
            continuous_step_state: ContinuousStepState::new(),
            next_diagnostic_id: Some(1),
            step_state: StepState::new(),
            configuration: WorldConfiguration::default(),
        })
    }

    pub(super) fn allocate_diagnostic_id(&mut self) -> Result<u64, ArenaInsertError> {
        let Some(id) = self.next_diagnostic_id else {
            return Err(ArenaInsertError::DiagnosticIdExhausted);
        };
        self.next_diagnostic_id = id.checked_add(1);
        Ok(id)
    }

    #[cfg(test)]
    fn set_next_diagnostic_id_for_test(&mut self, next: u64) {
        self.next_diagnostic_id = Some(next);
    }

    /// Creates a body from a reusable checked definition.
    ///
    /// # Errors
    ///
    /// Returns an arena error if body storage is exhausted.
    pub fn create_body(&mut self, definition: &BodyDef) -> Result<BodyId, ArenaInsertError> {
        self.ensure_not_poisoned_for_insert()?;
        let diagnostic_id = self.allocate_diagnostic_id()?;
        let body = self.bodies.insert(Body {
            diagnostic_id,
            state: BodyState::from_definition(definition),
            fixtures: Vec::new(),
            joints: Vec::new(),
            contacts: Vec::new(),
            pending_contact_destruction: false,
            pending_wake: false,
        })?;
        self.body_order.insert(0, body);
        self.debug_assert_body_order_invariant();
        Ok(body)
    }

    /// Returns an owned semantic snapshot of a live body.
    ///
    /// # Errors
    ///
    /// Returns a handle error when `body` is foreign, stale, or destroyed.
    pub fn body_snapshot(&self, body: BodyId) -> Result<BodySnapshot, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        self.bodies.get(body).map(|record| record.state.snapshot())
    }

    /// Clears accumulated force and torque from every live body.
    ///
    /// Use this after an application-managed sequence of sub-steps when
    /// automatic force clearing is disabled.
    ///
    /// # Errors
    ///
    /// Returns a typed no-effect error for a poisoned or locked world.
    pub fn clear_forces(&mut self) -> Result<(), WorldConfigurationError> {
        self.ensure_configuration_mutable()?;
        self.clear_force_accumulators();
        Ok(())
    }

    /// Sets a live body's linear velocity.
    ///
    /// A static body accepts this call without changing state. A nonzero
    /// velocity wakes a non-static body; zero velocity preserves its wake state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle or non-finite velocity.
    pub fn set_body_linear_velocity(
        &mut self,
        body: BodyId,
        velocity: Vec2,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| state.candidate_set_linear_velocity(velocity))
    }

    /// Sets a live body's angular velocity.
    ///
    /// A static body accepts this call without changing state. A nonzero
    /// velocity wakes a non-static body; zero velocity preserves its wake state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle or non-finite velocity.
    pub fn set_body_angular_velocity(
        &mut self,
        body: BodyId,
        angular_velocity: f32,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| {
            state.candidate_set_angular_velocity(angular_velocity)
        })
    }

    /// Applies force at a world point using the requested wake policy.
    ///
    /// Static and kinematic bodies, plus asleep bodies under
    /// [`WakePolicy::PreserveSleep`], accept this call without changing state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle, invalid input, or
    /// non-finite derived accumulation.
    pub fn apply_body_force(
        &mut self,
        body: BodyId,
        force: Vec2,
        point: Vec2,
        wake_policy: WakePolicy,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| {
            state.candidate_apply_force(force, point, wake_policy)
        })
    }

    /// Applies force at a body's center of mass using the requested wake policy.
    ///
    /// Static and kinematic bodies, plus asleep bodies under
    /// [`WakePolicy::PreserveSleep`], accept this call without changing state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle, invalid input, or
    /// non-finite force accumulation.
    pub fn apply_body_force_to_center(
        &mut self,
        body: BodyId,
        force: Vec2,
        wake_policy: WakePolicy,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| {
            state.candidate_apply_force_to_center(force, wake_policy)
        })
    }

    /// Applies torque using the requested wake policy.
    ///
    /// Static and kinematic bodies, plus asleep bodies under
    /// [`WakePolicy::PreserveSleep`], accept this call without changing state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle, invalid input, or
    /// non-finite torque accumulation.
    pub fn apply_body_torque(
        &mut self,
        body: BodyId,
        torque: f32,
        wake_policy: WakePolicy,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| {
            state.candidate_apply_torque(torque, wake_policy)
        })
    }

    /// Applies linear impulse at a world point using the requested wake policy.
    ///
    /// Static and kinematic bodies, plus asleep bodies under
    /// [`WakePolicy::PreserveSleep`], accept this call without changing state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle, invalid input, or
    /// non-finite derived velocity.
    pub fn apply_body_linear_impulse(
        &mut self,
        body: BodyId,
        impulse: Vec2,
        point: Vec2,
        wake_policy: WakePolicy,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| {
            state.candidate_apply_linear_impulse(impulse, point, wake_policy)
        })
    }

    /// Applies linear impulse at a body's center using the requested wake policy.
    ///
    /// Static and kinematic bodies, plus asleep bodies under
    /// [`WakePolicy::PreserveSleep`], accept this call without changing state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle, invalid input, or
    /// non-finite derived velocity.
    pub fn apply_body_linear_impulse_to_center(
        &mut self,
        body: BodyId,
        impulse: Vec2,
        wake_policy: WakePolicy,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| {
            state.candidate_apply_linear_impulse_to_center(impulse, wake_policy)
        })
    }

    /// Applies angular impulse using the requested wake policy.
    ///
    /// Static and kinematic bodies, plus asleep bodies under
    /// [`WakePolicy::PreserveSleep`], accept this call without changing state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle, invalid input, or
    /// non-finite derived angular velocity.
    pub fn apply_body_angular_impulse(
        &mut self,
        body: BodyId,
        impulse: f32,
        wake_policy: WakePolicy,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| {
            state.candidate_apply_angular_impulse(impulse, wake_policy)
        })
    }

    /// Changes whether a live body is awake.
    ///
    /// Sleeping clears velocity, accumulated force, accumulated torque, and
    /// sleep time atomically.
    ///
    /// # Errors
    ///
    /// Returns a typed error when `body` is foreign, stale, destroyed, or its
    /// world is poisoned.
    pub fn set_body_awake(&mut self, body: BodyId, awake: bool) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| Ok(state.candidate_set_awake(awake)))
    }

    /// Changes whether a live body may sleep automatically.
    ///
    /// Disabling sleep wakes the body immediately.
    ///
    /// # Errors
    ///
    /// Returns a typed error when `body` is foreign, stale, destroyed, or its
    /// world is poisoned.
    pub fn set_body_sleeping_allowed(
        &mut self,
        body: BodyId,
        sleeping_allowed: bool,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| {
            Ok(state.candidate_set_sleeping_allowed(sleeping_allowed))
        })
    }

    /// Sets a live body's linear damping without changing wake state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle or non-finite or negative damping.
    pub fn set_body_linear_damping(
        &mut self,
        body: BodyId,
        damping: f32,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| state.candidate_set_linear_damping(damping))
    }

    /// Sets a live body's angular damping without changing wake state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle or non-finite or negative damping.
    pub fn set_body_angular_damping(
        &mut self,
        body: BodyId,
        damping: f32,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| state.candidate_set_angular_damping(damping))
    }

    /// Sets a live body's gravity scale without changing wake state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle or non-finite gravity scale.
    pub fn set_body_gravity_scale(
        &mut self,
        body: BodyId,
        gravity_scale: f32,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| {
            state.candidate_set_gravity_scale(gravity_scale)
        })
    }

    /// Changes whether a live body receives continuous bullet treatment.
    ///
    /// # Errors
    ///
    /// Returns a typed error when `body` is foreign, stale, destroyed, or its
    /// world is poisoned.
    pub fn set_body_bullet(&mut self, body: BodyId, bullet: bool) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| Ok(state.candidate_set_bullet(bullet)))
    }

    /// Changes whether a live body has fixed rotation.
    ///
    /// A changed setting clears angular velocity and recomputes fixture-derived
    /// mass state before replacing the body state once.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle or invalid aggregate mass.
    pub fn set_body_fixed_rotation(
        &mut self,
        body: BodyId,
        fixed_rotation: bool,
    ) -> Result<(), BodyControlError> {
        self.ensure_not_poisoned_for_handle()?;
        let state = self.bodies.get(body)?.state;
        let fixture_mass_data = self.collect_fixture_mass_data(body, None, None);
        let candidate = state.candidate_set_fixed_rotation(fixed_rotation, &fixture_mass_data)?;
        self.body_mut_after_validation(body).state = candidate;
        self.invalidate_continuous_for_body(body);
        Ok(())
    }

    /// Changes the motion type of a live body.
    ///
    /// # Errors
    ///
    /// Returns [`BodyTypeChangeError::InvalidHandle`] when `body` is foreign, stale, or
    /// destroyed, or [`BodyTypeChangeError::InvalidAggregateMass`] when the complete target-type
    /// fixture aggregate is invalid. Either failure leaves body, contact, proxy, and fixture state
    /// unchanged.
    pub fn set_body_type(
        &mut self,
        body: BodyId,
        target_type: BodyType,
    ) -> Result<(), BodyTypeChangeError> {
        self.ensure_not_poisoned_for_handle()?;
        let record = self.bodies.get(body)?;
        if record.state.snapshot().body_type() == target_type {
            return Ok(());
        }
        let fixtures = record.fixtures.clone();
        let fixture_mass_data = self.collect_fixture_mass_data(body, None, None);
        let candidate = record
            .state
            .with_body_type_and_reset_mass_data(target_type, &fixture_mass_data)?;
        self.destroy_contacts_for_body(body);
        {
            let record = self.body_mut_after_validation(body);
            record.state = candidate;
            record.pending_contact_destruction = true;
            record.pending_wake = true;
        }
        self.touch_body_fixture_entries(body, &fixtures);
        self.invalidate_continuous_for_body(body);
        Ok(())
    }

    /// Changes a live body's position and angle after validating the complete candidate state.
    ///
    /// Accepted values retain their exact `f32` bits. A failure leaves the prior body state
    /// unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`BodyTransformError::InvalidHandle`] when `body` is invalid, or
    /// [`BodyTransformError::InvalidTransform`] when a position coordinate or angle is
    /// non-finite.
    pub fn set_body_transform(
        &mut self,
        body: BodyId,
        position: Vec2,
        angle: f32,
    ) -> Result<(), BodyTransformError> {
        self.ensure_not_poisoned_for_handle()?;
        let candidate = self
            .bodies
            .get(body)?
            .state
            .with_transform(position, angle)?;
        let record = self.bodies.get(body)?;
        let previous = record.state.transform();
        let fixtures = record.fixtures.clone();
        let active = record.state.snapshot().is_active();
        let synchronizations = if active {
            self.prepare_body_synchronizations(body, &fixtures, previous, candidate.transform())?
        } else {
            Vec::new()
        };
        self.apply_body_synchronizations(synchronizations);
        self.body_mut_after_validation(body).state = candidate;
        self.invalidate_continuous_for_body(body);
        Ok(())
    }

    /// Changes whether a live body participates in simulation.
    ///
    /// # Errors
    ///
    /// Returns a handle error without mutation when `body` is foreign, stale, or destroyed.
    pub fn set_body_active(
        &mut self,
        body: BodyId,
        active: bool,
    ) -> Result<(), BodyActivationError> {
        self.ensure_not_poisoned_for_handle()?;
        let record = self.bodies.get(body)?;
        if record.state.snapshot().is_active() == active {
            return Ok(());
        }
        let transform = record.state.transform();
        let fixtures = record.fixtures.clone();
        if active {
            let creations = self.prepare_body_fixture_creations(&fixtures, transform)?;
            self.create_body_fixture_entries(body, creations);
        } else {
            self.destroy_contacts_for_body(body);
            self.destroy_body_fixture_entries(body, fixtures);
        }
        let record = self.body_mut_after_validation(body);
        record.state.set_active(active);
        if !active {
            record.pending_contact_destruction = true;
        }
        self.invalidate_continuous_for_body(body);
        Ok(())
    }

    /// Creates a fixture attached to `body` by cloning a checked definition.
    ///
    /// # Errors
    ///
    /// Returns an error if `body` is invalid or fixture storage is exhausted.
    pub fn create_fixture(
        &mut self,
        body: BodyId,
        definition: &FixtureDef,
    ) -> Result<FixtureId, CreateObjectError> {
        self.ensure_not_poisoned_for_handle()?;
        let body_record = self.bodies.get(body)?;
        let maybe_prepared = if body_record.state.snapshot().is_active() {
            Some(FixtureProxies::prepare_creation(
                definition.shape(),
                body_record.state.transform(),
            )?)
        } else {
            None
        };
        let maybe_mass_state = if definition.density() > 0.0 {
            let fixture_mass = definition
                .shape()
                .compute_mass(definition.density())
                .map_err(|_error| CreateObjectError::InvalidFixtureMass)?;
            Some(self.prepare_body_mass_state(body, Some(fixture_mass), None)?)
        } else {
            None
        };
        let diagnostic_id = self.allocate_diagnostic_id()?;
        let fixture = self.fixtures.insert(Fixture {
            diagnostic_id,
            body,
            definition: definition.clone(),
            proxies: FixtureProxies::new(),
            contacts: Vec::new(),
            pending_refilter: false,
        })?;
        if let Some(prepared) = maybe_prepared {
            self.create_fixture_entries(fixture, body, prepared);
        }
        self.body_mut_after_validation(body)
            .fixtures
            .insert(0, fixture);
        if let Some(mass_state) = maybe_mass_state {
            self.body_mut_after_validation(body).state = mass_state;
        }
        self.invalidate_continuous_for_body(body);
        Ok(fixture)
    }

    /// Returns owned semantic state for a live fixture.
    ///
    /// # Errors
    ///
    /// Returns a handle error when `fixture` is foreign, stale, or destroyed.
    pub fn fixture_snapshot(
        &self,
        fixture: FixtureId,
    ) -> Result<WorldFixtureSnapshot, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        self.fixtures.get(fixture).map(|record| {
            WorldFixtureSnapshot::from_definition(
                record.body,
                &record.definition,
                record.proxies.len(),
            )
        })
    }

    pub(super) fn ray_cast_fixture_child(
        &self,
        fixture: FixtureId,
        child_index: ChildIndex,
        input: RayCastInput,
    ) -> Result<Option<RayCastHit>, CollisionError> {
        let fixture_record = self
            .fixtures
            .get(fixture)
            .expect("broad-phase fixture identities must remain live");
        let body = self
            .bodies
            .get(fixture_record.body)
            .expect("live fixture owners must remain live");
        fixture_record
            .definition
            .shape()
            .ray_cast(input, body.state.transform(), child_index)
    }

    /// Returns the number of shape children currently stored for broad-phase discovery.
    #[must_use]
    pub fn broad_phase_entry_count(&self) -> usize {
        self.broad_phase.proxy_count()
    }

    /// Returns the number of private automatic contact occurrences.
    #[must_use]
    pub fn contact_count(&self) -> usize {
        self.contact_manager.len()
    }

    /// Copies one body's complete Phase 6 diagnostic state.
    #[cfg(feature = "differential-internals")]
    #[doc(hidden)]
    pub fn rigid_body_diagnostic(
        &self,
        body: BodyId,
    ) -> Result<crate::rigid_differential::RigidBodyDiagnostic, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        self.bodies.get(body).map(|record| {
            crate::rigid_differential::RigidBodyDiagnostic::new(
                record.state.snapshot(),
                record.state.solver_linear(),
                record.state.solver_angular(),
            )
        })
    }

    /// Copies current manager occurrences in exact manager order.
    #[cfg(feature = "differential-internals")]
    #[doc(hidden)]
    #[must_use]
    pub fn rigid_contact_diagnostics(
        &self,
    ) -> Vec<crate::rigid_differential::RigidContactDiagnostic> {
        self.contact_manager.rigid_diagnostics()
    }

    /// Copies body identities in exact pinned newest-first world-list order.
    #[cfg(feature = "differential-internals")]
    #[doc(hidden)]
    #[must_use]
    pub fn rigid_body_order_diagnostic(&self) -> Vec<BodyId> {
        self.body_order.clone()
    }

    /// Builds owned evidence from the reviewed bounded ephemeral island graph.
    #[cfg(feature = "differential-internals")]
    #[doc(hidden)]
    pub fn rigid_island_diagnostics(
        &self,
    ) -> Result<
        Vec<crate::rigid_differential::RigidIslandDiagnostic>,
        crate::rigid_differential::RigidIslandBuildError,
    > {
        self.rigid_island_diagnostics_for_limits(IslandLimits::REVIEWED)
    }

    /// Builds owned island evidence with smaller diagnostic-only capacity limits.
    #[cfg(feature = "differential-internals")]
    #[doc(hidden)]
    pub fn rigid_island_diagnostics_with_limits(
        &self,
        max_bodies: usize,
        max_contacts: usize,
    ) -> Result<
        Vec<crate::rigid_differential::RigidIslandDiagnostic>,
        crate::rigid_differential::RigidIslandBuildError,
    > {
        self.rigid_island_diagnostics_for_limits(IslandLimits::diagnostic(max_bodies, max_contacts))
    }

    #[cfg(feature = "differential-internals")]
    fn rigid_island_diagnostics_for_limits(
        &self,
        limits: IslandLimits,
    ) -> Result<
        Vec<crate::rigid_differential::RigidIslandDiagnostic>,
        crate::rigid_differential::RigidIslandBuildError,
    > {
        let islands = build_islands(
            &self.body_order,
            &self.bodies,
            &self.joints,
            &self.contact_manager,
            limits,
        )
        .map_err(|error| match error {
            IslandBuildError::CapacityExceeded { resource, limit } => {
                crate::rigid_differential::RigidIslandBuildError::CapacityExceeded {
                    resource,
                    limit,
                }
            }
            IslandBuildError::InvalidGraph => {
                crate::rigid_differential::RigidIslandBuildError::InvalidGraph
            }
        })?;
        Ok(islands
            .into_iter()
            .map(|island| {
                let snapshots = island
                    .body_states
                    .iter()
                    .map(|state| state.snapshot())
                    .collect();
                let occurrences = island
                    .contact_indices
                    .iter()
                    .map(|index| self.contact_manager.contacts()[*index].ordinal + 1)
                    .collect();
                crate::rigid_differential::RigidIslandDiagnostic::new(
                    island.body_ids,
                    snapshots,
                    occurrences,
                    island.positions.len(),
                    island.velocities.len(),
                    island.joint_ids.len(),
                )
            })
            .collect())
    }

    /// Drains owned contact transitions produced outside [`World::step`].
    #[cfg(feature = "differential-internals")]
    #[doc(hidden)]
    pub fn rigid_drain_contact_transitions(&mut self) -> Vec<ContactTransition> {
        self.contact_manager.drain_transitions()
    }

    /// Recomputes a body's mass properties from its current fixtures in source list order.
    ///
    /// # Errors
    ///
    /// Returns a handle error without mutation when `body` is foreign, stale, or destroyed.
    pub fn reset_body_mass_data(&mut self, body: BodyId) -> Result<(), BodyMassResetError> {
        self.ensure_not_poisoned_for_handle()?;
        self.bodies.get(body)?;
        let mass_state = self.prepare_body_mass_state(body, None, None)?;
        self.body_mut_after_validation(body).state = mass_state;
        self.invalidate_continuous_for_body(body);
        Ok(())
    }

    /// Replaces current mass properties on a dynamic body.
    ///
    /// Static and kinematic bodies accept this operation as a source-compatible no-op.
    ///
    /// # Errors
    ///
    /// Returns a typed handle or derived-mass error without mutation.
    pub fn set_body_mass_data(
        &mut self,
        body: BodyId,
        data: BodyMassData,
    ) -> Result<(), BodyMassMutationError> {
        self.ensure_not_poisoned_for_handle()?;
        let prepared = self.bodies.get(body)?.state.with_custom_mass_data(data)?;
        self.body_mut_after_validation(body).state = prepared;
        self.invalidate_continuous_for_body(body);
        Ok(())
    }

    /// Changes fixture density without implicitly recomputing its body's mass.
    ///
    /// # Errors
    ///
    /// Returns a typed handle or checked material error without mutation.
    pub fn set_fixture_density(
        &mut self,
        fixture: FixtureId,
        density: f32,
    ) -> Result<(), FixtureMutationError> {
        self.ensure_not_poisoned_for_handle()?;
        let record = self.fixtures.get(fixture)?;
        if density > 0.0 && record.definition.shape().compute_mass(density).is_err() {
            return Err(FixtureMutationError::InvalidDerivedMass);
        }
        self.fixtures
            .get_mut(fixture)?
            .definition
            .set_density(density)?;
        Ok(())
    }

    /// Changes the friction used when future contacts are created.
    ///
    /// # Errors
    ///
    /// Returns a typed handle or checked material error without mutation.
    pub fn set_fixture_friction(
        &mut self,
        fixture: FixtureId,
        friction: f32,
    ) -> Result<(), FixtureMutationError> {
        self.ensure_not_poisoned_for_handle()?;
        self.fixtures
            .get_mut(fixture)?
            .definition
            .set_friction(friction)?;
        Ok(())
    }

    /// Changes the restitution used when future contacts are created.
    ///
    /// # Errors
    ///
    /// Returns a typed handle or checked material error without mutation.
    pub fn set_fixture_restitution(
        &mut self,
        fixture: FixtureId,
        restitution: f32,
    ) -> Result<(), FixtureMutationError> {
        self.ensure_not_poisoned_for_handle()?;
        self.fixtures
            .get_mut(fixture)?
            .definition
            .set_restitution(restitution)?;
        Ok(())
    }

    /// Changes whether a fixture reports overlap without collision response.
    ///
    /// A changed sensor state records the owning body's pending wake side effect.
    ///
    /// # Errors
    ///
    /// Returns a handle error without mutation when `fixture` is foreign, stale, or destroyed.
    pub fn set_fixture_sensor(
        &mut self,
        fixture: FixtureId,
        sensor: bool,
    ) -> Result<(), HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let record = self.fixtures.get(fixture)?;
        if record.definition.is_sensor() == sensor {
            return Ok(());
        }
        let body = record.body;
        self.fixtures
            .get_mut(fixture)?
            .definition
            .set_sensor(sensor);
        self.body_mut_after_validation(body).pending_wake = true;
        self.invalidate_continuous_for_body(body);
        Ok(())
    }

    /// Replaces collision filtering and touches every active broad-phase child.
    ///
    /// # Errors
    ///
    /// Returns a handle error without mutation when `fixture` is foreign, stale, or destroyed.
    pub fn set_fixture_filter(
        &mut self,
        fixture: FixtureId,
        filter: FilterData,
    ) -> Result<(), HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let body = self.fixtures.get(fixture)?.body;
        self.set_fixture_filter_after_validation(fixture, filter);
        self.invalidate_continuous_for_body(body);
        Ok(())
    }

    /// Creates a particle system.
    ///
    /// # Errors
    ///
    /// Returns an arena error if particle-system storage is exhausted.
    pub fn create_particle_system(&mut self) -> Result<ParticleSystemId, ArenaInsertError> {
        self.create_particle_system_with_def(&ParticleSystemDef::default())
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
        let diagnostic_id = self.allocate_diagnostic_id()?;
        let group = self.particle_groups.insert(ParticleGroup {
            diagnostic_id,
            system,
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
        self.create_particle_with_def(system, maybe_group, &crate::ParticleDef::default())
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

    /// Returns the number of live joints, including gear joints.
    #[must_use]
    pub fn joint_count(&self) -> usize {
        self.joints.iter().count()
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
        self.particle_snapshot(particle).is_ok()
    }

    /// Destroys a body and all attached joints and fixtures.
    ///
    /// # Errors
    ///
    /// Returns a handle error without mutation when `body` is foreign, stale, or destroyed.
    pub fn destroy_body(&mut self, body: BodyId) -> Result<DestructionReport, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let transition_checkpoint = self.contact_manager.transition_checkpoint();
        let root = self.bodies.get(body)?;
        let joints = root.joints.clone();
        let fixtures = root.fixtures.clone();
        let root_snapshot = ObjectSnapshot::Body {
            state: root.state.snapshot(),
            fixtures: fixtures.clone(),
            joints: joints.clone(),
        };
        let dependent_gears = self.collect_body_gear_dependents(&joints);
        let mut records =
            Vec::with_capacity(dependent_gears.len() + joints.len() + fixtures.len() + 1);
        let mut lifecycle = Vec::with_capacity(records.capacity() + root.contacts.len());

        for (gear, source) in &dependent_gears {
            let record = self.remove_joint(
                *gear,
                DestructionCause::GearDependencyCascade { source: *source },
            );
            lifecycle.push(LifecycleEvent::JointGoodbye(record.clone()));
            records.push(record);
        }

        for joint in joints {
            if dependent_gears.iter().any(|(gear, _source)| *gear == joint) {
                continue;
            }
            let record = self.remove_joint(joint, DestructionCause::BodyCascade { body });
            lifecycle.push(LifecycleEvent::JointGoodbye(record.clone()));
            records.push(record);
        }
        self.destroy_contacts_for_body(body);
        lifecycle.extend(
            self.contact_manager
                .drain_transitions_since(transition_checkpoint)
                .into_iter()
                .map(LifecycleEvent::ContactDestruction),
        );
        for fixture in fixtures {
            let record = self.remove_fixture(fixture, DestructionCause::BodyCascade { body }, None);
            lifecycle.extend(
                self.contact_manager
                    .drain_transitions()
                    .into_iter()
                    .map(LifecycleEvent::ContactDestruction),
            );
            lifecycle.push(LifecycleEvent::FixtureGoodbye(record.clone()));
            records.push(record);
        }
        let root = self.remove_body(body, DestructionCause::Explicit, root_snapshot);
        lifecycle.push(LifecycleEvent::Destruction(root.clone()));
        records.push(root);
        Ok(MutationReport::new(records, lifecycle))
    }

    /// Destroys one fixture after validating it before mutation.
    ///
    /// # Errors
    ///
    /// Returns [`FixtureDestructionError::InvalidHandle`] when `fixture` is foreign, stale, or
    /// destroyed, or [`FixtureDestructionError::InvalidAggregateMass`] when the complete
    /// remaining-fixture aggregate is invalid. Either failure leaves contacts, proxies, fixture
    /// storage, adjacency, and body mass state unchanged.
    pub fn destroy_fixture(
        &mut self,
        fixture: FixtureId,
    ) -> Result<DestructionReport, FixtureDestructionError> {
        self.ensure_not_poisoned_for_handle()?;
        let transition_checkpoint = self.contact_manager.transition_checkpoint();
        let body = self.fixtures.get(fixture)?.body;
        let candidate = self.prepare_body_mass_state(body, None, Some(fixture))?;
        let record = self.remove_fixture(fixture, DestructionCause::Explicit, Some(candidate));
        let mut lifecycle = self
            .contact_manager
            .drain_transitions_since(transition_checkpoint)
            .into_iter()
            .map(LifecycleEvent::ContactDestruction)
            .collect::<Vec<_>>();
        lifecycle.push(LifecycleEvent::Destruction(record.clone()));
        Ok(MutationReport::new(vec![record], lifecycle))
    }

    /// Destroys a particle system and all its groups and particles.
    ///
    /// # Errors
    ///
    /// Returns a handle error without mutation when `system` is foreign, stale, or destroyed.
    pub fn destroy_particle_system(
        &mut self,
        system: ParticleSystemId,
    ) -> Result<DestructionReport, HandleError> {
        self.destroy_particle_system_owned(system)
            .map(|(records, _removed)| records)
    }

    /// Destroys a particle system and returns its complete owned lane bundle.
    ///
    /// # Errors
    ///
    /// Returns a handle error without mutation when `system` is foreign, stale, or destroyed.
    pub fn destroy_particle_system_with_buffers(
        &mut self,
        system: ParticleSystemId,
    ) -> Result<ParticleBufferTeardown, HandleError> {
        let (report, removed) = self.destroy_particle_system_owned(system)?;
        let records = report.into_value();
        let capacity = removed.definition.capacity();
        let mode = if capacity.is_fixed() {
            ParticleBufferMode::Fixed {
                capacity: capacity.count(),
            }
        } else {
            ParticleBufferMode::Growable {
                initial_capacity: capacity.count(),
            }
        };
        let bundle = removed.storage.into_buffer_bundle(mode);
        Ok(ParticleBufferTeardown::new(records, bundle))
    }

    fn destroy_particle_system_owned(
        &mut self,
        system: ParticleSystemId,
    ) -> Result<(DestructionReport, ParticleSystem), HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        let transaction = self.capture_particle_system_destruction(system)?;
        let mut records =
            Vec::with_capacity(transaction.groups.len() + transaction.particles.len() + 1);
        let mut lifecycle = Vec::new();

        for group in transaction.groups {
            records.push(
                self.remove_particle_group(
                    group,
                    DestructionCause::ParticleSystemCascade { system },
                ),
            );
        }
        for snapshot in transaction.particles {
            let requested = snapshot
                .input
                .flags
                .contains(crate::ParticleFlags::DESTRUCTION_LISTENER);
            let record = Self::particle_destruction_record(
                snapshot,
                DestructionCause::ParticleSystemCascade { system },
            );
            if requested {
                lifecycle.push(LifecycleEvent::ParticleDestruction(record.clone()));
            }
            records.push(record);
        }
        let (root_record, removed) = self.remove_particle_system(
            system,
            DestructionCause::Explicit,
            transaction.root_snapshot,
        );
        lifecycle.push(LifecycleEvent::Destruction(root_record.clone()));
        records.push(root_record);
        Ok((MutationReport::new(records, lifecycle), removed))
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
        self.destroy_particle_now(particle)
    }

    fn capture_particle_system_destruction(
        &self,
        system: ParticleSystemId,
    ) -> Result<ParticleSystemDestructionTransaction, HandleError> {
        let root = self.particle_systems.get(system)?;
        let groups = root.groups.clone();
        let particles = root.storage.snapshots();
        let particle_ids = particles.iter().map(|snapshot| snapshot.id).collect();
        let root_snapshot = ObjectSnapshot::ParticleSystem {
            groups: groups.clone(),
            particles: particle_ids,
        };

        Ok(ParticleSystemDestructionTransaction {
            groups,
            particles,
            root_snapshot,
        })
    }

    pub(super) fn ensure_not_poisoned_for_handle(&self) -> Result<(), HandleError> {
        if self.step_state.is_poisoned() {
            return Err(HandleError::WorldPoisoned);
        }
        Ok(())
    }

    fn update_body_state(
        &mut self,
        body: BodyId,
        prepare: impl FnOnce(BodyState) -> Result<BodyState, BodyControlError>,
    ) -> Result<(), BodyControlError> {
        self.ensure_not_poisoned_for_handle()?;
        let state = self.bodies.get(body)?.state;
        let candidate = prepare(state)?;
        self.body_mut_after_validation(body).state = candidate;
        self.invalidate_continuous_for_body(body);
        Ok(())
    }

    fn invalidate_continuous_for_body(&mut self, body: BodyId) {
        self.continuous_step_state.invalidate();
        self.contact_manager.invalidate_toi_for_body(body);
    }

    pub(super) fn clear_force_accumulators(&mut self) {
        for body in self.bodies.values_mut() {
            body.state.clear_accumulated_forces();
        }
    }

    pub(super) fn ensure_not_poisoned_for_insert(&self) -> Result<(), ArenaInsertError> {
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
        remove_occurrence(&mut self.body_order, &body);
        self.debug_assert_body_order_invariant();
        DestructionRecord {
            destroyed: DestroyedId::Body(body),
            diagnostic_id: removed.diagnostic_id,
            cause,
            snapshot,
        }
    }

    fn remove_fixture(
        &mut self,
        fixture: FixtureId,
        cause: DestructionCause,
        maybe_mass_state: Option<BodyState>,
    ) -> DestructionRecord {
        self.destroy_contacts_for_fixture(fixture);
        let record = self
            .fixtures
            .get_mut(fixture)
            .expect("validated fixture adjacency remains live");
        let broad_phase_entry_count = record.proxies.len();
        record
            .proxies
            .destroy(&mut self.broad_phase, fixture, record.body);
        let removed = self
            .fixtures
            .remove(fixture)
            .expect("validated fixture adjacency remains live");
        remove_occurrence(
            &mut self.body_mut_after_validation(removed.body).fixtures,
            &fixture,
        );
        if let Some(mass_state) = maybe_mass_state {
            self.body_mut_after_validation(removed.body).state = mass_state;
        }
        self.invalidate_continuous_for_body(removed.body);
        DestructionRecord {
            destroyed: DestroyedId::Fixture(fixture),
            diagnostic_id: removed.diagnostic_id,
            cause,
            snapshot: ObjectSnapshot::Fixture {
                body: removed.body,
                state: WorldFixtureSnapshot::from_definition(
                    removed.body,
                    &removed.definition,
                    broad_phase_entry_count,
                ),
            },
        }
    }

    pub(super) fn remove_joint(
        &mut self,
        joint: JointId,
        cause: DestructionCause,
    ) -> DestructionRecord {
        let removed = self
            .joints
            .remove(joint)
            .expect("validated joint adjacency remains live");
        if let crate::JointDef::Gear(definition) = removed.definition {
            for dependency in definition.source_joints() {
                remove_occurrence(
                    &mut self
                        .joints
                        .get_mut(dependency)
                        .expect("live gear sources remain valid until dependent removal")
                        .reverse_gear_dependents,
                    &joint,
                );
            }
        }
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
                maybe_gear_dependencies: match removed.definition {
                    crate::JointDef::Gear(definition) => Some(definition.source_joints()),
                    _ => None,
                },
            },
        }
    }

    fn collect_body_gear_dependents(&self, joints: &[JointId]) -> Vec<(JointId, JointId)> {
        let mut dependents = Vec::new();
        for source in joints {
            let record = self
                .joints
                .get(*source)
                .expect("body joint adjacency contains only live joints");
            for gear in &record.reverse_gear_dependents {
                if dependents
                    .iter()
                    .any(|(existing, _source)| existing == gear)
                {
                    continue;
                }
                dependents.push((*gear, *source));
            }
        }
        dependents
    }

    fn remove_particle_system(
        &mut self,
        system: ParticleSystemId,
        cause: DestructionCause,
        snapshot: ObjectSnapshot,
    ) -> (DestructionRecord, ParticleSystem) {
        let removed = self
            .particle_systems
            .remove(system)
            .expect("validated destruction root and adjacency remain live");
        remove_occurrence(&mut self.particle_system_order, &system);
        (
            DestructionRecord {
                destroyed: DestroyedId::ParticleSystem(system),
                diagnostic_id: removed.diagnostic_id,
                cause,
                snapshot,
            },
            removed,
        )
    }

    fn remove_particle_group(
        &mut self,
        group: ParticleGroupId,
        cause: DestructionCause,
    ) -> DestructionRecord {
        let system = self
            .particle_groups
            .get(group)
            .expect("validated particle-group adjacency remains live")
            .system;
        let particles = self
            .system_mut_after_validation(system)
            .storage
            .clear_group(group)
            .expect("validated particle storage remains coherent during group removal");
        let removed = self
            .particle_groups
            .remove(group)
            .expect("validated particle-group adjacency remains live");
        remove_occurrence(
            &mut self.system_mut_after_validation(removed.system).groups,
            &group,
        );
        DestructionRecord {
            destroyed: DestroyedId::ParticleGroup(group),
            diagnostic_id: removed.diagnostic_id,
            cause,
            snapshot: ObjectSnapshot::ParticleGroup {
                system: removed.system,
                particles,
            },
        }
    }

    fn touch_body_fixture_entries(&mut self, body: BodyId, fixtures: &[FixtureId]) {
        for fixture in fixtures {
            let record = self
                .fixtures
                .get(*fixture)
                .expect("body fixture adjacency contains a live fixture");
            record.proxies.touch(&mut self.broad_phase, *fixture, body);
        }
    }

    fn prepare_body_mass_state(
        &self,
        body: BodyId,
        maybe_candidate: Option<MassData>,
        maybe_excluded_fixture: Option<FixtureId>,
    ) -> Result<BodyState, AggregateMassError> {
        let fixture_mass_data =
            self.collect_fixture_mass_data(body, maybe_candidate, maybe_excluded_fixture);
        self.bodies
            .get(body)
            .expect("validated body remains live during mass reset")
            .state
            .with_reset_mass_data(&fixture_mass_data)
    }

    fn collect_fixture_mass_data(
        &self,
        body: BodyId,
        maybe_candidate: Option<MassData>,
        maybe_excluded_fixture: Option<FixtureId>,
    ) -> Vec<MassData> {
        let fixture_ids = &self
            .bodies
            .get(body)
            .expect("validated body remains live during mass collection")
            .fixtures;
        let mut fixture_mass_data =
            Vec::with_capacity(fixture_ids.len() + usize::from(maybe_candidate.is_some()));
        if let Some(candidate) = maybe_candidate {
            fixture_mass_data.push(candidate);
        }
        fixture_mass_data.extend(fixture_ids.iter().filter_map(|fixture| {
            if Some(*fixture) == maybe_excluded_fixture {
                return None;
            }
            let definition = &self
                .fixtures
                .get(*fixture)
                .expect("body fixture adjacency contains a live fixture")
                .definition;
            if definition.density() == 0.0 {
                return None;
            }
            Some(
                definition
                    .shape()
                    .compute_mass(definition.density())
                    .expect("checked fixture shape and density produce valid mass data"),
            )
        }));
        fixture_mass_data
    }

    fn set_fixture_filter_after_validation(&mut self, fixture: FixtureId, filter: FilterData) {
        {
            let record = self
                .fixtures
                .get_mut(fixture)
                .expect("validated fixture remains live during refilter");
            record.definition.set_filter_data(filter);
            record.pending_refilter = true;
            record
                .proxies
                .set_filter(&mut self.broad_phase, fixture, record.body, filter);
        }
        self.contact_manager.flag_fixture_for_filtering(fixture);
    }

    #[cfg(test)]
    pub(super) fn find_new_contacts(&mut self) {
        let mut hook = NoDecisionHook;
        let mut hook_run = ContactHookRun::new(&mut hook, StepLimits::default());
        self.find_new_contacts_with_hook(&mut hook_run)
            .expect("default internal contact discovery remains within reviewed limits");
    }

    pub(super) fn find_new_contacts_with_hook<H: CollisionDecisionHook>(
        &mut self,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<(), StepError> {
        self.resolve_pending_body_wakes();
        self.contact_manager.find_new_contacts(
            &mut self.broad_phase,
            &mut self.bodies,
            &mut self.fixtures,
            &self.joints,
            hook_run,
        )
    }

    fn resolve_pending_body_wakes(&mut self) {
        for body_id in self.body_order.iter().copied() {
            let record = self
                .bodies
                .get_mut(body_id)
                .expect("source-ordered body remains live while resolving wake markers");
            if !record.pending_wake {
                continue;
            }
            record.state = record.state.candidate_set_awake(true);
            record.pending_wake = false;
        }
    }

    #[cfg(test)]
    pub(super) fn update_contacts(&mut self) {
        let mut hook = NoDecisionHook;
        let mut hook_run = ContactHookRun::new(&mut hook, StepLimits::default());
        self.update_contacts_with_hook(&mut hook_run)
            .expect("default contact hook remains within reviewed limits");
    }

    pub(super) fn update_contacts_with_hook<H: CollisionDecisionHook>(
        &mut self,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<(), StepError> {
        self.contact_manager.update_contacts(
            &self.broad_phase,
            &mut self.bodies,
            &mut self.fixtures,
            &self.joints,
            hook_run,
        )
    }

    pub(super) fn solve_contacts<H: CollisionDecisionHook>(
        &mut self,
        configuration: super::config::StepConfiguration,
        timing: StepTiming,
        maybe_failure_injection: Option<SolveFailureInjection>,
        contact_transitions: &[super::contact::ContactTransition],
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<Vec<ContactSolve>, StepError> {
        let candidate = self
            .prepare_world_step_candidate(configuration, timing, maybe_failure_injection)
            .map_err(|error| super::step::solver_step_error(error, contact_transitions))?;
        self.commit_world_step_candidate(candidate, hook_run)
    }

    #[allow(
        clippy::too_many_lines,
        reason = "the complete world step candidate must remain one prepare-before-commit transaction"
    )]
    fn prepare_world_step_candidate(
        &self,
        configuration: super::config::StepConfiguration,
        timing: StepTiming,
        maybe_failure_injection: Option<SolveFailureInjection>,
    ) -> Result<WorldStepCandidate, ContactSolveFailure> {
        let islands = build_islands(
            &self.body_order,
            &self.bodies,
            &self.joints,
            &self.contact_manager,
            IslandLimits::REVIEWED,
        )
        .map_err(contact_solve_build_error)?;
        let solutions = solve_islands(
            &islands,
            &self.contact_manager,
            &self.fixtures,
            &self.joints,
            IslandSolveParameters::new(
                self.gravity(),
                configuration,
                timing.time_step_ratio(),
                self.is_warm_starting_enabled(),
                maybe_failure_injection,
            ),
        )?;

        let mut body_states = Vec::new();
        body_states
            .try_reserve_exact(self.body_order.len())
            .map_err(|_| ContactSolveFailure::CapacityExceeded {
                resource: "world step body candidates",
                limit: self.body_order.len(),
            })?;
        for body_id in &self.body_order {
            if let Some(state) = maybe_solution_body_state(&solutions, *body_id) {
                body_states.push((*body_id, state));
            }
        }

        let contact_count = solutions
            .iter()
            .map(|solution| solution.contact_impulses.len())
            .sum();
        let joint_count = solutions
            .iter()
            .map(|solution| solution.joint_impulses.len())
            .sum();
        let mut contact_impulses = Vec::new();
        contact_impulses
            .try_reserve_exact(contact_count)
            .map_err(|_| ContactSolveFailure::CapacityExceeded {
                resource: "world step contact impulses",
                limit: contact_count,
            })?;
        let mut joint_impulses = Vec::new();
        joint_impulses.try_reserve_exact(joint_count).map_err(|_| {
            ContactSolveFailure::CapacityExceeded {
                resource: "world step joint impulses",
                limit: joint_count,
            }
        })?;
        for solution in solutions {
            contact_impulses.extend(solution.contact_impulses);
            joint_impulses.extend(solution.joint_impulses);
        }

        let mut contact_solves = Vec::new();
        contact_solves
            .try_reserve_exact(contact_impulses.len())
            .map_err(|_| ContactSolveFailure::CapacityExceeded {
                resource: "world step contact reports",
                limit: contact_impulses.len(),
            })?;
        for contact in &contact_impulses {
            let maybe_solve = self
                .contact_manager
                .maybe_staged_solve(contact.contact_index, &contact.impulses);
            let Some(solve) = maybe_solve else {
                return Err(ContactSolveFailure::UnsupportedTopology);
            };
            contact_solves.push(solve);
        }

        let mut synchronizations = Vec::new();
        for (body_id, state) in &body_states {
            let body = self
                .bodies
                .get(*body_id)
                .map_err(|_| ContactSolveFailure::UnsupportedTopology)?;
            if state.snapshot().body_type() == BodyType::Static || !state.snapshot().is_active() {
                continue;
            }
            #[cfg(feature = "differential-internals")]
            if let Some(SolveFailureInjection::ProxyBounds { fixture }) = maybe_failure_injection
                && body.fixtures.contains(&fixture)
            {
                return Err(ContactSolveFailure::InvalidProxyBounds);
            }
            let prepared = self
                .prepare_body_synchronizations(
                    *body_id,
                    &body.fixtures,
                    body.state.transform(),
                    state.transform(),
                )
                .map_err(|_error| ContactSolveFailure::InvalidProxyBounds)?;
            synchronizations.extend(prepared);
        }

        Ok(WorldStepCandidate {
            body_states,
            contact_impulses,
            joint_impulses,
            contact_solves,
            synchronizations,
            timing,
        })
    }

    fn commit_world_step_candidate<H: CollisionDecisionHook>(
        &mut self,
        candidate: WorldStepCandidate,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<Vec<ContactSolve>, StepError> {
        hook_run.ensure_lifecycle_capacity(candidate.contact_solves.len())?;
        for (body_id, state) in candidate.body_states {
            self.bodies
                .get_mut(body_id)
                .expect("staged island body remains live during commit")
                .state = state;
        }
        for contact in candidate.contact_impulses {
            self.contact_manager
                .commit_impulses(contact.contact_index, &contact.impulses);
        }
        for joint in candidate.joint_impulses {
            let record = self
                .joints
                .get_mut(joint.joint_id)
                .expect("staged island joint remains live during commit");
            record.runtime = joint.runtime;
        }
        for solve in &candidate.contact_solves {
            hook_run.record_discrete_solve(solve.clone())?;
        }
        self.apply_body_synchronizations(candidate.synchronizations);
        self.find_new_contacts_with_hook(hook_run)?;
        self.commit_step_timing(candidate.timing);
        Ok(candidate.contact_solves)
    }

    pub(super) fn preflight_contact_solver(&self) -> Result<(), ContactSolveFailure> {
        build_islands(
            &self.body_order,
            &self.bodies,
            &self.joints,
            &self.contact_manager,
            IslandLimits::REVIEWED,
        )
        .map(|_islands| ())
        .map_err(contact_solve_build_error)
    }

    #[cfg(test)]
    pub(super) fn set_body_solver_velocity_for_test(
        &mut self,
        body: BodyId,
        linear: Vec2,
        angular: f32,
    ) {
        self.bodies
            .get_mut(body)
            .expect("test body should remain live")
            .state
            .set_solver_motion(linear, angular);
    }

    #[cfg(test)]
    pub(super) fn body_solver_velocity_for_test(&self, body: BodyId) -> (Vec2, f32) {
        let state = self
            .bodies
            .get(body)
            .expect("test body should remain live")
            .state;
        (state.solver_linear(), state.solver_angular())
    }

    #[cfg(test)]
    pub(super) fn seed_first_contact_impulses_for_test(&mut self, normal: f32, tangent: f32) {
        self.contact_manager
            .seed_first_impulses_for_test(normal, tangent);
    }

    pub(super) fn destroy_contacts_for_body(&mut self, body: BodyId) {
        self.contact_manager
            .destroy_for_body(body, &mut self.bodies, &mut self.fixtures);
    }

    pub(super) fn destroy_contacts_for_fixture(&mut self, fixture: FixtureId) {
        self.contact_manager
            .destroy_for_fixture(fixture, &mut self.bodies, &mut self.fixtures);
    }

    pub(super) fn prepare_body_synchronizations(
        &self,
        body: BodyId,
        fixtures: &[FixtureId],
        previous: crate::math::Transform,
        current: crate::math::Transform,
    ) -> Result<Vec<(FixtureId, PreparedSynchronization)>, FixtureBoundsError> {
        fixtures
            .iter()
            .map(|fixture| {
                let record = self
                    .fixtures
                    .get(*fixture)
                    .expect("body fixture adjacency contains a live fixture");
                record
                    .proxies
                    .prepare_synchronization(
                        &self.broad_phase,
                        *fixture,
                        body,
                        record.definition.shape(),
                        previous,
                        current,
                    )
                    .map(|prepared| (*fixture, prepared))
            })
            .collect()
    }

    pub(super) fn apply_body_synchronizations(
        &mut self,
        synchronizations: Vec<(FixtureId, PreparedSynchronization)>,
    ) {
        for (fixture, prepared) in synchronizations {
            self.fixtures
                .get_mut(fixture)
                .expect("prepared fixture remains live during transform commit")
                .proxies
                .synchronize(&mut self.broad_phase, prepared);
        }
    }

    fn prepare_body_fixture_creations(
        &self,
        fixtures: &[FixtureId],
        transform: crate::math::Transform,
    ) -> Result<Vec<(FixtureId, PreparedFixtureBounds)>, FixtureBoundsError> {
        fixtures
            .iter()
            .map(|fixture| {
                let record = self
                    .fixtures
                    .get(*fixture)
                    .expect("body fixture adjacency contains a live fixture");
                FixtureProxies::prepare_creation(record.definition.shape(), transform)
                    .map(|prepared| (*fixture, prepared))
            })
            .collect()
    }

    fn create_body_fixture_entries(
        &mut self,
        body: BodyId,
        creations: Vec<(FixtureId, PreparedFixtureBounds)>,
    ) {
        for (fixture, prepared) in creations {
            self.create_fixture_entries(fixture, body, prepared);
        }
    }

    fn create_fixture_entries(
        &mut self,
        fixture: FixtureId,
        body: BodyId,
        prepared: PreparedFixtureBounds,
    ) {
        let record = self
            .fixtures
            .get_mut(fixture)
            .expect("prepared fixture remains live during entry creation");
        record.proxies.create(
            &mut self.broad_phase,
            fixture,
            body,
            record.definition.filter_data(),
            prepared,
        );
    }

    fn destroy_body_fixture_entries(&mut self, body: BodyId, fixtures: Vec<FixtureId>) {
        for fixture in fixtures {
            self.fixtures
                .get_mut(fixture)
                .expect("body fixture adjacency contains a live fixture")
                .proxies
                .destroy(&mut self.broad_phase, fixture, body);
        }
    }

    pub(super) fn body_mut_after_validation(&mut self, body: BodyId) -> &mut Body {
        self.bodies
            .get_mut(body)
            .expect("validated body remains live during one operation")
    }

    fn debug_assert_body_order_invariant(&self) {
        debug_assert_eq!(self.body_order.len(), self.bodies.iter().count());
        debug_assert!(
            self.body_order
                .iter()
                .all(|body| self.bodies.get(*body).is_ok())
        );
        debug_assert!(self.body_order.iter().enumerate().all(|(index, body)| {
            self.body_order[index + 1..]
                .iter()
                .all(|candidate| candidate != body)
        }));
    }

    pub(super) fn system_mut_after_validation(
        &mut self,
        system: ParticleSystemId,
    ) -> &mut ParticleSystem {
        self.particle_systems
            .get_mut(system)
            .expect("validated particle system remains live during one operation")
    }
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
        let root = world
            .create_body(&BodyDef::default())
            .expect("body should fit");
        let survivor = world
            .create_body(&BodyDef::default())
            .expect("body should fit");
        let first_fixture = world
            .create_fixture(root, &test_fixture_definition())
            .expect("fixture should fit");
        let second_fixture = world
            .create_fixture(root, &test_fixture_definition())
            .expect("fixture should fit");
        let first_joint = world
            .create_joint(
                crate::RevoluteJointDef::new(root, survivor)
                    .expect("distinct bodies form a valid joint")
                    .into(),
            )
            .expect("joint should fit");
        let second_joint = world
            .create_joint(
                crate::RevoluteJointDef::new(root, survivor)
                    .expect("distinct bodies form a valid joint")
                    .into(),
            )
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
            Some(ObjectSnapshot::Body {
                fixtures, joints, ..
            })
                if fixtures == &[second_fixture, first_fixture]
                    && joints == &[second_joint, first_joint]
        ));
    }

    #[test]
    fn invalid_body_destruction_is_state_preserving() {
        // Arrange
        let mut world = test_world();
        let body = world
            .create_body(&BodyDef::default())
            .expect("body should fit");
        let fixture = world
            .create_fixture(body, &test_fixture_definition())
            .expect("fixture should fit");
        let mut other = test_world();
        let foreign = other
            .create_body(&BodyDef::default())
            .expect("body should fit");

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
        let stale = world
            .create_body(&BodyDef::default())
            .expect("body should fit");
        world.destroy_body(stale).expect("body should be live");
        let survivor = world
            .create_body(&BodyDef::default())
            .expect("body should fit");

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
            Some(ObjectSnapshot::ParticleGroup {
                system: snapshot_system,
                particles,
            }) if *snapshot_system == system && particles == &[grouped]
        ));
        assert!(matches!(
            records.get(1).map(DestructionRecord::snapshot),
            Some(ObjectSnapshot::Particle {
                system: snapshot_system,
                maybe_group,
            }) if *snapshot_system == system && *maybe_group == Some(group)
        ));
        assert!(matches!(
            records.get(2).map(DestructionRecord::snapshot),
            Some(ObjectSnapshot::Particle {
                system: snapshot_system,
                maybe_group,
            }) if *snapshot_system == system && maybe_group.is_none()
        ));
        assert!(matches!(
            records.last().map(DestructionRecord::snapshot),
            Some(ObjectSnapshot::ParticleSystem { groups, particles })
                if groups == &[group] && particles == &[grouped, ungrouped]
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
        assert_eq!(
            world
                .particle_systems
                .get(system)
                .expect("system remains live")
                .storage
                .len(),
            1
        );
    }

    #[test]
    fn direct_dependent_destruction_updates_all_adjacency() {
        // Arrange
        let mut world = test_world();
        let first = world
            .create_body(&BodyDef::default())
            .expect("body should fit");
        let second = world
            .create_body(&BodyDef::default())
            .expect("body should fit");
        let fixture = world
            .create_fixture(first, &test_fixture_definition())
            .expect("fixture should fit");
        let joint = world
            .create_joint(
                crate::RevoluteJointDef::new(first, second)
                    .expect("distinct bodies form a valid joint")
                    .into(),
            )
            .expect("joint should fit");
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
        assert_eq!(system.storage.len(), 0);
    }

    #[test]
    fn owned_records_remain_usable_after_slot_reuse() {
        // Arrange
        let mut world = test_world();
        let body = world
            .create_body(&BodyDef::default())
            .expect("body should fit");
        let fixture = world
            .create_fixture(body, &test_fixture_definition())
            .expect("fixture should fit");

        // Act
        let records = world.destroy_body(body).expect("body should be live");
        let replacement = world
            .create_body(&BodyDef::default())
            .expect("reused slot should fit");

        // Assert
        assert_ne!(body, replacement);
        assert_eq!(records[0].destroyed(), DestroyedId::Fixture(fixture));
        assert_eq!(records[1].destroyed(), DestroyedId::Body(body));
        assert!(matches!(
            records[0].snapshot(),
            ObjectSnapshot::Fixture {
                body: snapshot_body,
                ..
            } if *snapshot_body == body
        ));
    }

    #[test]
    fn diagnostic_identity_exhaustion_rejects_insertion() {
        // Arrange
        let mut world = test_world();
        world.set_next_diagnostic_id_for_test(u64::MAX - 1);
        world
            .create_body(&BodyDef::default())
            .expect("penultimate ID should remain valid");
        world
            .create_body(&BodyDef::default())
            .expect("maximum ID should remain valid");

        // Act
        let result = world.create_body(&BodyDef::default());

        // Assert
        assert_eq!(result, Err(ArenaInsertError::DiagnosticIdExhausted));
        assert_eq!(world.bodies.iter().count(), 2);
        assert_eq!(
            world
                .bodies
                .iter()
                .map(|(_body, record)| record.diagnostic_id)
                .collect::<Vec<_>>(),
            vec![u64::MAX - 1, u64::MAX]
        );
    }

    #[test]
    fn sensor_change_records_pending_body_wake() {
        // Arrange
        let mut world = test_world();
        let body = world
            .create_body(&BodyDef::default())
            .expect("body should fit");
        let fixture = world
            .create_fixture(body, &test_fixture_definition())
            .expect("fixture should fit");

        // Act
        world
            .set_fixture_sensor(fixture, true)
            .expect("fixture should remain live");

        // Assert
        assert!(
            world
                .bodies
                .get(body)
                .expect("body should remain live")
                .pending_wake
        );
    }

    #[test]
    fn filter_change_records_refilter_and_touches_without_entry_churn() {
        // Arrange
        let mut world = test_world();
        let body = world
            .create_body(&BodyDef::default())
            .expect("body should fit");
        let fixture = world
            .create_fixture(body, &test_fixture_definition())
            .expect("fixture should fit");
        let before_count = world.broad_phase.proxy_count();

        // Act
        world
            .set_fixture_filter(fixture, FilterData::new(0x0002, 0x0004, -1))
            .expect("fixture should remain live");

        // Assert
        assert!(
            world
                .fixtures
                .get(fixture)
                .expect("fixture should remain live")
                .pending_refilter
        );
        assert_eq!(world.broad_phase.proxy_count(), before_count);
    }

    #[test]
    fn type_change_records_pending_wake_and_contact_destruction() {
        // Arrange
        let mut world = test_world();
        let definition = BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)
            .expect("body definition should be valid");
        let body = world.create_body(&definition).expect("body should fit");

        // Act
        world
            .set_body_type(body, BodyType::Static)
            .expect("body should remain live");

        // Assert
        let record = world.bodies.get(body).expect("body should remain live");
        assert!(record.pending_wake);
        assert!(record.pending_contact_destruction);
    }

    #[test]
    fn deactivation_records_pending_contact_destruction() {
        // Arrange
        let mut world = test_world();
        let body = world
            .create_body(&BodyDef::default())
            .expect("body should fit");

        // Act
        world
            .set_body_active(body, false)
            .expect("body should remain live");

        // Assert
        assert!(
            world
                .bodies
                .get(body)
                .expect("body should remain live")
                .pending_contact_destruction
        );
    }
}
