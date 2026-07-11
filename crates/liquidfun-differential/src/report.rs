use std::fmt::Write;

use liquidfun_test_protocol::{
    BuildEvidenceTier, CheckpointId, CollectionPolicy, DivergenceHorizon, EvidenceTier,
    FieldComparison, FieldPolicy, FloatBits, HarnessFailure, MathProbeFloatClass,
    MathProbeOperation, MathProbePolicyPath, MathProbeRequestRecord, MathProbeResult,
    MathProbeValue, MathProbeValueField, NonFinitePolicy, RequestId, ScenarioId, Sha256Hex,
    ToleranceProfile, ToleranceProfileVersion, ValidatedTrace, ZeroPolicy,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

const MAXIMUM_PHASE_BYTES: usize = 4 * 1024;

/// Stable broad category of a semantic mismatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
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

/// Error returned when a report phase name violates validated trace bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("phase name must be nonempty and no longer than 4096 bytes")]
pub struct PhaseNameError;

/// Validated stable phase name carried by a failure signature.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct PhaseName(Box<str>);

impl PhaseName {
    /// Validates an owned phase name.
    ///
    /// # Errors
    ///
    /// Returns [`PhaseNameError`] for empty or oversized input.
    pub fn new(value: impl Into<String>) -> Result<Self, PhaseNameError> {
        let value = value.into();
        if value.is_empty() || value.len() > MAXIMUM_PHASE_BYTES {
            return Err(PhaseNameError);
        }
        Ok(Self(value.into_boxed_str()))
    }

    /// Returns the validated phase spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Exact world-count field selected by deterministic comparison traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorldCountField {
    /// Rigid-body count.
    Bodies,
    /// Fixture count.
    Fixtures,
    /// Joint count.
    Joints,
    /// Contact count.
    Contacts,
    /// Particle-system count.
    ParticleSystems,
    /// Particle-group count.
    ParticleGroups,
    /// Particle count.
    Particles,
}

/// Closed typed semantic paths supported by the Phase-2 trace schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticPath {
    /// Stable checkpoint identity.
    CheckpointId,
    /// Ordered checkpoint ordinal.
    CheckpointOrdinal,
    /// Named trace phase.
    Phase,
    /// One exact world-count field.
    WorldCount(WorldCountField),
    /// Exact-bit accumulated simulation time.
    SimulationTime,
    /// Presence of a whole checkpoint.
    CheckpointPresence,
}

impl SemanticPath {
    fn human_name(self) -> &'static str {
        match self {
            Self::CheckpointId => "checkpoint_id",
            Self::CheckpointOrdinal => "checkpoint_ordinal",
            Self::Phase => "phase",
            Self::WorldCount(WorldCountField::Bodies) => "world_counts.bodies",
            Self::WorldCount(WorldCountField::Fixtures) => "world_counts.fixtures",
            Self::WorldCount(WorldCountField::Joints) => "world_counts.joints",
            Self::WorldCount(WorldCountField::Contacts) => "world_counts.contacts",
            Self::WorldCount(WorldCountField::ParticleSystems) => "world_counts.particle_systems",
            Self::WorldCount(WorldCountField::ParticleGroups) => "world_counts.particle_groups",
            Self::WorldCount(WorldCountField::Particles) => "world_counts.particles",
            Self::SimulationTime => "simulation_time",
            Self::CheckpointPresence => "checkpoint_presence",
        }
    }
}

/// Stable first-divergence identity used by replay and reduction.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct FailureSignature {
    checkpoint_id: CheckpointId,
    phase: PhaseName,
    semantic_path: SemanticPath,
    kind: MismatchKind,
}

impl FailureSignature {
    /// Creates one complete typed failure identity.
    #[must_use]
    pub const fn new(
        checkpoint_id: CheckpointId,
        phase: PhaseName,
        semantic_path: SemanticPath,
        kind: MismatchKind,
    ) -> Self {
        Self {
            checkpoint_id,
            phase,
            semantic_path,
            kind,
        }
    }

    /// Returns the first divergent checkpoint identity.
    #[must_use]
    pub const fn checkpoint_id(&self) -> &CheckpointId {
        &self.checkpoint_id
    }

    /// Returns the first divergent named phase.
    #[must_use]
    pub const fn phase(&self) -> &PhaseName {
        &self.phase
    }

