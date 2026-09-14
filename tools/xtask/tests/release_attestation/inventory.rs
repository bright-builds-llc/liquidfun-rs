use super::*;
use complete_fixture::{Checkout, assert_pass};

#[test]
fn attested_native_check_still_requires_the_upstream_discovery_scan() {
    // Arrange
    let fixture = Checkout::new("inventory-native-scan");
    fixture.materialize();
    let attestation = fixture.attest();
    fixture.project(&attestation);

    // Act
    let output = fixture.command(&["inventory", "check", "--attestation-commit", &attestation]);

    // Assert
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("third_party"));
}

#[test]
fn explicit_inventory_generation_is_deterministic_and_checks_from_later_d() {
    // Arrange
    let fixture = Checkout::new("inventory-projection");
    fixture.materialize();
    let attestation = fixture.attest();
    fixture.project(&attestation);
    let before = fs::read(fixture.root.join("COMPATIBILITY.md")).expect("generated report");
    fixture.commit(
        &["README.md", "COMPATIBILITY.md", "RELEASE.md"],
        "generated D",
    );

    // Act
    let generate = fixture.command(&[
        "inventory",
        "generate",
        "--attestation-commit",
        &attestation,
    ]);
    let check = fixture.command(&[
        "inventory",
        "check-report",
        "--attestation-commit",
        &attestation,
    ]);
    let docs = fixture.command(&["docs", "check", "--attestation-commit", &attestation]);

    // Assert
    assert_pass(&generate);
    assert_pass(&check);
    assert_pass(&docs);
    assert!(String::from_utf8_lossy(&before).contains(&format!(
        "Status: **release-ready**\n\nSource candidate: `{}`\n\nAttestation commit: `{attestation}`\n",
        fixture.candidate
    )));
    assert_eq!(
        before,
        fs::read(fixture.root.join("COMPATIBILITY.md")).expect("report")
    );
    assert!(!fixture.root.join("third_party").exists());
}

#[test]
fn later_discovery_changes_cannot_be_rendered_as_accepted_source() {
    // Arrange
    let fixture = Checkout::new("inventory-source-substitution");
    fixture.materialize();
    let attestation = fixture.attest();
    let report = fixture.root.join("COMPATIBILITY.md");
    let before = fs::read(&report).expect("report");
    let discovery = fixture.root.join("reference/discovery.json");
    let mut changed = fs::read(&discovery).expect("discovery");
    changed.push(b' ');
    fs::write(discovery, changed).expect("changed discovery");

    // Act
    let output = fixture.command(&[
        "inventory",
        "generate",
        "--attestation-commit",
        &attestation,
    ]);

    // Assert
    assert_failure_contains(&output, "inventory/source-input");
    assert_eq!(before, fs::read(report).expect("unchanged report"));
}

#[test]
fn explicit_docs_rejects_hand_edited_generated_counts() {
    // Arrange
    let fixture = Checkout::new("inventory-manual-edit");
    fixture.materialize();
    let attestation = fixture.attest();
    fixture.project(&attestation);
    let report = fixture.root.join("COMPATIBILITY.md");
    let changed = fs::read_to_string(&report)
        .expect("generated report")
        .replace("| Unexplained rows | 0 |", "| Unexplained rows | 1 |");
    fs::write(report, changed).expect("hand edited report");

    // Act
    let output = fixture.command(&["docs", "check", "--attestation-commit", &attestation]);

    // Assert
    assert_failure_contains(&output, "inventory/stale");
}

#[test]
fn missing_retained_payload_cannot_overwrite_report() {
    // Arrange
    let fixture = Checkout::new("inventory-missing-payload");
    fixture.materialize();
    let attestation = fixture.attest();
    let report = fixture.root.join("COMPATIBILITY.md");
    let before = fs::read(&report).expect("report");
    fs::remove_file(fixture.root.join("target/retained/artifact-0.json"))
        .expect("remove fixture payload");

    // Act
    let output = fixture.command(&[
        "inventory",
        "generate",
        "--attestation-commit",
        &attestation,
    ]);

    // Assert
    assert_failure_contains(&output, "inventory/attestation");
    assert_eq!(before, fs::read(report).expect("unchanged report"));
}
