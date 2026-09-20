use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

use super::PlaygroundError;
use super::counts::{BenchCounts, parse_counts};
use super::identity::{HostIdentity, host_identity, repository_root};
use crate::upstream;

const RUST_ENGINE: &str = "native_rust";
const CPP_ENGINE: &str = "pinned_cpp";

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct BenchSample {
    engine: String,
    particles: usize,
    warmup_steps: u32,
    measured_steps: u32,
    wall_ms: f64,
    ms_per_step: f64,
    steps_per_s: f64,
    realtime_factor: f64,
    compiler: String,
}

pub(super) fn run(args: &[String]) -> Result<(), PlaygroundError> {
    let counts = parse_counts(args)?;
    let repository_root = repository_root()?;
    configure_and_build_cpp()?;
    let rust_sample = run_rust_bench(&repository_root, &counts)?;
    let cpp_sample = run_cpp_bench(&repository_root, &counts)?;
    validate_sample(&rust_sample, RUST_ENGINE, &counts)?;
    validate_sample(&cpp_sample, CPP_ENGINE, &counts)?;
    let identity = host_identity(&repository_root);
    print!("{}", render_markdown(&identity, &rust_sample, &cpp_sample));
    Ok(())
}

fn configure_and_build_cpp() -> Result<(), PlaygroundError> {
    upstream::run(&[
        String::from("configure"),
        String::from("--preset"),
        String::from("oracle-release"),
    ])?;
    upstream::run(&[
        String::from("build"),
        String::from("--preset"),
        String::from("oracle-release"),
        String::from("--target"),
        String::from("playground-dam-break-bench"),
    ])?;
    Ok(())
}

fn run_rust_bench(root: &Path, counts: &BenchCounts) -> Result<BenchSample, PlaygroundError> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| String::from("cargo"));
    let output = Command::new(cargo)
        .current_dir(root)
        .args([
            "run",
            "-p",
            "liquidfun-wasm",
            "--release",
            "--quiet",
            "--bin",
            "dam-break-bench",
            "--",
            "--warmup",
            &counts.warmup_steps.to_string(),
            "--steps",
            &counts.measured_steps.to_string(),
        ])
        .output()
        .map_err(|error| PlaygroundError::new("rust-bench", error.to_string()))?;
    sample_from_command("native Rust", &output)
}

fn run_cpp_bench(root: &Path, counts: &BenchCounts) -> Result<BenchSample, PlaygroundError> {
    let binary = cpp_binary(root);
    let output = Command::new(&binary)
        .current_dir(root)
        .args([
            "--warmup",
            &counts.warmup_steps.to_string(),
            "--steps",
            &counts.measured_steps.to_string(),
        ])
        .output()
        .map_err(|error| {
            PlaygroundError::new("cpp-bench", format!("{}: {error}", binary.display()))
        })?;
    sample_from_command("pinned C++", &output)
}

fn cpp_binary(root: &Path) -> PathBuf {
    let name = if cfg!(windows) {
        "playground-dam-break-bench.exe"
    } else {
        "playground-dam-break-bench"
    };
    root.join("target/reference/oracle-release").join(name)
}

