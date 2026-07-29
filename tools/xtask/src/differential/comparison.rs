#[allow(
    clippy::wildcard_imports,
    reason = "this split module shares its parent private contract"
)]
use super::*;

#[allow(
    clippy::too_many_lines,
    reason = "one ordered traversal keeps every typed Phase 4 failure path fail-closed"
)]
pub(super) fn compare_math_probe_results(
    request: &MathProbeRequestRecord,
    actual: &[MathProbeResult],
    policy: &Phase4PolicyProfile,
    oracle_identity: &BuildIdentity,
    native_identity: &BuildIdentity,
) -> Result<(), DifferentialError> {
    for (engine, identity) in [
        ("C++ oracle", oracle_identity),
        ("native Rust", native_identity),
    ] {
        if identity.oracle_revision() != ORACLE_REVISION || identity.maybe_phase4().is_none() {
            return Err(DifferentialError::new(
                "identity",
                format!("{engine} identity is not bound to the Phase 4 oracle contract"),
            ));
        }
        if identity.evidence_tier() == BuildEvidenceTier::D3Exploratory {
            return Err(DifferentialError::new(
                "identity",
                format!("{engine} exploratory identity cannot authorize Phase 4 comparison"),
            ));
        }
    }
    let expected = NativeMathProbeExecutor::execute(request)
        .map_err(|error| DifferentialError::new("native", error.to_string()))?;
    let comparison_tier = comparison_evidence_tier(oracle_identity, native_identity);
    if expected.len() != actual.len() {
        return Err(phase4_harness_failure(
            request,
            policy,
            comparison_tier,
            oracle_identity,
            native_identity,
            Phase4HarnessFailureReason::ResultCount,
            None,
            expected.len().to_string(),
            actual.len().to_string(),
        )?);
    }
    for (case_index, (expected, actual)) in expected.iter().zip(actual).enumerate() {
        let structural_failure = if expected.case_id() != actual.case_id() {
            Some((
                Phase4HarnessFailureReason::CaseIdEcho,
                expected.case_id().to_owned(),
                actual.case_id().to_owned(),
            ))
        } else if expected.operation() != actual.operation() {
            Some((
                Phase4HarnessFailureReason::OperationEcho,
                format!("{:?}", expected.operation()),
                format!("{:?}", actual.operation()),
            ))
        } else if expected.policy_path() != actual.policy_path() {
            Some((
                Phase4HarnessFailureReason::PolicyPathEcho,
                expected.policy_path().as_str().to_owned(),
                actual.policy_path().as_str().to_owned(),
            ))
        } else if expected.horizon() != actual.horizon() {
            Some((
                Phase4HarnessFailureReason::HorizonEcho,
                format!("{:?}", expected.horizon()),
                format!("{:?}", actual.horizon()),
            ))
        } else if expected.values().len() != actual.values().len() {
            Some((
                Phase4HarnessFailureReason::ValueCount,
                expected.values().len().to_string(),
                actual.values().len().to_string(),
            ))
        } else if expected.discrete().len() != actual.discrete().len() {
            Some((
                Phase4HarnessFailureReason::DiscreteCount,
                expected.discrete().len().to_string(),
                actual.discrete().len().to_string(),
            ))
        } else {
            None
        };
        if let Some((reason, expected_context, actual_context)) = structural_failure {
            return Err(phase4_harness_failure(
                request,
                policy,
                comparison_tier,
                oracle_identity,
                native_identity,
                reason,
                Some(case_index),
                expected_context,
                actual_context,
            )?);
        }
        let Some(field_policy) = policy.field(expected.policy_path().as_str()) else {
            return Err(phase4_harness_failure(
                request,
                policy,
                comparison_tier,
                oracle_identity,
                native_identity,
                Phase4HarnessFailureReason::UnregisteredPolicy,
                Some(case_index),
                expected.policy_path().as_str(),
                "<missing>",
            )?);
        };
        if !horizons_match(expected.horizon(), field_policy.horizon()) {
            return Err(phase4_harness_failure(
                request,
                policy,
                comparison_tier,
                oracle_identity,
                native_identity,
                Phase4HarnessFailureReason::PolicyHorizon,
                Some(case_index),
                format!("{:?}", expected.horizon()),
                format!("{:?}", field_policy.horizon()),
            )?);
        }
        if !tier_authorizes(comparison_tier, field_policy.evidence_tier()) {
            return Err(phase4_harness_failure(
                request,
                policy,
                comparison_tier,
                oracle_identity,
                native_identity,
                Phase4HarnessFailureReason::PolicyTier,
                Some(case_index),
                format!("{:?}", field_policy.evidence_tier()),
                format!("{comparison_tier:?}"),
            )?);
        }
        for (expected_discrete, actual_discrete) in
            expected.discrete().iter().zip(actual.discrete())
        {
            if expected_discrete.field() != actual_discrete.field() {
                return Err(phase4_harness_failure(
                    request,
                    policy,
                    comparison_tier,
                    oracle_identity,
                    native_identity,
                    Phase4HarnessFailureReason::DiscreteFieldEcho,
                    Some(case_index),
                    format!("{:?}", expected_discrete.field()),
                    format!("{:?}", actual_discrete.field()),
                )?);
            }
            if expected_discrete.value() != actual_discrete.value() {
                let report = Phase4DiscreteMismatchReport::new(
                    request,
                    expected,
                    case_index,
                    *expected_discrete,
                    *actual_discrete,
                    policy.profile_id(),
                    policy.version(),
                    policy.profile_sha256(),
                    field_policy,
                    comparison_tier,
                    oracle_identity,
                    native_identity,
                )
                .map_err(|error| DifferentialError::new("report", error.to_string()))?;
                return Err(DifferentialError::phase4_evidence(
                    "physics-mismatch",
                    Phase4ComparisonEvidence::DiscreteMismatch(report),
                ));
            }
        }
        for (expected_value, actual_value) in expected.values().iter().zip(actual.values()) {
            if expected_value.field() != actual_value.field()
                || expected_value.class() != actual_value.class()
                || expected_value.is_negative() != actual_value.is_negative()
                || !float_values_match_with_policy(
                    expected_value.bits(),
                    actual_value.bits(),
                    field_policy,
                )
            {
                let report = Phase4MathMismatchReport::new(
                    request,
                    expected,
                    case_index,
                    *expected_value,
                    *actual_value,
                    policy.profile_id(),
                    policy.version(),
                    policy.profile_sha256(),
                    field_policy,
                    comparison_tier,
                    oracle_identity.identity_sha256(),
                    native_identity.identity_sha256(),
                )
                .map_err(|error| DifferentialError::new("report", error.to_string()))?;
                return Err(DifferentialError::phase4_evidence(
                    "physics-mismatch",
                    Phase4ComparisonEvidence::NumericMismatch(report),
                ));
            }
        }
    }
    Ok(())
}

