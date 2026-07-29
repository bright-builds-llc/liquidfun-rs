use liquidfun_test_protocol::{
    CollectionPolicy, DivergenceHorizon, EvidenceTier, FieldComparison, FieldPolicy, FloatBits,
    MathProbeFloatClass, MathProbeOperation, MathProbePolicyPath, MathProbeRequestRecord,
    MathProbeResult, MathProbeValue, MathProbeValueField, NonFinitePolicy, RequestId, ScenarioId,
    Sha256Hex, ToleranceProfileVersion, ZeroPolicy,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::ReportRenderError;

/// Exact authoritative bits plus derived, non-authoritative decimal diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FloatMismatchEvidence {
    expected_bits: FloatBits,
    actual_bits: FloatBits,
    expected_decimal: Box<str>,
    actual_decimal: Box<str>,
    expected_class: FloatClass,
    actual_class: FloatClass,
    absolute_difference_bits: FloatBits,
    relative_difference_bits: FloatBits,
    ulp_distance: u32,
}

impl FloatMismatchEvidence {
    pub(super) fn new(expected_bits: FloatBits, actual_bits: FloatBits) -> Self {
        let expected = expected_bits.to_f32();
        let actual = actual_bits.to_f32();
        let (absolute_difference, relative_difference) = numeric_distances(expected, actual);
        Self {
            expected_bits,
            actual_bits,
            expected_decimal: expected.to_string().into_boxed_str(),
            actual_decimal: actual.to_string().into_boxed_str(),
            expected_class: FloatClass::classify(expected_bits),
            actual_class: FloatClass::classify(actual_bits),
            absolute_difference_bits: FloatBits::from_f32(absolute_difference),
            relative_difference_bits: FloatBits::from_f32(relative_difference),
            ulp_distance: ulp_distance(expected_bits.bits(), actual_bits.bits()),
        }
    }

    /// Returns authoritative expected bits.
    #[must_use]
    pub const fn expected_bits(&self) -> FloatBits {
        self.expected_bits
    }

    /// Returns authoritative actual bits.
    #[must_use]
    pub const fn actual_bits(&self) -> FloatBits {
        self.actual_bits
    }

    /// Returns the derived expected decimal diagnostic.
    #[must_use]
    pub fn expected_decimal(&self) -> &str {
        &self.expected_decimal
    }

    /// Returns the derived actual decimal diagnostic.
    #[must_use]
    pub fn actual_decimal(&self) -> &str {
        &self.actual_decimal
    }

    /// Returns the expected IEEE-754 class and sign.
    #[must_use]
    pub const fn expected_class(&self) -> FloatClass {
        self.expected_class
    }

    /// Returns the actual IEEE-754 class and sign.
    #[must_use]
    pub const fn actual_class(&self) -> FloatClass {
        self.actual_class
    }

    /// Returns the non-authoritative absolute difference bits.
    #[must_use]
    pub const fn absolute_difference_bits(&self) -> FloatBits {
        self.absolute_difference_bits
    }

    /// Returns the non-authoritative relative difference bits.
    #[must_use]
    pub const fn relative_difference_bits(&self) -> FloatBits {
        self.relative_difference_bits
    }

    /// Returns the ordered representable-value distance.
    #[must_use]
    pub const fn ulp_distance(&self) -> u32 {
        self.ulp_distance
    }
}

/// Stable IEEE-754 class and sign diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FloatClass {
    /// Positive zero.
    PositiveZero,
    /// Negative zero.
    NegativeZero,
    /// Positive subnormal finite value.
    PositiveSubnormal,
    /// Negative subnormal finite value.
    NegativeSubnormal,
    /// Positive normal finite value.
    PositiveNormal,
    /// Negative normal finite value.
    NegativeNormal,
    /// Positive infinity.
    PositiveInfinity,
    /// Negative infinity.
    NegativeInfinity,
    /// Any NaN bit pattern.
    Nan,
}

