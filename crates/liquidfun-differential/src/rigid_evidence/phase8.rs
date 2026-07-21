//! Pure Phase 8 engine-to-protocol observation mapping.

use liquidfun::{JointId, JointKind, JointLimitState, JointSnapshot, JointSpecificSnapshot};
use liquidfun_test_protocol::{
    FieldComparison, FieldPolicy, FloatBits, Phase6PolicyProfile, Phase8PolicyProfile,
    RigidJointBranchState, RigidJointKind, RigidJointSnapshot, RigidWorldCheckpointResult,
    RigidWorldObservation, RigidWorldRequestRecord, RigidWorldResultRecord, ScenarioId,
};

use crate::float_values_match_with_policy;
use crate::rigid_world::{NativeRigidWorldError, TimelineExecutor};

use super::{
    EvidenceContext, Location, RigidComparisonFailure, RigidHarnessReport, RigidMismatchKind,
    RigidMismatchReport, compare_checkpoint_inherited, mismatch_with_context,
};

macro_rules! phase8_exact {
    ($request:expr, $profile:expr, $context:expr, $path:expr, $expected:expr, $actual:expr) => {
        if let Some(report) = exact(
            $request,
            $profile,
            $context,
            $path,
            RigidMismatchKind::Exact,
            &$expected,
            &$actual,
        )? {
            return Ok(Some(report));
        }
    };
}

macro_rules! phase8_float {
    ($request:expr, $profile:expr, $context:expr, $path:expr, $expected:expr, $actual:expr) => {
        if let Some(report) = float($request, $profile, $context, $path, $expected, $actual)? {
            return Ok(Some(report));
        }
    };
}

pub(crate) fn joint_observation(
    executor: &TimelineExecutor,
    joint_id: &ScenarioId,
    joint: JointId,
) -> Result<RigidJointSnapshot, NativeRigidWorldError> {
    let snapshot = executor.world.joint_snapshot(joint).map_err(|error| {
        NativeRigidWorldError::Declaration {
            checkpoint_id: joint_id.as_str().into(),
            message: error.to_string().into(),
        }
    })?;
    let [body_a, body_b] = snapshot.bodies();
    let inverse_timestep = 1.0 / f32::from_bits(liquidfun_test_protocol::RIGID_WORLD_TIMESTEP_BITS);
    let dependencies = dependencies(executor, &snapshot)?;
    let (branch_state, coordinate, speed) = joint_state(&snapshot);
    Ok(RigidJointSnapshot {
        joint_id: joint_id.clone(),
        joint_kind: joint_kind(snapshot.kind()),
        body_a_id: executor.semantic_body(body_a)?,
        body_b_id: executor.semantic_body(body_b)?,
        collide_connected: snapshot.collide_connected(),
        dependencies: dependencies.into_boxed_slice(),
        branch_state,
        coordinate_bits: FloatBits::from_f32(coordinate),
        speed_bits: FloatBits::from_f32(speed),
        reaction_force: {
            let value = executor
                .world
                .joint_reaction_force(joint, inverse_timestep)
                .map_err(|error| observation_error(joint_id, error))?;
            liquidfun_test_protocol::Vec2Bits {
                x_bits: FloatBits::from_f32(value.x),
                y_bits: FloatBits::from_f32(value.y),
            }
        },
        reaction_torque_bits: FloatBits::from_f32(
            executor
                .world
                .joint_reaction_torque(joint, inverse_timestep)
                .map_err(|error| observation_error(joint_id, error))?,
        ),
    })
}

fn dependencies(
    executor: &TimelineExecutor,
    snapshot: &JointSnapshot,
) -> Result<Vec<ScenarioId>, NativeRigidWorldError> {
    let JointSpecificSnapshot::Gear(gear) = snapshot.specific() else {
        return Ok(Vec::new());
    };
    gear.source_joints()
        .into_iter()
        .map(|joint| executor.semantic_joint(joint))
        .collect()
}

