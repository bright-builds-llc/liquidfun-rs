use serde::{Deserialize, Serialize};

mod contract;
#[cfg(test)]
#[path = "result/ownership_tests.rs"]
mod ownership_tests;
mod phase10;
mod phase8;
mod phase9;

pub use contract::{
    decode_rigid_world_result_jsonl, rigid_world_checkpoint_action_window,
    rigid_world_checkpoint_live_identities, validate_rigid_world_result_against_request,
};
use contract::{observations_are_empty, validate_result_bounds};
pub use phase10::*;

use super::{
    Phase9ParticleObservation, RigidBodyKind, RigidContactIdentity, RigidExpectedCounts,
    RigidFilterBits, RigidWorldAction, RigidWorldDecodeError, RigidWorldErrorKind,
    RigidWorldWitnessFamily, validation,
};
use crate::{
    FloatBits, ProtocolVersion, RequestId, ScenarioId, TraceSchemaVersion, TransformBits, Vec2Bits,
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