/// Typed first-divergence evidence for the Phase 4 pure-math corpus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Phase4MathMismatchReport {
    signature_sha256: Sha256Hex,
    request_id: RequestId,
    request_sha256: Sha256Hex,
    scenario_id: ScenarioId,
    scenario_sha256: Sha256Hex,
    case_id: Box<str>,
    case_index: u32,
    operation: MathProbeOperation,
    policy_path: MathProbePolicyPath,
    value_field: MathProbeValueField,
    actual_value_field: MathProbeValueField,
    policy_id: Box<str>,
    policy_version: ToleranceProfileVersion,
    policy_sha256: Sha256Hex,
    comparison: FieldComparison,
    zero_policy: ZeroPolicy,
    non_finite_policy: NonFinitePolicy,
    collection_policy: CollectionPolicy,
    policy_justification: Box<str>,
    horizon: DivergenceHorizon,
    evidence_tier: EvidenceTier,
    oracle_build_sha256: Sha256Hex,
    native_build_sha256: Sha256Hex,
    expected_class: MathProbeFloatClass,
    actual_class: MathProbeFloatClass,
    expected_negative: bool,
    actual_negative: bool,
    float_evidence: FloatMismatchEvidence,
    maybe_previous_case_id: Option<Box<str>>,
    maybe_next_case_id: Option<Box<str>>,
}

#[derive(Serialize)]
struct MathMismatchSignatureInput<'a> {
    request_id: &'a str,
    scenario_id: &'a str,
    case_id: &'a str,
    operation: MathProbeOperation,
    policy_path: MathProbePolicyPath,
    value_field: MathProbeValueField,
    policy_sha256: &'a str,
}

