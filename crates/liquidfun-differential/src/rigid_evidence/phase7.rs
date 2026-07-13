//! Closed Phase 7 observation comparison and first-divergence context.

mod context;
mod observation;
mod ray;

use std::fmt::Debug;

use liquidfun_test_protocol::{
    FieldComparison, FieldPolicy, FloatBits, Phase6PolicyProfile, Phase7PolicyProfile,
    RigidWorldCheckpointResult, RigidWorldObservation, RigidWorldRequestRecord,
    RigidWorldResultRecord,
};

use crate::{float_values_match_with_policy, multiset_values_match};

use super::{
    EvidenceContext, Location, RigidComparisonFailure, RigidCompletionContext, RigidHarnessReport,
    RigidMismatchKind, RigidMismatchReport, compare_checkpoint, mismatch_with_context,
};
use context::{checkpoint_action, checkpoint_context, observation_action, observation_entity};
use observation::{compare_body_observation, compare_step};
use ray::compare_ray;

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

pub(super) fn first_divergence(
    request: &RigidWorldRequestRecord,
    expected: &RigidWorldResultRecord,
    actual: &RigidWorldResultRecord,
    phase6_profile: &Phase6PolicyProfile,
    phase7_profile: &Phase7PolicyProfile,
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    for (timeline_index, (expected_timeline, actual_timeline)) in expected
        .timelines()
        .iter()
        .zip(actual.timelines())
        .enumerate()
    {
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
            if let Some(report) = compare_phase7_checkpoint(
                request,
                phase7_profile,
                location,
                expected_checkpoint,
                actual_checkpoint,
            )? {
                return Ok(Some(report));
            }
            if let Some(report) = compare_checkpoint(
                request,
                phase6_profile,
                location,
                expected_checkpoint,
                actual_checkpoint,
            ) {
                return Ok(Some(report));
            }
        }
    }
    Ok(None)
}

fn compare_phase7_checkpoint(
    request: &RigidWorldRequestRecord,
    profile: &Phase7PolicyProfile,
    location: Location,
    expected: &RigidWorldCheckpointResult,
    actual: &RigidWorldCheckpointResult,
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    let checkpoint_context = checkpoint_context(request, location, expected, actual);
    if let Some(report) =
        compare_body_snapshots(request, profile, checkpoint_context, expected, actual)?
    {
        return Ok(Some(report));
    }
    if let Some(report) =
        compare_contact_state(request, profile, checkpoint_context, expected, actual)?
    {
        return Ok(Some(report));
    }
    compare_observations(
        request,
        profile,
        location,
        &expected.observations,
        &actual.observations,
    )
}

fn compare_body_snapshots(
    request: &RigidWorldRequestRecord,
    profile: &Phase7PolicyProfile,
    context: EvidenceContext<'_>,
    expected: &RigidWorldCheckpointResult,
    actual: &RigidWorldCheckpointResult,
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    let expected_ids = expected
        .bodies
        .iter()
        .map(|body| body.body_id.clone())
        .collect::<Vec<_>>();
    let actual_ids = actual
        .bodies
        .iter()
        .map(|body| body.body_id.clone())
        .collect::<Vec<_>>();
    if let Some(report) = exact(
        request,
        profile,
        context,
        "rigid_world.phase7.island.body_order",
        RigidMismatchKind::Order,
        &expected_ids,
        &actual_ids,
    )? {
        return Ok(Some(report));
    }
    for (expected_body, actual_body) in expected.bodies.iter().zip(actual.bodies.iter()) {
        let entity = expected_body.body_id.as_str();
        let context = EvidenceContext {
            maybe_entity: Some(entity),
            ..context
        };
        phase7_exact!(
            request,
            profile,
            context,
            "rigid_world.phase7.body.id",
            expected_body.body_id,
            actual_body.body_id
        );
        phase7_float!(
            request,
            profile,
            context,
            "rigid_world.phase7.body.transform.position.x",
            expected_body.transform.position.x_bits,
            actual_body.transform.position.x_bits
        );
        phase7_float!(
            request,
            profile,
            context,
            "rigid_world.phase7.body.transform.position.y",
            expected_body.transform.position.y_bits,
            actual_body.transform.position.y_bits
        );
        phase7_float!(
            request,
            profile,
            context,
            "rigid_world.phase7.body.transform.angle",
            expected_body.transform.angle_bits,
            actual_body.transform.angle_bits
        );
        phase7_float!(
            request,
            profile,
            context,
            "rigid_world.phase7.body.linear_velocity.x",
            expected_body.linear_velocity.x_bits,
            actual_body.linear_velocity.x_bits
        );
        phase7_float!(
            request,
            profile,
            context,
            "rigid_world.phase7.body.linear_velocity.y",
            expected_body.linear_velocity.y_bits,
            actual_body.linear_velocity.y_bits
        );
        phase7_float!(
            request,
            profile,
            context,
            "rigid_world.phase7.body.angular_velocity",
            expected_body.angular_velocity_bits,
            actual_body.angular_velocity_bits
        );
    }
    Ok(None)
}

