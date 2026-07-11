//! First-divergence comparison and bounded Phase 5 evidence.

use liquidfun_test_protocol::{
    CollectionPolicy, CollisionProbeRequestRecord, CollisionProbeResult, FloatBits,
    Phase5PolicyProfile, Sha256Hex,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::float_values_match_with_policy;

/// Closed first-divergence class for collision evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "divergence_kind", rename_all = "snake_case")]
pub enum CollisionDivergence {
    /// Protocol, identity, profile, echo, or field alignment failure.
    Harness(Box<CollisionHarnessReport>),
    /// Aligned numeric values differ under the reviewed field policy.
    Numeric(Box<CollisionNumericReport>),
    /// Exact ordered semantic values differ.
    Order(Box<CollisionOrderReport>),
}

impl CollisionDivergence {
    /// Returns the deterministic replay signature.
    #[must_use]
    pub const fn signature_sha256(&self) -> &Sha256Hex {
        match self {
            Self::Harness(report) => &report.signature_sha256,
            Self::Numeric(report) => &report.signature_sha256,
            Self::Order(report) => &report.signature_sha256,
        }
    }

    /// Renders deterministic machine-readable evidence.
    ///
    /// # Errors
    ///
    /// Returns the serializer error if an invariant-breaking value cannot be encoded.
    pub fn render_machine(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}

/// Structural collision evidence, deliberately distinct from physics mismatches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CollisionHarnessReport {
    signature_sha256: Sha256Hex,
    reason: Box<str>,
    maybe_case_id: Option<Box<str>>,
    maybe_case_index: Option<u32>,
    expected: Box<str>,
    actual: Box<str>,
    request_sha256: Sha256Hex,
    profile_sha256: Sha256Hex,
    maybe_previous_case_id: Option<Box<str>>,
    maybe_next_case_id: Option<Box<str>>,
}

/// Exact-bit numeric collision mismatch evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CollisionNumericReport {
    signature_sha256: Sha256Hex,
    case_id: Box<str>,
    case_index: u32,
    field: Box<str>,
    expected_bits: FloatBits,
    actual_bits: FloatBits,
    expected_decimal: Box<str>,
    actual_decimal: Box<str>,
    absolute_difference_bits: FloatBits,
    policy_path: Box<str>,
    horizon: Box<str>,
    evidence_tier: Box<str>,
    request_sha256: Sha256Hex,
    profile_sha256: Sha256Hex,
    maybe_previous_case_id: Option<Box<str>>,
    maybe_next_case_id: Option<Box<str>>,
}

/// Exact discrete or collection-order collision mismatch evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CollisionOrderReport {
    signature_sha256: Sha256Hex,
    case_id: Box<str>,
    case_index: u32,
    field: Box<str>,
    expected: Box<str>,
    actual: Box<str>,
    policy_path: Box<str>,
    collection_policy: CollectionPolicy,
    request_sha256: Sha256Hex,
    profile_sha256: Sha256Hex,
    maybe_previous_case_id: Option<Box<str>>,
    maybe_next_case_id: Option<Box<str>>,
}