impl Phase4MathMismatchReport {
    /// Builds bounded typed evidence for the first numeric math-probe divergence.
    ///
    /// # Errors
    ///
    /// Returns [`ReportRenderError`] if canonical request, scenario, or signature serialization
    /// fails.
    ///
    /// # Panics
    ///
    /// Panics only if a validated, protocol-bounded case index cannot fit in `u32`.
    #[allow(
        clippy::too_many_arguments,
        reason = "the report binds the compared values, policy, tier, and both builds"
    )]
    pub fn new(
        request: &MathProbeRequestRecord,
        expected_result: &MathProbeResult,
        case_index: usize,
        expected_value: MathProbeValue,
        actual_value: MathProbeValue,
        policy_id: &str,
        policy_version: ToleranceProfileVersion,
        policy_sha256: &Sha256Hex,
        field_policy: &FieldPolicy,
        evidence_tier: EvidenceTier,
        oracle_build_sha256: &Sha256Hex,
        native_build_sha256: &Sha256Hex,
    ) -> Result<Self, ReportRenderError> {
        let request_bytes = serde_json::to_vec(request).map_err(ReportRenderError)?;
        let scenario_bytes = serde_json::to_vec(request.scenario()).map_err(ReportRenderError)?;
        let signature = MathMismatchSignatureInput {
            request_id: request.request_id().as_str(),
            scenario_id: request.scenario().scenario_id().as_str(),
            case_id: expected_result.case_id(),
            operation: expected_result.operation(),
            policy_path: expected_result.policy_path(),
            value_field: expected_value.field(),
            policy_sha256: policy_sha256.as_str(),
        };
        let signature_bytes = serde_json::to_vec(&signature).map_err(ReportRenderError)?;
        let case_index_u32 =
            u32::try_from(case_index).expect("validated bounded math probe case index fits in u32");
        Ok(Self {
            signature_sha256: Sha256Hex::from_digest(Sha256::digest(signature_bytes).into()),
            request_id: request.request_id().clone(),
            request_sha256: Sha256Hex::from_digest(Sha256::digest(request_bytes).into()),
            scenario_id: request.scenario().scenario_id().clone(),
            scenario_sha256: Sha256Hex::from_digest(Sha256::digest(scenario_bytes).into()),
            case_id: expected_result.case_id().into(),
            case_index: case_index_u32,
            operation: expected_result.operation(),
            policy_path: expected_result.policy_path(),
            value_field: expected_value.field(),
            actual_value_field: actual_value.field(),
            policy_id: policy_id.into(),
            policy_version,
            policy_sha256: policy_sha256.clone(),
            comparison: field_policy.comparison(),
            zero_policy: field_policy.zero_policy(),
            non_finite_policy: field_policy.non_finite_policy(),
            collection_policy: field_policy.collection_policy(),
            policy_justification: field_policy.justification().into(),
            horizon: field_policy.horizon(),
            evidence_tier,
            oracle_build_sha256: oracle_build_sha256.clone(),
            native_build_sha256: native_build_sha256.clone(),
            expected_class: expected_value.class(),
            actual_class: actual_value.class(),
            expected_negative: expected_value.is_negative(),
            actual_negative: actual_value.is_negative(),
            float_evidence: FloatMismatchEvidence::new(expected_value.bits(), actual_value.bits()),
            maybe_previous_case_id: case_index
                .checked_sub(1)
                .and_then(|index| request.scenario().cases().get(index))
                .map(|case| case.case_id().into()),
            maybe_next_case_id: request
                .scenario()
                .cases()
                .get(case_index.saturating_add(1))
                .map(|case| case.case_id().into()),
        })
    }

    /// Returns the stable first-divergence signature.
    #[must_use]
    pub const fn signature_sha256(&self) -> &Sha256Hex {
        &self.signature_sha256
    }

    /// Returns the policy-declared comparison horizon.
    #[must_use]
    pub const fn horizon(&self) -> DivergenceHorizon {
        self.horizon
    }

    /// Returns the effective authority tier of both compared builds.
    #[must_use]
    pub const fn evidence_tier(&self) -> EvidenceTier {
        self.evidence_tier
    }

    /// Returns the exact mismatched case ID.
    #[must_use]
    pub fn case_id(&self) -> &str {
        &self.case_id
    }

    /// Returns the exact mismatched scalar field.
    #[must_use]
    pub const fn value_field(&self) -> MathProbeValueField {
        self.value_field
    }

    /// Renders deterministic machine evidence.
    ///
    /// # Errors
    ///
    /// Returns [`ReportRenderError`] if serialization fails.
    pub fn render_machine(&self) -> Result<Vec<u8>, ReportRenderError> {
        serde_json::to_vec(self).map_err(ReportRenderError)
    }

    /// Renders a concise diagnostic derived from the typed record.
    #[must_use]
    pub fn render_human(&self) -> String {
        format!(
            "numeric mismatch at case {} field {:?}: expected {:#010x}, actual {:#010x}; policy {}, horizon {:?}, tier {:?}; signature {}",
            self.case_id,
            self.value_field,
            self.float_evidence.expected_bits().bits(),
            self.float_evidence.actual_bits().bits(),
            self.policy_id,
            self.horizon,
            self.evidence_tier,
            self.signature_sha256.as_str(),
        )
    }
}

impl FloatClass {
    fn classify(bits: FloatBits) -> Self {
        let value = bits.to_f32();
        if value.is_nan() {
            return Self::Nan;
        }
        if value == 0.0 {
            return if value.is_sign_negative() {
                Self::NegativeZero
            } else {
                Self::PositiveZero
            };
        }
        if value.is_infinite() {
            return if value.is_sign_negative() {
                Self::NegativeInfinity
            } else {
                Self::PositiveInfinity
            };
        }
        if value.is_subnormal() {
            return if value.is_sign_negative() {
                Self::NegativeSubnormal
            } else {
                Self::PositiveSubnormal
            };
        }
        if value.is_sign_negative() {
            Self::NegativeNormal
        } else {
            Self::PositiveNormal
        }
    }
}

