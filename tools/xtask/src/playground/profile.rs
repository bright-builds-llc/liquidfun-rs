//! Samply 0.13.1 wrap of the `[profile.profiling]` Dam Break binary.

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use super::PlaygroundError;
use super::counts::{BenchCounts, parse_counts};
use super::identity::{host_identity, repository_root};
use super::stamp;

const PINNED_SAMPLY_VERSION: &str = "0.13.1";
const MISSING_SAMPLY_MESSAGE: &str = "samply 0.13.1 is required. Install with `cargo install --locked samply --version 0.13.1` or `brew install samply`. On macOS, run `samply setup` after install.";
const PROFILE_OUTPUT_NAME: &str = "rust.json.gz";
const IDENTITY_FILE_NAME: &str = "profile-identity.json";

pub(super) fn run(args: &[String]) -> Result<(), PlaygroundError> {
    let counts = parse_counts(args)?;
    let samply_program = samply_program();
    verify_samply(&samply_program)?;

    let repository_root = repository_root()?;
    build_profiling_binary(&repository_root)?;

    let unix_seconds = stamp_unix_seconds()?;
    let stamp_dir = stamp::mint_exclusive_stamp(&repository_root, unix_seconds)?;
    let output_gz = stamp_dir.join(PROFILE_OUTPUT_NAME);
    let binary = profiling_dam_break_bench_bin(&repository_root);
    let record_argv = samply_record_argv(
        &output_gz,
        &binary,
        counts.warmup_steps,
        counts.measured_steps,
    );
    run_samply_record(&samply_program, &record_argv, &repository_root)?;
    require_recorded_profile(&output_gz)?;
    write_profile_identity(
        &stamp_dir,
        &repository_root,
        &samply_program,
        &record_argv,
        &binary,
        &counts,
    )?;
    eprintln!("wrote {}", output_gz.display());
    Ok(())
}

fn samply_program() -> OsString {
    env::var_os("LIQUIDFUN_XTASK_SAMPLY").unwrap_or_else(|| OsString::from("samply"))
}

fn cargo_program() -> OsString {
    env::var_os("LIQUIDFUN_XTASK_CARGO")
        .or_else(|| env::var_os("CARGO"))
        .unwrap_or_else(|| OsString::from("cargo"))
}