fn sample_from_command(
    label: &str,
    output: &std::process::Output,
) -> Result<BenchSample, PlaygroundError> {
    if !output.status.success() {
        return Err(PlaygroundError::new(
            "bench",
            format!(
                "{label} bench failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    let stdout = String::from_utf8(output.stdout.clone()).map_err(|error| {
        PlaygroundError::new(
            "bench",
            format!("{label} bench stdout is not UTF-8: {error}"),
        )
    })?;
    parse_bench_json(&stdout).map_err(|message| PlaygroundError::new("bench", message))
}

fn parse_bench_json(stdout: &str) -> Result<BenchSample, String> {
    let json = extract_json_object(stdout)
        .ok_or_else(|| String::from("bench stdout did not contain a JSON object"))?;
    serde_json::from_str(json).map_err(|error| format!("failed to parse bench JSON: {error}"))
}

fn extract_json_object(stdout: &str) -> Option<&str> {
    let start = stdout.find('{')?;
    let end = stdout.rfind('}')?;
    if end < start {
        return None;
    }
    Some(&stdout[start..=end])
}

fn validate_sample(
    sample: &BenchSample,
    engine: &str,
    counts: &BenchCounts,
) -> Result<(), PlaygroundError> {
    if sample.engine != engine {
        return Err(PlaygroundError::new(
            "bench",
            format!("expected engine `{engine}`, got `{}`", sample.engine),
        ));
    }
    if sample.warmup_steps != counts.warmup_steps || sample.measured_steps != counts.measured_steps
    {
        return Err(PlaygroundError::new(
            "bench",
            format!(
                "engine `{engine}` reported warmup {} / steps {}, expected {} / {}",
                sample.warmup_steps,
                sample.measured_steps,
                counts.warmup_steps,
                counts.measured_steps
            ),
        ));
    }
    if sample.particles != 1920 {
        return Err(PlaygroundError::new(
            "bench",
            format!(
                "engine `{engine}` reported {} particles, expected 1920",
                sample.particles
            ),
        ));
    }
    Ok(())
}

#[allow(dead_code)]
fn rust_over_cpp_ratio(_rust_wall_ms: f64, _cpp_wall_ms: f64) -> Result<f64, PlaygroundError> {
    Ok(0.0)
}

#[allow(dead_code)]
fn pair_report_json(
    _stamp: &str,
    _identity: &HostIdentity,
    _rust: &BenchSample,
    _cpp: &BenchSample,
    _ratio: f64,
) -> serde_json::Value {
    serde_json::json!({ "samply": true })
}

fn render_markdown(
    identity: &HostIdentity,
    rust_sample: &BenchSample,
    cpp_sample: &BenchSample,
) -> String {
    format!(
        concat!(
            "Unreviewed local playground Dam Break sample. Not a public performance claim and not Phase 12 evidence.\n\n",
            "- git HEAD: `{}`\n",
            "- OS/arch: `{}` / `{}`\n",
            "- CPU: `{}`\n",
            "- logical cores: `{}`\n",
            "- warmup steps: `{}`\n",
            "- measured steps: `{}`\n\n",
            "| Engine | Particles | Wall ms | ms/step | steps/s | Realtime factor | Compiler |\n",
            "| ------ | --------- | ------- | ------- | ------- | --------------- | -------- |\n",
            "{}\n",
            "{}\n"
        ),
        identity.git_head,
        identity.os,
        identity.arch,
        identity.cpu_brand,
        identity.logical_cores,
        rust_sample.warmup_steps,
        rust_sample.measured_steps,
        markdown_row(rust_sample, "native Rust"),
        markdown_row(cpp_sample, "pinned C++")
    )
}

fn markdown_row(sample: &BenchSample, label: &str) -> String {
    format!(
        "| {} | {} | {:.3} | {:.3} | {:.3} | {:.3} | `{}` |",
        label,
        sample.particles,
        sample.wall_ms,
        sample.ms_per_step,
        sample.steps_per_s,
        sample.realtime_factor,
        sample.compiler.replace('|', "\\|")
    )
}

#[cfg(test)]
mod tests {
    use super::super::identity::HostIdentity;
    use super::{
        BenchSample, extract_json_object, pair_report_json, parse_bench_json, render_markdown,
        rust_over_cpp_ratio,
    };

    fn sample(engine: &str, compiler: &str, wall_ms: f64) -> BenchSample {
        let ms_per_step = wall_ms / 600.0;
        BenchSample {
            engine: engine.to_owned(),
            particles: 1920,
            warmup_steps: 60,
            measured_steps: 600,
            wall_ms,
            ms_per_step,
            steps_per_s: 1000.0 / ms_per_step,
            realtime_factor: (10.0) / (wall_ms / 1000.0),
            compiler: compiler.to_owned(),
        }
    }

    #[test]
    fn extract_json_object_skips_leading_noise() {
        // Arrange
        let stdout = "note: ok\n{\"engine\":\"native_rust\"}\n";

        // Act
        let json = extract_json_object(stdout);

        // Assert
        assert_eq!(json, Some("{\"engine\":\"native_rust\"}"));
    }

    #[test]
    fn parse_bench_json_reads_required_fields() {
        // Arrange
        let stdout = r#"{"engine":"native_rust","particles":1920,"warmup_steps":60,"measured_steps":600,"wall_ms":12000.5,"ms_per_step":20.000833,"steps_per_s":49.9979,"realtime_factor":0.8333,"compiler":"rustc 1.97.0"}"#;

        // Act
        let sample = parse_bench_json(stdout).expect("JSON should parse");

        // Assert
        assert_eq!(sample.engine, "native_rust");
        assert_eq!(sample.particles, 1920);
        assert_eq!(sample.compiler, "rustc 1.97.0");
    }

    #[test]
    fn render_markdown_includes_unreviewed_banner_and_both_rows() {
        // Arrange
        let identity = HostIdentity {
            git_head: String::from("abc123"),
            os: String::from("macos"),
            arch: String::from("aarch64"),
            cpu_brand: String::from("Apple M-series"),
            logical_cores: 8,
        };
        let rust_sample = sample("native_rust", "rustc 1.97.0", 31_200.0);
        let cpp_sample = sample("pinned_cpp", "AppleClang 17.0.0", 6_000.0);

        // Act
        let markdown = render_markdown(&identity, &rust_sample, &cpp_sample);

        // Assert
        assert!(markdown.contains("Unreviewed local playground Dam Break sample"));
        assert!(markdown.contains("Not a public performance claim"));
        assert!(markdown.contains("`abc123`"));
        assert!(markdown.contains("native Rust"));
        assert!(markdown.contains("pinned C++"));
        assert!(markdown.contains("`rustc 1.97.0`"));
        assert!(markdown.contains("`AppleClang 17.0.0`"));
    }

    #[test]
    fn render_markdown_includes_rust_over_cpp_ratio() {
        // Arrange
        let identity = HostIdentity {
            git_head: String::from("abc123"),
            os: String::from("macos"),
            arch: String::from("aarch64"),
            cpu_brand: String::from("Apple M-series"),
            logical_cores: 8,
        };
        let rust_sample = sample("native_rust", "rustc 1.97.0", 31_200.0);
        let cpp_sample = sample("pinned_cpp", "AppleClang 17.0.0", 6_000.0);

        // Act
        let markdown = render_markdown(&identity, &rust_sample, &cpp_sample);

        // Assert
        assert!(markdown.contains("Rust/C++"));
        assert!(
            markdown.contains("5.20") || markdown.contains("5.2"),
            "markdown should include the 31200/6000 ratio, got:\n{markdown}"
        );
    }

    #[test]
    fn rust_over_cpp_ratio_divides_rust_wall_by_cpp_wall() {
        // Arrange / Act
        let ratio = rust_over_cpp_ratio(31_200.0, 6_000.0).expect("finite C++ wall should divide");

        // Assert
        assert_eq!(ratio, 31_200.0 / 6_000.0);
    }

    #[test]
    fn rust_over_cpp_ratio_rejects_zero_cpp_wall() {
        // Arrange / Act
        let result = rust_over_cpp_ratio(31_200.0, 0.0);

        // Assert
        let Err(error) = result else {
            panic!("zero C++ wall_ms must be a bench error, not Inf");
        };
        let display = error.to_string();
        assert!(
            display.contains("bench"),
            "error `{display}` should mention bench"
        );
    }

    #[test]
    fn rust_over_cpp_ratio_rejects_negative_cpp_wall() {
        // Arrange / Act
        let result = rust_over_cpp_ratio(31_200.0, -1.0);

        // Assert
        let Err(error) = result else {
            panic!("negative C++ wall_ms must be a bench error");
        };
        assert!(
            error.to_string().contains("bench"),
            "error `{error}` should mention bench"
        );
    }

    #[test]
    fn rust_over_cpp_ratio_rejects_non_finite_cpp_wall() {
        // Arrange / Act
        let nan_result = rust_over_cpp_ratio(31_200.0, f64::NAN);
        let inf_result = rust_over_cpp_ratio(31_200.0, f64::INFINITY);

        // Assert
        let Err(nan_error) = nan_result else {
            panic!("NaN C++ wall_ms must be a bench error");
        };
        let Err(inf_error) = inf_result else {
            panic!("infinite C++ wall_ms must be a bench error");
        };
        assert!(nan_error.to_string().contains("bench"));
        assert!(inf_error.to_string().contains("bench"));
    }

    fn report_fixture() -> serde_json::Value {
        let identity = HostIdentity {
            git_head: String::from("abc123"),
            os: String::from("macos"),
            arch: String::from("aarch64"),
            cpu_brand: String::from("Apple M-series"),
            logical_cores: 8,
        };
        let rust_sample = sample("native_rust", "rustc 1.97.0", 31_200.0);
        let cpp_sample = sample("pinned_cpp", "AppleClang 17.0.0", 6_000.0);
        pair_report_json(
            "2001-09-09T01-46-40Z",
            &identity,
            &rust_sample,
            &cpp_sample,
            31_200.0 / 6_000.0,
        )
    }

    #[test]
    fn pair_report_json_is_unprofiled_wall_clock_pair() {
        // Arrange / Act
        let report = report_fixture();

        // Assert
        assert_eq!(report["kind"], "unprofiled_pair");
        assert_eq!(report["timing_authority"], "unprofiled_wall_clock");
        assert_eq!(
            report["rust_over_cpp_ratio"].as_f64(),
            Some(31_200.0 / 6_000.0)
        );
        assert_eq!(report["git_head"], "abc123");
        assert_eq!(report["os"], "macos");
        assert_eq!(report["arch"], "aarch64");
        assert_eq!(report["particles"], 1920);
        assert_eq!(report["rust"]["wall_ms"].as_f64(), Some(31_200.0));
        assert_eq!(report["cpp"]["wall_ms"].as_f64(), Some(6_000.0));
        assert_eq!(
            report["rust"]["ms_per_step"].as_f64(),
            Some(31_200.0 / 600.0)
        );
        assert_eq!(report["cpp"]["ms_per_step"].as_f64(), Some(10.0));
        assert_eq!(report["rust"]["compiler"], "rustc 1.97.0");
        assert_eq!(report["cpp"]["compiler"], "AppleClang 17.0.0");
    }

    #[test]
    fn pair_report_json_omits_profile_keys() {
        // Arrange / Act
        let report = report_fixture();

        // Assert
        for forbidden in [
            "samply",
            "not_timing_authority",
            "cargo_profile",
            "rust.json.gz",
        ] {
            assert!(
                !json_contains_key(&report, forbidden),
                "pair.json must not contain `{forbidden}`"
            );
        }
    }

    fn json_contains_key(value: &serde_json::Value, key: &str) -> bool {
        match value {
            serde_json::Value::Object(map) => {
                map.contains_key(key) || map.values().any(|nested| json_contains_key(nested, key))
            }
            serde_json::Value::Array(items) => {
                items.iter().any(|nested| json_contains_key(nested, key))
            }
            _ => false,
        }
    }
}
