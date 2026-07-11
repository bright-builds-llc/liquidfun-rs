use liquidfun_test_protocol::{
    BuildIdentity, CollectionPolicy, DivergenceHorizon, EvidenceTier, FieldComparison, FieldPolicy,
    MathProbeDiscrete, MathProbeDiscreteField, MathProbeOperation, MathProbePolicyPath,
    MathProbeRequestRecord, MathProbeResult, NonFinitePolicy, RequestId, ScenarioId, Sha256Hex,
    ToleranceProfileVersion, ZeroPolicy,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{Phase4MathMismatchReport, ReportRenderError};

const MAXIMUM_CONTEXT_BYTES: usize = 512;

/// Closed typed evidence emitted by Phase 4 comparison failures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "evidence_kind", rename_all = "snake_case")]
pub enum Phase4ComparisonEvidence {
    /// A floating-point semantic divergence.
    NumericMismatch(Phase4MathMismatchReport),
    /// An exact discrete semantic divergence.
    DiscreteMismatch(Phase4DiscreteMismatchReport),
    /// A malformed result sequence or policy/configuration violation.
    HarnessFailure(Phase4HarnessFailureReport),
}

impl Phase4ComparisonEvidence {
    /// Renders deterministic bounded machine evidence.
    ///
    /// # Errors
    ///
    /// Returns [`ReportRenderError`] if serialization fails.
    pub fn render_machine(&self) -> Result<Vec<u8>, ReportRenderError> {
        serde_json::to_vec(self).map_err(ReportRenderError)
    }

    /// Renders a concise human diagnostic from the typed evidence.
    #[must_use]
    pub fn render_human(&self) -> String {
        match self {
            Self::NumericMismatch(report) => report.render_human(),
            Self::DiscreteMismatch(report) => report.render_human(),
            Self::HarnessFailure(report) => report.render_human(),
        }
    }
}

/// Closed reason for rejecting Phase 4 output as non-physics evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase4HarnessFailureReason {
    /// The oracle emitted the wrong number of results.
    ResultCount,
    /// A result echoed the wrong case identifier.
    CaseIdEcho,
    /// A result echoed the wrong operation.
    OperationEcho,
    /// A result echoed the wrong policy path.
    PolicyPathEcho,
    /// A result echoed the wrong requested horizon.
    HorizonEcho,
    /// A result emitted the wrong number of float values.
    ValueCount,
    /// A result emitted the wrong number of discrete values.
    DiscreteCount,
    /// A discrete value echoed the wrong field name.
    DiscreteFieldEcho,
    /// The request path has no reviewed policy.
    UnregisteredPolicy,
    /// The reviewed policy horizon disagrees with the request.
    PolicyHorizon,
    /// The effective build tier cannot authorize the selected policy.
    PolicyTier,
}

/// Bounded typed evidence for malformed Phase 4 output or policy configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Phase4HarnessFailureReport {
    signature_sha256: Sha256Hex,
    request_id: RequestId,
    request_sha256: Sha256Hex,
    scenario_id: ScenarioId,
    scenario_sha256: Sha256Hex,
    reason: Phase4HarnessFailureReason,
    maybe_case_id: Option<Box<str>>,
    maybe_case_index: Option<u32>,
    expected: Box<str>,
    actual: Box<str>,
    policy_id: Box<str>,
    policy_version: ToleranceProfileVersion,
    policy_sha256: Sha256Hex,
    evidence_tier: EvidenceTier,
    oracle_build_sha256: Sha256Hex,
    native_build_sha256: Sha256Hex,
    maybe_previous_case_id: Option<Box<str>>,
    maybe_next_case_id: Option<Box<str>>,
}

#[derive(Serialize)]
struct HarnessSignatureInput<'a> {
    request_id: &'a str,
    scenario_id: &'a str,
    reason: Phase4HarnessFailureReason,
    maybe_case_id: Option<&'a str>,
    policy_sha256: &'a str,
}

