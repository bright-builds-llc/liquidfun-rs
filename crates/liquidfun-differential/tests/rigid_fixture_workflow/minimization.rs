#[test]
fn minimized_regression_requires_reduced_complete_provenance()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1_mismatch")?;
    let original_bytes = fs::read(
        repository
            .root
            .join("protocol/fixtures/accepted/rigid-world-request.jsonl"),
    )?;
    let minimization = rigid_minimization(&repository.root, 4_096)?;
    assert_eq!(minimization.status(), MinimizationStatus::Complete);
    let minimized_native = NativeRigidWorldExecutor::execute(minimization.request())?;
    let minimized_value = serde_json::to_value(&minimized_native)?;
    assert!(
        minimized_value
            .pointer("/timelines/0/checkpoints/0/bodies/0/active")
            .is_some(),
        "the minimized request must retain the divergent body"
    );
    assert_fake_oracle_accepts(&repository, minimization.canonical_request_bytes())?;

    // Act
    let candidate = stage_rigid_candidate(
        &repository.root,
        "minimized-rigid",
        ArtifactKind::MinimizedRegression,
        OraclePreset::Debug,
        "oracle-debug",
        "one-shot",
        &git_head(&repository.root)?,
        Some(&minimization),
    )?;
    let reviewed = repository.review("minimized-rigid")?;

    // Assert
    let staged_request = fs::read(candidate.directory().join("request.jsonl"))?;
    assert_eq!(staged_request, minimization.canonical_request_bytes());
    assert_ne!(staged_request, original_bytes);
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(candidate.directory().join("report.json"))?)?;
    assert_eq!(report["status"], "complete");
    assert_eq!(
        report["target_signature_sha256"],
        minimization.target_signature().signature_sha256().as_str()
    );
    assert!(
        report["attempted_transforms"]
            .as_array()
            .is_some_and(|transforms| !transforms.is_empty())
    );
    assert!(
        report["accepted_transforms"]
            .as_array()
            .is_some_and(|transforms| !transforms.is_empty())
    );
    assert!(reviewed.status.success(), "{}", stderr(&reviewed));
    Ok(())
}

#[test]
fn minimized_regression_rejects_full_or_incomplete_requests_without_mutation()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1_mismatch")?;
    let incomplete = rigid_minimization(&repository.root, 0)?;
    assert!(matches!(
        incomplete.status(),
        MinimizationStatus::Incomplete(_)
    ));
    let before = FixtureMutationSnapshot::capture(&repository)?;

    // Act
    let missing = stage_rigid_candidate(
        &repository.root,
        "missing-minimization",
        ArtifactKind::MinimizedRegression,
        OraclePreset::Debug,
        "oracle-debug",
        "one-shot",
        &git_head(&repository.root)?,
        None,
    )
    .expect_err("a full request must not be labeled as minimized");
    let unfinished = stage_rigid_candidate(
        &repository.root,
        "incomplete-minimization",
        ArtifactKind::MinimizedRegression,
        OraclePreset::Debug,
        "oracle-debug",
        "one-shot",
        &git_head(&repository.root)?,
        Some(&incomplete),
    )
    .expect_err("an incomplete reduction must not be staged");

    // Assert
    assert!(
        missing
            .to_string()
            .contains("completed minimization result")
    );
    assert!(unfinished.to_string().contains("complete reduction"));
    assert_eq!(FixtureMutationSnapshot::capture(&repository)?, before);
    Ok(())
}

fn rigid_minimization(
    root: &Path,
    max_attempts: usize,
) -> Result<RigidMinimizationResult, Box<dyn std::error::Error>> {
    let effective_max_attempts = max_attempts.saturating_mul(4);
    let limits = HarnessLimits::phase2_default_v1();
    let request_bytes =
        fs::read(root.join("protocol/fixtures/accepted/rigid-world-request.jsonl"))?;
    let request = decode_rigid_world_request_jsonl(&request_bytes, &limits)?;
    let phase6 = Phase6PolicyProfile::parse_toml(&fs::read_to_string(
        root.join("protocol/tolerances/phase6-v1.toml"),
    )?)?;
    let phase7 = Phase7PolicyProfile::parse_toml(&fs::read_to_string(
        root.join("protocol/tolerances/phase7-v1.toml"),
    )?)?;
    let phase8 = Phase8PolicyProfile::parse_toml(&fs::read_to_string(
        root.join("protocol/tolerances/phase8-v1.toml"),
    )?)?;
    let Some(target) = rigid_mismatch_signature(&request, &phase6, &phase7, &phase8) else {
        return Err(io::Error::other("injected rigid mismatch unexpectedly matched").into());
    };
    minimize_rigid_world_request(
        &request,
        &target,
        MinimizationBudget::new(effective_max_attempts, Duration::from_secs(1)),
        |candidate| {
            RigidEvaluation::new(
                rigid_mismatch_signature(candidate, &phase6, &phase7, &phase8),
                Duration::ZERO,
            )
        },
    )
    .map_err(Into::into)
}

fn rigid_mismatch_signature(
    request: &RigidWorldRequestRecord,
    phase6: &Phase6PolicyProfile,
    phase7: &Phase7PolicyProfile,
    phase8: &Phase8PolicyProfile,
) -> Option<RigidFailureSignature> {
    let limits = HarnessLimits::phase2_default_v1();
    let native = NativeRigidWorldExecutor::execute(request).ok()?;
    let mut oracle_value = serde_json::to_value(&native).ok()?;
    let active = oracle_value.pointer_mut("/timelines/0/checkpoints/0/bodies/0/active")?;
    *active = serde_json::Value::Bool(false);
    let mut oracle_bytes = serde_json::to_vec(&oracle_value).ok()?;
    oracle_bytes.push(b'\n');
    let oracle = decode_rigid_world_result_jsonl(&oracle_bytes, &limits).ok()?;
    let outcome =
        compare_phase8_rigid_world_results(request, &native, &oracle, phase6, phase7, phase8)
            .ok()?;
    let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
        return None;
    };
    Some(report.signature().clone())
}

fn assert_fake_oracle_accepts(
    repository: &RigidFixtureRepository,
    request_bytes: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new(repository.oracle_directory.join(oracle_name()))
        .current_dir(&repository.root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or_else(|| io::Error::other("fake oracle stdin is unavailable"))?
        .write_all(request_bytes)?;
    let output = child.wait_with_output()?;
    if output.status.success() {
        return Ok(());
    }
    Err(io::Error::other(format!(
        "fake oracle rejected minimized request: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    ))
    .into())
}
