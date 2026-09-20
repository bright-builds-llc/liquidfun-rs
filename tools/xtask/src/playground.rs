//! Exploratory playground Dam Break native-versus-oracle step timing.

use std::env;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;

use serde::Deserialize;

use crate::upstream::{self, UpstreamError};

const USAGE: &str = "Usage: cargo xtask playground dam-break-bench [--warmup <n>] [--steps <n>]";
const DEFAULT_WARMUP_STEPS: u32 = 60;
const DEFAULT_MEASURED_STEPS: u32 = 600;
const RUST_ENGINE: &str = "native_rust";
const CPP_ENGINE: &str = "pinned_cpp";

/// Closed error returned by exploratory playground timing commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlaygroundError {
    kind: &'static str,
    message: String,
}

impl PlaygroundError {
    fn new(kind: &'static str, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    fn usage(message: impl Into<String>) -> Self {
        Self::new("usage", format!("{}\n\n{USAGE}", message.into()))
    }
}

impl Display for PlaygroundError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "playground/{}: {}", self.kind, self.message)
    }
}

impl Error for PlaygroundError {}

impl From<UpstreamError> for PlaygroundError {
    fn from(error: UpstreamError) -> Self {
        Self::new("upstream", error.to_string())
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct HostIdentity {
    git_head: String,
    os: String,
    arch: String,
    cpu_brand: String,
    logical_cores: usize,
}

struct BenchCounts {
    warmup_steps: u32,
    measured_steps: u32,
}

/// Runs the exploratory playground Dam Break native-versus-oracle pair.
///
/// # Errors
///
/// Returns a closed error when arguments, the oracle build, either bench, or
/// host identity collection fails.
pub(crate) fn run(args: &[String]) -> Result<(), PlaygroundError> {
    let (command, command_args) = args
        .split_first()
        .ok_or_else(|| PlaygroundError::usage("expected `dam-break-bench`"))?;
    if command != "dam-break-bench" {
        return Err(PlaygroundError::usage(format!(
            "unknown playground command `{command}`"
        )));
    }
    let counts = parse_counts(command_args)?;
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

fn parse_counts(args: &[String]) -> Result<BenchCounts, PlaygroundError> {
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
    Ok(BenchCounts {
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

fn host_identity(root: &Path) -> HostIdentity {
    HostIdentity {
        git_head: git_head(root),
        os: String::from(env::consts::OS),
        arch: String::from(env::consts::ARCH),
        cpu_brand: cpu_brand(),
        logical_cores: thread::available_parallelism().map_or(1, usize::from),
    }
}

fn git_head(root: &Path) -> String {
    let output = Command::new("git")
        .current_dir(root)
        .args(["rev-parse", "HEAD"])
        .output();
    let Ok(output) = output else {
        return String::from("unknown");
    };
    if !output.status.success() {
        return String::from("unknown");
    }
    String::from_utf8(output.stdout)
        .ok()
        .and_then(|text| {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_owned())
            }
        })
        .unwrap_or_else(|| String::from("unknown"))
}

fn cpu_brand() -> String {
    if cfg!(target_os = "macos") {
        return first_command_line("sysctl", &["-n", "machdep.cpu.brand_string"]);
    }
    if cfg!(target_os = "linux") {
        return linux_cpu_brand();
    }
    String::from("unknown")
}

fn first_command_line(program: &str, args: &[&str]) -> String {
    let output = Command::new(program).args(args).output();
    let Ok(output) = output else {
        return String::from("unknown");
    };
    if !output.status.success() {
        return String::from("unknown");
    }
    String::from_utf8(output.stdout)
        .ok()
        .and_then(|text| {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_owned())
            }
        })
        .unwrap_or_else(|| String::from("unknown"))
}

fn linux_cpu_brand() -> String {
    let Ok(contents) = std::fs::read_to_string("/proc/cpuinfo") else {
        return String::from("unknown");
    };
    for line in contents.lines() {
        let Some(value) = line.strip_prefix("model name") else {
            continue;
        };
        let Some(brand) = value.split(':').nth(1) else {
            continue;
        };
        let trimmed = brand.trim();
        if !trimmed.is_empty() {
            return trimmed.to_owned();
        }
    }
    String::from("unknown")
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

fn repository_root() -> Result<PathBuf, PlaygroundError> {
    let current_dir = env::current_dir().map_err(|error| {
        PlaygroundError::new(
            "repository",
            format!("failed to read current directory: {error}"),
        )
    })?;
    let Some(root) = current_dir.ancestors().find(|candidate| {
        candidate.join("Cargo.toml").is_file()
            && candidate.join("crates/liquidfun/Cargo.toml").is_file()
    }) else {
        return Err(PlaygroundError::new(
            "repository",
            "could not find the liquidfun Cargo workspace",
        ));
    };
    Ok(root.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::{
        BenchSample, HostIdentity, PlaygroundError, extract_json_object, parse_bench_json,
        parse_counts, render_markdown, run,
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
    fn missing_subcommand_is_a_usage_error() {
        // Arrange / Act
        let result = run(&[]);

        // Assert
        assert_eq!(
            result,
            Err(PlaygroundError::usage("expected `dam-break-bench`"))
        );
    }

    #[test]
    fn unknown_subcommand_is_a_usage_error() {
        // Arrange / Act
        let result = run(&[String::from("nope")]);

        // Assert
        assert_eq!(
            result,
            Err(PlaygroundError::usage("unknown playground command `nope`"))
        );
    }

    #[test]
    fn parse_counts_defaults_to_locked_warmup_and_steps() {
        // Arrange / Act
        let counts = parse_counts(&[]).expect("defaults should parse");

        // Assert
        assert_eq!(counts.warmup_steps, 60);
        assert_eq!(counts.measured_steps, 600);
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
}
