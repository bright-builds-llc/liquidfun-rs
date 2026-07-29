//! Closed numeric policies and mismatch construction.

use super::{
    Digest, FloatBits, Location, PHASE9_ABSOLUTE_RELATIVE_ABSOLUTE,
    PHASE9_ABSOLUTE_RELATIVE_RELATIVE, PHASE9_MAX_ULPS, PHASE9_REGISTRY_ID, Phase9ComparatorError,
    Phase9Mismatch, Phase9ParticleObservation, Sha256, Sha256Hex, Vec2Bits, phase9_policy_for_path,
};

pub(super) fn compare_vec_ulps(
    location: Location,
    path: &'static str,
    expected: Vec2Bits,
    actual: Vec2Bits,
) -> Result<Option<Phase9Mismatch>, Phase9ComparatorError> {
    compare_ulps(location, path, expected.x_bits, actual.x_bits).and_then(|maybe| {
        maybe.map_or_else(
            || compare_ulps(location, path, expected.y_bits, actual.y_bits),
            |found| Ok(Some(found)),
        )
    })
}

pub(super) fn compare_vec_exact(
    location: Location,
    path: &'static str,
    expected: Vec2Bits,
    actual: Vec2Bits,
) -> Option<Phase9Mismatch> {
    bits_exact(location, path, expected.x_bits, actual.x_bits)
        .or_else(|| bits_exact(location, path, expected.y_bits, actual.y_bits))
}

pub(super) fn compare_ulps(
    location: Location,
    path: &'static str,
    expected: FloatBits,
    actual: FloatBits,
) -> Result<Option<Phase9Mismatch>, Phase9ComparatorError> {
    finite(path, expected, actual)?;
    Ok(
        (ulp_distance(expected.bits(), actual.bits()) > PHASE9_MAX_ULPS)
            .then(|| numeric_mismatch(location, path, expected, actual)),
    )
}

pub(super) fn compare_absolute_relative(
    location: Location,
    path: &'static str,
    expected: FloatBits,
    actual: FloatBits,
) -> Result<Option<Phase9Mismatch>, Phase9ComparatorError> {
    finite(path, expected, actual)?;
    let expected_value = expected.to_f32();
    let actual_value = actual.to_f32();
    let difference = (expected_value - actual_value).abs();
    let matches = difference <= PHASE9_ABSOLUTE_RELATIVE_ABSOLUTE
        || difference
            <= PHASE9_ABSOLUTE_RELATIVE_RELATIVE * expected_value.abs().max(actual_value.abs());
    Ok((!matches).then(|| numeric_mismatch(location, path, expected, actual)))
}

pub(super) fn compare_dimensioned(
    location: Location,
    path: &'static str,
    expected: FloatBits,
    actual: FloatBits,
    maximum: f32,
) -> Result<Option<Phase9Mismatch>, Phase9ComparatorError> {
    finite(path, expected, actual)?;
    Ok(((expected.to_f32() - actual.to_f32()).abs() > maximum)
        .then(|| numeric_mismatch(location, path, expected, actual)))
}

pub(super) fn finite(
    path: &'static str,
    expected: FloatBits,
    actual: FloatBits,
) -> Result<(), Phase9ComparatorError> {
    if expected.to_f32().is_finite() && actual.to_f32().is_finite() {
        return Ok(());
    }
    Err(Phase9ComparatorError::NonFinite { path })
}

pub(super) fn exact<T: std::fmt::Debug + PartialEq>(
    location: Location,
    path: &'static str,
    expected: &T,
    actual: &T,
) -> Option<Phase9Mismatch> {
    (expected != actual).then(|| {
        mismatch(
            location,
            path,
            format!("{expected:?}"),
            format!("{actual:?}"),
            None,
        )
    })
}

pub(super) fn bits_exact(
    location: Location,
    path: &'static str,
    expected: FloatBits,
    actual: FloatBits,
) -> Option<Phase9Mismatch> {
    (expected != actual).then(|| numeric_mismatch(location, path, expected, actual))
}

pub(super) fn numeric_mismatch(
    location: Location,
    path: &'static str,
    expected: FloatBits,
    actual: FloatBits,
) -> Phase9Mismatch {
    mismatch(
        location,
        path,
        format!("0x{:08x}", expected.bits()),
        format!("0x{:08x}", actual.bits()),
        Some((expected, actual)),
    )
}

pub(super) fn mismatch(
    location: Location,
    path: &'static str,
    expected: String,
    actual: String,
    maybe_bits: Option<(FloatBits, FloatBits)>,
) -> Phase9Mismatch {
    let kind = phase9_policy_for_path(path)
        .expect("every comparator path is a closed reviewed Phase 9 policy");
    let input = format!(
        "{PHASE9_REGISTRY_ID}\0{}\0{}\0{}\0{path}\0{kind:?}\0{expected}\0{actual}\0{:?}",
        location.timeline, location.checkpoint, location.observation, maybe_bits,
    );
    Phase9Mismatch {
        signature_sha256: Sha256Hex::from_digest(Sha256::digest(input.as_bytes()).into()),
        timeline_index: location.timeline,
        checkpoint_index: location.checkpoint,
        observation_index: location.observation,
        semantic_path: path,
        kind,
        expected: expected.into(),
        actual: actual.into(),
        maybe_expected_bits: maybe_bits.map(|bits| bits.0),
        maybe_actual_bits: maybe_bits.map(|bits| bits.1),
    }
}

pub(super) fn policy_error(reason: String) -> Phase9ComparatorError {
    Phase9ComparatorError::PolicyRegistry {
        reason: reason.into(),
    }
}

pub(super) const fn observation_kind(observation: &Phase9ParticleObservation) -> &'static str {
    match observation {
        Phase9ParticleObservation::System { .. } => "system",
        Phase9ParticleObservation::Particle { .. } => "particle",
        Phase9ParticleObservation::Lifecycle { .. } => "lifecycle",
        Phase9ParticleObservation::ParticleContact { .. } => "particle_contact",
        Phase9ParticleObservation::BodyContact { .. } => "body_contact",
        Phase9ParticleObservation::Statistics { .. } => "statistics",
        Phase9ParticleObservation::Query { .. } => "query",
        Phase9ParticleObservation::RayCast { .. } => "ray_cast",
        Phase9ParticleObservation::MixedState { .. } => "mixed_state",
    }
}

pub(super) fn ulp_distance(left: u32, right: u32) -> u32 {
    ordered_float_bits(left).abs_diff(ordered_float_bits(right))
}

pub(super) const fn ordered_float_bits(bits: u32) -> u32 {
    if bits & 0x8000_0000 == 0 {
        bits | 0x8000_0000
    } else {
        !bits
    }
}
