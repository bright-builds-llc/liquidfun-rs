//! Persist the native all-catalog playground scene survey into a new evidence stamp.

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

mod survey_table;

use survey_table::{RankedScene, catalog_scene_ids, rank_scene_samples, render_ranked_table};

const SPOT_FILE_NAME: &str = "scene-spot.json";
const REQUIRED_KIND: &str = "native_scene_spot";
const DEFAULT_WARMUP_STEPS: u32 = 60;
const DEFAULT_MEASURED_STEPS: u32 = 120;
const DEFAULT_RUNS: u32 = 3;
/// Compile-time catalog copy; the CLI fixture repository has no `web/` tree.
const CATALOG_SCENES_TS: &str = include_str!("../../../../web/src/catalog/scenes.ts");
const DISCLAIMER: &str = "Unreviewed local survey of every playground catalog scene. Not a C++ pair, not Phase 12, and not the Dam Break 3× number.";
const INTERACTION_LABELS: [&str; 2] = ["default", "scripted"];
const SPREAD_FIELDS: [&str; 3] = ["median_ms_per_step", "min_ms_per_step", "max_ms_per_step"];

struct SpotCounts {
    warmup_steps: u32,
    measured_steps: u32,
    runs: u32,
}

pub(super) fn run(args: &[String]) -> Result<(), PlaygroundError> {
    let counts = parse_spot_counts(args)?;
    let repository_root = repository_root()?;
    let output = run_spot_bin(&repository_root, &counts)?;
    let scenes = parse_scene_samples(&output)?;
    let catalog = catalog_scene_ids(CATALOG_SCENES_TS)?;
    validate_scene_samples(&scenes, &catalog)?;
    let ranked = rank_scene_samples(&scenes)?;
    let report = assemble_report(&scenes, &counts, &repository_root)?;
    let unix_seconds = stamp_unix_seconds()?;
    let stamp_dir = stamp::mint_exclusive_stamp(&repository_root, unix_seconds)?;
    write_spot_json(&stamp_dir, &report)?;
    let summary = render_summary(&stamp_dir, &report, &ranked);
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
            "--runs",
            &counts.runs.to_string(),
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

fn validate_scene_samples(
    scenes: &[serde_json::Value],
    catalog: &[&str],
) -> Result<(), PlaygroundError> {
    let names: Vec<&str> = scenes
        .iter()
        .filter_map(|scene| scene.get("scene").and_then(serde_json::Value::as_str))
        .collect();
    let unique: BTreeSet<&str> = names.iter().copied().collect();
    if names.len() != scenes.len() || unique.len() != names.len() {
        return Err(PlaygroundError::new(
            "spot",
            format!("survey scenes must be named once each, got {names:?}"),
        ));
    }
    let expected: BTreeSet<&str> = catalog.iter().copied().collect();
    if unique != expected {
        let missing: Vec<&str> = expected.difference(&unique).copied().collect();
        let unexpected: Vec<&str> = unique.difference(&expected).copied().collect();
        return Err(PlaygroundError::new(
            "spot",
            format!(
                "survey scenes must match web/src/catalog/scenes.ts: missing {missing:?}, unexpected {unexpected:?}"
            ),
        ));
    }
    scenes.iter().try_for_each(validate_scene_sample)
}

fn validate_scene_sample(scene: &serde_json::Value) -> Result<(), PlaygroundError> {
    if scene.get("timed_out") != Some(&serde_json::Value::Bool(false)) {
        return Err(PlaygroundError::new(
            "spot",
            "survey scenes require timed_out false",
        ));
    }
    for field in SPREAD_FIELDS {
        let maybe_value = scene.get(field).and_then(serde_json::Value::as_f64);
        if !maybe_value.is_some_and(f64::is_finite) {
            return Err(PlaygroundError::new(
                "spot",
                format!("survey scenes require finite {field}"),
            ));
        }
    }
    let maybe_interaction = scene.get("interaction").and_then(serde_json::Value::as_str);
    if !maybe_interaction.is_some_and(|label| INTERACTION_LABELS.contains(&label)) {
        return Err(PlaygroundError::new(
            "spot",
            "survey scenes require interaction `default` or `scripted`",
        ));
    }
    Ok(())
}

fn assemble_report(
    scenes: &[serde_json::Value],
    counts: &SpotCounts,
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
        "warmup_steps": counts.warmup_steps,
        "measured_steps": counts.measured_steps,
        "runs": counts.runs,
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

fn render_summary(stamp_dir: &Path, report: &serde_json::Value, ranked: &[RankedScene]) -> String {
    let stamp_name = stamp_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown");
    let scene_count = report
        .get("scenes")
        .and_then(serde_json::Value::as_array)
        .map_or(0, Vec::len);
    let count_field = |field: &str| {
        report
            .get(field)
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_default()
    };
    format!(
        concat!(
            "Native playground scene survey (`not_timing_authority`).\n\n",
            "{disclaimer}\n\n",
            "{table}\n",
            "- stamp: `{stamp}`\n",
            "- scenes: `{count}`\n",
            "- warmup steps: `{warmup}`\n",
            "- measured steps: `{steps}`\n",
            "- runs: `{runs}`\n",
            "- artifact: `{file}`\n"
        ),
        disclaimer = DISCLAIMER,
        table = render_ranked_table(ranked),
        stamp = stamp_name,
        count = scene_count,
        warmup = count_field("warmup_steps"),
        steps = count_field("measured_steps"),
        runs = count_field("runs"),
        file = SPOT_FILE_NAME,
    )
}

fn parse_spot_counts(args: &[String]) -> Result<SpotCounts, PlaygroundError> {
    let mut warmup_steps = DEFAULT_WARMUP_STEPS;
    let mut measured_steps = DEFAULT_MEASURED_STEPS;
    let mut runs = DEFAULT_RUNS;
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
            "--runs" => {
                runs = parse_u32_flag(args, index, "--runs")?;
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
    if runs == 0 {
        return Err(PlaygroundError::usage("--runs must be greater than 0"));
    }
    Ok(SpotCounts {
        warmup_steps,
        measured_steps,
        runs,
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
    use std::collections::BTreeSet;

    use super::{
        CATALOG_SCENES_TS, catalog_scene_ids, extract_json_object, parse_spot_counts,
        validate_scene_samples,
    };
    use crate::playground::PlaygroundError;

    const CATALOG: [&str; 3] = ["a", "b", "c"];

    fn valid_sample(scene: &str) -> serde_json::Value {
        serde_json::json!({
            "scene": scene,
            "interaction": "default",
            "median_ms_per_step": 1.0,
            "min_ms_per_step": 0.9,
            "max_ms_per_step": 1.1,
            "start_particles": 1,
            "end_particles": 2,
            "timed_out": false,
        })
    }

    fn full_catalog_samples() -> Vec<serde_json::Value> {
        CATALOG.iter().map(|scene| valid_sample(scene)).collect()
    }

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
    fn embedded_catalog_has_unique_ids_including_dam_break() {
        // Arrange
        let source = CATALOG_SCENES_TS;

        // Act
        let ids = catalog_scene_ids(source).expect("embedded catalog should parse");

        // Assert
        let unique: BTreeSet<&str> = ids.iter().copied().collect();
        assert_eq!(unique.len(), ids.len());
        assert!(ids.contains(&"dam-break"));
    }

    #[test]
    fn validate_scene_samples_accepts_full_catalog() {
        // Arrange
        let samples = full_catalog_samples();

        // Act
        let result = validate_scene_samples(&samples, &CATALOG);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn validate_scene_samples_rejects_missing_scene() {
        // Arrange
        let mut samples = full_catalog_samples();
        samples.pop();

        // Act
        let result = validate_scene_samples(&samples, &CATALOG);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn validate_scene_samples_rejects_duplicate_scene() {
        // Arrange
        let samples = vec![valid_sample("a"), valid_sample("a"), valid_sample("b")];

        // Act
        let result = validate_scene_samples(&samples, &CATALOG);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn validate_scene_samples_rejects_timed_out() {
        // Arrange
        let mut samples = full_catalog_samples();
        samples[0]["timed_out"] = serde_json::Value::Bool(true);

        // Act
        let result = validate_scene_samples(&samples, &CATALOG);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn validate_scene_samples_rejects_missing_median() {
        // Arrange
        let mut samples = full_catalog_samples();
        samples[1]
            .as_object_mut()
            .expect("sample object")
            .remove("median_ms_per_step");

        // Act
        let result = validate_scene_samples(&samples, &CATALOG);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn parse_spot_counts_rejects_zero_runs() {
        // Arrange
        let args = ["--runs".to_owned(), "0".to_owned()];

        // Act
        let result = parse_spot_counts(&args).map(|counts| counts.runs);

        // Assert
        assert_eq!(
            result,
            Err(PlaygroundError::usage("--runs must be greater than 0"))
        );
    }

    #[test]
    fn parse_spot_counts_defaults_to_three_runs() {
        // Arrange
        let args: [String; 0] = [];

        // Act
        let result = parse_spot_counts(&args).map(|counts| counts.runs);

        // Assert
        assert_eq!(result, Ok(3));
    }
}
