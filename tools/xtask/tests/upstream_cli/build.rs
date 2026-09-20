use super::{RepositoryFixture, TestResult, assert_failure_category, stderr};

#[test]
fn upstream_tests_build_selects_the_complete_test_suite() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args(["upstream", "build", "--preset", "upstream-tests"]);
    // Act
    let output = command.output()?;
    // Assert
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        fixture.cmake_arguments()?,
        [
            "--build",
            "--preset",
            "upstream-tests",
            "--target",
            "liquidfun-upstream-tests"
        ]
    );
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn build_targets_only_the_registered_reference_executable() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args(["upstream", "build", "--preset", "oracle-debug"]);

    // Act
    let output = command.output()?;

    // Assert
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        fixture.cmake_arguments()?,
        [
            "--build",
            "--preset",
            "oracle-debug",
            "--target",
            "liquidfun-reference",
        ]
    );
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn build_accepts_the_registered_phase9_lifecycle_contact_witness() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args([
        "upstream",
        "build",
        "--preset",
        "oracle-debug",
        "--target",
        "phase9-lifecycle-contact-witness",
    ]);

    // Act
    let output = command.output()?;

    // Assert
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        fixture.cmake_arguments()?,
        [
            "--build",
            "--preset",
            "oracle-debug",
            "--target",
            "phase9-lifecycle-contact-witness",
        ]
    );
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn build_accepts_the_registered_phase10_group_topology_witness() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args([
        "upstream",
        "build",
        "--preset",
        "oracle-debug",
        "--target",
        "phase10-group-topology-witness",
    ]);

    // Act
    let output = command.output()?;

    // Assert
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        fixture.cmake_arguments()?,
        [
            "--build",
            "--preset",
            "oracle-debug",
            "--target",
            "phase10-group-topology-witness",
        ]
    );
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn build_accepts_the_registered_playground_dam_break_bench() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args([
        "upstream",
        "build",
        "--preset",
        "oracle-release",
        "--target",
        "playground-dam-break-bench",
    ]);

    // Act
    let output = command.output()?;

    // Assert
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        fixture.cmake_arguments()?,
        [
            "--build",
            "--preset",
            "oracle-release",
            "--target",
            "playground-dam-break-bench",
        ]
    );
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn build_rejects_unregistered_target_before_cmake() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args([
        "upstream",
        "build",
        "--preset",
        "oracle-debug",
        "--target",
        "untrusted-target",
    ]);

    // Act
    let output = command.output()?;

    // Assert
    assert_failure_category(&output, "upstream/target");
    assert!(!fixture.cmake_marker.exists());
    fixture.cleanup()?;
    Ok(())
}
