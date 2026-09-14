//! Behavioral coverage for the bounded candidate evidence orchestration.

use std::{path::Path, process::Command};

#[test]
fn candidate_evidence_preserves_identity_and_attempt_boundaries()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");

    // Act
    let output = Command::new("timeout")
        .args(["120", "python3", "-B"])
        .arg(root.join("tools/xtask/tests/phase15_candidate_evidence/checks.py"))
        .current_dir(&root)
        .output()?;

    // Assert
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
