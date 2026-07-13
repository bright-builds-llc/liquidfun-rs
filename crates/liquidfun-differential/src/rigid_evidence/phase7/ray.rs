//! Final-interval ray identity and numeric comparison.

use std::collections::BTreeMap;

use liquidfun_test_protocol::{
    FieldPolicy, FloatBits, Phase7PolicyProfile, RigidFixtureChildOccurrence, RigidRayCompletion,
    RigidRayHitObservation, RigidRayObservation, RigidWorldRequestRecord,
};

use crate::{float_values_match_with_policy, multiset_values_match};

use super::{exact, float, policy};
use crate::rigid_evidence::{
    EvidenceContext, RigidComparisonFailure, RigidHarnessReport, RigidMismatchKind,
    RigidMismatchReport, mismatch_with_context,
};

const RAY_FRACTION_PATH: &str = "rigid_world.phase7.ray.fraction";

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
    let fraction_policy = policy(profile, RAY_FRACTION_PATH)?;
    let final_max_fraction_bits = expected.final_max_fraction_bits;
    let expected_hits =
        hits_within_final_interval(&expected.hits, final_max_fraction_bits, fraction_policy);
    let actual_hits =
        hits_within_final_interval(&actual.hits, final_max_fraction_bits, fraction_policy);
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
    let mut expected_ids = hit_identities(expected);
    let mut actual_ids = hit_identities(actual);
    expected_ids.sort_by(compare_occurrence_identity);
    actual_ids.sort_by(compare_occurrence_identity);
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

    let expected_groups = group_hits_by_identity(expected);
    let actual_groups = group_hits_by_identity(actual);
    for (identity, expected_group) in &expected_groups {
        let actual_group = actual_groups
            .get(identity)
            .expect("equal identity multisets contain the same canonical groups");
        if let Some(report) =
            compare_hit_group(request, profile, context, expected_group, actual_group)?
        {
            return Ok(Some(report));
        }
    }
    Ok(None)
}

fn compare_hit_group(
    request: &RigidWorldRequestRecord,
    profile: &Phase7PolicyProfile,
    context: EvidenceContext<'_>,
    expected: &[&RigidRayHitObservation],
    actual: &[&RigidRayHitObservation],
) -> Result<Option<RigidMismatchReport>, RigidComparisonFailure> {
    let mut actual_to_expected = vec![None; actual.len()];
    for expected_index in 0..expected.len() {
        let mut visited_actual = vec![false; actual.len()];
        let _ = try_match_hit(
            profile,
            expected,
            actual,
            expected_index,
            &mut actual_to_expected,
            &mut visited_actual,
        )?;
    }
    if actual_to_expected.iter().all(Option::is_some) {
        return Ok(None);
    }

    let mut expected_matched = vec![false; expected.len()];
    for expected_index in actual_to_expected.iter().flatten() {
        expected_matched[*expected_index] = true;
    }
    let expected_hit = expected_matched
        .iter()
        .position(|matched| !matched)
        .map(|index| expected[index])
        .expect("a non-perfect equal-size matching leaves an expected hit unmatched");
    let actual_hit = actual_to_expected
        .iter()
        .position(Option::is_none)
        .map(|index| actual[index])
        .expect("a non-perfect equal-size matching leaves an actual hit unmatched");
    if hit_values_match(profile, expected_hit, actual_hit)? {
        return Err(matching_invariant_failure(expected_hit, actual_hit));
    }
    let Some(report) = compare_hit(request, profile, context, expected_hit, actual_hit)? else {
        return Err(matching_invariant_failure(expected_hit, actual_hit));
    };
    Ok(Some(report))
}

fn matching_invariant_failure(
    expected: &RigidRayHitObservation,
    actual: &RigidRayHitObservation,
) -> RigidComparisonFailure {
    RigidComparisonFailure::Harness(RigidHarnessReport {
        reason: "maximum hit matching left a policy-compatible pair unmatched"
            .to_owned()
            .into_boxed_str(),
        expected: format!("{:?}", hit_numeric_bits(expected)).into_boxed_str(),
        actual: format!("{:?}", hit_numeric_bits(actual)).into_boxed_str(),
    })
}

fn try_match_hit(
    profile: &Phase7PolicyProfile,
    expected: &[&RigidRayHitObservation],
    actual: &[&RigidRayHitObservation],
    expected_index: usize,
    actual_to_expected: &mut [Option<usize>],
    visited_actual: &mut [bool],
) -> Result<bool, RigidComparisonFailure> {
    for actual_index in 0..actual.len() {
        if visited_actual[actual_index]
            || !hit_values_match(profile, expected[expected_index], actual[actual_index])?
        {
            continue;
        }
        visited_actual[actual_index] = true;
        let maybe_previous_expected = actual_to_expected[actual_index];
        let can_reassign = match maybe_previous_expected {
            Some(previous_expected) => try_match_hit(
                profile,
                expected,
                actual,
                previous_expected,
                actual_to_expected,
                visited_actual,
            )?,
            None => true,
        };
        if can_reassign {
            actual_to_expected[actual_index] = Some(expected_index);
            return Ok(true);
        }
    }
    Ok(false)
}

fn group_hits_by_identity<'a>(
    hits: &[&'a RigidRayHitObservation],
) -> BTreeMap<(liquidfun_test_protocol::ScenarioId, u32), Vec<&'a RigidRayHitObservation>> {
    let mut groups = BTreeMap::new();
    for hit in hits {
        groups
            .entry((hit.fixture_id.clone(), hit.child_index))
            .or_insert_with(Vec::new)
            .push(*hit);
    }
    for group in groups.values_mut() {
        group.sort_by_key(|hit| hit_numeric_bits(hit));
    }
    groups
}

fn hit_numeric_bits(hit: &RigidRayHitObservation) -> [u32; 5] {
    [
        hit.fraction_bits.bits(),
        hit.point.x_bits.bits(),
        hit.point.y_bits.bits(),
        hit.normal.x_bits.bits(),
        hit.normal.y_bits.bits(),
    ]
}

fn compare_occurrence_identity(
    left: &RigidFixtureChildOccurrence,
    right: &RigidFixtureChildOccurrence,
) -> std::cmp::Ordering {
    left.fixture_id
        .cmp(&right.fixture_id)
        .then(left.child_index.cmp(&right.child_index))
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
            RAY_FRACTION_PATH,
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

fn hits_within_final_interval<'a>(
    hits: &'a [RigidRayHitObservation],
    final_max_fraction_bits: FloatBits,
    fraction_policy: &FieldPolicy,
) -> Vec<&'a RigidRayHitObservation> {
    let final_max_fraction = final_max_fraction_bits.to_f32();
    hits.iter()
        .filter(|hit| {
            hit.fraction_bits.to_f32() <= final_max_fraction
                || float_values_match_with_policy(
                    hit.fraction_bits,
                    final_max_fraction_bits,
                    fraction_policy,
                )
        })
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
