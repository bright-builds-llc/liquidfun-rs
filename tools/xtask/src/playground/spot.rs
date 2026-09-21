//! Persist native five-scene spot-checks into a new evidence stamp.

use std::collections::BTreeSet;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use super::PlaygroundError;
use super::identity::{host_identity, repository_root};
use super::stamp;

const SPOT_FILE_NAME: &str = "scene-spot.json";
const REQUIRED_KIND: &str = "native_scene_spot";
const DEFAULT_WARMUP_STEPS: u32 = 60;
const DEFAULT_MEASURED_STEPS: u32 = 120;
const REQUIRED_SCENES: [&str; 5] = [
    "fountain",
    "float-or-sink",
    "color-mixer",
    "jelly-drop",
    "water-wheel",
];
const DISCLAIMER: &str =
    "Unreviewed local sample. Not a C++ pair, not Phase 12, and not the Dam Break 3× number.";

struct SpotCounts {
    warmup_steps: u32,
    measured_steps: u32,
}

pub(super) fn run(args: &[String]) -> Result<(), PlaygroundError> {
    let counts = parse_spot_counts(args)?;
    let repository_root = repository_root()?;
    let output = run_spot_bin(&repository_root, &counts)?;
    let scenes = parse_scene_samples(&output)?;
    validate_scene_samples(&scenes)?;
    let report = assemble_report(&scenes, &repository_root)?;
    let unix_seconds = stamp_unix_seconds()?;
    let stamp_dir = stamp::mint_exclusive_stamp(&repository_root, unix_seconds)?;
    write_spot_json(&stamp_dir, &report)?;
    let summary = render_summary(&stamp_dir, &report);
    print!("{summary}");
    eprintln!("wrote {}", stamp_dir.join(SPOT_FILE_NAME).display());
    Ok(())
}

fn cargo_program() -> OsString {
    env::var_os("LIQUIDFUN_XTASK_CARGO")
        .or_else(|| env::var_os("CARGO"))
        .unwrap_or_else(|| OsString::from("cargo"))
}

fn run_spot_bin(root: &Path, counts: &SpotCounts) -> Result<Output, PlaygroundError> {
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
            "playground-scene-spot",
            "--",
            "--warmup",
            &counts.warmup_steps.to_string(),
            "--steps",
            &counts.measured_steps.to_string(),
        ])
        .output()
        .map_err(|error| {
            PlaygroundError::new(
                "spot",
                format!("{}: {error}", PathBuf::from(&cargo).display()),
            )
        })
}

