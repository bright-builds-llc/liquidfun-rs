//! Fixture CLI provenance checks for dirty generator inputs.

use std::{
    fs, io,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

struct GitFixture {
    root: PathBuf,
}

impl GitFixture {
    fn new() -> io::Result<Self> {
        let sequence = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/fixture-cli-tests")
            .join(format!("{}-{sequence}", std::process::id()));
        fs::create_dir_all(root.join("crates/liquidfun-differential/src"))?;
        fs::write(
            root.join("crates/liquidfun-differential/src/main.rs"),
            "fn main() {}\n",
        )?;
        run_git(&root, &["init", "--quiet"])?;
        run_git(&root, &["config", "user.name", "Fixture User"])?;
        run_git(&root, &["config", "user.email", "fixture@example.invalid"])?;
        run_git(&root, &["add", "."])?;
        run_git(&root, &["commit", "--quiet", "-m", "fixture"])?;
        Ok(Self { root })
    }

    fn stage_command(&self) -> io::Result<Output> {
        Command::new(env!("CARGO_BIN_EXE_liquidfun-differential"))
            .current_dir(&self.root)
            .args([
                "fixture",
                "stage",
                "--scenario",
                "empty-world",
                "--preset",
                "oracle-debug",
                "--session-profile",
                "one-shot",
                "--artifact-kind",
                "reviewed-trace",
                "--artifact-id",
                "dirty-generator",
            ])
            .output()
    }
}

impl Drop for GitFixture {
    fn drop(&mut self) {
        let _ignored = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn fixture_stage_rejects_modified_generator_source() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let fixture = GitFixture::new()?;
    fs::write(
        fixture
            .root
            .join("crates/liquidfun-differential/src/main.rs"),
        "fn main() { changed(); }\n",
    )?;

    // Act
    let output = fixture.stage_command()?;

    // Assert
    assert_eq!(output.status.code(), Some(64));
    assert!(String::from_utf8_lossy(&output.stderr).contains("generator inputs are dirty"));
    Ok(())
}

#[test]
fn fixture_stage_rejects_untracked_generator_source() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let fixture = GitFixture::new()?;
    fs::write(
        fixture
            .root
            .join("crates/liquidfun-differential/src/untracked.rs"),
        "pub fn changed() {}\n",
    )?;

    // Act
    let output = fixture.stage_command()?;

    // Assert
    assert_eq!(output.status.code(), Some(64));
    assert!(String::from_utf8_lossy(&output.stderr).contains("generator inputs are dirty"));
    Ok(())
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
