use super::{
    ActionReferences, BoundedVec, HashMap, HashSet, MAXIMUM_AGGREGATE_ITEMS, Phase9ActionState,
    Phase9ParticleDeclaration, Phase9ParticleSystemDeclaration, Phase10ActionState,
    RawActionRecord, RawBodyDeclaration, RawFixtureDeclaration, RawTimeline, RigidBodyDeclaration,
    RigidFixtureDeclaration, RigidJointDeclaration, RigidJointDefinition, RigidWorldAction,
    RigidWorldActionRecord, RigidWorldDecodeError, RigidWorldErrorKind, RigidWorldTimeline,
    RigidWorldWitnessFamily, ScenarioId, validate_action, validate_checkpoints, validate_joints,
    validate_nonnegative, validate_phase8_behavior, validate_phase9_declarations, validate_ropes,
    validate_shape, validate_transform, validation,
};

pub(super) fn validate_timeline(
    raw: RawTimeline,
) -> Result<RigidWorldTimeline, RigidWorldDecodeError> {
    let bodies = validate_bodies(raw.bodies.into_vec())?;
    let body_ids = bodies
        .iter()
        .map(|body| body.body_id.clone())
        .collect::<HashSet<_>>();
    let fixtures = validate_fixtures(raw.fixtures.into_vec(), &body_ids)?;
    let fixture_owners = fixtures
        .iter()
        .map(|fixture| (fixture.fixture_id.clone(), fixture.owner_body_id.clone()))
        .collect::<HashMap<_, _>>();
    let fixture_shapes = fixtures
        .iter()
        .map(|fixture| (fixture.fixture_id.clone(), fixture.shape.clone()))
        .collect::<HashMap<_, _>>();
    let joints = validate_joints(
        raw.joints.map_or_else(Vec::new, BoundedVec::into_vec),
        &body_ids,
    )?;
    let ropes = validate_ropes(raw.ropes.map_or_else(Vec::new, BoundedVec::into_vec))?;
    let particle_systems = raw
        .particle_systems
        .map_or_else(Vec::new, BoundedVec::into_vec);
    let particles = raw.particles.map_or_else(Vec::new, BoundedVec::into_vec);
    let (particle_system_ids, particle_owners) =
        validate_phase9_timeline_declarations(&particle_systems, &particles)?;
    let joint_ids = joints
        .iter()
        .map(|joint| joint.joint_id.clone())
        .collect::<HashSet<_>>();
    let joint_definitions = joints
        .iter()
        .map(|joint| (joint.joint_id.clone(), joint.definition.clone()))
        .collect::<HashMap<_, _>>();
    let joint_bodies = joints
        .iter()
        .map(|joint| {
            (
                joint.joint_id.clone(),
                [joint.body_a_id.clone(), joint.body_b_id.clone()],
            )
        })
        .collect::<HashMap<_, _>>();
    let gear_dependents = collect_gear_dependents(&joints);
    let rope_ids = ropes
        .iter()
        .map(|rope| rope.rope_id.clone())
        .collect::<HashSet<_>>();
    let references = ActionReferences {
        body_ids: &body_ids,
        fixture_owners: &fixture_owners,
        fixture_shapes: &fixture_shapes,
        joint_ids: &joint_ids,
        joint_definitions: &joint_definitions,
        joint_bodies: &joint_bodies,
        gear_dependents: &gear_dependents,
        rope_ids: &rope_ids,
        particle_system_ids: &particle_system_ids,
        particle_owners: &particle_owners,
    };
    let actions = validate_actions(raw.actions.into_vec(), raw.witness_family, &references)?;
    validate_phase8_behavior(
        raw.witness_family,
        &bodies,
        &fixtures,
        &joints,
        &ropes,
        &actions,
    )?;
    let checkpoints = validate_checkpoints(
        raw.checkpoints.into_vec(),
        raw.witness_family,
        &actions,
        &body_ids,
        &fixture_owners,
    )?;
    validate_timeline_aggregate_limit([
        bodies.len(),
        fixtures.len(),
        joints.len(),
        ropes.len(),
        actions.len(),
        checkpoints.len(),
    ])?;
    Ok(RigidWorldTimeline {
        witness_family: raw.witness_family,
        bodies: bodies.into_boxed_slice(),
        fixtures: fixtures.into_boxed_slice(),
        joints: joints.into_boxed_slice(),
        ropes: ropes.into_boxed_slice(),
        particle_systems: particle_systems.into_boxed_slice(),
        particles: particles.into_boxed_slice(),
        actions: actions.into_boxed_slice(),
        checkpoints: checkpoints.into_boxed_slice(),
    })
}