fn verify_samply(samply_program: &OsString) -> Result<(), PlaygroundError> {
    let output = Command::new(samply_program)
        .arg("--version")
        .output()
        .map_err(|_error| missing_samply_error())?;
    if !output.status.success() {
        return Err(missing_samply_error());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    require_samply_0_13_1(&stdout)
}

fn build_profiling_binary(root: &Path) -> Result<(), PlaygroundError> {
    let cargo = cargo_program();
    let output = Command::new(&cargo)
        .current_dir(root)
        .args([
            "build",
            "-p",
            "liquidfun-wasm",
            "--bin",
            "dam-break-bench",
            "--profile",
            "profiling",
        ])
        .output()
        .map_err(|error| {
            PlaygroundError::new(
                "cargo",
                format!("{}: {error}", PathBuf::from(&cargo).display()),
            )
        })?;
    if output.status.success() {
        return Ok(());
    }
    Err(PlaygroundError::new(
        "cargo",
        format!(
            "cargo build --profile profiling failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ),
    ))
}

fn run_samply_record(
    samply_program: &OsString,
    argv: &[OsString],
    root: &Path,
) -> Result<(), PlaygroundError> {
    let output = Command::new(samply_program)
        .args(argv)
        .current_dir(root)
        .output()
        .map_err(|error| PlaygroundError::new("samply", error.to_string()))?;
    if output.status.success() {
        return Ok(());
    }
    Err(PlaygroundError::new(
        "samply",
        format!(
            "samply record failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ),
    ))
}

fn require_recorded_profile(output_gz: &Path) -> Result<(), PlaygroundError> {
    let metadata = fs::metadata(output_gz).map_err(|error| {
        PlaygroundError::new(
            "samply",
            format!("samply did not write {}: {error}", output_gz.display()),
        )
    })?;
    if metadata.len() == 0 {
        return Err(PlaygroundError::new(
            "samply",
            format!("samply wrote empty {}", output_gz.display()),
        ));
    }
    Ok(())
}

fn write_profile_identity(
    stamp_dir: &Path,
    repository_root: &Path,
    samply_program: &OsString,
    record_argv: &[OsString],
    binary: &Path,
    counts: &BenchCounts,
) -> Result<(), PlaygroundError> {
    let identity = host_identity(repository_root);
    let compiler = rustc_version()?;
    let command = identity_command(samply_program, record_argv);
    let report = serde_json::json!({
        "kind": "samply_cpu",
        "not_timing_authority": true,
        "cargo_profile": "profiling",
        "samply_version": PINNED_SAMPLY_VERSION,
        "warmup_included_in_samples": true,
        "git_head": identity.git_head,
        "os": identity.os,
        "arch": identity.arch,
        "cpu_brand": identity.cpu_brand,
        "logical_cores": identity.logical_cores,
        "compiler": compiler,
        "binary": identity_binary_path(repository_root, binary),
        "output": PROFILE_OUTPUT_NAME,
        "command": command,
        "warmup_steps": counts.warmup_steps,
        "measured_steps": counts.measured_steps,
    });
    let json = serde_json::to_string_pretty(&report).map_err(|error| {
        PlaygroundError::new(
            "profile",
            format!("failed to serialize profile-identity.json: {error}"),
        )
    })?;
    let path = stamp_dir.join(IDENTITY_FILE_NAME);
    fs::write(&path, json.as_bytes()).map_err(|error| {
        PlaygroundError::new(
            "profile",
            format!("failed to write {}: {error}", path.display()),
        )
    })
}

fn identity_command(samply_program: &OsString, record_argv: &[OsString]) -> Vec<String> {
    let mut command = Vec::with_capacity(record_argv.len() + 1);
    command.push(samply_program.to_string_lossy().into_owned());
    command.extend(
        record_argv
            .iter()
            .map(|argument| argument.to_string_lossy().into_owned()),
    );
    command
}

fn identity_binary_path(repository_root: &Path, binary: &Path) -> String {
    match binary.strip_prefix(repository_root) {
        Ok(relative) => relative.display().to_string(),
        Err(_error) => binary.display().to_string(),
    }
}

fn rustc_version() -> Result<String, PlaygroundError> {
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));
    let output = Command::new(&rustc)
        .arg("--version")
        .output()
        .map_err(|error| {
            PlaygroundError::new(
                "compiler",
                format!("failed to read rustc --version: {error}"),
            )
        })?;
    if !output.status.success() {
        return Err(PlaygroundError::new(
            "compiler",
            "failed to read rustc --version",
        ));
    }
    let text = String::from_utf8(output.stdout).map_err(|error| {
        PlaygroundError::new(
            "compiler",
            format!("rustc --version stdout is not UTF-8: {error}"),
        )
    })?;
    let maybe_first_line = text.lines().next();
    let Some(first_line) = maybe_first_line else {
        return Err(PlaygroundError::new(
            "compiler",
            "rustc --version produced no output",
        ));
    };
    let trimmed = first_line.trim();
    if trimmed.is_empty() {
        return Err(PlaygroundError::new(
            "compiler",
            "rustc --version produced no output",
        ));
    }
    Ok(trimmed.to_owned())
}

fn stamp_unix_seconds() -> Result<u64, PlaygroundError> {
    match env::var("LIQUIDFUN_XTASK_STAMP_UNIX") {
        Ok(raw) => parse_stamp_unix(&raw),
        Err(env::VarError::NotPresent) => SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .map_err(|error| {
                PlaygroundError::new(
                    "stamp",
                    format!("system clock is before Unix epoch: {error}"),
                )
            }),
        Err(env::VarError::NotUnicode(_)) => Err(PlaygroundError::new(
            "stamp",
            "LIQUIDFUN_XTASK_STAMP_UNIX must be a u64 unix-seconds integer",
        )),
    }
}

fn parse_stamp_unix(raw: &str) -> Result<u64, PlaygroundError> {
    if raw.contains('/') || raw.contains('\\') || raw.contains("..") {
        return Err(PlaygroundError::new(
            "stamp",
            "LIQUIDFUN_XTASK_STAMP_UNIX must be a u64 unix-seconds integer, not a path",
        ));
    }
    raw.parse::<u64>().map_err(|_| {
        PlaygroundError::new(
            "stamp",
            "LIQUIDFUN_XTASK_STAMP_UNIX must be a u64 unix-seconds integer",
        )
    })
}

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
    profiling_dam_break_bench_bin_with_target_dir(repository_root, env::var_os("CARGO_TARGET_DIR"))
}

fn profiling_dam_break_bench_bin_with_target_dir(
    repository_root: &Path,
    maybe_target_dir: Option<OsString>,
) -> PathBuf {
    let file_name = if cfg!(windows) {
        "dam-break-bench.exe"
    } else {
        "dam-break-bench"
    };
    cargo_target_dir(repository_root, maybe_target_dir)
        .join("profiling")
        .join(file_name)
}

fn cargo_target_dir(repository_root: &Path, maybe_target_dir: Option<OsString>) -> PathBuf {
    let Some(target_dir) = maybe_target_dir else {
        return repository_root.join("target");
    };
    let path = PathBuf::from(target_dir);
    if path.is_absolute() {
        path
    } else {
        repository_root.join(path)
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::Path;

    use super::{
        missing_samply_error, parse_samply_version, profiling_dam_break_bench_bin_with_target_dir,
        require_samply_version, samply_record_argv,
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

    #[test]
    fn profiling_binary_uses_named_profile_dir_and_optional_cargo_target_dir() {
        // Arrange
        let root = Path::new("/repo");

        // Act
        let default_path = profiling_dam_break_bench_bin_with_target_dir(root, None);
        let override_path = profiling_dam_break_bench_bin_with_target_dir(
            root,
            Some(OsString::from("/tmp/custom-target")),
        );

        // Assert
        assert!(
            default_path.ends_with("target/profiling/dam-break-bench")
                || default_path.ends_with("target/profiling/dam-break-bench.exe"),
            "default path should be under target/profiling, got {}",
            default_path.display()
        );
        assert!(
            override_path.ends_with("profiling/dam-break-bench")
                || override_path.ends_with("profiling/dam-break-bench.exe"),
            "CARGO_TARGET_DIR override should be honored, got {}",
            override_path.display()
        );
        assert!(
            override_path.starts_with("/tmp/custom-target"),
            "absolute CARGO_TARGET_DIR should replace root/target, got {}",
            override_path.display()
        );
    }
}
