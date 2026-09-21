//! Pure samply 0.13.1 version check and argv builder for playground CPU profiles.
//!
//! Production process spawn belongs to 22-04. This module is unused by the
//! unprofiled `dam-break-bench` pair.

#![allow(dead_code)]

use std::ffi::OsString;
use std::path::Path;

use super::PlaygroundError;

fn parse_samply_version(_stdout: &str) -> Option<String> {
    None
}

fn missing_samply_error() -> PlaygroundError {
    PlaygroundError::new("samply", "skip")
}

fn require_samply_version(_stdout: &str) -> Result<(), PlaygroundError> {
    Ok(())
}

fn samply_record_argv(
    _output_gz: &Path,
    _binary: &Path,
    _warmup_steps: u32,
    _measured_steps: u32,
) -> Vec<OsString> {
    vec![OsString::from("cargo")]
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::Path;

    use super::{
        missing_samply_error, parse_samply_version, require_samply_version, samply_record_argv,
    };

    fn assert_fail_closed_install_text(display: &str) {
        assert!(
            display.contains("0.13.1"),
            "error `{display}` should pin samply 0.13.1"
        );
        assert!(
            display.contains("cargo install --locked samply --version 0.13.1"),
            "error `{display}` should include cargo install --locked"
        );
        assert!(
            display.contains("brew install samply"),
            "error `{display}` should include brew install samply"
        );
        assert!(
            display.contains("samply setup"),
            "error `{display}` should mention samply setup on macOS"
        );
        assert!(
            !display.to_ascii_lowercase().contains("skip"),
            "error `{display}` must not mention skipping"
        );
    }

    #[test]
    fn parse_samply_version_reads_pinned_0_13_1() {
        // Arrange
        let stdout = "samply 0.13.1\n";

        // Act
        let maybe_version = parse_samply_version(stdout);

        // Assert
        assert_eq!(maybe_version, Some(String::from("0.13.1")));
    }

    #[test]
    fn parse_samply_version_rejects_other_version() {
        // Arrange
        let stdout = "samply 0.12.0";

        // Act
        let maybe_version = parse_samply_version(stdout);

        // Assert
        assert_eq!(maybe_version, None);
    }

    #[test]
    fn parse_samply_version_rejects_empty_stdout() {
        // Arrange
        let stdout = "";

        // Act
        let maybe_version = parse_samply_version(stdout);

        // Assert
        assert_eq!(maybe_version, None);
    }

    #[test]
    fn require_samply_version_accepts_pinned_stdout() {
        // Arrange
        let stdout = "samply 0.13.1";

        // Act
        let result = require_samply_version(stdout);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn require_samply_version_rejects_other_version_with_install_text() {
        // Arrange
        let stdout = "samply 0.13.2";

        // Act
        let result = require_samply_version(stdout);

        // Assert
        let Err(error) = result else {
            panic!("samply 0.13.2 must be a closed error, not Ok");
        };
        assert_fail_closed_install_text(&error.to_string());
    }

    #[test]
    fn missing_samply_error_includes_install_text_and_not_skip() {
        // Arrange / Act
        let display = missing_samply_error().to_string();

        // Assert
        assert_fail_closed_install_text(&display);
    }

    #[test]
    fn samply_record_argv_matches_locked_flag_sequence() {
        // Arrange
        let output = Path::new("/tmp/stamp/rust.json.gz");
        let binary = Path::new("/repo/target/profiling/dam-break-bench");

        // Act
        let argv = samply_record_argv(output, binary, 60, 600);

        // Assert
        assert_eq!(argv.first(), Some(&OsString::from("record")));
        assert!(
            !argv.iter().any(|flag| flag == "cargo"),
            "argv must wrap the built binary, not cargo"
        );
        assert_eq!(
            argv,
            [
                OsString::from("record"),
                OsString::from("--save-only"),
                OsString::from("--unstable-presymbolicate"),
                OsString::from("-o"),
                output.as_os_str().to_os_string(),
                OsString::from("--"),
                binary.as_os_str().to_os_string(),
                OsString::from("--warmup"),
                OsString::from("60"),
                OsString::from("--steps"),
                OsString::from("600"),
            ]
        );
    }
}
