//! Native execution for the closed Phase 6 rigid-world timelines.

mod evidence;
mod model;
mod phase7;
mod phase8;
mod phase9;
pub use phase9::{
    PHASE9_REGISTRY_ID, PHASE9_REQUIRED_POLICY_PATHS, Phase9PolicyKind,
    phase9_observation_is_declared, phase9_policy_for_path,
};

use evidence::{
    capture_checkpoint, collect_direct_transitions, collect_mutation_report, collect_step_report,
    observe_step, remove_destroyed_mapping,
};
use model::{
    action_error, body_created_witness, body_declaration, body_type, checked_u32,
    declaration_error, feature, filter_data, fixture_declaration, fixture_definition,
    native_body_mass_data, rigid_body_kind, rigid_filter, transform_bits, vec2, vec2_bits,
};

use liquidfun::collision::shape::{CircleShape, PolygonShape};
use liquidfun::collision::{FeatureKind, FilterData, ManifoldKind, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyId, BodyMassData, BodyType, DestroyedId, DestructionCause, DestructionRecord,
    FixtureDef, FixtureId, JointId, ManagedContactSnapshot, StepConfiguration, StepLimits,
    StepReport, World,
};
use liquidfun_test_protocol::{
    FloatBits, RIGID_WORLD_POSITION_ITERATIONS, RIGID_WORLD_TIMESTEP_BITS,
    RIGID_WORLD_VELOCITY_ITERATIONS, RigidBodyDeclaration, RigidBodyKind, RigidBodySnapshot,
    RigidContactEvent, RigidContactEventKind, RigidContactFeature, RigidContactIdentity,
    RigidContactResult, RigidDestructionRecord, RigidExpectedCheckpoint, RigidExpectedCounts,
    RigidFeatureKind, RigidFilterBits, RigidFixtureDeclaration, RigidFixtureShape,
    RigidFixtureSnapshot, RigidManifoldKind, RigidManifoldPoint, RigidManifoldResult,
    RigidWorldAction, RigidWorldActionRecord, RigidWorldDecodeError, RigidWorldObservation,
    RigidWorldRequestRecord, RigidWorldResultRecord, RigidWorldTimeline, RigidWorldTimelineResult,
    RigidWorldWitness, RigidWorldWitnessFamily, ScenarioId, TransformBits, Vec2Bits,
    validate_rigid_world_result_against_request,
};

