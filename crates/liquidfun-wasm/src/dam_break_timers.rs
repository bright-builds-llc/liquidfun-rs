//! Native-only Dam Break Medium parent-phase `step_profiled` timers.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::dam_break_bench::EXPECTED_PARTICLE_COUNT;

/// One Phase 12 parent wall-clock total, in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParentWallMs {
    /// Summed parent `Duration` converted with `as_secs_f64() * 1000.0`.
    pub wall_ms: f64,
}

/// Closed error for the native Dam Break parent-timer diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DamBreakTimersError {
    /// Scene construction failed.
    SceneConstruction,
    /// The live scene particle count is not the locked Medium recipe.
    ParticleCount {
        /// Particles present after construction.
        actual: usize,
    },
    /// Warm-up `advance` or measured `advance_profiled` failed.
    StepFailed {
        /// Whether the failure happened during warm-up or the timed loop.
        phase: &'static str,
    },
}

impl Display for DamBreakTimersError {
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
                write!(formatter, "Dam Break {phase} step_profiled failed")
            }
        }
    }
}

impl Error for DamBreakTimersError {}

/// Coarse Phase 12 parent totals for a Dam Break diagnostic run.
#[derive(Debug, Clone, PartialEq)]
pub struct DamBreakTimersReport {
    /// Envelope kind. Never the unprofiled 3× gate.
    pub kind: &'static str,
    /// Phase 12 profile schema token.
    pub schema: &'static str,
    /// Always true: these walls are not timing authority.
    pub not_timing_authority: bool,
    /// Particles present for the timed steps.
    pub particles: usize,
    /// Untimed ordinary `advance(1)` steps taken before the sample.
    pub warmup_steps: u32,
    /// Timed `advance_profiled` count.
    pub measured_steps: u32,
    /// Parent token → summed wall milliseconds.
    pub parents: BTreeMap<&'static str, ParentWallMs>,
}

impl DamBreakTimersReport {
    /// Renders one JSON object for the xtask driver.
    #[must_use]
    pub fn to_json(&self) -> String {
        String::from("{}")
    }
}

/// Builds playground Dam Break Medium/Normal and aggregates parent timers.
///
/// # Errors
///
/// Returns a closed error when construction, the locked particle count, or a
/// step fails.
pub fn run_dam_break_timers(
    _warmup_steps: u32,
    _measured_steps: u32,
) -> Result<DamBreakTimersReport, DamBreakTimersError> {
    Ok(DamBreakTimersReport {
        kind: "unprofiled_pair",
        schema: "missing",
        not_timing_authority: false,
        particles: 0,
        warmup_steps: 0,
        measured_steps: 0,
        parents: BTreeMap::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::run_dam_break_timers;
    use crate::dam_break_bench::EXPECTED_PARTICLE_COUNT;

    #[test]
    fn one_profiled_step_emits_required_parent_tokens() {
        // Arrange / Act
        let report = run_dam_break_timers(0, 1).expect("one Dam Break profiled step should run");

        // Assert
        for token in ["particle_prepare", "particle_solve", "rigid_solve"] {
            let Some(parent) = report.parents.get(token) else {
                panic!("parents must contain `{token}`");
            };
            assert!(
                parent.wall_ms >= 0.0,
                "`{token}` wall_ms must be >= 0, got {}",
                parent.wall_ms
            );
        }
    }

    #[test]
    fn one_profiled_step_is_labeled_not_timing_authority() {
        // Arrange / Act
        let report = run_dam_break_timers(0, 1).expect("one Dam Break profiled step should run");
        let json = report.to_json();

        // Assert
        assert_eq!(report.kind, "step_profiled_parents");
        assert!(report.not_timing_authority);
        assert_eq!(report.schema, "phase12-profile-v1");
        assert!(json.contains("\"kind\":\"step_profiled_parents\""));
        assert!(json.contains("\"not_timing_authority\":true"));
    }

    #[test]
    fn one_profiled_step_reports_locked_particle_count() {
        // Arrange / Act
        let report = run_dam_break_timers(0, 1).expect("one Dam Break profiled step should run");

        // Assert
        assert_eq!(report.particles, EXPECTED_PARTICLE_COUNT);
    }

    #[test]
    fn session_advance_cap_stays_four_steps() {
        // Arrange
        let session_source = include_str!("session.rs");

        // Act / Assert
        assert!(session_source.contains("MAX_ADVANCE_STEPS: u32 = 4"));
    }

    #[test]
    fn dam_break_bench_does_not_call_step_profiled() {
        // Arrange
        let bench = include_str!("dam_break_bench.rs");
        let bin = include_str!("bin/dam_break_bench.rs");

        // Act / Assert
        assert!(!bench.contains("step_profiled"));
        assert!(!bench.contains("advance_profiled"));
        assert!(!bin.contains("step_profiled"));
        assert!(!bin.contains("advance_profiled"));
    }
}
