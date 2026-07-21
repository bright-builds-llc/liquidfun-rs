use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[cfg(test)]
#[path = "result/ownership_tests.rs"]
mod ownership_tests;
mod phase10;
mod phase8;
mod phase9;

use phase8::{ExpectedObservation, expected_observation, validate_phase8_observation_contract};
use phase9::Phase9ResultState;
use phase10::Phase10ResultState;
pub use phase10::*;

use super::{
    PHASE9_MAXIMUM_IDENTITIES, Phase9ParticleObservation, Phase10Operation, RigidBodyKind,
    RigidContactIdentity, RigidExpectedCounts, RigidFilterBits, RigidWorldAction,
    RigidWorldDecodeError, RigidWorldErrorKind, RigidWorldRequestRecord, RigidWorldWitnessFamily,
    validation,
};
use crate::{
    FloatBits, HarnessLimits, ProtocolVersion, RecordLimit, RequestId, ScenarioId,
    TraceSchemaVersion, TransformBits, Vec2Bits, decode_jsonl,
};

const MAXIMUM_RESULT_TIMELINES: usize = RigidWorldWitnessFamily::ALL.len();
const MAXIMUM_RESULT_CHECKPOINTS: usize = 64;
const MAXIMUM_RESULT_BODIES: usize = 64;
const MAXIMUM_RESULT_FIXTURES: usize = 128;
const MAXIMUM_RESULT_CONTACTS: usize = 128;
const MAXIMUM_RESULT_EVENTS: usize = 256;
const MAXIMUM_RESULT_DESTRUCTIONS: usize = 256;
const MAXIMUM_MANIFOLD_POINTS: usize = 2;
const MAXIMUM_RESULT_AGGREGATE: usize = 4_096;
const MAXIMUM_RESULT_OBSERVATIONS: usize = 256;
const MAXIMUM_QUERY_OCCURRENCES: usize = 256;
const MAXIMUM_RAY_HITS: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidStepCompletion {
    Complete,
    ContinuousPending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidPartialProgressClassification {
    ContinuousWorkBudgetExhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RigidStepOutcome {
    Completed {
        completion: RigidStepCompletion,
    },
    Partial {
        classification: RigidPartialProgressClassification,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "the protocol transports independent upstream body flags exactly"
)]
pub struct RigidBodyControlSnapshot {
    pub body_id: ScenarioId,
    pub linear_velocity: Vec2Bits,
    pub angular_velocity_bits: FloatBits,
    pub awake: bool,
    pub bullet: bool,
    pub sleeping_allowed: bool,
    pub fixed_rotation: bool,
    pub linear_damping_bits: FloatBits,
    pub angular_damping_bits: FloatBits,
    pub gravity_scale_bits: FloatBits,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidFixtureChildOccurrence {
    pub fixture_id: ScenarioId,
    pub child_index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidQueryCompletion {
    Exhausted,
    Terminated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidQueryObservation {
    pub completion: RigidQueryCompletion,
    pub occurrences: Box<[RigidFixtureChildOccurrence]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidRayHitObservation {
    pub fixture_id: ScenarioId,
    pub child_index: u32,
    pub point: Vec2Bits,
    pub normal: Vec2Bits,
    pub fraction_bits: FloatBits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidRayCompletion {
    Exhausted,
    Terminated,
}

/// Exact initial maximum fraction for every closed rigid-world ray cast.
pub const RIGID_RAY_INITIAL_MAX_FRACTION_BITS: FloatBits = FloatBits::new(1.0_f32.to_bits());

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidRayObservation {
    pub completion: RigidRayCompletion,
    pub final_max_fraction_bits: FloatBits,
    pub hits: Box<[RigidRayHitObservation]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RigidWorldObservation {
    BodyState {
        state: RigidBodyControlSnapshot,
    },
    Step {
        outcome: RigidStepOutcome,
    },
    Query {
        observation: RigidQueryObservation,
    },
    RayCast {
        observation: RigidRayObservation,
    },
    OriginShift {
        shift: Vec2Bits,
    },
    Joint {
        snapshot: RigidJointSnapshot,
    },
    Rope {
        snapshot: RigidRopeSnapshot,
    },
    Lifecycle {
        event: RigidLifecycleObservation,
    },
    Reconstruction {
        record: RigidReconstructionObservation,
    },
    Diagnostics {
        snapshot: RigidDiagnosticsObservation,
    },
    Particle {
        observation: Phase9ParticleObservation,
    },
    ParticleGroup {
        observation: Phase10Observation,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidJointBranchState {
    Inactive,
    AtLower,
    AtUpper,
    Equal,
    Active,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidJointSnapshot {
    pub joint_id: ScenarioId,
    pub joint_kind: super::RigidJointKind,
    pub body_a_id: ScenarioId,
    pub body_b_id: ScenarioId,
    pub collide_connected: bool,
    pub dependencies: Box<[ScenarioId]>,
    pub branch_state: RigidJointBranchState,
    pub coordinate_bits: FloatBits,
    pub speed_bits: FloatBits,
    pub reaction_force: Vec2Bits,
    pub reaction_torque_bits: FloatBits,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidRopeSnapshot {
    pub rope_id: ScenarioId,
    pub vertices: Box<[Vec2Bits]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidLifecycleObservationKind {
    FilterDecision,
    ContactCreated,
    BeginContact,
    PreSolve,
    PostSolve,
    EndContact,
    ContactDestroyed,
    JointGoodbye,
    FixtureGoodbye,
    BodyDestroyed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidLifecycleObservation {
    pub ordinal: u32,
    pub kind: RigidLifecycleObservationKind,
    pub maybe_contact: Option<RigidContactIdentity>,
    pub maybe_entity_id: Option<ScenarioId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidReconstructionKind {
    Body,
    Fixture,
    Joint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidReconstructionSupport {
    Supported,
    UnsupportedMouseJoint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidReconstructionObservation {
    pub ordinal: u32,
    pub kind: RigidReconstructionKind,
    pub entity_id: ScenarioId,
    pub support: RigidReconstructionSupport,
    pub dependency_ids: Box<[ScenarioId]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidDiagnosticsObservation {
    pub body_count: u32,
    pub fixture_count: u32,
    pub joint_count: u32,
    pub contact_count: u32,
    pub tree_height: u32,
    pub tree_max_balance: u32,
    pub tree_quality_bits: FloatBits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidManifoldKind {
    Circles,
    FaceA,
    FaceB,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidFeatureKind {
    Vertex,
    Face,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidContactFeature {
    pub index_a: u8,
    pub index_b: u8,
    pub kind_a: RigidFeatureKind,
    pub kind_b: RigidFeatureKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidManifoldPoint {
    pub point: Vec2Bits,
    pub feature: RigidContactFeature,
    pub normal_impulse_bits: FloatBits,
    pub tangent_impulse_bits: FloatBits,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidManifoldResult {
    pub manifold_kind: RigidManifoldKind,
    pub local_normal: Vec2Bits,
    pub local_point: Vec2Bits,
    pub points: Box<[RigidManifoldPoint]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidBodySnapshot {
    pub body_id: ScenarioId,
    pub body_kind: RigidBodyKind,
    pub transform: TransformBits,
    pub active: bool,
    pub linear_velocity: Vec2Bits,
    pub angular_velocity_bits: FloatBits,
    pub mass_bits: FloatBits,
    pub local_center: Vec2Bits,
    pub inertia_bits: FloatBits,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidFixtureSnapshot {
    pub fixture_id: ScenarioId,
    pub owner_body_id: ScenarioId,
    pub sensor: bool,
    pub density_bits: FloatBits,
    pub friction_bits: FloatBits,
    pub restitution_bits: FloatBits,
    pub filter: RigidFilterBits,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidContactResult {
    pub identity: RigidContactIdentity,
    pub touching: bool,
    pub enabled: bool,
    pub sensor: bool,
    pub mixed_friction_bits: FloatBits,
    pub mixed_restitution_bits: FloatBits,
    pub maybe_manifold: Option<RigidManifoldResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidContactEventKind {
    Created,
    Begin,
    Persist,
    End,
    PreSolve,
    PostSolve,
    Destroyed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidContactEvent {
    pub kind: RigidContactEventKind,
    pub contact: RigidContactIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RigidDestructionRecord {
    Contact { contact: RigidContactIdentity },
    Fixture { fixture_id: ScenarioId },
    Body { body_id: ScenarioId },
    Joint { joint_id: ScenarioId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidWorldCheckpointResult {
    pub checkpoint_id: ScenarioId,
    pub phase: Box<str>,
    pub counts: RigidExpectedCounts,
    pub bodies: Box<[RigidBodySnapshot]>,
    pub fixtures: Box<[RigidFixtureSnapshot]>,
    pub contacts: Box<[RigidContactResult]>,
    pub events: Box<[RigidContactEvent]>,
    pub destructions: Box<[RigidDestructionRecord]>,
    #[serde(default, skip_serializing_if = "observations_are_empty")]
    pub observations: Box<[RigidWorldObservation]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidWorldTimelineResult {
    pub witness_family: RigidWorldWitnessFamily,
    pub checkpoints: Box<[RigidWorldCheckpointResult]>,
}

/// Declaration-ordered live identities expected at one rigid-world checkpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RigidCheckpointLiveIdentities<'a> {
    body_ids: Vec<&'a ScenarioId>,
    fixture_ids: Vec<&'a ScenarioId>,
    fixtures: Vec<&'a super::RigidFixtureDeclaration>,
}

impl<'a> RigidCheckpointLiveIdentities<'a> {
    /// Returns live body identities in declaration order.
    #[must_use]
    pub fn body_ids(&self) -> &[&'a ScenarioId] {
        &self.body_ids
    }

    /// Returns live fixture identities in declaration order.
    #[must_use]
    pub fn fixture_ids(&self) -> &[&'a ScenarioId] {
        &self.fixture_ids
    }
}

fn fixture_belongs_to_live_body<'a>(
    live_body_ids: &[&ScenarioId],
    mut live_fixture_owners: impl Iterator<Item = (&'a ScenarioId, &'a ScenarioId)>,
    fixture_id: &ScenarioId,
    body_id: &ScenarioId,
) -> bool {
    live_body_ids.contains(&body_id)
        && live_fixture_owners.any(|(candidate_fixture_id, owner_body_id)| {
            candidate_fixture_id == fixture_id && owner_body_id == body_id
        })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidWorldResultRecord {
    protocol_version: ProtocolVersion,
    record_kind: RigidWorldResultKind,
    request_id: RequestId,
    trace_schema_version: TraceSchemaVersion,
    scenario_id: ScenarioId,
    timelines: Box<[RigidWorldTimelineResult]>,
}

impl RigidWorldResultRecord {
    /// Creates one bounded semantic rigid-world result record.
    ///
    /// # Errors
    ///
    /// Returns [`RigidWorldDecodeError`] when any result collection or aggregate
    /// exceeds the reviewed Phase 6 limits.
    pub fn new(
        request_id: RequestId,
        scenario_id: ScenarioId,
        timelines: Vec<RigidWorldTimelineResult>,
    ) -> Result<Self, RigidWorldDecodeError> {
        let record = Self {
            protocol_version: ProtocolVersion::new(ProtocolVersion::SUPPORTED)
                .map_err(|_| validation(RigidWorldErrorKind::InvalidIdentifier))?,
            record_kind: RigidWorldResultKind::RigidWorldResult,
            request_id,
            trace_schema_version: TraceSchemaVersion::new(TraceSchemaVersion::SUPPORTED)
                .map_err(|_| validation(RigidWorldErrorKind::InvalidIdentifier))?,
            scenario_id,
            timelines: timelines.into_boxed_slice(),
        };
        validate_result_bounds(&record)?;
        Ok(record)
    }

    #[must_use]
    pub const fn request_id(&self) -> &RequestId {
        &self.request_id
    }

    #[must_use]
    pub const fn scenario_id(&self) -> &ScenarioId {
        &self.scenario_id
    }

    #[must_use]
    pub fn timelines(&self) -> &[RigidWorldTimelineResult] {
        &self.timelines
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RigidWorldResultKind {
    RigidWorldResult,
}

/// Decodes one newline-complete bounded rigid-world result record.
///
/// # Errors
///
/// Returns [`RigidWorldDecodeError`] for framing, closed-field, version, result
/// count, sensor-manifold, or aggregate-limit failures.
pub fn decode_rigid_world_result_jsonl(
    bytes: &[u8],
    limits: &HarnessLimits,
) -> Result<RigidWorldResultRecord, RigidWorldDecodeError> {
    let result = decode_jsonl::<RigidWorldResultRecord>(bytes, limits, RecordLimit::Output)?;
    validate_result_bounds(&result)?;
    Ok(result)
}

/// Validates result identity, checkpoints, counts, and declaration ordering.
///
/// # Errors
///
/// Returns [`RigidWorldDecodeError`] when the result disagrees with the request
/// declaration or checkpoint contract.
pub fn validate_rigid_world_result_against_request(
    request: &RigidWorldRequestRecord,
    result: &RigidWorldResultRecord,
) -> Result<(), RigidWorldDecodeError> {
    if result.request_id() != request.request_id()
        || result.scenario_id() != request.scenario().scenario_id()
        || result.timelines().len() != request.scenario().timelines().len()
    {
        return Err(validation(RigidWorldErrorKind::ResultTimelineMismatch));
    }

    for (expected_timeline, actual_timeline) in request
        .scenario()
        .timelines()
        .iter()
        .zip(result.timelines())
    {
        if expected_timeline.witness_family() != actual_timeline.witness_family
            || expected_timeline.checkpoints().len() != actual_timeline.checkpoints.len()
        {
            return Err(validation(RigidWorldErrorKind::ResultTimelineMismatch));
        }

        for (checkpoint_index, (expected, actual)) in expected_timeline
            .checkpoints()
            .iter()
            .zip(actual_timeline.checkpoints.iter())
            .enumerate()
        {
            if expected.checkpoint_id() != &actual.checkpoint_id
                || expected.phase() != actual.phase.as_ref()
                || expected.counts() != actual.counts
            {
                return Err(validation(RigidWorldErrorKind::ResultCheckpointMismatch));
            }
            let live_identities =
                rigid_world_checkpoint_live_identities(expected_timeline, checkpoint_index)
                    .ok_or_else(|| validation(RigidWorldErrorKind::ResultCheckpointMismatch))?;
            validate_checkpoint_declaration_order(&live_identities, actual)?;
            validate_checkpoint_observations(
                expected_timeline,
                checkpoint_index,
                &actual.observations,
            )?;
        }
    }
    Ok(())
}

#[allow(
    clippy::too_many_lines,
    reason = "the retained phase observation ordering contract is kept in one explicit walk"
)]
fn validate_checkpoint_observations(
    timeline: &super::RigidWorldTimeline,
    checkpoint_index: usize,
    observations: &[RigidWorldObservation],
) -> Result<(), RigidWorldDecodeError> {
    let actions = rigid_world_checkpoint_action_window(timeline, checkpoint_index)
        .ok_or_else(|| validation(RigidWorldErrorKind::ResultCheckpointMismatch))?;
    validate_phase8_observation_contract(timeline.witness_family(), actions, observations)?;
    let first_action = actions
        .first()
        .ok_or_else(|| validation(RigidWorldErrorKind::ResultCheckpointMismatch))?;
    let action_start = timeline
        .actions()
        .iter()
        .position(|action| action.action_id() == first_action.action_id())
        .ok_or_else(|| validation(RigidWorldErrorKind::ResultCheckpointMismatch))?;
    let fixture_owners = rigid_fixture_owners(timeline);
    let mut live_bodies = HashSet::new();
    let mut live_fixtures = HashSet::new();
    let mut created_body_ids = HashSet::new();
    let mut phase9_state = Phase9ResultState::new(timeline);
    let mut phase10_state = Phase10ResultState::default();
    for action in &timeline.actions()[..action_start] {
        super::types::apply_lifecycle_action(
            action.action(),
            &fixture_owners,
            &mut live_bodies,
            &mut live_fixtures,
        );
        if let RigidWorldAction::CreateBody { body_id } = action.action() {
            created_body_ids.insert(body_id.clone());
        }
        if let RigidWorldAction::Particle { action } = action.action() {
            phase9_state.apply(action);
        } else if matches!(action.action(), RigidWorldAction::Step { .. }) {
            phase9_state.advance_step();
        }
        if let RigidWorldAction::ParticleGroup { operation } = action.action() {
            phase10_state.apply(operation);
        }
    }

    let mut actual_observations = observations.iter().peekable();
    for action in actions {
        super::types::apply_lifecycle_action(
            action.action(),
            &fixture_owners,
            &mut live_bodies,
            &mut live_fixtures,
        );
        if let RigidWorldAction::CreateBody { body_id } = action.action() {
            created_body_ids.insert(body_id.clone());
        }
        if let RigidWorldAction::Particle {
            action: particle_action,
        } = action.action()
        {
            phase9_state.apply(particle_action);
            let Some(actual) = actual_observations.next() else {
                return Err(validation(RigidWorldErrorKind::ResultObservationMismatch));
            };
            let live_identities =
                rigid_world_live_identities(timeline, &live_bodies, &live_fixtures);
            phase9_state.validate(particle_action, &live_identities, actual)?;
            continue;
        }
        if let RigidWorldAction::ParticleGroup { operation } = action.action() {
            phase10_state.apply(operation);
            if matches!(operation, Phase10Operation::InspectState) {
                let Some(actual) = actual_observations.next() else {
                    return Err(validation(RigidWorldErrorKind::ResultObservationMismatch));
                };
                let RigidWorldObservation::ParticleGroup { observation } = actual else {
                    return Err(validation(RigidWorldErrorKind::ResultObservationMismatch));
                };
                let Phase10Observation::State { state } = observation;
                let live_identities =
                    rigid_world_live_identities(timeline, &live_bodies, &live_fixtures);
                if state.body_contacts.iter().any(|contact| {
                    !fixture_belongs_to_live_body(
                        &live_identities.body_ids,
                        live_identities
                            .fixtures
                            .iter()
                            .map(|fixture| (fixture.fixture_id(), fixture.owner_body_id())),
                        &contact.fixture_id,
                        &contact.body_id,
                    )
                }) {
                    return Err(validation(RigidWorldErrorKind::ResultObservationMismatch));
                }
                phase10_state
                    .validate(state, &created_body_ids)
                    .map_err(|_| validation(RigidWorldErrorKind::ResultObservationMismatch))?;
            }
            continue;
        }
        if matches!(action.action(), RigidWorldAction::Step { .. }) {
            phase9_state.advance_step();
        }
        let Some(expected) = expected_observation(action.action()) else {
            continue;
        };
        let live_identities = rigid_world_live_identities(timeline, &live_bodies, &live_fixtures);
        while matches!(
            actual_observations.peek(),
            Some(RigidWorldObservation::Lifecycle { .. })
        ) {
            actual_observations.next();
        }
        if matches!(expected, ExpectedObservation::Reconstruction) {
            let mut ordinal = 0_u32;
            while let Some(RigidWorldObservation::Reconstruction { record }) =
                actual_observations.peek()
            {
                if record.ordinal != ordinal {
                    return Err(validation(RigidWorldErrorKind::ResultObservationMismatch));
                }
                ordinal = ordinal
                    .checked_add(1)
                    .ok_or_else(|| validation(RigidWorldErrorKind::AggregateLimitExceeded))?;
                actual_observations.next();
            }
            if ordinal == 0 {
                return Err(validation(RigidWorldErrorKind::ResultObservationMismatch));
            }
            continue;
        }
        let Some(actual) = actual_observations.next() else {
            return Err(validation(RigidWorldErrorKind::ResultObservationMismatch));
        };
        if !expected.matches(&live_identities, actual) {
            return Err(validation(RigidWorldErrorKind::ResultObservationMismatch));
        }
    }
    if actual_observations
        .any(|observation| !matches!(observation, RigidWorldObservation::Lifecycle { .. }))
    {
        return Err(validation(RigidWorldErrorKind::ResultObservationMismatch));
    }
    Ok(())
}

fn validate_checkpoint_declaration_order(
    expected: &RigidCheckpointLiveIdentities<'_>,
    checkpoint: &RigidWorldCheckpointResult,
) -> Result<(), RigidWorldDecodeError> {
    let actual_bodies = checkpoint
        .bodies
        .iter()
        .map(|body| &body.body_id)
        .collect::<Vec<_>>();
    if expected.body_ids != actual_bodies {
        return Err(validation(
            RigidWorldErrorKind::ResultDeclarationOrderMismatch,
        ));
    }

    let actual_fixtures = checkpoint
        .fixtures
        .iter()
        .map(|fixture| &fixture.fixture_id)
        .collect::<Vec<_>>();
    if expected.fixture_ids != actual_fixtures {
        return Err(validation(
            RigidWorldErrorKind::ResultDeclarationOrderMismatch,
        ));
    }
    Ok(())
}

/// Returns the actions owned by one checkpoint, excluding actions owned by the prior checkpoint.
#[must_use]
pub fn rigid_world_checkpoint_action_window(
    timeline: &super::RigidWorldTimeline,
    checkpoint_index: usize,
) -> Option<&[super::RigidWorldActionRecord]> {
    let checkpoint = timeline.checkpoints().get(checkpoint_index)?;
    let action_end = timeline
        .actions()
        .iter()
        .position(|action| action.action_id() == checkpoint.after_action_id())?;
    let action_start = if checkpoint_index == 0 {
        0
    } else {
        let previous = timeline.checkpoints().get(checkpoint_index - 1)?;
        timeline
            .actions()
            .iter()
            .position(|action| action.action_id() == previous.after_action_id())?
            .checked_add(1)?
    };
    timeline.actions().get(action_start..=action_end)
}

/// Replays lifecycle actions through one checkpoint and returns exact live identities.
#[must_use]
pub fn rigid_world_checkpoint_live_identities(
    timeline: &super::RigidWorldTimeline,
    checkpoint_index: usize,
) -> Option<RigidCheckpointLiveIdentities<'_>> {
    let action_window = rigid_world_checkpoint_action_window(timeline, checkpoint_index)?;
    let action_end = action_window.last()?.action_id();
    let action_end = timeline
        .actions()
        .iter()
        .position(|action| action.action_id() == action_end)?;
    let fixture_owners = rigid_fixture_owners(timeline);
    let mut live_bodies = HashSet::new();
    let mut live_fixtures = HashSet::new();
    for action in &timeline.actions()[..=action_end] {
        super::types::apply_lifecycle_action(
            action.action(),
            &fixture_owners,
            &mut live_bodies,
            &mut live_fixtures,
        );
    }
    Some(rigid_world_live_identities(
        timeline,
        &live_bodies,
        &live_fixtures,
    ))
}

fn rigid_fixture_owners(timeline: &super::RigidWorldTimeline) -> HashMap<ScenarioId, ScenarioId> {
    timeline
        .fixtures()
        .iter()
        .map(|fixture| {
            (
                fixture.fixture_id().clone(),
                fixture.owner_body_id().clone(),
            )
        })
        .collect()
}

fn rigid_world_live_identities<'a>(
    timeline: &'a super::RigidWorldTimeline,
    live_bodies: &HashSet<ScenarioId>,
    live_fixtures: &HashSet<ScenarioId>,
) -> RigidCheckpointLiveIdentities<'a> {
    let fixtures = timeline
        .fixtures()
        .iter()
        .filter(|fixture| live_fixtures.contains(fixture.fixture_id()))
        .collect::<Vec<_>>();
    RigidCheckpointLiveIdentities {
        body_ids: timeline
            .bodies()
            .iter()
            .map(super::RigidBodyDeclaration::body_id)
            .filter(|body_id| live_bodies.contains(*body_id))
            .collect(),
        fixture_ids: fixtures
            .iter()
            .map(|fixture| fixture.fixture_id())
            .collect(),
        fixtures,
    }
}

fn validate_result_bounds(result: &RigidWorldResultRecord) -> Result<(), RigidWorldDecodeError> {
    if result.timelines.is_empty() || result.timelines.len() > MAXIMUM_RESULT_TIMELINES {
        return Err(validation(RigidWorldErrorKind::AggregateLimitExceeded));
    }
    let mut aggregate = 0_usize;
    for timeline in &result.timelines {
        if timeline.checkpoints.is_empty()
            || timeline.checkpoints.len() > MAXIMUM_RESULT_CHECKPOINTS
        {
            return Err(validation(RigidWorldErrorKind::AggregateLimitExceeded));
        }
        for checkpoint in &timeline.checkpoints {
            if checkpoint.bodies.len() > MAXIMUM_RESULT_BODIES
                || checkpoint.fixtures.len() > MAXIMUM_RESULT_FIXTURES
                || checkpoint.contacts.len() > MAXIMUM_RESULT_CONTACTS
                || checkpoint.events.len() > MAXIMUM_RESULT_EVENTS
                || checkpoint.destructions.len() > MAXIMUM_RESULT_DESTRUCTIONS
                || checkpoint.observations.len() > MAXIMUM_RESULT_OBSERVATIONS
            {
                return Err(validation(RigidWorldErrorKind::AggregateLimitExceeded));
            }
            for observation in &checkpoint.observations {
                if let RigidWorldObservation::ParticleGroup { observation } = observation {
                    observation
                        .validate()
                        .map_err(|_| validation(RigidWorldErrorKind::InvalidParticleGroupResult))?;
                }
            }
            let manifold_points = checkpoint
                .contacts
                .iter()
                .map(|contact| {
                    contact
                        .maybe_manifold
                        .as_ref()
                        .map_or(0, |manifold| manifold.points.len())
                })
                .sum::<usize>();
            if checkpoint
                .observations
                .iter()
                .any(|observation| match observation {
                    RigidWorldObservation::Query { observation } => {
                        observation.occurrences.len() > MAXIMUM_QUERY_OCCURRENCES
                    }
                    RigidWorldObservation::RayCast { observation } => {
                        observation.hits.len() > MAXIMUM_RAY_HITS
                    }
                    RigidWorldObservation::Rope { snapshot } => {
                        snapshot.vertices.len() > super::RIGID_WORLD_MAXIMUM_ROPE_VERTICES
                    }
                    RigidWorldObservation::Particle { observation } => {
                        phase9_observation_exceeds_bounds(observation)
                    }
                    _ => false,
                })
            {
                return Err(validation(RigidWorldErrorKind::AggregateLimitExceeded));
            }
            if checkpoint.contacts.iter().any(|contact| {
                contact
                    .maybe_manifold
                    .as_ref()
                    .is_some_and(|manifold| manifold.points.len() > MAXIMUM_MANIFOLD_POINTS)
                    || contact.sensor && contact.maybe_manifold.is_some()
            }) {
                return Err(validation(RigidWorldErrorKind::InvalidGeometry));
            }
            let actual_counts = RigidExpectedCounts {
                bodies: checked_u32(checkpoint.bodies.len())?,
                fixtures: checked_u32(checkpoint.fixtures.len())?,
                contacts: checked_u32(checkpoint.contacts.len())?,
                manifold_points: checked_u32(manifold_points)?,
                events: checked_u32(checkpoint.events.len())?,
                destructions: checked_u32(checkpoint.destructions.len())?,
            };
            if actual_counts != checkpoint.counts {
                return Err(validation(RigidWorldErrorKind::ExpectedCountMismatch));
            }
            aggregate = aggregate
                .checked_add(
                    checkpoint.bodies.len()
                        + checkpoint.fixtures.len()
                        + checkpoint.contacts.len()
                        + manifold_points
                        + checkpoint.events.len()
                        + checkpoint.destructions.len()
                        + checkpoint.observations.len(),
                )
                .ok_or_else(|| validation(RigidWorldErrorKind::AggregateLimitExceeded))?;
            if aggregate > MAXIMUM_RESULT_AGGREGATE {
                return Err(validation(RigidWorldErrorKind::AggregateLimitExceeded));
            }
        }
    }
    Ok(())
}

fn phase9_observation_exceeds_bounds(observation: &Phase9ParticleObservation) -> bool {
    match observation {
        Phase9ParticleObservation::System { particle_ids, .. }
        | Phase9ParticleObservation::Query { particle_ids, .. }
        | Phase9ParticleObservation::MixedState { particle_ids, .. } => {
            particle_ids.len() > PHASE9_MAXIMUM_IDENTITIES
        }
        Phase9ParticleObservation::RayCast {
            particle_ids,
            fractions_bits,
            ..
        } => {
            particle_ids.len() > PHASE9_MAXIMUM_IDENTITIES
                || fractions_bits.len() != particle_ids.len()
        }
        Phase9ParticleObservation::Statistics { statistics } => {
            statistics.stuck_particle_ids.len() > PHASE9_MAXIMUM_IDENTITIES
        }
        Phase9ParticleObservation::Particle { .. }
        | Phase9ParticleObservation::Lifecycle { .. }
        | Phase9ParticleObservation::ParticleContact { .. }
        | Phase9ParticleObservation::BodyContact { .. } => false,
    }
}

fn observations_are_empty(observations: &[RigidWorldObservation]) -> bool {
    observations.is_empty()
}

fn checked_u32(value: usize) -> Result<u32, RigidWorldDecodeError> {
    u32::try_from(value).map_err(|_| validation(RigidWorldErrorKind::AggregateLimitExceeded))
}
