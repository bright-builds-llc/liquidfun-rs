#[allow(
    clippy::wildcard_imports,
    reason = "this split module shares its parent private contract"
)]
use super::*;

pub(super) fn run_rigid_world_minimization(
    repository_root: &Path,
    request: &RigidWorldRequestRecord,
    target: &RigidFailureSignature,
    phase6_policy: &Phase6PolicyProfile,
    phase7_policy: &Phase7PolicyProfile,
    phase8_policy: &Phase8PolicyProfile,
    preset: &str,
) -> Result<(), DifferentialError> {
    let mut maybe_evaluation_error = None;
    let result = reduce_rigid_world_mismatch(
        request,
        target,
        MinimizationBudget::new(
            RIGID_MINIMIZATION_MAXIMUM_ATTEMPTS,
            RIGID_MINIMIZATION_DEADLINE,
        ),
        |candidate| {
            evaluate_rigid_world_candidate(
                repository_root,
                candidate,
                phase6_policy,
                phase7_policy,
                phase8_policy,
                preset,
                &mut maybe_evaluation_error,
            )
        },
    )?;
    if let Some(error) = maybe_evaluation_error {
        return Err(error);
    }

    let report = RigidMinimizationMachineReport::new(target, request, &result)?;
    let mut report_bytes = serde_json::to_vec(&report)
        .map_err(|error| DifferentialError::new("report", error.to_string()))?;
    report_bytes.push(b'\n');
    let receipt = persist_rigid_minimization_artifact(
        repository_root,
        &RigidMinimizationArtifactRequest {
            request_id: request.request_id(),
            request_jsonl: result.canonical_request_bytes(),
            report_json: &report_bytes,
        },
    )
    .map_err(|error| DifferentialError::new("minimization-persistence", error.to_string()))?;
    println!("{}", String::from_utf8_lossy(&report_bytes).trim_end());
    eprintln!(
        "rigid-world minimization {:?}: {} -> {} request bytes; persisted {}",
        result.status(),
        report.original_request_bytes,
        result.canonical_request_bytes().len(),
        receipt.directory().display()
    );
    Ok(())
}

pub(super) fn reduce_rigid_world_mismatch<F>(
    request: &RigidWorldRequestRecord,
    target: &RigidFailureSignature,
    budget: MinimizationBudget,
    evaluator: F,
) -> Result<RigidMinimizationResult, DifferentialError>
where
    F: FnMut(&RigidWorldRequestRecord) -> RigidEvaluation,
{
    minimize_rigid_world_request(request, target, budget, evaluator)
        .map_err(|error| DifferentialError::new("minimization", error.to_string()))
}

pub(super) fn evaluate_rigid_world_candidate(
    repository_root: &Path,
    request: &RigidWorldRequestRecord,
    phase6_policy: &Phase6PolicyProfile,
    phase7_policy: &Phase7PolicyProfile,
    phase8_policy: &Phase8PolicyProfile,
    preset: &str,
    maybe_error: &mut Option<DifferentialError>,
) -> RigidEvaluation {
    if maybe_error.is_some() {
        return RigidEvaluation::new(None, Duration::ZERO);
    }
    let started = Instant::now();
    let evaluation = (|| {
        let captured = execute_rigid_world_once(repository_root, request, preset)?;
        let native = NativeRigidWorldExecutor::execute(request)
            .map_err(|error| DifferentialError::new("native", error.to_string()))?;
        compare_phase8_rigid_world_results(
            request,
            &native,
            captured.result(),
            phase6_policy,
            phase7_policy,
            phase8_policy,
        )
        .map_err(|error| {
            DifferentialError::new(
                "rigid-harness",
                serde_json::to_string(&error).unwrap_or_else(|_| format!("{error:?}")),
            )
        })
    })();
    let elapsed = started.elapsed();
    match evaluation {
        Ok(RigidComparisonOutcome::PhysicsMismatch(report)) => {
            RigidEvaluation::new(Some(report.signature().clone()), elapsed)
        }
        Ok(RigidComparisonOutcome::Match) => RigidEvaluation::new(None, elapsed),
        Err(error) => {
            *maybe_error = Some(error);
            RigidEvaluation::new(None, elapsed)
        }
    }
}

#[derive(Serialize)]
pub(super) struct RigidMinimizationMachineReport<'a> {
    result_kind: &'static str,
    status: MinimizationStatus,
    target_signature: &'a RigidFailureSignature,
    original_request_bytes: usize,
    minimized_request_bytes: usize,
    attempts: usize,
    evaluations: usize,
    rejected_invalid_candidates: usize,
    rejected_changed_signatures: usize,
    attempted_transforms: &'a [RigidScenarioTransform],
    accepted_transforms: &'a [RigidScenarioTransform],
    request_sha256: String,
}