/// Compares two ordered result streams and stops at the first typed divergence.
///
/// # Errors
///
/// Returns a structural, numeric, or ordered-semantic first divergence.
pub fn compare_collision_probe_results(
    request: &CollisionProbeRequestRecord,
    expected: &[CollisionProbeResult],
    actual: &[CollisionProbeResult],
    profile: &Phase5PolicyProfile,
) -> Result<(), CollisionDivergence> {
    let request_hash = hash_serialized(request);
    if request.tolerance_profile_sha256() != profile.profile_sha256() {
        return Err(harness(
            request,
            profile,
            &request_hash,
            "profile_identity",
            None,
            profile.profile_sha256().as_str(),
            request.tolerance_profile_sha256().as_str(),
        ));
    }
    if expected.len() != request.scenario().cases().len() || actual.len() != expected.len() {
        return Err(harness(
            request,
            profile,
            &request_hash,
            "result_count",
            None,
            expected.len().to_string(),
            actual.len().to_string(),
        ));
    }

    for (index, ((case, expected), actual)) in request
        .scenario()
        .cases()
        .iter()
        .zip(expected)
        .zip(actual)
        .enumerate()
    {
        let structural = [
            (
                "case_id",
                case.case_id().to_owned(),
                actual.case_id().to_owned(),
            ),
            (
                "operation",
                format!("{:?}", case.operation()),
                format!("{:?}", actual.operation()),
            ),
            (
                "policy_path",
                case.policy_path().to_owned(),
                actual.policy_path().to_owned(),
            ),
            (
                "horizon",
                format!("{:?}", case.horizon()),
                format!("{:?}", actual.horizon()),
            ),
            (
                "collection_policy",
                format!("{:?}", case.collection_policy()),
                format!("{:?}", actual.collection_policy()),
            ),
        ];
        if let Some((field, wanted, found)) = structural
            .into_iter()
            .find(|(_field, wanted, found)| wanted != found)
        {
            return Err(harness(
                request,
                profile,
                &request_hash,
                field,
                Some(index),
                wanted,
                found,
            ));
        }
        if expected.case_id() != actual.case_id()
            || expected.operation() != actual.operation()
            || expected.policy_path() != actual.policy_path()
            || expected.horizon() != actual.horizon()
            || expected.collection_policy() != actual.collection_policy()
        {
            return Err(harness(
                request,
                profile,
                &request_hash,
                "expected_result_echo",
                Some(index),
                format!("{expected:?}"),
                format!("{actual:?}"),
            ));
        }
        let Some(field_policy) = profile.field(case.policy_path()) else {
            return Err(harness(
                request,
                profile,
                &request_hash,
                "unregistered_policy",
                Some(index),
                case.policy_path(),
                "missing",
            ));
        };
        if expected.numeric().len() != actual.numeric().len()
            || expected.discrete().len() != actual.discrete().len()
        {
            return Err(harness(
                request,
                profile,
                &request_hash,
                "field_count",
                Some(index),
                format!(
                    "numeric={},discrete={}",
                    expected.numeric().len(),
                    expected.discrete().len()
                ),
                format!(
                    "numeric={},discrete={}",
                    actual.numeric().len(),
                    actual.discrete().len()
                ),
            ));
        }
        for (expected_value, actual_value) in expected.numeric().iter().zip(actual.numeric()) {
            if expected_value.field() != actual_value.field() {
                return Err(harness(
                    request,
                    profile,
                    &request_hash,
                    "numeric_field_echo",
                    Some(index),
                    expected_value.field(),
                    actual_value.field(),
                ));
            }
            if !float_values_match_with_policy(
                expected_value.bits(),
                actual_value.bits(),
                field_policy,
            ) {
                return Err(CollisionDivergence::Numeric(Box::new(numeric_report(
                    request,
                    profile,
                    &request_hash,
                    index,
                    expected_value.field(),
                    expected_value.bits(),
                    actual_value.bits(),
                ))));
            }
        }
        for (field_index, (expected_value, actual_value)) in expected
            .discrete()
            .iter()
            .zip(actual.discrete())
            .enumerate()
        {
            if expected_value != actual_value {
                return Err(CollisionDivergence::Order(Box::new(order_report(
                    request,
                    profile,
                    &request_hash,
                    index,
                    format!("discrete[{field_index}]"),
                    format!("{}={}", expected_value.field(), expected_value.value()),
                    format!("{}={}", actual_value.field(), actual_value.value()),
                ))));
            }
        }
        let payloads_match = match case.collection_policy() {
            CollectionPolicy::Set => {
                let mut left = expected.payload_ids().to_vec();
                let mut right = actual.payload_ids().to_vec();
                left.sort_unstable();
                right.sort_unstable();
                left == right
            }
            CollectionPolicy::Ordered => expected.payload_ids() == actual.payload_ids(),
            CollectionPolicy::Multiset => false,
        };
        if !payloads_match {
            return Err(CollisionDivergence::Order(Box::new(order_report(
                request,
                profile,
                &request_hash,
                index,
                "payload_ids",
                format!("{:?}", expected.payload_ids()),
                format!("{:?}", actual.payload_ids()),
            ))));
        }
    }
    Ok(())
}

