//! Equal-minimum ray identity and numeric comparison.

use liquidfun_test_protocol::{
    Phase7PolicyProfile, RigidFixtureChildOccurrence, RigidRayHitObservation, RigidRayObservation,
    RigidWorldRequestRecord,
};

use crate::set_values_match;

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
    let expected_nonminimum = nonminimum_hits(&expected.hits);
    let actual_nonminimum = nonminimum_hits(&actual.hits);
    let expected_nonminimum_ids = hit_identities(&expected_nonminimum);
    let actual_nonminimum_ids = hit_identities(&actual_nonminimum);
    if let Some(report) = exact(
        request,
        profile,
        context,
        "rigid_world.phase7.ray.hit.identity",
        RigidMismatchKind::Order,
        &expected_nonminimum_ids,
        &actual_nonminimum_ids,
    )? {
        return Ok(Some(report));
    }
    for expected_hit in expected_minimum {
        let Some(actual_hit) = actual_minimum.iter().find(|actual_hit| {
            expected_hit.fixture_id == actual_hit.fixture_id
                && expected_hit.child_index == actual_hit.child_index
        }) else {
            continue;
        };
        if let Some(report) = compare_hit(request, profile, context, expected_hit, actual_hit)? {
            return Ok(Some(report));
        }
    }
    for (expected_hit, actual_hit) in expected_nonminimum.iter().zip(actual_nonminimum.iter()) {
        if let Some(report) = compare_hit(request, profile, context, expected_hit, actual_hit)? {
            return Ok(Some(report));
        }
    }
    Ok(None)
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
    for (path, expected, actual) in [
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
    ] {
        if let Some(report) = float(request, profile, context, path, expected, actual)? {
            return Ok(Some(report));
        }
    }
    Ok(None)
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

fn nonminimum_hits(hits: &[RigidRayHitObservation]) -> Vec<&RigidRayHitObservation> {
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
        .filter(|hit| hit.fraction_bits != minimum_bits)
        .collect()
}

fn hit_identities(hits: &[&RigidRayHitObservation]) -> Vec<RigidFixtureChildOccurrence> {
    hits.iter()
        .map(|hit| RigidFixtureChildOccurrence {
            fixture_id: hit.fixture_id.clone(),
            child_index: hit.child_index,
        })
        .collect()
}