#[allow(
    clippy::too_many_arguments,
    reason = "the helper binds the failure to request, policy, tier, and both builds"
)]
pub(super) fn phase4_harness_failure(
    request: &MathProbeRequestRecord,
    policy: &Phase4PolicyProfile,
    evidence_tier: EvidenceTier,
    oracle_identity: &BuildIdentity,
    native_identity: &BuildIdentity,
    reason: Phase4HarnessFailureReason,
    maybe_case_index: Option<usize>,
    expected: impl Into<String>,
    actual: impl Into<String>,
) -> Result<DifferentialError, DifferentialError> {
    let report = Phase4HarnessFailureReport::new(
        request,
        reason,
        maybe_case_index,
        expected,
        actual,
        policy.profile_id(),
        policy.version(),
        policy.profile_sha256(),
        evidence_tier,
        oracle_identity,
        native_identity,
    )
    .map_err(|error| DifferentialError::new("report", error.to_string()))?;
    Ok(DifferentialError::phase4_evidence(
        "harness-failure",
        Phase4ComparisonEvidence::HarnessFailure(report),
    ))
}

pub(super) fn comparison_evidence_tier(
    oracle_identity: &BuildIdentity,
    native_identity: &BuildIdentity,
) -> EvidenceTier {
    match (
        oracle_identity.evidence_tier(),
        native_identity.evidence_tier(),
    ) {
        (BuildEvidenceTier::D1Canonical, BuildEvidenceTier::D1Canonical) => {
            EvidenceTier::D1Canonical
        }
        (BuildEvidenceTier::D3Exploratory, _) | (_, BuildEvidenceTier::D3Exploratory) => {
            EvidenceTier::D3Exploratory
        }
        _ => EvidenceTier::D2Supported,
    }
}

pub(super) const fn horizons_match(request: MathProbeHorizon, policy: DivergenceHorizon) -> bool {
    match (request, policy) {
        (MathProbeHorizon::Operation, DivergenceHorizon::Operation) => true,
        (
            MathProbeHorizon::ScenarioSteps {
                steps: request_steps,
            },
            DivergenceHorizon::ScenarioSteps {
                steps: policy_steps,
            },
        ) => request_steps == policy_steps,
        _ => false,
    }
}

pub(super) const fn tier_authorizes(actual: EvidenceTier, policy: EvidenceTier) -> bool {
    matches!(
        actual,
        EvidenceTier::D1Canonical | EvidenceTier::D2Supported
    ) && matches!(
        policy,
        EvidenceTier::D1Canonical | EvidenceTier::D2Supported
    )
}

pub(super) fn verify_math_probe_determinism(
    repository_root: &Path,
    request: &MathProbeRequestRecord,
    preset: &str,
    runs: usize,
) -> Result<(), DifferentialError> {
    let mut baseline = None;
    for run in 0..runs {
        let capture = execute_math_probe_once(repository_root, request, preset)?;
        if let Some(expected) = &baseline
            && expected != &capture.response_bytes
        {
            return Err(DifferentialError::new(
                "determinism",
                format!("D0 response bytes changed on run {}", run + 1),
            ));
        }
        baseline = Some(capture.response_bytes);
    }
    println!("math-probes D0: {runs} byte-identical {preset} runs");
    Ok(())
}

pub(super) fn verify_collision_probe_determinism(
    repository_root: &Path,
    request: &CollisionProbeRequestRecord,
    preset: &str,
    runs: usize,
) -> Result<(), DifferentialError> {
    let mut baseline = None;
    for run in 0..runs {
        let capture = execute_collision_probe_once(repository_root, request, preset)?;
        if let Some(expected) = &baseline
            && expected != &capture.response_bytes
        {
            return Err(DifferentialError::new(
                "determinism",
                format!("collision D0 response bytes changed on run {}", run + 1),
            ));
        }
        baseline = Some(capture.response_bytes);
    }
    println!("collision-probes D0: {runs} byte-identical {preset} runs");
    Ok(())
}