pub(super) fn validate_timeline_aggregate_limit(
    item_counts: [usize; 6],
) -> Result<(), RigidWorldDecodeError> {
    if item_counts.into_iter().sum::<usize>() > MAXIMUM_AGGREGATE_ITEMS {
        return Err(validation(RigidWorldErrorKind::AggregateLimitExceeded));
    }
    Ok(())
}

pub(super) fn validate_phase9_timeline_declarations(
    particle_systems: &[Phase9ParticleSystemDeclaration],
    particles: &[Phase9ParticleDeclaration],
) -> Result<(HashSet<ScenarioId>, HashMap<ScenarioId, ScenarioId>), RigidWorldDecodeError> {
    validate_phase9_declarations(particle_systems, particles)?;
    let system_ids = particle_systems
        .iter()
        .map(|system| system.system_id.clone())
        .collect();
    let particle_owners = particles
        .iter()
        .map(|particle| (particle.particle_id.clone(), particle.system_id.clone()))
        .collect();
    Ok((system_ids, particle_owners))
}

pub(super) fn collect_gear_dependents(
    joints: &[RigidJointDeclaration],
) -> HashMap<ScenarioId, Vec<ScenarioId>> {
    let mut dependents: HashMap<ScenarioId, Vec<ScenarioId>> = HashMap::new();
    for joint in joints {
        if let RigidJointDefinition::Gear {
            joint_a_id,
            joint_b_id,
            ..
        } = &joint.definition
        {
            for dependency_id in [joint_a_id, joint_b_id] {
                dependents
                    .entry(dependency_id.clone())
                    .or_default()
                    .push(joint.joint_id.clone());
            }
        }
    }
    dependents
}

pub(super) fn validate_bodies(
    raw_bodies: Vec<RawBodyDeclaration>,
) -> Result<Vec<RigidBodyDeclaration>, RigidWorldDecodeError> {
    if raw_bodies.is_empty() {
        return Err(validation(RigidWorldErrorKind::UnknownBody));
    }
    let mut ids = HashSet::with_capacity(raw_bodies.len());
    raw_bodies
        .into_iter()
        .map(|raw| {
            if !ids.insert(raw.body_id.clone()) {
                return Err(validation(RigidWorldErrorKind::DuplicateBodyId));
            }
            validate_transform(raw.transform)?;
            Ok(RigidBodyDeclaration {
                body_id: raw.body_id,
                body_kind: raw.body_kind,
                transform: raw.transform,
                active: raw.active,
            })
        })
        .collect()
}

pub(super) fn validate_fixtures(
    raw_fixtures: Vec<RawFixtureDeclaration>,
    body_ids: &HashSet<ScenarioId>,
) -> Result<Vec<RigidFixtureDeclaration>, RigidWorldDecodeError> {
    if raw_fixtures.is_empty() {
        return Err(validation(RigidWorldErrorKind::UnknownFixture));
    }
    let mut ids = HashSet::with_capacity(raw_fixtures.len());
    raw_fixtures
        .into_iter()
        .map(|raw| {
            if !ids.insert(raw.fixture_id.clone()) {
                return Err(validation(RigidWorldErrorKind::DuplicateFixtureId));
            }
            if !body_ids.contains(&raw.owner_body_id) {
                return Err(validation(RigidWorldErrorKind::InvalidOwner));
            }
            validate_shape(&raw.shape)?;
            validate_nonnegative(raw.density_bits)?;
            validate_nonnegative(raw.friction_bits)?;
            validate_nonnegative(raw.restitution_bits)?;
            Ok(RigidFixtureDeclaration {
                fixture_id: raw.fixture_id,
                owner_body_id: raw.owner_body_id,
                shape: raw.shape,
                density_bits: raw.density_bits,
                friction_bits: raw.friction_bits,
                restitution_bits: raw.restitution_bits,
                sensor: raw.sensor,
                filter: raw.filter,
            })
        })
        .collect()
}

