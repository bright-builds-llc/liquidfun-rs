use super::*;

const PHASE10_PROMOTION_IDS: [&str; 5] = [
    "public-api.liquidfun-box2d-box2d-particle-b2particleassembly-h",
    "public-api.liquidfun-box2d-box2d-particle-b2particlegroup-h",
    "source-area.liquidfun-box2d-box2d-particle",
    "subsystem.particle-groups-pairs-and-triads",
    "subsystem.particle-solver-behaviors",
];

#[test]
fn check_accepts_exact_phase10_supported_outcomes() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    fixture.write_compatibility(&promoted_phase10_entries()?)?;

    // Act
    let output = fixture.generate()?;

    // Assert
    assert_success(&output);
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_incomplete_phase10_rows_and_dimensions() -> TestResult {
    for dimension in [
        "implemented",
        "unit_tested",
        "differentially_validated",
        "platform_validated",
    ] {
        // Arrange
        let fixture = InventoryFixture::new()?;
        assert_success(&fixture.discover()?);
        let mut entries = promoted_phase10_entries()?;
        phase10_entry_mut(&mut entries)["evidence"][dimension] = not_evidenced();
        fixture.write_compatibility(&entries)?;

        // Act
        let output = fixture.generate()?;

        // Assert
        assert_failure_category(&output, "inventory/evidence");
        fixture.cleanup()?;
    }

    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let mut entries = promoted_phase10_entries()?;
    entries.retain(|entry| entry["id"] != PHASE10_PROMOTION_IDS[4]);
    fixture.write_compatibility(&entries)?;

    // Act
    let output = fixture.generate()?;

    // Assert
    assert_failure_category(&output, "inventory/evidence");
    assert!(stderr(&output).contains("missing scoped row"));
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_every_failed_stale_or_mixed_phase10_authority() -> TestResult {
    for corruption in [
        "failed-run/29831597090",
        "failed-canonical/8495653581",
        "failed-sanitizer/8495705068",
        "historical-run/29439515367",
        "historical-run/29583793056",
        "historical-run/29625083184",
        "historical-run/29652578231",
        "historical-artifact/8352859391",
        "historical-artifact/8352881868",
        "historical-artifact/8408156562",
        "historical-artifact/8408174081",
        "historical-artifact/8423580554",
        "historical-artifact/8431920189",
        "historical-artifact/8431922578",
        "historical-sha/a87f84bbdbfe55fb732d74c481c4a4bda9eec958",
        "historical-digest/f237d6f1ebe0e59f65a5ae0609140eecdd8b32247e9d2064c83748be1ab9f5ea",
        "historical-digest/95ad57e5d5711ae6aa93847ad1efd4a04025bd2956b4996535fa0e5f45a5893f",
        "historical-digest/3a339387b4c4acccc15b5fc4944d6bec9c7e1d315f4753034ae52a5ff97f2e64",
        "historical-digest/ee75462d49275c5b7d02b8677eb6f9bf82c241c6b993c16d6df08a2ae231a070",
        "historical-summary/09-16-SUMMARY.md",
        "historical-sha/b27fc14f6b29fb82ca815fa1effba71bae09d424",
        "historical-digest/faaf24c870826251f0dd1d507ba9c335269b78433ba1ce2ee0e1995336f0139a",
        "historical-digest/f4b30cebed7b81a41282a33d45b81231485a2fa0c3a958c7b68a3ecbad086e7c",
        "historical-summary/09-23-SUMMARY.md",
        "historical-sha/7ed430c497efbaa8585ee9ef3862be1abda29ef5",
        "historical-digest/f7478565688e7250257bc8c1d066456853604394c61e7cbe38ffcc11e73c5c5b",
        "historical-sha/22b31c0e1be8896df622b1decd58ba2853a60b04",
        "historical-digest/ea333de6ac32d64c1c5b4e80738275451f0e51994b7f78e70961597d48e77500",
        "historical-digest/99fa817d3b891a8942709e4b4af2bd4fa0aedbde0fc4c19b398829f02128a6c6",
        "historical-digest/662b9514472c1d6d8186115577f43c5987870a2a24592156b46631f1c28b4a3e",
        "historical-digest/671d16f1c7af0f948760b9cdc62b3ed1fefb7307889a46334230605365aefe80",
        "mixed-run/29832646127/29831597090",
        "mixed-sha/b20328aec9697353e322e022cd289e65d5a31340/341fa70b50898d5bdf3f427240794f19210b881b",
        "mixed-job/88641473476/88641473497",
        "mixed-artifact/8496062831/8496084932",
        "wrong-api-digest",
        "wrong-archive-digest",
        "wrong-platform-or-toolchain",
    ] {
        // Arrange
        let fixture = InventoryFixture::new()?;
        assert_success(&fixture.discover()?);
        let mut entries = promoted_phase10_entries()?;
        phase10_entry_mut(&mut entries)["evidence"]["platform_validated"]["references"] =
            json!([corruption]);
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
fn check_rejects_every_semantic_authority_corruption() -> TestResult {
    for corruption in [
        "missing-leaf",
        "duplicate-leaf",
        "unknown-leaf",
        "missing-policy",
        "duplicate-policy",
        "unknown-policy",
        "non-match-supported-leaf",
        "failed-canonical-log",
        "failed-sanitizer-log",
        "failed-d0",
        "failed-replay",
        "failed-debug-release",
        "failed-provenance",
        "incomplete-inherited-proof",
        "missing-control-witness",
        "missing-activation-witness",
        "aliased-proof-role",
        "wrong-payload-digest",
        "wrong-trace-digest",
        "wrong-leaf-set-digest",
        "wrong-policy-set-digest",
        "wrong-manifest-digest",
    ] {
        // Arrange
        let fixture = InventoryFixture::new()?;
        assert_success(&fixture.discover()?);
        let mut entries = promoted_phase10_entries()?;
        phase10_entry_mut(&mut entries)["evidence"]["differentially_validated"]["references"] =
            json!([format!("corrupt-phase10/{corruption}")]);
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
fn check_rejects_non_supported_outcome_for_promoted_phase10_row() -> TestResult {
    for dimension in ["documented_difference", "intentionally_unsupported"] {
        // Arrange
        let fixture = InventoryFixture::new()?;
        assert_success(&fixture.discover()?);
        let mut entries = promoted_phase10_entries()?;
        let entry = phase10_entry_mut(&mut entries);
        entry["evidence"][dimension] = evidenced();
        if dimension == "intentionally_unsupported" {
            entry["applicability"]["status"] = json!("reviewed_exclusion");
        }
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
fn check_rejects_phase10_authority_for_deferred_scope() -> TestResult {
    for claim in [
        "example",
        "testbed",
        "performance",
        "broad-platform",
        "release-readiness",
    ] {
        // Arrange
        let fixture = InventoryFixture::new()?;
        assert_success(&fixture.discover()?);
        let mut entries = promoted_phase10_entries()?;
        entries[0]["evidence"]["platform_validated"] = json!({
            "status": "evidenced",
            "references": [format!(
                "https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29832646127#{claim}"
            )]
        });
        entries[0]["evidence"]["implemented"] = evidenced();
        fixture.write_compatibility(&entries)?;

        // Act
        let output = fixture.generate()?;

        // Assert
        assert_failure_category(&output, "inventory/evidence");
        assert!(stderr(&output).contains("out-of-scope row"));
        fixture.cleanup()?;
    }
    Ok(())
}

fn promoted_phase10_entries() -> Result<Vec<Value>, Box<dyn Error>> {
    let ledger: Value = serde_json::from_slice(&fs::read(
        workspace_root().join("reference/compatibility.json"),
    )?)?;
    let maybe_evidence = ledger["entries"].as_array().and_then(|entries| {
        entries
            .iter()
            .find(|entry| entry["id"] == PHASE10_PROMOTION_IDS[0])
            .map(|entry| entry["evidence"].clone())
    });
    let Some(evidence) = maybe_evidence else {
        return Err("repository ledger is missing the Phase 10 promotion row".into());
    };
    let mut entries = promoted_phase9_entries();
    entries.extend(PHASE10_PROMOTION_IDS.into_iter().map(|id| {
        compatibility_entry(
            id,
            "subsystem",
            "phase10-fixture",
            "liquidfun::particle",
            &evidence,
        )
    }));
    entries.sort_by(|left, right| left["id"].as_str().cmp(&right["id"].as_str()));
    Ok(entries)
}

fn phase10_entry_mut(entries: &mut [Value]) -> &mut Value {
    let maybe_entry = entries
        .iter_mut()
        .find(|entry| entry["id"] == PHASE10_PROMOTION_IDS[0]);
    let Some(entry) = maybe_entry else {
        panic!("promoted Phase 10 fixture must include the particle assembly row");
    };
    entry
}
