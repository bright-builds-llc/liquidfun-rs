use super::*;

const PHASE11_PROMOTION_IDS: [&str; 4] = [
    "subsystem.headless-catalog-execution",
    "subsystem.headless-public-observation-and-debug-draw",
    "subsystem.headless-reviewed-upstream-equivalence",
    "subsystem.headless-semantic-checkpoints-and-comparison",
];

const PHASE11_AUTHORITY_INPUTS: [&str; 12] = [
    "reference/artifacts/phase11/exact-ref.json",
    "reference/scenario-catalog.json",
    "reference/artifacts/phase11/scenario-mappings.json",
    "crates/liquidfun-differential/tests/fixtures/catalog/phase11-v1.json",
    "crates/liquidfun-differential/tests/fixtures/catalog/cases/particle-groups.jsonl",
    "crates/liquidfun-differential/tests/fixtures/catalog/cases/queries-callbacks-mutations.jsonl",
    "crates/liquidfun-differential/tests/fixtures/catalog/cases/rigid-joint-rope.jsonl",
    "protocol/tolerances/phase6-v1.toml",
    "protocol/tolerances/phase7-v1.toml",
    "protocol/tolerances/phase8-v1.toml",
    "crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/phase9-v1.json",
    "crates/liquidfun-differential/tests/fixtures/rigid_world/phase10/phase10-v1.json",
];

