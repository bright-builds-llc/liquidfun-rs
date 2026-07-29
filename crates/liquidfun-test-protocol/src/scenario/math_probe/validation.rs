use std::collections::HashSet;

use super::{
    MAXIMUM_SCENARIO_STEPS, MathProbeCase, MathProbeDecodeError, MathProbeErrorKind,
    MathProbeHorizon, MathProbeInput, MathProbeOperation, MathProbePolicyPath,
    MathProbeRequestRecord, MathProbeScenario, RawCase, RawRequest, RawSource,
};
use crate::{RequestId, ScenarioId, ScenarioSource};

pub(super) fn validate_request(
    raw: RawRequest,
) -> Result<MathProbeRequestRecord, MathProbeDecodeError> {
    let request_id = RequestId::new(raw.request_id.into_string())
        .map_err(|_| MathProbeDecodeError::Validation(MathProbeErrorKind::InvalidIdentifier))?;
    let scenario_id = ScenarioId::new(raw.scenario.scenario_id.into_string())
        .map_err(|_| MathProbeDecodeError::Validation(MathProbeErrorKind::InvalidIdentifier))?;
    let source = validate_source(raw.scenario.source)?;
    let raw_cases = raw.scenario.cases.into_vec();
    if raw_cases.is_empty() {
        return Err(MathProbeDecodeError::Validation(
            MathProbeErrorKind::NoCases,
        ));
    }
    let mut ids = HashSet::with_capacity(raw_cases.len());
    let mut cases = Vec::with_capacity(raw_cases.len());
    for raw_case in raw_cases {
        validate_case(&raw_case)?;
        let case_id = raw_case.case_id.into_string();
        ScenarioId::new(case_id.clone())
            .map_err(|_| MathProbeDecodeError::Validation(MathProbeErrorKind::InvalidIdentifier))?;
        if !ids.insert(case_id.clone()) {
            return Err(MathProbeDecodeError::Validation(
                MathProbeErrorKind::DuplicateCaseId,
            ));
        }
        cases.push(MathProbeCase {
            case_id: case_id.into_boxed_str(),
            operation: raw_case.operation,
            policy_path: raw_case.policy_path,
            horizon: raw_case.horizon,
            input: raw_case.input,
        });
    }
    Ok(MathProbeRequestRecord {
        protocol_version: raw.protocol_version,
        record_kind: raw.record_kind,
        request_id,
        scenario_schema_version: raw.scenario_schema_version,
        requested_trace_schema_version: raw.requested_trace_schema_version,
        tolerance_profile_version: raw.tolerance_profile_version,
        tolerance_profile_sha256: raw.tolerance_profile_sha256,
        scenario: MathProbeScenario {
            scenario_id,
            source,
            cases,
        },
    })
}

