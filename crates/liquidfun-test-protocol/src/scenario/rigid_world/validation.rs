use std::collections::{HashMap, HashSet};

use serde::Deserialize;

mod geometry;
mod phase8;

use geometry::{
    validate_nonnegative, validate_positive, validate_shape, validate_transform, validate_vec2,
};
use phase8::validate_phase8_behavior;

use super::{
    PHASE9_MAXIMUM_IDENTITIES, PHASE9_MAXIMUM_PARTICLE_SYSTEMS, PHASE9_MAXIMUM_PARTICLES,
    Phase9ParticleAction, Phase9ParticleDeclaration, Phase9ParticleSystemDeclaration,
    Phase10ActionState, Phase10Operation, RIGID_WORLD_MAXIMUM_ACTIONS,
    RIGID_WORLD_MAXIMUM_CONTINUOUS_WORK, RIGID_WORLD_MAXIMUM_DIRECTIVES,
    RIGID_WORLD_MAXIMUM_ITERATIONS, RIGID_WORLD_MAXIMUM_JOINTS, RIGID_WORLD_MAXIMUM_ROPE_VERTICES,
    RIGID_WORLD_MAXIMUM_ROPES, RIGID_WORLD_POSITION_ITERATIONS, RIGID_WORLD_TIMESTEP_BITS,
    RIGID_WORLD_VELOCITY_ITERATIONS, RigidAabbBits, RigidBodyDeclaration, RigidBodyKind,
    RigidContactDirectiveTarget, RigidContactIdentity, RigidExpectedCheckpoint,
    RigidExpectedCounts, RigidExpectedTransition, RigidFilterBits, RigidFixtureChildSelector,
    RigidFixtureDeclaration, RigidFixtureShape, RigidJointDeclaration, RigidJointDefinition,
    RigidJointKind, RigidJointMutation, RigidPreSolveDirective, RigidQueryDirectiveRule,
    RigidRayDirective, RigidRayDirectiveRule, RigidRopeDeclaration, RigidWorldAction,
    RigidWorldActionRecord, RigidWorldDecodeError, RigidWorldErrorKind, RigidWorldRequestKind,
    RigidWorldRequestRecord, RigidWorldScenario, RigidWorldTimeline, RigidWorldWitness,
    RigidWorldWitnessFamily, validation,
};
use crate::{
    FloatBits, HarnessLimits, ProtocolVersion, RecordLimit, RequestId, ScenarioId,
    ScenarioSchemaVersion, ScenarioSource, Sha256Hex, ToleranceProfileVersion, TraceSchemaVersion,
    TransformBits,
    codec::{BoundedString, BoundedVec, decode_jsonl},
};

