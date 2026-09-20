//! Native-only Dam Break Medium/Normal `World::step` timer.

use std::env;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::process::Command;
use std::time::Instant;

use crate::scene::SceneId;
use crate::session::SessionCore;

/// Locked playground Dam Break Medium water / Normal gravity particle count.
pub const EXPECTED_PARTICLE_COUNT: usize = 1920;
/// Locked playground particle radius.
#[allow(
    clippy::unreadable_literal,
    reason = "this decimal is the locked playground recipe compared to C++ source"
)]
pub const RECIPE_PARTICLE_RADIUS: f32 = 0.06324555;
/// Locked playground particle spacing.
#[allow(
    clippy::unreadable_literal,
    reason = "this decimal is the locked playground recipe compared to C++ source"
)]
pub const RECIPE_PARTICLE_SPACING: f32 = 0.101193;
/// Locked playground particle grid columns.
pub const RECIPE_PARTICLE_COLUMNS: u8 = 48;
/// Locked playground particle grid rows.
pub const RECIPE_PARTICLE_ROWS: u8 = 40;
/// Locked playground particle-grid origin x.
pub const RECIPE_ORIGIN_X: f32 = -4.7;
/// Locked playground particle-grid origin y.
pub const RECIPE_ORIGIN_Y: f32 = 0.4;
/// Locked playground dynamic-circle radius.
pub const RECIPE_CIRCLE_RADIUS: f32 = 0.75;
/// Locked playground dynamic-circle x.
pub const RECIPE_CIRCLE_X: f32 = 2.5;
/// Locked playground dynamic-circle y.
pub const RECIPE_CIRCLE_Y: f32 = 5.5;
/// Locked playground gravity y.
pub const RECIPE_GRAVITY_Y: f32 = -10.0;
/// Untimed first-contact steps before the measured sample.
pub const DEFAULT_WARMUP_STEPS: u32 = 60;
/// Timed `World::step` count. 600 steps is 10 s of simulated time at 1/60.
pub const DEFAULT_MEASURED_STEPS: u32 = 600;

const ENGINE: &str = "native_rust";
const SIMULATED_STEP_SECONDS: f64 = 1.0 / 60.0;

/// Closed error for the native Dam Break step timer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DamBreakBenchError {
    /// Scene construction failed.
    SceneConstruction,
    /// The live scene particle count is not the locked Medium recipe.
    ParticleCount {
        /// Particles present after construction.
        actual: usize,
    },
    /// Warm-up or measured `World::step` failed.
    StepFailed {
        /// Whether the failure happened during warm-up or the timed loop.
        phase: &'static str,
    },
    /// `rustc --version` could not be read.
    CompilerVersion,
}

impl Display for DamBreakBenchError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::SceneConstruction => {
                write!(
                    formatter,
                    "Dam Break Medium/Normal scene construction failed"
                )
            }
            Self::ParticleCount { actual } => write!(
                formatter,
                "Dam Break particle count was {actual}, expected {EXPECTED_PARTICLE_COUNT}"
            ),
            Self::StepFailed { phase } => {
                write!(formatter, "Dam Break {phase} World::step failed")
            }
            Self::CompilerVersion => write!(formatter, "failed to read rustc --version"),
        }
    }
}

impl Error for DamBreakBenchError {}

/// One native Dam Break `World::step` timing sample.
#[derive(Debug, Clone, PartialEq)]
pub struct DamBreakBenchReport {
    /// Engine label written into JSON.
    pub engine: &'static str,
    /// Particles present for the timed steps.
    pub particles: usize,
    /// Untimed steps taken before the sample.
    pub warmup_steps: u32,
    /// Timed `World::step` count.
    pub measured_steps: u32,
    /// Wall time of the timed loop, in milliseconds.
    pub wall_ms: f64,
    /// Mean wall milliseconds per timed step.
    pub ms_per_step: f64,
    /// Timed steps completed per wall second.
    pub steps_per_s: f64,
    /// Simulated seconds per wall second; 1.0 would keep up with 1/60.
    pub realtime_factor: f64,
    /// `rustc --version` first line.
    pub compiler: String,
}

impl DamBreakBenchReport {
    /// Renders one JSON object for the xtask driver.
    #[must_use]
    pub fn to_json(&self) -> String {
        format!(
            "{{\"engine\":\"{}\",\"particles\":{},\"warmup_steps\":{},\"measured_steps\":{},\"wall_ms\":{:.6},\"ms_per_step\":{:.6},\"steps_per_s\":{:.6},\"realtime_factor\":{:.6},\"compiler\":\"{}\"}}",
            self.engine,
            self.particles,
            self.warmup_steps,
            self.measured_steps,
            self.wall_ms,
            self.ms_per_step,
            self.steps_per_s,
            self.realtime_factor,
            json_escape(&self.compiler)
        )
    }
}