fn compare_contact_state(
    request: &RigidWorldRequestRecord,
    profile: &Phase7PolicyProfile,
    context: EvidenceContext<'_>,
    expected: &RigidWorldCheckpointResult,
    actual: &RigidWorldCheckpointResult,
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    let expected_ids = expected
        .contacts
        .iter()
        .map(|contact| contact.identity.clone())
        .collect::<Vec<_>>();
    let actual_ids = actual
        .contacts
        .iter()
        .map(|contact| contact.identity.clone())
        .collect::<Vec<_>>();
    if let Some(report) = exact(
        request,
        profile,
        context,
        "rigid_world.phase7.island.contact_order",
        RigidMismatchKind::Order,
        &expected_ids,
        &actual_ids,
    )? {
        return Ok(Some(report));
    }
    if let Some(report) = exact(
        request,
        profile,
        context,
        "rigid_world.phase7.contact.transitions.order",
        RigidMismatchKind::Order,
        &expected.events,
        &actual.events,
    )? {
        return Ok(Some(report));
    }
    for (expected_contact, actual_contact) in expected.contacts.iter().zip(actual.contacts.iter()) {
        let entity = format!("{:?}", expected_contact.identity);
        let context = EvidenceContext {
            maybe_entity: Some(&entity),
            ..context
        };
        phase7_exact!(
            request,
            profile,
            context,
            "rigid_world.phase7.contact.identity",
            expected_contact.identity,
            actual_contact.identity
        );
        if let (Some(expected_manifold), Some(actual_manifold)) = (
            &expected_contact.maybe_manifold,
            &actual_contact.maybe_manifold,
        ) {
            for (expected_point, actual_point) in expected_manifold
                .points
                .iter()
                .zip(actual_manifold.points.iter())
            {
                phase7_float!(
                    request,
                    profile,
                    context,
                    "rigid_world.phase7.contact.normal_impulse",
                    expected_point.normal_impulse_bits,
                    actual_point.normal_impulse_bits
                );
                phase7_float!(
                    request,
                    profile,
                    context,
                    "rigid_world.phase7.contact.tangent_impulse",
                    expected_point.tangent_impulse_bits,
                    actual_point.tangent_impulse_bits
                );
            }
        }
    }
    Ok(None)
}

fn compare_observations(
    request: &RigidWorldRequestRecord,
    profile: &Phase7PolicyProfile,
    location: Location,
    expected: &[RigidWorldObservation],
    actual: &[RigidWorldObservation],
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    let mut maybe_completion = None;
    let length = expected.len().min(actual.len());
    for index in 0..length {
        let expected_observation = &expected[index];
        let actual_observation = &actual[index];
        let (action_id, stage) = observation_action(request, location, index)
            .unwrap_or_else(|| checkpoint_action(request, location));
        let context = EvidenceContext {
            location,
            maybe_action_id: Some(action_id),
            maybe_stage: Some(stage),
            maybe_entity: observation_entity(expected_observation),
            maybe_completion_context: maybe_completion,
        };
        if observation_kind(expected_observation) != observation_kind(actual_observation) {
            return exact(
                request,
                profile,
                context,
                "rigid_world.phase7.observations.order",
                RigidMismatchKind::Order,
                &observation_kind(expected_observation),
                &observation_kind(actual_observation),
            );
        }
        if let Some(report) = compare_observation(
            request,
            profile,
            context,
            expected_observation,
            actual_observation,
        )? {
            return Ok(Some(report));
        }
        if let (
            RigidWorldObservation::Step {
                outcome: expected_outcome,
            },
            RigidWorldObservation::Step {
                outcome: actual_outcome,
            },
        ) = (expected_observation, actual_observation)
        {
            maybe_completion = Some(RigidCompletionContext::new(
                *expected_outcome,
                *actual_outcome,
            ));
        }
    }
    let context = EvidenceContext {
        location,
        maybe_action_id: None,
        maybe_stage: None,
        maybe_entity: None,
        maybe_completion_context: maybe_completion,
    };
    exact(
        request,
        profile,
        context,
        "rigid_world.phase7.observations.order",
        RigidMismatchKind::Order,
        &expected.len(),
        &actual.len(),
    )
}

