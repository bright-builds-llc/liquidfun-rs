#[test]
fn transaction_real_binary_stages_replays_and_promotes_canonical_rigid_trace()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1")?;

    // Act
    let staged = repository.stage("canonical-rigid")?;
    let reviewed = repository.review("canonical-rigid")?;
    let promoted = repository.promote("canonical-rigid")?;

    // Assert
    assert!(staged.status.success(), "{}", stderr(&staged));
    assert!(reviewed.status.success(), "{}", stderr(&reviewed));
    assert!(promoted.status.success(), "{}", stderr(&promoted));
    assert!(
        repository
            .candidate("canonical-rigid")
            .join("review.toml")
            .is_file()
    );
    assert!(
        repository
            .root
            .join("reference/artifacts/traces/phase-08-rigid-world-v1.jsonl")
            .is_file()
    );
    let manifest = fs::read_to_string(repository.root.join("reference/artifacts/manifest.toml"))?;
    assert!(manifest.contains("phase-08-rigid-world-v1.jsonl"));
    Ok(())
}

#[test]
fn real_binary_rejects_d2_before_staging_or_accepted_mutation()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d2")?;
    let manifest_before = fs::read(repository.root.join("reference/artifacts/manifest.toml"))?;

    // Act
    let output = repository.stage("noncanonical-rigid")?;

    // Assert
    assert!(!output.status.success());
    assert!(stderr(&output).contains("requires D1 canonical authority"));
    assert!(!repository.candidate("noncanonical-rigid").exists());
    assert!(!repository.root.join("target/differential/staging").exists());
    assert_eq!(
        fs::read(repository.root.join("reference/artifacts/manifest.toml"))?,
        manifest_before
    );
    assert!(
        !repository
            .root
            .join("reference/artifacts/traces/phase-08-rigid-world-v1.jsonl")
            .exists()
    );
    Ok(())
}

#[test]
fn stale_request_policy_rejects_before_staging_or_oracle_execution()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1")?;
    repository.set_request_policy_hash(&"0".repeat(64))?;
    repository.set_behavior("rigid_d1_nonzero")?;
    let before = FixtureMutationSnapshot::capture(&repository)?;

    // Act
    let error = stage_rigid_candidate(
        &repository.root,
        "stale-policy",
        ArtifactKind::ReviewedTrace,
        OraclePreset::Debug,
        "oracle-debug",
        "one-shot",
        "7f20402173fd143a3988c921bc384459c6a858f2",
        None,
    )
    .expect_err("stale request provenance must fail closed");

    // Assert
    assert!(error.to_string().contains("request policy hash"));
    assert_eq!(FixtureMutationSnapshot::capture(&repository)?, before);
    assert!(!repository.candidate("stale-policy").exists());
    Ok(())
}

#[test]
fn stale_adapter_real_binary_rejects_without_fixture_mutation()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1_stale_adapter")?;
    let before = FixtureMutationSnapshot::capture(&repository)?;

    // Act
    let output = repository.stage("stale-adapter")?;

    // Assert
    assert!(!output.status.success());
    let diagnostic = stderr(&output);
    assert!(diagnostic.contains("adapter digest differs from current checkout inputs"));
    assert!(diagnostic.len() < 1024);
    assert!(!diagnostic.contains(repository.root.to_string_lossy().as_ref()));
    assert_eq!(FixtureMutationSnapshot::capture(&repository)?, before);
    assert!(!repository.candidate("stale-adapter").exists());
    Ok(())
}

#[test]
fn stale_compile_real_binary_rejects_without_fixture_mutation()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1_stale_compile")?;
    let before = FixtureMutationSnapshot::capture(&repository)?;

    // Act
    let output = repository.stage("stale-compile")?;

    // Assert
    assert!(!output.status.success());
    let diagnostic = stderr(&output);
    assert!(diagnostic.contains("compile-command digest differs"));
    assert!(diagnostic.len() < 1024);
    assert!(!diagnostic.contains(repository.root.to_string_lossy().as_ref()));
    assert_eq!(FixtureMutationSnapshot::capture(&repository)?, before);
    assert!(!repository.candidate("stale-compile").exists());
    Ok(())
}

#[test]
fn review_and_promotion_recompute_checkout_identity_before_writes()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1")?;
    let staged = repository.stage("identity-drift")?;
    assert!(staged.status.success(), "{}", stderr(&staged));
    let adapter_path = repository.adapter_input();
    let adapter_before = fs::read(&adapter_path)?;
    fs::write(&adapter_path, b"fixture adapter interface changed\n")?;
    let before_review = FixtureMutationSnapshot::capture(&repository)?;

    // Act
    let stale_review = repository.review("identity-drift")?;

    // Assert
    assert!(!stale_review.status.success());
    assert!(stderr(&stale_review).contains("adapter digest differs"));
    assert_eq!(
        FixtureMutationSnapshot::capture(&repository)?,
        before_review
    );
    assert!(
        !repository
            .candidate("identity-drift")
            .join("review.toml")
            .exists()
    );

    // Arrange
    fs::write(&adapter_path, adapter_before)?;
    let reviewed = repository.review("identity-drift")?;
    assert!(reviewed.status.success(), "{}", stderr(&reviewed));
    let compile_path = repository.compile_database();
    let compile = fs::read_to_string(&compile_path)?;
    fs::write(
        &compile_path,
        compile.replace("-DREVIEWED=1", "-DREVIEWED=2"),
    )?;
    let before_promotion = FixtureMutationSnapshot::capture(&repository)?;

    // Act
    let stale_promotion = repository.promote("identity-drift")?;

    // Assert
    assert!(!stale_promotion.status.success());
    assert!(stderr(&stale_promotion).contains("compile-command digest differs"));
    assert_eq!(
        FixtureMutationSnapshot::capture(&repository)?,
        before_promotion
    );
    assert!(
        repository
            .candidate("identity-drift")
            .join("review.toml")
            .is_file()
    );
    assert!(
        !repository
            .root
            .join("reference/artifacts/traces/phase-08-rigid-world-v1.jsonl")
            .exists()
    );
    Ok(())
}

#[test]
fn transaction_replay_rejects_dirty_rigid_candidate_before_review()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1")?;
    let staged = repository.stage("dirty-rigid")?;
    assert!(staged.status.success(), "{}", stderr(&staged));
    fs::write(
        repository.candidate("dirty-rigid").join("trace.jsonl"),
        b"tampered\n",
    )?;

    // Act
    let reviewed = repository.review("dirty-rigid")?;

    // Assert
    assert!(!reviewed.status.success());
    assert!(stderr(&reviewed).contains("SHA-256 mismatch"));
    assert!(
        !repository
            .candidate("dirty-rigid")
            .join("review.toml")
            .exists()
    );
    Ok(())
}

#[test]
fn real_binary_propagates_rigid_child_failure_without_candidate_creation()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1")?;
    repository.set_behavior("rigid_d1_nonzero")?;

    // Act
    let output = repository.stage("failed-child")?;

    // Assert
    assert!(!output.status.success());
    assert!(!repository.candidate("failed-child").exists());
    Ok(())
}