fn joint_state(snapshot: &JointSnapshot) -> (RigidJointBranchState, f32, f32) {
    match snapshot.specific() {
        JointSpecificSnapshot::Revolute(state) => (
            branch_state(state.limit_state()),
            state.angle(),
            state.speed(),
        ),
        JointSpecificSnapshot::Prismatic(state) => (
            branch_state(state.limit_state()),
            state.translation(),
            state.speed(),
        ),
        JointSpecificSnapshot::Distance(state) => {
            (RigidJointBranchState::Inactive, state.current_length(), 0.0)
        }
        JointSpecificSnapshot::Pulley(state) => (
            RigidJointBranchState::Inactive,
            state.current_length_a() + state.ratio() * state.current_length_b(),
            0.0,
        ),
        JointSpecificSnapshot::Mouse(_) | JointSpecificSnapshot::Friction(_) => {
            (RigidJointBranchState::Inactive, 0.0, 0.0)
        }
        JointSpecificSnapshot::Gear(state) => (
            RigidJointBranchState::Active,
            state.coordinate1() + state.ratio() * state.coordinate2(),
            0.0,
        ),
        JointSpecificSnapshot::Wheel(state) => (
            RigidJointBranchState::Active,
            state.translation(),
            state.speed(),
        ),
        JointSpecificSnapshot::Weld(_) => (RigidJointBranchState::Active, 0.0, 0.0),
        JointSpecificSnapshot::Rope(state) => (
            branch_state(state.limit_state()),
            state.current_length(),
            0.0,
        ),
        JointSpecificSnapshot::Motor(state) => {
            (RigidJointBranchState::Active, state.angular_error(), 0.0)
        }
        JointSpecificSnapshot::Pending => (RigidJointBranchState::Inactive, 0.0, 0.0),
    }
}

const fn branch_state(state: JointLimitState) -> RigidJointBranchState {
    match state {
        JointLimitState::Inactive => RigidJointBranchState::Inactive,
        JointLimitState::AtLower => RigidJointBranchState::AtLower,
        JointLimitState::AtUpper => RigidJointBranchState::AtUpper,
        JointLimitState::Equal => RigidJointBranchState::Equal,
    }
}

pub(crate) const fn joint_kind(kind: JointKind) -> RigidJointKind {
    match kind {
        JointKind::Revolute => RigidJointKind::Revolute,
        JointKind::Prismatic => RigidJointKind::Prismatic,
        JointKind::Distance => RigidJointKind::Distance,
        JointKind::Pulley => RigidJointKind::Pulley,
        JointKind::Mouse => RigidJointKind::Mouse,
        JointKind::Gear => RigidJointKind::Gear,
        JointKind::Wheel => RigidJointKind::Wheel,
        JointKind::Weld => RigidJointKind::Weld,
        JointKind::Friction => RigidJointKind::Friction,
        JointKind::Rope => RigidJointKind::Rope,
        JointKind::Motor => RigidJointKind::Motor,
    }
}

fn observation_error(
    joint_id: &ScenarioId,
    error: impl std::fmt::Display,
) -> NativeRigidWorldError {
    NativeRigidWorldError::Declaration {
        checkpoint_id: joint_id.as_str().into(),
        message: error.to_string().into(),
    }
}

pub(super) fn first_divergence(
    request: &RigidWorldRequestRecord,
    expected: &RigidWorldResultRecord,
    actual: &RigidWorldResultRecord,
    phase6_profile: &Phase6PolicyProfile,
    profile: &Phase8PolicyProfile,
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    for (timeline_index, (expected_timeline, actual_timeline)) in expected
        .timelines()
        .iter()
        .zip(actual.timelines())
        .enumerate()
    {
        if !liquidfun_test_protocol::RigidWorldWitnessFamily::PHASE8_REQUIRED
            .contains(&expected_timeline.witness_family)
        {
            continue;
        }
        for (checkpoint_index, (expected_checkpoint, actual_checkpoint)) in expected_timeline
            .checkpoints
            .iter()
            .zip(actual_timeline.checkpoints.iter())
            .enumerate()
        {
            let location = Location {
                timeline_index,
                checkpoint_index,
            };
            if let Some(report) = compare_checkpoint_inherited(
                request,
                phase6_profile,
                location,
                expected_checkpoint,
                actual_checkpoint,
            ) {
                return Ok(Some(report));
            }
            if let Some(report) = compare_observations(
                request,
                profile,
                location,
                expected_checkpoint,
                actual_checkpoint,
            )? {
                return Ok(Some(report));
            }
        }
    }
    Ok(None)
}

