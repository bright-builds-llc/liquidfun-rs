//! Bounded subprocess controls for Miri producer evidence.

use std::process::Command;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn miri_scanner_errors_cannot_become_clean_evidence() -> TestResult {
    // Arrange / Act
    for mode in ["missing", "error", "finding"] {
        let output = Command::new("timeout")
            .args([
                "10s",
                "bash",
                "tools/xtask/tests/safety_evidence_contract/miri-control.sh",
                mode,
            ])
            .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
            .output()?;
        // Assert
        assert_eq!(output.status.code(), Some(64));
        assert!(String::from_utf8_lossy(&output.stderr).contains("Miri source scan"));
    }
    Ok(())
}

#[test]
fn miri_requires_the_endpoint_and_nonzero_default_mode() -> TestResult {
    // Arrange / Act
    for mode in ["missing-endpoint", "zero-default", "wrong-flags", "valid"] {
        let output = Command::new("timeout")
            .args([
                "10s",
                "bash",
                "tools/xtask/tests/safety_evidence_contract/miri-control.sh",
                mode,
            ])
            .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
            .output()?;
        // Assert
        assert_eq!(
            output.status.success(),
            mode == "valid",
            "{mode}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        if mode != "valid" {
            assert!(String::from_utf8_lossy(&output.stderr).contains("Miri math modes"));
        }
    }
    Ok(())
}