pub(super) fn validate_actions(
    raw_actions: Vec<RawActionRecord>,
    family: RigidWorldWitnessFamily,
    references: &ActionReferences<'_>,
) -> Result<Vec<RigidWorldActionRecord>, RigidWorldDecodeError> {
    if raw_actions.is_empty() {
        return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
    }
    let mut ids = HashSet::with_capacity(raw_actions.len());
    let mut action_kinds = HashSet::new();
    let mut live_bodies = HashSet::new();
    let mut live_fixtures = HashSet::new();
    let mut created_bodies = HashSet::new();
    let mut created_fixtures = HashSet::new();
    let mut live_joints = HashSet::new();
    let mut live_ropes = HashSet::new();
    let mut created_joints = HashSet::new();
    let mut created_ropes = HashSet::new();
    let mut phase9_state = Phase9ActionState::default();
    let mut phase10_state = Phase10ActionState::default();
    let reserved_semantic_ids = references
        .body_ids
        .iter()
        .chain(references.fixture_owners.keys())
        .chain(references.joint_ids.iter())
        .chain(references.rope_ids.iter())
        .chain(references.particle_system_ids.iter())
        .chain(references.particle_owners.keys())
        .cloned()
        .collect::<HashSet<_>>();
    let mut actions = Vec::with_capacity(raw_actions.len());

    for raw in raw_actions {
        let phase = raw.phase.into_string();
        if phase.trim().is_empty() {
            return Err(validation(RigidWorldErrorKind::CheckpointPhaseMismatch));
        }
        if matches!(&raw.action, RigidWorldAction::ParticleGroup { .. }) && phase != "phase10" {
            return Err(validation(RigidWorldErrorKind::CheckpointPhaseMismatch));
        }
        if !ids.insert(raw.action_id.clone()) {
            return Err(validation(RigidWorldErrorKind::DuplicateActionId));
        }
        validate_action(
            &raw.action,
            references.body_ids,
            references.fixture_owners,
            references.fixture_shapes,
            &mut live_bodies,
            &mut live_fixtures,
            &mut created_bodies,
            &mut created_fixtures,
            references.joint_ids,
            references.joint_definitions,
            references.joint_bodies,
            references.gear_dependents,
            references.rope_ids,
            &mut live_joints,
            &mut live_ropes,
            &mut created_joints,
            &mut created_ropes,
            references.particle_system_ids,
            references.particle_owners,
            &mut phase9_state,
            &reserved_semantic_ids,
            &mut phase10_state,
        )?;
        action_kinds.insert(raw.action.action_kind());
        actions.push(RigidWorldActionRecord {
            action_id: raw.action_id,
            phase: phase.into_boxed_str(),
            action: raw.action,
        });
    }

    if !live_bodies.is_empty()
        || !live_fixtures.is_empty()
        || created_bodies.len() != references.body_ids.len()
        || created_fixtures.len() != references.fixture_owners.len()
        || !live_joints.is_empty()
        || !live_ropes.is_empty()
        || created_joints.len() != references.joint_ids.len()
        || created_ropes.len() != references.rope_ids.len()
        || !phase9_state.live_systems.is_empty()
        || !phase9_state.live_particles.is_empty()
        || !phase9_state.pending_particles.is_empty()
        || !phase10_state.is_closed()
        || family
            .required_action_kinds()
            .iter()
            .any(|kind| !action_kinds.contains(kind))
    {
        return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
    }
    Ok(actions)
}