    /// Returns the deterministic typed semantic path.
    #[must_use]
    pub const fn semantic_path(&self) -> SemanticPath {
        self.semantic_path
    }

    /// Returns the mismatch category.
    #[must_use]
    pub const fn kind(&self) -> MismatchKind {
        self.kind
    }

    /// Returns an otherwise identical signature with a different mismatch kind.
    #[must_use]
    pub fn with_kind(mut self, kind: MismatchKind) -> Self {
        self.kind = kind;
        self
    }
}

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
    fn new(expected_bits: FloatBits, actual_bits: FloatBits) -> Self {
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

/// Machine-readable first divergence and bounded neighboring context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MismatchReport {
    signature: FailureSignature,
    checkpoint_ordinal: u32,
    maybe_previous_checkpoint_id: Option<CheckpointId>,
    maybe_next_checkpoint_id: Option<CheckpointId>,
    request_id: RequestId,
    request_sha256: Sha256Hex,
    scenario_sha256: Sha256Hex,
    policy_id: Box<str>,
    policy_sha256: Sha256Hex,
    horizon: DivergenceHorizon,
    evidence_tier: EvidenceTier,
    sibling_mismatch_count: u32,
    #[serde(rename = "float_evidence")]
    maybe_float_evidence: Option<FloatMismatchEvidence>,
}

impl MismatchReport {
    pub(crate) fn discrete(
        expected: &ValidatedTrace,
        actual: &ValidatedTrace,
        checkpoint_index: usize,
        semantic_path: SemanticPath,
        kind: MismatchKind,
        policy: &ToleranceProfile,
    ) -> Self {
        Self::new(
            expected,
            actual,
            checkpoint_index,
            semantic_path,
            kind,
            policy,
            DivergenceHorizon::PhaseLocal,
            None,
        )
    }

