use super::*;

#[test]
fn check_accepts_fresh_phase9_authority() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    fixture.write_compatibility(&promoted_phase9_entries())?;

    // Act
    let output = fixture.generate()?;

    // Assert
    assert_success(&output);
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_incomplete_phase9_promotion() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let mut entries = promoted_phase9_entries();
    phase9_entry_mut(&mut entries)["evidence"]["platform_validated"] = not_evidenced();
    fixture.write_compatibility(&entries)?;

    // Act
    let output = fixture.generate()?;

    // Assert
    assert_failure_category(&output, "inventory/evidence");
    assert!(stderr(&output).contains("incomplete Phase 9 promotion"));
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_noncanonical_phase9_promotion() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let mut entries = promoted_phase9_entries();
    phase9_entry_mut(&mut entries)["evidence"]["platform_validated"]["references"][1] =
        json!("sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
    fixture.write_compatibility(&entries)?;

    // Act
    let output = fixture.generate()?;

    // Assert
    assert_failure_category(&output, "inventory/evidence");
    assert!(stderr(&output).contains("noncanonical Phase 9 authority"));
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_superseded_phase9_authority() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let mut entries = promoted_phase9_entries();
    phase9_entry_mut(&mut entries)["evidence"]["platform_validated"]["references"][0] =
        json!("https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29439515367");
    fixture.write_compatibility(&entries)?;

    // Act
    let output = fixture.generate()?;

    // Assert
    assert_failure_category(&output, "inventory/evidence");
    assert!(stderr(&output).contains("noncanonical Phase 9 authority"));
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_superseded_phase9_differential_reference() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let mut entries = promoted_phase9_entries();
    phase9_entry_mut(&mut entries)["evidence"]["differentially_validated"]["references"] =
        json!([".planning/phases/09-particle-storage-lifecycle-and-coupling/09-16-SUMMARY.md"]);
    fixture.write_compatibility(&entries)?;

    // Act
    let output = fixture.generate()?;

    // Assert
    assert_failure_category(&output, "inventory/evidence");
    assert!(stderr(&output).contains("superseded Phase 9 authority"));
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_failed_phase9_trace() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let mut entries = promoted_phase9_entries();
    phase9_entry_mut(&mut entries)["evidence"]["platform_validated"]["references"][2] = json!(
        "phase9-canonical-29439515367-a87f84bbdbfe55fb732d74c481c4a4bda9eec958/identity.json#trace-sha256=3a339387b4c4acccc15b5fc4944d6bec9c7e1d315f4753034ae52a5ff97f2e64"
    );
    fixture.write_compatibility(&entries)?;

    // Act
    let output = fixture.generate()?;

    // Assert
    assert_failure_category(&output, "inventory/evidence");
    assert!(stderr(&output).contains("noncanonical Phase 9 authority"));
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_incomplete_phase9_manifest() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let mut entries = promoted_phase9_entries();
    phase9_entry_mut(&mut entries)["evidence"]["platform_validated"]["references"][3] = json!(
        "phase9-canonical-29439515367-a87f84bbdbfe55fb732d74c481c4a4bda9eec958/identity.json#manifest-sha256=36cfaad1f56505f8427408733e2231ad613984a4cb3eb3b8d757e7a14b2c38e0"
    );
    fixture.write_compatibility(&entries)?;

    // Act
    let output = fixture.generate()?;

    // Assert
    assert_failure_category(&output, "inventory/evidence");
    assert!(stderr(&output).contains("noncanonical Phase 9 authority"));
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_partial_phase9_authority() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let mut entries = promoted_phase9_entries();
    let maybe_references =
        phase9_entry_mut(&mut entries)["evidence"]["platform_validated"]["references"]
            .as_array_mut();
    let Some(references) = maybe_references else {
        panic!("promoted Phase 9 fixture must contain authority references");
    };
    let maybe_removed = references.pop();
    assert!(maybe_removed.is_some());
    fixture.write_compatibility(&entries)?;

    // Act
    let output = fixture.generate()?;

    // Assert
    assert_failure_category(&output, "inventory/evidence");
    assert!(stderr(&output).contains("noncanonical Phase 9 authority"));
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_substituted_phase9_artifact() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let mut entries = promoted_phase9_entries();
    phase9_entry_mut(&mut entries)["evidence"]["platform_validated"]["references"][4] = json!(
        "https://api.github.com/repos/bright-builds-llc/liquidfun-rs/actions/artifacts/8408156562/zip#sha256=faaf24c870826251f0dd1d507ba9c335269b78433ba1ce2ee0e1995336f0139a"
    );
    fixture.write_compatibility(&entries)?;

    // Act
    let output = fixture.generate()?;

    // Assert
    assert_failure_category(&output, "inventory/evidence");
    assert!(stderr(&output).contains("noncanonical Phase 9 authority"));
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_every_prior_or_failed_phase9_authority_marker() -> TestResult {
    for marker in [
        "29439515367",
        "29583793056",
        "29625083184",
        "29652578231",
        "8352859391",
        "8352881868",
        "a87f84bbdbfe55fb732d74c481c4a4bda9eec958",
        "f237d6f1ebe0e59f65a5ae0609140eecdd8b32247e9d2064c83748be1ab9f5ea",
        "95ad57e5d5711ae6aa93847ad1efd4a04025bd2956b4996535fa0e5f45a5893f",
        "8408156562",
        "8408174081",
        "b27fc14f6b29fb82ca815fa1effba71bae09d424",
        "faaf24c870826251f0dd1d507ba9c335269b78433ba1ce2ee0e1995336f0139a",
        "f4b30cebed7b81a41282a33d45b81231485a2fa0c3a958c7b68a3ecbad086e7c",
        "8423580554",
        "8431920189",
        "8431922578",
        "7ed430c497efbaa8585ee9ef3862be1abda29ef5",
        "f7478565688e7250257bc8c1d066456853604394c61e7cbe38ffcc11e73c5c5b",
    ] {
        // Arrange
        let fixture = InventoryFixture::new()?;
        assert_success(&fixture.discover()?);
        let mut entries = promoted_phase9_entries();
        phase9_entry_mut(&mut entries)["evidence"]["differentially_validated"]["references"] =
            json!([format!("rejected-authority/{marker}")]);
        fixture.write_compatibility(&entries)?;

        // Act
        let output = fixture.generate()?;

        // Assert
        assert_failure_category(&output, "inventory/evidence");
        fixture.cleanup()?;
    }
    Ok(())
}

#[test]
fn check_rejects_semantically_incomplete_phase9_differential_claims() -> TestResult {
    for unsupported_claim in [
        "missing-schema-v4-proof-topology",
        "baseline-substituted-for-required-independent-role",
        "forbidden-cross-run-role-alias",
        "missing-retained-rigid-comparator-record",
        "wrong-retained-rigid-policy-digest",
        "non-match-retained-rigid-outcome",
        "particle-only-comparator",
        "wrong-seven-case-cardinality",
        "wrong-58-binding-cardinality",
        "58-label-manifest-without-semantic-binding-digest",
        "wrong-action-binding",
        "wrong-checkpoint-binding",
        "wrong-observation-binding",
        "zero-positive-energy-witness",
        "empty-stuck-candidate-witness",
        "failed-phase9-log",
        "incomplete-phase9-policy-array",
        "wrong-22-policy-array",
        "payload-digest-mismatch",
        "trace-digest-mismatch",
        "binding-digest-mismatch",
        "manifest-digest-mismatch",
    ] {
        // Arrange
        let fixture = InventoryFixture::new()?;
        assert_success(&fixture.discover()?);
        let mut entries = promoted_phase9_entries();
        phase9_entry_mut(&mut entries)["evidence"]["differentially_validated"]["references"] =
            json!([format!("unsupported-phase9-evidence/{unsupported_claim}")]);
        fixture.write_compatibility(&entries)?;

        // Act
        let output = fixture.generate()?;

        // Assert
        assert_failure_category(&output, "inventory/evidence");
        fixture.cleanup()?;
    }
    Ok(())
}

#[test]
fn check_rejects_mixed_run_and_canonical_as_sanitizer_artifacts() -> TestResult {
    for substituted_reference in [
        "https://api.github.com/repos/bright-builds-llc/liquidfun-rs/actions/artifacts/8423580554/zip#sha256=failed-run-canonical",
        "phase9-sanitizer-29652578231-22b31c0e1be8896df622b1decd58ba2853a60b04/identity.json#trace-sha256=mixed-run",
        "phase9-sanitizer-29661682074-22b31c0e1be8896df622b1decd58ba2853a60b04/identity.json#trace-sha256=mixed-sha",
        "phase9-canonical-29661682074-9f2169ad1ad3c72adeae5e4fb1ea188b20ba84ce/identity.json#trace-sha256=mixed-job",
        "https://api.github.com/repos/bright-builds-llc/liquidfun-rs/actions/artifacts/8434547024/zip#sha256=canonical-used-as-sanitizer",
    ] {
        // Arrange
        let fixture = InventoryFixture::new()?;
        assert_success(&fixture.discover()?);
        let mut entries = promoted_phase9_entries();
        phase9_entry_mut(&mut entries)["evidence"]["platform_validated"]["references"][4] =
            json!(substituted_reference);
        fixture.write_compatibility(&entries)?;

        // Act
        let output = fixture.generate()?;

        // Assert
        assert_failure_category(&output, "inventory/evidence");
        assert!(stderr(&output).contains("noncanonical Phase 9 authority"));
        fixture.cleanup()?;
    }
    Ok(())
}

#[test]
fn check_rejects_each_phase10_behavior_claim_during_phase9_promotion() -> TestResult {
    for deferred_claim in [
        "particle-group-behavior",
        "particle-group-topology",
        "particle-pair-behavior",
        "particle-triad-behavior",
        "complete-particle-source-area",
        "particle-solver-behavior",
        "cross-engine-stable-id-rotation",
    ] {
        // Arrange
        let fixture = InventoryFixture::new()?;
        assert_success(&fixture.discover()?);
        let mut entries = promoted_phase9_entries();
        phase9_entry_mut(&mut entries)["evidence"]["differentially_validated"]["references"] =
            json!([format!("deferred-phase10/{deferred_claim}")]);
        fixture.write_compatibility(&entries)?;

        // Act
        let output = fixture.generate()?;

        // Assert
        assert_failure_category(&output, "inventory/evidence");
        fixture.cleanup()?;
    }
    Ok(())
}

#[test]
fn check_rejects_each_phase10_claim_during_phase9_promotion() -> TestResult {
    // Arrange
    for id in PHASE9_DEFERRED_IDS {
        let fixture = InventoryFixture::new()?;
        assert_success(&fixture.discover()?);
        let mut entries = promoted_phase9_entries();
        let mut deferred_evidence = valid_evidence();
        deferred_evidence["implemented"] = evidenced();
        entries.push(compatibility_entry(
            id,
            if id.starts_with("subsystem.") {
                "subsystem"
            } else if id.starts_with("source-area.") {
                "source_area"
            } else {
                "public_api"
            },
            &format!("phase10-fixture/{id}"),
            "liquidfun::particle",
            &deferred_evidence,
        ));
        entries.sort_by(|left, right| left["id"].as_str().cmp(&right["id"].as_str()));
        fixture.write_compatibility(&entries)?;

        // Act
        let output = fixture.generate()?;

        // Assert
        assert_failure_category(&output, "inventory/evidence");
        assert!(stderr(&output).contains("deferred Phase 10 row"));
        fixture.cleanup()?;
    }
    Ok(())
}
