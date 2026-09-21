//! Native-only Dam Break Medium parent-phase `step_profiled` timers.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use liquidfun::{DiagnosticProfileParent, DiagnosticProfileSchema, DiagnosticStepProfile};

use crate::dam_break_bench::EXPECTED_PARTICLE_COUNT;
use crate::scene::SceneId;
use crate::session::SessionCore;

const KIND: &str = "step_profiled_parents";

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
        let not_timing_authority = if self.not_timing_authority {
            "true"
        } else {
            "false"
        };
        format!(
            "{{\"kind\":\"{}\",\"schema\":\"{}\",\"not_timing_authority\":{},\"particles\":{},\"warmup_steps\":{},\"measured_steps\":{},\"parents\":{}}}",
            self.kind,
            self.schema,
            not_timing_authority,
            self.particles,
            self.warmup_steps,
            self.measured_steps,
            parents_json(&self.parents),
        )
    }
}

/// Builds playground Dam Break Medium/Normal and aggregates parent timers.
///
/// # Errors
///
/// Returns a closed error when construction, the locked particle count, or a
/// step fails.
pub fn run_dam_break_timers(
    warmup_steps: u32,
    measured_steps: u32,
) -> Result<DamBreakTimersReport, DamBreakTimersError> {
    if measured_steps == 0 {
        return Err(DamBreakTimersError::StepFailed { phase: "measured" });
    }

    let mut session = SessionCore::create(SceneId::DamBreak)
        .map_err(|_error| DamBreakTimersError::SceneConstruction)?;
    let particles = session.particle_count();
    if particles != EXPECTED_PARTICLE_COUNT {
        return Err(DamBreakTimersError::ParticleCount { actual: particles });
    }

    for _ in 0..warmup_steps {
        session
            .advance(1)
            .map_err(|_error| DamBreakTimersError::StepFailed { phase: "warmup" })?;
    }

    let mut parent_totals = empty_parent_totals();
    for _ in 0..measured_steps {
        let profile = session
            .advance_profiled()
            .map_err(|_error| DamBreakTimersError::StepFailed { phase: "measured" })?;
        accumulate_parents(&mut parent_totals, &profile);
    }

    Ok(DamBreakTimersReport {
        kind: KIND,
        schema: DiagnosticProfileSchema::Phase12V1.as_str(),
        not_timing_authority: true,
        particles,
        warmup_steps,
        measured_steps,
        parents: parent_totals,
    })
}

fn empty_parent_totals() -> BTreeMap<&'static str, ParentWallMs> {
    let mut totals = BTreeMap::new();
    for parent in DiagnosticProfileParent::ALL {
        totals.insert(parent.as_str(), ParentWallMs { wall_ms: 0.0 });
    }
    totals
}

fn accumulate_parents(
    totals: &mut BTreeMap<&'static str, ParentWallMs>,
    profile: &DiagnosticStepProfile,
) {
    for timing in profile.phases() {
        let Some(parent) = timing.phase().maybe_common_parent() else {
            continue;
        };
        let token = parent.as_str();
        let Some(entry) = totals.get_mut(token) else {
            continue;
        };
        entry.wall_ms += timing.duration().as_secs_f64() * 1000.0;
    }
}

fn parents_json(parents: &BTreeMap<&'static str, ParentWallMs>) -> String {
    let entries: Vec<String> = DiagnosticProfileParent::ALL
        .iter()
        .map(|parent| {
            let token = parent.as_str();
            let wall_ms = parents.get(token).map_or(0.0, |entry| entry.wall_ms);
            format!("\"{token}\":{{\"wall_ms\":{wall_ms:.6}}}")
        })
        .collect();
    format!("{{{}}}", entries.join(","))
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
