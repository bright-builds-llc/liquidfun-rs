use super::{
    RepositoryFixture, TestResult, WRONG_REVISION, assert_failure_category, stderr, stdout,
};

#[test]
fn verify_accepts_matching_upstream_identity() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args(["upstream", "verify"]);

    // Act
    let output = command.output()?;

    // Assert
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(stdout(&output).contains("upstream verified:"));
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn verify_rejects_wrong_lock_sha() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    fixture.write_lock(WRONG_REVISION)?;
    let mut command = fixture.command()?;
    command.args(["upstream", "verify"]);

    // Act
    let output = command.output()?;

    // Assert
    assert_failure_category(&output, "upstream/identity");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn verify_rejects_dirty_checkout() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command
        .args(["upstream", "verify"])
        .env("LIQUIDFUN_TEST_DIRTY", "1");

    // Act
    let output = command.output()?;

    // Assert
    assert_failure_category(&output, "upstream/dirty");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn verify_rejects_missing_submodule() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    fixture.remove_submodule()?;
    let mut command = fixture.command()?;
    command.args(["upstream", "verify"]);

    // Act
    let output = command.output()?;

    // Assert
    assert_failure_category(&output, "upstream/missing-submodule");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn verify_rejects_origin_url_mismatch() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command
        .args(["upstream", "verify"])
        .env("LIQUIDFUN_TEST_REMOTE_URL", "https://example.com/fork.git");

    // Act
    let output = command.output()?;

    // Assert
    assert_failure_category(&output, "upstream/identity");
    fixture.cleanup()?;
    Ok(())
}
