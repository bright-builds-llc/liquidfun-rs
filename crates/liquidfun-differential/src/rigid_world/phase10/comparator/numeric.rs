//! Numeric policy evaluation and deterministic mismatch construction.

use liquidfun_test_protocol::{
    FloatBits, Phase10ValidationKind, Sha256Hex, TransformBits, Vec2Bits,
};
use sha2::{Digest, Sha256};

use super::{PHASE10_POLICY_REGISTRY, Phase10ComparatorError, Phase10Mismatch, Phase10PolicyKind};

const REGISTRY_ID: &str = "phase10-semantic-v1";

pub(super) fn numeric_transform(
    scenario: &str,
    entity: &str,
    index: usize,
    path: &'static str,
    expected: TransformBits,
    actual: TransformBits,
) -> Result<Option<Phase10Mismatch>, Phase10ComparatorError> {
    numeric_vec(
        scenario,
        entity,
        index,
        path,
        expected.position,
        actual.position,
    )?
    .map_or_else(
        || {
            numeric(
                scenario,
                entity,
                index,
                path,
                expected.angle_bits,
                actual.angle_bits,
            )
        },
        |found| Ok(Some(found)),
    )
}

pub(super) fn numeric_vec(
    scenario: &str,
    entity: &str,
    index: usize,
    path: &'static str,
    expected: Vec2Bits,
    actual: Vec2Bits,
) -> Result<Option<Phase10Mismatch>, Phase10ComparatorError> {
    numeric(
        scenario,
        entity,
        index.saturating_mul(2),
        path,
        expected.x_bits,
        actual.x_bits,
    )?
    .map_or_else(
        || {
            numeric(
                scenario,
                entity,
                index.saturating_mul(2).saturating_add(1),
                path,
                expected.y_bits,
                actual.y_bits,
            )
        },
        |found| Ok(Some(found)),
    )
}

pub(super) fn numeric(
    scenario: &str,
    entity: &str,
    index: usize,
    path: &'static str,
    expected: FloatBits,
    actual: FloatBits,
) -> Result<Option<Phase10Mismatch>, Phase10ComparatorError> {
    let policy = policy(path);
    let expected_value = expected.to_f32();
    let actual_value = actual.to_f32();
    if !expected_value.is_finite() || !actual_value.is_finite() {
        return Err(Phase10ComparatorError::ResultValidation {
            side: if expected_value.is_finite() {
                "actual"
            } else {
                "expected"
            },
            kind: Phase10ValidationKind::InvalidFloat,
        });
    }
    let matches = match policy {
        Phase10PolicyKind::ExactBits | Phase10PolicyKind::ExactDiscrete => expected == actual,
        Phase10PolicyKind::Ulps { maximum } => {
            ulp_distance(expected.bits(), actual.bits()) <= maximum
        }
        Phase10PolicyKind::AbsoluteRelative { absolute, relative } => {
            let difference = (expected_value - actual_value).abs();
            difference <= absolute
                || difference <= relative * expected_value.abs().max(actual_value.abs())
        }
        Phase10PolicyKind::DimensionedAbsolute { maximum } => {
            (expected_value - actual_value).abs() <= maximum
        }
    };
    Ok((!matches).then(|| {
        make_mismatch(
            scenario,
            "state",
            entity,
            index,
            path,
            format!("0x{:08x}", expected.bits()),
            format!("0x{:08x}", actual.bits()),
        )
    }))
}

pub(super) fn mismatch_if<T: std::fmt::Debug + PartialEq>(
    scenario: &str,
    operation: &'static str,
    entity: &str,
    index: usize,
    path: &'static str,
    expected: &T,
    actual: &T,
) -> Option<Phase10Mismatch> {
    (expected != actual).then(|| {
        make_mismatch(
            scenario,
            operation,
            entity,
            index,
            path,
            format!("{expected:?}"),
            format!("{actual:?}"),
        )
    })
}

fn make_mismatch(
    scenario: &str,
    operation: &'static str,
    entity: &str,
    index: usize,
    path: &'static str,
    expected: String,
    actual: String,
) -> Phase10Mismatch {
    let policy = policy(path);
    let input = format!(
        "{REGISTRY_ID}\0{scenario}\0{operation}\0{entity}\0{index}\0{path}\0{policy:?}\0{expected}\0{actual}"
    );
    Phase10Mismatch {
        signature_sha256: Sha256Hex::from_digest(Sha256::digest(input.as_bytes()).into()),
        semantic_path: path,
        policy,
        scenario: scenario.into(),
        operation,
        entity: entity.into(),
        index,
        expected: expected.into(),
        actual: actual.into(),
    }
}

fn policy(path: &'static str) -> Phase10PolicyKind {
    PHASE10_POLICY_REGISTRY
        .iter()
        .find(|candidate| candidate.path == path)
        .map(|candidate| candidate.kind)
        .expect("every comparator field has one closed Phase 10 policy")
}

fn ulp_distance(left: u32, right: u32) -> u32 {
    ordered_float_bits(left).abs_diff(ordered_float_bits(right))
}

const fn ordered_float_bits(bits: u32) -> u32 {
    if bits & 0x8000_0000 == 0 {
        bits | 0x8000_0000
    } else {
        !bits
    }
}
