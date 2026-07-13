//! Body-control and configured-step observation comparison.

use liquidfun_test_protocol::{
    Phase7PolicyProfile, RigidBodyControlSnapshot, RigidStepOutcome, RigidWorldRequestRecord,
};

use crate::rigid_evidence::{
    EvidenceContext, RigidComparisonFailure, RigidCompletionContext, RigidMismatchKind,
    RigidMismatchReport,
};

use super::{exact, float};

macro_rules! phase7_exact {
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

macro_rules! phase7_float {
    ($request:expr, $profile:expr, $context:expr, $path:expr, $expected:expr, $actual:expr) => {
        if let Some(report) = float($request, $profile, $context, $path, $expected, $actual)? {
            return Ok(Some(report));
        }
    };
}

pub(super) fn compare_body_observation(
    request: &RigidWorldRequestRecord,
    profile: &Phase7PolicyProfile,
    context: EvidenceContext<'_>,
    expected: &RigidBodyControlSnapshot,
    actual: &liquidfun_test_protocol::RigidBodyControlSnapshot,
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    phase7_exact!(
        request,
        profile,
        context,
        "rigid_world.phase7.body.id",
        expected.body_id,
        actual.body_id
    );
    phase7_exact!(
        request,
        profile,
        context,
        "rigid_world.phase7.body.awake",
        expected.awake,
        actual.awake
    );
    phase7_exact!(
        request,
        profile,
        context,
        "rigid_world.phase7.body.bullet",
        expected.bullet,
        actual.bullet
    );
    phase7_exact!(
        request,
        profile,
        context,
        "rigid_world.phase7.body.sleeping_allowed",
        expected.sleeping_allowed,
        actual.sleeping_allowed
    );
    phase7_exact!(
        request,
        profile,
        context,
        "rigid_world.phase7.body.fixed_rotation",
        expected.fixed_rotation,
        actual.fixed_rotation
    );
    phase7_float!(
        request,
        profile,
        context,
        "rigid_world.phase7.body.linear_velocity.x",
        expected.linear_velocity.x_bits,
        actual.linear_velocity.x_bits
    );
    phase7_float!(
        request,
        profile,
        context,
        "rigid_world.phase7.body.linear_velocity.y",
        expected.linear_velocity.y_bits,
        actual.linear_velocity.y_bits
    );
    phase7_float!(
        request,
        profile,
        context,
        "rigid_world.phase7.body.angular_velocity",
        expected.angular_velocity_bits,
        actual.angular_velocity_bits
    );
    phase7_float!(
        request,
        profile,
        context,
        "rigid_world.phase7.body.linear_damping",
        expected.linear_damping_bits,
        actual.linear_damping_bits
    );
    phase7_float!(
        request,
        profile,
        context,
        "rigid_world.phase7.body.angular_damping",
        expected.angular_damping_bits,
        actual.angular_damping_bits
    );
    phase7_float!(
        request,
        profile,
        context,
        "rigid_world.phase7.body.gravity_scale",
        expected.gravity_scale_bits,
        actual.gravity_scale_bits
    );
    Ok(None)
}

pub(super) fn compare_step(
    request: &RigidWorldRequestRecord,
    profile: &Phase7PolicyProfile,
    mut context: EvidenceContext<'_>,
    expected: RigidStepOutcome,
    actual: RigidStepOutcome,
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    context.maybe_completion_context = Some(RigidCompletionContext::new(expected, actual));
    match (expected, actual) {
        (
            RigidStepOutcome::Completed {
                completion: expected,
            },
            RigidStepOutcome::Completed { completion: actual },
        ) => exact(
            request,
            profile,
            context,
            "rigid_world.phase7.step.completion",
            RigidMismatchKind::Exact,
            &expected,
            &actual,
        ),
        (
            RigidStepOutcome::Partial {
                classification: expected,
            },
            RigidStepOutcome::Partial {
                classification: actual,
            },
        ) => exact(
            request,
            profile,
            context,
            "rigid_world.phase7.step.partial_classification",
            RigidMismatchKind::Exact,
            &expected,
            &actual,
        ),
        _ => exact(
            request,
            profile,
            context,
            "rigid_world.phase7.step.outcome.kind",
            RigidMismatchKind::Exact,
            &step_kind(expected),
            &step_kind(actual),
        ),
    }
}

const fn step_kind(outcome: RigidStepOutcome) -> &'static str {
    match outcome {
        RigidStepOutcome::Completed { .. } => "completed",
        RigidStepOutcome::Partial { .. } => "partial",
    }
}