fn harness(
    request: &CollisionProbeRequestRecord,
    profile: &Phase5PolicyProfile,
    request_hash: &Sha256Hex,
    reason: impl Into<Box<str>>,
    maybe_index: Option<usize>,
    expected: impl Into<Box<str>>,
    actual: impl Into<Box<str>>,
) -> CollisionDivergence {
    let reason = reason.into();
    let maybe_case_id = maybe_index
        .and_then(|index| request.scenario().cases().get(index))
        .map(|case| case.case_id().into());
    let signature = hash_bytes(
        format!(
            "{}|{}|{}|{}",
            request.request_id().as_str(),
            reason,
            maybe_case_id.as_deref().unwrap_or("<none>"),
            profile.profile_sha256().as_str()
        )
        .as_bytes(),
    );
    CollisionDivergence::Harness(Box::new(CollisionHarnessReport {
        signature_sha256: signature,
        reason,
        maybe_case_id,
        maybe_case_index: maybe_index.map(bounded_index),
        expected: expected.into(),
        actual: actual.into(),
        request_sha256: request_hash.clone(),
        profile_sha256: profile.profile_sha256().clone(),
        maybe_previous_case_id: neighbor(request, maybe_index, -1),
        maybe_next_case_id: neighbor(request, maybe_index, 1),
    }))
}

fn numeric_report(
    request: &CollisionProbeRequestRecord,
    profile: &Phase5PolicyProfile,
    request_hash: &Sha256Hex,
    index: usize,
    field: &str,
    expected: FloatBits,
    actual: FloatBits,
) -> CollisionNumericReport {
    let case = &request.scenario().cases()[index];
    let signature = hash_bytes(
        format!(
            "{}|{}|{}|{}",
            request.request_id().as_str(),
            case.case_id(),
            field,
            profile.profile_sha256().as_str()
        )
        .as_bytes(),
    );
    CollisionNumericReport {
        signature_sha256: signature,
        case_id: case.case_id().into(),
        case_index: bounded_index(index),
        field: field.into(),
        expected_bits: expected,
        actual_bits: actual,
        expected_decimal: expected.to_f32().to_string().into_boxed_str(),
        actual_decimal: actual.to_f32().to_string().into_boxed_str(),
        absolute_difference_bits: FloatBits::from_f32((expected.to_f32() - actual.to_f32()).abs()),
        policy_path: case.policy_path().into(),
        horizon: format!("{:?}", case.horizon())
            .to_ascii_lowercase()
            .into_boxed_str(),
        evidence_tier: "d2_supported".into(),
        request_sha256: request_hash.clone(),
        profile_sha256: profile.profile_sha256().clone(),
        maybe_previous_case_id: neighbor(request, Some(index), -1),
        maybe_next_case_id: neighbor(request, Some(index), 1),
    }
}

fn order_report(
    request: &CollisionProbeRequestRecord,
    profile: &Phase5PolicyProfile,
    request_hash: &Sha256Hex,
    index: usize,
    field: impl Into<Box<str>>,
    expected: impl Into<Box<str>>,
    actual: impl Into<Box<str>>,
) -> CollisionOrderReport {
    let case = &request.scenario().cases()[index];
    let field = field.into();
    let signature = hash_bytes(
        format!(
            "{}|{}|{}|{}",
            request.request_id().as_str(),
            case.case_id(),
            field,
            profile.profile_sha256().as_str()
        )
        .as_bytes(),
    );
    CollisionOrderReport {
        signature_sha256: signature,
        case_id: case.case_id().into(),
        case_index: bounded_index(index),
        field,
        expected: expected.into(),
        actual: actual.into(),
        policy_path: case.policy_path().into(),
        collection_policy: case.collection_policy(),
        request_sha256: request_hash.clone(),
        profile_sha256: profile.profile_sha256().clone(),
        maybe_previous_case_id: neighbor(request, Some(index), -1),
        maybe_next_case_id: neighbor(request, Some(index), 1),
    }
}

fn neighbor(
    request: &CollisionProbeRequestRecord,
    maybe_index: Option<usize>,
    offset: isize,
) -> Option<Box<str>> {
    let index = maybe_index?.checked_add_signed(offset)?;
    request
        .scenario()
        .cases()
        .get(index)
        .map(|case| case.case_id().into())
}

fn bounded_index(index: usize) -> u32 {
    u32::try_from(index).unwrap_or(u32::MAX)
}
fn hash_serialized(value: &impl Serialize) -> Sha256Hex {
    hash_bytes(&serde_json::to_vec(value).expect("validated protocol records serialize"))
}
fn hash_bytes(bytes: &[u8]) -> Sha256Hex {
    Sha256Hex::from_digest(Sha256::digest(bytes).into())
}
