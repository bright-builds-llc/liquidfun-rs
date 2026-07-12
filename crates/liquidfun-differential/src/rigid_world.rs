//! Native execution for the closed Phase 6 rigid-world timelines.

use liquidfun::collision::shape::{CircleShape, PolygonShape};
use liquidfun::collision::{FeatureKind, FilterData, ManifoldKind, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyId, BodyMassData, BodyType, DestroyedId, DestructionCause, DestructionRecord,
    FixtureDef, FixtureId, ManagedContactSnapshot, StepHook, StepLimits, StepReport, World,
};
use liquidfun_test_protocol::{
    FloatBits, RigidBodyDeclaration, RigidBodyKind, RigidBodySnapshot, RigidContactEvent,
    RigidContactEventKind, RigidContactFeature, RigidContactIdentity, RigidContactResult,
    RigidDestructionRecord, RigidExpectedCheckpoint, RigidExpectedCounts, RigidFeatureKind,
    RigidFilterBits, RigidFixtureDeclaration, RigidFixtureShape, RigidFixtureSnapshot,
    RigidManifoldKind, RigidManifoldPoint, RigidManifoldResult, RigidWorldAction,
    RigidWorldActionRecord, RigidWorldDecodeError, RigidWorldRequestRecord, RigidWorldResultRecord,
    RigidWorldTimeline, RigidWorldTimelineResult, RigidWorldWitness, RigidWorldWitnessFamily,
    ScenarioId, TransformBits, Vec2Bits, validate_rigid_world_result_against_request,
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

struct TimelineExecutor {
    family: RigidWorldWitnessFamily,
    world: World,
    bodies: Vec<(ScenarioId, BodyId)>,
    fixtures: Vec<(ScenarioId, FixtureId)>,
    seen_occurrences: Vec<u32>,
    maybe_last_contact: Option<RigidContactIdentity>,
    events: Vec<RigidContactEvent>,
    destructions: Vec<RigidDestructionRecord>,
    observations: Vec<Observation>,
}

impl TimelineExecutor {
    fn new(family: RigidWorldWitnessFamily) -> Result<Self, NativeRigidWorldError> {
        let world = World::new().map_err(|error| NativeRigidWorldError::Action {
            action_id: "world-create".into(),
            message: error.to_string().into(),
        })?;
        Ok(Self {
            family,
            world,
            bodies: Vec::new(),
            fixtures: Vec::new(),
            seen_occurrences: Vec::new(),
            maybe_last_contact: None,
            events: Vec::new(),
            destructions: Vec::new(),
            observations: Vec::new(),
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

    fn semantic_fixture(&self, fixture: FixtureId) -> Result<ScenarioId, NativeRigidWorldError> {
        self.fixtures
            .iter()
            .find_map(|(id, candidate)| (*candidate == fixture).then(|| id.clone()))
            .ok_or_else(|| NativeRigidWorldError::Declaration {
                checkpoint_id: "contact-map".into(),
                message: "manager contact referenced an undeclared fixture".into(),
            })
    }

    fn contact_identity(
        &self,
        contact: &ManagedContactSnapshot,
    ) -> Result<RigidContactIdentity, NativeRigidWorldError> {
        let fixtures = contact.fixtures();
        let children = contact.child_indices();
        let occurrence = u32::try_from(contact.differential_occurrence()).map_err(|_| {
            NativeRigidWorldError::Declaration {
                checkpoint_id: "contact-occurrence".into(),
                message: "contact occurrence exceeded the protocol representation".into(),
            }
        })?;
        RigidContactIdentity::new(
            self.semantic_fixture(fixtures[0])?,
            checked_u32(children[0].get(), "contact-child")?,
            self.semantic_fixture(fixtures[1])?,
            checked_u32(children[1].get(), "contact-child")?,
            occurrence,
        )
        .map_err(Into::into)
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
            let data = BodyMassData::new(mass_bits.to_f32(), vec2(*center), inertia_bits.to_f32())
                .map_err(|error| action_error(record, error))?;
            executor
                .world
                .set_body_mass_data(body, data)
                .map_err(|error| action_error(record, error))?;
            executor.push_observation(RigidWorldWitness::CustomMassSet, None);
        }
        RigidWorldAction::Step { .. } => {
            let report = executor
                .world
                .step(&mut NativeHook, StepLimits::default())
                .map_err(|error| action_error(record, error))?;
            collect_step_report(executor, &report)?;
            observe_step(executor, record.phase());
        }
        RigidWorldAction::DestroyFixture { fixture_id } => {
            let fixture = executor.fixture(fixture_id, record)?;
            let record_result = executor
                .world
                .destroy_fixture(fixture)
                .map_err(|error| action_error(record, error))?;
            collect_direct_transitions(executor)?;
            push_object_destruction(executor, &record_result)?;
            executor
                .fixtures
                .retain(|(_, candidate)| *candidate != fixture);
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
            collect_direct_transitions(executor)?;
            for destruction in &records {
                if matches!(destruction.destroyed(), DestroyedId::Body(_)) {
                    push_object_destruction(executor, destruction)?;
                }
                remove_destroyed_mapping(executor, destruction.destroyed());
            }
            let witness = if executor.family == RigidWorldWitnessFamily::SingleContactLifecycle {
                RigidWorldWitness::BodyCascadeEndOrdered
            } else {
                RigidWorldWitness::BodyDestroyed
            };
            executor.push_observation(witness, executor.maybe_last_contact.clone());
        }
    }
    Ok(())
}

fn capture_checkpoint(
    executor: &mut TimelineExecutor,
    timeline: &RigidWorldTimeline,
    expected: &RigidExpectedCheckpoint,
) -> Result<liquidfun_test_protocol::RigidWorldCheckpointResult, NativeRigidWorldError> {
    let bodies = timeline
        .bodies()
        .iter()
        .filter_map(|declaration| {
            executor
                .bodies
                .iter()
                .find_map(|(id, body)| (id == declaration.body_id()).then_some(*body))
                .map(|body| body_result(executor, declaration, body))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let fixtures = timeline
        .fixtures()
        .iter()
        .filter_map(|declaration| {
            executor
                .fixtures
                .iter()
                .find_map(|(id, fixture)| (id == declaration.fixture_id()).then_some(*fixture))
                .map(|fixture| fixture_result(executor, declaration, fixture))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let contacts = executor
        .world
        .rigid_contact_diagnostics()
        .iter()
        .map(|diagnostic| contact_result(executor, diagnostic.contact()))
        .collect::<Result<Vec<_>, _>>()?;
    let result = liquidfun_test_protocol::RigidWorldCheckpointResult {
        checkpoint_id: expected.checkpoint_id().clone(),
        phase: expected.phase().into(),
        counts: counts(
            &bodies,
            &fixtures,
            &contacts,
            &executor.events,
            &executor.destructions,
        )?,
        bodies: bodies.into_boxed_slice(),
        fixtures: fixtures.into_boxed_slice(),
        contacts: contacts.into_boxed_slice(),
        events: std::mem::take(&mut executor.events).into_boxed_slice(),
        destructions: std::mem::take(&mut executor.destructions).into_boxed_slice(),
    };
    validate_checkpoint(expected, &result, &executor.observations)?;
    executor.observations.clear();
    Ok(result)
}

fn collect_step_report(
    executor: &mut TimelineExecutor,
    report: &StepReport,
) -> Result<(), NativeRigidWorldError> {
    collect_transitions(executor, report.contact_transitions())?;
    for event in report.events() {
        if event.maybe_pre_solve().is_some() {
            let contact = executor.contact_identity(event.contact())?;
            executor.events.push(RigidContactEvent {
                kind: RigidContactEventKind::PreSolve,
                contact,
            });
        }
    }
    for solve in report.contact_solves() {
        let contact = executor.contact_identity(solve.contact())?;
        executor.events.push(RigidContactEvent {
            kind: RigidContactEventKind::PostSolve,
            contact,
        });
    }
    Ok(())
}

fn collect_direct_transitions(
    executor: &mut TimelineExecutor,
) -> Result<(), NativeRigidWorldError> {
    let transitions = executor.world.rigid_drain_contact_transitions();
    collect_transitions(executor, &transitions)
}

fn collect_transitions(
    executor: &mut TimelineExecutor,
    transitions: &[liquidfun::ContactTransition],
) -> Result<(), NativeRigidWorldError> {
    for transition in transitions {
        let identity = executor.contact_identity(transition.contact())?;
        executor.maybe_last_contact = Some(identity.clone());
        let occurrence = identity.occurrence();
        if transition.kind() == liquidfun::ContactTransitionKind::Begin
            && !executor.seen_occurrences.contains(&occurrence)
        {
            executor.seen_occurrences.push(occurrence);
            executor.events.push(RigidContactEvent {
                kind: RigidContactEventKind::Created,
                contact: identity.clone(),
            });
        }
        let kind = match transition.kind() {
            liquidfun::ContactTransitionKind::Begin => RigidContactEventKind::Begin,
            liquidfun::ContactTransitionKind::Persist => RigidContactEventKind::Persist,
            liquidfun::ContactTransitionKind::End => RigidContactEventKind::End,
            _ => {
                return Err(NativeRigidWorldError::Declaration {
                    checkpoint_id: "contact-transition".into(),
                    message: "unsupported contact transition entered Phase 6 evidence".into(),
                });
            }
        };
        executor.events.push(RigidContactEvent {
            kind,
            contact: identity.clone(),
        });
        if transition.kind() == liquidfun::ContactTransitionKind::End {
            executor.events.push(RigidContactEvent {
                kind: RigidContactEventKind::Destroyed,
                contact: identity.clone(),
            });
            executor
                .destructions
                .push(RigidDestructionRecord::Contact { contact: identity });
        }
    }
    Ok(())
}

fn observe_step(executor: &mut TimelineExecutor, phase: &str) {
    let maybe_contact = executor.maybe_last_contact.clone();
    let witnesses: &[RigidWorldWitness] = match phase {
        "step-zero" => &[RigidWorldWitness::ZeroContactStep],
        "contact-begin" => &[
            RigidWorldWitness::ContactCreated,
            RigidWorldWitness::ContactBegin,
            RigidWorldWitness::ManifoldActive,
            RigidWorldWitness::ContactSolved,
        ],
        "contact-persist" => &[
            RigidWorldWitness::ContactPersisted,
            RigidWorldWitness::WarmStartTransferred,
        ],
        "sensor" => &[
            RigidWorldWitness::SensorTouching,
            RigidWorldWitness::SensorWithoutManifold,
        ],
        "filter-remove" => &[RigidWorldWitness::FilterRemovedContact],
        "filter-recreate" => &[RigidWorldWitness::FilterRecreatedContact],
        "reactivate" => &[RigidWorldWitness::ReactivationRecreatedContact],
        _ => &[],
    };
    for witness in witnesses {
        executor.push_observation(
            *witness,
            witness
                .requires_contact_identity()
                .then(|| maybe_contact.clone())
                .flatten(),
        );
    }
}

fn validate_checkpoint(
    expected: &RigidExpectedCheckpoint,
    actual: &liquidfun_test_protocol::RigidWorldCheckpointResult,
    observations: &[Observation],
) -> Result<(), NativeRigidWorldError> {
    if expected.counts() != actual.counts {
        return Err(declaration_error(expected, "semantic counts differ"));
    }
    for transition in expected.transitions() {
        let observed = observations.iter().any(|observation| {
            observation.witness == transition.witness()
                && observation.maybe_contact.as_ref() == transition.maybe_contact()
        });
        if !observed {
            return Err(declaration_error(
                expected,
                format!("missing transition witness `{:?}`", transition.witness()),
            ));
        }
    }
    Ok(())
}

fn body_result(
    executor: &TimelineExecutor,
    declaration: &RigidBodyDeclaration,
    body: BodyId,
) -> Result<RigidBodySnapshot, NativeRigidWorldError> {
    let diagnostic = executor
        .world
        .rigid_body_diagnostic(body)
        .map_err(|error| NativeRigidWorldError::Declaration {
            checkpoint_id: declaration.body_id().as_str().into(),
            message: error.to_string().into(),
        })?;
    let snapshot = diagnostic.snapshot();
    Ok(RigidBodySnapshot {
        body_id: declaration.body_id().clone(),
        body_kind: rigid_body_kind(snapshot.body_type()),
        transform: transform_bits(snapshot.position(), snapshot.angle()),
        active: snapshot.is_active(),
        linear_velocity: vec2_bits(diagnostic.linear_velocity()),
        angular_velocity_bits: FloatBits::from_f32(diagnostic.angular_velocity()),
        mass_bits: FloatBits::from_f32(snapshot.mass()),
        local_center: vec2_bits(snapshot.local_center()),
        inertia_bits: FloatBits::from_f32(snapshot.rotational_inertia()),
    })
}

fn fixture_result(
    executor: &TimelineExecutor,
    declaration: &RigidFixtureDeclaration,
    fixture: FixtureId,
) -> Result<RigidFixtureSnapshot, NativeRigidWorldError> {
    let snapshot = executor.world.fixture_snapshot(fixture).map_err(|error| {
        NativeRigidWorldError::Declaration {
            checkpoint_id: declaration.fixture_id().as_str().into(),
            message: error.to_string().into(),
        }
    })?;
    Ok(RigidFixtureSnapshot {
        fixture_id: declaration.fixture_id().clone(),
        owner_body_id: declaration.owner_body_id().clone(),
        sensor: snapshot.is_sensor(),
        density_bits: FloatBits::from_f32(snapshot.density()),
        friction_bits: FloatBits::from_f32(snapshot.friction()),
        restitution_bits: FloatBits::from_f32(snapshot.restitution()),
        filter: rigid_filter(snapshot.filter_data()),
    })
}

fn contact_result(
    executor: &TimelineExecutor,
    contact: &ManagedContactSnapshot,
) -> Result<RigidContactResult, NativeRigidWorldError> {
    let maybe_manifold = contact
        .maybe_manifold()
        .map(|manifold| manifold_result(manifold, contact))
        .transpose()?;
    Ok(RigidContactResult {
        identity: executor.contact_identity(contact)?,
        touching: contact.is_touching(),
        enabled: contact.is_enabled(),
        sensor: contact.is_sensor(),
        mixed_friction_bits: FloatBits::from_f32(contact.friction()),
        mixed_restitution_bits: FloatBits::from_f32(contact.restitution()),
        maybe_manifold,
    })
}

fn manifold_result(
    manifold: &liquidfun::collision::Manifold,
    contact: &ManagedContactSnapshot,
) -> Result<RigidManifoldResult, NativeRigidWorldError> {
    let kind = manifold
        .kind()
        .ok_or_else(|| NativeRigidWorldError::Declaration {
            checkpoint_id: "manifold".into(),
            message: "contact carried an empty active manifold".into(),
        })?;
    let points = manifold
        .points()
        .iter()
        .zip(contact.points())
        .map(|(geometry, impulses)| RigidManifoldPoint {
            point: vec2_bits(geometry.local_point()),
            feature: feature(geometry.feature_id()),
            normal_impulse_bits: FloatBits::from_f32(impulses.normal_impulse()),
            tangent_impulse_bits: FloatBits::from_f32(impulses.tangent_impulse()),
        })
        .collect();
    Ok(RigidManifoldResult {
        manifold_kind: match kind {
            ManifoldKind::Circles => RigidManifoldKind::Circles,
            ManifoldKind::FaceA => RigidManifoldKind::FaceA,
            ManifoldKind::FaceB => RigidManifoldKind::FaceB,
        },
        local_normal: vec2_bits(manifold.local_normal().unwrap_or(Vec2::ZERO)),
        local_point: vec2_bits(manifold.local_point().unwrap_or(Vec2::ZERO)),
        points,
    })
}

fn push_object_destruction(
    executor: &mut TimelineExecutor,
    record: &DestructionRecord,
) -> Result<(), NativeRigidWorldError> {
    let destruction = match record.destroyed() {
        DestroyedId::Body(body) => RigidDestructionRecord::Body {
            body_id: semantic_body(executor, body)?,
        },
        DestroyedId::Fixture(fixture) => RigidDestructionRecord::Fixture {
            fixture_id: executor.semantic_fixture(fixture)?,
        },
        _ => return Ok(()),
    };
    if matches!(record.cause(), DestructionCause::Explicit) {
        executor.destructions.push(destruction);
    }
    Ok(())
}

fn remove_destroyed_mapping(executor: &mut TimelineExecutor, destroyed: DestroyedId) {
    match destroyed {
        DestroyedId::Body(body) => executor.bodies.retain(|(_, candidate)| *candidate != body),
        DestroyedId::Fixture(fixture) => {
            executor
                .fixtures
                .retain(|(_, candidate)| *candidate != fixture);
        }
        _ => {}
    }
}

fn semantic_body(
    executor: &TimelineExecutor,
    body: BodyId,
) -> Result<ScenarioId, NativeRigidWorldError> {
    executor
        .bodies
        .iter()
        .find_map(|(id, candidate)| (*candidate == body).then(|| id.clone()))
        .ok_or_else(|| NativeRigidWorldError::Declaration {
            checkpoint_id: "body-map".into(),
            message: "destruction referenced an undeclared body".into(),
        })
}

fn counts(
    bodies: &[RigidBodySnapshot],
    fixtures: &[RigidFixtureSnapshot],
    contacts: &[RigidContactResult],
    events: &[RigidContactEvent],
    destructions: &[RigidDestructionRecord],
) -> Result<RigidExpectedCounts, NativeRigidWorldError> {
    let manifold_points = contacts
        .iter()
        .map(|contact| {
            contact
                .maybe_manifold
                .as_ref()
                .map_or(0, |value| value.points.len())
        })
        .sum();
    Ok(RigidExpectedCounts {
        bodies: checked_u32(bodies.len(), "body-count")?,
        fixtures: checked_u32(fixtures.len(), "fixture-count")?,
        contacts: checked_u32(contacts.len(), "contact-count")?,
        manifold_points: checked_u32(manifold_points, "manifold-point-count")?,
        events: checked_u32(events.len(), "event-count")?,
        destructions: checked_u32(destructions.len(), "destruction-count")?,
    })
}

fn body_declaration<'a>(
    timeline: &'a RigidWorldTimeline,
    id: &ScenarioId,
    action: &RigidWorldActionRecord,
) -> Result<&'a RigidBodyDeclaration, NativeRigidWorldError> {
    timeline
        .bodies()
        .iter()
        .find(|declaration| declaration.body_id() == id)
        .ok_or_else(|| action_error(action, format!("missing declaration for body `{id}`")))
}

fn fixture_declaration<'a>(
    timeline: &'a RigidWorldTimeline,
    id: &ScenarioId,
    action: &RigidWorldActionRecord,
) -> Result<&'a RigidFixtureDeclaration, NativeRigidWorldError> {
    timeline
        .fixtures()
        .iter()
        .find(|declaration| declaration.fixture_id() == id)
        .ok_or_else(|| action_error(action, format!("missing declaration for fixture `{id}`")))
}

fn fixture_definition(declaration: &RigidFixtureDeclaration) -> Result<FixtureDef, String> {
    let shape = match declaration.shape() {
        RigidFixtureShape::Circle {
            center,
            radius_bits,
        } => Shape::from(
            CircleShape::new(vec2(*center), radius_bits.to_f32())
                .map_err(|error| error.to_string())?,
        ),
        RigidFixtureShape::Polygon { vertices } => {
            let vertices = vertices.iter().copied().map(vec2).collect::<Vec<_>>();
            Shape::from(PolygonShape::new(&vertices).map_err(|error| error.to_string())?)
        }
    };
    FixtureDef::new(
        shape,
        declaration.density_bits().to_f32(),
        declaration.friction_bits().to_f32(),
        declaration.restitution_bits().to_f32(),
        declaration.sensor(),
        filter_data(declaration.filter()),
    )
    .map_err(|error| error.to_string())
}

fn action_error(
    action: &RigidWorldActionRecord,
    message: impl std::fmt::Display,
) -> NativeRigidWorldError {
    NativeRigidWorldError::Action {
        action_id: action.action_id().as_str().into(),
        message: message.to_string().into(),
    }
}

fn declaration_error(
    checkpoint: &RigidExpectedCheckpoint,
    message: impl std::fmt::Display,
) -> NativeRigidWorldError {
    NativeRigidWorldError::Declaration {
        checkpoint_id: checkpoint.checkpoint_id().as_str().into(),
        message: message.to_string().into(),
    }
}

fn checked_u32(value: usize, field: &'static str) -> Result<u32, NativeRigidWorldError> {
    u32::try_from(value).map_err(|_| NativeRigidWorldError::Declaration {
        checkpoint_id: field.into(),
        message: "value exceeded the protocol representation".into(),
    })
}

const fn body_type(kind: RigidBodyKind) -> BodyType {
    match kind {
        RigidBodyKind::Static => BodyType::Static,
        RigidBodyKind::Kinematic => BodyType::Kinematic,
        RigidBodyKind::Dynamic => BodyType::Dynamic,
    }
}

const fn rigid_body_kind(kind: BodyType) -> RigidBodyKind {
    match kind {
        BodyType::Static => RigidBodyKind::Static,
        BodyType::Kinematic => RigidBodyKind::Kinematic,
        BodyType::Dynamic => RigidBodyKind::Dynamic,
    }
}

const fn body_created_witness(kind: RigidBodyKind) -> RigidWorldWitness {
    match kind {
        RigidBodyKind::Static => RigidWorldWitness::StaticBodyCreated,
        RigidBodyKind::Kinematic => RigidWorldWitness::KinematicBodyCreated,
        RigidBodyKind::Dynamic => RigidWorldWitness::DynamicBodyCreated,
    }
}

const fn filter_data(filter: RigidFilterBits) -> FilterData {
    FilterData::new(
        filter.category_bits(),
        filter.mask_bits(),
        filter.group_index(),
    )
}

const fn rigid_filter(filter: FilterData) -> RigidFilterBits {
    RigidFilterBits::new(
        filter.category_bits(),
        filter.mask_bits(),
        filter.group_index(),
    )
}

fn vec2(bits: Vec2Bits) -> Vec2 {
    Vec2::new(bits.x_bits.to_f32(), bits.y_bits.to_f32())
}

fn vec2_bits(value: Vec2) -> Vec2Bits {
    Vec2Bits {
        x_bits: FloatBits::from_f32(value.x),
        y_bits: FloatBits::from_f32(value.y),
    }
}

fn transform_bits(position: Vec2, angle: f32) -> TransformBits {
    TransformBits {
        position: vec2_bits(position),
        angle_bits: FloatBits::from_f32(angle),
    }
}

const fn feature(value: liquidfun::collision::ContactFeatureId) -> RigidContactFeature {
    RigidContactFeature {
        index_a: value.index_a(),
        index_b: value.index_b(),
        kind_a: feature_kind(value.kind_a()),
        kind_b: feature_kind(value.kind_b()),
    }
}

const fn feature_kind(kind: FeatureKind) -> RigidFeatureKind {
    match kind {
        FeatureKind::Vertex => RigidFeatureKind::Vertex,
        FeatureKind::Face => RigidFeatureKind::Face,
    }
}

struct NativeHook;

impl StepHook for NativeHook {}