#[test]
fn check_accepts_exact_phase11_supported_outcomes() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    install_phase11_authority(&fixture)?;
    fixture.write_compatibility(&promoted_phase11_entries()?)?;

    // Act
    let output = fixture.generate()?;

    // Assert
    assert_success(&output);
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_incomplete_phase11_rows_and_dimensions() -> TestResult {
    for dimension in [
        "implemented",
        "unit_tested",
        "differentially_validated",
        "platform_validated",
    ] {
        // Arrange
        let fixture = InventoryFixture::new()?;
        assert_success(&fixture.discover()?);
        install_phase11_authority(&fixture)?;
        let mut entries = promoted_phase11_entries()?;
        phase11_entry_mut(&mut entries)["evidence"][dimension] = not_evidenced();
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
    install_phase11_authority(&fixture)?;
    let mut entries = promoted_phase11_entries()?;
    entries.retain(|entry| entry["id"] != PHASE11_PROMOTION_IDS[3]);
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
fn check_rejects_every_phase11_authority_identity_mismatch() -> TestResult {
    for (pointer, replacement) in [
        ("/approved_sha", json!("mixed-sha")),
        ("/run/id", json!(29_899_265_024_u64)),
        ("/run/jobs/canonical/id", json!(0)),
        ("/run/jobs/sanitizer/id", json!(0)),
        ("/toolchains/platform", json!("broad-platform")),
        ("/toolchains/clang", json!("unlocked")),
        ("/artifacts/canonical/id", json!(8_521_315_244_u64)),
        ("/artifacts/sanitizer/id", json!(8_521_345_417_u64)),
        ("/artifacts/canonical/live_sha256", json!("mixed-live")),
        (
            "/artifacts/sanitizer/archive_sha256",
            json!("mixed-archive"),
        ),
        ("/semantic/catalog/sha256", json!("stale-catalog")),
        ("/semantic/mapping/sha256", json!("stale-mapping")),
    ] {
        // Arrange
        let fixture = promoted_fixture()?;
        mutate_authority(&fixture, pointer, replacement)?;

        // Act
        let output = fixture.generate()?;

        // Assert
        assert_failure_category(&output, "inventory/evidence");
        fixture.cleanup()?;
    }
    Ok(())
}

#[test]
fn check_rejects_omitted_proofs_unsupported_leaves_and_diagnostic_claims() -> TestResult {
    for (pointer, replacement) in [
        ("/semantic/inherited_proofs", json!([])),
        ("/semantic/cases/0/observation_leaves", json!([])),
        (
            "/semantic/cases/0/primitive_leaves/0",
            json!("pixel.frame-rate"),
        ),
        (
            "/semantic/cases/0/numeric_policies/0",
            json!("unknown.policy"),
        ),
        ("/semantic/cases/0/outcomes/sanitizer", json!("unsupported")),
        ("/promotion/excluded_claims", json!(["performance"])),
        ("/review/review_id", json!("unreviewed")),
    ] {
        // Arrange
        let fixture = promoted_fixture()?;
        mutate_authority(&fixture, pointer, replacement)?;

        // Act
        let output = fixture.generate()?;

        // Assert
        assert_failure_category(&output, "inventory/evidence");
        fixture.cleanup()?;
    }
    Ok(())
}

#[test]
fn check_rejects_stale_phase11_mapping_and_inherited_proof_files() -> TestResult {
    for relative in [
        "reference/artifacts/phase11/scenario-mappings.json",
        "protocol/tolerances/phase8-v1.toml",
        "crates/liquidfun-differential/tests/fixtures/rigid_world/phase10/phase10-v1.json",
    ] {
        // Arrange
        let fixture = promoted_fixture()?;
        let mut bytes = fs::read(fixture.root.join(relative))?;
        bytes.push(b'\n');
        fs::write(fixture.root.join(relative), bytes)?;

        // Act
        let output = fixture.generate()?;

        // Assert
        assert_failure_category(&output, "inventory/evidence");
        assert!(stderr(&output).contains("stale or unreviewed digest"));
        fixture.cleanup()?;
    }
    Ok(())
}

#[test]
fn check_rejects_failed_phase11_candidates_in_every_scoped_row() -> TestResult {
    for rejected in ["29899265024", "8521315244", "8521345417"] {
        // Arrange
        let fixture = InventoryFixture::new()?;
        assert_success(&fixture.discover()?);
        install_phase11_authority(&fixture)?;
        let mut entries = promoted_phase11_entries()?;
        phase11_entry_mut(&mut entries)["evidence"]["platform_validated"]["references"] =
            json!([rejected]);
        fixture.write_compatibility(&entries)?;

        // Act
        let output = fixture.generate()?;

        // Assert
        assert_failure_category(&output, "inventory/evidence");
        assert!(stderr(&output).contains("rejected Phase 11 authority"));
        fixture.cleanup()?;
    }
    Ok(())
}

#[test]
fn check_rejects_non_supported_or_diagnostic_phase11_outcomes() -> TestResult {
    for dimension in ["documented_difference", "intentionally_unsupported"] {
        // Arrange
        let fixture = InventoryFixture::new()?;
        assert_success(&fixture.discover()?);
        install_phase11_authority(&fixture)?;
        let mut entries = promoted_phase11_entries()?;
        let entry = phase11_entry_mut(&mut entries);
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
fn check_rejects_phase11_authority_for_out_of_scope_row() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let mut entries = InventoryFixture::valid_entries();
    entries[0]["evidence"]["implemented"] = evidenced();
    entries[0]["evidence"]["platform_validated"] = json!({
        "status": "evidenced",
        "references": [
            "https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29927362730"
        ]
    });
    fixture.write_compatibility(&entries)?;

    // Act
    let output = fixture.generate()?;

    // Assert
    assert_failure_category(&output, "inventory/evidence");
    assert!(stderr(&output).contains("out-of-scope row"));
    fixture.cleanup()?;
    Ok(())
}

fn promoted_fixture() -> Result<InventoryFixture, Box<dyn Error>> {
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    install_phase11_authority(&fixture)?;
    fixture.write_compatibility(&promoted_phase11_entries()?)?;
    Ok(fixture)
}

fn install_phase11_authority(fixture: &InventoryFixture) -> io::Result<()> {
    for relative in PHASE11_AUTHORITY_INPUTS {
        let destination = fixture.root.join(relative);
        let Some(parent) = destination.parent() else {
            return Err(io::Error::other("Phase 11 fixture path has no parent"));
        };
        fs::create_dir_all(parent)?;
        fs::copy(workspace_root().join(relative), destination)?;
    }
    Ok(())
}

fn mutate_authority(
    fixture: &InventoryFixture,
    pointer: &str,
    replacement: Value,
) -> Result<(), Box<dyn Error>> {
    let path = fixture
        .root
        .join("reference/artifacts/phase11/exact-ref.json");
    let mut authority: Value = serde_json::from_slice(&fs::read(&path)?)?;
    let Some(value) = authority.pointer_mut(pointer) else {
        return Err(format!("authority fixture is missing JSON pointer `{pointer}`").into());
    };
    *value = replacement;
    fs::write(path, serde_json::to_vec_pretty(&authority)?)?;
    Ok(())
}

fn promoted_phase11_entries() -> Result<Vec<Value>, Box<dyn Error>> {
    let ledger: Value = serde_json::from_slice(&fs::read(
        workspace_root().join("reference/compatibility.json"),
    )?)?;
    let maybe_evidence = ledger["entries"].as_array().and_then(|entries| {
        entries
            .iter()
            .find(|entry| entry["id"] == PHASE11_PROMOTION_IDS[0])
            .map(|entry| entry["evidence"].clone())
    });
    let Some(evidence) = maybe_evidence else {
        return Err("repository ledger is missing the Phase 11 promotion row".into());
    };
    let mut entries = promoted_phase9_entries();
    entries.extend(PHASE11_PROMOTION_IDS.into_iter().map(|id| {
        compatibility_entry(
            id,
            "subsystem",
            "phase11-fixture",
            "liquidfun::headless",
            &evidence,
        )
    }));
    entries.sort_by(|left, right| left["id"].as_str().cmp(&right["id"].as_str()));
    Ok(entries)
}

fn phase11_entry_mut(entries: &mut [Value]) -> &mut Value {
    let maybe_entry = entries
        .iter_mut()
        .find(|entry| entry["id"] == PHASE11_PROMOTION_IDS[0]);
    let Some(entry) = maybe_entry else {
        panic!("promoted Phase 11 fixture must include the catalog row");
    };
    entry
}