fn compare_observation(
    request: &RigidWorldRequestRecord,
    profile: &Phase7PolicyProfile,
    context: EvidenceContext<'_>,
    expected: &RigidWorldObservation,
    actual: &RigidWorldObservation,
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    match (expected, actual) {
        (
            RigidWorldObservation::BodyState { state: expected },
            RigidWorldObservation::BodyState { state: actual },
        ) => compare_body_observation(request, profile, context, expected, actual),
        (
            RigidWorldObservation::Step { outcome: expected },
            RigidWorldObservation::Step { outcome: actual },
        ) => compare_step(request, profile, context, *expected, *actual),
        (
            RigidWorldObservation::Query {
                observation: expected,
            },
            RigidWorldObservation::Query {
                observation: actual,
            },
        ) => {
            phase7_exact!(
                request,
                profile,
                context,
                "rigid_world.phase7.query.completion",
                expected.completion,
                actual.completion
            );
            let policy = policy(profile, "rigid_world.phase7.query.occurrences.identity")?;
            if !multiset_values_match(&expected.occurrences, &actual.occurrences) {
                return Ok(Some(mismatch_with_context(
                    request,
                    profile.profile_sha256(),
                    policy,
                    context,
                    "rigid_world.phase7.query.occurrences.identity",
                    RigidMismatchKind::Order,
                    format!("{:?}", expected.occurrences),
                    format!("{:?}", actual.occurrences),
                    None,
                )));
            }
            Ok(None)
        }
        (
            RigidWorldObservation::RayCast {
                observation: expected,
            },
            RigidWorldObservation::RayCast {
                observation: actual,
            },
        ) => compare_ray(request, profile, context, expected, actual),
        (
            RigidWorldObservation::OriginShift { shift: expected },
            RigidWorldObservation::OriginShift { shift: actual },
        ) => {
            phase7_float!(
                request,
                profile,
                context,
                "rigid_world.phase7.origin_shift.x",
                expected.x_bits,
                actual.x_bits
            );
            phase7_float!(
                request,
                profile,
                context,
                "rigid_world.phase7.origin_shift.y",
                expected.y_bits,
                actual.y_bits
            );
            Ok(None)
        }
        _ => unreachable!("observation discriminants were compared before payloads"),
    }
}

fn exact<T: Debug + PartialEq>(
    request: &RigidWorldRequestRecord,
    profile: &Phase7PolicyProfile,
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
    profile: &Phase7PolicyProfile,
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
    profile: &'a Phase7PolicyProfile,
    path: &'static str,
) -> Result<&'a FieldPolicy, RigidComparisonFailure> {
    profile.field(path).ok_or_else(|| {
        RigidComparisonFailure::Harness(RigidHarnessReport {
            reason: "unregistered_phase7_observable".into(),
            expected: path.into(),
            actual: "missing".into(),
        })
    })
}

fn harness(path: &str, expected: &str, actual: &FieldPolicy) -> RigidComparisonFailure {
    RigidComparisonFailure::Harness(RigidHarnessReport {
        reason: "incompatible_phase7_policy".into(),
        expected: format!("{path}:{expected}").into_boxed_str(),
        actual: format!("{:?}", actual.comparison()).into_boxed_str(),
    })
}

const fn observation_kind(observation: &RigidWorldObservation) -> &'static str {
    match observation {
        RigidWorldObservation::BodyState { .. } => "body_state",
        RigidWorldObservation::Step { .. } => "step",
        RigidWorldObservation::Query { .. } => "query",
        RigidWorldObservation::RayCast { .. } => "ray_cast",
        RigidWorldObservation::OriginShift { .. } => "origin_shift",
    }
}