    pub(crate) fn numeric(
        expected: &ValidatedTrace,
        actual: &ValidatedTrace,
        checkpoint_index: usize,
        expected_bits: FloatBits,
        actual_bits: FloatBits,
        policy: &ToleranceProfile,
    ) -> Self {
        Self::new(
            expected,
            actual,
            checkpoint_index,
            SemanticPath::SimulationTime,
            MismatchKind::Numeric,
            policy,
            DivergenceHorizon::Unavailable,
            Some(FloatMismatchEvidence::new(expected_bits, actual_bits)),
        )
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "one constructor binds the full legacy first-divergence contract"
    )]
    fn new(
        expected: &ValidatedTrace,
        actual: &ValidatedTrace,
        checkpoint_index: usize,
        semantic_path: SemanticPath,
        kind: MismatchKind,
        policy: &ToleranceProfile,
        horizon: DivergenceHorizon,
        maybe_float_evidence: Option<FloatMismatchEvidence>,
    ) -> Self {
        let checkpoint = &expected.checkpoints()[checkpoint_index];
        let phase = PhaseName::new(checkpoint.phase())
            .expect("validated checkpoint phases satisfy report bounds");
        let signature = FailureSignature::new(
            checkpoint.checkpoint_id().clone(),
            phase,
            semantic_path,
            kind,
        );
        Self {
            signature,
            checkpoint_ordinal: checkpoint.ordinal(),
            maybe_previous_checkpoint_id: checkpoint_index
                .checked_sub(1)
                .and_then(|index| expected.checkpoints().get(index))
                .map(|checkpoint| checkpoint.checkpoint_id().clone()),
            maybe_next_checkpoint_id: expected
                .checkpoints()
                .get(checkpoint_index + 1)
                .map(|checkpoint| checkpoint.checkpoint_id().clone()),
            request_id: expected.request_id().clone(),
            request_sha256: request_contract_sha256(expected),
            scenario_sha256: expected.scenario_sha256().clone(),
            policy_id: policy.profile_id().into(),
            policy_sha256: policy.profile_sha256().clone(),
            horizon,
            evidence_tier: report_evidence_tier(
                horizon,
                expected.evidence_tier(),
                actual.evidence_tier(),
            ),
            sibling_mismatch_count: 0,
            maybe_float_evidence,
        }
    }

    /// Returns the stable reduction identity.
    #[must_use]
    pub const fn signature(&self) -> &FailureSignature {
        &self.signature
    }

    /// Returns the first divergent checkpoint ordinal.
    #[must_use]
    pub const fn checkpoint_ordinal(&self) -> u32 {
        self.checkpoint_ordinal
    }

    /// Returns the previous ordered checkpoint identity, when present.
    #[must_use]
    pub const fn maybe_previous_checkpoint_id(&self) -> Option<&CheckpointId> {
        self.maybe_previous_checkpoint_id.as_ref()
    }

    /// Returns the next ordered checkpoint identity, when present.
    #[must_use]
    pub const fn maybe_next_checkpoint_id(&self) -> Option<&CheckpointId> {
        self.maybe_next_checkpoint_id.as_ref()
    }

    /// Returns the stable request identity.
    #[must_use]
    pub const fn request_id(&self) -> &RequestId {
        &self.request_id
    }

    /// Returns a deterministic hash of the comparable request contract.
    #[must_use]
    pub const fn request_sha256(&self) -> &Sha256Hex {
        &self.request_sha256
    }

    /// Returns the canonical scenario hash.
    #[must_use]
    pub const fn scenario_sha256(&self) -> &Sha256Hex {
        &self.scenario_sha256
    }

    /// Returns the reviewed tolerance policy identity.
    #[must_use]
    pub const fn policy_sha256(&self) -> &Sha256Hex {
        &self.policy_sha256
    }

    /// Returns the fixed comparison horizon.
    #[must_use]
    pub const fn horizon(&self) -> DivergenceHorizon {
        self.horizon
    }

    /// Returns the authority tier for this comparison.
    #[must_use]
    pub const fn evidence_tier(&self) -> EvidenceTier {
        self.evidence_tier
    }

    /// Returns the bounded number of later mismatches summarized separately.
    #[must_use]
    pub const fn sibling_mismatch_count(&self) -> u32 {
        self.sibling_mismatch_count
    }

    /// Returns exact float evidence for numeric mismatches.
    #[must_use]
    pub const fn maybe_float_evidence(&self) -> Option<&FloatMismatchEvidence> {
        self.maybe_float_evidence.as_ref()
    }

    /// Renders deterministic parseable machine evidence.
    ///
    /// # Errors
    ///
    /// Returns [`ReportRenderError`] if typed report serialization fails.
    pub fn render_machine(&self) -> Result<Vec<u8>, ReportRenderError> {
        serde_json::to_vec(self).map_err(ReportRenderError)
    }

    /// Renders a concise human diagnostic from the same typed evidence.
    #[must_use]
    pub fn render_human(&self) -> String {
        let mut rendered = format!(
            "{:?} mismatch at checkpoint {} phase {} path {}",
            self.signature.kind,
            self.signature.checkpoint_id,
            self.signature.phase.as_str(),
            self.signature.semantic_path.human_name(),
        );
        if let Some(evidence) = &self.maybe_float_evidence {
            write!(
                rendered,
                ": expected 0x{:08x} ({}), actual 0x{:08x} ({})",
                evidence.expected_bits.bits(),
                evidence.expected_decimal,
                evidence.actual_bits.bits(),
                evidence.actual_decimal,
            )
            .expect("writing to an owned String cannot fail");
        }
        if let Some(evidence) = &self.maybe_float_evidence {
            write!(
                rendered,
                "; absolute 0x{:08x}, relative 0x{:08x}, ulps {}",
                evidence.absolute_difference_bits.bits(),
                evidence.relative_difference_bits.bits(),
                evidence.ulp_distance,
            )
            .expect("writing to an owned String cannot fail");
        }
        write!(
            rendered,
            "; policy {}, horizon {:?}, tier {:?}",
            self.policy_id, self.horizon, self.evidence_tier,
        )
        .expect("writing to an owned String cannot fail");
        rendered
    }
}

const fn weakest_build_evidence_tier(
    expected: BuildEvidenceTier,
    actual: BuildEvidenceTier,
) -> EvidenceTier {
    match (expected, actual) {
        (BuildEvidenceTier::D1Canonical, BuildEvidenceTier::D1Canonical) => {
            EvidenceTier::D1Canonical
        }
        (BuildEvidenceTier::D3Exploratory, _) | (_, BuildEvidenceTier::D3Exploratory) => {
            EvidenceTier::D3Exploratory
        }
        _ => EvidenceTier::D2Supported,
    }
}

