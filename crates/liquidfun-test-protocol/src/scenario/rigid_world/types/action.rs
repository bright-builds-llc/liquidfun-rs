use super::{
    Deserialize, FloatBits, Phase9ParticleAction, Phase10Operation, RigidAabbBits, RigidBodyKind,
    RigidContactDirectiveTarget, RigidFilterBits, RigidJointMutation, RigidPreSolveDirective,
    RigidQueryDirectiveRule, RigidRayDirectiveRule, RigidWakePolicy, ScenarioId, Serialize,
    TransformBits, Vec2Bits,
};

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
    Particle {
        action: Phase9ParticleAction,
    },
    ParticleGroup {
        operation: Phase10Operation,
    },
    DestroyFixture {
        fixture_id: ScenarioId,
    },
    DestroyBody {
        body_id: ScenarioId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(in crate::scenario::rigid_world) enum RigidWorldActionKind {
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
    Particle,
    ParticleGroup,
    DestroyFixture,
    DestroyBody,
}

impl RigidWorldAction {
    pub(in crate::scenario::rigid_world) const fn action_kind(&self) -> RigidWorldActionKind {
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
            Self::Particle { .. } => RigidWorldActionKind::Particle,
            Self::ParticleGroup { .. } => RigidWorldActionKind::ParticleGroup,
            Self::DestroyFixture { .. } => RigidWorldActionKind::DestroyFixture,
            Self::DestroyBody { .. } => RigidWorldActionKind::DestroyBody,
        }
    }
}
