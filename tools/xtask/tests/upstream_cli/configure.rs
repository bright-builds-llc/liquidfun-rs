use super::{REVISION, RepositoryFixture, TestResult, assert_failure_category, stderr};

#[test]
fn configure_rejects_unknown_preset() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args(["upstream", "configure", "--preset", "untrusted"]);

    // Act
    let output = command.output()?;

    // Assert
    assert_failure_category(&output, "upstream/preset");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn configure_passes_revision_and_adapter_digest_as_structured_arguments() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let expected_digest = fixture.adapter_source_digest()?;
    let mut command = fixture.command()?;
    command.args(["upstream", "configure", "--preset", "oracle-debug"]);

    // Act
    let output = command.output()?;

    // Assert
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        fixture.cmake_arguments()?,
        [
            "--preset".to_owned(),
            "oracle-debug".to_owned(),
            format!("-DLIQUIDFUN_EXPECTED_ORACLE_REVISION={REVISION}"),
            format!("-DLIQUIDFUN_EXPECTED_ADAPTER_SHA256={expected_digest}"),
        ]
    );
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn configure_digest_changes_when_vendored_json_header_changes() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let original_digest = fixture.adapter_source_digest()?;
    fixture.overwrite_adapter_input(
        "tools/reference/vendor/nlohmann/json.hpp",
        b"mutated vendored parser\n",
    )?;
    let changed_digest = fixture.adapter_source_digest()?;
    let mut command = fixture.command()?;
    command.args(["upstream", "configure", "--preset", "oracle-debug"]);

    // Act
    let output = command.output()?;

    // Assert
    assert_ne!(changed_digest, original_digest);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(fixture.cmake_arguments()?.contains(&format!(
        "-DLIQUIDFUN_EXPECTED_ADAPTER_SHA256={changed_digest}"
    )));
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn configure_rejects_extra_path_input_before_cmake() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args([
        "upstream",
        "configure",
        "--preset",
        "oracle-debug",
        "../untrusted",
    ]);

    // Act
    let output = command.output()?;

    // Assert
    assert_failure_category(&output, "upstream/usage");
    assert!(!fixture.cmake_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn configure_propagates_cmake_failure() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command
        .args(["upstream", "configure", "--preset", "oracle-debug"])
        .env("LIQUIDFUN_TEST_CMAKE_FAIL", "1");

    // Act
    let output = command.output()?;

    // Assert
    assert_failure_category(&output, "upstream/process");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn configure_retains_stdout_only_cmake_failure_diagnostics() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command
        .args(["upstream", "configure", "--preset", "oracle-debug"])
        .env("LIQUIDFUN_TEST_CMAKE_FAIL_STDOUT", "1");

    // Act
    let output = command.output()?;

    // Assert
    assert_failure_category(&output, "upstream/process");
    assert!(
        stderr(&output).contains("stdout:\nsimulated compiler failure on stdout"),
        "expected stdout-only compiler diagnostic in stderr: {}",
        stderr(&output)
    );
    fixture.cleanup()?;
    Ok(())
}
