//! Effectful CLI shell around the deterministic typed minimizer.

use std::{
    path::Path,
    process::ExitCode,
    time::{Duration, Instant},
};

use liquidfun_differential::{
    DifferentialRunOutcome, Evaluation, FailureSignature, MinimizationArtifactRequest,
    MinimizationBudget, MinimizationResult, MinimizationStatus, OraclePreset, ScenarioTransform,
    SessionProfile, minimize, persist_minimization_artifact, run_scenario_request,
};
use liquidfun_test_protocol::{
    HarnessLimits, RecordLimit, ScenarioRequestRecord, ValidatedScenarioV1,
    decode_scenario_request_jsonl, encode_jsonl,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::{
    CliError, MachineReport, json_line, persist_outcome_bundle, render_outcome, write_bytes,
};

const MAXIMUM_ATTEMPTS: usize = 128;
const MINIMIZATION_DEADLINE: Duration = Duration::from_secs(30);

pub(super) fn run(
    repository_root: &Path,
    request_bytes: &[u8],
    preset: OraclePreset,
    profile: SessionProfile,
    outcome: DifferentialRunOutcome,
) -> Result<ExitCode, CliError> {
    let request =
        decode_scenario_request_jsonl(request_bytes, &HarnessLimits::phase2_default_v1())?;
    let mismatch = match outcome {
        DifferentialRunOutcome::PhysicsMismatch(report) => report,
        DifferentialRunOutcome::HarnessFailure(failure) => {
            return render_outcome(
                repository_root,
                request_bytes,
                preset,
                profile,
                DifferentialRunOutcome::HarnessFailure(failure),
            );
        }
        DifferentialRunOutcome::Match(_) => return Err(CliError::MinimizeRequiresMismatch),
    };
    persist_initial_mismatch(
        repository_root,
        &request,
        request_bytes,
        preset,
        profile,
        &mismatch,
    )?;

    let target = mismatch.signature().clone();
    let mut maybe_evaluation_error = None;
    let result = minimize(
        request.scenario(),
        &target,
        MinimizationBudget::new(MAXIMUM_ATTEMPTS, MINIMIZATION_DEADLINE),
        |candidate| {
            evaluate_candidate(
                repository_root,
                &request,
                candidate,
                preset,
                profile,
                &mut maybe_evaluation_error,
            )
        },
    )?;
    if let Some(error) = maybe_evaluation_error {
        return Err(error);
    }

    let original_scenario_bytes = serde_json::to_vec(request.scenario())?;
    let report = MinimizationMachineReport::new(&target, request.scenario(), &result);
    let report_bytes = json_line(&report)?;
    let receipt = persist_minimization_artifact(
        repository_root,
        &MinimizationArtifactRequest {
            request_id: request.request_id(),
            scenario_json: result.canonical_scenario_bytes(),
            report_json: &report_bytes,
        },
    )?;
    write_bytes(&report_bytes)?;
    eprintln!(
        "minimization {:?}: {} -> {} scenario bytes; persisted {}",
        result.status(),
        original_scenario_bytes.len(),
        result.canonical_scenario_bytes().len(),
        receipt.directory().display()
    );
    Ok(ExitCode::SUCCESS)
}

fn persist_initial_mismatch(
    repository_root: &Path,
    request: &ScenarioRequestRecord,
    request_bytes: &[u8],
    preset: OraclePreset,
    profile: SessionProfile,
    mismatch: &liquidfun_differential::MismatchReport,
) -> Result<(), CliError> {
    let report_bytes = json_line(&MachineReport::mismatch(mismatch))?;
    persist_outcome_bundle(
        repository_root,
        request,
        request_bytes,
        &report_bytes,
        preset,
        profile,
        "physics_mismatch",
        None,
        b"",
    )
}

fn evaluate_candidate(
    repository_root: &Path,
    original_request: &ScenarioRequestRecord,
    candidate: &ValidatedScenarioV1,
    preset: OraclePreset,
    profile: SessionProfile,
    maybe_error: &mut Option<CliError>,
) -> Evaluation {
    if maybe_error.is_some() {
        return Evaluation::new(None, Duration::ZERO);
    }
    let request = original_request.with_scenario(candidate.clone());
    let candidate_bytes = match encode_jsonl(
        &request,
        &HarnessLimits::phase2_default_v1(),
        RecordLimit::Input,
    ) {
        Ok(bytes) => bytes,
        Err(error) => {
            *maybe_error = Some(CliError::MinimizationEncode(error.to_string()));
            return Evaluation::new(None, Duration::ZERO);
        }
    };
    let started = Instant::now();
    let outcome = run_scenario_request(
        repository_root,
        request.clone(),
        preset,
        profile,
        super::ORACLE_REVISION,
    );
    let elapsed = started.elapsed();
    match outcome {
        Ok(DifferentialRunOutcome::PhysicsMismatch(report)) => {
            Evaluation::new(Some(report.signature().clone()), elapsed)
        }
        Ok(DifferentialRunOutcome::Match(_)) => Evaluation::new(None, elapsed),
        Ok(DifferentialRunOutcome::HarnessFailure(failure)) => {
            let machine = MachineReport::harness(failure.kind().as_str());
            let persistence = json_line(&machine).and_then(|report_bytes| {
                persist_outcome_bundle(
                    repository_root,
                    &request,
                    &candidate_bytes,
                    &report_bytes,
                    preset,
                    profile,
                    "harness_failure",
                    failure
                        .evidence()
                        .maybe_session_identity_sha256()
                        .map(liquidfun_test_protocol::Sha256Hex::as_str),
                    failure.evidence().stderr().retained(),
                )
            });
            *maybe_error = Some(match persistence {
                Ok(()) => CliError::MinimizationHarness(failure.kind().as_str().to_owned()),
                Err(error) => error,
            });
            Evaluation::new(None, elapsed)
        }
        Err(error) => {
            *maybe_error = Some(CliError::Runner(error));
            Evaluation::new(None, elapsed)
        }
    }
}

#[derive(Serialize)]
struct MinimizationMachineReport<'a> {
    result_kind: &'static str,
    status: MinimizationStatus,
    target_signature: &'a FailureSignature,
    original_commands: usize,
    original_checkpoints: usize,
    minimized_commands: usize,
    minimized_checkpoints: usize,
    attempts: usize,
    evaluations: usize,
    rejected_invalid_candidates: usize,
    rejected_changed_signatures: usize,
    attempted_transforms: &'a [ScenarioTransform],
    accepted_transforms: &'a [ScenarioTransform],
    scenario_sha256: String,
    scenario: &'a ValidatedScenarioV1,
}

impl<'a> MinimizationMachineReport<'a> {
    fn new(
        target: &'a FailureSignature,
        original: &ValidatedScenarioV1,
        result: &'a MinimizationResult,
    ) -> Self {
        Self {
            result_kind: "minimization",
            status: result.status(),
            target_signature: target,
            original_commands: original.commands().len(),
            original_checkpoints: original.checkpoints().len(),
            minimized_commands: result.scenario().commands().len(),
            minimized_checkpoints: result.scenario().checkpoints().len(),
            attempts: result.attempts(),
            evaluations: result.evaluations(),
            rejected_invalid_candidates: result.rejected_invalid_candidates(),
            rejected_changed_signatures: result.rejected_changed_signatures(),
            attempted_transforms: result.attempted_transforms(),
            accepted_transforms: result.accepted_transforms(),
            scenario_sha256: format!("{:x}", Sha256::digest(result.canonical_scenario_bytes())),
            scenario: result.scenario(),
        }
    }
}