/// Builds playground Dam Break Medium/Normal and times only `World::step`.
///
/// # Errors
///
/// Returns a closed error when construction, the locked particle count, a step,
/// or `rustc --version` fails.
pub fn run_dam_break_bench(
    warmup_steps: u32,
    measured_steps: u32,
) -> Result<DamBreakBenchReport, DamBreakBenchError> {
    if measured_steps == 0 {
        return Err(DamBreakBenchError::StepFailed { phase: "measured" });
    }

    let mut session = SessionCore::create(SceneId::DamBreak)
        .map_err(|_error| DamBreakBenchError::SceneConstruction)?;
    let particles = session.particle_count();
    if particles != EXPECTED_PARTICLE_COUNT {
        return Err(DamBreakBenchError::ParticleCount { actual: particles });
    }

    for _ in 0..warmup_steps {
        session
            .advance(1)
            .map_err(|_error| DamBreakBenchError::StepFailed { phase: "warmup" })?;
    }

    let started = Instant::now();
    for _ in 0..measured_steps {
        session
            .advance(1)
            .map_err(|_error| DamBreakBenchError::StepFailed { phase: "measured" })?;
    }
    let wall_ms = duration_as_millis(started.elapsed());
    Ok(DamBreakBenchReport {
        engine: ENGINE,
        particles,
        warmup_steps,
        measured_steps,
        wall_ms,
        ms_per_step: wall_ms / f64::from(measured_steps),
        steps_per_s: f64::from(measured_steps) / (wall_ms / 1000.0),
        realtime_factor: (SIMULATED_STEP_SECONDS * f64::from(measured_steps)) / (wall_ms / 1000.0),
        compiler: rustc_version()?,
    })
}

fn duration_as_millis(elapsed: std::time::Duration) -> f64 {
    (elapsed.as_secs_f64() * 1000.0).max(f64::EPSILON)
}

fn rustc_version() -> Result<String, DamBreakBenchError> {
    let rustc = env::var("RUSTC").unwrap_or_else(|_| String::from("rustc"));
    let output = Command::new(rustc)
        .arg("--version")
        .output()
        .map_err(|_error| DamBreakBenchError::CompilerVersion)?;
    if !output.status.success() {
        return Err(DamBreakBenchError::CompilerVersion);
    }
    let text =
        String::from_utf8(output.stdout).map_err(|_error| DamBreakBenchError::CompilerVersion)?;
    let Some(first_line) = text.lines().next() else {
        return Err(DamBreakBenchError::CompilerVersion);
    };
    let trimmed = first_line.trim();
    if trimmed.is_empty() {
        return Err(DamBreakBenchError::CompilerVersion);
    }
    Ok(trimmed.to_owned())
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::{EXPECTED_PARTICLE_COUNT, json_escape, run_dam_break_bench};
    use std::fs;
    use std::path::PathBuf;

    fn cpp_bench_source() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tools/reference/src/playground_dam_break_bench.cpp")
    }

    #[test]
    fn cpp_source_contains_locked_decimal_literals() {
        // Arrange
        let source = fs::read_to_string(cpp_bench_source()).expect("C++ Dam Break bench source");

        // Act / Assert
        for literal in [
            "0.06324555",
            "0.101193",
            "48",
            "40",
            "-4.7",
            "0.4",
            "0.75",
            "2.5",
            "5.5",
            "-10",
        ] {
            assert!(
                source.contains(literal),
                "C++ Dam Break bench must contain `{literal}`"
            );
        }
    }

    #[test]
    fn json_escape_quotes_compiler_text() {
        // Arrange
        let compiler = r#"rustc 1.97.0 "ok""#;

        // Act
        let escaped = json_escape(compiler);

        // Assert
        assert_eq!(escaped, r#"rustc 1.97.0 \"ok\""#);
    }

    #[test]
    fn one_timed_step_reports_locked_particle_count() {
        // Arrange / Act
        let report = run_dam_break_bench(0, 1).expect("one native Dam Break step should run");

        // Assert
        assert_eq!(report.particles, EXPECTED_PARTICLE_COUNT);
        assert_eq!(report.engine, "native_rust");
        assert_eq!(report.measured_steps, 1);
        assert!(report.wall_ms > 0.0);
        assert!(report.ms_per_step > 0.0);
        assert!(report.realtime_factor > 0.0);
        assert!(report.compiler.contains("rustc"));
        assert!(report.to_json().contains("\"engine\":\"native_rust\""));
    }
}
