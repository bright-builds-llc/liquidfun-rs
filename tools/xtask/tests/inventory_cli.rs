//! Command-level coverage for deterministic compatibility inventory tooling.

use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};

const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
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