fn numeric_distances(expected: f32, actual: f32) -> (f32, f32) {
    if !expected.is_finite() || !actual.is_finite() {
        return (f32::INFINITY, f32::INFINITY);
    }
    let absolute = (expected - actual).abs();
    let scale = expected.abs().max(actual.abs());
    let relative = if scale == 0.0 { 0.0 } else { absolute / scale };
    (absolute, relative)
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

#[cfg(test)]
mod tests {
    use liquidfun_test_protocol::{
        DivergenceHorizon, EvidenceTier, FloatBits, HarnessLimits, MathProbeHorizon,
        MathProbeValue, Phase4PolicyProfile, Sha256Hex, decode_math_probe_request_jsonl,
    };

    use crate::NativeMathProbeExecutor;

    use super::Phase4MathMismatchReport;

    #[test]
    fn phase4_math_reports_preserve_tiers_and_exact_policy_horizons() {
        // Arrange
        let request = decode_math_probe_request_jsonl(
            include_bytes!("../../../../protocol/fixtures/accepted/math-probe-request.jsonl"),
            &HarnessLimits::phase2_default_v1(),
        )
        .expect("checked-in math request should decode");
        let policy = Phase4PolicyProfile::parse_toml(include_str!(
            "../../../../protocol/tolerances/phase4-v1.toml"
        ))
        .expect("checked-in policy should parse");
        let results = NativeMathProbeExecutor::execute(&request)
            .expect("checked-in math request should execute");
        let oracle_hash = Sha256Hex::new("11".repeat(32)).expect("fixture hash is valid");
        let native_hash = Sha256Hex::new("22".repeat(32)).expect("fixture hash is valid");
        let expectations = [
            (
                MathProbeHorizon::Operation,
                DivergenceHorizon::Operation,
                EvidenceTier::D1Canonical,
            ),
            (
                MathProbeHorizon::ScenarioSteps { steps: 4 },
                DivergenceHorizon::ScenarioSteps { steps: 4 },
                EvidenceTier::D2Supported,
            ),
            (
                MathProbeHorizon::ScenarioSteps { steps: 32 },
                DivergenceHorizon::ScenarioSteps { steps: 32 },
                EvidenceTier::D3Exploratory,
            ),
        ];

        for (request_horizon, policy_horizon, tier) in expectations {
            let case_index = results
                .iter()
                .position(|result| {
                    result.horizon() == request_horizon && !result.values().is_empty()
                })
                .expect("fixture should cover each required horizon");
            let expected_result = &results[case_index];
            let expected_value = *expected_result
                .values()
                .first()
                .expect("math result should contain a scalar value");
            let actual_value = MathProbeValue::new(
                expected_value.field(),
                FloatBits::new(expected_value.bits().bits().wrapping_add(1)),
            );
            let field_policy = policy
                .field(expected_result.policy_path().as_str())
                .expect("every request path should have a policy");

            // Act
            let first = Phase4MathMismatchReport::new(
                &request,
                expected_result,
                case_index,
                expected_value,
                actual_value,
                policy.profile_id(),
                policy.version(),
                policy.profile_sha256(),
                field_policy,
                tier,
                &oracle_hash,
                &native_hash,
            )
            .expect("typed mismatch evidence should build");
            let second = Phase4MathMismatchReport::new(
                &request,
                expected_result,
                case_index,
                expected_value,
                actual_value,
                policy.profile_id(),
                policy.version(),
                policy.profile_sha256(),
                field_policy,
                tier,
                &oracle_hash,
                &native_hash,
            )
            .expect("the same mismatch evidence should build again");

            // Assert
            assert_eq!(first.horizon(), policy_horizon);
            assert_eq!(first.evidence_tier(), tier);
            assert_eq!(first.signature_sha256(), second.signature_sha256());
            assert_eq!(first.case_id(), expected_result.case_id());
            assert_eq!(first.value_field(), expected_value.field());
            assert_eq!(first.oracle_build_sha256, oracle_hash);
            assert_eq!(first.native_build_sha256, native_hash);
            assert!(first.render_machine().expect("report should render").len() < 4096);
        }
    }
}
