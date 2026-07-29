use super::*;

pub(super) fn rigid_child_failure_status_is_propagated() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command
        .args([
            "differential",
            "compare",
            "--scenario",
            "rigid-world",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "one-shot",
        ])
        .env("LIQUIDFUN_TEST_DIFFERENTIAL_FAIL", "1");

    // Act
    let output = command.output()?;

    // Assert
    assert_failure_category(&output, "differential/process");
    assert!(stderr(&output).contains("status 42"));
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn rigid_ci_commands_stay_in_the_native_reference_workflow() {
    // Arrange
    let native_reference_workflow = include_str!("../../../../.github/workflows/oracle.yml");
    let cargo_workflow = include_str!("../../../../.github/workflows/ci.yml");
    let required = [
        "cargo xtask differential compare --scenario rigid-world --preset oracle-debug --session-profile one-shot",
        "cargo xtask differential compare --scenario rigid-world --preset oracle-release --session-profile one-shot",
        "cargo xtask differential replay --scenario rigid-world --preset oracle-debug --session-profile one-shot",
        "cargo xtask differential verify-determinism --scenario rigid-world --preset oracle-debug --runs 2",
    ];

    // Act
    let all_reference_commands_present = required
        .iter()
        .all(|command| native_reference_workflow.contains(command));
    let cargo_only_isolated = ["submodules: recursive", "cmake", "oracle", "rigid-world"]
        .iter()
        .all(|forbidden| !cargo_workflow.to_ascii_lowercase().contains(forbidden));

    // Assert
    assert!(all_reference_commands_present);
    assert!(cargo_only_isolated);
}

pub(super) fn sanitizer_rigid_protocol_and_compare_run_before_read_only_assertion() {
    // Arrange
    let workflow = include_str!("../../../../.github/workflows/oracle.yml");
    let sanitizer_job = workflow
        .split("  sanitizer-linux:")
        .nth(1)
        .and_then(|suffix| suffix.split("  portability-macos:").next())
        .expect("sanitizer job must remain in the Oracle workflow");
    let build = "cargo xtask upstream build --preset oracle-asan-ubsan";
    let protocol_build = "cmake --build target/reference/oracle-asan-ubsan --target liquidfun-reference-protocol-tests";
    let protocol = "ctest --test-dir target/reference/oracle-asan-ubsan";
    let rigid = "cargo xtask differential compare --scenario rigid-world --preset oracle-asan-ubsan --session-profile one-shot";
    let read_only = "git diff --exit-code -- protocol scenarios reference COMPATIBILITY.md";

    // Act
    let positions = [build, protocol_build, protocol, rigid, read_only]
        .map(|marker| sanitizer_job.find(marker));

    // Assert
    assert!(positions.iter().all(Option::is_some));
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
}

pub(super) fn sanitizer_rigid_commands_use_fail_fast_environment_without_status_suppression() {
    // Arrange
    let workflow = include_str!("../../../../.github/workflows/oracle.yml");
    let sanitizer_job = workflow
        .split("  sanitizer-linux:")
        .nth(1)
        .and_then(|suffix| suffix.split("  portability-macos:").next())
        .expect("sanitizer job must remain in the Oracle workflow");
    let fail_fast = "UBSAN_OPTIONS=halt_on_error=1:print_stacktrace=1 ASAN_OPTIONS=abort_on_error=1:halt_on_error=1";
    let required_commands = [
        "ctest --test-dir target/reference/oracle-asan-ubsan",
        "cargo xtask differential compare --scenario rigid-world --preset oracle-asan-ubsan --session-profile one-shot",
    ];

    // Act / Assert
    for command in required_commands {
        let expected = format!("{fail_fast} {command}");
        assert!(sanitizer_job.contains(&expected), "missing `{expected}`");
    }
    assert!(!sanitizer_job.contains("continue-on-error:"));
    assert!(!sanitizer_job.contains("|| true"));
    assert!(!sanitizer_job.contains("|| echo"));
}

pub(super) fn sanitizer_rigid_compare_passes_only_the_reviewed_one_shot_shape() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    let arguments = [
        "differential",
        "compare",
        "--scenario",
        "rigid-world",
        "--preset",
        "oracle-asan-ubsan",
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
    Ok(())
}

pub(super) fn collision_compile_database_identity_is_covered_by_unit_digest_tests() {
    // The unit digest fixtures name collision_probe.cpp explicitly and are run by this filter.
    assert!(
        include_str!("../../../../crates/liquidfun-differential/src/oracle_identity.rs")
            .contains("collision_probe.cpp")
    );
}

