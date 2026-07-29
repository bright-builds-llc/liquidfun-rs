#[allow(
    clippy::wildcard_imports,
    reason = "this split module shares its parent private contract"
)]
use super::*;

pub(super) fn run_math_probe_command(
    repository_root: &Path,
    invocation: &MathProbeInvocation,
) -> Result<(), DifferentialError> {
    let request_bytes = read_regular_file(repository_root, MATH_PROBE_REQUEST)?;
    let policy_bytes = read_regular_file(repository_root, PHASE4_POLICY)?;
    let request =
        decode_math_probe_request_jsonl(&request_bytes, &HarnessLimits::phase2_default_v1())
            .map_err(|error| DifferentialError::new("protocol", error.to_string()))?;
    let policy_text = std::str::from_utf8(&policy_bytes)
        .map_err(|error| DifferentialError::new("protocol", error.to_string()))?;
    let policy = Phase4PolicyProfile::parse_toml(policy_text)
        .map_err(|error| DifferentialError::new("policy", error.to_string()))?;
    if request.tolerance_profile_sha256() != policy.profile_sha256() {
        return Err(DifferentialError::new(
            "policy",
            format!(
                "request policy hash {} does not match checked-in profile {}",
                request.tolerance_profile_sha256().as_str(),
                policy.profile_sha256().as_str()
            ),
        ));
    }

    if invocation.action == MathProbeAction::VerifyDeterminism {
        return verify_math_probe_determinism(
            repository_root,
            &request,
            &invocation.preset,
            invocation.runs,
        );
    }

    let capture = execute_math_probe_once(repository_root, &request, &invocation.preset)?;
    let native_adapter = EmptyWorldAdapter::new(ORACLE_REVISION)
        .map_err(|error| DifferentialError::new("identity", error.to_string()))?;
    let expected_native_digest = native_source_manifest_sha256(repository_root)?;
    if native_adapter
        .build_identity()
        .adapter_content_sha256()
        .as_str()
        != expected_native_digest
    {
        return Err(DifferentialError::new(
            "identity",
            "native adapter digest differs from independently hashed reviewed math inputs",
        ));
    }
    compare_math_probe_results(
        &request,
        &capture.results,
        &policy,
        &capture.oracle_identity,
        native_adapter.build_identity(),
    )?;
    let action = match invocation.action {
        MathProbeAction::Compare => "compare",
        MathProbeAction::Replay => "replay",
        MathProbeAction::Minimize => unreachable!("math probes do not support minimization"),
        MathProbeAction::VerifyDeterminism => unreachable!("handled before execution"),
    };
    println!(
        "math-probes {action}: {} ordered cases matched under {} ({})",
        capture.results.len(),
        policy.profile_id(),
        invocation.preset
    );
    Ok(())
}

pub(super) fn run_collision_probe_command(
    repository_root: &Path,
    invocation: &MathProbeInvocation,
) -> Result<(), DifferentialError> {
    let request_bytes = read_regular_file(repository_root, COLLISION_PROBE_REQUEST)?;
    let policy_bytes = read_regular_file(repository_root, PHASE5_POLICY)?;
    let request =
        decode_collision_probe_request_jsonl(&request_bytes, &HarnessLimits::phase2_default_v1())
            .map_err(|error| DifferentialError::new("protocol", error.to_string()))?;
    let policy_text = std::str::from_utf8(&policy_bytes)
        .map_err(|error| DifferentialError::new("protocol", error.to_string()))?;
    let policy = Phase5PolicyProfile::parse_toml(policy_text)
        .map_err(|error| DifferentialError::new("policy", error.to_string()))?;
    if request.tolerance_profile_sha256() != policy.profile_sha256() {
        return Err(DifferentialError::new(
            "policy",
            "collision request policy hash differs from the checked-in Phase 5 profile",
        ));
    }
    if invocation.action == MathProbeAction::VerifyDeterminism {
        return verify_collision_probe_determinism(
            repository_root,
            &request,
            &invocation.preset,
            invocation.runs,
        );
    }
    let capture = execute_collision_probe_once(repository_root, &request, &invocation.preset)?;
    let native = NativeCollisionProbeExecutor::execute(&request)
        .map_err(|error| DifferentialError::new("native", error.to_string()))?;
    compare_collision_probe_results(&request, &native, &capture.results, &policy).map_err(
        |divergence| {
            DifferentialError::new(
                "collision",
                format!(
                    "first divergence {}: {}",
                    divergence.signature_sha256().as_str(),
                    String::from_utf8_lossy(
                        &divergence
                            .render_machine()
                            .unwrap_or_else(|_| b"{}".to_vec())
                    )
                ),
            )
        },
    )?;
    let action = match invocation.action {
        MathProbeAction::Compare => "compare",
        MathProbeAction::Replay => "replay",
        MathProbeAction::Minimize => unreachable!("collision probes do not support minimization"),
        MathProbeAction::VerifyDeterminism => unreachable!("handled before execution"),
    };
    println!(
        "collision-probes {action}: {} ordered cases matched under {} ({})",
        capture.results.len(),
        policy.profile_id(),
        invocation.preset
    );
    Ok(())
}

