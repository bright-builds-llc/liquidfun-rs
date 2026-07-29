use super::*;

pub(super) fn compare_passes_only_canonical_structured_arguments() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args([
        "differential",
        "compare",
        "--scenario",
        "empty-world",
        "--preset",
        "oracle-debug",
        "--session-profile",
        "one-shot",
    ]);

    // Act
    let output = command.output()?;

    // Assert
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        fixture.differential_arguments()?,
        [
            "compare",
            "--scenario",
            "empty-world",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "one-shot",
        ]
    );
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn replay_and_minimize_pass_only_named_scenario_arguments() -> TestResult {
    // Arrange
    for action in ["replay", "minimize"] {
        let fixture = RepositoryFixture::new()?;
        let mut command = fixture.command()?;
        command.args([
            "differential",
            action,
            "--scenario",
            "empty-world",
            "--preset",
            "oracle-release",
            "--session-profile",
            "reuse",
        ]);

        // Act
        let output = command.output()?;

        // Assert
        assert!(output.status.success(), "{}", stderr(&output));
        assert_eq!(
            fixture.differential_arguments()?,
            [
                action,
                "--scenario",
                "empty-world",
                "--preset",
                "oracle-release",
                "--session-profile",
                "reuse",
            ]
        );
        fixture.cleanup()?;
    }
    Ok(())
}

pub(super) fn math_probe_compare_and_replay_pass_only_reviewed_arguments() -> TestResult {
    // Arrange
    for action in ["compare", "replay"] {
        let fixture = RepositoryFixture::new()?;
        let mut command = fixture.command()?;
        let arguments = [
            "differential",
            action,
            "--scenario",
            "math-probes",
            "--preset",
            "oracle-release",
            "--session-profile",
            "one-shot",
        ];
        command.args(arguments);

        // Act
        let output = command.output()?;

        // Assert
        assert!(output.status.success(), "{}", stderr(&output));
        assert_eq!(fixture.differential_arguments()?, &arguments[1..]);
        fixture.cleanup()?;
    }
    Ok(())
}

