//! Command-level coverage for deterministic compatibility inventory tooling.

use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};

const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const PHASE9_AUTHORITY_REFERENCES: [&str; 15] = [
    "https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29661682074",
    "https://api.github.com/repos/bright-builds-llc/liquidfun-rs/actions/artifacts/8434547024/zip#sha256=22a37f91965eaf494b3e1fea041e1c54da9be03c06da5e276a641ee6cf536084",
    "phase9-canonical-29661682074-9f2169ad1ad3c72adeae5e4fb1ea188b20ba84ce/identity.json#trace-sha256=eefec714082fc701fb6ec2cebd15ed9353114a8cc17f975b71c666b33fd3ccf7",
    "phase9-canonical-29661682074-9f2169ad1ad3c72adeae5e4fb1ea188b20ba84ce/identity.json#manifest-sha256=74998e953e79f5ed04a58097d43abbca3cc814bee4fc86d0fd552d2951b1ae7c",
    "https://api.github.com/repos/bright-builds-llc/liquidfun-rs/actions/artifacts/8434557009/zip#sha256=849b8dba5b4c5a0f5e6ea4cddf10bf8243a71bdeec3b75676677358aa34d4316",
    "phase9-sanitizer-29661682074-9f2169ad1ad3c72adeae5e4fb1ea188b20ba84ce/identity.json#trace-sha256=3c697421472ee087d265cb9a6268ab04ef76dce37c39ed6b4202fa1a36c7dbdd",
    "phase9-sanitizer-29661682074-9f2169ad1ad3c72adeae5e4fb1ea188b20ba84ce/identity.json#manifest-sha256=74998e953e79f5ed04a58097d43abbca3cc814bee4fc86d0fd552d2951b1ae7c",
    "phase9-manifest.json#semantic-manifest-sha256=a319f771c5d9e952b9389160bb3ad19ce487da43271e62568828ce2ae22a33aa",
    "https://github.com/bright-builds-llc/liquidfun-rs/commit/9f2169ad1ad3c72adeae5e4fb1ea188b20ba84ce",
    ".planning/phases/09-particle-storage-lifecycle-and-coupling/09-31-SUMMARY.md",
    "TESTING.md#approved-phase-9-evidence-run-2026-07-18-wr-02",
    "https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29661682074/job/88125511292",
    "https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29661682074/job/88125511305",
    "phase9-manifest.json#payload-digest-set-sha256=72797909ebb807c4c7dc591b4fa8987b26f3f26e43b967e080db4363f26b509d",
    "phase9-manifest.json#binding-digest-set-sha256=2e0e4212a62aec27b371bcd8dc9301966e0f712b0d28736e39f3993cc3ab3134",
];
const PHASE9_DIFFERENTIAL_REFERENCES: [&str; 7] = [
    "crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/phase9-v1.json",
    "crates/liquidfun-differential/tests/phase9_corpus.rs",
    "crates/liquidfun-differential/src/rigid_world/phase9.rs",
    "crates/liquidfun-differential/src/rigid_world/phase9/comparator.rs",
    "tools/xtask/src/phase9_evidence.rs",
    ".planning/phases/09-particle-storage-lifecycle-and-coupling/09-27-SUMMARY.md",
    ".planning/phases/09-particle-storage-lifecycle-and-coupling/09-28-SUMMARY.md",
];
const PHASE9_DEFERRED_IDS: [&str; 5] = [
    "public-api.liquidfun-box2d-box2d-particle-b2particleassembly-h",
    "public-api.liquidfun-box2d-box2d-particle-b2particlegroup-h",
    "source-area.liquidfun-box2d-box2d-particle",
    "subsystem.particle-groups-pairs-and-triads",
    "subsystem.particle-solver-behaviors",
];
static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

type TestResult = Result<(), Box<dyn Error>>;

struct InventoryFixture {
    root: PathBuf,
}

