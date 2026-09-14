//! Execute the Linux coverage producer against bounded compiler/tool fixtures.

use std::process::Command;

#[test]
fn rust_coverage_envelope_matches_the_executed_compiler() -> Result<(), Box<dyn std::error::Error>>
{
    // Arrange / Act
    for mode in ["nightly", "failed", "override", "stable"] {
        let output = Command::new("timeout")
            .args([
                "10s",
                "bash",
                "tools/xtask/tests/coverage_workflow/rust-identity.sh",
                mode,
            ])
            .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
            // The fixture supplies its own tools, independently of enclosing coverage wrappers.
            .env_remove("RUSTC")
            .env_remove("RUSTC_WRAPPER")
            .env_remove("RUSTC_WORKSPACE_WRAPPER")
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