/// Typed failure while mapping a validated rigid timeline onto native world APIs.
#[derive(Debug, thiserror::Error)]
pub enum NativeRigidWorldError {
    /// A semantic action could not resolve or execute through the checked world API.
    #[error("native rigid action `{action_id}` failed: {message}")]
    Action {
        /// Stable action identity.
        action_id: Box<str>,
        /// Bounded checked-world diagnostic.
        message: Box<str>,
    },
    /// Produced state disagreed with the validated declaration contract.
    #[error("native rigid declaration disagreement at `{checkpoint_id}`: {message}")]
    Declaration {
        /// Stable checkpoint identity.
        checkpoint_id: Box<str>,
        /// Bounded mismatch diagnostic.
        message: Box<str>,
    },
    /// Result construction or protocol validation rejected the aggregate.
    #[error(transparent)]
    Result(#[from] RigidWorldDecodeError),
    /// A completed timeline retained native world state.
    #[error("native rigid timeline `{family:?}` failed terminal reset proof")]
    Reset {
        /// Timeline family that retained state.
        family: RigidWorldWitnessFamily,
    },
}

/// Stateless native executor for one validated Phase 6 request.
pub struct NativeRigidWorldExecutor;

impl NativeRigidWorldExecutor {
    /// Executes every timeline through a fresh native [`World`].
    ///
    /// # Errors
    ///
    /// Returns [`NativeRigidWorldError`] without an accepted partial result when semantic
    /// resolution, checked world execution, declaration validation, or reset proof fails.
    pub fn execute(
        request: &RigidWorldRequestRecord,
    ) -> Result<RigidWorldResultRecord, NativeRigidWorldError> {
        let timelines = request
            .scenario()
            .timelines()
            .iter()
            .map(execute_timeline)
            .collect::<Result<Vec<_>, _>>()?;
        let result = RigidWorldResultRecord::new(
            request.request_id().clone(),
            request.scenario().scenario_id().clone(),
            timelines,
        )?;
        validate_native_rigid_world_result(request, &result)?;
        Ok(result)
    }
}

/// Revalidates a native result against its complete request declaration.
///
/// # Errors
///
/// Returns [`NativeRigidWorldError::Declaration`] when identity, timeline, checkpoint, count,
/// or declaration order differs.
pub fn validate_native_rigid_world_result(
    request: &RigidWorldRequestRecord,
    result: &RigidWorldResultRecord,
) -> Result<(), NativeRigidWorldError> {
    validate_rigid_world_result_against_request(request, result).map_err(|error| {
        NativeRigidWorldError::Declaration {
            checkpoint_id: request.scenario().scenario_id().as_str().into(),
            message: error.to_string().into(),
        }
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Observation {
    witness: RigidWorldWitness,
    maybe_contact: Option<RigidContactIdentity>,
}

pub(crate) struct TimelineExecutor {
    pub(crate) family: RigidWorldWitnessFamily,
    pub(crate) world: World,
    pub(crate) bodies: Vec<(ScenarioId, BodyId)>,
    pub(crate) fixtures: Vec<(ScenarioId, FixtureId)>,
    fixture_owners: Vec<(FixtureId, BodyId)>,
    pub(crate) joints: Vec<(ScenarioId, JointId)>,
    pub(crate) particle_systems: Vec<(ScenarioId, liquidfun::ParticleSystemId)>,
    pub(crate) particles: Vec<(
        ScenarioId,
        liquidfun::ParticleSystemId,
        liquidfun::ParticleId,
    )>,
    ropes: Vec<(ScenarioId, liquidfun::rope::Rope)>,
    filter_directives: Vec<(FixtureId, FixtureId, bool)>,
    pre_solve_directives: Vec<(FixtureId, FixtureId, liquidfun::PreSolveDirective)>,
    contact_identities: Vec<(u64, RigidContactIdentity)>,
    seen_manager_occurrences: Vec<u64>,
    seen_lifecycle_occurrences: Vec<u64>,
    next_lifecycle_ordinal: u32,
    maybe_last_contact: Option<RigidContactIdentity>,
    events: Vec<RigidContactEvent>,
    destructions: Vec<RigidDestructionRecord>,
    observations: Vec<Observation>,
    semantic_observations: Vec<RigidWorldObservation>,
}

impl TimelineExecutor {
    fn new(family: RigidWorldWitnessFamily) -> Result<Self, NativeRigidWorldError> {
        let mut world = World::new().map_err(|error| NativeRigidWorldError::Action {
            action_id: "world-create".into(),
            message: error.to_string().into(),
        })?;
        if matches!(
            family,
            RigidWorldWitnessFamily::NonCollidingBodyFixtureLifecycle
                | RigidWorldWitnessFamily::SingleContactLifecycle
        ) {
            world
                .set_continuous_physics_enabled(false)
                .map_err(|error| NativeRigidWorldError::Action {
                    action_id: "world-configure".into(),
                    message: error.to_string().into(),
                })?;
        }
        Ok(Self {
            family,
            world,
            bodies: Vec::new(),
            fixtures: Vec::new(),
            fixture_owners: Vec::new(),
            joints: Vec::new(),
            particle_systems: Vec::new(),
            particles: Vec::new(),
            ropes: Vec::new(),
            filter_directives: Vec::new(),
            pre_solve_directives: Vec::new(),
            contact_identities: Vec::new(),
            seen_manager_occurrences: Vec::new(),
            seen_lifecycle_occurrences: Vec::new(),
            next_lifecycle_ordinal: 0,
            maybe_last_contact: None,
            events: Vec::new(),
            destructions: Vec::new(),
            observations: Vec::new(),
            semantic_observations: Vec::new(),
        })
    }

    fn body(
        &self,
        id: &ScenarioId,
        action: &RigidWorldActionRecord,
    ) -> Result<BodyId, NativeRigidWorldError> {
        self.bodies
            .iter()
            .find_map(|(candidate, body)| (candidate == id).then_some(*body))
            .ok_or_else(|| action_error(action, format!("unknown body `{id}`")))
    }

    fn fixture(
        &self,
        id: &ScenarioId,
        action: &RigidWorldActionRecord,
    ) -> Result<FixtureId, NativeRigidWorldError> {
        self.fixtures
            .iter()
            .find_map(|(candidate, fixture)| (candidate == id).then_some(*fixture))
            .ok_or_else(|| action_error(action, format!("unknown fixture `{id}`")))
    }

    pub(crate) fn joint(
        &self,
        id: &ScenarioId,
        action: &RigidWorldActionRecord,
    ) -> Result<JointId, NativeRigidWorldError> {
        self.joints
            .iter()
            .find_map(|(candidate, joint)| (candidate == id).then_some(*joint))
            .ok_or_else(|| action_error(action, format!("unknown joint `{id}`")))
    }

    pub(crate) fn semantic_body(&self, body: BodyId) -> Result<ScenarioId, NativeRigidWorldError> {
        self.bodies
            .iter()
            .find_map(|(id, candidate)| (*candidate == body).then(|| id.clone()))
            .ok_or_else(|| NativeRigidWorldError::Declaration {
                checkpoint_id: "body-map".into(),
                message: "native body was not mapped to a semantic identity".into(),
            })
    }

    pub(crate) fn semantic_joint(
        &self,
        joint: JointId,
    ) -> Result<ScenarioId, NativeRigidWorldError> {
        self.joints
            .iter()
            .find_map(|(id, candidate)| (*candidate == joint).then(|| id.clone()))
            .ok_or_else(|| NativeRigidWorldError::Declaration {
                checkpoint_id: "joint-map".into(),
                message: "native joint was not mapped to a semantic identity".into(),
            })
    }

    pub(crate) fn semantic_fixture(
        &self,
        fixture: FixtureId,
    ) -> Result<ScenarioId, NativeRigidWorldError> {
        self.fixtures
            .iter()
            .find_map(|(id, candidate)| (*candidate == fixture).then(|| id.clone()))
            .ok_or_else(|| NativeRigidWorldError::Declaration {
                checkpoint_id: "contact-map".into(),
                message: "manager contact referenced an undeclared fixture".into(),
            })
    }

    fn contact_identity(
        &mut self,
        contact: &ManagedContactSnapshot,
    ) -> Result<RigidContactIdentity, NativeRigidWorldError> {
        let manager_occurrence = contact.differential_occurrence();
        if let Some((_, identity)) = self
            .contact_identities
            .iter()
            .find(|(candidate, _)| *candidate == manager_occurrence)
        {
            return Ok(identity.clone());
        }
        let fixtures = contact.fixtures();
        let children = contact.child_indices();
        let first_fixture_id = self.semantic_fixture(fixtures[0])?;
        let second_fixture_id = self.semantic_fixture(fixtures[1])?;
        let child_a = checked_u32(children[0].get(), "contact-child")?;
        let child_b = checked_u32(children[1].get(), "contact-child")?;
        let prior_occurrences = self
            .contact_identities
            .iter()
            .filter(|(_, identity)| {
                identity.fixture_a_id() == &first_fixture_id
                    && identity.child_a() == child_a
                    && identity.fixture_b_id() == &second_fixture_id
                    && identity.child_b() == child_b
            })
            .count();
        let occurrence = u32::try_from(prior_occurrences)
            .ok()
            .and_then(|count| count.checked_add(1))
            .ok_or_else(|| NativeRigidWorldError::Declaration {
                checkpoint_id: "contact-occurrence".into(),
                message: "contact occurrence exceeded the protocol representation".into(),
            })?;
        let identity = RigidContactIdentity::new(
            first_fixture_id,
            child_a,
            second_fixture_id,
            child_b,
            occurrence,
        )?;
        self.contact_identities
            .push((manager_occurrence, identity.clone()));
        Ok(identity)
    }

    fn push_observation(
        &mut self,
        witness: RigidWorldWitness,
        maybe_contact: Option<RigidContactIdentity>,
    ) {
        let observation = Observation {
            witness,
            maybe_contact,
        };
        if !self.observations.contains(&observation) {
            self.observations.push(observation);
        }
    }
}

fn execute_timeline(
    timeline: &RigidWorldTimeline,
) -> Result<RigidWorldTimelineResult, NativeRigidWorldError> {
    let mut executor = TimelineExecutor::new(timeline.witness_family())?;
    let mut checkpoints = Vec::with_capacity(timeline.checkpoints().len());
    for action in timeline.actions() {
        execute_action(&mut executor, timeline, action)?;
        for expected in timeline
            .checkpoints()
            .iter()
            .filter(|checkpoint| checkpoint.after_action_id() == action.action_id())
        {
            checkpoints.push(capture_checkpoint(&mut executor, timeline, expected)?);
        }
    }
    if !executor.bodies.is_empty()
        || !executor.fixtures.is_empty()
        || !executor.joints.is_empty()
        || !executor.ropes.is_empty()
        || !executor.particle_systems.is_empty()
        || !executor.particles.is_empty()
        || executor.world.joint_count() != 0
        || executor.world.contact_count() != 0
    {
        return Err(NativeRigidWorldError::Reset {
            family: executor.family,
        });
    }
    Ok(RigidWorldTimelineResult {
        witness_family: timeline.witness_family(),
        checkpoints: checkpoints.into_boxed_slice(),
    })
}

#[allow(
    clippy::too_many_lines,
    reason = "closed action dispatch remains auditable in one place"
)]
fn execute_action(
    executor: &mut TimelineExecutor,
    timeline: &RigidWorldTimeline,
    record: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    if phase9::execute_action(executor, timeline, record)? {
        return Ok(());
    }
    if phase8::execute_action(executor, timeline, record)? {
        return Ok(());
    }
    if phase7::execute_action(executor, record)? {
        return Ok(());
    }
    match record.action() {
        RigidWorldAction::CreateBody { body_id } => {
            let declaration = body_declaration(timeline, body_id, record)?;
            let definition = BodyDef::new(
                body_type(declaration.body_kind()),
                vec2(declaration.transform().position),
                declaration.transform().angle_bits.to_f32(),
                declaration.active(),
            )
            .map_err(|error| action_error(record, error))?;
            let body = executor
                .world
                .create_body(&definition)
                .map_err(|error| action_error(record, error))?;
            executor.bodies.push((body_id.clone(), body));
            executor.push_observation(body_created_witness(declaration.body_kind()), None);
        }
        RigidWorldAction::CreateFixture { fixture_id } => {
            let declaration = fixture_declaration(timeline, fixture_id, record)?;
            let owner = executor.body(declaration.owner_body_id(), record)?;
            let definition =
                fixture_definition(declaration).map_err(|message| action_error(record, message))?;
            let fixture = executor
                .world
                .create_fixture(owner, &definition)
                .map_err(|error| action_error(record, error))?;
            executor.fixtures.push((fixture_id.clone(), fixture));
            executor.fixture_owners.push((fixture, owner));
            executor.push_observation(RigidWorldWitness::FixturesCreated, None);
        }
        RigidWorldAction::InspectBody { body_id } => {
            let body = executor.body(body_id, record)?;
            executor
                .world
                .body_snapshot(body)
                .map_err(|error| action_error(record, error))?;
            executor.push_observation(RigidWorldWitness::BodyInspected, None);
        }
        RigidWorldAction::InspectFixture { fixture_id } => {
            let fixture = executor.fixture(fixture_id, record)?;
            executor
                .world
                .fixture_snapshot(fixture)
                .map_err(|error| action_error(record, error))?;
            executor.push_observation(RigidWorldWitness::FixtureInspected, None);
        }
        RigidWorldAction::SetBodyTransform { body_id, transform } => {
            let body = executor.body(body_id, record)?;
            executor
                .world
                .set_body_transform(
                    body,
                    vec2(transform.position),
                    transform.angle_bits.to_f32(),
                )
                .map_err(|error| action_error(record, error))?;
            executor.push_observation(RigidWorldWitness::BodyTransformChanged, None);
        }
        RigidWorldAction::SetBodyType { body_id, body_kind } => {
            let body = executor.body(body_id, record)?;
            executor
                .world
                .set_body_type(body, body_type(*body_kind))
                .map_err(|error| action_error(record, error))?;
            executor.push_observation(RigidWorldWitness::BodyTypeChanged, None);
        }
        RigidWorldAction::SetBodyActive { body_id, active } => {
            let body = executor.body(body_id, record)?;
            executor
                .world
                .set_body_active(body, *active)
                .map_err(|error| action_error(record, error))?;
            collect_direct_transitions(executor)?;
            let witness = match (record.phase(), active) {
                ("deactivate", false) => Some(RigidWorldWitness::DeactivationDestroyedContact),
                (_, false) => Some(RigidWorldWitness::BodyDeactivated),
                ("reactivate", true) => None,
                (_, true) => Some(RigidWorldWitness::BodyReactivated),
            };
            if let Some(witness) = witness {
                executor.push_observation(witness, executor.maybe_last_contact.clone());
            }
        }
        RigidWorldAction::SetFixtureSensor { fixture_id, sensor } => {
            let fixture = executor.fixture(fixture_id, record)?;
            executor
                .world
                .set_fixture_sensor(fixture, *sensor)
                .map_err(|error| action_error(record, error))?;
            if executor.family == RigidWorldWitnessFamily::NonCollidingBodyFixtureLifecycle {
                executor.push_observation(
                    if *sensor {
                        RigidWorldWitness::SensorEnabled
                    } else {
                        RigidWorldWitness::SensorDisabled
                    },
                    None,
                );
            }
        }
        RigidWorldAction::SetFixtureMaterial {
            fixture_id,
            friction_bits,
            restitution_bits,
        } => {
            let fixture = executor.fixture(fixture_id, record)?;
            executor
                .world
                .set_fixture_friction(fixture, friction_bits.to_f32())
                .and_then(|()| {
                    executor
                        .world
                        .set_fixture_restitution(fixture, restitution_bits.to_f32())
                })
                .map_err(|error| action_error(record, error))?;
            executor.push_observation(RigidWorldWitness::MaterialChanged, None);
        }
        RigidWorldAction::SetFixtureFilter { fixture_id, filter } => {
            let fixture = executor.fixture(fixture_id, record)?;
            executor
                .world
                .set_fixture_filter(fixture, filter_data(*filter))
                .map_err(|error| action_error(record, error))?;
            if executor.family == RigidWorldWitnessFamily::NonCollidingBodyFixtureLifecycle {
                executor.push_observation(RigidWorldWitness::FilterChanged, None);
            }
        }
        RigidWorldAction::SetFixtureDensity {
            fixture_id,
            density_bits,
        } => {
            let fixture = executor.fixture(fixture_id, record)?;
            executor
                .world
                .set_fixture_density(fixture, density_bits.to_f32())
                .map_err(|error| action_error(record, error))?;
            executor.push_observation(RigidWorldWitness::DensityChangedWithoutMassReset, None);
        }
        RigidWorldAction::ResetMassData { body_id } => {
            let body = executor.body(body_id, record)?;
            executor
                .world
                .reset_body_mass_data(body)
                .map_err(|error| action_error(record, error))?;
            executor.push_observation(RigidWorldWitness::MassReset, None);
        }
        RigidWorldAction::SetCustomMassData {
            body_id,
            mass_bits,
            center,
            inertia_bits,
        } => {
            let body = executor.body(body_id, record)?;
            let data = native_body_mass_data(*mass_bits, *center, *inertia_bits)
                .map_err(|error| action_error(record, error))?;
            executor
                .world
                .set_body_mass_data(body, data)
                .map_err(|error| action_error(record, error))?;
            executor.push_observation(RigidWorldWitness::CustomMassSet, None);
        }
        RigidWorldAction::Step {
            timestep_bits,
            velocity_iterations,
            position_iterations,
        } => {
            if timestep_bits.bits() != RIGID_WORLD_TIMESTEP_BITS
                || *velocity_iterations != RIGID_WORLD_VELOCITY_ITERATIONS
                || *position_iterations != RIGID_WORLD_POSITION_ITERATIONS
            {
                return Err(action_error(record, "unsupported Phase 6 step tuple"));
            }
            let configuration = StepConfiguration::new(
                timestep_bits.to_f32(),
                *velocity_iterations,
                *position_iterations,
            )
            .map_err(|error| action_error(record, error))?;
            let report = phase8::step(executor, configuration, StepLimits::default())
                .map_err(|error| action_error(record, error))?;
            collect_step_report(executor, &report)?;
            observe_step(executor, record.phase());
            phase8::collect_step_lifecycle(executor, &report)?;
        }
        RigidWorldAction::DestroyFixture { fixture_id } => {
            let fixture = executor.fixture(fixture_id, record)?;
            let record_result = executor
                .world
                .destroy_fixture(fixture)
                .map_err(|error| action_error(record, error))?;
            collect_mutation_report(executor, &record_result)?;
            phase8::collect_mutation_lifecycle(executor, record_result.lifecycle())?;
            executor
                .fixtures
                .retain(|(_, candidate)| *candidate != fixture);
            executor
                .fixture_owners
                .retain(|(candidate, _owner)| *candidate != fixture);
            let witness = if executor.family == RigidWorldWitnessFamily::SingleContactLifecycle {
                RigidWorldWitness::FixtureDestroyedContact
            } else {
                RigidWorldWitness::FixtureDestroyed
            };
            executor.push_observation(witness, executor.maybe_last_contact.clone());
        }
        RigidWorldAction::DestroyBody { body_id } => {
            let body = executor.body(body_id, record)?;
            let records = executor
                .world
                .destroy_body(body)
                .map_err(|error| action_error(record, error))?;
            collect_mutation_report(executor, &records)?;
            phase8::collect_mutation_lifecycle(executor, records.lifecycle())?;
            for destruction in &records {
                remove_destroyed_mapping(executor, destruction.destroyed());
                phase8::remove_destroyed_mapping(executor, destruction.destroyed());
            }
            let witness = if executor.family == RigidWorldWitnessFamily::SingleContactLifecycle {
                RigidWorldWitness::BodyCascadeEndOrdered
            } else {
                RigidWorldWitness::BodyDestroyed
            };
            executor.push_observation(witness, executor.maybe_last_contact.clone());
        }
        _ => {
            return Err(action_error(
                record,
                "unsupported rigid-world action reached the Phase 6 dispatcher",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_centered_inertia_defense_rejects_equality_before_world_mutation() {
        // Arrange
        let mut world = World::new().expect("world key should remain available");
        let definition = BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)
            .expect("dynamic body definition should be valid");
        let body = world.create_body(&definition).expect("body should fit");
        let before = world.body_snapshot(body).expect("body should remain live");

        // Act
        let result = native_body_mass_data(
            FloatBits::from_f32(1.0),
            Vec2Bits {
                x_bits: FloatBits::from_f32(1.0),
                y_bits: FloatBits::from_f32(0.0),
            },
            FloatBits::from_f32(1.0),
        );
        let after = world.body_snapshot(body).expect("body should remain live");

        // Assert
        assert_eq!(
            result,
            Err(liquidfun::BodyMassDataError::NonPositiveCenteredRotationalInertia)
        );
        assert_eq!(after, before);
    }
}