pub(super) fn collision_compare_replay_and_determinism_pass_only_reviewed_arguments() -> TestResult
{
    // Arrange
    for action in ["compare", "replay"] {
        let fixture = RepositoryFixture::new()?;
        let mut command = fixture.command()?;
        let arguments = [
            "differential",
            action,
            "--scenario",
            "collision-probes",
            "--preset",
            "oracle-release",
            "--session-profile",
            "one-shot",
        ];
        command.args(arguments);

        // Act
        let output = command.output()?;

        // Assert
        assert!(output.status.success(), "{}", stderr(&output));
        assert_eq!(fixture.differential_arguments()?, &arguments[1..]);
        fixture.cleanup()?;
    }

    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args([
        "differential",
        "verify-determinism",
        "--scenario",
        "collision-probes",
        "--preset",
        "oracle-debug",
        "--runs",
        "2",
    ]);
    let output = command.output()?;
    assert!(output.status.success(), "{}", stderr(&output));
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn rigid_compare_replay_and_minimize_pass_only_reviewed_arguments() -> TestResult {
    // Arrange
    for action in ["compare", "replay", "minimize"] {
        let fixture = RepositoryFixture::new()?;
        let mut command = fixture.command()?;
        let arguments = [
            "differential",
            action,
            "--scenario",
            "rigid-world",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "one-shot",
        ];
        command.args(arguments);

        // Act
        let output = command.output()?;

        // Assert
        assert!(output.status.success(), "{}", stderr(&output));
        assert_eq!(fixture.differential_arguments()?, &arguments[1..]);
        fixture.cleanup()?;
    }
    Ok(())
}

pub(super) fn rigid_determinism_accepts_exactly_two_debug_runs() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    let arguments = [
        "differential",
        "verify-determinism",
        "--scenario",
        "rigid-world",
        "--preset",
        "oracle-debug",
        "--runs",
        "2",
    ];
    command.args(arguments);

    // Act
    let output = command.output()?;

    // Assert
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(fixture.differential_arguments()?, &arguments[1..]);
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn rigid_fixture_stage_passes_only_fixed_lifecycle_metadata() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    let arguments = [
        "differential",
        "fixture",
        "stage",
        "--scenario",
        "rigid-world",
        "--preset",
        "oracle-debug",
        "--session-profile",
        "one-shot",
        "--artifact-kind",
        "reviewed-trace",
        "--artifact-id",
        "rigid-trace-1",
    ];
    command.args(arguments);

    // Act
    let output = command.output()?;

    // Assert
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(fixture.differential_arguments()?, &arguments[1..]);
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn rigid_fixture_real_binary_accepts_d1_and_rejects_d2_before_effects() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    prepare_real_rigid_repository(&fixture.root, "rigid_d1")?;
    let real_differential = debug_binary("liquidfun-differential");
    let arguments = [
        "differential",
        "fixture",
        "stage",
        "--scenario",
        "rigid-world",
        "--preset",
        "oracle-debug",
        "--session-profile",
        "one-shot",
        "--artifact-kind",
        "reviewed-trace",
        "--artifact-id",
        "xtask-rigid-d1",
    ];
    let mut accepted = fixture.command()?;
    accepted
        .env("LIQUIDFUN_XTASK_DIFFERENTIAL", &real_differential)
        .args(arguments);

    // Act
    let accepted_output = accepted.output()?;
    fs::write(
        fixture
            .root
            .join("target/reference/oracle-debug/behavior.txt"),
        "rigid_d2",
    )?;
    let mut rejected = fixture.command()?;
    rejected
        .env("LIQUIDFUN_XTASK_DIFFERENTIAL", &real_differential)
        .args([
            "differential",
            "fixture",
            "stage",
            "--scenario",
            "rigid-world",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "one-shot",
            "--artifact-kind",
            "reviewed-trace",
            "--artifact-id",
            "xtask-rigid-d2",
        ]);
    let rejected_output = rejected.output()?;

    // Assert
    assert!(
        accepted_output.status.success(),
        "{}",
        stderr(&accepted_output)
    );
    assert!(
        fixture
            .root
            .join("target/differential/staging/xtask-rigid-d1/candidate.toml")
            .is_file()
    );
    assert!(!rejected_output.status.success());
    assert!(stderr(&rejected_output).contains("requires D1 canonical authority"));
    assert!(
        !fixture
            .root
            .join("target/differential/staging/xtask-rigid-d2")
            .exists()
    );
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn rigid_fixture_stale_identity_real_binary_rejects_before_effects() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    prepare_real_rigid_repository(&fixture.root, "rigid_d1_stale_adapter")?;
    let manifest_path = fixture.root.join("reference/artifacts/manifest.toml");
    let manifest_before = fs::read(&manifest_path)?;
    let real_differential = debug_binary("liquidfun-differential");
    let mut command = fixture.command()?;
    command
        .env("LIQUIDFUN_XTASK_DIFFERENTIAL", &real_differential)
        .args([
            "differential",
            "fixture",
            "stage",
            "--scenario",
            "rigid-world",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "one-shot",
            "--artifact-kind",
            "reviewed-trace",
            "--artifact-id",
            "xtask-rigid-stale-adapter",
        ]);

    // Act
    let output = command.output()?;

    // Assert
    assert!(!output.status.success());
    assert!(stderr(&output).contains("adapter digest differs from current checkout inputs"));
    assert!(!fixture.root.join("target/differential/staging").exists());
    assert_eq!(fs::read(manifest_path)?, manifest_before);
    assert!(
        !fixture
            .root
            .join("reference/artifacts/traces/phase-08-rigid-world-v1.jsonl")
            .exists()
    );
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn rigid_commands_reject_unreviewed_shapes_before_effects() -> TestResult {
    // Arrange
    let scenario_file_option = ["--scenario", "-file"].concat();
    let cases = [
        vec![
            "differential",
            "compare",
            "--scenario",
            "../rigid-world.jsonl",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "one-shot",
        ],
        vec![
            "differential",
            "compare",
            "--scenario",
            "rigid-world",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "reuse",
        ],
        vec![
            "differential",
            "verify-determinism",
            "--scenario",
            "rigid-world",
            "--preset",
            "oracle-debug",
            "--runs",
            "3",
        ],
        vec![
            "differential",
            "compare",
            "--scenario",
            "rigid-world",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "one-shot",
            scenario_file_option.as_str(),
            "../outside.jsonl",
        ],
    ];

    for arguments in cases {
        let fixture = RepositoryFixture::new()?;
        let mut command = fixture.command()?;
        command.args(arguments);

        // Act
        let output = command.output()?;

        // Assert
        assert_failure_category(&output, "differential/usage");
        assert!(!fixture.differential_marker.exists());
        fixture.cleanup()?;
    }
    Ok(())
}
