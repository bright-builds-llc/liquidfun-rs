use std::collections::{HashMap, HashSet};

use serde::Deserialize;

mod geometry;

use geometry::{
    validate_nonnegative, validate_positive, validate_shape, validate_transform, validate_vec2,
};

use super::{
    RigidBodyDeclaration, RigidBodyKind, RigidContactIdentity, RigidExpectedCheckpoint,
    RigidExpectedCounts, RigidExpectedTransition, RigidFilterBits, RigidFixtureDeclaration,
    RigidFixtureShape, RigidWorldAction, RigidWorldActionRecord, RigidWorldDecodeError,
    RigidWorldErrorKind, RigidWorldRequestKind, RigidWorldRequestRecord, RigidWorldScenario,
    RigidWorldTimeline, RigidWorldWitness, RigidWorldWitnessFamily, validation,
};
use crate::{
    FloatBits, HarnessLimits, ProtocolVersion, RecordLimit, RequestId, ScenarioId,
    ScenarioSchemaVersion, ScenarioSource, Sha256Hex, ToleranceProfileVersion, TraceSchemaVersion,
    TransformBits,
    codec::{BoundedString, BoundedVec, decode_jsonl},
};

const MAXIMUM_ID_BYTES: usize = 128;
const MAXIMUM_STRING_BYTES: usize = 4 * 1024;
const MAXIMUM_TIMELINES: usize = 2;
const MAXIMUM_BODIES: usize = 64;
const MAXIMUM_FIXTURES: usize = 128;
const MAXIMUM_ACTIONS: usize = 128;
const MAXIMUM_CHECKPOINTS: usize = 64;
const MAXIMUM_TRANSITIONS: usize = 64;
const MAXIMUM_POLYGON_VERTICES: usize = 8;
const MAXIMUM_AGGREGATE_ITEMS: usize = 2_048;
const MAXIMUM_SOLVER_ITERATIONS: u32 = 255;

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
    actions: BoundedVec<RawActionRecord, MAXIMUM_ACTIONS>,
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
    if RigidWorldWitnessFamily::REQUIRED
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
    let actions = validate_actions(
        raw.actions.into_vec(),
        raw.witness_family,
        &body_ids,
        &fixture_owners,
    )?;
    let checkpoints = validate_checkpoints(
        raw.checkpoints.into_vec(),
        raw.witness_family,
        &actions,
        &body_ids,
        &fixture_owners,
    )?;
    let aggregate = bodies.len() + fixtures.len() + actions.len() + checkpoints.len();
    if aggregate > MAXIMUM_AGGREGATE_ITEMS {
        return Err(validation(RigidWorldErrorKind::AggregateLimitExceeded));
    }
    Ok(RigidWorldTimeline {
        witness_family: raw.witness_family,
        bodies: bodies.into_boxed_slice(),
        fixtures: fixtures.into_boxed_slice(),
        actions: actions.into_boxed_slice(),
        checkpoints: checkpoints.into_boxed_slice(),
    })
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
    body_ids: &HashSet<ScenarioId>,
    fixture_owners: &HashMap<ScenarioId, ScenarioId>,
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
            body_ids,
            fixture_owners,
            &mut live_bodies,
            &mut live_fixtures,
            &mut created_bodies,
            &mut created_fixtures,
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
        || created_bodies.len() != body_ids.len()
        || created_fixtures.len() != fixture_owners.len()
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
    live_bodies: &mut HashSet<ScenarioId>,
    live_fixtures: &mut HashSet<ScenarioId>,
    created_bodies: &mut HashSet<ScenarioId>,
    created_fixtures: &mut HashSet<ScenarioId>,
) -> Result<(), RigidWorldDecodeError> {
    match action {
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
        | RigidWorldAction::SetBodyActive { body_id, .. } => {
            require_live(body_id, live_bodies, RigidWorldErrorKind::UnknownBody)?;
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
            validate_positive(*mass_bits)?;
            validate_vec2(*center)?;
            validate_nonnegative(*inertia_bits)?;
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
            validate_positive(*timestep_bits)?;
            if *velocity_iterations == 0
                || *position_iterations == 0
                || *velocity_iterations > MAXIMUM_SOLVER_ITERATIONS
                || *position_iterations > MAXIMUM_SOLVER_ITERATIONS
            {
                return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
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
        }
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
        match &action.action {
            RigidWorldAction::CreateBody { body_id } => {
                bodies.insert(body_id.clone());
            }
            RigidWorldAction::CreateFixture { fixture_id } => {
                fixtures.insert(fixture_id.clone());
            }
            RigidWorldAction::DestroyFixture { fixture_id } => {
                fixtures.remove(fixture_id);
            }
            RigidWorldAction::DestroyBody { body_id } => {
                bodies.remove(body_id);
                fixtures.retain(|fixture_id| fixture_owners.get(fixture_id) != Some(body_id));
            }
            _ => {}
        }
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
