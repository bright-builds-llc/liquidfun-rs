use std::time::Duration;

use liquidfun_test_protocol::{
    CollectionPolicy, DiscretePolicy, EngineKind, FieldComparison, FieldPolicy, FloatBits,
    FloatPolicy, HarnessFailure, HarnessFailureEvidence, HarnessFailureKind, HarnessLimits,
    NonFinitePolicy, StderrEvidence, ToleranceProfile, ValidatedTrace, WorldCounts, ZeroPolicy,
};

use crate::{MismatchKind, MismatchReport, SemanticPath, WorldCountField};

/// Complete differential result for two compatible validated traces.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(
    clippy::large_enum_variant,
    reason = "the private result keeps owned mismatch evidence allocation-free on the error path"
)]
pub enum DifferentialOutcome {
    /// Every typed semantic observable matched its reviewed policy.
    Match,
    /// Compatible traces differed in physics-visible semantics.
    PhysicsMismatch(MismatchReport),
}

/// Compares two complete validated traces under one reviewed typed profile.
///
/// # Errors
///
/// Returns a harness failure before observing values when the trace contracts are incompatible.
pub fn compare(
    expected: &ValidatedTrace,
    actual: &ValidatedTrace,
    policy: &ToleranceProfile,
) -> Result<DifferentialOutcome, HarnessFailure> {
    ensure_compatible(expected, actual, policy)?;

    let expected_checkpoints = expected.checkpoints();
    let actual_checkpoints = actual.checkpoints();
    for (checkpoint_index, (expected_checkpoint, actual_checkpoint)) in expected_checkpoints
        .iter()
        .zip(actual_checkpoints)
        .enumerate()
    {
        if !exact_values_match(
            expected_checkpoint.checkpoint_id(),
            actual_checkpoint.checkpoint_id(),
        ) {
            return Ok(physics_mismatch(MismatchReport::discrete(
                expected,
                actual,
                checkpoint_index,
                SemanticPath::CheckpointId,
                MismatchKind::Order,
                policy,
            )));
        }
        if !exact_values_match(&expected_checkpoint.ordinal(), &actual_checkpoint.ordinal()) {
            return Ok(physics_mismatch(MismatchReport::discrete(
                expected,
                actual,
                checkpoint_index,
                SemanticPath::CheckpointOrdinal,
                MismatchKind::Order,
                policy,
            )));
        }
        if !exact_values_match(&expected_checkpoint.phase(), &actual_checkpoint.phase()) {
            return Ok(physics_mismatch(MismatchReport::discrete(
                expected,
                actual,
                checkpoint_index,
                SemanticPath::Phase,
                MismatchKind::Order,
                policy,
            )));
        }
        if let Some(field) = first_world_count_difference(
            expected_checkpoint.world_counts(),
            actual_checkpoint.world_counts(),
            policy.world_counts(),
        ) {
            return Ok(physics_mismatch(MismatchReport::discrete(
                expected,
                actual,
                checkpoint_index,
                SemanticPath::WorldCount(field),
                MismatchKind::Exact,
                policy,
            )));
        }
        if !float_values_match(
            expected_checkpoint.simulation_time_bits(),
            actual_checkpoint.simulation_time_bits(),
            policy.simulation_time(),
        ) {
            return Ok(physics_mismatch(MismatchReport::numeric(
                expected,
                actual,
                checkpoint_index,
                expected_checkpoint.simulation_time_bits(),
                actual_checkpoint.simulation_time_bits(),
                policy,
            )));
        }
    }

    if expected_checkpoints.len() > actual_checkpoints.len() {
        return Ok(physics_mismatch(MismatchReport::discrete(
            expected,
            actual,
            actual_checkpoints.len(),
            SemanticPath::CheckpointPresence,
            MismatchKind::Missing,
            policy,
        )));
    }
    if expected_checkpoints.len() < actual_checkpoints.len() {
        return Ok(physics_mismatch(MismatchReport::discrete(
            actual,
            expected,
            expected_checkpoints.len(),
            SemanticPath::CheckpointPresence,
            MismatchKind::Unexpected,
            policy,
        )));
    }

    Ok(DifferentialOutcome::Match)
}

fn ensure_compatible(
    expected: &ValidatedTrace,
    actual: &ValidatedTrace,
    policy: &ToleranceProfile,
) -> Result<(), HarnessFailure> {
    if expected.protocol_version() != actual.protocol_version()
        || expected.trace_schema_version() != actual.trace_schema_version()
        || expected.tolerance_profile_version() != actual.tolerance_profile_version()
        || expected.tolerance_profile_version() != policy.version()
    {
        return Err(harness_failure(HarnessFailureKind::UnsupportedVersion));
    }
    if expected.tolerance_profile_sha256() != actual.tolerance_profile_sha256()
        || expected.tolerance_profile_sha256() != policy.profile_sha256()
        || policy.checkpoints() != CollectionPolicy::Ordered
    {
        return Err(harness_failure(HarnessFailureKind::WrongProvenance));
    }
    if expected.request_id() != actual.request_id() {
        return Err(harness_failure(HarnessFailureKind::RequestIdMismatch));
    }
    if expected.scenario_id() != actual.scenario_id()
        || expected.scenario_sha256() != actual.scenario_sha256()
        || expected.source() != actual.source()
    {
        return Err(harness_failure(HarnessFailureKind::TraceIdentityMismatch));
    }
    if expected.engine_kind() != EngineKind::CppOracle
        || actual.engine_kind() != EngineKind::NativeRust
    {
        return Err(harness_failure(HarnessFailureKind::WrongProvenance));
    }
    Ok(())
}