pub(super) fn collision_required_families_are_validated_before_oracle_execution() {
    let bytes =
        include_bytes!("../../../../protocol/fixtures/accepted/collision-probe-request.jsonl");
    let request = liquidfun_test_protocol::decode_collision_probe_request_jsonl(
        bytes,
        &liquidfun_test_protocol::HarnessLimits::phase2_default_v1(),
    )
    .expect("checked-in collision corpus should be fail-closed and complete");
    for family in liquidfun_test_protocol::CollisionWitnessFamily::REQUIRED {
        assert!(
            request
                .scenario()
                .cases()
                .iter()
                .any(|case| case.witness_family() == family)
        );
    }
}

pub(super) fn math_probe_determinism_accepts_only_two_reviewed_runs() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args([
        "differential",
        "verify-determinism",
        "--scenario",
        "math-probes",
        "--preset",
        "oracle-debug",
        "--runs",
        "2",
    ]);

    // Act
    let output = command.output()?;

    // Assert
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        fixture.differential_arguments()?,
        [
            "verify-determinism",
            "--scenario",
            "math-probes",
            "--preset",
            "oracle-debug",
            "--runs",
            "2",
        ]
    );
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn math_probe_commands_reject_unreviewed_profiles_and_run_counts() -> TestResult {
    // Arrange
    let cases = [
        vec![
            "differential",
            "compare",
            "--scenario",
            "math-probes",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "reuse",
        ],
        vec![
            "differential",
            "verify-determinism",
            "--scenario",
            "math-probes",
            "--preset",
            "oracle-debug",
            "--runs",
            "3",
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

pub(super) fn fixture_stage_passes_only_lifecycle_metadata() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args([
        "differential",
        "fixture",
        "stage",
        "--scenario",
        "empty-world",
        "--preset",
        "oracle-debug",
        "--session-profile",
        "one-shot",
        "--artifact-kind",
        "reviewed-trace",
        "--artifact-id",
        "trace-1",
    ]);

    // Act
    let output = command.output()?;

    // Assert
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        fixture.differential_arguments()?,
        [
            "fixture",
            "stage",
            "--scenario",
            "empty-world",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "one-shot",
            "--artifact-kind",
            "reviewed-trace",
            "--artifact-id",
            "trace-1",
        ]
    );
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn fixture_review_and_promote_pass_only_candidate_metadata() -> TestResult {
    // Arrange
    let cases = [
        vec![
            "fixture",
            "review",
            "--artifact-id",
            "trace-1",
            "--reviewer",
            "maintainer",
            "--reviewed-at",
            "2026-07-10T11:30:00Z",
            "--review-status",
            "approved",
        ],
        vec!["fixture", "promote", "--artifact-id", "trace-1"],
    ];

    for arguments in cases {
        let fixture = RepositoryFixture::new()?;
        let mut command = fixture.command()?;
        command.arg("differential").args(&arguments);

        // Act
        let output = command.output()?;

        // Assert
        assert!(output.status.success(), "{}", stderr(&output));
        assert_eq!(fixture.differential_arguments()?, arguments);
        fixture.cleanup()?;
    }
    Ok(())
}

pub(super) fn compare_rejects_unregistered_scenario_before_starting_runner() -> TestResult {
    assert_rejected_value("--scenario", "../outside")
}

pub(super) fn compare_rejects_unregistered_preset_before_starting_runner() -> TestResult {
    assert_rejected_value("--preset", "untrusted")
}

pub(super) fn compare_rejects_unregistered_profile_before_starting_runner() -> TestResult {
    assert_rejected_value("--session-profile", "unbounded")
}

pub(super) fn replay_rejects_arbitrary_request_path_before_starting_runner() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args([
        "differential",
        "replay",
        "--exact-request",
        "../outside.jsonl",
        "--preset",
        "oracle-debug",
        "--session-profile",
        "one-shot",
    ]);

    // Act
    let output = command.output()?;

    // Assert
    assert_failure_category(&output, "differential/usage");
    assert!(!fixture.differential_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn compare_rejects_extra_option_before_starting_runner() -> TestResult {
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
        "--output",
        "../outside.jsonl",
    ]);

    // Act
    let output = command.output()?;

    // Assert
    assert_failure_category(&output, "differential/usage");
    assert!(!fixture.differential_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn compare_propagates_differential_child_failure() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command
        .args([
            "differential",
            "compare",
            "--scenario",
            "empty-world",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "one-shot",
        ])
        .env("LIQUIDFUN_TEST_DIFFERENTIAL_FAIL", "1");

    // Act
    let output = command.output()?;

    // Assert
    assert_failure_category(&output, "differential/process");
    assert!(stderr(&output).contains("status 42"));
    assert!(stderr(&output).contains("simulated differential failure"));
    fixture.cleanup()?;
    Ok(())
}
