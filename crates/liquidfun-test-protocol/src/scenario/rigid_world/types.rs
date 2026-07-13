use serde::{Deserialize, Serialize};

use super::witness_registry::{RigidWorldWitness, RigidWorldWitnessFamily};
use crate::{
    CodecError, FloatBits, ProtocolVersion, RequestId, ScenarioId, ScenarioSchemaVersion,
    ScenarioSource, Sha256Hex, ToleranceProfileVersion, TraceSchemaVersion, TransformBits,
    Vec2Bits,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidWorldErrorKind {
    NoTimelines,
    DuplicateWitnessFamily,
    MissingWitnessFamily,
    DuplicateBodyId,
    DuplicateFixtureId,
    DuplicateActionId,
    DuplicateCheckpointId,
    DuplicateWitness,
    InvalidIdentifier,
    InvalidSource,
    InvalidGeometry,
    InvalidMaterial,
    InvalidOwner,
    UnknownBody,
    UnknownFixture,
    InvalidActionOrder,
    InvalidCheckpointOrder,
    CheckpointPhaseMismatch,
    ExpectedCountMismatch,
    MissingWitness,
    UnexpectedWitness,
    InvalidContactIdentity,
    InvalidBodyControl,
    InvalidStepConfiguration,
    InvalidQueryDirective,
    InvalidRayDirective,
    AggregateLimitExceeded,
    ResultTimelineMismatch,
    ResultCheckpointMismatch,
    ResultDeclarationOrderMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidWakePolicy {
    Wake,
    PreserveSleep,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidAabbBits {
    pub lower: Vec2Bits,
    pub upper: Vec2Bits,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidFixtureChildSelector {
    pub fixture_id: ScenarioId,
    pub child_index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidQueryDirective {
    Continue,
    Terminate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidQueryDirectiveRule {
    pub target: RigidFixtureChildSelector,
    pub directive: RigidQueryDirective,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RigidRayDirective {
    Ignore,
    Terminate,
    Continue,
    Clip { fraction_bits: FloatBits },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidRayDirectiveRule {
    pub target: RigidFixtureChildSelector,
    pub directive: RigidRayDirective,
}

#[derive(Debug, thiserror::Error)]
pub enum RigidWorldDecodeError {
    #[error(transparent)]
    Codec(#[from] CodecError),
    #[error("rigid-world validation failed: {0:?}")]
    Validation(RigidWorldErrorKind),
}

impl RigidWorldDecodeError {
    #[must_use]
    pub const fn rigid_world_kind(&self) -> Option<RigidWorldErrorKind> {
        match self {
            Self::Codec(_) => None,
            Self::Validation(kind) => Some(*kind),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidBodyKind {
    Static,
    Kinematic,
    Dynamic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RigidFixtureShape {
    Circle {
        center: Vec2Bits,
        radius_bits: FloatBits,
    },
    Polygon {
        vertices: Box<[Vec2Bits]>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidFilterBits {
    category_bits: u16,
    mask_bits: u16,
    group_index: i16,
}

impl RigidFilterBits {
    #[must_use]
    pub const fn new(category_bits: u16, mask_bits: u16, group_index: i16) -> Self {
        Self {
            category_bits,
            mask_bits,
            group_index,
        }
    }

    #[must_use]
    pub const fn category_bits(self) -> u16 {
        self.category_bits
    }

    #[must_use]
    pub const fn mask_bits(self) -> u16 {
        self.mask_bits
    }

    #[must_use]
    pub const fn group_index(self) -> i16 {
        self.group_index
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RigidBodyDeclaration {
    pub(super) body_id: ScenarioId,
    pub(super) body_kind: RigidBodyKind,
    pub(super) transform: TransformBits,
    pub(super) active: bool,
}

impl RigidBodyDeclaration {
    #[must_use]
    pub const fn body_id(&self) -> &ScenarioId {
        &self.body_id
    }

    #[must_use]
    pub const fn body_kind(&self) -> RigidBodyKind {
        self.body_kind
    }

    #[must_use]
    pub const fn transform(&self) -> TransformBits {
        self.transform
    }

    #[must_use]
    pub const fn active(&self) -> bool {
        self.active
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RigidFixtureDeclaration {
    pub(super) fixture_id: ScenarioId,
    pub(super) owner_body_id: ScenarioId,
    pub(super) shape: RigidFixtureShape,
    pub(super) density_bits: FloatBits,
    pub(super) friction_bits: FloatBits,
    pub(super) restitution_bits: FloatBits,
    pub(super) sensor: bool,
    pub(super) filter: RigidFilterBits,
}

impl RigidFixtureDeclaration {
    #[must_use]
    pub const fn fixture_id(&self) -> &ScenarioId {
        &self.fixture_id
    }

    #[must_use]
    pub const fn owner_body_id(&self) -> &ScenarioId {
        &self.owner_body_id
    }

    #[must_use]
    pub const fn shape(&self) -> &RigidFixtureShape {
        &self.shape
    }

    #[must_use]
    pub const fn density_bits(&self) -> FloatBits {
        self.density_bits
    }

    #[must_use]
    pub const fn friction_bits(&self) -> FloatBits {
        self.friction_bits
    }

    #[must_use]
    pub const fn restitution_bits(&self) -> FloatBits {
        self.restitution_bits
    }

    #[must_use]
    pub const fn sensor(&self) -> bool {
        self.sensor
    }

    #[must_use]
    pub const fn filter(&self) -> RigidFilterBits {
        self.filter
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RigidWorldAction {
    CreateBody {
        body_id: ScenarioId,
    },
    CreateFixture {
        fixture_id: ScenarioId,
    },
    InspectBody {
        body_id: ScenarioId,
    },
    InspectFixture {
        fixture_id: ScenarioId,
    },
    SetBodyTransform {
        body_id: ScenarioId,
        transform: TransformBits,
    },
    SetBodyType {
        body_id: ScenarioId,
        body_kind: RigidBodyKind,
    },
    SetBodyActive {
        body_id: ScenarioId,
        active: bool,
    },
    SetLinearVelocity {
        body_id: ScenarioId,
        velocity: Vec2Bits,
    },
    SetAngularVelocity {
        body_id: ScenarioId,
        angular_velocity_bits: FloatBits,
    },
    ApplyForce {
        body_id: ScenarioId,
        force: Vec2Bits,
        point: Vec2Bits,
        wake_policy: RigidWakePolicy,
    },
    ApplyTorque {
        body_id: ScenarioId,
        torque_bits: FloatBits,
        wake_policy: RigidWakePolicy,
    },
    ApplyLinearImpulse {
        body_id: ScenarioId,
        impulse: Vec2Bits,
        point: Vec2Bits,
        wake_policy: RigidWakePolicy,
    },
    ApplyAngularImpulse {
        body_id: ScenarioId,
        impulse_bits: FloatBits,
        wake_policy: RigidWakePolicy,
    },
    SetBodyDamping {
        body_id: ScenarioId,
        linear_damping_bits: FloatBits,
        angular_damping_bits: FloatBits,
    },
    SetGravityScale {
        body_id: ScenarioId,
        gravity_scale_bits: FloatBits,
    },
    SetFixedRotation {
        body_id: ScenarioId,
        fixed_rotation: bool,
    },
    SetSleepingAllowed {
        body_id: ScenarioId,
        sleeping_allowed: bool,
    },
    SetAwake {
        body_id: ScenarioId,
        awake: bool,
    },
    SetBullet {
        body_id: ScenarioId,
        bullet: bool,
    },
    SetFixtureSensor {
        fixture_id: ScenarioId,
        sensor: bool,
    },
    SetFixtureMaterial {
        fixture_id: ScenarioId,
        friction_bits: FloatBits,
        restitution_bits: FloatBits,
    },
    SetFixtureFilter {
        fixture_id: ScenarioId,
        filter: RigidFilterBits,
    },
    SetFixtureDensity {
        fixture_id: ScenarioId,
        density_bits: FloatBits,
    },
    ResetMassData {
        body_id: ScenarioId,
    },
    SetCustomMassData {
        body_id: ScenarioId,
        mass_bits: FloatBits,
        center: Vec2Bits,
        inertia_bits: FloatBits,
    },
    Step {
        timestep_bits: FloatBits,
        velocity_iterations: u32,
        position_iterations: u32,
    },
    SetWorldGravity {
        gravity: Vec2Bits,
    },
    SetAutomaticForceClearing {
        enabled: bool,
    },
    SetWarmStarting {
        enabled: bool,
    },
    SetContinuousPhysics {
        enabled: bool,
    },
    SetSubStepping {
        enabled: bool,
    },
    ClearForces,
    ConfiguredStep {
        timestep_bits: FloatBits,
        velocity_iterations: u32,
        position_iterations: u32,
        continuous_work_budget: u32,
    },
    QueryAabb {
        aabb: RigidAabbBits,
        directive_rules: Box<[RigidQueryDirectiveRule]>,
    },
    RayCast {
        start: Vec2Bits,
        end: Vec2Bits,
        directive_rules: Box<[RigidRayDirectiveRule]>,
    },
    ShiftOrigin {
        shift: Vec2Bits,
    },
    DestroyFixture {
        fixture_id: ScenarioId,
    },
    DestroyBody {
        body_id: ScenarioId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum RigidWorldActionKind {
    CreateBody,
    CreateFixture,
    InspectBody,
    InspectFixture,
    SetBodyTransform,
    SetBodyType,
    SetBodyActive,
    SetLinearVelocity,
    SetAngularVelocity,
    ApplyForce,
    ApplyTorque,
    ApplyLinearImpulse,
    ApplyAngularImpulse,
    SetBodyDamping,
    SetGravityScale,
    SetFixedRotation,
    SetSleepingAllowed,
    SetAwake,
    SetBullet,
    SetFixtureSensor,
    SetFixtureMaterial,
    SetFixtureFilter,
    SetFixtureDensity,
    ResetMassData,
    SetCustomMassData,
    Step,
    SetWorldGravity,
    SetAutomaticForceClearing,
    SetWarmStarting,
    SetContinuousPhysics,
    SetSubStepping,
    ClearForces,
    ConfiguredStep,
    QueryAabb,
    RayCast,
    ShiftOrigin,
    DestroyFixture,
    DestroyBody,
}

impl RigidWorldAction {
    pub(super) const fn action_kind(&self) -> RigidWorldActionKind {
        match self {
            Self::CreateBody { .. } => RigidWorldActionKind::CreateBody,
            Self::CreateFixture { .. } => RigidWorldActionKind::CreateFixture,
            Self::InspectBody { .. } => RigidWorldActionKind::InspectBody,
            Self::InspectFixture { .. } => RigidWorldActionKind::InspectFixture,
            Self::SetBodyTransform { .. } => RigidWorldActionKind::SetBodyTransform,
            Self::SetBodyType { .. } => RigidWorldActionKind::SetBodyType,
            Self::SetBodyActive { .. } => RigidWorldActionKind::SetBodyActive,
            Self::SetLinearVelocity { .. } => RigidWorldActionKind::SetLinearVelocity,
            Self::SetAngularVelocity { .. } => RigidWorldActionKind::SetAngularVelocity,
            Self::ApplyForce { .. } => RigidWorldActionKind::ApplyForce,
            Self::ApplyTorque { .. } => RigidWorldActionKind::ApplyTorque,
            Self::ApplyLinearImpulse { .. } => RigidWorldActionKind::ApplyLinearImpulse,
            Self::ApplyAngularImpulse { .. } => RigidWorldActionKind::ApplyAngularImpulse,
            Self::SetBodyDamping { .. } => RigidWorldActionKind::SetBodyDamping,
            Self::SetGravityScale { .. } => RigidWorldActionKind::SetGravityScale,
            Self::SetFixedRotation { .. } => RigidWorldActionKind::SetFixedRotation,
            Self::SetSleepingAllowed { .. } => RigidWorldActionKind::SetSleepingAllowed,
            Self::SetAwake { .. } => RigidWorldActionKind::SetAwake,
            Self::SetBullet { .. } => RigidWorldActionKind::SetBullet,
            Self::SetFixtureSensor { .. } => RigidWorldActionKind::SetFixtureSensor,
            Self::SetFixtureMaterial { .. } => RigidWorldActionKind::SetFixtureMaterial,
            Self::SetFixtureFilter { .. } => RigidWorldActionKind::SetFixtureFilter,
            Self::SetFixtureDensity { .. } => RigidWorldActionKind::SetFixtureDensity,
            Self::ResetMassData { .. } => RigidWorldActionKind::ResetMassData,
            Self::SetCustomMassData { .. } => RigidWorldActionKind::SetCustomMassData,
            Self::Step { .. } => RigidWorldActionKind::Step,
            Self::SetWorldGravity { .. } => RigidWorldActionKind::SetWorldGravity,
            Self::SetAutomaticForceClearing { .. } => {
                RigidWorldActionKind::SetAutomaticForceClearing
            }
            Self::SetWarmStarting { .. } => RigidWorldActionKind::SetWarmStarting,
            Self::SetContinuousPhysics { .. } => RigidWorldActionKind::SetContinuousPhysics,
            Self::SetSubStepping { .. } => RigidWorldActionKind::SetSubStepping,
            Self::ClearForces => RigidWorldActionKind::ClearForces,
            Self::ConfiguredStep { .. } => RigidWorldActionKind::ConfiguredStep,
            Self::QueryAabb { .. } => RigidWorldActionKind::QueryAabb,
            Self::RayCast { .. } => RigidWorldActionKind::RayCast,
            Self::ShiftOrigin { .. } => RigidWorldActionKind::ShiftOrigin,
            Self::DestroyFixture { .. } => RigidWorldActionKind::DestroyFixture,
            Self::DestroyBody { .. } => RigidWorldActionKind::DestroyBody,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RigidWorldActionRecord {
    pub(super) action_id: ScenarioId,
    pub(super) phase: Box<str>,
    pub(super) action: RigidWorldAction,
}

impl RigidWorldActionRecord {
    #[must_use]
    pub const fn action_id(&self) -> &ScenarioId {
        &self.action_id
    }

    #[must_use]
    pub fn phase(&self) -> &str {
        &self.phase
    }

    #[must_use]
    pub const fn action(&self) -> &RigidWorldAction {
        &self.action
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidExpectedCounts {
    pub bodies: u32,
    pub fixtures: u32,
    pub contacts: u32,
    pub manifold_points: u32,
    pub events: u32,
    pub destructions: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidContactIdentity {
    fixture_a_id: ScenarioId,
    child_a: u32,
    fixture_b_id: ScenarioId,
    child_b: u32,
    occurrence: u32,
}

impl RigidContactIdentity {
    /// Creates one oriented semantic fixture-child occurrence identity.
    ///
    /// # Errors
    ///
    /// Returns [`RigidWorldDecodeError`] when both fixture IDs are equal or the
    /// occurrence ordinal is zero.
    #[allow(
        clippy::similar_names,
        reason = "fixture_a_id and fixture_b_id mirror the oriented protocol contract"
    )]
    pub fn new(
        fixture_a_id: ScenarioId,
        child_a: u32,
        fixture_b_id: ScenarioId,
        child_b: u32,
        occurrence: u32,
    ) -> Result<Self, RigidWorldDecodeError> {
        if fixture_a_id == fixture_b_id || occurrence == 0 {
            return Err(validation(RigidWorldErrorKind::InvalidContactIdentity));
        }
        Ok(Self {
            fixture_a_id,
            child_a,
            fixture_b_id,
            child_b,
            occurrence,
        })
    }

    #[must_use]
    pub const fn fixture_a_id(&self) -> &ScenarioId {
        &self.fixture_a_id
    }

    #[must_use]
    pub const fn child_a(&self) -> u32 {
        self.child_a
    }

    #[must_use]
    pub const fn fixture_b_id(&self) -> &ScenarioId {
        &self.fixture_b_id
    }

    #[must_use]
    pub const fn child_b(&self) -> u32 {
        self.child_b
    }

    #[must_use]
    pub const fn occurrence(&self) -> u32 {
        self.occurrence
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RigidExpectedTransition {
    pub(super) witness: RigidWorldWitness,
    pub(super) maybe_contact: Option<RigidContactIdentity>,
}

impl RigidExpectedTransition {
    #[must_use]
    pub const fn witness(&self) -> RigidWorldWitness {
        self.witness
    }

    #[must_use]
    pub const fn maybe_contact(&self) -> Option<&RigidContactIdentity> {
        self.maybe_contact.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RigidExpectedCheckpoint {
    pub(super) checkpoint_id: ScenarioId,
    pub(super) after_action_id: ScenarioId,
    pub(super) phase: Box<str>,
    pub(super) counts: RigidExpectedCounts,
    pub(super) transitions: Box<[RigidExpectedTransition]>,
}

impl RigidExpectedCheckpoint {
    #[must_use]
    pub const fn checkpoint_id(&self) -> &ScenarioId {
        &self.checkpoint_id
    }

    #[must_use]
    pub const fn after_action_id(&self) -> &ScenarioId {
        &self.after_action_id
    }

    #[must_use]
    pub fn phase(&self) -> &str {
        &self.phase
    }

    #[must_use]
    pub const fn counts(&self) -> RigidExpectedCounts {
        self.counts
    }

    #[must_use]
    pub fn transitions(&self) -> &[RigidExpectedTransition] {
        &self.transitions
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RigidWorldTimeline {
    pub(super) witness_family: RigidWorldWitnessFamily,
    pub(super) bodies: Box<[RigidBodyDeclaration]>,
    pub(super) fixtures: Box<[RigidFixtureDeclaration]>,
    pub(super) actions: Box<[RigidWorldActionRecord]>,
    pub(super) checkpoints: Box<[RigidExpectedCheckpoint]>,
}

impl RigidWorldTimeline {
    #[must_use]
    pub const fn witness_family(&self) -> RigidWorldWitnessFamily {
        self.witness_family
    }

    #[must_use]
    pub fn bodies(&self) -> &[RigidBodyDeclaration] {
        &self.bodies
    }

    #[must_use]
    pub fn fixtures(&self) -> &[RigidFixtureDeclaration] {
        &self.fixtures
    }

    #[must_use]
    pub fn actions(&self) -> &[RigidWorldActionRecord] {
        &self.actions
    }

    #[must_use]
    pub fn checkpoints(&self) -> &[RigidExpectedCheckpoint] {
        &self.checkpoints
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RigidWorldScenario {
    pub(super) scenario_id: ScenarioId,
    pub(super) source: ScenarioSource,
    pub(super) timelines: Box<[RigidWorldTimeline]>,
}

impl RigidWorldScenario {
    #[must_use]
    pub const fn scenario_id(&self) -> &ScenarioId {
        &self.scenario_id
    }

    #[must_use]
    pub const fn source(&self) -> &ScenarioSource {
        &self.source
    }

    #[must_use]
    pub fn timelines(&self) -> &[RigidWorldTimeline] {
        &self.timelines
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RigidWorldRequestRecord {
    pub(super) protocol_version: ProtocolVersion,
    pub(super) record_kind: RigidWorldRequestKind,
    pub(super) request_id: RequestId,
    pub(super) scenario_schema_version: ScenarioSchemaVersion,
    pub(super) requested_trace_schema_version: TraceSchemaVersion,
    pub(super) tolerance_profile_version: ToleranceProfileVersion,
    pub(super) tolerance_profile_sha256: Sha256Hex,
    pub(super) scenario: RigidWorldScenario,
}

impl RigidWorldRequestRecord {
    #[must_use]
    pub const fn request_id(&self) -> &RequestId {
        &self.request_id
    }

    #[must_use]
    pub const fn scenario(&self) -> &RigidWorldScenario {
        &self.scenario
    }

    #[must_use]
    pub const fn tolerance_profile_sha256(&self) -> &Sha256Hex {
        &self.tolerance_profile_sha256
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum RigidWorldRequestKind {
    RigidWorldRequest,
}

pub(super) const fn validation(kind: RigidWorldErrorKind) -> RigidWorldDecodeError {
    RigidWorldDecodeError::Validation(kind)
}
