//! Command-level coverage for deterministic compatibility inventory tooling.

use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};

const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const PHASE9_AUTHORITY_REFERENCES: [&str; 10] = [
    "https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29439515367",
    "https://api.github.com/repos/bright-builds-llc/liquidfun-rs/actions/artifacts/8352859391/zip#sha256=f237d6f1ebe0e59f65a5ae0609140eecdd8b32247e9d2064c83748be1ab9f5ea",
    "phase9-canonical-29439515367-a87f84bbdbfe55fb732d74c481c4a4bda9eec958/identity.json#trace-sha256=3a339387b4c4acccc15b5fc4944d6bec9c7e1d315f4753034ae52a5ff97f2e64",
    "phase9-canonical-29439515367-a87f84bbdbfe55fb732d74c481c4a4bda9eec958/identity.json#manifest-sha256=36cfaad1f56505f8427408733e2231ad613984a4cb3eb3b8d757e7a14b2c38e0",
    "https://api.github.com/repos/bright-builds-llc/liquidfun-rs/actions/artifacts/8352881868/zip#sha256=95ad57e5d5711ae6aa93847ad1efd4a04025bd2956b4996535fa0e5f45a5893f",
    "phase9-sanitizer-29439515367-a87f84bbdbfe55fb732d74c481c4a4bda9eec958/identity.json#trace-sha256=ee75462d49275c5b7d02b8677eb6f9bf82c241c6b993c16d6df08a2ae231a070",
    "phase9-sanitizer-29439515367-a87f84bbdbfe55fb732d74c481c4a4bda9eec958/identity.json#manifest-sha256=0c89f0136eda6689118d3eaa909defb1d182d5723e7a64ea1e958396066dce15",
    "https://github.com/bright-builds-llc/liquidfun-rs/commit/a87f84bbdbfe55fb732d74c481c4a4bda9eec958",
    ".planning/phases/09-particle-storage-lifecycle-and-coupling/09-16-SUMMARY.md",
    "TESTING.md#phase-9-canonical-evidence-2026-07-15",
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
fn check_rejects_phase10_claim_during_phase9_promotion() -> TestResult {
    // Arrange
    let fixture = InventoryFixture::new()?;
    assert_success(&fixture.discover()?);
    let mut entries = promoted_phase9_entries();
    let mut deferred_evidence = valid_evidence();
    deferred_evidence["implemented"] = evidenced();
    entries.push(compatibility_entry(
        "subsystem.particle-solver-behaviors",
        "subsystem",
        "phase10-fixture",
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
        "differentially_validated": evidenced(),
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