const MAXIMUM_ID_BYTES: usize = 128;
const MAXIMUM_STRING_BYTES: usize = 4 * 1024;
const MAXIMUM_TIMELINES: usize = 19;
const MAXIMUM_BODIES: usize = 64;
const MAXIMUM_FIXTURES: usize = 128;
const MAXIMUM_CHECKPOINTS: usize = 64;
const MAXIMUM_TRANSITIONS: usize = 64;
const MAXIMUM_POLYGON_VERTICES: usize = 8;
const MAXIMUM_AGGREGATE_ITEMS: usize = 2_048;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRequest {
    protocol_version: ProtocolVersion,
    record_kind: RigidWorldRequestKind,
    request_id: BoundedString<MAXIMUM_ID_BYTES>,
    scenario_schema_version: ScenarioSchemaVersion,
    requested_trace_schema_version: TraceSchemaVersion,
    tolerance_profile_version: ToleranceProfileVersion,
    tolerance_profile_sha256: Sha256Hex,
    scenario: RawScenario,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawScenario {
    scenario_id: BoundedString<MAXIMUM_ID_BYTES>,
    source: RawSource,
    timelines: BoundedVec<RawTimeline, MAXIMUM_TIMELINES>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RawSource {
    Named {
        name: BoundedString<MAXIMUM_STRING_BYTES>,
    },
    Seeded {
        generator_id: BoundedString<MAXIMUM_STRING_BYTES>,
        generator_version: u32,
        seed: u64,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTimeline {
    witness_family: RigidWorldWitnessFamily,
    bodies: BoundedVec<RawBodyDeclaration, MAXIMUM_BODIES>,
    fixtures: BoundedVec<RawFixtureDeclaration, MAXIMUM_FIXTURES>,
    joints: Option<BoundedVec<RigidJointDeclaration, RIGID_WORLD_MAXIMUM_JOINTS>>,
    ropes: Option<BoundedVec<RawRopeDeclaration, RIGID_WORLD_MAXIMUM_ROPES>>,
    particle_systems:
        Option<BoundedVec<Phase9ParticleSystemDeclaration, PHASE9_MAXIMUM_PARTICLE_SYSTEMS>>,
    particles: Option<BoundedVec<Phase9ParticleDeclaration, PHASE9_MAXIMUM_PARTICLES>>,
    actions: BoundedVec<RawActionRecord, RIGID_WORLD_MAXIMUM_ACTIONS>,
    checkpoints: BoundedVec<RawCheckpoint, MAXIMUM_CHECKPOINTS>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBodyDeclaration {
    body_id: ScenarioId,
    body_kind: RigidBodyKind,
    transform: TransformBits,
    active: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFixtureDeclaration {
    fixture_id: ScenarioId,
    owner_body_id: ScenarioId,
    shape: RigidFixtureShape,
    density_bits: FloatBits,
    friction_bits: FloatBits,
    restitution_bits: FloatBits,
    sensor: bool,
    filter: RigidFilterBits,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRopeDeclaration {
    rope_id: ScenarioId,
    vertices: BoundedVec<crate::Vec2Bits, RIGID_WORLD_MAXIMUM_ROPE_VERTICES>,
    masses_bits: BoundedVec<FloatBits, RIGID_WORLD_MAXIMUM_ROPE_VERTICES>,
    gravity: crate::Vec2Bits,
    damping_bits: FloatBits,
    stretch_stiffness_bits: FloatBits,
    bend_stiffness_bits: FloatBits,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawActionRecord {
    action_id: ScenarioId,
    phase: BoundedString<MAXIMUM_STRING_BYTES>,
    action: RigidWorldAction,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCheckpoint {
    checkpoint_id: ScenarioId,
    after_action_id: ScenarioId,
    phase: BoundedString<MAXIMUM_STRING_BYTES>,
    counts: RigidExpectedCounts,
    transitions: BoundedVec<RawTransition, MAXIMUM_TRANSITIONS>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTransition {
    witness: RigidWorldWitness,
    maybe_contact: Option<RigidContactIdentity>,
}

struct ActionReferences<'a> {
    body_ids: &'a HashSet<ScenarioId>,
    fixture_owners: &'a HashMap<ScenarioId, ScenarioId>,
    fixture_shapes: &'a HashMap<ScenarioId, RigidFixtureShape>,
    joint_ids: &'a HashSet<ScenarioId>,
    joint_definitions: &'a HashMap<ScenarioId, RigidJointDefinition>,
    joint_bodies: &'a HashMap<ScenarioId, [ScenarioId; 2]>,
    gear_dependents: &'a HashMap<ScenarioId, Vec<ScenarioId>>,
    rope_ids: &'a HashSet<ScenarioId>,
    particle_system_ids: &'a HashSet<ScenarioId>,
    particle_owners: &'a HashMap<ScenarioId, ScenarioId>,
}

#[derive(Default)]
struct Phase9ActionState {
    created_systems: HashSet<ScenarioId>,
    live_systems: HashSet<ScenarioId>,
    created_particles: HashSet<ScenarioId>,
    live_particles: HashSet<ScenarioId>,
    pending_particles: HashSet<ScenarioId>,
}

/// Decodes one newline-complete bounded rigid-world request record.
///
/// # Errors
///
/// Returns [`RigidWorldDecodeError`] for framing, closed-field, declaration,
/// lifecycle, checkpoint, witness, contact-identity, or resource-limit failures.
pub fn decode_rigid_world_request_jsonl(
    bytes: &[u8],
    limits: &HarnessLimits,
) -> Result<RigidWorldRequestRecord, RigidWorldDecodeError> {
    let raw = decode_jsonl::<RawRequest>(bytes, limits, RecordLimit::Input)?;
    validate_request(raw)
}

fn validate_request(raw: RawRequest) -> Result<RigidWorldRequestRecord, RigidWorldDecodeError> {
    let request_id = RequestId::new(raw.request_id.into_string())
        .map_err(|_| validation(RigidWorldErrorKind::InvalidIdentifier))?;
    let scenario_id = ScenarioId::new(raw.scenario.scenario_id.into_string())
        .map_err(|_| validation(RigidWorldErrorKind::InvalidIdentifier))?;
    let source = validate_source(raw.scenario.source)?;
    let raw_timelines = raw.scenario.timelines.into_vec();
    if raw_timelines.is_empty() {
        return Err(validation(RigidWorldErrorKind::NoTimelines));
    }

    let mut families = HashSet::with_capacity(raw_timelines.len());
    let mut timelines = Vec::with_capacity(raw_timelines.len());
    for raw_timeline in raw_timelines {
        if !families.insert(raw_timeline.witness_family) {
            return Err(validation(RigidWorldErrorKind::DuplicateWitnessFamily));
        }
        timelines.push(validate_timeline(raw_timeline)?);
    }
    if RigidWorldWitnessFamily::ALL
        .iter()
        .any(|family| !families.contains(family))
    {
        return Err(validation(RigidWorldErrorKind::MissingWitnessFamily));
    }

    Ok(RigidWorldRequestRecord {
        protocol_version: raw.protocol_version,
        record_kind: raw.record_kind,
        request_id,
        scenario_schema_version: raw.scenario_schema_version,
        requested_trace_schema_version: raw.requested_trace_schema_version,
        tolerance_profile_version: raw.tolerance_profile_version,
        tolerance_profile_sha256: raw.tolerance_profile_sha256,
        scenario: RigidWorldScenario {
            scenario_id,
            source,
            timelines: timelines.into_boxed_slice(),
        },
    })
}

fn validate_timeline(raw: RawTimeline) -> Result<RigidWorldTimeline, RigidWorldDecodeError> {
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

fn validate_timeline_aggregate_limit(item_counts: [usize; 6]) -> Result<(), RigidWorldDecodeError> {
    if item_counts.into_iter().sum::<usize>() > MAXIMUM_AGGREGATE_ITEMS {
        return Err(validation(RigidWorldErrorKind::AggregateLimitExceeded));
    }
    Ok(())
}

fn validate_phase9_timeline_declarations(
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

fn collect_gear_dependents(
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

fn validate_bodies(
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

fn validate_fixtures(
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

fn validate_actions(
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
    let declared_particle_ids = references
        .particle_owners
        .keys()
        .cloned()
        .collect::<HashSet<_>>();
    let mut actions = Vec::with_capacity(raw_actions.len());

    for raw in raw_actions {
        let phase = raw.phase.into_string();
        if phase.trim().is_empty() {
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
            &declared_particle_ids,
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

#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "one explicit lifecycle transition function keeps the closed action registry auditable"
)]
fn validate_action(
    action: &RigidWorldAction,
    body_ids: &HashSet<ScenarioId>,
    fixture_owners: &HashMap<ScenarioId, ScenarioId>,
    fixture_shapes: &HashMap<ScenarioId, RigidFixtureShape>,
    live_bodies: &mut HashSet<ScenarioId>,
    live_fixtures: &mut HashSet<ScenarioId>,
    created_bodies: &mut HashSet<ScenarioId>,
    created_fixtures: &mut HashSet<ScenarioId>,
    joint_ids: &HashSet<ScenarioId>,
    joint_definitions: &HashMap<ScenarioId, RigidJointDefinition>,
    joint_bodies: &HashMap<ScenarioId, [ScenarioId; 2]>,
    gear_dependents: &HashMap<ScenarioId, Vec<ScenarioId>>,
    rope_ids: &HashSet<ScenarioId>,
    live_joints: &mut HashSet<ScenarioId>,
    live_ropes: &mut HashSet<ScenarioId>,
    created_joints: &mut HashSet<ScenarioId>,
    created_ropes: &mut HashSet<ScenarioId>,
    particle_system_ids: &HashSet<ScenarioId>,
    particle_owners: &HashMap<ScenarioId, ScenarioId>,
    phase9_state: &mut Phase9ActionState,
    declared_particle_ids: &HashSet<ScenarioId>,
    phase10_state: &mut Phase10ActionState,
) -> Result<(), RigidWorldDecodeError> {
    match action {
        RigidWorldAction::Particle { action } => {
            validate_phase9_action(action, particle_system_ids, particle_owners, phase9_state)?;
        }
        RigidWorldAction::ParticleGroup { operation } => {
            validate_phase10_action(
                operation,
                particle_system_ids,
                &phase9_state.live_systems,
                declared_particle_ids,
                phase10_state,
            )?;
        }
        RigidWorldAction::CreateBody { body_id } => {
            if !body_ids.contains(body_id)
                || !created_bodies.insert(body_id.clone())
                || !live_bodies.insert(body_id.clone())
            {
                return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
            }
        }
        RigidWorldAction::CreateFixture { fixture_id } => {
            let Some(owner) = fixture_owners.get(fixture_id) else {
                return Err(validation(RigidWorldErrorKind::UnknownFixture));
            };
            if !live_bodies.contains(owner)
                || !created_fixtures.insert(fixture_id.clone())
                || !live_fixtures.insert(fixture_id.clone())
            {
                return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
            }
        }
        RigidWorldAction::InspectBody { body_id }
        | RigidWorldAction::ResetMassData { body_id }
        | RigidWorldAction::SetBodyType { body_id, .. }
        | RigidWorldAction::SetBodyActive { body_id, .. }
        | RigidWorldAction::SetFixedRotation { body_id, .. }
        | RigidWorldAction::SetSleepingAllowed { body_id, .. }
        | RigidWorldAction::SetAwake { body_id, .. }
        | RigidWorldAction::SetBullet { body_id, .. } => {
            require_live(body_id, live_bodies, RigidWorldErrorKind::UnknownBody)?;
        }
        RigidWorldAction::SetLinearVelocity { body_id, velocity } => {
            require_live(body_id, live_bodies, RigidWorldErrorKind::UnknownBody)?;
            validate_vec2(*velocity)?;
        }
        RigidWorldAction::SetAngularVelocity {
            body_id,
            angular_velocity_bits,
        }
        | RigidWorldAction::ApplyTorque {
            body_id,
            torque_bits: angular_velocity_bits,
            ..
        }
        | RigidWorldAction::ApplyAngularImpulse {
            body_id,
            impulse_bits: angular_velocity_bits,
            ..
        }
        | RigidWorldAction::SetGravityScale {
            body_id,
            gravity_scale_bits: angular_velocity_bits,
        } => {
            require_live(body_id, live_bodies, RigidWorldErrorKind::UnknownBody)?;
            validate_finite(
                *angular_velocity_bits,
                RigidWorldErrorKind::InvalidBodyControl,
            )?;
        }
        RigidWorldAction::ApplyForce {
            body_id,
            force,
            point,
            ..
        }
        | RigidWorldAction::ApplyLinearImpulse {
            body_id,
            impulse: force,
            point,
            ..
        } => {
            require_live(body_id, live_bodies, RigidWorldErrorKind::UnknownBody)?;
            validate_vec2(*force)?;
            validate_vec2(*point)?;
        }
        RigidWorldAction::SetBodyDamping {
            body_id,
            linear_damping_bits,
            angular_damping_bits,
        } => {
            require_live(body_id, live_bodies, RigidWorldErrorKind::UnknownBody)?;
            validate_nonnegative(*linear_damping_bits)?;
            validate_nonnegative(*angular_damping_bits)?;
        }
        RigidWorldAction::SetBodyTransform { body_id, transform } => {
            require_live(body_id, live_bodies, RigidWorldErrorKind::UnknownBody)?;
            validate_transform(*transform)?;
        }
        RigidWorldAction::SetCustomMassData {
            body_id,
            mass_bits,
            center,
            inertia_bits,
        } => {
            require_live(body_id, live_bodies, RigidWorldErrorKind::UnknownBody)?;
            validate_custom_mass(*mass_bits, *center, *inertia_bits)?;
        }
        RigidWorldAction::InspectFixture { fixture_id }
        | RigidWorldAction::SetFixtureSensor { fixture_id, .. }
        | RigidWorldAction::SetFixtureFilter { fixture_id, .. } => {
            require_live(
                fixture_id,
                live_fixtures,
                RigidWorldErrorKind::UnknownFixture,
            )?;
        }
        RigidWorldAction::SetFixtureMaterial {
            fixture_id,
            friction_bits,
            restitution_bits,
        } => {
            require_live(
                fixture_id,
                live_fixtures,
                RigidWorldErrorKind::UnknownFixture,
            )?;
            validate_nonnegative(*friction_bits)?;
            validate_nonnegative(*restitution_bits)?;
        }
        RigidWorldAction::SetFixtureDensity {
            fixture_id,
            density_bits,
        } => {
            require_live(
                fixture_id,
                live_fixtures,
                RigidWorldErrorKind::UnknownFixture,
            )?;
            validate_nonnegative(*density_bits)?;
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
                return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
            }
        }
        RigidWorldAction::SetWorldGravity { gravity }
        | RigidWorldAction::ShiftOrigin { shift: gravity } => {
            validate_vec2(*gravity)?;
        }
        RigidWorldAction::SetAutomaticForceClearing { .. }
        | RigidWorldAction::SetWarmStarting { .. }
        | RigidWorldAction::SetContinuousPhysics { .. }
        | RigidWorldAction::SetSubStepping { .. }
        | RigidWorldAction::ClearForces
        | RigidWorldAction::RequestReconstruction
        | RigidWorldAction::RequestDiagnostics => {}
        RigidWorldAction::ConfiguredStep {
            timestep_bits,
            velocity_iterations,
            position_iterations,
            continuous_work_budget,
        } => {
            let timestep = timestep_bits.to_f32();
            if !timestep.is_finite()
                || timestep <= 0.0
                || !(1..=RIGID_WORLD_MAXIMUM_ITERATIONS).contains(velocity_iterations)
                || !(1..=RIGID_WORLD_MAXIMUM_ITERATIONS).contains(position_iterations)
                || !(1..=RIGID_WORLD_MAXIMUM_CONTINUOUS_WORK).contains(continuous_work_budget)
            {
                return Err(validation(RigidWorldErrorKind::InvalidStepConfiguration));
            }
        }
        RigidWorldAction::QueryAabb {
            aabb,
            directive_rules,
        } => {
            validate_aabb(*aabb)?;
            validate_query_rules(directive_rules, live_fixtures, fixture_shapes)?;
        }
        RigidWorldAction::RayCast {
            start,
            end,
            directive_rules,
        } => {
            validate_vec2(*start)?;
            validate_vec2(*end)?;
            validate_ray_geometry(*start, *end)?;
            validate_ray_rules(directive_rules, live_fixtures, fixture_shapes)?;
        }
        RigidWorldAction::CreateJoint { joint_id } => {
            if !joint_ids.contains(joint_id)
                || !created_joints.insert(joint_id.clone())
                || !live_joints.insert(joint_id.clone())
            {
                return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
            }
        }
        RigidWorldAction::InspectJoint { joint_id } => {
            require_live(joint_id, live_joints, RigidWorldErrorKind::UnknownJoint)?;
        }
        RigidWorldAction::MutateJoint { joint_id, mutation } => {
            require_live(joint_id, live_joints, RigidWorldErrorKind::UnknownJoint)?;
            let Some(joint_definition) = joint_definitions.get(joint_id) else {
                return Err(validation(RigidWorldErrorKind::UnknownJoint));
            };
            validate_joint_mutation(joint_definition.joint_kind(), *mutation)?;
            if !joint_mutation_changes_definition(joint_definition, *mutation) {
                return Err(validation(RigidWorldErrorKind::InvalidJointDefinition));
            }
        }
        RigidWorldAction::DestroyJoint { joint_id } => {
            if !live_joints.contains(joint_id) {
                return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
            }
            remove_joint_cascade(joint_id, live_joints, gear_dependents);
        }
        RigidWorldAction::CreateRope { rope_id } => {
            if !rope_ids.contains(rope_id)
                || !created_ropes.insert(rope_id.clone())
                || !live_ropes.insert(rope_id.clone())
            {
                return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
            }
        }
        RigidWorldAction::SetRopeAngle {
            rope_id,
            angle_bits,
        } => {
            require_live(rope_id, live_ropes, RigidWorldErrorKind::UnknownRope)?;
            validate_finite(*angle_bits, RigidWorldErrorKind::InvalidRopeDefinition)?;
        }
        RigidWorldAction::StepRope {
            rope_id,
            timestep_bits,
            iterations,
        } => {
            require_live(rope_id, live_ropes, RigidWorldErrorKind::UnknownRope)?;
            let timestep = timestep_bits.to_f32();
            if !timestep.is_finite()
                || timestep <= 0.0
                || !(1..=RIGID_WORLD_MAXIMUM_ITERATIONS).contains(iterations)
            {
                return Err(validation(RigidWorldErrorKind::InvalidRopeDefinition));
            }
        }
        RigidWorldAction::InspectRope { rope_id } => {
            require_live(rope_id, live_ropes, RigidWorldErrorKind::UnknownRope)?;
        }
        RigidWorldAction::DestroyRope { rope_id } => {
            if !live_ropes.remove(rope_id) {
                return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
            }
        }
        RigidWorldAction::SetContactFilterDirective { target, .. }
        | RigidWorldAction::SetPreSolveDirective { target, .. } => {
            validate_contact_directive_target(target, live_fixtures)?;
            if let RigidWorldAction::SetPreSolveDirective { directive, .. } = action {
                validate_pre_solve_directive(*directive)?;
            }
        }
        RigidWorldAction::DestroyFixture { fixture_id } => {
            if !live_fixtures.remove(fixture_id) {
                return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
            }
        }
        RigidWorldAction::DestroyBody { body_id } => {
            if !live_bodies.remove(body_id) {
                return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
            }
            live_fixtures.retain(|fixture_id| fixture_owners.get(fixture_id) != Some(body_id));
            let attached = live_joints
                .iter()
                .filter(|joint_id| {
                    joint_bodies
                        .get(*joint_id)
                        .is_some_and(|endpoints| endpoints.contains(body_id))
                })
                .cloned()
                .collect::<Vec<_>>();
            for joint_id in attached {
                remove_joint_cascade(&joint_id, live_joints, gear_dependents);
            }
        }
    }
    Ok(())
}

fn validate_phase10_action(
    operation: &Phase10Operation,
    particle_system_ids: &HashSet<ScenarioId>,
    live_system_ids: &HashSet<ScenarioId>,
    particle_ids: &HashSet<ScenarioId>,
    state: &mut Phase10ActionState,
) -> Result<(), RigidWorldDecodeError> {
    state
        .apply(
            operation,
            particle_system_ids,
            live_system_ids,
            particle_ids,
        )
        .map_err(|_| validation(RigidWorldErrorKind::InvalidParticleGroupAction))
}

fn validate_phase9_declarations(
    systems: &[Phase9ParticleSystemDeclaration],
    particles: &[Phase9ParticleDeclaration],
) -> Result<(), RigidWorldDecodeError> {
    let mut system_ids = HashSet::with_capacity(systems.len());
    for system in systems {
        let capacity = system.buffer_mode.capacity();
        if !system_ids.insert(system.system_id.clone())
            || capacity == 0
            || capacity > PHASE9_MAXIMUM_PARTICLES
            || system
                .maximum_count
                .is_some_and(|maximum| maximum == 0 || maximum > PHASE9_MAXIMUM_PARTICLES)
        {
            return Err(validation(RigidWorldErrorKind::InvalidParticleDefinition));
        }
        validate_positive(system.density_bits)?;
        validate_finite(
            system.gravity_scale_bits,
            RigidWorldErrorKind::InvalidParticleDefinition,
        )?;
        validate_positive(system.radius_bits)?;
        validate_nonnegative(system.damping_bits)?;
        validate_positive(system.lifetime_granularity_bits)?;
    }

    let mut particle_ids = HashSet::with_capacity(particles.len());
    for particle in particles {
        if !particle_ids.insert(particle.particle_id.clone())
            || !system_ids.contains(&particle.system_id)
        {
            return Err(validation(RigidWorldErrorKind::InvalidParticleDefinition));
        }
        validate_vec2(particle.position)?;
        validate_vec2(particle.velocity)?;
        validate_finite(
            particle.lifetime_bits,
            RigidWorldErrorKind::InvalidParticleDefinition,
        )?;
    }
    Ok(())
}

fn validate_phase9_action(
    action: &Phase9ParticleAction,
    system_ids: &HashSet<ScenarioId>,
    particle_owners: &HashMap<ScenarioId, ScenarioId>,
    state: &mut Phase9ActionState,
) -> Result<(), RigidWorldDecodeError> {
    validate_phase9_action_shape(action)?;
    match action {
        Phase9ParticleAction::CreateSystem { system_id } => {
            if !system_ids.contains(system_id)
                || !state.created_systems.insert(system_id.clone())
                || !state.live_systems.insert(system_id.clone())
            {
                return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
            }
        }
        Phase9ParticleAction::DestroySystem { system_id } => {
            if !state.live_systems.remove(system_id) {
                return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
            }
            state
                .live_particles
                .retain(|particle_id| particle_owners.get(particle_id) != Some(system_id));
            state
                .pending_particles
                .retain(|particle_id| particle_owners.get(particle_id) != Some(system_id));
        }
        Phase9ParticleAction::CreateParticle { particle_id } => {
            let Some(owner) = particle_owners.get(particle_id) else {
                return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
            };
            if !state.live_systems.contains(owner)
                || !state.created_particles.insert(particle_id.clone())
                || !state.live_particles.insert(particle_id.clone())
            {
                return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
            }
        }
        Phase9ParticleAction::InspectSystem { system_id }
        | Phase9ParticleAction::InspectParticleContact { system_id, .. }
        | Phase9ParticleAction::InspectBodyContact { system_id, .. }
        | Phase9ParticleAction::SetPaused { system_id, .. }
        | Phase9ParticleAction::Compact { system_id }
        | Phase9ParticleAction::RequestStatistics { system_id } => {
            require_live_phase9_system(system_id, state)?;
            if matches!(action, Phase9ParticleAction::Compact { .. }) {
                state
                    .pending_particles
                    .retain(|particle_id| particle_owners.get(particle_id) != Some(system_id));
            }
        }
        Phase9ParticleAction::InspectParticle { particle_id }
        | Phase9ParticleAction::SetPosition { particle_id, .. }
        | Phase9ParticleAction::SetVelocity { particle_id, .. } => {
            require_live_phase9_particle(particle_id, particle_owners, state)?;
        }
        Phase9ParticleAction::MarkForDestruction { particle_id } => {
            require_live_phase9_particle(particle_id, particle_owners, state)?;
            state.live_particles.remove(particle_id);
            state.pending_particles.insert(particle_id.clone());
        }
        Phase9ParticleAction::ApplyForce { particle_ids, .. }
        | Phase9ParticleAction::ApplyImpulse { particle_ids, .. } => {
            let mut maybe_owner: Option<&ScenarioId> = None;
            for particle_id in particle_ids {
                let owner = require_live_phase9_particle(particle_id, particle_owners, state)?;
                if maybe_owner.is_some_and(|expected| expected != owner) {
                    return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
                }
                maybe_owner = Some(owner);
            }
        }
        Phase9ParticleAction::QueryAabb { system_id, .. }
        | Phase9ParticleAction::RayCast { system_id, .. } => {
            if let Some(system_id) = system_id {
                require_live_phase9_system(system_id, state)?;
            }
        }
        Phase9ParticleAction::InspectOccurrence { .. } => {}
    }
    Ok(())
}

fn validate_phase9_action_shape(
    action: &Phase9ParticleAction,
) -> Result<(), RigidWorldDecodeError> {
    match action {
        Phase9ParticleAction::SetPosition { position, .. }
        | Phase9ParticleAction::SetVelocity {
            velocity: position, ..
        } => validate_vec2(*position)?,
        Phase9ParticleAction::ApplyForce {
            particle_ids,
            force,
        }
        | Phase9ParticleAction::ApplyImpulse {
            particle_ids,
            impulse: force,
        } => {
            if particle_ids.is_empty() || particle_ids.len() > PHASE9_MAXIMUM_IDENTITIES {
                return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
            }
            let mut ids = HashSet::with_capacity(particle_ids.len());
            if particle_ids.iter().any(|id| !ids.insert(id)) {
                return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
            }
            validate_vec2(*force)?;
        }
        Phase9ParticleAction::QueryAabb { lower, upper, .. } => {
            validate_aabb(RigidAabbBits {
                lower: *lower,
                upper: *upper,
            })?;
        }
        Phase9ParticleAction::RayCast { start, end, .. } => {
            validate_ray_geometry(*start, *end)?;
        }
        Phase9ParticleAction::InspectParticleContact { contact_index, .. }
        | Phase9ParticleAction::InspectBodyContact { contact_index, .. } => {
            if *contact_index >= PHASE9_MAXIMUM_IDENTITIES {
                return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
            }
        }
        Phase9ParticleAction::InspectOccurrence { occurrence_index } => {
            if *occurrence_index >= PHASE9_MAXIMUM_IDENTITIES {
                return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
            }
        }
        Phase9ParticleAction::CreateSystem { .. }
        | Phase9ParticleAction::DestroySystem { .. }
        | Phase9ParticleAction::CreateParticle { .. }
        | Phase9ParticleAction::InspectSystem { .. }
        | Phase9ParticleAction::InspectParticle { .. }
        | Phase9ParticleAction::SetPaused { .. }
        | Phase9ParticleAction::MarkForDestruction { .. }
        | Phase9ParticleAction::Compact { .. }
        | Phase9ParticleAction::RequestStatistics { .. } => {}
    }
    Ok(())
}

fn require_live_phase9_system(
    system_id: &ScenarioId,
    state: &Phase9ActionState,
) -> Result<(), RigidWorldDecodeError> {
    if !state.live_systems.contains(system_id) {
        return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
    }
    Ok(())
}

fn require_live_phase9_particle<'a>(
    particle_id: &ScenarioId,
    particle_owners: &'a HashMap<ScenarioId, ScenarioId>,
    state: &Phase9ActionState,
) -> Result<&'a ScenarioId, RigidWorldDecodeError> {
    let Some(owner) = particle_owners.get(particle_id) else {
        return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
    };
    if !state.live_systems.contains(owner) || !state.live_particles.contains(particle_id) {
        return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
    }
    Ok(owner)
}

fn validate_joints(
    joints: Vec<RigidJointDeclaration>,
    body_ids: &HashSet<ScenarioId>,
) -> Result<Vec<RigidJointDeclaration>, RigidWorldDecodeError> {
    let mut ids = HashSet::with_capacity(joints.len());
    let mut kinds = HashMap::with_capacity(joints.len());
    let mut endpoints: HashMap<ScenarioId, [ScenarioId; 2]> = HashMap::with_capacity(joints.len());
    for joint in &joints {
        if ids.contains(&joint.joint_id) {
            return Err(validation(RigidWorldErrorKind::DuplicateJointId));
        }
        if joint.body_a_id == joint.body_b_id
            || !body_ids.contains(&joint.body_a_id)
            || !body_ids.contains(&joint.body_b_id)
        {
            return Err(validation(RigidWorldErrorKind::InvalidOwner));
        }
        validate_joint_definition(&joint.definition)?;
        if let RigidJointDefinition::Gear {
            joint_a_id,
            joint_b_id,
            ..
        } = &joint.definition
        {
            let maybe_source_a = endpoints.get(joint_a_id);
            let maybe_source_b = endpoints.get(joint_b_id);
            if joint_a_id == joint_b_id
                || !matches!(
                    kinds.get(joint_a_id),
                    Some(RigidJointKind::Revolute | RigidJointKind::Prismatic)
                )
                || !matches!(
                    kinds.get(joint_b_id),
                    Some(RigidJointKind::Revolute | RigidJointKind::Prismatic)
                )
                || !matches!(
                    (maybe_source_a, maybe_source_b),
                    (Some([_, moving_a]), Some([_, moving_b]))
                        if moving_a != moving_b
                            && moving_a == &joint.body_a_id
                            && moving_b == &joint.body_b_id
                )
            {
                return Err(validation(RigidWorldErrorKind::InvalidJointDependency));
            }
        }
        ids.insert(joint.joint_id.clone());
        kinds.insert(joint.joint_id.clone(), joint.definition.joint_kind());
        endpoints.insert(
            joint.joint_id.clone(),
            [joint.body_a_id.clone(), joint.body_b_id.clone()],
        );
    }
    Ok(joints)
}

fn remove_joint_cascade(
    joint_id: &ScenarioId,
    live_joints: &mut HashSet<ScenarioId>,
    gear_dependents: &HashMap<ScenarioId, Vec<ScenarioId>>,
) {
    if let Some(dependents) = gear_dependents.get(joint_id) {
        for dependent in dependents.iter().rev() {
            live_joints.remove(dependent);
        }
    }
    live_joints.remove(joint_id);
}

#[allow(
    clippy::too_many_lines,
    reason = "closed joint definitions are audited exhaustively"
)]
fn validate_joint_definition(
    definition: &RigidJointDefinition,
) -> Result<(), RigidWorldDecodeError> {
    let invalid = || validation(RigidWorldErrorKind::InvalidJointDefinition);
    match definition {
        RigidJointDefinition::Revolute {
            local_anchor_a,
            local_anchor_b,
            reference_angle_bits,
            lower_angle_bits,
            upper_angle_bits,
            motor_speed_bits,
            max_motor_torque_bits,
            ..
        } => {
            validate_vec2(*local_anchor_a)?;
            validate_vec2(*local_anchor_b)?;
            for bits in [
                *reference_angle_bits,
                *lower_angle_bits,
                *upper_angle_bits,
                *motor_speed_bits,
            ] {
                validate_finite(bits, RigidWorldErrorKind::InvalidJointDefinition)?;
            }
            validate_nonnegative(*max_motor_torque_bits).map_err(|_| invalid())?;
            if lower_angle_bits.to_f32() > upper_angle_bits.to_f32() {
                return Err(invalid());
            }
        }
        RigidJointDefinition::Prismatic {
            local_anchor_a,
            local_anchor_b,
            local_axis_a,
            reference_angle_bits,
            lower_translation_bits,
            upper_translation_bits,
            motor_speed_bits,
            max_motor_force_bits,
            ..
        } => {
            for vector in [*local_anchor_a, *local_anchor_b, *local_axis_a] {
                validate_vec2(vector)?;
            }
            validate_nonzero_vector(*local_axis_a, RigidWorldErrorKind::InvalidJointDefinition)?;
            for bits in [
                *reference_angle_bits,
                *lower_translation_bits,
                *upper_translation_bits,
                *motor_speed_bits,
            ] {
                validate_finite(bits, RigidWorldErrorKind::InvalidJointDefinition)?;
            }
            validate_nonnegative(*max_motor_force_bits).map_err(|_| invalid())?;
            if lower_translation_bits.to_f32() > upper_translation_bits.to_f32() {
                return Err(invalid());
            }
        }
        RigidJointDefinition::Distance {
            local_anchor_a,
            local_anchor_b,
            length_bits,
            frequency_bits,
            damping_ratio_bits,
        } => {
            validate_vec2(*local_anchor_a)?;
            validate_vec2(*local_anchor_b)?;
            validate_positive(*length_bits).map_err(|_| invalid())?;
            validate_nonnegative(*frequency_bits).map_err(|_| invalid())?;
            validate_unit_interval(*damping_ratio_bits)?;
        }
        RigidJointDefinition::Pulley {
            ground_anchor_a,
            ground_anchor_b,
            local_anchor_a,
            local_anchor_b,
            length_a_bits,
            length_b_bits,
            ratio_bits,
        } => {
            for vector in [
                *ground_anchor_a,
                *ground_anchor_b,
                *local_anchor_a,
                *local_anchor_b,
            ] {
                validate_vec2(vector)?;
            }
            for bits in [*length_a_bits, *length_b_bits, *ratio_bits] {
                validate_positive(bits).map_err(|_| invalid())?;
            }
        }
        RigidJointDefinition::Mouse {
            target,
            max_force_bits,
            frequency_bits,
            damping_ratio_bits,
        } => {
            validate_vec2(*target)?;
            validate_nonnegative(*max_force_bits).map_err(|_| invalid())?;
            validate_nonnegative(*frequency_bits).map_err(|_| invalid())?;
            validate_unit_interval(*damping_ratio_bits)?;
        }
        RigidJointDefinition::Gear { ratio_bits, .. } => {
            validate_finite(*ratio_bits, RigidWorldErrorKind::InvalidJointDefinition)?;
        }
        RigidJointDefinition::Wheel {
            local_anchor_a,
            local_anchor_b,
            local_axis_a,
            motor_speed_bits,
            max_motor_torque_bits,
            frequency_bits,
            damping_ratio_bits,
            ..
        } => {
            for vector in [*local_anchor_a, *local_anchor_b, *local_axis_a] {
                validate_vec2(vector)?;
            }
            validate_nonzero_vector(*local_axis_a, RigidWorldErrorKind::InvalidJointDefinition)?;
            validate_finite(
                *motor_speed_bits,
                RigidWorldErrorKind::InvalidJointDefinition,
            )?;
            validate_nonnegative(*max_motor_torque_bits).map_err(|_| invalid())?;
            validate_nonnegative(*frequency_bits).map_err(|_| invalid())?;
            validate_unit_interval(*damping_ratio_bits)?;
        }
        RigidJointDefinition::Weld {
            local_anchor_a,
            local_anchor_b,
            reference_angle_bits,
            frequency_bits,
            damping_ratio_bits,
        } => {
            validate_vec2(*local_anchor_a)?;
            validate_vec2(*local_anchor_b)?;
            validate_finite(
                *reference_angle_bits,
                RigidWorldErrorKind::InvalidJointDefinition,
            )?;
            validate_nonnegative(*frequency_bits).map_err(|_| invalid())?;
            validate_unit_interval(*damping_ratio_bits)?;
        }
        RigidJointDefinition::Friction {
            local_anchor_a,
            local_anchor_b,
            max_force_bits,
            max_torque_bits,
        } => {
            validate_vec2(*local_anchor_a)?;
            validate_vec2(*local_anchor_b)?;
            validate_nonnegative(*max_force_bits).map_err(|_| invalid())?;
            validate_nonnegative(*max_torque_bits).map_err(|_| invalid())?;
        }
        RigidJointDefinition::Rope {
            local_anchor_a,
            local_anchor_b,
            max_length_bits,
        } => {
            validate_vec2(*local_anchor_a)?;
            validate_vec2(*local_anchor_b)?;
            validate_positive(*max_length_bits).map_err(|_| invalid())?;
        }
        RigidJointDefinition::Motor {
            linear_offset,
            angular_offset_bits,
            max_force_bits,
            max_torque_bits,
            correction_factor_bits,
        } => {
            validate_vec2(*linear_offset)?;
            validate_finite(
                *angular_offset_bits,
                RigidWorldErrorKind::InvalidJointDefinition,
            )?;
            validate_nonnegative(*max_force_bits).map_err(|_| invalid())?;
            validate_nonnegative(*max_torque_bits).map_err(|_| invalid())?;
            validate_unit_interval(*correction_factor_bits)?;
        }
    }
    Ok(())
}

fn validate_ropes(
    raw_ropes: Vec<RawRopeDeclaration>,
) -> Result<Vec<RigidRopeDeclaration>, RigidWorldDecodeError> {
    let ropes = raw_ropes
        .into_iter()
        .map(|raw| RigidRopeDeclaration {
            rope_id: raw.rope_id,
            vertices: raw.vertices.into_vec().into_boxed_slice(),
            masses_bits: raw.masses_bits.into_vec().into_boxed_slice(),
            gravity: raw.gravity,
            damping_bits: raw.damping_bits,
            stretch_stiffness_bits: raw.stretch_stiffness_bits,
            bend_stiffness_bits: raw.bend_stiffness_bits,
        })
        .collect::<Vec<_>>();
    let mut ids = HashSet::with_capacity(ropes.len());
    for rope in &ropes {
        if !ids.insert(rope.rope_id.clone()) {
            return Err(validation(RigidWorldErrorKind::DuplicateRopeId));
        }
        if rope.vertices.len() < 3
            || rope.vertices.len() > RIGID_WORLD_MAXIMUM_ROPE_VERTICES
            || rope.vertices.len() != rope.masses_bits.len()
        {
            return Err(validation(RigidWorldErrorKind::InvalidRopeDefinition));
        }
        for vertex in &rope.vertices {
            validate_vec2(*vertex)?;
        }
        for mass in &rope.masses_bits {
            validate_nonnegative(*mass)
                .map_err(|_| validation(RigidWorldErrorKind::InvalidRopeDefinition))?;
        }
        validate_vec2(rope.gravity)?;
        validate_nonnegative(rope.damping_bits)
            .map_err(|_| validation(RigidWorldErrorKind::InvalidRopeDefinition))?;
        validate_unit_interval(rope.stretch_stiffness_bits)?;
        validate_unit_interval(rope.bend_stiffness_bits)?;
    }
    Ok(ropes)
}

fn joint_mutation_changes_definition(
    definition: &RigidJointDefinition,
    mutation: RigidJointMutation,
) -> bool {
    if let Some(changed) = limit_or_motor_mutation_changes_definition(definition, mutation) {
        return changed;
    }

    match (definition, mutation) {
        (
            RigidJointDefinition::Distance { length_bits, .. },
            RigidJointMutation::Length {
                length_bits: mutation_bits,
            },
        ) => mutation_bits != *length_bits,
        (
            RigidJointDefinition::Distance { frequency_bits, .. }
            | RigidJointDefinition::Mouse { frequency_bits, .. }
            | RigidJointDefinition::Wheel { frequency_bits, .. }
            | RigidJointDefinition::Weld { frequency_bits, .. },
            RigidJointMutation::Frequency {
                frequency_bits: mutation_bits,
            },
        ) => mutation_bits != *frequency_bits,
        (
            RigidJointDefinition::Distance {
                damping_ratio_bits, ..
            }
            | RigidJointDefinition::Mouse {
                damping_ratio_bits, ..
            }
            | RigidJointDefinition::Wheel {
                damping_ratio_bits, ..
            }
            | RigidJointDefinition::Weld {
                damping_ratio_bits, ..
            },
            RigidJointMutation::DampingRatio {
                damping_ratio_bits: mutation_bits,
            },
        ) => mutation_bits != *damping_ratio_bits,
        (
            RigidJointDefinition::Mouse { target, .. },
            RigidJointMutation::MouseTarget {
                target: mutation_target,
            },
        ) => mutation_target != *target,
        (
            RigidJointDefinition::Mouse { max_force_bits, .. }
            | RigidJointDefinition::Friction { max_force_bits, .. }
            | RigidJointDefinition::Motor { max_force_bits, .. },
            RigidJointMutation::MaxForce { force_bits },
        ) => force_bits != *max_force_bits,
        (
            RigidJointDefinition::Friction {
                max_torque_bits, ..
            }
            | RigidJointDefinition::Motor {
                max_torque_bits, ..
            },
            RigidJointMutation::MaxTorque { torque_bits },
        ) => torque_bits != *max_torque_bits,
        (
            RigidJointDefinition::Gear { ratio_bits, .. },
            RigidJointMutation::GearRatio {
                ratio_bits: mutation_bits,
            },
        ) => mutation_bits != *ratio_bits,
        (
            RigidJointDefinition::Rope {
                max_length_bits, ..
            },
            RigidJointMutation::RopeMaxLength {
                max_length_bits: mutation_bits,
            },
        ) => mutation_bits != *max_length_bits,
        (
            RigidJointDefinition::Motor { linear_offset, .. },
            RigidJointMutation::LinearOffset { offset },
        ) => offset != *linear_offset,
        (
            RigidJointDefinition::Motor {
                angular_offset_bits,
                ..
            },
            RigidJointMutation::AngularOffset { offset_bits },
        ) => offset_bits != *angular_offset_bits,
        (
            RigidJointDefinition::Motor {
                correction_factor_bits,
                ..
            },
            RigidJointMutation::CorrectionFactor { factor_bits },
        ) => factor_bits != *correction_factor_bits,
        _ => false,
    }
}

fn limit_or_motor_mutation_changes_definition(
    definition: &RigidJointDefinition,
    mutation: RigidJointMutation,
) -> Option<bool> {
    let changed = match (definition, mutation) {
        (
            RigidJointDefinition::Revolute { limit_enabled, .. }
            | RigidJointDefinition::Prismatic { limit_enabled, .. },
            RigidJointMutation::LimitEnabled { enabled },
        ) => enabled != *limit_enabled,
        (
            RigidJointDefinition::Revolute {
                lower_angle_bits,
                upper_angle_bits,
                ..
            },
            RigidJointMutation::Limits {
                lower_bits,
                upper_bits,
            },
        ) => lower_bits != *lower_angle_bits || upper_bits != *upper_angle_bits,
        (
            RigidJointDefinition::Prismatic {
                lower_translation_bits,
                upper_translation_bits,
                ..
            },
            RigidJointMutation::Limits {
                lower_bits,
                upper_bits,
            },
        ) => lower_bits != *lower_translation_bits || upper_bits != *upper_translation_bits,
        (
            RigidJointDefinition::Revolute { motor_enabled, .. }
            | RigidJointDefinition::Prismatic { motor_enabled, .. }
            | RigidJointDefinition::Wheel { motor_enabled, .. },
            RigidJointMutation::MotorEnabled { enabled },
        ) => enabled != *motor_enabled,
        (
            RigidJointDefinition::Revolute {
                motor_speed_bits, ..
            }
            | RigidJointDefinition::Prismatic {
                motor_speed_bits, ..
            }
            | RigidJointDefinition::Wheel {
                motor_speed_bits, ..
            },
            RigidJointMutation::MotorSpeed { speed_bits },
        ) => speed_bits != *motor_speed_bits,
        (
            RigidJointDefinition::Prismatic {
                max_motor_force_bits,
                ..
            },
            RigidJointMutation::MaxMotorForce { force_bits },
        ) => force_bits != *max_motor_force_bits,
        (
            RigidJointDefinition::Revolute {
                max_motor_torque_bits,
                ..
            }
            | RigidJointDefinition::Wheel {
                max_motor_torque_bits,
                ..
            },
            RigidJointMutation::MaxMotorTorque { torque_bits },
        ) => torque_bits != *max_motor_torque_bits,
        _ => return None,
    };
    Some(changed)
}

fn validate_joint_mutation(
    joint_kind: RigidJointKind,
    mutation: RigidJointMutation,
) -> Result<(), RigidWorldDecodeError> {
    if !joint_mutation_is_supported(joint_kind, mutation) {
        return Err(validation(RigidWorldErrorKind::InvalidJointDefinition));
    }

    match mutation {
        RigidJointMutation::LimitEnabled { .. } | RigidJointMutation::MotorEnabled { .. } => {}
        RigidJointMutation::Limits {
            lower_bits,
            upper_bits,
        } => {
            validate_finite(lower_bits, RigidWorldErrorKind::InvalidJointDefinition)?;
            validate_finite(upper_bits, RigidWorldErrorKind::InvalidJointDefinition)?;
            if lower_bits.to_f32() > upper_bits.to_f32() {
                return Err(validation(RigidWorldErrorKind::InvalidJointDefinition));
            }
        }
        RigidJointMutation::MotorSpeed { speed_bits }
        | RigidJointMutation::AngularOffset {
            offset_bits: speed_bits,
        }
        | RigidJointMutation::GearRatio {
            ratio_bits: speed_bits,
        } => validate_finite(speed_bits, RigidWorldErrorKind::InvalidJointDefinition)?,
        RigidJointMutation::MaxMotorForce { force_bits }
        | RigidJointMutation::MaxForce { force_bits }
        | RigidJointMutation::MaxMotorTorque {
            torque_bits: force_bits,
        }
        | RigidJointMutation::MaxTorque {
            torque_bits: force_bits,
        }
        | RigidJointMutation::Frequency {
            frequency_bits: force_bits,
        } => validate_nonnegative(force_bits)
            .map_err(|_| validation(RigidWorldErrorKind::InvalidJointDefinition))?,
        RigidJointMutation::Length { length_bits }
        | RigidJointMutation::RopeMaxLength {
            max_length_bits: length_bits,
        } => validate_positive(length_bits)
            .map_err(|_| validation(RigidWorldErrorKind::InvalidJointDefinition))?,
        RigidJointMutation::DampingRatio { damping_ratio_bits } => {
            validate_unit_interval(damping_ratio_bits)?;
        }
        RigidJointMutation::MouseTarget { target }
        | RigidJointMutation::LinearOffset { offset: target } => validate_vec2(target)?,
        RigidJointMutation::CorrectionFactor { factor_bits } => {
            validate_unit_interval(factor_bits)?;
        }
    }
    Ok(())
}

fn joint_mutation_is_supported(joint_kind: RigidJointKind, mutation: RigidJointMutation) -> bool {
    match joint_kind {
        RigidJointKind::Revolute => matches!(
            mutation,
            RigidJointMutation::LimitEnabled { .. }
                | RigidJointMutation::Limits { .. }
                | RigidJointMutation::MotorEnabled { .. }
                | RigidJointMutation::MotorSpeed { .. }
                | RigidJointMutation::MaxMotorTorque { .. }
        ),
        RigidJointKind::Prismatic => matches!(
            mutation,
            RigidJointMutation::LimitEnabled { .. }
                | RigidJointMutation::Limits { .. }
                | RigidJointMutation::MotorEnabled { .. }
                | RigidJointMutation::MotorSpeed { .. }
                | RigidJointMutation::MaxMotorForce { .. }
        ),
        RigidJointKind::Distance => matches!(
            mutation,
            RigidJointMutation::Length { .. }
                | RigidJointMutation::Frequency { .. }
                | RigidJointMutation::DampingRatio { .. }
        ),
        RigidJointKind::Pulley => false,
        RigidJointKind::Mouse => matches!(
            mutation,
            RigidJointMutation::MouseTarget { .. }
                | RigidJointMutation::MaxForce { .. }
                | RigidJointMutation::Frequency { .. }
                | RigidJointMutation::DampingRatio { .. }
        ),
        RigidJointKind::Gear => matches!(mutation, RigidJointMutation::GearRatio { .. }),
        RigidJointKind::Wheel => matches!(
            mutation,
            RigidJointMutation::MotorEnabled { .. }
                | RigidJointMutation::MotorSpeed { .. }
                | RigidJointMutation::MaxMotorTorque { .. }
                | RigidJointMutation::Frequency { .. }
                | RigidJointMutation::DampingRatio { .. }
        ),
        RigidJointKind::Weld => matches!(
            mutation,
            RigidJointMutation::Frequency { .. } | RigidJointMutation::DampingRatio { .. }
        ),
        RigidJointKind::Friction => matches!(
            mutation,
            RigidJointMutation::MaxForce { .. } | RigidJointMutation::MaxTorque { .. }
        ),
        RigidJointKind::Rope => {
            matches!(mutation, RigidJointMutation::RopeMaxLength { .. })
        }
        RigidJointKind::Motor => matches!(
            mutation,
            RigidJointMutation::LinearOffset { .. }
                | RigidJointMutation::AngularOffset { .. }
                | RigidJointMutation::MaxForce { .. }
                | RigidJointMutation::MaxTorque { .. }
                | RigidJointMutation::CorrectionFactor { .. }
        ),
    }
}

fn validate_unit_interval(bits: FloatBits) -> Result<(), RigidWorldDecodeError> {
    let value = bits.to_f32();
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(validation(RigidWorldErrorKind::InvalidJointDefinition));
    }
    Ok(())
}

fn validate_nonzero_vector(
    vector: crate::Vec2Bits,
    kind: RigidWorldErrorKind,
) -> Result<(), RigidWorldDecodeError> {
    let x = vector.x_bits.to_f32();
    let y = vector.y_bits.to_f32();
    if x == 0.0 && y == 0.0 {
        return Err(validation(kind));
    }
    Ok(())
}

fn validate_contact_directive_target(
    target: &RigidContactDirectiveTarget,
    live_fixtures: &HashSet<ScenarioId>,
) -> Result<(), RigidWorldDecodeError> {
    if target.fixture_a_id == target.fixture_b_id
        || !live_fixtures.contains(&target.fixture_a_id)
        || !live_fixtures.contains(&target.fixture_b_id)
    {
        return Err(validation(RigidWorldErrorKind::InvalidContactDirective));
    }
    Ok(())
}

fn validate_pre_solve_directive(
    directive: RigidPreSolveDirective,
) -> Result<(), RigidWorldDecodeError> {
    for bits in [
        directive.maybe_friction_bits,
        directive.maybe_restitution_bits,
    ]
    .into_iter()
    .flatten()
    {
        validate_nonnegative(bits)
            .map_err(|_| validation(RigidWorldErrorKind::InvalidContactDirective))?;
    }
    if let Some(bits) = directive.maybe_tangent_speed_bits {
        validate_finite(bits, RigidWorldErrorKind::InvalidContactDirective)?;
    }
    Ok(())
}

fn validate_finite(
    value: FloatBits,
    kind: RigidWorldErrorKind,
) -> Result<(), RigidWorldDecodeError> {
    if !value.to_f32().is_finite() {
        return Err(validation(kind));
    }
    Ok(())
}

fn validate_aabb(aabb: RigidAabbBits) -> Result<(), RigidWorldDecodeError> {
    validate_vec2(aabb.lower)?;
    validate_vec2(aabb.upper)?;
    if aabb.lower.x_bits.to_f32() > aabb.upper.x_bits.to_f32()
        || aabb.lower.y_bits.to_f32() > aabb.upper.y_bits.to_f32()
    {
        return Err(validation(RigidWorldErrorKind::InvalidQueryDirective));
    }
    Ok(())
}

fn validate_query_rules(
    rules: &[RigidQueryDirectiveRule],
    live_fixtures: &HashSet<ScenarioId>,
    fixture_shapes: &HashMap<ScenarioId, RigidFixtureShape>,
) -> Result<(), RigidWorldDecodeError> {
    if rules.len() > RIGID_WORLD_MAXIMUM_DIRECTIVES {
        return Err(validation(RigidWorldErrorKind::AggregateLimitExceeded));
    }
    validate_unique_selectors(
        rules.iter().map(|rule| &rule.target),
        live_fixtures,
        fixture_shapes,
        RigidWorldErrorKind::InvalidQueryDirective,
    )
}

fn validate_ray_geometry(
    start: crate::Vec2Bits,
    end: crate::Vec2Bits,
) -> Result<(), RigidWorldDecodeError> {
    let direction_x = end.x_bits.to_f32() - start.x_bits.to_f32();
    let direction_y = end.y_bits.to_f32() - start.y_bits.to_f32();
    let squared_x = direction_x * direction_x;
    let squared_y = direction_y * direction_y;
    let length_squared = squared_x + squared_y;
    if !direction_x.is_finite()
        || !direction_y.is_finite()
        || !squared_x.is_finite()
        || !squared_y.is_finite()
        || !length_squared.is_finite()
        || length_squared == 0.0
    {
        return Err(validation(RigidWorldErrorKind::InvalidRayDirective));
    }
    Ok(())
}

fn validate_ray_rules(
    rules: &[RigidRayDirectiveRule],
    live_fixtures: &HashSet<ScenarioId>,
    fixture_shapes: &HashMap<ScenarioId, RigidFixtureShape>,
) -> Result<(), RigidWorldDecodeError> {
    if rules.len() > RIGID_WORLD_MAXIMUM_DIRECTIVES {
        return Err(validation(RigidWorldErrorKind::AggregateLimitExceeded));
    }
    validate_unique_selectors(
        rules.iter().map(|rule| &rule.target),
        live_fixtures,
        fixture_shapes,
        RigidWorldErrorKind::InvalidRayDirective,
    )?;
    for rule in rules {
        if let RigidRayDirective::Clip { fraction_bits } = rule.directive {
            let fraction = fraction_bits.to_f32();
            if !fraction.is_finite() || fraction <= 0.0 || fraction > 1.0 {
                return Err(validation(RigidWorldErrorKind::InvalidRayDirective));
            }
        }
    }
    Ok(())
}

fn validate_unique_selectors<'a>(
    selectors: impl Iterator<Item = &'a RigidFixtureChildSelector>,
    live_fixtures: &HashSet<ScenarioId>,
    fixture_shapes: &HashMap<ScenarioId, RigidFixtureShape>,
    kind: RigidWorldErrorKind,
) -> Result<(), RigidWorldDecodeError> {
    let mut unique = HashSet::new();
    for selector in selectors {
        let maybe_shape = fixture_shapes.get(&selector.fixture_id);
        if !live_fixtures.contains(&selector.fixture_id)
            || maybe_shape.is_none_or(|shape| selector.child_index >= shape_child_count(shape))
            || !unique.insert(selector.clone())
        {
            return Err(validation(kind));
        }
    }
    Ok(())
}

const fn shape_child_count(shape: &RigidFixtureShape) -> u32 {
    match shape {
        RigidFixtureShape::Circle { .. } | RigidFixtureShape::Polygon { .. } => 1,
    }
}

fn validate_custom_mass(
    mass_bits: FloatBits,
    center: crate::Vec2Bits,
    inertia_bits: FloatBits,
) -> Result<(), RigidWorldDecodeError> {
    validate_positive(mass_bits)?;
    validate_vec2(center)?;
    validate_nonnegative(inertia_bits)?;

    let mass = mass_bits.to_f32();
    let origin_inertia = inertia_bits.to_f32();
    if origin_inertia == 0.0 {
        return Ok(());
    }
    let center_x = center.x_bits.to_f32();
    let center_y = center.y_bits.to_f32();
    let squared_center = [center_x * center_x, center_y * center_y];
    let center_dot = squared_center[0] + squared_center[1];
    let parallel_axis = mass * center_dot;
    let centered_inertia = origin_inertia - parallel_axis;
    if !squared_center[0].is_finite()
        || !squared_center[1].is_finite()
        || !center_dot.is_finite()
        || !parallel_axis.is_finite()
        || !centered_inertia.is_finite()
        || centered_inertia <= 0.0
    {
        return Err(validation(RigidWorldErrorKind::InvalidGeometry));
    }
    Ok(())
}

fn validate_checkpoints(
    raw_checkpoints: Vec<RawCheckpoint>,
    family: RigidWorldWitnessFamily,
    actions: &[RigidWorldActionRecord],
    body_ids: &HashSet<ScenarioId>,
    fixture_owners: &HashMap<ScenarioId, ScenarioId>,
) -> Result<Vec<RigidExpectedCheckpoint>, RigidWorldDecodeError> {
    if raw_checkpoints.is_empty() {
        return Err(validation(RigidWorldErrorKind::InvalidCheckpointOrder));
    }
    let action_positions = actions
        .iter()
        .enumerate()
        .map(|(index, action)| (action.action_id.clone(), index))
        .collect::<HashMap<_, _>>();
    let live_counts = action_live_counts(actions, fixture_owners);
    let mut checkpoint_ids = HashSet::with_capacity(raw_checkpoints.len());
    let mut witnesses = HashSet::new();
    let mut previous_action_index = None;
    let mut checkpoints = Vec::with_capacity(raw_checkpoints.len());

    for raw in raw_checkpoints {
        if !checkpoint_ids.insert(raw.checkpoint_id.clone()) {
            return Err(validation(RigidWorldErrorKind::DuplicateCheckpointId));
        }
        let Some(&action_index) = action_positions.get(&raw.after_action_id) else {
            return Err(validation(RigidWorldErrorKind::InvalidCheckpointOrder));
        };
        if previous_action_index.is_some_and(|previous| action_index <= previous) {
            return Err(validation(RigidWorldErrorKind::InvalidCheckpointOrder));
        }
        previous_action_index = Some(action_index);
        let phase = raw.phase.into_string();
        if phase.trim().is_empty() || phase.as_str() != actions[action_index].phase.as_ref() {
            return Err(validation(RigidWorldErrorKind::CheckpointPhaseMismatch));
        }
        let (body_count, fixture_count) = live_counts[action_index];
        if raw.counts.bodies != checked_u32(body_count)?
            || raw.counts.fixtures != checked_u32(fixture_count)?
            || raw.counts.manifold_points > raw.counts.contacts.saturating_mul(2)
            || (family == RigidWorldWitnessFamily::NonCollidingBodyFixtureLifecycle
                && (raw.counts.contacts != 0 || raw.counts.manifold_points != 0))
            || (family == RigidWorldWitnessFamily::SingleContactLifecycle
                && raw.counts.contacts > 1)
        {
            return Err(validation(RigidWorldErrorKind::ExpectedCountMismatch));
        }
        let transitions = raw
            .transitions
            .into_vec()
            .into_iter()
            .map(|transition| {
                if !witnesses.insert(transition.witness) {
                    return Err(validation(RigidWorldErrorKind::DuplicateWitness));
                }
                if transition.witness.requires_contact_identity()
                    != transition.maybe_contact.is_some()
                {
                    return Err(validation(RigidWorldErrorKind::InvalidContactIdentity));
                }
                if let Some(contact) = &transition.maybe_contact {
                    validate_contact_identity(contact, body_ids, fixture_owners)?;
                }
                Ok(RigidExpectedTransition {
                    witness: transition.witness,
                    maybe_contact: transition.maybe_contact,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        checkpoints.push(RigidExpectedCheckpoint {
            checkpoint_id: raw.checkpoint_id,
            after_action_id: raw.after_action_id,
            phase: phase.into_boxed_str(),
            counts: raw.counts,
            transitions: transitions.into_boxed_slice(),
        });
    }

    let required = family.required_witnesses();
    if required.iter().any(|witness| !witnesses.contains(witness)) {
        return Err(validation(RigidWorldErrorKind::MissingWitness));
    }
    if witnesses.iter().any(|witness| !required.contains(witness)) {
        return Err(validation(RigidWorldErrorKind::UnexpectedWitness));
    }
    Ok(checkpoints)
}

fn action_live_counts(
    actions: &[RigidWorldActionRecord],
    fixture_owners: &HashMap<ScenarioId, ScenarioId>,
) -> Vec<(usize, usize)> {
    let mut bodies = HashSet::new();
    let mut fixtures = HashSet::new();
    let mut counts = Vec::with_capacity(actions.len());
    for action in actions {
        super::types::apply_lifecycle_action(
            action.action(),
            fixture_owners,
            &mut bodies,
            &mut fixtures,
        );
        counts.push((bodies.len(), fixtures.len()));
    }
    counts
}

fn validate_contact_identity(
    contact: &RigidContactIdentity,
    _body_ids: &HashSet<ScenarioId>,
    fixture_owners: &HashMap<ScenarioId, ScenarioId>,
) -> Result<(), RigidWorldDecodeError> {
    if contact.fixture_a_id() == contact.fixture_b_id()
        || !fixture_owners.contains_key(contact.fixture_a_id())
        || !fixture_owners.contains_key(contact.fixture_b_id())
        || contact.child_a() != 0
        || contact.child_b() != 0
        || contact.occurrence() == 0
    {
        return Err(validation(RigidWorldErrorKind::InvalidContactIdentity));
    }
    Ok(())
}

fn validate_source(raw: RawSource) -> Result<ScenarioSource, RigidWorldDecodeError> {
    match raw {
        RawSource::Named { name } => {
            let name = name.into_string();
            if name.trim().is_empty() {
                return Err(validation(RigidWorldErrorKind::InvalidSource));
            }
            Ok(ScenarioSource::Named {
                name: name.into_boxed_str(),
            })
        }
        RawSource::Seeded {
            generator_id,
            generator_version,
            seed,
        } => {
            let generator_id = generator_id.into_string();
            if generator_id.trim().is_empty() || generator_version == 0 {
                return Err(validation(RigidWorldErrorKind::InvalidSource));
            }
            Ok(ScenarioSource::Seeded {
                generator_id: generator_id.into_boxed_str(),
                generator_version,
                seed,
            })
        }
    }
}

fn require_live(
    id: &ScenarioId,
    live: &HashSet<ScenarioId>,
    kind: RigidWorldErrorKind,
) -> Result<(), RigidWorldDecodeError> {
    if !live.contains(id) {
        return Err(validation(kind));
    }
    Ok(())
}

fn checked_u32(value: usize) -> Result<u32, RigidWorldDecodeError> {
    u32::try_from(value).map_err(|_| validation(RigidWorldErrorKind::AggregateLimitExceeded))
}

#[cfg(test)]
mod phase9_tests {
    use super::*;

    fn id(value: &str) -> ScenarioId {
        ScenarioId::new(value).expect("test scenario identity should be valid")
    }

    fn system_declaration() -> Phase9ParticleSystemDeclaration {
        Phase9ParticleSystemDeclaration {
            system_id: id("system"),
            buffer_mode: super::super::Phase9ParticleBufferMode::Growable {
                initial_capacity: 1,
            },
            paused: false,
            strict_contact_check: false,
            stuck_threshold: 0,
            density_bits: FloatBits::from_f32(1.0),
            gravity_scale_bits: FloatBits::from_f32(1.0),
            radius_bits: FloatBits::from_f32(0.1),
            damping_bits: FloatBits::from_f32(0.0),
            destruction_by_age: false,
            lifetime_granularity_bits: FloatBits::from_f32(1.0 / 60.0),
            maximum_count: None,
        }
    }

    #[test]
    fn phase9_declaration_accepts_negative_finite_lifetime_bits() {
        // Arrange
        let systems = [system_declaration()];
        let particles = [Phase9ParticleDeclaration {
            particle_id: id("particle"),
            system_id: id("system"),
            position: crate::Vec2Bits {
                x_bits: FloatBits::from_f32(0.0),
                y_bits: FloatBits::from_f32(0.0),
            },
            velocity: crate::Vec2Bits {
                x_bits: FloatBits::from_f32(0.0),
                y_bits: FloatBits::from_f32(0.0),
            },
            flags_bits: 0,
            color: [0; 4],
            lifetime_bits: FloatBits::from_f32(-1.0),
        }];

        // Act
        let result = validate_phase9_declarations(&systems, &particles);

        // Assert
        assert!(result.is_ok());
    }
}