impl InventoryFixture {
    fn new() -> io::Result<Self> {
        let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let root = workspace_root().join(format!(
            "target/xtask-inventory-fixtures/{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("third_party/liquidfun/liquidfun/Box2D/Box2D/Common"))?;
        fs::create_dir_all(root.join("third_party/liquidfun/liquidfun/Box2D/Unittests"))?;
        fs::create_dir_all(root.join("third_party/liquidfun/liquidfun/Box2D/Testbed/Tests"))?;
        fs::create_dir_all(root.join("third_party/liquidfun/liquidfun/Box2D/HelloWorld"))?;
        fs::create_dir_all(root.join("reference"))?;
        fs::write(
            root.join("reference/upstream-lock.toml"),
            format!("schema_version = 1\nrevision = \"{REVISION}\"\n"),
        )?;
        fs::write(
            root.join("third_party/liquidfun/liquidfun/Box2D/Box2D/Common/b2Fixture.h"),
            "// fixture\n",
        )?;
        fs::write(
            root.join("third_party/liquidfun/liquidfun/Box2D/HelloWorld/HelloWorld.cpp"),
            "// fixture\n",
        )?;
        fs::write(
            root.join("third_party/liquidfun/liquidfun/Box2D/CMakeLists.txt"),
            "# fixture\n",
        )?;
        let fixture = Self { root };
        fixture.write_compatibility(&Self::valid_entries())?;
        Ok(fixture)
    }

    fn command(&self, arguments: &[&str]) -> io::Result<Output> {
        Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(arguments)
            .env("LIQUIDFUN_XTASK_ROOT", &self.root)
            .output()
    }

    fn discover(&self) -> io::Result<Output> {
        self.command(&["inventory", "discover"])
    }

    fn generate(&self) -> io::Result<Output> {
        self.command(&["inventory", "generate"])
    }

    fn check(&self) -> io::Result<Output> {
        self.command(&["inventory", "check"])
    }

    fn check_report(&self) -> io::Result<Output> {
        self.command(&["inventory", "check-report"])
    }

    fn valid_entries() -> Vec<Value> {
        let evidence = valid_evidence();
        vec![
            compatibility_entry(
                "example.hello-world",
                "example",
                "liquidfun/Box2D/HelloWorld/HelloWorld.cpp",
                "unassigned",
                &evidence,
            ),
            compatibility_entry(
                "public-api.fixture",
                "public_api",
                "liquidfun/Box2D/Box2D/Common/b2Fixture.h",
                "liquidfun::internal",
                &evidence,
            ),
        ]
    }

    fn write_compatibility(&self, entries: &[Value]) -> io::Result<()> {
        let ledger = json!({
            "schema_version": 1,
            "oracle_revision": REVISION,
            "sort_contract": "entries are ordered lexicographically by id",
            "evidence_dimensions": [
                "investigated",
                "planned",
                "implemented",
                "unit_tested",
                "differentially_validated",
                "platform_validated",
                "documented_difference",
                "intentionally_unsupported"
            ],
            "entries": entries
        });
        let bytes = serde_json::to_vec_pretty(&ledger).map_err(io::Error::other)?;
        fs::write(self.root.join("reference/compatibility.json"), bytes)
    }

    fn cleanup(self) -> io::Result<()> {
        fs::remove_dir_all(self.root)
    }
}

#[test]
fn check_rejects_unknown_schema_fields() -> TestResult {
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

#[test]
fn check_rejects_duplicate_stable_ids() -> TestResult {
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

#[test]
fn check_rejects_distinct_ids_for_the_same_upstream_mapping() -> TestResult {
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

#[test]
fn check_rejects_unmapped_discovery_entries() -> TestResult {
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

#[test]
fn check_rejects_differential_evidence_without_dependencies() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let mut entries = InventoryFixture::valid_entries();
    entries[0]["evidence"]["differentially_validated"] = evidenced();
    fixture.write_compatibility(&entries)?;

    // Act
    let output = fixture.check()?;

    // Assert
    assert_failure_category(&output, "inventory/evidence");
    fixture.cleanup()?;
    Ok(())
}

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

#[test]
fn discover_and_generate_are_byte_deterministic() -> TestResult {
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

#[test]
fn report_check_uses_validated_ledgers_without_native_sources() -> TestResult {
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

fn compatibility_entry(
    id: &str,
    kind: &str,
    upstream_path: &str,
    rust_target: &str,
    evidence: &Value,
) -> Value {
    json!({
        "id": id,
        "kind": kind,
        "upstream_path": upstream_path,
        "upstream_symbol": null,
        "applicability": {
            "status": "applicable",
            "rationale": "Fixture remains in scope."
        },
        "rust_target": rust_target,
        "provenance_ref": "reference/upstream-lock.toml#revision",
        "notice_refs": ["fixture-notice"],
        "evidence": evidence
    })
}

fn valid_evidence() -> Value {
    json!({
        "investigated": evidenced(),
        "planned": evidenced(),
        "implemented": not_evidenced(),
        "unit_tested": not_evidenced(),
        "differentially_validated": not_evidenced(),
        "platform_validated": not_evidenced(),
        "documented_difference": not_evidenced(),
        "intentionally_unsupported": not_evidenced()
    })
}

fn promoted_phase9_entries() -> Vec<Value> {
    let evidence = json!({
        "investigated": evidenced(),
        "planned": evidenced(),
        "implemented": evidenced(),
        "unit_tested": evidenced(),
        "differentially_validated": {
            "status": "evidenced",
            "references": PHASE9_DIFFERENTIAL_REFERENCES
        },
        "platform_validated": {
            "status": "evidenced",
            "references": PHASE9_AUTHORITY_REFERENCES
        },
        "documented_difference": not_evidenced(),
        "intentionally_unsupported": not_evidenced()
    });
    let mut entries = InventoryFixture::valid_entries();
    entries.extend(
        [
            "public-api.liquidfun-box2d-box2d-particle-b2particle-h",
            "public-api.liquidfun-box2d-box2d-particle-b2particlesystem-h",
            "subsystem.particle-contacts-and-coupling",
            "subsystem.particle-storage-and-lifecycle",
        ]
        .into_iter()
        .map(|id| {
            compatibility_entry(
                id,
                "subsystem",
                "phase9-fixture",
                "liquidfun::particle",
                &evidence,
            )
        }),
    );
    entries.sort_by(|left, right| left["id"].as_str().cmp(&right["id"].as_str()));
    entries
}

fn phase9_entry_mut(entries: &mut [Value]) -> &mut Value {
    let maybe_entry = entries
        .iter_mut()
        .find(|entry| entry["id"] == "public-api.liquidfun-box2d-box2d-particle-b2particle-h");
    let Some(entry) = maybe_entry else {
        panic!("promoted Phase 9 fixture must include the particle API row");
    };
    entry
}

fn evidenced() -> Value {
    json!({"status": "evidenced", "references": ["fixture"]})
}

fn not_evidenced() -> Value {
    json!({"status": "not_evidenced", "references": []})
}

fn assert_success(output: &Output) {
    assert!(output.status.success(), "{}", stderr(output));
}

fn assert_failure_category(output: &Output, category: &str) {
    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert!(
        stderr(output).contains(category),
        "expected `{category}` in stderr:\n{}",
        stderr(output)
    );
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .components()
        .collect()
}
