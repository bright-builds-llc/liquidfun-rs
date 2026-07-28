use super::*;

pub(super) fn check_rejects_unknown_schema_fields() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let mut entries = InventoryFixture::valid_entries();
    entries[0]["unknown"] = json!(true);
    fixture.write_compatibility(&entries)?;

    // Act
    let output = fixture.check()?;

    // Assert
    assert_failure_category(&output, "inventory/schema");
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn check_rejects_duplicate_stable_ids() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let mut entries = InventoryFixture::valid_entries();
    entries.insert(1, entries[0].clone());
    fixture.write_compatibility(&entries)?;

    // Act
    let output = fixture.check()?;

    // Assert
    assert_failure_category(&output, "inventory/duplicate-id");
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn check_rejects_distinct_ids_for_the_same_upstream_mapping() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let mut entries = InventoryFixture::valid_entries();
    let mut duplicate_mapping = entries[0].clone();
    duplicate_mapping["id"] = json!("example.hello-world-copy");
    entries.insert(1, duplicate_mapping);
    fixture.write_compatibility(&entries)?;

    // Act
    let output = fixture.check()?;

    // Assert
    assert_failure_category(&output, "inventory/duplicate-mapping");
    let error = stderr(&output);
    assert!(error.contains("example.hello-world"));
    assert!(error.contains("example.hello-world-copy"));
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn check_rejects_unmapped_discovery_entries() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let mut entries = InventoryFixture::valid_entries();
    let maybe_removed_public_api = entries.pop();
    assert!(maybe_removed_public_api.is_some());
    fixture.write_compatibility(&entries)?;

    // Act
    let output = fixture.check()?;

    // Assert
    assert_failure_category(&output, "inventory/unmapped-discovery");
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn check_rejects_differential_evidence_without_dependencies() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let mut entries = InventoryFixture::valid_entries();
    entries[1]["evidence"]["implemented"] = not_evidenced();
    entries[1]["evidence"]["unit_tested"] = not_evidenced();
    fixture.write_compatibility(&entries)?;

    // Act
    let output = fixture.check()?;

    // Assert
    assert_failure_category(&output, "inventory/evidence");
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn check_rejects_omitted_release_identity() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let entries = InventoryFixture::valid_entries();
    let mut dispositions = release_dispositions(&entries);
    let maybe_removed = dispositions.pop();
    assert!(maybe_removed.is_some());
    fixture.write_ledger(&entries, &dispositions)?;

    // Act
    let output = fixture.check()?;

    // Assert
    assert_failure_category(&output, "inventory/release-join");
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn check_rejects_duplicate_release_identity() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let entries = InventoryFixture::valid_entries();
    let mut dispositions = release_dispositions(&entries);
    dispositions.insert(1, dispositions[0].clone());
    fixture.write_ledger(&entries, &dispositions)?;

    // Act
    let output = fixture.check()?;

    // Assert
    assert_failure_category(&output, "inventory/release-join");
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn check_rejects_unexplained_release_row() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let entries = InventoryFixture::valid_entries();
    let mut dispositions = release_dispositions(&entries);
    dispositions[0]["outcome"] = json!("d1_canonical");
    dispositions[0]["references"] = json!(["fixture"]);
    fixture.write_ledger(&entries, &dispositions)?;

    // Act
    let output = fixture.check()?;

    // Assert
    assert_failure_category(&output, "inventory/release-outcome");
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn check_rejects_nonterminal_corpus_item() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let mut item = terminal_corpus_item();
    let Some(object) = item.as_object_mut() else {
        panic!("terminal corpus fixture must be a JSON object");
    };
    object.remove("applicability");
    object.remove("disposition");
    object.remove("compatibility_impact");
    object.remove("evidence");
    object.remove("review");
    fixture.write_corpus(&item)?;

    // Act
    let output = fixture.check()?;

    // Assert
    assert_failure_category(&output, "inventory/corpus-terminal-outcome");
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn check_rejects_mixed_commit_parity_evidence() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let mut entries = InventoryFixture::valid_entries();
    entries[1]["evidence"]["implemented"]["references"] =
        json!(["https://example.test/commit/1111111111111111111111111111111111111111"]);
    entries[1]["evidence"]["differentially_validated"]["references"] =
        json!(["https://example.test/commit/2222222222222222222222222222222222222222"]);
    fixture.write_compatibility(&entries)?;

    // Act
    let output = fixture.check()?;

    // Assert
    assert_failure_category(&output, "inventory/release-commit");
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn check_rejects_empty_release_rationale() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let entries = InventoryFixture::valid_entries();
    let mut dispositions = release_dispositions(&entries);
    dispositions[0]["rationale"] = json!("");
    fixture.write_ledger(&entries, &dispositions)?;

    // Act
    let output = fixture.check()?;

    // Assert
    assert_failure_category(&output, "inventory/release-rationale");
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn check_rejects_coverage_as_parity_authority() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let mut entries = InventoryFixture::valid_entries();
    entries[1]["evidence"]["differentially_validated"]["references"] =
        json!(["reference/coverage/contract.json"]);
    fixture.write_compatibility(&entries)?;

    // Act
    let output = fixture.check()?;

    // Assert
    assert_failure_category(&output, "inventory/release-authority");
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn check_rejects_d2_as_parity_authority() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let mut entries = InventoryFixture::valid_entries();
    entries[1]["evidence"]["differentially_validated"]["references"] =
        json!(["reference/platform/support.json"]);
    fixture.write_compatibility(&entries)?;

    // Act
    let output = fixture.check()?;

    // Assert
    assert_failure_category(&output, "inventory/release-authority");
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn discover_and_generate_are_byte_deterministic() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    assert_success(&fixture.generate()?);
    let first_discovery = fs::read(fixture.root.join("reference/discovery.json"))?;
    let first_report = fs::read(fixture.root.join("COMPATIBILITY.md"))?;

    // Act
    assert_success(&fixture.discover()?);
    assert_success(&fixture.generate()?);
    let second_discovery = fs::read(fixture.root.join("reference/discovery.json"))?;
    let second_report = fs::read(fixture.root.join("COMPATIBILITY.md"))?;

    // Assert
    assert_eq!(first_discovery, second_discovery);
    assert_eq!(first_report, second_report);
    assert_success(&fixture.check()?);
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn report_check_uses_validated_ledgers_without_native_sources() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    assert_success(&fixture.generate()?);
    fs::remove_dir_all(fixture.root.join("third_party"))?;

    // Act
    let report_output = fixture.check_report()?;
    let full_output = fixture.check()?;

    // Assert
    assert_success(&report_output);
    assert_failure_category(&full_output, "inventory/discovery");
    fixture.cleanup()?;
    Ok(())
}
