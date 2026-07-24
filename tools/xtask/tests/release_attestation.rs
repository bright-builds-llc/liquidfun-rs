//! Fail-closed acceptance tests for frozen-source release attestation.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

static TEST_ORDINAL: AtomicU64 = AtomicU64::new(1);

struct Fixture {
    repository: PathBuf,
    root: PathBuf,
    candidate: String,
    source_tree_sha256: String,
    manifest_path: PathBuf,
    report_path: PathBuf,
}

impl Fixture {
    fn minimal(name: &str) -> Self {
        let repository = repository_root();
        let ordinal = TEST_ORDINAL.fetch_add(1, Ordering::Relaxed);
        let root = repository
            .join("target/xtask-attestation-tests")
            .join(format!("{name}-{}-{ordinal}", std::process::id()));
        fs::create_dir_all(&root).expect("fixture directory");
        let candidate = git_output(&repository, &["rev-parse", "HEAD"]);
        let source_tree = Command::new("git")
            .current_dir(&repository)
            .args(["ls-tree", "-r", "-z", "--full-tree", &candidate])
            .output()
            .expect("git ls-tree");
        assert!(source_tree.status.success(), "git ls-tree failed");
        let source_tree_sha256 = sha256(&source_tree.stdout);
        let manifest_path = root.join("candidate-manifest.json");
        let report_path = root.join("audit-report.json");
        write_json(
            &manifest_path,
            &json!({
                "schema_version": 1,
                "candidate_commit": candidate,
                "items": [],
            }),
        );
        write_json(
            &report_path,
            &json!({
                "schema_version": 1,
                "decision": "ready",
                "candidate_commit": candidate,
                "evidence_count": 0,
                "evidence": [],
            }),
        );
        Self {
            repository,
            root,
            candidate,
            source_tree_sha256,
            manifest_path,
            report_path,
        }
    }

    fn source_path(&self, ready: bool) -> PathBuf {
        self.source_path_with(
            ready,
            &self.candidate,
            &self.source_tree_sha256,
            &sha256(&fs::read(&self.manifest_path).expect("manifest")),
            &sha256(&fs::read(&self.report_path).expect("report")),
        )
    }

    fn source_path_with(
        &self,
        ready: bool,
        candidate: &str,
        source_tree_sha256: &str,
        manifest_sha256: &str,
        report_sha256: &str,
    ) -> PathBuf {
        let path = self.root.join("source-candidate.json");
        write_json(
            &path,
            &json!({
                "schema_version": 1,
                "ready": ready,
                "source_candidate_commit": candidate,
                "source_tree_sha256": source_tree_sha256,
                "candidate_manifest_sha256": manifest_sha256,
                "audit_report_sha256": report_sha256,
            }),
        );
        path
    }

    fn run(&self, source_path: &Path) -> Output {
        Command::new(env!("CARGO_BIN_EXE_xtask"))
            .current_dir(&self.repository)
            .args([
                "release",
                "attestation",
                "validate-worktree",
                "--source",
                &repository_relative(&self.repository, source_path),
                "--manifest",
                &repository_relative(&self.repository, &self.manifest_path),
                "--report",
                &repository_relative(&self.repository, &self.report_path),
            ])
            .output()
            .expect("release attestation process")
    }
}

#[test]
fn missing_source_record_fails_closed() {
    // Arrange
    let fixture = Fixture::minimal("missing");
    let source_path = fixture.root.join("missing-source.json");

    // Act
    let output = fixture.run(&source_path);

    // Assert
    assert_failure_contains(&output, "release/attestation-input");
}

#[test]
fn non_ready_source_record_fails_closed() {
    // Arrange
    let fixture = Fixture::minimal("not-ready");
    let source_path = fixture.source_path(false);

    // Act
    let output = fixture.run(&source_path);

    // Assert
    assert_failure_contains(&output, "release/attestation-source");
}

