//! Behavioral coverage for the bounded candidate evidence orchestration.

#![cfg(unix)]

use std::{path::Path, process::Command};

#[test]
fn native_process_capture_preserves_bounds_and_owned_tree_cleanup()
-> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");

    // Act
    let output = Command::new("timeout")
        .args(["90", "python3", "-B"])
        .arg(root.join("tools/xtask/tests/phase15_candidate_evidence/process_controls.py"))
        .current_dir(&root)
        .output()?;

    // Assert
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn docs_ci_restores_only_explicit_attested_release_inputs() -> Result<(), Box<dyn std::error::Error>>
{
    // Arrange
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");

    // Act
    let output = Command::new("timeout")
        .args(["120", "python3", "-B"])
        .arg(root.join("tools/xtask/tests/phase15_candidate_evidence/docs_ci_checks.py"))
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
