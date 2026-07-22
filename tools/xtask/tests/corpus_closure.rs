//! Command-level coverage for semantic corpus closure and reporting.

use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};

static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

type TestResult = Result<(), Box<dyn Error>>;

struct ClosureFixture {
    root: PathBuf,
}

impl ClosureFixture {
    fn new() -> io::Result<Self> {
        let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let root = workspace_root().join(format!(
            "target/xtask-corpus-closure-fixtures/{}-{id}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        for relative in authority_paths() {
            let source = workspace_root().join(relative);
            let destination = root.join(relative);
            let Some(parent) = destination.parent() else {
                return Err(io::Error::other("fixture authority path needs a parent"));
            };
            fs::create_dir_all(parent)?;
            fs::copy(source, destination)?;
        }
        Ok(Self { root })
    }

    fn command(&self, arguments: &[&str]) -> io::Result<Output> {
        Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(arguments)
            .env("LIQUIDFUN_XTASK_ROOT", &self.root)
            .output()
    }

    fn generate_report(&self) -> io::Result<Output> {
        self.command(&["inventory", "corpus", "generate-report"])
    }

    fn check_closure(&self) -> io::Result<Output> {
        self.command(&["inventory", "corpus", "check-closure"])
    }

    fn corpus(&self) -> Result<Value, Box<dyn Error>> {
        Ok(serde_json::from_slice(&fs::read(
            self.root.join("reference/upstream-corpus.json"),
        )?)?)
    }

    fn write_corpus(&self, corpus: &Value) -> io::Result<()> {
        let mut bytes = serde_json::to_vec_pretty(corpus).map_err(io::Error::other)?;
        bytes.push(b'\n');
        fs::write(self.root.join("reference/upstream-corpus.json"), bytes)
    }

    fn cleanup(self) -> io::Result<()> {
        fs::remove_dir_all(self.root)
    }
}

#[test]
fn complete_corpus_closure_and_report_are_byte_stable() -> TestResult {
    // Arrange
    let fixture = ClosureFixture::new()?;

    // Act
    assert_success(&fixture.generate_report()?);
    let first = fs::read(fixture.root.join("UPSTREAM-CORPUS.md"))?;
    assert_success(&fixture.check_closure()?);
    assert_success(&fixture.generate_report()?);
    let second = fs::read(fixture.root.join("UPSTREAM-CORPUS.md"))?;

    // Assert
    assert_eq!(first, second);
    let report = String::from_utf8(second)?;
    assert!(report.contains("- Semantic items: 388"));
    assert!(report.contains("- Unresolved items: 0"));
    assert!(report.contains("## Item-level outcomes"));
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn closure_rejects_unresolved_unknown_duplicate_and_unmapped_items() -> TestResult {
    for mutation in [
        "unresolved",
        "unknown-evidence",
        "duplicate",
        "unmapped-source",
    ] {
        // Arrange
        let fixture = ClosureFixture::new()?;
        let mut corpus = fixture.corpus()?;
        let items = corpus["items"]
            .as_array_mut()
            .ok_or("corpus items must be an array")?;
        match mutation {
            "unresolved" => {
                let item = items.first_mut().ok_or("fixture needs one item")?;
                for field in [
                    "applicability",
                    "disposition",
                    "compatibility_impact",
                    "evidence",
                    "review",
                ] {
                    item.as_object_mut()
                        .ok_or("corpus item must be an object")?
                        .remove(field);
                }
            }
            "unknown-evidence" => {
                items[0]["evidence"][0]["reference"] =
                    json!("reference/scenario-catalog.json#scenario=unknown-scenario");
            }
            "duplicate" => items.push(items[0].clone()),
            "unmapped-source" => {
                items[0]["source"]["path"] = json!("liquidfun/Box2D/Unknown.cpp");
            }
            _ => return Err("unknown test mutation".into()),
        }
        fixture.write_corpus(&corpus)?;

        // Act
        let output = fixture.generate_report()?;

        // Assert
        assert_failure_category(&output, "inventory/corpus-");
        fixture.cleanup()?;
    }
    Ok(())
}

#[test]
fn closure_rejects_stale_mapping_vague_review_and_report_drift() -> TestResult {
    // Arrange: stale mapping identity
    let fixture = ClosureFixture::new()?;
    let mut mappings: Value = serde_json::from_slice(&fs::read(
        fixture
            .root
            .join("reference/artifacts/phase11/scenario-mappings.json"),
    )?)?;
    mappings["records"][0]["test_ids"] = json!(["stale-test-id"]);
    fs::write(
        fixture
            .root
            .join("reference/artifacts/phase11/scenario-mappings.json"),
        serde_json::to_vec_pretty(&mappings)?,
    )?;

    // Act / Assert
    assert_failure_category(&fixture.generate_report()?, "inventory/corpus-mapping");
    fixture.cleanup()?;

    // Arrange: vague rationale
    let fixture = ClosureFixture::new()?;
    let mut corpus = fixture.corpus()?;
    corpus["items"][0]["review"]["rationale"] = json!("n/a");
    fixture.write_corpus(&corpus)?;

    // Act / Assert
    assert_failure_category(&fixture.generate_report()?, "inventory/corpus-rationale");
    fixture.cleanup()?;

    // Arrange: generated report drift
    let fixture = ClosureFixture::new()?;
    assert_success(&fixture.generate_report()?);
    fs::write(fixture.root.join("UPSTREAM-CORPUS.md"), "stale\n")?;

    // Act / Assert
    assert_failure_category(&fixture.check_closure()?, "inventory/corpus-report");
    fixture.cleanup()?;
    Ok(())
}

fn authority_paths() -> [&'static str; 6] {
    [
        "reference/upstream-lock.toml",
        "reference/upstream-corpus.json",
        "reference/discovery.json",
        "reference/compatibility.json",
        "reference/scenario-catalog.json",
        "reference/artifacts/phase11/scenario-mappings.json",
    ]
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "expected success; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_failure_category(output: &Output, category: &str) {
    assert!(!output.status.success(), "command unexpectedly succeeded");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(category),
        "expected `{category}` in stderr: {stderr}"
    );
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .components()
        .collect()
}
