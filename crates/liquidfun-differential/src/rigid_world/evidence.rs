//! Checkpoint capture and source-ordered semantic evidence.

use super::{
    BodyId, DestroyedId, DestructionCause, DestructionRecord, FixtureId, FloatBits,
    ManagedContactSnapshot, ManifoldKind, NativeRigidWorldError, Observation, RigidBodyDeclaration,
    RigidBodySnapshot, RigidContactEvent, RigidContactEventKind, RigidContactResult,
    RigidDestructionRecord, RigidExpectedCheckpoint, RigidExpectedCounts, RigidFixtureDeclaration,
    RigidFixtureSnapshot, RigidManifoldKind, RigidManifoldPoint, RigidManifoldResult,
    RigidWorldTimeline, RigidWorldWitness, RigidWorldWitnessFamily, ScenarioId, StepReport,
    TimelineExecutor, Vec2, checked_u32, declaration_error, feature, rigid_body_kind, rigid_filter,
    transform_bits, vec2_bits,
};

pub(super) fn capture_checkpoint(
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
    let contact_diagnostics = executor.world.rigid_contact_diagnostics();
    let contacts = contact_diagnostics
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
        observations: std::mem::take(&mut executor.semantic_observations).into_boxed_slice(),
    };
    validate_checkpoint(
        expected,
        &result,
        &executor.observations,
        RigidWorldWitnessFamily::REQUIRED.contains(&timeline.witness_family()),
    )?;
    executor.observations.clear();
    Ok(result)
}

pub(super) fn collect_step_report(
    executor: &mut TimelineExecutor,
    report: &StepReport,
) -> Result<(), NativeRigidWorldError> {
    let mut collected_hook_occurrences = Vec::new();
    for transition in report.contact_transitions() {
        collect_transitions(executor, std::slice::from_ref(transition))?;
        let manager_occurrence = transition.contact().differential_occurrence();
        let maybe_hook = report.events().iter().find(|event| {
            event.maybe_pre_solve().is_some()
                && event.contact().differential_occurrence() == manager_occurrence
        });
        if let Some(event) = maybe_hook {
            let contact = executor.contact_identity(event.contact())?;
            executor.events.push(RigidContactEvent {
                kind: RigidContactEventKind::PreSolve,
                contact,
            });
            collected_hook_occurrences.push(manager_occurrence);
        }
    }
    for event in report.events() {
        if event.maybe_pre_solve().is_some()
            && !collected_hook_occurrences.contains(&event.contact().differential_occurrence())
        {
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
    collect_continuous_solves(executor, report.continuous_contact_solves())?;
    Ok(())
}

pub(super) fn collect_continuous_solves(
    executor: &mut TimelineExecutor,
    solves: &[liquidfun::ContactSolve],
) -> Result<(), NativeRigidWorldError> {
    for solve in solves {
        let contact = executor.contact_identity(solve.contact())?;
        executor.events.push(RigidContactEvent {
            kind: RigidContactEventKind::PreSolve,
            contact: contact.clone(),
        });
        executor.events.push(RigidContactEvent {
            kind: RigidContactEventKind::PostSolve,
            contact,
        });
    }
    Ok(())
}

pub(super) fn collect_direct_transitions(
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
        let manager_occurrence = transition.contact().differential_occurrence();
        let identity = executor.contact_identity(transition.contact())?;
        executor.maybe_last_contact = Some(identity.clone());
        if transition.kind() == liquidfun::ContactTransitionKind::Begin
            && !executor
                .seen_manager_occurrences
                .contains(&manager_occurrence)
        {
            executor.seen_manager_occurrences.push(manager_occurrence);
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

pub(super) fn observe_step(executor: &mut TimelineExecutor, phase: &str) {
    let maybe_contact = executor.maybe_last_contact.clone();
    let witnesses: &[RigidWorldWitness] = match phase {
        "static-kinematic-admission" => &[RigidWorldWitness::StaticKinematicOverlapRejected],
        "kinematic-kinematic-admission" => &[RigidWorldWitness::KinematicKinematicOverlapRejected],
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
    enforce_runtime_transitions: bool,
) -> Result<(), NativeRigidWorldError> {
    if expected.counts() != actual.counts {
        return Err(declaration_error(
            expected,
            format!(
                "semantic counts differ: expected {:?}, actual {:?}",
                expected.counts(),
                actual.counts
            ),
        ));
    }
    // Phase 7 transition declarations close the witness registry and pin any
    // relevant contact identities. Their runtime behavior is represented by
    // typed semantic observations and compared as differential evidence.
    if !enforce_runtime_transitions {
        return Ok(());
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
    executor: &mut TimelineExecutor,
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

pub(super) fn push_object_destruction(
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

pub(super) fn remove_destroyed_mapping(executor: &mut TimelineExecutor, destroyed: DestroyedId) {
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
