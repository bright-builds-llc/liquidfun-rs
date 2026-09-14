use super::*;
use complete_fixture::{Checkout, assert_pass};

#[test]
fn fresh_checkout_restores_payloads_and_validates_explicit_c_a_from_later_docs_d() {
    // Arrange
    let fixture = Checkout::new("fresh");
    fixture.materialize();
    assert_pass(&fixture.validate(None));
    let attestation = fixture.attest();
    assert_eq!(
        git_output(&fixture.root, &["rev-parse", "HEAD^"]),
        fixture.candidate
    );
    let fresh_root = fixture.root.with_extension("fresh");
    git_output(
        &fixture.root,
        &[
            "clone",
            "--quiet",
            "--no-local",
            ".",
            fresh_root.to_str().expect("fixture path"),
        ],
    );
    let fresh = Checkout {
        root: fresh_root,
        candidate: fixture.candidate.clone(),
    };
    assert_failure_contains(&fresh.validate(Some(&attestation)), "release/artifact-path");
    fs::create_dir_all(fresh.root.join("target/retained")).expect("restored target directory");
    for entry in fs::read_dir(fixture.root.join("target/retained")).expect("retained entries") {
        let entry = entry.expect("retained entry");
        fs::copy(
            entry.path(),
            fresh.root.join("target/retained").join(entry.file_name()),
        )
        .expect("restore exact payload bytes");
    }
    git_output(&fresh.root, &["config", "user.name", "Attestation Test"]);
    git_output(
        &fresh.root,
        &["config", "user.email", "attestation@example.invalid"],
    );
    fresh.project(&attestation);
    let docs_commit = fresh.commit(
        &["README.md", "COMPATIBILITY.md", "RELEASE.md"],
        "projection D",
    );

    // Act
    let explicit = fresh.validate(Some(&attestation));
    let substituted = fresh.validate(Some(&docs_commit));
    let projected = fresh.command(&["docs", "check", "--attestation-commit", &attestation]);
    let implicit = fresh.command(&["docs", "check"]);

    // Assert
    assert_pass(&explicit);
    assert!(String::from_utf8_lossy(&explicit.stdout).contains(&fixture.candidate));
    assert_failure_contains(&substituted, "release/attestation-diff");
    assert_pass(&projected);
    assert_failure_contains(&implicit, "docs/phase12-public-contract");
}

#[test]
fn intervening_non_attestation_changes_are_rejected_even_when_reverted() {
    // Arrange, Act, Assert: the entire C..A history is confined, not just its net diff.
    for path in [".planning/STATE.md", "scripts/producer.sh", "README.md"] {
        let fixture = Checkout::new("intervening");
        let destination = fixture.root.join(path);
        let maybe_original = if destination.is_file() {
            Some(fs::read(&destination).expect("original bytes"))
        } else {
            None
        };
        fs::create_dir_all(destination.parent().expect("parent")).expect("parent directory");
        fs::write(&destination, "intervening edit\n").expect("intervening file");
        fixture.commit(&[path], "intervening change");
        if let Some(original) = maybe_original {
            fs::write(&destination, original).expect("restore file");
        } else {
            fs::remove_file(&destination).expect("remove fixture file");
        }
        fixture.commit(&[path], "revert intervening change");
        fixture.materialize();
        let attestation = fixture.attest();
        let output = fixture.validate(Some(&attestation));
        assert_failure_contains(&output, "release/attestation-diff");
        assert!(!String::from_utf8_lossy(&output.stdout).contains("VALID"));
    }
}

#[test]
fn committed_records_cannot_be_replaced_by_different_worktree_bytes() {
    // Arrange, Act, Assert: independently hashed records must also be the bytes committed at A.
    for relative in [
        "source-candidate.json",
        "candidate-manifest.json",
        "audit-report.json",
    ] {
        let fixture = Checkout::new("tampered-record");
        fixture.materialize();
        let attestation = fixture.attest();
        let path = fixture.root.join("reference/release").join(relative);
        let mut bytes = fs::read(&path).expect("record");
        bytes.push(b' ');
        fs::write(&path, bytes).expect("tampered record");
        let output = fixture.validate(Some(&attestation));
        assert!(!output.status.success());
        assert!(!String::from_utf8_lossy(&output.stdout).contains("VALID"));
    }
}

#[test]
fn ready_projection_rejects_tampered_retained_artifacts() {
    // Arrange
    let fixture = Checkout::new("tampered-payload");
    fixture.materialize();
    let attestation = fixture.attest();
    fixture.project(&attestation);
    fs::write(fixture.root.join("target/retained/artifact-0.json"), b"{}")
        .expect("tampered artifact");

    // Act
    let output = fixture.command(&["docs", "check", "--attestation-commit", &attestation]);

    // Assert
    assert_failure_contains(&output, "docs/readiness: release/artifact-hash");
}

#[test]
fn ready_projection_rejects_stale_or_substituted_attestation_identity() {
    // Arrange
    let fixture = Checkout::new("stale");
    fixture.materialize();
    let attestation = fixture.attest();
    fixture.project(&attestation);

    // Act
    let output = fixture.command(&["docs", "check", "--attestation-commit", &fixture.candidate]);

    // Assert
    assert_failure_contains(&output, "docs/readiness: release/attestation-diff");
}

#[test]
fn ready_projection_requires_the_accepted_source_identity_in_every_document() {
    // Arrange
    let fixture = Checkout::new("wrong-projection");
    fixture.materialize();
    let attestation = fixture.attest();
    fixture.project(&attestation);
    let path = fixture.root.join("README.md");
    let text = fs::read_to_string(&path)
        .expect("README")
        .replace(&fixture.candidate, &attestation);
    fs::write(path, text).expect("substituted source identity");

    // Act
    let output = fixture.command(&["docs", "check", "--attestation-commit", &attestation]);

    // Assert
    assert_failure_contains(&output, "docs/phase12-public-contract");
}