pub(super) fn run_rigid_world_command(
    repository_root: &Path,
    invocation: &MathProbeInvocation,
) -> Result<(), DifferentialError> {
    let phase6_policy_bytes = read_regular_file(repository_root, PHASE6_POLICY)?;
    let phase6_policy_text = std::str::from_utf8(&phase6_policy_bytes)
        .map_err(|error| DifferentialError::new("protocol", error.to_string()))?;
    let phase6_policy = Phase6PolicyProfile::parse_toml(phase6_policy_text)
        .map_err(|error| DifferentialError::new("policy", error.to_string()))?;
    let phase7_policy_bytes = read_regular_file(repository_root, PHASE7_POLICY)?;
    let phase7_policy_text = std::str::from_utf8(&phase7_policy_bytes)
        .map_err(|error| DifferentialError::new("protocol", error.to_string()))?;
    let phase7_policy = Phase7PolicyProfile::parse_toml(phase7_policy_text)
        .map_err(|error| DifferentialError::new("policy", error.to_string()))?;
    let phase8_policy_bytes = read_regular_file(repository_root, PHASE8_POLICY)?;
    let phase8_policy_text = std::str::from_utf8(&phase8_policy_bytes)
        .map_err(|error| DifferentialError::new("protocol", error.to_string()))?;
    let phase8_policy = Phase8PolicyProfile::parse_toml(phase8_policy_text)
        .map_err(|error| DifferentialError::new("policy", error.to_string()))?;
    let request = rigid_world_request(repository_root, &phase8_policy)?;

    if invocation.action == MathProbeAction::VerifyDeterminism {
        return verify_rigid_world_determinism(
            repository_root,
            &request,
            &invocation.preset,
            invocation.runs,
        );
    }

    let captured = execute_rigid_world_once(repository_root, &request, &invocation.preset)?;
    let native = NativeRigidWorldExecutor::execute(&request)
        .map_err(|error| DifferentialError::new("native", error.to_string()))?;
    let outcome = compare_phase8_rigid_world_results(
        &request,
        &native,
        captured.result(),
        &phase6_policy,
        &phase7_policy,
        &phase8_policy,
    )
    .map_err(|error| {
        DifferentialError::new(
            "rigid-harness",
            serde_json::to_string(&error).unwrap_or_else(|_| format!("{error:?}")),
        )
    })?;
    match outcome {
        RigidComparisonOutcome::PhysicsMismatch(report)
            if invocation.action == MathProbeAction::Minimize =>
        {
            return run_rigid_world_minimization(
                repository_root,
                &request,
                report.signature(),
                &phase6_policy,
                &phase7_policy,
                &phase8_policy,
                &invocation.preset,
            );
        }
        RigidComparisonOutcome::PhysicsMismatch(report) => {
            return Err(DifferentialError::new(
                "physics-mismatch",
                String::from_utf8_lossy(
                    &report
                        .render_machine()
                        .map_err(|error| DifferentialError::new("report", error.to_string()))?,
                )
                .into_owned(),
            ));
        }
        RigidComparisonOutcome::Match if invocation.action == MathProbeAction::Minimize => {
            return Err(DifferentialError::new(
                "minimization",
                "rigid-world minimization requires a captured first-divergence signature",
            ));
        }
        RigidComparisonOutcome::Match => {}
    }

    let native_identity = EmptyWorldAdapter::new(ORACLE_REVISION)
        .map_err(|error| DifferentialError::new("identity", error.to_string()))?;
    let action = match invocation.action {
        MathProbeAction::Compare => "compare",
        MathProbeAction::Replay => "replay",
        MathProbeAction::Minimize | MathProbeAction::VerifyDeterminism => {
            unreachable!("handled before matched output")
        }
    };
    println!(
        "rigid-world {action}: {} required families matched under {} ({}); oracle={}, native={}",
        request.scenario().timelines().len(),
        phase8_policy.profile_id(),
        invocation.preset,
        build_evidence_label(captured.identity().evidence_tier()),
        build_evidence_label(native_identity.build_identity().evidence_tier()),
    );
    Ok(())
}
