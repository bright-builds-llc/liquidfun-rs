use std::collections::{HashMap, HashSet};

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
    ResultObservationMismatch,
    DuplicateJointId,
    DuplicateRopeId,
    UnknownJoint,
    UnknownRope,
    InvalidJointDefinition,
    InvalidJointDependency,
    InvalidRopeDefinition,
    InvalidContactDirective,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidJointKind {
    Revolute,
    Prismatic,
    Distance,
    Pulley,
    Mouse,
    Gear,
    Wheel,
    Weld,
    Friction,
    Rope,
    Motor,
}

impl RigidJointKind {
    pub const ALL: [Self; 11] = [
        Self::Revolute,
        Self::Prismatic,
        Self::Distance,
        Self::Pulley,
        Self::Mouse,
        Self::Gear,
        Self::Wheel,
        Self::Weld,
        Self::Friction,
        Self::Rope,
        Self::Motor,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RigidJointDefinition {
    Revolute {
        local_anchor_a: Vec2Bits,
        local_anchor_b: Vec2Bits,
        reference_angle_bits: FloatBits,
        lower_angle_bits: FloatBits,
        upper_angle_bits: FloatBits,
        motor_speed_bits: FloatBits,
        max_motor_torque_bits: FloatBits,
        limit_enabled: bool,
        motor_enabled: bool,
    },
    Prismatic {
        local_anchor_a: Vec2Bits,
        local_anchor_b: Vec2Bits,
        local_axis_a: Vec2Bits,
        reference_angle_bits: FloatBits,
        lower_translation_bits: FloatBits,
        upper_translation_bits: FloatBits,
        motor_speed_bits: FloatBits,
        max_motor_force_bits: FloatBits,
        limit_enabled: bool,
        motor_enabled: bool,
    },
    Distance {
        local_anchor_a: Vec2Bits,
        local_anchor_b: Vec2Bits,
        length_bits: FloatBits,
        frequency_bits: FloatBits,
        damping_ratio_bits: FloatBits,
    },
    Pulley {
        ground_anchor_a: Vec2Bits,
        ground_anchor_b: Vec2Bits,
        local_anchor_a: Vec2Bits,
        local_anchor_b: Vec2Bits,
        length_a_bits: FloatBits,
        length_b_bits: FloatBits,
        ratio_bits: FloatBits,
    },
    Mouse {
        target: Vec2Bits,
        max_force_bits: FloatBits,
        frequency_bits: FloatBits,
        damping_ratio_bits: FloatBits,
    },
    Gear {
        joint_a_id: ScenarioId,
        joint_b_id: ScenarioId,
        ratio_bits: FloatBits,
    },
    Wheel {
        local_anchor_a: Vec2Bits,
        local_anchor_b: Vec2Bits,
        local_axis_a: Vec2Bits,
        motor_speed_bits: FloatBits,
        max_motor_torque_bits: FloatBits,
        frequency_bits: FloatBits,
        damping_ratio_bits: FloatBits,
        motor_enabled: bool,
    },
    Weld {
        local_anchor_a: Vec2Bits,
        local_anchor_b: Vec2Bits,
        reference_angle_bits: FloatBits,
        frequency_bits: FloatBits,
        damping_ratio_bits: FloatBits,
    },
    Friction {
        local_anchor_a: Vec2Bits,
        local_anchor_b: Vec2Bits,
        max_force_bits: FloatBits,
        max_torque_bits: FloatBits,
    },
    Rope {
        local_anchor_a: Vec2Bits,
        local_anchor_b: Vec2Bits,
        max_length_bits: FloatBits,
    },
    Motor {
        linear_offset: Vec2Bits,
        angular_offset_bits: FloatBits,
        max_force_bits: FloatBits,
        max_torque_bits: FloatBits,
        correction_factor_bits: FloatBits,
    },
}

impl RigidJointDefinition {
    #[must_use]
    pub const fn joint_kind(&self) -> RigidJointKind {
        match self {
            Self::Revolute { .. } => RigidJointKind::Revolute,
            Self::Prismatic { .. } => RigidJointKind::Prismatic,
            Self::Distance { .. } => RigidJointKind::Distance,
            Self::Pulley { .. } => RigidJointKind::Pulley,
            Self::Mouse { .. } => RigidJointKind::Mouse,
            Self::Gear { .. } => RigidJointKind::Gear,
            Self::Wheel { .. } => RigidJointKind::Wheel,
            Self::Weld { .. } => RigidJointKind::Weld,
            Self::Friction { .. } => RigidJointKind::Friction,
            Self::Rope { .. } => RigidJointKind::Rope,
            Self::Motor { .. } => RigidJointKind::Motor,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidJointDeclaration {
    pub joint_id: ScenarioId,
    pub body_a_id: ScenarioId,
    pub body_b_id: ScenarioId,
    pub collide_connected: bool,
    pub definition: RigidJointDefinition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidRopeDeclaration {
    pub rope_id: ScenarioId,
    pub vertices: Box<[Vec2Bits]>,
    pub masses_bits: Box<[FloatBits]>,
    pub gravity: Vec2Bits,
    pub damping_bits: FloatBits,
    pub stretch_stiffness_bits: FloatBits,
    pub bend_stiffness_bits: FloatBits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RigidJointMutation {
    LimitEnabled {
        enabled: bool,
    },
    Limits {
        lower_bits: FloatBits,
        upper_bits: FloatBits,
    },
    MotorEnabled {
        enabled: bool,
    },
    MotorSpeed {
        speed_bits: FloatBits,
    },
    MaxMotorForce {
        force_bits: FloatBits,
    },
    MaxMotorTorque {
        torque_bits: FloatBits,
    },
    Length {
        length_bits: FloatBits,
    },
    Frequency {
        frequency_bits: FloatBits,
    },
    DampingRatio {
        damping_ratio_bits: FloatBits,
    },
    MouseTarget {
        target: Vec2Bits,
    },
    MaxForce {
        force_bits: FloatBits,
    },
    MaxTorque {
        torque_bits: FloatBits,
    },
    GearRatio {
        ratio_bits: FloatBits,
    },
    RopeMaxLength {
        max_length_bits: FloatBits,
    },
    LinearOffset {
        offset: Vec2Bits,
    },
    AngularOffset {
        offset_bits: FloatBits,
    },
    CorrectionFactor {
        factor_bits: FloatBits,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidContactDirectiveTarget {
    pub fixture_a_id: ScenarioId,
    pub fixture_b_id: ScenarioId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigidPreSolveDirective {
    pub enabled: bool,
    pub maybe_friction_bits: Option<FloatBits>,
    pub maybe_restitution_bits: Option<FloatBits>,
    pub maybe_tangent_speed_bits: Option<FloatBits>,
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
    CreateJoint {
        joint_id: ScenarioId,
    },
    InspectJoint {
        joint_id: ScenarioId,
    },
    MutateJoint {
        joint_id: ScenarioId,
        mutation: RigidJointMutation,
    },
    DestroyJoint {
        joint_id: ScenarioId,
    },
    CreateRope {
        rope_id: ScenarioId,
    },
    SetRopeAngle {
        rope_id: ScenarioId,
        angle_bits: FloatBits,
    },
    StepRope {
        rope_id: ScenarioId,
        timestep_bits: FloatBits,
        iterations: u32,
    },
    InspectRope {
        rope_id: ScenarioId,
    },
    DestroyRope {
        rope_id: ScenarioId,
    },
    SetContactFilterDirective {
        target: RigidContactDirectiveTarget,
        should_collide: bool,
    },
    SetPreSolveDirective {
        target: RigidContactDirectiveTarget,
        directive: RigidPreSolveDirective,
    },
    RequestReconstruction,
    RequestDiagnostics,
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
    CreateJoint,
    InspectJoint,
    MutateJoint,
    DestroyJoint,
    CreateRope,
    SetRopeAngle,
    StepRope,
    InspectRope,
    DestroyRope,
    SetContactFilterDirective,
    SetPreSolveDirective,
    RequestReconstruction,
    RequestDiagnostics,
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
            Self::CreateJoint { .. } => RigidWorldActionKind::CreateJoint,
            Self::InspectJoint { .. } => RigidWorldActionKind::InspectJoint,
            Self::MutateJoint { .. } => RigidWorldActionKind::MutateJoint,
            Self::DestroyJoint { .. } => RigidWorldActionKind::DestroyJoint,
            Self::CreateRope { .. } => RigidWorldActionKind::CreateRope,
            Self::SetRopeAngle { .. } => RigidWorldActionKind::SetRopeAngle,
            Self::StepRope { .. } => RigidWorldActionKind::StepRope,
            Self::InspectRope { .. } => RigidWorldActionKind::InspectRope,
            Self::DestroyRope { .. } => RigidWorldActionKind::DestroyRope,
            Self::SetContactFilterDirective { .. } => {
                RigidWorldActionKind::SetContactFilterDirective
            }
            Self::SetPreSolveDirective { .. } => RigidWorldActionKind::SetPreSolveDirective,
            Self::RequestReconstruction => RigidWorldActionKind::RequestReconstruction,
            Self::RequestDiagnostics => RigidWorldActionKind::RequestDiagnostics,
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
    #[serde(default, skip_serializing_if = "joints_are_empty")]
    pub(super) joints: Box<[RigidJointDeclaration]>,
    #[serde(default, skip_serializing_if = "ropes_are_empty")]
    pub(super) ropes: Box<[RigidRopeDeclaration]>,
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
    pub fn joints(&self) -> &[RigidJointDeclaration] {
        &self.joints
    }

    #[must_use]
    pub fn ropes(&self) -> &[RigidRopeDeclaration] {
        &self.ropes
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

pub(super) fn apply_lifecycle_action(
    action: &RigidWorldAction,
    fixture_owners: &HashMap<ScenarioId, ScenarioId>,
    live_bodies: &mut HashSet<ScenarioId>,
    live_fixtures: &mut HashSet<ScenarioId>,
) {
    match action {
        RigidWorldAction::CreateBody { body_id } => {
            live_bodies.insert(body_id.clone());
        }
        RigidWorldAction::CreateFixture { fixture_id } => {
            live_fixtures.insert(fixture_id.clone());
        }
        RigidWorldAction::DestroyFixture { fixture_id } => {
            live_fixtures.remove(fixture_id);
        }
        RigidWorldAction::DestroyBody { body_id } => {
            live_bodies.remove(body_id);
            live_fixtures.retain(|fixture_id| fixture_owners.get(fixture_id) != Some(body_id));
        }
        _ => {}
    }
}

fn joints_are_empty(joints: &[RigidJointDeclaration]) -> bool {
    joints.is_empty()
}

fn ropes_are_empty(ropes: &[RigidRopeDeclaration]) -> bool {
    ropes.is_empty()
}