impl Phase4HarnessFailureReport {
    /// Constructs bounded non-physics evidence.
    ///
    /// # Errors
    ///
    /// Returns [`ReportRenderError`] if canonical hashing fails.
    #[allow(
        clippy::too_many_arguments,
        reason = "harness evidence binds request, policy, builds, reason, and bounded context"
    )]
    pub fn new(
        request: &MathProbeRequestRecord,
        reason: Phase4HarnessFailureReason,
        maybe_case_index: Option<usize>,
        expected: impl Into<String>,
        actual: impl Into<String>,
        policy_id: &str,
        policy_version: ToleranceProfileVersion,
        policy_sha256: &Sha256Hex,
        evidence_tier: EvidenceTier,
        oracle_identity: &BuildIdentity,
        native_identity: &BuildIdentity,
    ) -> Result<Self, ReportRenderError> {
        let maybe_case = maybe_case_index.and_then(|index| request.scenario().cases().get(index));
        let signature = HarnessSignatureInput {
            request_id: request.request_id().as_str(),
            scenario_id: request.scenario().scenario_id().as_str(),
            reason,
            maybe_case_id: maybe_case.map(|case| case.case_id()),
            policy_sha256: policy_sha256.as_str(),
        };
        Ok(Self {
            signature_sha256: hash_serialized(&signature)?,
            request_id: request.request_id().clone(),
            request_sha256: hash_serialized(request)?,
            scenario_id: request.scenario().scenario_id().clone(),
            scenario_sha256: hash_serialized(request.scenario())?,
            reason,
            maybe_case_id: maybe_case.map(|case| case.case_id().into()),
            maybe_case_index: maybe_case_index.map(bounded_case_index),
            expected: bounded_context(expected.into()),
            actual: bounded_context(actual.into()),
            policy_id: policy_id.into(),
            policy_version,
            policy_sha256: policy_sha256.clone(),
            evidence_tier,
            oracle_build_sha256: oracle_identity.identity_sha256().clone(),
            native_build_sha256: native_identity.identity_sha256().clone(),
            maybe_previous_case_id: previous_case_id(request, maybe_case_index),
            maybe_next_case_id: next_case_id(request, maybe_case_index),
        })
    }

    /// Returns the closed failure reason.
    #[must_use]
    pub const fn reason(&self) -> Phase4HarnessFailureReason {
        self.reason
    }

    /// Returns the stable failure signature.
    #[must_use]
    pub const fn signature_sha256(&self) -> &Sha256Hex {
        &self.signature_sha256
    }

    /// Renders a concise diagnostic from the typed evidence.
    #[must_use]
    pub fn render_human(&self) -> String {
        format!(
            "Phase 4 harness failure {:?}: expected {}, actual {}; signature {}",
            self.reason,
            self.expected,
            self.actual,
            self.signature_sha256.as_str(),
        )
    }
}

/// Typed evidence for an exact discrete Phase 4 divergence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Phase4DiscreteMismatchReport {
    signature_sha256: Sha256Hex,
    request_id: RequestId,
    request_sha256: Sha256Hex,
    scenario_id: ScenarioId,
    scenario_sha256: Sha256Hex,
    case_id: Box<str>,
    case_index: u32,
    operation: MathProbeOperation,
    policy_path: MathProbePolicyPath,
    expected_field: MathProbeDiscreteField,
    actual_field: MathProbeDiscreteField,
    expected_value: bool,
    actual_value: bool,
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
    maybe_previous_case_id: Option<Box<str>>,
    maybe_next_case_id: Option<Box<str>>,
}

#[derive(Serialize)]
struct DiscreteSignatureInput<'a> {
    request_id: &'a str,
    scenario_id: &'a str,
    case_id: &'a str,
    operation: MathProbeOperation,
    policy_path: MathProbePolicyPath,
    field: MathProbeDiscreteField,
    policy_sha256: &'a str,
}

