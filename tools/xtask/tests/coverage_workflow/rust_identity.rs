//! Execute the Linux coverage producer against bounded compiler/tool fixtures.

use std::process::Command;

#[test]
fn rust_coverage_envelope_matches_the_executed_compiler() -> Result<(), Box<dyn std::error::Error>>
{
    // Arrange / Act
    for mode in ["nightly", "failed", "stable"] {
        let output = Command::new("timeout")
            .args([
                "10s",
                "bash",
                "tools/xtask/tests/coverage_workflow/rust-identity.sh",
                mode,
            ])
            .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
            .output()?;
        // Assert
        assert!(
            output.status.success(),
            "{mode}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}
