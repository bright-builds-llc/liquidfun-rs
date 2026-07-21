//! Command-level coverage for pinned semantic corpus discovery.

use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};

const WRONG_REVISION: &str = "0000000000000000000000000000000000000000";
static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

type TestResult = Result<(), Box<dyn Error>>;

struct CorpusFixture {
    root: PathBuf,
}

impl CorpusFixture {
    fn new() -> Result<Self, Box<dyn Error>> {
        let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "liquidfun-corpus-discovery-{}-{id}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        fs::create_dir_all(&root)?;
        git(&root, &["init", "-q"])?;
        fs::write(root.join("Cargo.toml"), "[workspace]\nresolver = \"3\"\n")?;

        let upstream = root.join("third_party/liquidfun");
        fs::create_dir_all(&upstream)?;
        git(&upstream, &["init", "-q"])?;
        write_valid_sources(&upstream)?;
        let revision = commit_all(&upstream, "fixture upstream")?;

        fs::create_dir_all(root.join("reference"))?;
        write_lock(&root, &revision)?;
        commit_all(&root, "fixture root")?;
        Ok(Self { root })
    }

    fn run(&self, arguments: &[&str]) -> io::Result<Output> {
        Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(arguments)
            .env("LIQUIDFUN_XTASK_ROOT", &self.root)
            .output()
    }

    fn refresh(&self) -> io::Result<Output> {
        self.run(&["inventory", "corpus", "refresh"])
    }

    fn check_snapshot(&self) -> io::Result<Output> {
        self.run(&["inventory", "corpus", "check-snapshot"])
    }

    fn commit_upstream_change(&self, message: &str) -> TestResult {
        let upstream = self.root.join("third_party/liquidfun");
        let revision = commit_all(&upstream, message)?;
        write_lock(&self.root, &revision)?;
        commit_all(&self.root, "refresh gitlink")?;
        Ok(())
    }

    fn snapshot_bytes(&self) -> io::Result<Vec<u8>> {
        fs::read(self.root.join("reference/upstream-corpus.json"))
    }

    fn cleanup(self) -> io::Result<()> {
        fs::remove_dir_all(self.root)
    }
}

