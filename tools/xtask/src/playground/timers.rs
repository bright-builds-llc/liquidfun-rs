//! Persist Dam Break parent-phase timers into a new evidence stamp.

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use super::PlaygroundError;
use super::counts::{BenchCounts, parse_counts};
use super::identity::{host_identity, repository_root};
use super::stamp;

const TIMERS_FILE_NAME: &str = "timers.json";
const REQUIRED_KIND: &str = "step_profiled_parents";

pub(super) fn run(args: &[String]) -> Result<(), PlaygroundError> {
    let counts = parse_counts(args)?;
    let repository_root = repository_root()?;
    let output = run_timer_bin(&repository_root, &counts)?;
    let mut report = parse_timer_json(&output)?;
    validate_timer_report(&report)?;
    merge_host_identity(&mut report, &repository_root)?;
    let unix_seconds = stamp_unix_seconds()?;
    let stamp_dir = stamp::mint_exclusive_stamp(&repository_root, unix_seconds)?;
    write_timers_json(&stamp_dir, &report)?;
    let summary = render_summary(&stamp_dir, &report);
    print!("{summary}");
    eprintln!("wrote {}", stamp_dir.join(TIMERS_FILE_NAME).display());
    Ok(())
}

fn cargo_program() -> OsString {
    env::var_os("LIQUIDFUN_XTASK_CARGO")
        .or_else(|| env::var_os("CARGO"))
        .unwrap_or_else(|| OsString::from("cargo"))
}

fn run_timer_bin(root: &Path, counts: &BenchCounts) -> Result<Output, PlaygroundError> {
    let cargo = cargo_program();
    Command::new(&cargo)
        .current_dir(root)
        .args([
            "run",
            "-p",
            "liquidfun-wasm",
            "--release",
            "--quiet",
            "--bin",
            "dam-break-timers",
            "--",
            "--warmup",
            &counts.warmup_steps.to_string(),
            "--steps",
            &counts.measured_steps.to_string(),
        ])
        .output()
        .map_err(|error| {
            PlaygroundError::new(
                "timers",
                format!("{}: {error}", PathBuf::from(&cargo).display()),
            )
        })
}

fn parse_timer_json(output: &Output) -> Result<serde_json::Value, PlaygroundError> {
    if !output.status.success() {
        return Err(PlaygroundError::new(
            "timers",
            format!(
                "dam-break-timers failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    let stdout = String::from_utf8(output.stdout.clone()).map_err(|error| {
        PlaygroundError::new(
            "timers",
            format!("dam-break-timers stdout is not UTF-8: {error}"),
        )
    })?;
    let json = extract_json_object(&stdout).ok_or_else(|| {
        PlaygroundError::new(
            "timers",
            "dam-break-timers stdout did not contain a JSON object",
        )
    })?;
    serde_json::from_str(json).map_err(|error| {
        PlaygroundError::new("timers", format!("failed to parse timers JSON: {error}"))
    })
}

fn extract_json_object(stdout: &str) -> Option<&str> {
    let start = stdout.find('{')?;
    let end = stdout.rfind('}')?;
    if end < start {
        return None;
    }
    Some(&stdout[start..=end])
}

fn validate_timer_report(report: &serde_json::Value) -> Result<(), PlaygroundError> {
    let maybe_kind = report.get("kind").and_then(serde_json::Value::as_str);
    if maybe_kind != Some(REQUIRED_KIND) {
        return Err(PlaygroundError::new(
            "timers",
            format!("expected kind `{REQUIRED_KIND}`, got {maybe_kind:?}"),
        ));
    }
    if report.get("not_timing_authority") != Some(&serde_json::Value::Bool(true)) {
        return Err(PlaygroundError::new(
            "timers",
            "timers require not_timing_authority true",
        ));
    }
    Ok(())
}

fn merge_host_identity(
    report: &mut serde_json::Value,
    repository_root: &Path,
) -> Result<(), PlaygroundError> {
    let identity = host_identity(repository_root);
    let Some(object) = report.as_object_mut() else {
        return Err(PlaygroundError::new(
            "timers",
            "timers JSON must be an object",
        ));
    };
    if !object.contains_key("git_head") {
        object.insert(
            String::from("git_head"),
            serde_json::Value::String(identity.git_head),
        );
    }
    if !object.contains_key("os") {
        object.insert(String::from("os"), serde_json::Value::String(identity.os));
    }
    if !object.contains_key("arch") {
        object.insert(
            String::from("arch"),
            serde_json::Value::String(identity.arch),
        );
    }
    if !object.contains_key("compiler") {
        object.insert(
            String::from("compiler"),
            serde_json::Value::String(rustc_version()?),
        );
    }
    Ok(())
}

fn write_timers_json(stamp_dir: &Path, report: &serde_json::Value) -> Result<(), PlaygroundError> {
    let json = serde_json::to_string_pretty(report).map_err(|error| {
        PlaygroundError::new(
            "timers",
            format!("failed to serialize timers.json: {error}"),
        )
    })?;
    let path = stamp_dir.join(TIMERS_FILE_NAME);
    fs::write(&path, json.as_bytes()).map_err(|error| {
        PlaygroundError::new(
            "timers",
            format!("failed to write {}: {error}", path.display()),
        )
    })
}

fn render_summary(stamp_dir: &Path, report: &serde_json::Value) -> String {
    let stamp_name = stamp_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown");
    let particles = report
        .get("particles")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let warmup_steps = report
        .get("warmup_steps")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let measured_steps = report
        .get("measured_steps")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    format!(
        concat!(
            "Dam Break parent timers (`not_timing_authority`).\n\n",
            "- stamp: `{stamp}`\n",
            "- particles: `{particles}`\n",
            "- warmup steps: `{warmup}`\n",
            "- measured steps: `{measured}`\n",
            "- artifact: `{file}`\n"
        ),
        stamp = stamp_name,
        particles = particles,
        warmup = warmup_steps,
        measured = measured_steps,
        file = TIMERS_FILE_NAME,
    )
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

#[cfg(test)]
mod tests {
    use super::{extract_json_object, validate_timer_report};

    #[test]
    fn extract_json_object_skips_leading_noise() {
        // Arrange
        let stdout = "note: ok\n{\"kind\":\"step_profiled_parents\"}\n";

        // Act
        let json = extract_json_object(stdout);

        // Assert
        assert_eq!(json, Some("{\"kind\":\"step_profiled_parents\"}"));
    }

    #[test]
    fn validate_timer_report_requires_kind_and_boolean_flag() {
        // Arrange
        let ok = serde_json::json!({
            "kind": "step_profiled_parents",
            "not_timing_authority": true
        });
        let wrong_kind = serde_json::json!({
            "kind": "unprofiled_pair",
            "not_timing_authority": true
        });
        let string_flag = serde_json::json!({
            "kind": "step_profiled_parents",
            "not_timing_authority": "true"
        });

        // Act / Assert
        assert!(validate_timer_report(&ok).is_ok());
        assert!(validate_timer_report(&wrong_kind).is_err());
        assert!(validate_timer_report(&string_flag).is_err());
    }
}
