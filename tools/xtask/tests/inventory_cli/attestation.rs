use super::*;

#[test]
fn invalid_explicit_attestation_cannot_overwrite_generated_report() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    assert_success(&fixture.generate()?);
    let report = fixture.root.join("COMPATIBILITY.md");
    let before = fs::read(&report)?;

    // Act
    let output = fixture.command(&["inventory", "generate", "--attestation-commit", "HEAD"])?;

    // Assert
    assert_failure_category(&output, "inventory/attestation");
    assert_eq!(before, fs::read(&report)?);
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn default_generation_stays_non_ready_without_attestation() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);

    // Act
    assert_success(&fixture.generate()?);
    let report = fs::read_to_string(fixture.root.join("COMPATIBILITY.md"))?;

    // Assert
    assert!(report.contains("Status: **not release-ready**"));
    assert!(!report.contains("Attestation commit:"));
    assert_success(&fixture.check_report()?);
    fixture.cleanup()?;
    Ok(())
}