#[test]
fn repeated_refresh_is_byte_identical_and_discovers_semantic_items() -> TestResult {
    // Arrange
    let fixture = CorpusFixture::new()?;

    // Act
    assert_success(&fixture.refresh()?);
    let first = fixture.snapshot_bytes()?;
    assert_success(&fixture.refresh()?);
    let second = fixture.snapshot_bytes()?;
    let snapshot: Value = serde_json::from_slice(&second)?;

    // Assert
    assert_eq!(first, second);
    let items = snapshot["items"]
        .as_array()
        .ok_or("items must be an array")?;
    assert_eq!(items.len(), 5);
    assert!(items.iter().any(|item| {
        item["kind"] == "upstream_test" && item["source"]["symbol"] == "Fixture.Works"
    }));
    assert!(items.iter().any(|item| {
        item["kind"] == "testbed_entry"
            && item["source"]["symbol"] == "Registered|Registered::Create"
    }));
    assert!(
        items.iter().any(|item| {
            item["kind"] == "example" && item["source"]["symbol"] == "Rope::Create"
        })
    );
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_snapshot_never_reads_the_upstream_checkout() -> TestResult {
    // Arrange
    let fixture = CorpusFixture::new()?;
    assert_success(&fixture.refresh()?);
    fs::rename(
        fixture.root.join("third_party"),
        fixture.root.join("third-party-hidden"),
    )?;

    // Act
    let output = fixture.check_snapshot()?;

    // Assert
    assert_success(&output);
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn refresh_preserves_complete_reviewed_classification() -> TestResult {
    // Arrange
    let fixture = CorpusFixture::new()?;
    assert_success(&fixture.refresh()?);
    let mut snapshot: Value = serde_json::from_slice(&fixture.snapshot_bytes()?)?;
    let item = snapshot["items"]
        .as_array_mut()
        .ok_or("items must be an array")?
        .iter_mut()
        .find(|item| item["source"]["symbol"] == "Fixture.Works")
        .ok_or("fixture test must be discovered")?;
    item["applicability"] = json!("applicable");
    item["disposition"] = json!("native_port");
    item["compatibility_impact"] = json!("behavioral");
    item["evidence"] = json!([{
        "kind": "native_test",
        "reference": "crates/liquidfun/tests/fixture.rs#works"
    }]);
    item["review"] = json!({
        "reviewer": "phase-11-plan-02",
        "reviewed_on": "2026-07-21",
        "rationale": "The native fixture test exercises the same observable behavior."
    });
    let mut bytes = serde_json::to_vec_pretty(&snapshot)?;
    bytes.push(b'\n');
    fs::write(fixture.root.join("reference/upstream-corpus.json"), bytes)?;

    // Act
    assert_success(&fixture.refresh()?);
    let refreshed: Value = serde_json::from_slice(&fixture.snapshot_bytes()?)?;

    // Assert
    let item = refreshed["items"]
        .as_array()
        .ok_or("items must be an array")?
        .iter()
        .find(|item| item["source"]["symbol"] == "Fixture.Works")
        .ok_or("fixture test must remain discovered")?;
    assert_eq!(item["disposition"], "native_port");
    assert_eq!(item["review"]["reviewer"], "phase-11-plan-02");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn malformed_test_macro_fails_closed() -> TestResult {
    // Arrange
    let fixture = CorpusFixture::new()?;
    fs::write(
        fixture
            .root
            .join("third_party/liquidfun/liquidfun/Box2D/Unittests/Fixture/FixtureTests.cpp"),
        "TEST_F(Fixture Works) {}\n",
    )?;
    fixture.commit_upstream_change("malformed macro")?;

    // Act
    let output = fixture.refresh()?;

    // Assert
    assert_failure_category(&output, "inventory/corpus-source");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn duplicate_testbed_registration_fails_closed() -> TestResult {
    // Arrange
    let fixture = CorpusFixture::new()?;
    let entries = fixture
        .root
        .join("third_party/liquidfun/liquidfun/Box2D/Testbed/Tests/TestEntries.cpp");
    fs::write(
        entries,
        "TestEntry g_testEntries[] = {\n{\"Registered\", Registered::Create},\n{\"Registered\", Registered::Create},\n{NULL, NULL}\n};\n",
    )?;
    fixture.commit_upstream_change("duplicate registration")?;

    // Act
    let output = fixture.refresh()?;

    // Assert
    assert_failure_category(&output, "inventory/corpus-source");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn unknown_test_source_fails_closed() -> TestResult {
    // Arrange
    let fixture = CorpusFixture::new()?;
    let unknown = fixture
        .root
        .join("third_party/liquidfun/liquidfun/Box2D/Unittests/Unknown/UnknownTests.cpp");
    fs::create_dir_all(unknown.parent().ok_or("unknown source needs a parent")?)?;
    fs::write(unknown, "int helper_only = 0;\n")?;
    fixture.commit_upstream_change("unknown test source")?;

    // Act
    let output = fixture.refresh()?;

    // Assert
    assert_failure_category(&output, "inventory/corpus-source");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn stale_snapshot_identity_fails_closed() -> TestResult {
    // Arrange
    let fixture = CorpusFixture::new()?;
    assert_success(&fixture.refresh()?);
    fs::write(
        fixture
            .root
            .join("third_party/liquidfun/liquidfun/Box2D/Unittests/Fixture/FixtureTests.cpp"),
        "TEST_F(Fixture, Renamed) {}\n",
    )?;
    fixture.commit_upstream_change("rename test identity")?;

    // Act
    let output = fixture.refresh()?;

    // Assert
    assert_failure_category(&output, "inventory/corpus-snapshot");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn wrong_revision_fails_before_source_discovery() -> TestResult {
    // Arrange
    let fixture = CorpusFixture::new()?;
    write_lock(&fixture.root, WRONG_REVISION)?;

    // Act
    let output = fixture.refresh()?;

    // Assert
    assert_failure_category(&output, "inventory/revision");
    fixture.cleanup()?;
    Ok(())
}

fn write_valid_sources(upstream: &Path) -> io::Result<()> {
    let unittests = upstream.join("liquidfun/Box2D/Unittests/Fixture");
    let tests = upstream.join("liquidfun/Box2D/Testbed/Tests");
    let hello_world = upstream.join("liquidfun/Box2D/HelloWorld");
    fs::create_dir_all(&unittests)?;
    fs::create_dir_all(&tests)?;
    fs::create_dir_all(&hello_world)?;
    fs::write(
        unittests.join("FixtureTests.cpp"),
        "TEST_F(Fixture, Works) {}\n",
    )?;
    fs::write(
        tests.join("Registered.h"),
        "class Registered : public Test { public: static Test* Create(); };\n",
    )?;
    fs::write(
        tests.join("Rope.h"),
        "class Rope : public Test { public: static Test* Create(); };\n",
    )?;
    fs::write(
        tests.join("TestEntries.cpp"),
        "TestEntry g_testEntries[] = {\n{\"Registered\", Registered::Create},\n{NULL, NULL}\n};\n",
    )?;
    fs::write(
        hello_world.join("HelloWorld.cpp"),
        "int main() { return 0; }\n",
    )
}

fn write_lock(root: &Path, revision: &str) -> io::Result<()> {
    fs::write(
        root.join("reference/upstream-lock.toml"),
        format!("schema_version = 1\nrevision = \"{revision}\"\n"),
    )
}

fn commit_all(root: &Path, message: &str) -> Result<String, Box<dyn Error>> {
    git(root, &["add", "."])?;
    git(
        root,
        &[
            "-c",
            "user.name=Corpus Fixture",
            "-c",
            "user.email=corpus@example.invalid",
            "commit",
            "-q",
            "-m",
            message,
        ],
    )?;
    let output = git(root, &["rev-parse", "HEAD"])?;
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn git(root: &Path, arguments: &[&str]) -> io::Result<Output> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .output()?;
    if output.status.success() {
        return Ok(output);
    }
    Err(io::Error::other(String::from_utf8_lossy(&output.stderr)))
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "expected success; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_failure_category(output: &Output, category: &str) {
    assert!(!output.status.success(), "expected failure");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(category),
        "expected `{category}` in stderr: {stderr}"
    );
}
