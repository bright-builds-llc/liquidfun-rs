use std::collections::{HashMap, HashSet};

use serde::Deserialize;

mod geometry;
mod phase8;

mod action;
mod checks;
mod joints;
mod timeline;

use action::{validate_action, validate_phase9_declarations};
use checks::{
    require_live, validate_aabb, validate_checkpoints, validate_contact_directive_target,
    validate_custom_mass, validate_finite, validate_nonzero_vector, validate_pre_solve_directive,
    validate_query_rules, validate_ray_geometry, validate_ray_rules, validate_source,
    validate_unit_interval,
};
use geometry::{
    validate_nonnegative, validate_positive, validate_shape, validate_transform, validate_vec2,
};
use joints::{
    joint_mutation_changes_definition, remove_joint_cascade, validate_joint_mutation,
    validate_joints, validate_ropes,
};
use phase8::validate_phase8_behavior;
use timeline::validate_timeline;

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

#[cfg(test)]
mod phase9_tests;
