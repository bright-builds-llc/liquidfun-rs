//! Command-level coverage for deterministic compatibility inventory tooling.

use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};

#[path = "inventory_cli/phase10.rs"]
mod phase10;
#[path = "inventory_cli/phase11.rs"]
mod phase11;
#[path = "inventory_cli/phase9.rs"]
mod phase9;

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
        fs::create_dir_all(root.join("reference/artifacts"))?;
        fs::create_dir_all(root.join("reference/coverage"))?;
        fs::create_dir_all(root.join("reference/performance"))?;
        fs::create_dir_all(root.join("reference/platform"))?;
        fs::create_dir_all(root.join("reference/regressions"))?;
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
        fs::write(
            root.join("reference/artifacts/manifest.toml"),
            "schema_version = 2\n",
        )?;
        fs::write(
            root.join("reference/performance/manifest.toml"),
            "schema_version = 1\n",
        )?;
        fs::write(
            root.join("reference/regressions/manifest.toml"),
            "schema_version = 1\n",
        )?;
        fs::write(
            root.join("reference/coverage/contract.json"),
            "{\"schema_version\":1,\"parity_authority\":false}\n",
        )?;
        fs::write(
            root.join("reference/platform/support.json"),
            "{\"schema_version\":1,\"evidence_tier\":\"d2_supported\"}\n",
        )?;
        let fixture = Self { root };
        fixture.write_corpus(&terminal_corpus_item())?;
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
        vec![
            compatibility_entry(
                "example.hello-world",
                "example",
                "liquidfun/Box2D/HelloWorld/HelloWorld.cpp",
                "unassigned",
                &valid_evidence(),
            ),
            compatibility_entry(
                "public-api.fixture",
                "public_api",
                "liquidfun/Box2D/Box2D/Common/b2Fixture.h",
                "liquidfun::internal",
                &d1_evidence(),
            ),
        ]
    }

    fn write_compatibility(&self, entries: &[Value]) -> io::Result<()> {
        self.write_ledger(entries, &release_dispositions(entries))
    }

    fn write_ledger(&self, entries: &[Value], release_dispositions: &[Value]) -> io::Result<()> {
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
            "entries": entries,
            "release_dispositions": release_dispositions
        });
        let bytes = serde_json::to_vec_pretty(&ledger).map_err(io::Error::other)?;
        fs::write(self.root.join("reference/compatibility.json"), bytes)
    }

    fn write_corpus(&self, item: &Value) -> io::Result<()> {
        let corpus = json!({
            "schema_version": 1,
            "oracle_revision": REVISION,
            "items": [item]
        });
        let bytes = serde_json::to_vec_pretty(&corpus).map_err(io::Error::other)?;
        fs::write(self.root.join("reference/upstream-corpus.json"), bytes)
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

#[test]
fn check_rejects_omitted_release_identity() -> TestResult {
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

#[test]
fn check_rejects_duplicate_release_identity() -> TestResult {
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

#[test]
fn check_rejects_unexplained_release_row() -> TestResult {
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

#[test]
fn check_rejects_nonterminal_corpus_item() -> TestResult {
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

#[test]
fn check_rejects_mixed_commit_parity_evidence() -> TestResult {
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

#[test]
fn check_rejects_empty_release_rationale() -> TestResult {
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

#[test]
fn check_rejects_coverage_as_parity_authority() -> TestResult {
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

#[test]
fn check_rejects_d2_as_parity_authority() -> TestResult {
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

fn d1_evidence() -> Value {
    json!({
        "investigated": evidenced(),
        "planned": evidenced(),
        "implemented": evidenced(),
        "unit_tested": evidenced(),
        "differentially_validated": evidenced(),
        "platform_validated": not_evidenced(),
        "documented_difference": not_evidenced(),
        "intentionally_unsupported": not_evidenced()
    })
}

fn release_dispositions(entries: &[Value]) -> Vec<Value> {
    entries
        .iter()
        .map(|entry| {
            let id = &entry["id"];
            if id == "example.hello-world" {
                return json!({
                    "id": id,
                    "outcome": "corpus_terminal",
                    "rationale": "The terminal semantic corpus review accounts for this upstream example.",
                    "references": ["reference/upstream-corpus.json#id=example.hello-world"]
                });
            }
            json!({
                "id": id,
                "outcome": "d1_canonical",
                "rationale": "Implementation, unit, and differential records provide canonical parity evidence.",
                "references": ["fixture"]
            })
        })
        .collect()
}

fn terminal_corpus_item() -> Value {
    json!({
        "id": "example.hello-world",
        "kind": "example",
        "source": {
            "path": "liquidfun/Box2D/HelloWorld/HelloWorld.cpp",
            "symbol": "main"
        },
        "applicability": "applicable",
        "disposition": "equivalent_evidence",
        "compatibility_impact": "behavioral",
        "evidence": [{
            "kind": "compatibility_ledger",
            "reference": "reference/compatibility.json#id=example.hello-world"
        }],
        "review": {
            "reviewer": "phase-12-plan-14-test",
            "reviewed_on": "2026-07-23",
            "rationale": "The fixture terminal review binds this upstream example to its compatibility identity."
        }
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