fn parse_scene_samples(output: &Output) -> Result<Vec<serde_json::Value>, PlaygroundError> {
    if !output.status.success() {
        return Err(PlaygroundError::new(
            "spot",
            format!(
                "playground-scene-spot failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    let stdout = String::from_utf8(output.stdout.clone()).map_err(|error| {
        PlaygroundError::new(
            "spot",
            format!("playground-scene-spot stdout is not UTF-8: {error}"),
        )
    })?;
    let mut samples = Vec::new();
    for line in stdout.lines() {
        let Some(json) = extract_json_object(line) else {
            continue;
        };
        let value: serde_json::Value = serde_json::from_str(json).map_err(|error| {
            PlaygroundError::new("spot", format!("failed to parse scene JSON: {error}"))
        })?;
        if value
            .get("scene")
            .and_then(serde_json::Value::as_str)
            .is_some()
        {
            samples.push(value);
        }
    }
    if samples.is_empty() {
        return Err(PlaygroundError::new(
            "spot",
            "playground-scene-spot stdout did not contain scene JSON objects",
        ));
    }
    Ok(samples)
}

fn extract_json_object(stdout: &str) -> Option<&str> {
    let start = stdout.find('{')?;
    let end = stdout.rfind('}')?;
    if end < start {
        return None;
    }
    Some(&stdout[start..=end])
}

fn validate_scene_samples(scenes: &[serde_json::Value]) -> Result<(), PlaygroundError> {
    let names: BTreeSet<&str> = scenes
        .iter()
        .filter_map(|scene| scene.get("scene").and_then(serde_json::Value::as_str))
        .collect();
    let required: BTreeSet<&str> = REQUIRED_SCENES.into_iter().collect();
    if names != required {
        return Err(PlaygroundError::new(
            "spot",
            format!("expected scenes {required:?}, got {names:?}"),
        ));
    }
    for scene in scenes {
        if scene.get("timed_out") != Some(&serde_json::Value::Bool(false)) {
            return Err(PlaygroundError::new(
                "spot",
                "spot-check scenes require timed_out false",
            ));
        }
        let maybe_wall = scene.get("wall_ms").and_then(serde_json::Value::as_f64);
        let Some(wall_ms) = maybe_wall else {
            return Err(PlaygroundError::new(
                "spot",
                "spot-check scenes require finite wall_ms",
            ));
        };
        if !wall_ms.is_finite() {
            return Err(PlaygroundError::new(
                "spot",
                "spot-check scenes require finite wall_ms",
            ));
        }
    }
    Ok(())
}

fn assemble_report(
    scenes: &[serde_json::Value],
    repository_root: &Path,
) -> Result<serde_json::Value, PlaygroundError> {
    let identity = host_identity(repository_root);
    Ok(serde_json::json!({
        "kind": REQUIRED_KIND,
        "not_timing_authority": true,
        "disclaimer": DISCLAIMER,
        "git_head": identity.git_head,
        "os": identity.os,
        "arch": identity.arch,
        "cpu_brand": identity.cpu_brand,
        "logical_cores": identity.logical_cores,
        "compiler": rustc_version()?,
        "scenes": scenes,
    }))
}

fn write_spot_json(stamp_dir: &Path, report: &serde_json::Value) -> Result<(), PlaygroundError> {
    if report.get("rust_over_cpp_ratio").is_some() {
        return Err(PlaygroundError::new(
            "spot",
            "scene-spot.json must not contain rust_over_cpp_ratio",
        ));
    }
    let json = serde_json::to_string_pretty(report).map_err(|error| {
        PlaygroundError::new(
            "spot",
            format!("failed to serialize scene-spot.json: {error}"),
        )
    })?;
    let path = stamp_dir.join(SPOT_FILE_NAME);
    fs::write(&path, json.as_bytes()).map_err(|error| {
        PlaygroundError::new(
            "spot",
            format!("failed to write {}: {error}", path.display()),
        )
    })
}

fn render_summary(stamp_dir: &Path, report: &serde_json::Value) -> String {
    let stamp_name = stamp_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown");
    let scene_count = report
        .get("scenes")
        .and_then(serde_json::Value::as_array)
        .map_or(0, Vec::len);
    format!(
        concat!(
            "Native playground scene spot-check (`not_timing_authority`).\n\n",
            "- stamp: `{stamp}`\n",
            "- scenes: `{count}`\n",
            "- artifact: `{file}`\n"
        ),
        stamp = stamp_name,
        count = scene_count,
        file = SPOT_FILE_NAME,
    )
}

fn parse_spot_counts(args: &[String]) -> Result<SpotCounts, PlaygroundError> {
    let mut warmup_steps = DEFAULT_WARMUP_STEPS;
    let mut measured_steps = DEFAULT_MEASURED_STEPS;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--warmup" => {
                warmup_steps = parse_u32_flag(args, index, "--warmup")?;
                index += 2;
            }
            "--steps" => {
                measured_steps = parse_u32_flag(args, index, "--steps")?;
                index += 2;
            }
            unknown => {
                return Err(PlaygroundError::usage(format!(
                    "unknown argument `{unknown}`"
                )));
            }
        }
    }
    if measured_steps == 0 {
        return Err(PlaygroundError::usage("--steps must be greater than 0"));
    }
    Ok(SpotCounts {
        warmup_steps,
        measured_steps,
    })
}

fn parse_u32_flag(args: &[String], index: usize, flag: &str) -> Result<u32, PlaygroundError> {
    let Some(raw) = args.get(index + 1) else {
        return Err(PlaygroundError::usage(format!(
            "{flag} requires a non-negative integer"
        )));
    };
    raw.parse::<u32>()
        .map_err(|_error| PlaygroundError::usage(format!("{flag} requires a non-negative integer")))
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
    use super::{REQUIRED_KIND, extract_json_object, validate_scene_samples};

    #[test]
    fn extract_json_object_skips_leading_noise() {
        // Arrange
        let line = "note: ok {\"scene\":\"fountain\"}";

        // Act
        let json = extract_json_object(line);

        // Assert
        assert_eq!(json, Some("{\"scene\":\"fountain\"}"));
    }

    #[test]
    fn validate_scene_samples_requires_five_named_scenes() {
        // Arrange
        let ok = vec![
            serde_json::json!({"scene":"fountain","wall_ms":1.0,"timed_out":false}),
            serde_json::json!({"scene":"float-or-sink","wall_ms":1.0,"timed_out":false}),
            serde_json::json!({"scene":"color-mixer","wall_ms":1.0,"timed_out":false}),
            serde_json::json!({"scene":"jelly-drop","wall_ms":1.0,"timed_out":false}),
            serde_json::json!({"scene":"water-wheel","wall_ms":1.0,"timed_out":false}),
        ];
        let missing = vec![ok[0].clone()];

        // Act / Assert
        assert!(validate_scene_samples(&ok).is_ok());
        assert!(validate_scene_samples(&missing).is_err());
        assert_eq!(REQUIRED_KIND, "native_scene_spot");
    }
}
