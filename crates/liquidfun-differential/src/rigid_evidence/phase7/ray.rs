//! Equal-minimum ray identity and numeric comparison.

use liquidfun_test_protocol::{
    FloatBits, Phase7PolicyProfile, RigidFixtureChildOccurrence, RigidRayCompletion,
    RigidRayDirective, RigidRayHitObservation, RigidRayObservation, RigidWorldAction,
    RigidWorldRequestRecord,
};

use crate::{float_values_match_with_policy, multiset_values_match, set_values_match};

use super::{exact, float, policy};
use crate::rigid_evidence::{
    EvidenceContext, RigidComparisonFailure, RigidMismatchKind, RigidMismatchReport,
    mismatch_with_context,
};

pub(super) fn compare_ray(
    request: &RigidWorldRequestRecord,
    profile: &Phase7PolicyProfile,
    context: EvidenceContext<'_>,
    expected: &RigidRayObservation,
    actual: &RigidRayObservation,
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    if let Some(report) = exact(
        request,
        profile,
        context,
        "rigid_world.phase7.ray.completion",
        RigidMismatchKind::Exact,
        &expected.completion,
        &actual.completion,
    )? {
        return Ok(Some(report));
    }
    if expected.completion == RigidRayCompletion::Terminated {
        return compare_terminated_count(request, profile, context, expected, actual);
    }
    if ray_uses_clipping(request, context) {
        return compare_closest_hits(request, profile, context, expected, actual);
    }
    compare_hit_multisets(request, profile, context, &expected.hits, &actual.hits)
}

fn compare_terminated_count(
    request: &RigidWorldRequestRecord,
    profile: &Phase7PolicyProfile,
    context: EvidenceContext<'_>,
    expected: &RigidRayObservation,
    actual: &RigidRayObservation,
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    exact(
        request,
        profile,
        context,
        "rigid_world.phase7.ray.hit.identity",
        RigidMismatchKind::Order,
        &expected.hits.len(),
        &actual.hits.len(),
    )
}

fn compare_closest_hits(
    request: &RigidWorldRequestRecord,
    profile: &Phase7PolicyProfile,
    context: EvidenceContext<'_>,
    expected: &RigidRayObservation,
    actual: &RigidRayObservation,
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    let expected_minimum = minimum_hits(&expected.hits);
    let actual_minimum = minimum_hits(&actual.hits);
    let expected_ids = hit_identities(&expected_minimum);
    let actual_ids = hit_identities(&actual_minimum);
    let identity_path = "rigid_world.phase7.ray.equal_minimum.identities";
    let identity_policy = policy(profile, identity_path)?;
    if !set_values_match(&expected_ids, &actual_ids) {
        return Ok(Some(mismatch_with_context(
            request,
            profile.profile_sha256(),
            identity_policy,
            context,
            identity_path,
            RigidMismatchKind::Order,
            format!("{expected_ids:?}"),
            format!("{actual_ids:?}"),
            None,
        )));
    }

    for expected_hit in unique_hits_by_identity(expected_minimum) {
        let mut has_matching_actual = false;
        for actual_hit in actual_minimum.iter().copied() {
            if same_identity(expected_hit, actual_hit)
                && hit_values_match(profile, expected_hit, actual_hit)?
            {
                has_matching_actual = true;
                break;
            }
        }
        if !has_matching_actual
            && let Some(actual_hit) = actual_minimum
                .iter()
                .copied()
                .find(|actual_hit| same_identity(expected_hit, actual_hit))
            && let Some(report) = compare_hit(request, profile, context, expected_hit, actual_hit)?
        {
            return Ok(Some(report));
        }
    }
    Ok(None)
}

fn compare_hit_multisets(
    request: &RigidWorldRequestRecord,
    profile: &Phase7PolicyProfile,
    context: EvidenceContext<'_>,
    expected: &[RigidRayHitObservation],
    actual: &[RigidRayHitObservation],
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    let expected_hits = expected.iter().collect::<Vec<_>>();
    let actual_hits = actual.iter().collect::<Vec<_>>();
    let expected_ids = hit_identities(&expected_hits);
    let actual_ids = hit_identities(&actual_hits);
    let identity_path = "rigid_world.phase7.ray.hit.identity";
    let identity_policy = policy(profile, identity_path)?;
    if !multiset_values_match(&expected_ids, &actual_ids) {
        return Ok(Some(mismatch_with_context(
            request,
            profile.profile_sha256(),
            identity_policy,
            context,
            identity_path,
            RigidMismatchKind::Order,
            format!("{expected_ids:?}"),
            format!("{actual_ids:?}"),
            None,
        )));
    }

    let mut matched = vec![false; actual.len()];
    for expected_hit in expected {
        let mut maybe_index = None;
        for (index, actual_hit) in actual.iter().enumerate() {
            if !matched[index]
                && same_identity(expected_hit, actual_hit)
                && hit_values_match(profile, expected_hit, actual_hit)?
            {
                maybe_index = Some(index);
                break;
            }
        }
        if let Some(index) = maybe_index {
            matched[index] = true;
            continue;
        }
        let actual_hit = actual
            .iter()
            .enumerate()
            .find(|(index, actual_hit)| !matched[*index] && same_identity(expected_hit, actual_hit))
            .map(|(_, hit)| hit)
            .expect("equal identity multisets leave an unmatched peer");
        if let Some(report) = compare_hit(request, profile, context, expected_hit, actual_hit)? {
            return Ok(Some(report));
        }
    }
    Ok(None)
}

