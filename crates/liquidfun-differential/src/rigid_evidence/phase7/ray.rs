//! Final-interval ray identity and numeric comparison.

use liquidfun_test_protocol::{
    FloatBits, Phase7PolicyProfile, RigidFixtureChildOccurrence, RigidRayCompletion,
    RigidRayHitObservation, RigidRayObservation, RigidWorldRequestRecord,
};

use crate::{float_values_match_with_policy, multiset_values_match};

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
    if let Some(report) = exact(
        request,
        profile,
        context,
        "rigid_world.phase7.ray.final_max_fraction",
        RigidMismatchKind::Exact,
        &expected.final_max_fraction_bits.bits(),
        &actual.final_max_fraction_bits.bits(),
    )? {
        return Ok(Some(report));
    }
    if expected.completion == RigidRayCompletion::Terminated {
        return compare_terminated_count(request, profile, context, expected, actual);
    }
    let final_max_fraction = expected.final_max_fraction_bits.to_f32();
    let expected_hits = hits_within_final_interval(&expected.hits, final_max_fraction);
    let actual_hits = hits_within_final_interval(&actual.hits, final_max_fraction);
    compare_hit_multisets(request, profile, context, &expected_hits, &actual_hits)
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

fn compare_hit_multisets(
    request: &RigidWorldRequestRecord,
    profile: &Phase7PolicyProfile,
    context: EvidenceContext<'_>,
    expected: &[&RigidRayHitObservation],
    actual: &[&RigidRayHitObservation],
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    let expected_ids = hit_identities(expected);
    let actual_ids = hit_identities(actual);
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
        for (index, actual_hit) in actual.iter().copied().enumerate() {
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
            .copied()
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

fn hits_within_final_interval(
    hits: &[RigidRayHitObservation],
    final_max_fraction: f32,
) -> Vec<&RigidRayHitObservation> {
    hits.iter()
        .filter(|hit| hit.fraction_bits.to_f32() <= final_max_fraction)
        .collect()
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