fn compare_observations(
    request: &RigidWorldRequestRecord,
    profile: &Phase8PolicyProfile,
    location: Location,
    expected: &RigidWorldCheckpointResult,
    actual: &RigidWorldCheckpointResult,
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    let context = EvidenceContext::checkpoint(location);
    let expected_lifecycle_count = expected
        .observations
        .iter()
        .filter(|observation| matches!(observation, RigidWorldObservation::Lifecycle { .. }))
        .count();
    let actual_lifecycle_count = actual
        .observations
        .iter()
        .filter(|observation| matches!(observation, RigidWorldObservation::Lifecycle { .. }))
        .count();
    if let Some(report) = exact(
        request,
        profile,
        context,
        "rigid_world.phase8.lifecycle.multiplicity",
        RigidMismatchKind::Order,
        &expected_lifecycle_count,
        &actual_lifecycle_count,
    )? {
        return Ok(Some(report));
    }
    let expected_order = expected
        .observations
        .iter()
        .map(observation_kind)
        .collect::<Vec<_>>();
    let actual_order = actual
        .observations
        .iter()
        .map(observation_kind)
        .collect::<Vec<_>>();
    if let Some(report) = exact(
        request,
        profile,
        context,
        "rigid_world.phase8.observations.order",
        RigidMismatchKind::Order,
        &expected_order,
        &actual_order,
    )? {
        return Ok(Some(report));
    }
    for (expected_observation, actual_observation) in
        expected.observations.iter().zip(actual.observations.iter())
    {
        let maybe_report = match (expected_observation, actual_observation) {
            (
                RigidWorldObservation::Joint { snapshot: expected },
                RigidWorldObservation::Joint { snapshot: actual },
            ) => compare_joint(request, profile, context, expected, actual)?,
            (
                RigidWorldObservation::Rope { snapshot: expected },
                RigidWorldObservation::Rope { snapshot: actual },
            ) => compare_rope(request, profile, context, expected, actual)?,
            (
                RigidWorldObservation::Lifecycle { event: expected },
                RigidWorldObservation::Lifecycle { event: actual },
            ) => compare_lifecycle(request, profile, context, expected, actual)?,
            (
                RigidWorldObservation::Reconstruction { record: expected },
                RigidWorldObservation::Reconstruction { record: actual },
            ) => compare_reconstruction(request, profile, context, expected, actual)?,
            (
                RigidWorldObservation::Diagnostics { snapshot: expected },
                RigidWorldObservation::Diagnostics { snapshot: actual },
            ) => compare_diagnostics(request, profile, context, expected, actual)?,
            _ => exact(
                request,
                profile,
                context,
                "rigid_world.phase8.field.presence",
                RigidMismatchKind::Exact,
                expected_observation,
                actual_observation,
            )?,
        };
        if maybe_report.is_some() {
            return Ok(maybe_report);
        }
    }
    Ok(None)
}

fn compare_joint(
    request: &RigidWorldRequestRecord,
    profile: &Phase8PolicyProfile,
    context: EvidenceContext<'_>,
    expected: &RigidJointSnapshot,
    actual: &RigidJointSnapshot,
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    let context = EvidenceContext {
        maybe_entity: Some(expected.joint_id.as_str()),
        ..context
    };
    phase8_exact!(
        request,
        profile,
        context,
        "rigid_world.phase8.joint.id",
        expected.joint_id,
        actual.joint_id
    );
    phase8_exact!(
        request,
        profile,
        context,
        "rigid_world.phase8.joint.kind",
        expected.joint_kind,
        actual.joint_kind
    );
    phase8_exact!(
        request,
        profile,
        context,
        "rigid_world.phase8.joint.body_ids",
        (&expected.body_a_id, &expected.body_b_id),
        (&actual.body_a_id, &actual.body_b_id)
    );
    phase8_exact!(
        request,
        profile,
        context,
        "rigid_world.phase8.joint.collide_connected",
        expected.collide_connected,
        actual.collide_connected
    );
    phase8_exact!(
        request,
        profile,
        context,
        "rigid_world.phase8.joint.dependencies.order",
        &expected.dependencies,
        &actual.dependencies
    );
    phase8_exact!(
        request,
        profile,
        context,
        "rigid_world.phase8.joint.branch_state",
        expected.branch_state,
        actual.branch_state
    );
    phase8_float!(
        request,
        profile,
        context,
        "rigid_world.phase8.joint.coordinate",
        expected.coordinate_bits,
        actual.coordinate_bits
    );
    phase8_float!(
        request,
        profile,
        context,
        "rigid_world.phase8.joint.speed",
        expected.speed_bits,
        actual.speed_bits
    );
    phase8_float!(
        request,
        profile,
        context,
        "rigid_world.phase8.joint.reaction_force.x",
        expected.reaction_force.x_bits,
        actual.reaction_force.x_bits
    );
    phase8_float!(
        request,
        profile,
        context,
        "rigid_world.phase8.joint.reaction_force.y",
        expected.reaction_force.y_bits,
        actual.reaction_force.y_bits
    );
    phase8_float!(
        request,
        profile,
        context,
        "rigid_world.phase8.joint.reaction_torque",
        expected.reaction_torque_bits,
        actual.reaction_torque_bits
    );
    Ok(None)
}