fn validate_source(raw: RawSource) -> Result<ScenarioSource, MathProbeDecodeError> {
    match raw {
        RawSource::Named { name } => {
            let name = name.into_string();
            if name.trim().is_empty() {
                return Err(MathProbeDecodeError::Validation(
                    MathProbeErrorKind::InvalidSource,
                ));
            }
            Ok(ScenarioSource::Named {
                name: name.into_boxed_str(),
            })
        }
        RawSource::Seeded {
            generator_id,
            generator_version,
            seed,
        } => {
            let generator_id = generator_id.into_string();
            if generator_id.trim().is_empty() || generator_version == 0 {
                return Err(MathProbeDecodeError::Validation(
                    MathProbeErrorKind::InvalidSource,
                ));
            }
            Ok(ScenarioSource::Seeded {
                generator_id: generator_id.into_boxed_str(),
                generator_version,
                seed,
            })
        }
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "one exhaustive table keeps the closed operation/input/path contract auditable"
)]
fn validate_case(raw: &RawCase) -> Result<(), MathProbeDecodeError> {
    let input_matches = matches!(
        (raw.operation, &raw.input),
        (
            MathProbeOperation::IsValid | MathProbeOperation::Abs | MathProbeOperation::InvSqrt,
            MathProbeInput::Scalar { .. }
        ) | (
            MathProbeOperation::Min | MathProbeOperation::Max,
            MathProbeInput::Binary { .. }
        ) | (MathProbeOperation::Clamp, MathProbeInput::Clamp { .. })
            | (
                MathProbeOperation::VecLength | MathProbeOperation::VecNormalize,
                MathProbeInput::Vector2 { .. }
            )
            | (
                MathProbeOperation::Dot | MathProbeOperation::Cross,
                MathProbeInput::VectorPair { .. }
            )
            | (
                MathProbeOperation::Mat22Solve,
                MathProbeInput::Mat22Solve { .. }
            )
            | (
                MathProbeOperation::Mat33Solve,
                MathProbeInput::Mat33Solve { .. }
            )
            | (
                MathProbeOperation::Mat22Inverse,
                MathProbeInput::Mat22 { .. }
            )
            | (
                MathProbeOperation::Mat33SymInverse,
                MathProbeInput::Mat33 { .. }
            )
            | (
                MathProbeOperation::Rotation,
                MathProbeInput::Rotation { .. }
            )
            | (
                MathProbeOperation::Transform,
                MathProbeInput::Transform { .. }
            )
            | (
                MathProbeOperation::SweepTransform,
                MathProbeInput::SweepTransform { .. }
            )
            | (
                MathProbeOperation::SweepAdvance,
                MathProbeInput::SweepAdvance { .. }
            )
            | (
                MathProbeOperation::SweepNormalize,
                MathProbeInput::Sweep { .. }
            )
            | (
                MathProbeOperation::Cancellation,
                MathProbeInput::Cancellation { .. }
            )
            | (
                MathProbeOperation::HalfwayRounding,
                MathProbeInput::HalfwayRounding { .. }
            )
            | (
                MathProbeOperation::Overflow | MathProbeOperation::Underflow,
                MathProbeInput::Scale { .. }
            )
            | (
                MathProbeOperation::FmaWitness,
                MathProbeInput::FmaWitness { .. }
            )
    );
    if !input_matches {
        return Err(MathProbeDecodeError::Validation(
            MathProbeErrorKind::OperationInputMismatch,
        ));
    }
    let expected_path = match (raw.operation, raw.horizon) {
        (MathProbeOperation::IsValid, _) => MathProbePolicyPath::MathBranchIsValid,
        (MathProbeOperation::Abs, _) => MathProbePolicyPath::MathOperationAbs,
        (MathProbeOperation::Min, _) => MathProbePolicyPath::MathOperationMin,
        (MathProbeOperation::Max, _) => MathProbePolicyPath::MathPassThroughMax,
        (MathProbeOperation::Clamp, _) => MathProbePolicyPath::MathOperationClamp,
        (MathProbeOperation::InvSqrt, _) => MathProbePolicyPath::MathOperationInvSqrt,
        (MathProbeOperation::VecLength, _) => MathProbePolicyPath::MathVectorLength,
        (MathProbeOperation::VecNormalize, _) => MathProbePolicyPath::MathVectorNormalize,
        (MathProbeOperation::Dot, _) => MathProbePolicyPath::MathVectorDot,
        (MathProbeOperation::Cross, _) => MathProbePolicyPath::MathVectorCross,
        (MathProbeOperation::Mat22Solve, _) => MathProbePolicyPath::MathMatrix22Solve,
        (MathProbeOperation::Mat33Solve, _) => MathProbePolicyPath::MathMatrix33Solve,
        (MathProbeOperation::Mat22Inverse, _) => MathProbePolicyPath::MathMatrix22Inverse,
        (MathProbeOperation::Mat33SymInverse, _) => {
            MathProbePolicyPath::MathMatrix33SymmetricInverse
        }
        (MathProbeOperation::Rotation, _) => MathProbePolicyPath::MathRotation,
        (MathProbeOperation::Transform, MathProbeHorizon::Operation) => {
            MathProbePolicyPath::MathTransformOperation
        }
        (MathProbeOperation::Transform, MathProbeHorizon::ScenarioSteps { steps: 32 }) => {
            MathProbePolicyPath::MathTransformSteps32
        }
        (MathProbeOperation::SweepTransform, _) => MathProbePolicyPath::MathSweepTransform,
        (MathProbeOperation::SweepAdvance, MathProbeHorizon::ScenarioSteps { steps: 4 }) => {
            MathProbePolicyPath::MathSweepAdvanceSteps4
        }
        (MathProbeOperation::Transform | MathProbeOperation::SweepAdvance, _) => {
            return Err(MathProbeDecodeError::Validation(
                MathProbeErrorKind::InvalidHorizon,
            ));
        }
        (MathProbeOperation::SweepNormalize, _) => MathProbePolicyPath::MathSweepNormalize,
        (MathProbeOperation::Cancellation, _) => MathProbePolicyPath::MathArithmeticCancellation,
        (MathProbeOperation::HalfwayRounding, _) => {
            MathProbePolicyPath::MathArithmeticHalfwayRounding
        }
        (MathProbeOperation::Overflow, _) => MathProbePolicyPath::MathArithmeticOverflow,
        (MathProbeOperation::Underflow, _) => MathProbePolicyPath::MathArithmeticUnderflow,
        (MathProbeOperation::FmaWitness, _) => MathProbePolicyPath::MathArithmeticFmaWitness,
    };
    if raw.policy_path != expected_path {
        return Err(MathProbeDecodeError::Validation(
            MathProbeErrorKind::PolicyPathMismatch,
        ));
    }
    let steps = raw.horizon.steps();
    if steps == 0 || usize::try_from(steps).map_or(true, |value| value > MAXIMUM_SCENARIO_STEPS) {
        return Err(MathProbeDecodeError::Validation(
            MathProbeErrorKind::InvalidHorizon,
        ));
    }
    match (&raw.horizon, &raw.input) {
        (
            MathProbeHorizon::ScenarioSteps { steps },
            MathProbeInput::SweepAdvance { fractions_bits, .. },
        ) if usize::try_from(*steps).ok() == Some(fractions_bits.as_slice().len())
            && !fractions_bits.as_slice().is_empty() => {}
        (MathProbeHorizon::Operation, MathProbeInput::SweepAdvance { fractions_bits, .. })
            if fractions_bits.as_slice().len() == 1 => {}
        (MathProbeHorizon::ScenarioSteps { .. }, MathProbeInput::Transform { .. })
        | (MathProbeHorizon::Operation, _) => {}
        _ => {
            return Err(MathProbeDecodeError::Validation(
                MathProbeErrorKind::InvalidHorizon,
            ));
        }
    }
    Ok(())
}