impl Phase4DiscreteMismatchReport {
    /// Constructs exact discrete mismatch evidence.
    ///
    /// # Errors
    ///
    /// Returns [`ReportRenderError`] if canonical hashing fails.
    #[allow(
        clippy::too_many_arguments,
        reason = "discrete evidence binds values, policy, tier, and both builds"
    )]
    pub fn new(
        request: &MathProbeRequestRecord,
        expected_result: &MathProbeResult,
        case_index: usize,
        expected: MathProbeDiscrete,
        actual: MathProbeDiscrete,
        policy_id: &str,
        policy_version: ToleranceProfileVersion,
        policy_sha256: &Sha256Hex,
        field_policy: &FieldPolicy,
        evidence_tier: EvidenceTier,
        oracle_identity: &BuildIdentity,
        native_identity: &BuildIdentity,
    ) -> Result<Self, ReportRenderError> {
        let signature = DiscreteSignatureInput {
            request_id: request.request_id().as_str(),
            scenario_id: request.scenario().scenario_id().as_str(),
            case_id: expected_result.case_id(),
            operation: expected_result.operation(),
            policy_path: expected_result.policy_path(),
            field: expected.field(),
            policy_sha256: policy_sha256.as_str(),
        };
        Ok(Self {
            signature_sha256: hash_serialized(&signature)?,
            request_id: request.request_id().clone(),
            request_sha256: hash_serialized(request)?,
            scenario_id: request.scenario().scenario_id().clone(),
            scenario_sha256: hash_serialized(request.scenario())?,
            case_id: expected_result.case_id().into(),
            case_index: bounded_case_index(case_index),
            operation: expected_result.operation(),
            policy_path: expected_result.policy_path(),
            expected_field: expected.field(),
            actual_field: actual.field(),
            expected_value: expected.value(),
            actual_value: actual.value(),
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
            oracle_build_sha256: oracle_identity.identity_sha256().clone(),
            native_build_sha256: native_identity.identity_sha256().clone(),
            maybe_previous_case_id: previous_case_id(request, Some(case_index)),
            maybe_next_case_id: next_case_id(request, Some(case_index)),
        })
    }

    /// Returns the stable failure signature.
    #[must_use]
    pub const fn signature_sha256(&self) -> &Sha256Hex {
        &self.signature_sha256
    }

    /// Renders a concise diagnostic from the typed evidence.
    #[must_use]
    pub fn render_human(&self) -> String {
        format!(
            "discrete mismatch at case {} field {:?}: expected {}, actual {}; policy {}, horizon {:?}, tier {:?}; signature {}",
            self.case_id,
            self.expected_field,
            self.expected_value,
            self.actual_value,
            self.policy_id,
            self.horizon,
            self.evidence_tier,
            self.signature_sha256.as_str(),
        )
    }
}

fn hash_serialized(value: &impl Serialize) -> Result<Sha256Hex, ReportRenderError> {
    let bytes = serde_json::to_vec(value).map_err(ReportRenderError)?;
    Ok(Sha256Hex::from_digest(Sha256::digest(bytes).into()))
}

fn bounded_context(mut value: String) -> Box<str> {
    if value.len() > MAXIMUM_CONTEXT_BYTES {
        let mut boundary = MAXIMUM_CONTEXT_BYTES;
        while !value.is_char_boundary(boundary) {
            boundary -= 1;
        }
        value.truncate(boundary);
    }
    value.into_boxed_str()
}

fn bounded_case_index(index: usize) -> u32 {
    u32::try_from(index).expect("validated bounded math probe case index fits in u32")
}

fn previous_case_id(
    request: &MathProbeRequestRecord,
    maybe_case_index: Option<usize>,
) -> Option<Box<str>> {
    maybe_case_index
        .and_then(|index| index.checked_sub(1))
        .and_then(|index| request.scenario().cases().get(index))
        .map(|case| case.case_id().into())
}

fn next_case_id(
    request: &MathProbeRequestRecord,
    maybe_case_index: Option<usize>,
) -> Option<Box<str>> {
    maybe_case_index
        .and_then(|index| index.checked_add(1))
        .and_then(|index| request.scenario().cases().get(index))
        .map(|case| case.case_id().into())
}
