//! Executable fail-closed contract for canonical compiler provisioning.

#[cfg(unix)]
#[path = "canonical_toolchain_workflow/execution.rs"]
mod execution;

use std::{fs, path::PathBuf};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const INSTALLER: &str = "scripts/install-canonical-clang.sh";
const UPSTREAM_COMMIT: &str = "eeed6742908255f0eeb12bb8e314366eff3c0a21";
const UPSTREAM_SHA256: &str = "9474ecd78b52aba6e923976b1e9773f5613027cc7e237b9956986cb536e02a36";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("xtask belongs to the workspace")
        .to_path_buf()
}

#[test]
fn acquisition_has_one_immutable_reviewed_source() -> TestResult {
    // Arrange
    let source = fs::read_to_string(root().join(INSTALLER))?;
    // Act
    let expected_url = format!(
        "https://raw.githubusercontent.com/opencollab/llvm-jenkins.debian.net/{UPSTREAM_COMMIT}/llvm.sh"
    );
    // Assert
    assert!(source.contains(&expected_url));
    assert!(source.contains(UPSTREAM_SHA256));
    assert!(source.contains("sha256sum --check --strict"));
    assert!(!source.contains("https://apt.llvm.org/llvm.sh"));
    Ok(())
}
