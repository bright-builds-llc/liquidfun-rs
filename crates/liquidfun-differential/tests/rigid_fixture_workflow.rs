//! Real-binary lifecycle coverage for canonical rigid-world evidence.

use std::{
    fs, io,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_REPOSITORY: AtomicU64 = AtomicU64::new(0);

struct RigidFixtureRepository {
    root: PathBuf,
    oracle_directory: PathBuf,
}

impl RigidFixtureRepository {
    fn new(behavior: &str) -> io::Result<Self> {
        let sequence = NEXT_REPOSITORY.fetch_add(1, Ordering::Relaxed);
        let root = workspace_root().join(format!(
            "target/rigid-fixture-workflow-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("protocol/fixtures/accepted"))?;
        fs::create_dir_all(root.join("protocol/tolerances"))?;
        fs::create_dir_all(root.join("reference/artifacts"))?;
        fs::create_dir_all(root.join("scenarios/regressions"))?;
        fs::copy(
            workspace_root().join("protocol/fixtures/accepted/rigid-world-request.jsonl"),
            root.join("protocol/fixtures/accepted/rigid-world-request.jsonl"),
        )?;
        fs::copy(
            workspace_root().join("protocol/tolerances/phase6-v1.toml"),
            root.join("protocol/tolerances/phase6-v1.toml"),
        )?;
        fs::copy(
            workspace_root().join("reference/artifacts/manifest.toml"),
            root.join("reference/artifacts/manifest.toml"),
        )?;
        fs::write(root.join("THIRD_PARTY_NOTICES.md"), "fixture notices\n")?;
        run_git(&root, &["init", "--quiet"])?;
        run_git(&root, &["config", "user.name", "Fixture User"])?;
        run_git(&root, &["config", "user.email", "fixture@example.invalid"])?;
        run_git(&root, &["add", "."])?;
        run_git(&root, &["commit", "--quiet", "-m", "fixture"])?;

        let oracle_directory = root.join("target/reference/oracle-debug");
        fs::create_dir_all(&oracle_directory)?;
        fs::copy(
            env!("CARGO_BIN_EXE_liquidfun-fake-oracle"),
            oracle_directory.join(oracle_name()),
        )?;
        fs::write(oracle_directory.join("behavior.txt"), behavior)?;
        Ok(Self {
            root,
            oracle_directory,
        })
    }

    fn stage(&self, artifact_id: &str) -> io::Result<Output> {
        self.command(&[
            "fixture",
            "stage",
            "--scenario",
            "rigid-world",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "one-shot",
            "--artifact-kind",
            "reviewed-trace",
            "--artifact-id",
            artifact_id,
        ])
    }

    fn review(&self, artifact_id: &str) -> io::Result<Output> {
        self.command(&[
            "fixture",
            "review",
            "--artifact-id",
            artifact_id,
            "--reviewer",
            "fixture-reviewer",
            "--reviewed-at",
            "2026-07-12T12:00:00Z",
            "--review-status",
            "approved",
        ])
    }

    fn promote(&self, artifact_id: &str) -> io::Result<Output> {
        self.command(&["fixture", "promote", "--artifact-id", artifact_id])
    }

    fn command(&self, arguments: &[&str]) -> io::Result<Output> {
        Command::new(env!("CARGO_BIN_EXE_liquidfun-differential"))
            .current_dir(&self.root)
            .args(arguments)
            .output()
    }

    fn set_behavior(&self, behavior: &str) -> io::Result<()> {
        fs::write(self.oracle_directory.join("behavior.txt"), behavior)
    }

    fn candidate(&self, artifact_id: &str) -> PathBuf {
        self.root
            .join("target/differential/staging")
            .join(artifact_id)
    }
}

impl Drop for RigidFixtureRepository {
    fn drop(&mut self) {
        let _ignored = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn transaction_real_binary_stages_replays_and_promotes_canonical_rigid_trace()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1")?;

    // Act
    let staged = repository.stage("canonical-rigid")?;
    let reviewed = repository.review("canonical-rigid")?;
    let promoted = repository.promote("canonical-rigid")?;

    // Assert
    assert!(staged.status.success(), "{}", stderr(&staged));
    assert!(reviewed.status.success(), "{}", stderr(&reviewed));
    assert!(promoted.status.success(), "{}", stderr(&promoted));
    assert!(
        repository
            .candidate("canonical-rigid")
            .join("review.toml")
            .is_file()
    );
    assert!(
        repository
            .root
            .join("reference/artifacts/traces/phase-06-rigid-world-v1.jsonl")
            .is_file()
    );
    let manifest = fs::read_to_string(repository.root.join("reference/artifacts/manifest.toml"))?;
    assert!(manifest.contains("phase-06-rigid-world-v1.jsonl"));
    Ok(())
}

#[test]
fn real_binary_rejects_d2_before_staging_or_accepted_mutation()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d2")?;
    let manifest_before = fs::read(repository.root.join("reference/artifacts/manifest.toml"))?;

    // Act
    let output = repository.stage("noncanonical-rigid")?;

    // Assert
    assert!(!output.status.success());
    assert!(stderr(&output).contains("requires D1 canonical authority"));
    assert!(!repository.candidate("noncanonical-rigid").exists());
    assert!(!repository.root.join("target/differential/staging").exists());
    assert_eq!(
        fs::read(repository.root.join("reference/artifacts/manifest.toml"))?,
        manifest_before
    );
    assert!(
        !repository
            .root
            .join("reference/artifacts/traces/phase-06-rigid-world-v1.jsonl")
            .exists()
    );
    Ok(())
}

#[test]
fn transaction_replay_rejects_dirty_rigid_candidate_before_review()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1")?;
    let staged = repository.stage("dirty-rigid")?;
    assert!(staged.status.success(), "{}", stderr(&staged));
    fs::write(
        repository.candidate("dirty-rigid").join("trace.jsonl"),
        b"tampered\n",
    )?;

    // Act
    let reviewed = repository.review("dirty-rigid")?;

    // Assert
    assert!(!reviewed.status.success());
    assert!(stderr(&reviewed).contains("SHA-256 mismatch"));
    assert!(
        !repository
            .candidate("dirty-rigid")
            .join("review.toml")
            .exists()
    );
    Ok(())
}

#[test]
fn real_binary_propagates_rigid_child_failure_without_candidate_creation()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let repository = RigidFixtureRepository::new("rigid_d1")?;
    repository.set_behavior("rigid_d1_nonzero")?;

    // Act
    let output = repository.stage("failed-child")?;

    // Assert
    assert!(!output.status.success());
    assert!(!repository.candidate("failed-child").exists());
    Ok(())
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should be present")
        .to_path_buf()
}

fn oracle_name() -> &'static str {
    if cfg!(windows) {
        "liquidfun-reference.exe"
    } else {
        "liquidfun-reference"
    }
}

fn run_git(root: &Path, arguments: &[&str]) -> io::Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .output()?;
    if output.status.success() {
        return Ok(());
    }
    Err(io::Error::other(format!(
        "git {} failed: {}",
        arguments.join(" "),
        String::from_utf8_lossy(&output.stderr).trim()
    )))
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
