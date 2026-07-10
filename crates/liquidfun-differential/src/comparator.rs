use std::time::Duration;

use liquidfun_test_protocol::{
    CollectionPolicy, DiscretePolicy, EngineKind, FloatBits, FloatPolicy, HarnessFailure,
    HarnessFailureEvidence, HarnessFailureKind, HarnessLimits, StderrEvidence, ToleranceProfile,
    ValidatedTrace, WorldCounts,
};

/// Stable broad category of a semantic mismatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MismatchKind {
    /// An expected semantic value was absent.
    Missing,
    /// An unrequested semantic value was present.
    Unexpected,
    /// An exact discrete value differed.
    Exact,
    /// A floating value violated its field policy.
    Numeric,
    /// Solver-significant order differed.
    Order,
    /// An explicitly unordered multiset had different multiplicity.
    Multiplicity,
}

/// Typed semantic mismatch produced only after trace compatibility succeeds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MismatchReport {
    kind: MismatchKind,
}

impl MismatchReport {
    fn new(kind: MismatchKind) -> Self {
        Self { kind }
    }

    /// Returns the semantic mismatch category.
    #[must_use]
    pub const fn kind(&self) -> MismatchKind {
        self.kind
    }
}

/// Complete differential result for two compatible validated traces.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    for (expected_checkpoint, actual_checkpoint) in
        expected_checkpoints.iter().zip(actual_checkpoints)
    {
        if !exact_values_match(
            expected_checkpoint.checkpoint_id(),
            actual_checkpoint.checkpoint_id(),
        ) || !exact_values_match(&expected_checkpoint.ordinal(), &actual_checkpoint.ordinal())
            || !exact_values_match(&expected_checkpoint.phase(), &actual_checkpoint.phase())
        {
            return Ok(physics_mismatch(MismatchKind::Order));
        }
        if !world_counts_match(
            expected_checkpoint.world_counts(),
            actual_checkpoint.world_counts(),
            policy.world_counts(),
        ) {
            return Ok(physics_mismatch(MismatchKind::Exact));
        }
        if !float_values_match(
            expected_checkpoint.simulation_time_bits(),
            actual_checkpoint.simulation_time_bits(),
            policy.simulation_time(),
        ) {
            return Ok(physics_mismatch(MismatchKind::Numeric));
        }
    }

    if expected_checkpoints.len() > actual_checkpoints.len() {
        return Ok(physics_mismatch(MismatchKind::Missing));
    }
    if expected_checkpoints.len() < actual_checkpoints.len() {
        return Ok(physics_mismatch(MismatchKind::Unexpected));
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

fn world_counts_match(expected: WorldCounts, actual: WorldCounts, policy: DiscretePolicy) -> bool {
    match policy {
        DiscretePolicy::Exact => {
            expected.bodies() == actual.bodies()
                && expected.fixtures() == actual.fixtures()
                && expected.joints() == actual.joints()
                && expected.contacts() == actual.contacts()
                && expected.particle_systems() == actual.particle_systems()
                && expected.particle_groups() == actual.particle_groups()
                && expected.particles() == actual.particles()
        }
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

fn physics_mismatch(kind: MismatchKind) -> DifferentialOutcome {
    DifferentialOutcome::PhysicsMismatch(MismatchReport::new(kind))
}

fn harness_failure(kind: HarnessFailureKind) -> HarnessFailure {
    let limits = HarnessLimits::phase2_default_v1();
    let stderr = StderrEvidence::new(Vec::new(), 0, &limits)
        .expect("empty stderr is always within the reviewed limit");
    let evidence = HarnessFailureEvidence::new(Duration::ZERO, stderr, false, false, &limits);
    HarnessFailure::new(kind, evidence)
}