fn compare_rope(
    request: &RigidWorldRequestRecord,
    profile: &Phase8PolicyProfile,
    context: EvidenceContext<'_>,
    expected: &liquidfun_test_protocol::RigidRopeSnapshot,
    actual: &liquidfun_test_protocol::RigidRopeSnapshot,
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    let context = EvidenceContext {
        maybe_entity: Some(expected.rope_id.as_str()),
        ..context
    };
    phase8_exact!(
        request,
        profile,
        context,
        "rigid_world.phase8.rope.id",
        expected.rope_id,
        actual.rope_id
    );
    phase8_exact!(
        request,
        profile,
        context,
        "rigid_world.phase8.rope.vertex_count",
        expected.vertices.len(),
        actual.vertices.len()
    );
    for (expected_vertex, actual_vertex) in expected.vertices.iter().zip(actual.vertices.iter()) {
        phase8_float!(
            request,
            profile,
            context,
            "rigid_world.phase8.rope.vertex.x",
            expected_vertex.x_bits,
            actual_vertex.x_bits
        );
        phase8_float!(
            request,
            profile,
            context,
            "rigid_world.phase8.rope.vertex.y",
            expected_vertex.y_bits,
            actual_vertex.y_bits
        );
    }
    Ok(None)
}

fn compare_lifecycle(
    request: &RigidWorldRequestRecord,
    profile: &Phase8PolicyProfile,
    context: EvidenceContext<'_>,
    expected: &liquidfun_test_protocol::RigidLifecycleObservation,
    actual: &liquidfun_test_protocol::RigidLifecycleObservation,
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    phase8_exact!(
        request,
        profile,
        context,
        "rigid_world.phase8.lifecycle.order",
        expected.ordinal,
        actual.ordinal
    );
    phase8_exact!(
        request,
        profile,
        context,
        "rigid_world.phase8.lifecycle.kind",
        expected.kind,
        actual.kind
    );
    phase8_exact!(
        request,
        profile,
        context,
        "rigid_world.phase8.lifecycle.identity",
        (&expected.maybe_contact, &expected.maybe_entity_id),
        (&actual.maybe_contact, &actual.maybe_entity_id)
    );
    Ok(None)
}

fn compare_reconstruction(
    request: &RigidWorldRequestRecord,
    profile: &Phase8PolicyProfile,
    context: EvidenceContext<'_>,
    expected: &liquidfun_test_protocol::RigidReconstructionObservation,
    actual: &liquidfun_test_protocol::RigidReconstructionObservation,
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    phase8_exact!(
        request,
        profile,
        context,
        "rigid_world.phase8.reconstruction.order",
        (expected.ordinal, &expected.entity_id),
        (actual.ordinal, &actual.entity_id)
    );
    phase8_exact!(
        request,
        profile,
        context,
        "rigid_world.phase8.reconstruction.kind",
        expected.kind,
        actual.kind
    );
    phase8_exact!(
        request,
        profile,
        context,
        "rigid_world.phase8.reconstruction.support",
        expected.support,
        actual.support
    );
    phase8_exact!(
        request,
        profile,
        context,
        "rigid_world.phase8.reconstruction.dependencies.order",
        &expected.dependency_ids,
        &actual.dependency_ids
    );
    Ok(None)
}

