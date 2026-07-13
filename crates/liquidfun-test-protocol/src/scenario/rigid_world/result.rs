use serde::{Deserialize, Serialize};

use super::{
    RigidBodyKind, RigidContactIdentity, RigidExpectedCounts, RigidFilterBits, RigidWorldAction,
    RigidWorldDecodeError, RigidWorldErrorKind, RigidWorldRequestRecord, RigidWorldWitnessFamily,
    validation,
};
use crate::{
    FloatBits, HarnessLimits, ProtocolVersion, RecordLimit, RequestId, ScenarioId,
    TraceSchemaVersion, TransformBits, Vec2Bits, decode_jsonl,
};

const MAXIMUM_RESULT_TIMELINES: usize = 9;
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidRayObservation {
    pub completion: RigidRayCompletion,
    pub hits: Box<[RigidRayHitObservation]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RigidWorldObservation {
    BodyState { state: RigidBodyControlSnapshot },
    Step { outcome: RigidStepOutcome },
    Query { observation: RigidQueryObservation },
    RayCast { observation: RigidRayObservation },
    OriginShift { shift: Vec2Bits },
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

        let mut action_start = 0;
        for (expected, actual) in expected_timeline
            .checkpoints()
            .iter()
            .zip(actual_timeline.checkpoints.iter())
        {
            if expected.checkpoint_id() != &actual.checkpoint_id
                || expected.phase() != actual.phase.as_ref()
                || expected.counts() != actual.counts
            {
                return Err(validation(RigidWorldErrorKind::ResultCheckpointMismatch));
            }
            validate_checkpoint_declaration_order(expected_timeline, actual)?;
            let action_end = expected_timeline
                .actions()
                .iter()
                .position(|action| action.action_id() == expected.after_action_id())
                .ok_or_else(|| validation(RigidWorldErrorKind::ResultCheckpointMismatch))?;
            validate_checkpoint_observations(
                &expected_timeline.actions()[action_start..=action_end],
                &actual.observations,
            )?;
            action_start = action_end + 1;
        }
    }
    Ok(())
}

fn validate_checkpoint_observations(
    actions: &[super::RigidWorldActionRecord],
    observations: &[RigidWorldObservation],
) -> Result<(), RigidWorldDecodeError> {
    let expected = actions
        .iter()
        .filter_map(|action| expected_observation(action.action()))
        .collect::<Vec<_>>();
    if expected.len() != observations.len()
        || expected
            .iter()
            .zip(observations)
            .any(|(expected, actual)| !expected.matches(actual))
    {
        return Err(validation(RigidWorldErrorKind::ResultObservationMismatch));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum ExpectedObservation<'a> {
    BodyState(&'a ScenarioId),
    Step,
    Query,
    RayCast,
    OriginShift(Vec2Bits),
}

impl ExpectedObservation<'_> {
    fn matches(self, actual: &RigidWorldObservation) -> bool {
        match (self, actual) {
            (Self::BodyState(expected), RigidWorldObservation::BodyState { state }) => {
                expected == &state.body_id
            }
            (Self::Step, RigidWorldObservation::Step { .. })
            | (Self::Query, RigidWorldObservation::Query { .. })
            | (Self::RayCast, RigidWorldObservation::RayCast { .. }) => true,
            (Self::OriginShift(expected), RigidWorldObservation::OriginShift { shift }) => {
                expected == *shift
            }
            _ => false,
        }
    }
}

fn expected_observation(action: &RigidWorldAction) -> Option<ExpectedObservation<'_>> {
    match action {
        RigidWorldAction::SetLinearVelocity { body_id, .. }
        | RigidWorldAction::SetAngularVelocity { body_id, .. }
        | RigidWorldAction::ApplyForce { body_id, .. }
        | RigidWorldAction::ApplyTorque { body_id, .. }
        | RigidWorldAction::ApplyLinearImpulse { body_id, .. }
        | RigidWorldAction::ApplyAngularImpulse { body_id, .. }
        | RigidWorldAction::SetBodyDamping { body_id, .. }
        | RigidWorldAction::SetGravityScale { body_id, .. }
        | RigidWorldAction::SetFixedRotation { body_id, .. }
        | RigidWorldAction::SetSleepingAllowed { body_id, .. }
        | RigidWorldAction::SetAwake { body_id, .. }
        | RigidWorldAction::SetBullet { body_id, .. } => {
            Some(ExpectedObservation::BodyState(body_id))
        }
        RigidWorldAction::ConfiguredStep { .. } => Some(ExpectedObservation::Step),
        RigidWorldAction::QueryAabb { .. } => Some(ExpectedObservation::Query),
        RigidWorldAction::RayCast { .. } => Some(ExpectedObservation::RayCast),
        RigidWorldAction::ShiftOrigin { shift } => Some(ExpectedObservation::OriginShift(*shift)),
        _ => None,
    }
}

fn validate_checkpoint_declaration_order(
    timeline: &super::RigidWorldTimeline,
    checkpoint: &RigidWorldCheckpointResult,
) -> Result<(), RigidWorldDecodeError> {
    let declared_bodies = timeline
        .bodies()
        .iter()
        .map(super::RigidBodyDeclaration::body_id)
        .collect::<Vec<_>>();
    let actual_bodies = checkpoint
        .bodies
        .iter()
        .map(|body| &body.body_id)
        .collect::<Vec<_>>();
    if !is_declared_subsequence(&declared_bodies, &actual_bodies) {
        return Err(validation(
            RigidWorldErrorKind::ResultDeclarationOrderMismatch,
        ));
    }

    let declared_fixtures = timeline
        .fixtures()
        .iter()
        .map(super::RigidFixtureDeclaration::fixture_id)
        .collect::<Vec<_>>();
    let actual_fixtures = checkpoint
        .fixtures
        .iter()
        .map(|fixture| &fixture.fixture_id)
        .collect::<Vec<_>>();
    if !is_declared_subsequence(&declared_fixtures, &actual_fixtures) {
        return Err(validation(
            RigidWorldErrorKind::ResultDeclarationOrderMismatch,
        ));
    }
    Ok(())
}

fn is_declared_subsequence<T: PartialEq>(declared: &[T], actual: &[T]) -> bool {
    let mut next = 0;
    for item in actual {
        let Some(offset) = declared[next..]
            .iter()
            .position(|declared_item| declared_item == item)
        else {
            return false;
        };
        next += offset + 1;
    }
    true
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

fn observations_are_empty(observations: &[RigidWorldObservation]) -> bool {
    observations.is_empty()
}

fn checked_u32(value: usize) -> Result<u32, RigidWorldDecodeError> {
    u32::try_from(value).map_err(|_| validation(RigidWorldErrorKind::AggregateLimitExceeded))
}
