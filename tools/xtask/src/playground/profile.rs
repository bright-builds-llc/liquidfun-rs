//! Pure samply 0.13.1 version check and argv builder for playground CPU profiles.
//!
//! Production process spawn belongs to 22-04. This module is unused by the
//! unprofiled `dam-break-bench` pair.

#![allow(dead_code)]

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use super::PlaygroundError;

const PINNED_SAMPLY_VERSION: &str = "0.13.1";
const MISSING_SAMPLY_MESSAGE: &str = "samply 0.13.1 is required. Install with `cargo install --locked samply --version 0.13.1` or `brew install samply`. On macOS, run `samply setup` after install.";

fn parse_samply_version(stdout: &str) -> Option<String> {
    let maybe_line = stdout.lines().map(str::trim).find(|line| !line.is_empty());
    let line = maybe_line?;
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if !tokens.contains(&"samply") {
        return None;
    }
    let maybe_version = tokens
        .into_iter()
        .find(|token| *token == PINNED_SAMPLY_VERSION);
    maybe_version.map(str::to_owned)
}

fn missing_samply_error() -> PlaygroundError {
    PlaygroundError::new("samply", MISSING_SAMPLY_MESSAGE)
}

fn require_samply_version(stdout: &str) -> Result<(), PlaygroundError> {
    let maybe_version = parse_samply_version(stdout);
    let Some(_version) = maybe_version else {
        return Err(missing_samply_error());
    };
    Ok(())
}

fn require_samply_0_13_1(stdout: &str) -> Result<(), PlaygroundError> {
    require_samply_version(stdout)
}

fn samply_record_argv(
    output_gz: &Path,
    binary: &Path,
    warmup_steps: u32,
    measured_steps: u32,
) -> Vec<OsString> {
    vec![
        OsString::from("record"),
        OsString::from("--save-only"),
        OsString::from("--unstable-presymbolicate"),
        OsString::from("-o"),
        output_gz.as_os_str().to_os_string(),
        OsString::from("--"),
        binary.as_os_str().to_os_string(),
        OsString::from("--warmup"),
        OsString::from(warmup_steps.to_string()),
        OsString::from("--steps"),
        OsString::from(measured_steps.to_string()),
    ]
}

fn profiling_dam_break_bench_bin(repository_root: &Path) -> PathBuf {
    let file_name = if cfg!(windows) {
        "dam-break-bench.exe"
    } else {
        "dam-break-bench"
    };
    repository_root.join("target/profiling").join(file_name)
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
