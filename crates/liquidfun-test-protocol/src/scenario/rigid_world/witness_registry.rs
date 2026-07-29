use serde::{Deserialize, Serialize};

use super::types::RigidWorldActionKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidWorldWitnessFamily {
    NonCollidingBodyFixtureLifecycle,
    SingleContactLifecycle,
    BodyControlAndForcePolicy,
    MultiContactIslandAndWarmStart,
    SleepingAndWaking,
    ContinuousCollisionAndSubStepping,
    ContinuousBudgetResume,
    WorldQueryAndRayCast,
    OriginShiftCovariance,
    JointDefinitionsAndMutations,
    RevolutePrismaticLimitsAndMotors,
    DistancePulleyMouseConstraints,
    WheelWeldFrictionRopeMotorConstraints,
    GearDependenciesAndFourBodySolver,
    MixedJointIslandOrderAndCollisionSuppression,
    StandaloneRopeEvolution,
    ContactFilterListenerAndPreSolveTiming,
    DestructionListenerAndDependencyCascades,
    DiagnosticReconstructionAndDumpOrder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidWorldWitness {
    StaticBodyCreated,
    KinematicBodyCreated,
    DynamicBodyCreated,
    FixturesCreated,
    BodyInspected,
    FixtureInspected,
    BodyTransformChanged,
    BodyTypeChanged,
    BodyDeactivated,
    BodyReactivated,
    SensorEnabled,
    SensorDisabled,
    MaterialChanged,
    FilterChanged,
    DensityChangedWithoutMassReset,
    MassReset,
    CustomMassSet,
    StaticKinematicOverlapRejected,
    KinematicKinematicOverlapRejected,
    ZeroContactStep,
    FixtureDestroyed,
    BodyDestroyed,
    ContactCreated,
    ContactBegin,
    ContactPersisted,
    ManifoldActive,
    ContactSolved,
    WarmStartTransferred,
    SensorTouching,
    SensorWithoutManifold,
    FilterRemovedContact,
    FilterRecreatedContact,
    DeactivationDestroyedContact,
    ReactivationRecreatedContact,
    FixtureDestroyedContact,
    BodyCascadeEndOrdered,
    ForceWakePolicy,
    ForcePreserveSleepPolicy,
    ImpulseWakePolicy,
    VelocityWakePolicy,
    DampingAndGravityScaleApplied,
    FixedRotationApplied,
    AutomaticForceClearingApplied,
    ManualForceClearingApplied,
    MultiContactIslandSolved,
    IslandTraversalOrdered,
    WarmStartApplied,
    WarmStartDisabledThenStored,
    SleepingThresholdReached,
    WholeIslandSlept,
    MutationWokeBody,
    ContactWokeIsland,
    ActivationPreservedSleep,
    ContinuousPhysicsPreventedTunneling,
    DisabledContinuousPhysicsTunneled,
    BulletStateSelectedContinuousContact,
    ContinuousStepCompleted,
    SubStepReportedPending,
    SubStepResumeCompleted,
    ContinuousTransitionsOrdered,
    ContinuousBudgetExhausted,
    ContinuousBudgetStateCoherent,
    ContinuousBudgetResumeCompleted,
    QueryPreservedDuplicateOccurrences,
    QueryExhausted,
    QueryTerminated,
    QueryExplicitFilterApplied,
    RayMissed,
    RayRejectedInvalidDirective,
    RayIgnoredHit,
    RayContinuedWithoutClipping,
    RayClipped,
    RayTerminated,
    RayNearestHitSelected,
    RayEqualFractionTieSet,
    OriginShiftRejectedWhileLocked,
    OriginShiftRejectedNonFinite,
    OriginShiftRejectedOverflow,
    OriginShiftTranslatedBodies,
    OriginShiftPreservedQueryHits,
    OriginShiftPreservedRayFractionsAndNormals,
    OriginShiftPreservedTopology,
    AllJointKindsCreated,
    JointMutationApplied,
    JointPositiveStepCompleted,
    JointWarmStartStepCompleted,
    AllJointKindsInspected,
    RevoluteLimitsAndMotorStepped,
    PrismaticLimitsAndMotorStepped,
    JointLowerLimitObserved,
    JointUpperLimitObserved,
    JointEqualLimitObserved,
    RevolutePrismaticReactionObserved,
    RigidDistanceStepped,
    SoftDistanceStepped,
    PulleyRatioStepped,
    MouseTargetAndForceStepped,
    DistancePulleyMouseReactionObserved,
    WheelSpringAndMotorStepped,
    RigidWeldStepped,
    SoftWeldStepped,
    FrictionCapsStepped,
    RopeJointInactiveStepped,
    RopeJointUpperLimitStepped,
    MotorCorrectionStepped,
    CoupledJointReactionObserved,
    GearRevoluteRevoluteStepped,
    GearRevolutePrismaticStepped,
    GearPrismaticRevoluteStepped,
    GearPrismaticPrismaticStepped,
    GearSignedRatiosObserved,
    GearFourBodyTopologyObserved,
    GearDependencyOrderObserved,
    MixedJointContactOrderObserved,
    JointCollisionSuppressed,
    JointCollisionRestored,
    MixedJointWarmStartObserved,
    MixedJointReactionObserved,
    StandaloneRopePositiveStep,
    StandaloneRopeAngleObserved,
    StandaloneRopeVerticesObserved,
    FilterRejectionObserved,
    FilterAdmissionObserved,
    CallbackLifecycleOrderObserved,
    PreSolveDisableObserved,
    PreSolveMaterialObserved,
    CallbackRepeatedOccurrenceObserved,
    TouchingContactEstablished,
    ExplicitJointNoGoodbye,
    ExplicitFixtureNoGoodbye,
    DependentGearDestroyedFirst,
    DestructionLifecycleOrderObserved,
    ReconstructionDependencyOrderObserved,
    UnsupportedMouseReconstructionObserved,
    DiagnosticsCountsObserved,
}

const JOINT_DEFINITION_WITNESSES: [RigidWorldWitness; 5] = [
    RigidWorldWitness::AllJointKindsCreated,
    RigidWorldWitness::JointMutationApplied,
    RigidWorldWitness::JointPositiveStepCompleted,
    RigidWorldWitness::JointWarmStartStepCompleted,
    RigidWorldWitness::AllJointKindsInspected,
];
const REVOLUTE_PRISMATIC_WITNESSES: [RigidWorldWitness; 6] = [
    RigidWorldWitness::RevoluteLimitsAndMotorStepped,
    RigidWorldWitness::PrismaticLimitsAndMotorStepped,
    RigidWorldWitness::JointLowerLimitObserved,
    RigidWorldWitness::JointUpperLimitObserved,
    RigidWorldWitness::JointEqualLimitObserved,
    RigidWorldWitness::RevolutePrismaticReactionObserved,
];
const DISTANCE_PULLEY_MOUSE_WITNESSES: [RigidWorldWitness; 5] = [
    RigidWorldWitness::RigidDistanceStepped,
    RigidWorldWitness::SoftDistanceStepped,
    RigidWorldWitness::PulleyRatioStepped,
    RigidWorldWitness::MouseTargetAndForceStepped,
    RigidWorldWitness::DistancePulleyMouseReactionObserved,
];
const COUPLED_JOINT_WITNESSES: [RigidWorldWitness; 8] = [
    RigidWorldWitness::WheelSpringAndMotorStepped,
    RigidWorldWitness::RigidWeldStepped,
    RigidWorldWitness::SoftWeldStepped,
    RigidWorldWitness::FrictionCapsStepped,
    RigidWorldWitness::RopeJointInactiveStepped,
    RigidWorldWitness::RopeJointUpperLimitStepped,
    RigidWorldWitness::MotorCorrectionStepped,
    RigidWorldWitness::CoupledJointReactionObserved,
];
const GEAR_WITNESSES: [RigidWorldWitness; 7] = [
    RigidWorldWitness::GearRevoluteRevoluteStepped,
    RigidWorldWitness::GearRevolutePrismaticStepped,
    RigidWorldWitness::GearPrismaticRevoluteStepped,
    RigidWorldWitness::GearPrismaticPrismaticStepped,
    RigidWorldWitness::GearSignedRatiosObserved,
    RigidWorldWitness::GearFourBodyTopologyObserved,
    RigidWorldWitness::GearDependencyOrderObserved,
];
const MIXED_JOINT_WITNESSES: [RigidWorldWitness; 5] = [
    RigidWorldWitness::MixedJointContactOrderObserved,
    RigidWorldWitness::JointCollisionSuppressed,
    RigidWorldWitness::JointCollisionRestored,
    RigidWorldWitness::MixedJointWarmStartObserved,
    RigidWorldWitness::MixedJointReactionObserved,
];
const STANDALONE_ROPE_WITNESSES: [RigidWorldWitness; 3] = [
    RigidWorldWitness::StandaloneRopePositiveStep,
    RigidWorldWitness::StandaloneRopeAngleObserved,
    RigidWorldWitness::StandaloneRopeVerticesObserved,
];
const CALLBACK_TIMING_WITNESSES: [RigidWorldWitness; 6] = [
    RigidWorldWitness::FilterRejectionObserved,
    RigidWorldWitness::FilterAdmissionObserved,
    RigidWorldWitness::CallbackLifecycleOrderObserved,
    RigidWorldWitness::PreSolveDisableObserved,
    RigidWorldWitness::PreSolveMaterialObserved,
    RigidWorldWitness::CallbackRepeatedOccurrenceObserved,
];
const DESTRUCTION_TIMING_WITNESSES: [RigidWorldWitness; 5] = [
    RigidWorldWitness::TouchingContactEstablished,
    RigidWorldWitness::ExplicitJointNoGoodbye,
    RigidWorldWitness::ExplicitFixtureNoGoodbye,
    RigidWorldWitness::DependentGearDestroyedFirst,
    RigidWorldWitness::DestructionLifecycleOrderObserved,
];
const DIAGNOSTIC_WITNESSES: [RigidWorldWitness; 3] = [
    RigidWorldWitness::ReconstructionDependencyOrderObserved,
    RigidWorldWitness::UnsupportedMouseReconstructionObserved,
    RigidWorldWitness::DiagnosticsCountsObserved,
];

const NON_COLLIDING_WITNESSES: [RigidWorldWitness; 22] = [
    RigidWorldWitness::StaticBodyCreated,
    RigidWorldWitness::KinematicBodyCreated,
    RigidWorldWitness::DynamicBodyCreated,
    RigidWorldWitness::FixturesCreated,
    RigidWorldWitness::BodyInspected,
    RigidWorldWitness::FixtureInspected,
    RigidWorldWitness::BodyTransformChanged,
    RigidWorldWitness::BodyTypeChanged,
    RigidWorldWitness::BodyDeactivated,
    RigidWorldWitness::BodyReactivated,
    RigidWorldWitness::SensorEnabled,
    RigidWorldWitness::SensorDisabled,
    RigidWorldWitness::MaterialChanged,
    RigidWorldWitness::FilterChanged,
    RigidWorldWitness::DensityChangedWithoutMassReset,
    RigidWorldWitness::MassReset,
    RigidWorldWitness::CustomMassSet,
    RigidWorldWitness::StaticKinematicOverlapRejected,
    RigidWorldWitness::KinematicKinematicOverlapRejected,
    RigidWorldWitness::ZeroContactStep,
    RigidWorldWitness::FixtureDestroyed,
    RigidWorldWitness::BodyDestroyed,
];

const SINGLE_CONTACT_WITNESSES: [RigidWorldWitness; 14] = [
    RigidWorldWitness::ContactCreated,
    RigidWorldWitness::ContactBegin,
    RigidWorldWitness::ContactPersisted,
    RigidWorldWitness::ManifoldActive,
    RigidWorldWitness::ContactSolved,
    RigidWorldWitness::WarmStartTransferred,
    RigidWorldWitness::SensorTouching,
    RigidWorldWitness::SensorWithoutManifold,
    RigidWorldWitness::FilterRemovedContact,
    RigidWorldWitness::FilterRecreatedContact,
    RigidWorldWitness::DeactivationDestroyedContact,
    RigidWorldWitness::ReactivationRecreatedContact,
    RigidWorldWitness::FixtureDestroyedContact,
    RigidWorldWitness::BodyCascadeEndOrdered,
];

const BODY_CONTROL_WITNESSES: [RigidWorldWitness; 8] = [
    RigidWorldWitness::ForceWakePolicy,
    RigidWorldWitness::ForcePreserveSleepPolicy,
    RigidWorldWitness::ImpulseWakePolicy,
    RigidWorldWitness::VelocityWakePolicy,
    RigidWorldWitness::DampingAndGravityScaleApplied,
    RigidWorldWitness::FixedRotationApplied,
    RigidWorldWitness::AutomaticForceClearingApplied,
    RigidWorldWitness::ManualForceClearingApplied,
];

const ISLAND_WITNESSES: [RigidWorldWitness; 4] = [
    RigidWorldWitness::MultiContactIslandSolved,
    RigidWorldWitness::IslandTraversalOrdered,
    RigidWorldWitness::WarmStartApplied,
    RigidWorldWitness::WarmStartDisabledThenStored,
];

const SLEEP_WITNESSES: [RigidWorldWitness; 5] = [
    RigidWorldWitness::SleepingThresholdReached,
    RigidWorldWitness::WholeIslandSlept,
    RigidWorldWitness::MutationWokeBody,
    RigidWorldWitness::ContactWokeIsland,
    RigidWorldWitness::ActivationPreservedSleep,
];

const CCD_WITNESSES: [RigidWorldWitness; 7] = [
    RigidWorldWitness::ContinuousPhysicsPreventedTunneling,
    RigidWorldWitness::DisabledContinuousPhysicsTunneled,
    RigidWorldWitness::BulletStateSelectedContinuousContact,
    RigidWorldWitness::ContinuousStepCompleted,
    RigidWorldWitness::SubStepReportedPending,
    RigidWorldWitness::SubStepResumeCompleted,
    RigidWorldWitness::ContinuousTransitionsOrdered,
];

const BUDGET_WITNESSES: [RigidWorldWitness; 3] = [
    RigidWorldWitness::ContinuousBudgetExhausted,
    RigidWorldWitness::ContinuousBudgetStateCoherent,
    RigidWorldWitness::ContinuousBudgetResumeCompleted,
];

const QUERY_RAY_WITNESSES: [RigidWorldWitness; 12] = [
    RigidWorldWitness::QueryPreservedDuplicateOccurrences,
    RigidWorldWitness::QueryExhausted,
    RigidWorldWitness::QueryTerminated,
    RigidWorldWitness::QueryExplicitFilterApplied,
    RigidWorldWitness::RayMissed,
    RigidWorldWitness::RayRejectedInvalidDirective,
    RigidWorldWitness::RayIgnoredHit,
    RigidWorldWitness::RayContinuedWithoutClipping,
    RigidWorldWitness::RayClipped,
    RigidWorldWitness::RayTerminated,
    RigidWorldWitness::RayNearestHitSelected,
    RigidWorldWitness::RayEqualFractionTieSet,
];

const ORIGIN_SHIFT_WITNESSES: [RigidWorldWitness; 7] = [
    RigidWorldWitness::OriginShiftRejectedWhileLocked,
    RigidWorldWitness::OriginShiftRejectedNonFinite,
    RigidWorldWitness::OriginShiftRejectedOverflow,
    RigidWorldWitness::OriginShiftTranslatedBodies,
    RigidWorldWitness::OriginShiftPreservedQueryHits,
    RigidWorldWitness::OriginShiftPreservedRayFractionsAndNormals,
    RigidWorldWitness::OriginShiftPreservedTopology,
];

const NON_COLLIDING_ACTIONS: [RigidWorldActionKind; 16] = [
    RigidWorldActionKind::CreateBody,
    RigidWorldActionKind::CreateFixture,
    RigidWorldActionKind::InspectBody,
    RigidWorldActionKind::InspectFixture,
    RigidWorldActionKind::SetBodyTransform,
    RigidWorldActionKind::SetBodyType,
    RigidWorldActionKind::SetBodyActive,
    RigidWorldActionKind::SetFixtureSensor,
    RigidWorldActionKind::SetFixtureMaterial,
    RigidWorldActionKind::SetFixtureFilter,
    RigidWorldActionKind::SetFixtureDensity,
    RigidWorldActionKind::ResetMassData,
    RigidWorldActionKind::SetCustomMassData,
    RigidWorldActionKind::Step,
    RigidWorldActionKind::DestroyFixture,
    RigidWorldActionKind::DestroyBody,
];

const SINGLE_CONTACT_ACTIONS: [RigidWorldActionKind; 8] = [
    RigidWorldActionKind::CreateBody,
    RigidWorldActionKind::CreateFixture,
    RigidWorldActionKind::SetBodyActive,
    RigidWorldActionKind::SetFixtureSensor,
    RigidWorldActionKind::SetFixtureFilter,
    RigidWorldActionKind::Step,
    RigidWorldActionKind::DestroyFixture,
    RigidWorldActionKind::DestroyBody,
];

const BODY_CONTROL_ACTIONS: [RigidWorldActionKind; 18] = [
    RigidWorldActionKind::CreateBody,
    RigidWorldActionKind::CreateFixture,
    RigidWorldActionKind::SetLinearVelocity,
    RigidWorldActionKind::SetAngularVelocity,
    RigidWorldActionKind::ApplyForce,
    RigidWorldActionKind::ApplyTorque,
    RigidWorldActionKind::ApplyLinearImpulse,
    RigidWorldActionKind::ApplyAngularImpulse,
    RigidWorldActionKind::SetBodyDamping,
    RigidWorldActionKind::SetGravityScale,
    RigidWorldActionKind::SetFixedRotation,
    RigidWorldActionKind::SetSleepingAllowed,
    RigidWorldActionKind::SetAwake,
    RigidWorldActionKind::SetWorldGravity,
    RigidWorldActionKind::SetAutomaticForceClearing,
    RigidWorldActionKind::ClearForces,
    RigidWorldActionKind::ConfiguredStep,
    RigidWorldActionKind::DestroyBody,
];

const ISLAND_ACTIONS: [RigidWorldActionKind; 5] = [
    RigidWorldActionKind::CreateBody,
    RigidWorldActionKind::CreateFixture,
    RigidWorldActionKind::SetWarmStarting,
    RigidWorldActionKind::ConfiguredStep,
    RigidWorldActionKind::DestroyBody,
];

const SLEEP_ACTIONS: [RigidWorldActionKind; 7] = [
    RigidWorldActionKind::CreateBody,
    RigidWorldActionKind::CreateFixture,
    RigidWorldActionKind::SetBodyActive,
    RigidWorldActionKind::SetLinearVelocity,
    RigidWorldActionKind::ApplyForce,
    RigidWorldActionKind::SetSleepingAllowed,
    RigidWorldActionKind::ConfiguredStep,
];

const CCD_ACTIONS: [RigidWorldActionKind; 7] = [
    RigidWorldActionKind::CreateBody,
    RigidWorldActionKind::CreateFixture,
    RigidWorldActionKind::SetBullet,
    RigidWorldActionKind::SetContinuousPhysics,
    RigidWorldActionKind::SetSubStepping,
    RigidWorldActionKind::SetLinearVelocity,
    RigidWorldActionKind::ConfiguredStep,
];

const BUDGET_ACTIONS: [RigidWorldActionKind; 4] = [
    RigidWorldActionKind::CreateBody,
    RigidWorldActionKind::CreateFixture,
    RigidWorldActionKind::SetBullet,
    RigidWorldActionKind::ConfiguredStep,
];

const QUERY_RAY_ACTIONS: [RigidWorldActionKind; 4] = [
    RigidWorldActionKind::CreateBody,
    RigidWorldActionKind::CreateFixture,
    RigidWorldActionKind::QueryAabb,
    RigidWorldActionKind::RayCast,
];

const ORIGIN_SHIFT_ACTIONS: [RigidWorldActionKind; 5] = [
    RigidWorldActionKind::CreateBody,
    RigidWorldActionKind::CreateFixture,
    RigidWorldActionKind::QueryAabb,
    RigidWorldActionKind::RayCast,
    RigidWorldActionKind::ShiftOrigin,
];

const JOINT_DEFINITION_ACTIONS: [RigidWorldActionKind; 6] = [
    RigidWorldActionKind::CreateJoint,
    RigidWorldActionKind::MutateJoint,
    RigidWorldActionKind::Step,
    RigidWorldActionKind::InspectJoint,
    RigidWorldActionKind::DestroyJoint,
    RigidWorldActionKind::DestroyBody,
];
const JOINT_EXECUTION_ACTIONS: [RigidWorldActionKind; 5] = [
    RigidWorldActionKind::CreateJoint,
    RigidWorldActionKind::Step,
    RigidWorldActionKind::InspectJoint,
    RigidWorldActionKind::DestroyJoint,
    RigidWorldActionKind::DestroyBody,
];
const ROPE_ACTIONS: [RigidWorldActionKind; 5] = [
    RigidWorldActionKind::CreateRope,
    RigidWorldActionKind::SetRopeAngle,
    RigidWorldActionKind::StepRope,
    RigidWorldActionKind::InspectRope,
    RigidWorldActionKind::DestroyRope,
];
const CALLBACK_ACTIONS: [RigidWorldActionKind; 5] = [
    RigidWorldActionKind::SetContactFilterDirective,
    RigidWorldActionKind::SetPreSolveDirective,
    RigidWorldActionKind::Step,
    RigidWorldActionKind::InspectBody,
    RigidWorldActionKind::DestroyBody,
];
const DESTRUCTION_ACTIONS: [RigidWorldActionKind; 6] = [
    RigidWorldActionKind::CreateJoint,
    RigidWorldActionKind::Step,
    RigidWorldActionKind::InspectBody,
    RigidWorldActionKind::DestroyJoint,
    RigidWorldActionKind::DestroyFixture,
    RigidWorldActionKind::DestroyBody,
];
const DIAGNOSTIC_ACTIONS: [RigidWorldActionKind; 2] = [
    RigidWorldActionKind::RequestReconstruction,
    RigidWorldActionKind::RequestDiagnostics,
];

impl RigidWorldWitnessFamily {
    pub const REQUIRED: [Self; 2] = [
        Self::NonCollidingBodyFixtureLifecycle,
        Self::SingleContactLifecycle,
    ];

    /// Complete closed registry used by generated schema and Phase 7 evidence.
    pub const ALL: [Self; 19] = [
        Self::NonCollidingBodyFixtureLifecycle,
        Self::SingleContactLifecycle,
        Self::BodyControlAndForcePolicy,
        Self::MultiContactIslandAndWarmStart,
        Self::SleepingAndWaking,
        Self::ContinuousCollisionAndSubStepping,
        Self::ContinuousBudgetResume,
        Self::WorldQueryAndRayCast,
        Self::OriginShiftCovariance,
        Self::JointDefinitionsAndMutations,
        Self::RevolutePrismaticLimitsAndMotors,
        Self::DistancePulleyMouseConstraints,
        Self::WheelWeldFrictionRopeMotorConstraints,
        Self::GearDependenciesAndFourBodySolver,
        Self::MixedJointIslandOrderAndCollisionSuppression,
        Self::StandaloneRopeEvolution,
        Self::ContactFilterListenerAndPreSolveTiming,
        Self::DestructionListenerAndDependencyCascades,
        Self::DiagnosticReconstructionAndDumpOrder,
    ];

    /// Phase 7 witness families without the retained Phase 6 compatibility corpus.
    pub const PHASE7_REQUIRED: [Self; 7] = [
        Self::BodyControlAndForcePolicy,
        Self::MultiContactIslandAndWarmStart,
        Self::SleepingAndWaking,
        Self::ContinuousCollisionAndSubStepping,
        Self::ContinuousBudgetResume,
        Self::WorldQueryAndRayCast,
        Self::OriginShiftCovariance,
    ];

    /// New Phase 8 witness families without the retained Phase 6/7 corpus.
    pub const PHASE8_REQUIRED: [Self; 10] = [
        Self::JointDefinitionsAndMutations,
        Self::RevolutePrismaticLimitsAndMotors,
        Self::DistancePulleyMouseConstraints,
        Self::WheelWeldFrictionRopeMotorConstraints,
        Self::GearDependenciesAndFourBodySolver,
        Self::MixedJointIslandOrderAndCollisionSuppression,
        Self::StandaloneRopeEvolution,
        Self::ContactFilterListenerAndPreSolveTiming,
        Self::DestructionListenerAndDependencyCascades,
        Self::DiagnosticReconstructionAndDumpOrder,
    ];

    #[must_use]
    pub const fn required_witnesses(self) -> &'static [RigidWorldWitness] {
        match self {
            Self::NonCollidingBodyFixtureLifecycle => &NON_COLLIDING_WITNESSES,
            Self::SingleContactLifecycle => &SINGLE_CONTACT_WITNESSES,
            Self::BodyControlAndForcePolicy => &BODY_CONTROL_WITNESSES,
            Self::MultiContactIslandAndWarmStart => &ISLAND_WITNESSES,
            Self::SleepingAndWaking => &SLEEP_WITNESSES,
            Self::ContinuousCollisionAndSubStepping => &CCD_WITNESSES,
            Self::ContinuousBudgetResume => &BUDGET_WITNESSES,
            Self::WorldQueryAndRayCast => &QUERY_RAY_WITNESSES,
            Self::OriginShiftCovariance => &ORIGIN_SHIFT_WITNESSES,
            Self::JointDefinitionsAndMutations => &JOINT_DEFINITION_WITNESSES,
            Self::RevolutePrismaticLimitsAndMotors => &REVOLUTE_PRISMATIC_WITNESSES,
            Self::DistancePulleyMouseConstraints => &DISTANCE_PULLEY_MOUSE_WITNESSES,
            Self::WheelWeldFrictionRopeMotorConstraints => &COUPLED_JOINT_WITNESSES,
            Self::GearDependenciesAndFourBodySolver => &GEAR_WITNESSES,
            Self::MixedJointIslandOrderAndCollisionSuppression => &MIXED_JOINT_WITNESSES,
            Self::StandaloneRopeEvolution => &STANDALONE_ROPE_WITNESSES,
            Self::ContactFilterListenerAndPreSolveTiming => &CALLBACK_TIMING_WITNESSES,
            Self::DestructionListenerAndDependencyCascades => &DESTRUCTION_TIMING_WITNESSES,
            Self::DiagnosticReconstructionAndDumpOrder => &DIAGNOSTIC_WITNESSES,
        }
    }

    pub(super) const fn required_action_kinds(self) -> &'static [RigidWorldActionKind] {
        match self {
            Self::NonCollidingBodyFixtureLifecycle => &NON_COLLIDING_ACTIONS,
            Self::SingleContactLifecycle => &SINGLE_CONTACT_ACTIONS,
            Self::BodyControlAndForcePolicy => &BODY_CONTROL_ACTIONS,
            Self::MultiContactIslandAndWarmStart => &ISLAND_ACTIONS,
            Self::SleepingAndWaking => &SLEEP_ACTIONS,
            Self::ContinuousCollisionAndSubStepping => &CCD_ACTIONS,
            Self::ContinuousBudgetResume => &BUDGET_ACTIONS,
            Self::WorldQueryAndRayCast => &QUERY_RAY_ACTIONS,
            Self::OriginShiftCovariance => &ORIGIN_SHIFT_ACTIONS,
            Self::JointDefinitionsAndMutations => &JOINT_DEFINITION_ACTIONS,
            Self::RevolutePrismaticLimitsAndMotors
            | Self::DistancePulleyMouseConstraints
            | Self::WheelWeldFrictionRopeMotorConstraints
            | Self::GearDependenciesAndFourBodySolver
            | Self::MixedJointIslandOrderAndCollisionSuppression => &JOINT_EXECUTION_ACTIONS,
            Self::StandaloneRopeEvolution => &ROPE_ACTIONS,
            Self::ContactFilterListenerAndPreSolveTiming => &CALLBACK_ACTIONS,
            Self::DestructionListenerAndDependencyCascades => &DESTRUCTION_ACTIONS,
            Self::DiagnosticReconstructionAndDumpOrder => &DIAGNOSTIC_ACTIONS,
        }
    }
}

mod identity;

#[cfg(test)]
mod tests;