#[test]
fn current_head_cannot_replace_a_mixed_manifest_candidate() {
    // Arrange
    let fixture = Fixture::minimal("mixed-candidate");
    write_json(
        &fixture.manifest_path,
        &json!({
            "schema_version": 1,
            "candidate_commit": "1111111111111111111111111111111111111111",
            "items": [],
        }),
    );
    let source_path = fixture.source_path_with(
        true,
        &fixture.candidate,
        &fixture.source_tree_sha256,
        &sha256(&fs::read(&fixture.manifest_path).expect("manifest")),
        &sha256(&fs::read(&fixture.report_path).expect("report")),
    );

    // Act
    let output = fixture.run(&source_path);

    // Assert
    assert_failure_contains(&output, "release/attestation-candidate");
}

#[test]
fn malformed_source_record_is_rejected() {
    // Arrange
    let fixture = Fixture::minimal("malformed-source");
    let source_path = fixture.root.join("source-candidate.json");
    write_json(
        &source_path,
        &json!({
            "schema_version": 1,
            "ready": true,
            "source_candidate_commit": fixture.candidate,
            "source_tree_sha256": fixture.source_tree_sha256,
            "candidate_manifest_sha256": sha256(
                &fs::read(&fixture.manifest_path).expect("manifest")
            ),
            "audit_report_sha256": sha256(&fs::read(&fixture.report_path).expect("report")),
            "substituted": true,
        }),
    );

    // Act
    let output = fixture.run(&source_path);

    // Assert
    assert_failure_contains(&output, "release/attestation-source");
}

#[test]
fn source_tree_hash_is_independently_recomputed() {
    // Arrange
    let fixture = Fixture::minimal("source-tree");
    let source_path = fixture.source_path_with(
        true,
        &fixture.candidate,
        &"0".repeat(64),
        &sha256(&fs::read(&fixture.manifest_path).expect("manifest")),
        &sha256(&fs::read(&fixture.report_path).expect("report")),
    );

    // Act
    let output = fixture.run(&source_path);

    // Assert
    assert_failure_contains(&output, "release/attestation-source-tree");
}

#[test]
fn manifest_and_report_hashes_are_independently_recomputed() {
    // Arrange
    let manifest_fixture = Fixture::minimal("manifest-hash");
    let manifest_source = manifest_fixture.source_path_with(
        true,
        &manifest_fixture.candidate,
        &manifest_fixture.source_tree_sha256,
        &"0".repeat(64),
        &sha256(&fs::read(&manifest_fixture.report_path).expect("report")),
    );
    let report_fixture = Fixture::minimal("report-hash");
    let report_source = report_fixture.source_path_with(
        true,
        &report_fixture.candidate,
        &report_fixture.source_tree_sha256,
        &sha256(&fs::read(&report_fixture.manifest_path).expect("manifest")),
        &"0".repeat(64),
    );

    // Act
    let manifest_output = manifest_fixture.run(&manifest_source);
    let report_output = report_fixture.run(&report_source);

    // Assert
    assert_failure_contains(&manifest_output, "release/attestation-manifest-hash");
    assert_failure_contains(&report_output, "release/attestation-report-hash");
}

fn write_json(path: &Path, value: &Value) {
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("JSON serializes"),
    )
    .expect("JSON writes");
}

fn git_output(repository: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(repository)
        .args(args)
        .output()
        .expect("git command");
    assert!(output.status.success(), "git command failed");
    String::from_utf8(output.stdout)
        .expect("git output is UTF-8")
        .trim()
        .to_owned()
}

fn assert_failure_contains(output: &Output, category: &str) {
    assert!(
        !output.status.success(),
        "attestation unexpectedly passed: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(category), "{category}: {stderr}");
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("xtask is nested under tools/")
        .to_path_buf()
}

fn repository_relative(repository: &Path, target_path: &Path) -> String {
    target_path
        .strip_prefix(repository)
        .expect("fixture remains in repository")
        .to_string_lossy()
        .replace('\\', "/")
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