fn ray_uses_clipping(request: &RigidWorldRequestRecord, context: EvidenceContext<'_>) -> bool {
    let Some(action_id) = context.maybe_action_id else {
        return false;
    };
    request.scenario().timelines()[context.location.timeline_index]
        .actions()
        .iter()
        .find(|record| record.action_id().as_str() == action_id)
        .is_some_and(|record| {
            let RigidWorldAction::RayCast {
                directive_rules, ..
            } = record.action()
            else {
                return false;
            };
            directive_rules
                .iter()
                .any(|rule| matches!(rule.directive, RigidRayDirective::Clip { .. }))
        })
}

fn hit_values_match(
    profile: &Phase7PolicyProfile,
    expected: &RigidRayHitObservation,
    actual: &RigidRayHitObservation,
) -> Result<bool, RigidComparisonFailure> {
    for (path, expected, actual) in hit_float_fields(expected, actual) {
        if !float_values_match_with_policy(expected, actual, policy(profile, path)?) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn compare_hit(
    request: &RigidWorldRequestRecord,
    profile: &Phase7PolicyProfile,
    context: EvidenceContext<'_>,
    expected: &RigidRayHitObservation,
    actual: &RigidRayHitObservation,
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    let entity = format!("{}:{}", expected.fixture_id, expected.child_index);
    let context = EvidenceContext {
        maybe_entity: Some(&entity),
        ..context
    };
    for (path, expected, actual) in hit_float_fields(expected, actual) {
        if let Some(report) = float(request, profile, context, path, expected, actual)? {
            return Ok(Some(report));
        }
    }
    Ok(None)
}

fn hit_float_fields(
    expected: &RigidRayHitObservation,
    actual: &RigidRayHitObservation,
) -> [(&'static str, FloatBits, FloatBits); 5] {
    [
        (
            "rigid_world.phase7.ray.fraction",
            expected.fraction_bits,
            actual.fraction_bits,
        ),
        (
            "rigid_world.phase7.ray.point.x",
            expected.point.x_bits,
            actual.point.x_bits,
        ),
        (
            "rigid_world.phase7.ray.point.y",
            expected.point.y_bits,
            actual.point.y_bits,
        ),
        (
            "rigid_world.phase7.ray.normal.x",
            expected.normal.x_bits,
            actual.normal.x_bits,
        ),
        (
            "rigid_world.phase7.ray.normal.y",
            expected.normal.y_bits,
            actual.normal.y_bits,
        ),
    ]
}

fn minimum_hits(hits: &[RigidRayHitObservation]) -> Vec<&RigidRayHitObservation> {
    let Some(minimum_bits) = hits
        .iter()
        .min_by(|left, right| {
            left.fraction_bits
                .to_f32()
                .total_cmp(&right.fraction_bits.to_f32())
        })
        .map(|hit| hit.fraction_bits)
    else {
        return Vec::new();
    };
    hits.iter()
        .filter(|hit| hit.fraction_bits == minimum_bits)
        .collect()
}

fn unique_hits_by_identity(hits: Vec<&RigidRayHitObservation>) -> Vec<&RigidRayHitObservation> {
    let mut unique: Vec<&RigidRayHitObservation> = Vec::new();
    for hit in hits {
        if !unique.iter().any(|candidate| same_identity(hit, candidate)) {
            unique.push(hit);
        }
    }
    unique
}

fn same_identity(left: &RigidRayHitObservation, right: &RigidRayHitObservation) -> bool {
    left.fixture_id == right.fixture_id && left.child_index == right.child_index
}

fn hit_identities(hits: &[&RigidRayHitObservation]) -> Vec<RigidFixtureChildOccurrence> {
    hits.iter()
        .map(|hit| RigidFixtureChildOccurrence {
            fixture_id: hit.fixture_id.clone(),
            child_index: hit.child_index,
        })
        .collect()
}