const fn report_evidence_tier(
    horizon: DivergenceHorizon,
    expected: BuildEvidenceTier,
    actual: BuildEvidenceTier,
) -> EvidenceTier {
    if matches!(horizon, DivergenceHorizon::Unavailable) {
        return EvidenceTier::D3Exploratory;
    }
    weakest_build_evidence_tier(expected, actual)
}

#[cfg(test)]
mod tests {
    use liquidfun_test_protocol::{
        BuildEvidenceTier, DivergenceHorizon, EvidenceTier, FloatBits, HarnessLimits,
        MathProbeHorizon, MathProbeValue, Phase4PolicyProfile, Sha256Hex,
        decode_math_probe_request_jsonl,
    };

    use crate::NativeMathProbeExecutor;

    use super::{Phase4MathMismatchReport, report_evidence_tier, weakest_build_evidence_tier};

    #[test]
    fn report_evidence_tier_is_the_weakest_validated_build() {
        // Arrange / Act / Assert
        assert_eq!(
            weakest_build_evidence_tier(
                BuildEvidenceTier::D1Canonical,
                BuildEvidenceTier::D1Canonical
            ),
            EvidenceTier::D1Canonical
        );
        assert_eq!(
            weakest_build_evidence_tier(
                BuildEvidenceTier::D1Canonical,
                BuildEvidenceTier::D2Supported
            ),
            EvidenceTier::D2Supported
        );
        assert_eq!(
            weakest_build_evidence_tier(
                BuildEvidenceTier::D2Supported,
                BuildEvidenceTier::D3Exploratory
            ),
            EvidenceTier::D3Exploratory
        );
    }

    #[test]
    fn unavailable_horizon_forces_exploratory_authority() {
        // Arrange / Act / Assert
        assert_eq!(
            report_evidence_tier(
                DivergenceHorizon::Unavailable,
                BuildEvidenceTier::D1Canonical,
                BuildEvidenceTier::D1Canonical,
            ),
            EvidenceTier::D3Exploratory
        );
        assert_eq!(
            report_evidence_tier(
                DivergenceHorizon::Unavailable,
                BuildEvidenceTier::D1Canonical,
                BuildEvidenceTier::D2Supported,
            ),
            EvidenceTier::D3Exploratory
        );
    }

    #[test]
    fn phase4_math_reports_preserve_tiers_and_exact_policy_horizons() {
        // Arrange
        let request = decode_math_probe_request_jsonl(
            include_bytes!("../../../protocol/fixtures/accepted/math-probe-request.jsonl"),
            &HarnessLimits::phase2_default_v1(),
        )
        .expect("checked-in math request should decode");
        let policy = Phase4PolicyProfile::parse_toml(include_str!(
            "../../../protocol/tolerances/phase4-v1.toml"
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

/// Error produced while rendering deterministic machine evidence.
#[derive(Debug, thiserror::Error)]
#[error("mismatch report serialization failed: {0}")]
pub struct ReportRenderError(pub(crate) serde_json::Error);

/// Harness diagnostics remain a separate type from semantic mismatch reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessFailureReport {
    failure: HarnessFailure,
}

impl HarnessFailureReport {
    /// Wraps one classified non-physics harness failure.
    #[must_use]
    pub const fn new(failure: HarnessFailure) -> Self {
        Self { failure }
    }

    /// Returns the classified non-physics failure.
    #[must_use]
    pub const fn failure(&self) -> &HarnessFailure {
        &self.failure
    }
}

fn request_contract_sha256(trace: &ValidatedTrace) -> Sha256Hex {
    let mut hasher = Sha256::new();
    update_hash_field(&mut hasher, &trace.protocol_version().get().to_be_bytes());
    update_hash_field(&mut hasher, trace.request_id().as_str().as_bytes());
    update_hash_field(
        &mut hasher,
        &trace.trace_schema_version().get().to_be_bytes(),
    );
    update_hash_field(&mut hasher, trace.scenario_id().as_str().as_bytes());
    update_hash_field(&mut hasher, trace.scenario_sha256().as_str().as_bytes());
    update_hash_field(
        &mut hasher,
        &trace.tolerance_profile_version().get().to_be_bytes(),
    );
    update_hash_field(
        &mut hasher,
        trace.tolerance_profile_sha256().as_str().as_bytes(),
    );
    Sha256Hex::from_digest(hasher.finalize().into())
}

fn update_hash_field(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update(bytes.len().to_be_bytes());
    hasher.update(bytes);
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