fn compare_diagnostics(
    request: &RigidWorldRequestRecord,
    profile: &Phase8PolicyProfile,
    context: EvidenceContext<'_>,
    expected: &liquidfun_test_protocol::RigidDiagnosticsObservation,
    actual: &liquidfun_test_protocol::RigidDiagnosticsObservation,
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    phase8_exact!(
        request,
        profile,
        context,
        "rigid_world.phase8.diagnostics.counts",
        (
            expected.body_count,
            expected.fixture_count,
            expected.joint_count,
            expected.contact_count,
            expected.tree_height,
            expected.tree_max_balance
        ),
        (
            actual.body_count,
            actual.fixture_count,
            actual.joint_count,
            actual.contact_count,
            actual.tree_height,
            actual.tree_max_balance
        )
    );
    phase8_float!(
        request,
        profile,
        context,
        "rigid_world.phase8.diagnostics.tree_quality",
        expected.tree_quality_bits,
        actual.tree_quality_bits
    );
    Ok(None)
}

fn observation_kind(observation: &RigidWorldObservation) -> &'static str {
    match observation {
        RigidWorldObservation::BodyState { .. } => "body_state",
        RigidWorldObservation::Step { .. } => "step",
        RigidWorldObservation::Query { .. } => "query",
        RigidWorldObservation::RayCast { .. } => "ray_cast",
        RigidWorldObservation::OriginShift { .. } => "origin_shift",
        RigidWorldObservation::Joint { .. } => "joint",
        RigidWorldObservation::Rope { .. } => "rope",
        RigidWorldObservation::Lifecycle { .. } => "lifecycle",
        RigidWorldObservation::Reconstruction { .. } => "reconstruction",
        RigidWorldObservation::Diagnostics { .. } => "diagnostics",
        RigidWorldObservation::Particle { .. } => "particle",
        RigidWorldObservation::ParticleGroup { .. } => "particle_group",
    }
}

fn exact<T: std::fmt::Debug + PartialEq>(
    request: &RigidWorldRequestRecord,
    profile: &Phase8PolicyProfile,
    context: EvidenceContext<'_>,
    path: &'static str,
    kind: RigidMismatchKind,
    expected: &T,
    actual: &T,
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    let policy = policy(profile, path)?;
    if policy.comparison() != FieldComparison::ExactDiscrete {
        return Err(harness(path, "exact_discrete", policy));
    }
    Ok((expected != actual).then(|| {
        mismatch_with_context(
            request,
            profile.profile_sha256(),
            policy,
            context,
            path,
            kind,
            format!("{expected:?}"),
            format!("{actual:?}"),
            None,
        )
    }))
}

fn float(
    request: &RigidWorldRequestRecord,
    profile: &Phase8PolicyProfile,
    context: EvidenceContext<'_>,
    path: &'static str,
    expected: FloatBits,
    actual: FloatBits,
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    let policy = policy(profile, path)?;
    if !matches!(policy.comparison(), FieldComparison::Float { .. }) {
        return Err(harness(path, "float", policy));
    }
    Ok(
        (!float_values_match_with_policy(expected, actual, policy)).then(|| {
            mismatch_with_context(
                request,
                profile.profile_sha256(),
                policy,
                context,
                path,
                RigidMismatchKind::Numeric,
                format!("0x{:08x}", expected.bits()),
                format!("0x{:08x}", actual.bits()),
                Some((expected, actual)),
            )
        }),
    )
}

fn policy<'a>(
    profile: &'a Phase8PolicyProfile,
    path: &'static str,
) -> Result<&'a FieldPolicy, RigidComparisonFailure> {
    profile.field(path).ok_or_else(|| {
        RigidComparisonFailure::Harness(RigidHarnessReport {
            reason: "missing_policy".into(),
            expected: path.into(),
            actual: "unregistered".into(),
        })
    })
}

fn harness(path: &str, expected: &str, actual: &FieldPolicy) -> RigidComparisonFailure {
    RigidComparisonFailure::Harness(RigidHarnessReport {
        reason: "policy_kind".into(),
        expected: format!("{path}:{expected}").into_boxed_str(),
        actual: format!("{:?}", actual.comparison()).into_boxed_str(),
    })
}