/// Compares one exact typed value without coercion.
#[must_use]
pub fn exact_values_match<T: PartialEq + ?Sized>(expected: &T, actual: &T) -> bool {
    expected == actual
}

/// Compares collection members without order while preserving every occurrence.
#[must_use]
pub fn multiset_values_match<T: PartialEq>(expected: &[T], actual: &[T]) -> bool {
    if expected.len() != actual.len() {
        return false;
    }
    let mut matched = vec![false; actual.len()];
    expected.iter().all(|expected_value| {
        let Some(index) = actual.iter().enumerate().find_map(|(index, actual_value)| {
            (!matched[index] && expected_value == actual_value).then_some(index)
        }) else {
            return false;
        };
        matched[index] = true;
        true
    })
}

/// Compares unique collection membership without observing traversal order.
#[must_use]
pub fn set_values_match<T: PartialEq>(expected: &[T], actual: &[T]) -> bool {
    expected
        .iter()
        .all(|expected_value| actual.contains(expected_value))
        && actual
            .iter()
            .all(|actual_value| expected.contains(actual_value))
}

fn first_world_count_difference(
    expected: WorldCounts,
    actual: WorldCounts,
    policy: DiscretePolicy,
) -> Option<WorldCountField> {
    match policy {
        DiscretePolicy::Exact => [
            (WorldCountField::Bodies, expected.bodies(), actual.bodies()),
            (
                WorldCountField::Fixtures,
                expected.fixtures(),
                actual.fixtures(),
            ),
            (WorldCountField::Joints, expected.joints(), actual.joints()),
            (
                WorldCountField::Contacts,
                expected.contacts(),
                actual.contacts(),
            ),
            (
                WorldCountField::ParticleSystems,
                expected.particle_systems(),
                actual.particle_systems(),
            ),
            (
                WorldCountField::ParticleGroups,
                expected.particle_groups(),
                actual.particle_groups(),
            ),
            (
                WorldCountField::Particles,
                expected.particles(),
                actual.particles(),
            ),
        ]
        .into_iter()
        .find_map(|(field, expected, actual)| (expected != actual).then_some(field)),
    }
}

/// Applies one exhaustive authoritative floating-point field policy.
#[must_use]
pub fn float_values_match(expected: FloatBits, actual: FloatBits, policy: FloatPolicy) -> bool {
    if matches!(policy, FloatPolicy::ExactBits) {
        return expected == actual;
    }

    let expected_value = expected.to_f32();
    let actual_value = actual.to_f32();
    if expected_value.is_nan() || actual_value.is_nan() {
        return false;
    }
    if expected == actual {
        return true;
    }
    if expected_value == 0.0 && actual_value == 0.0 {
        return false;
    }
    if expected_value.is_infinite() || actual_value.is_infinite() {
        return false;
    }

    match policy {
        FloatPolicy::ExactBits => expected == actual,
        FloatPolicy::Absolute { max_bits } => valid_threshold(max_bits)
            .is_some_and(|maximum| (expected_value - actual_value).abs() <= maximum),
        FloatPolicy::AbsoluteRelative {
            absolute_bits,
            relative_bits,
        } => {
            let Some(absolute) = valid_threshold(absolute_bits) else {
                return false;
            };
            let Some(relative) = valid_threshold(relative_bits) else {
                return false;
            };
            let difference = (expected_value - actual_value).abs();
            difference <= absolute
                || difference <= relative * expected_value.abs().max(actual_value.abs())
        }
        FloatPolicy::Ulps { max } => ulp_distance(expected.bits(), actual.bits()) <= max,
    }
}

/// Applies one complete reviewed Phase 4 field policy.
#[must_use]
pub fn float_values_match_with_policy(
    expected: FloatBits,
    actual: FloatBits,
    field_policy: &FieldPolicy,
) -> bool {
    let FieldComparison::Float { policy } = field_policy.comparison() else {
        return false;
    };
    let expected_value = expected.to_f32();
    let actual_value = actual.to_f32();

    if expected_value.is_nan() || actual_value.is_nan() {
        return field_policy.non_finite_policy() == NonFinitePolicy::ExactBitsTransport
            && expected == actual;
    }
    if expected_value.is_infinite() || actual_value.is_infinite() {
        return match field_policy.non_finite_policy() {
            NonFinitePolicy::SameSignInfinity => {
                expected_value.is_infinite()
                    && actual_value.is_infinite()
                    && expected_value.is_sign_negative() == actual_value.is_sign_negative()
            }
            NonFinitePolicy::ExactBitsTransport => expected == actual,
            NonFinitePolicy::RejectArithmeticNaN => false,
        };
    }
    if expected_value == 0.0
        && actual_value == 0.0
        && field_policy.zero_policy() == ZeroPolicy::SignInsensitive
    {
        return true;
    }

    float_values_match(expected, actual, policy)
}

fn valid_threshold(bits: FloatBits) -> Option<f32> {
    let value = bits.to_f32();
    (value.is_finite() && value >= 0.0).then_some(value)
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

fn physics_mismatch(report: MismatchReport) -> DifferentialOutcome {
    DifferentialOutcome::PhysicsMismatch(report)
}

fn harness_failure(kind: HarnessFailureKind) -> HarnessFailure {
    let limits = HarnessLimits::phase2_default_v1();
    let stderr = StderrEvidence::new(Vec::new(), 0, &limits)
        .expect("empty stderr is always within the reviewed limit");
    let evidence = HarnessFailureEvidence::new(Duration::ZERO, stderr, false, false, &limits);
    HarnessFailure::new(kind, evidence)
}