impl<'a> RigidMinimizationMachineReport<'a> {
    pub(super) fn new(
        target: &'a RigidFailureSignature,
        original: &RigidWorldRequestRecord,
        result: &'a RigidMinimizationResult,
    ) -> Result<Self, DifferentialError> {
        let original_request_bytes = serde_json::to_vec(original)
            .map_err(|error| DifferentialError::new("report", error.to_string()))?
            .len()
            + 1;
        Ok(Self {
            result_kind: "rigid_world_minimization",
            status: result.status(),
            target_signature: target,
            original_request_bytes,
            minimized_request_bytes: result.canonical_request_bytes().len(),
            attempts: result.attempts(),
            evaluations: result.evaluations(),
            rejected_invalid_candidates: result.rejected_invalid_candidates(),
            rejected_changed_signatures: result.rejected_changed_signatures(),
            attempted_transforms: result.attempted_transforms(),
            accepted_transforms: result.accepted_transforms(),
            request_sha256: format!("{:x}", Sha256::digest(result.canonical_request_bytes())),
        })
    }
}

pub(super) fn rigid_world_request(
    repository_root: &Path,
    policy: &Phase8PolicyProfile,
) -> Result<RigidWorldRequestRecord, DifferentialError> {
    let request_bytes = read_regular_file(repository_root, RIGID_WORLD_REQUEST)?;
    let request =
        decode_rigid_world_request_jsonl(&request_bytes, &HarnessLimits::phase2_default_v1())
            .map_err(|error| DifferentialError::new("protocol", error.to_string()))?;
    if request.tolerance_profile_sha256() != policy.profile_sha256() {
        return Err(DifferentialError::new(
            "policy",
            format!(
                "rigid-world request policy hash {} does not match checked-in profile {}",
                request.tolerance_profile_sha256().as_str(),
                policy.profile_sha256().as_str()
            ),
        ));
    }
    Ok(request)
}

pub(super) fn execute_rigid_world_once(
    repository_root: &Path,
    request: &RigidWorldRequestRecord,
    preset: &str,
) -> Result<CapturedRigidWorld, DifferentialError> {
    let oracle_preset = match preset {
        "oracle-debug" => OraclePreset::Debug,
        "oracle-release" => OraclePreset::Release,
        "oracle-asan-ubsan" => OraclePreset::AsanUbsan,
        _ => return Err(DifferentialError::usage("unregistered rigid-world preset")),
    };
    let oracle_program = OracleExecutable::resolve(repository_root, oracle_preset)
        .map_err(|error| DifferentialError::new("oracle", error.to_string()))?;
    let captured = execute_rigid_world_process(&oracle_program, request, ORACLE_REVISION).map_err(
        |error| {
            DifferentialError::process(format!(
                "{}; stderr bytes {}, killed {}, reaped {}: {}",
                error,
                error.stderr_bytes(),
                error.child_killed(),
                error.child_reaped(),
                String::from_utf8_lossy(error.retained_stderr()).trim_end()
            ))
        },
    )?;
    if captured.identity().cmake_preset() != preset || captured.identity().maybe_phase4().is_none()
    {
        return Err(DifferentialError::new(
            "identity",
            "oracle handshake lacks the requested rigid-world build identity",
        ));
    }
    validate_oracle_checkout_identity(repository_root, preset, captured.identity())
        .map_err(|error| DifferentialError::new("identity", error.to_string()))?;
    Ok(captured)
}

pub(super) fn verify_rigid_world_determinism(
    repository_root: &Path,
    request: &RigidWorldRequestRecord,
    preset: &str,
    runs: usize,
) -> Result<(), DifferentialError> {
    let mut maybe_oracle_baseline: Option<Vec<u8>> = None;
    let mut maybe_native_baseline: Option<Vec<u8>> = None;
    for run in 0..runs {
        let capture = execute_rigid_world_once(repository_root, request, preset)?;
        if let Some(expected) = &maybe_oracle_baseline
            && expected.as_slice() != capture.response_bytes()
        {
            return Err(DifferentialError::new(
                "determinism",
                format!("rigid oracle D0 response bytes changed on run {}", run + 1),
            ));
        }
        maybe_oracle_baseline = Some(capture.response_bytes().to_vec());

        let native = NativeRigidWorldExecutor::execute(request)
            .map_err(|error| DifferentialError::new("native", error.to_string()))?;
        let native_bytes = serde_json::to_vec(&native)
            .map_err(|error| DifferentialError::new("protocol", error.to_string()))?;
        if let Some(expected) = &maybe_native_baseline
            && expected != &native_bytes
        {
            return Err(DifferentialError::new(
                "determinism",
                format!("native rigid D0 bytes changed on run {}", run + 1),
            ));
        }
        maybe_native_baseline = Some(native_bytes);
    }
    println!("rigid-world D0: {runs} byte-identical native and {preset} runs");
    Ok(())
}
