//! Native-only five-scene `SessionCore` stepper for playground spot-checks.

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::time::{Duration, Instant};

use crate::scene::SceneId;
use crate::session::SessionCore;

/// Untimed first-contact steps before the measured sample.
pub const DEFAULT_WARMUP_STEPS: u32 = 60;
/// Timed `advance(1)` count. Shorter than Dam Break's 600-step gate recipe.
pub const DEFAULT_MEASURED_STEPS: u32 = 120;
/// Inclusive wall timeout covering warmup plus measured steps for one scene.
pub const SCENE_WALL_TIMEOUT: Duration = Duration::from_mins(3);

const SPOT_SCENES: [(&str, SceneId); 5] = [
    ("fountain", SceneId::Fountain),
    ("float-or-sink", SceneId::FloatOrSink),
    ("color-mixer", SceneId::ColorMixer),
    ("jelly-drop", SceneId::JellyDrop),
    ("water-wheel", SceneId::WaterWheel),
];

/// Closed error for the native five-scene spot-check timer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneSpotError {
    /// Scene construction failed.
    SceneConstruction {
        /// Hyphenated scene id.
        scene: &'static str,
    },
    /// Warm-up or measured `advance(1)` failed.
    StepFailed {
        /// Hyphenated scene id.
        scene: &'static str,
        /// Whether the failure happened during warm-up or the timed loop.
        phase: &'static str,
    },
    /// One scene exceeded [`SCENE_WALL_TIMEOUT`].
    TimedOut {
        /// Hyphenated scene id.
        scene: &'static str,
    },
}

impl Display for SceneSpotError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::SceneConstruction { scene } => {
                write!(formatter, "{scene} scene construction failed")
            }
            Self::StepFailed { scene, phase } => {
                write!(formatter, "{scene} {phase} SessionCore::advance(1) failed")
            }
            Self::TimedOut { scene } => write!(
                formatter,
                "{scene} exceeded {}s wall timeout",
                SCENE_WALL_TIMEOUT.as_secs()
            ),
        }
    }
}

impl Error for SceneSpotError {}

/// One native playground scene wall-clock sample.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneSpotSample {
    /// Hyphenated scene id.
    pub scene: &'static str,
    /// Untimed steps taken before the sample.
    pub warmup_steps: u32,
    /// Timed `advance(1)` count.
    pub measured_steps: u32,
    /// Live particles after construction, before warmup.
    pub start_particles: usize,
    /// Live particles after warmup plus measured steps.
    pub end_particles: usize,
    /// Wall time of the timed loop, in milliseconds.
    pub wall_ms: f64,
    /// Mean wall milliseconds per timed step.
    pub ms_per_step: f64,
    /// Always false on success; timeouts fail the process instead.
    pub timed_out: bool,
}

impl SceneSpotSample {
    /// Renders one JSON object for the xtask driver.
    #[must_use]
    pub fn to_json(&self) -> String {
        format!(
            "{{\"scene\":\"{}\",\"warmup_steps\":{},\"measured_steps\":{},\"start_particles\":{},\"end_particles\":{},\"wall_ms\":{:.6},\"ms_per_step\":{:.6},\"timed_out\":{}}}",
            json_escape(self.scene),
            self.warmup_steps,
            self.measured_steps,
            self.start_particles,
            self.end_particles,
            self.wall_ms,
            self.ms_per_step,
            self.timed_out,
        )
    }
}

/// Builds the five non-Dam-Break playground scenes and times `advance(1)`.
///
/// # Errors
///
/// Returns a closed error when construction, a step, a live particle snapshot,
/// or the per-scene wall timeout fails.
pub fn run_scene_spot(
    warmup_steps: u32,
    measured_steps: u32,
) -> Result<Vec<SceneSpotSample>, SceneSpotError> {
    if measured_steps == 0 {
        return Err(SceneSpotError::StepFailed {
            scene: "all",
            phase: "measured",
        });
    }

    let mut samples = Vec::with_capacity(SPOT_SCENES.len());
    for (scene, id) in SPOT_SCENES {
        samples.push(time_scene(scene, id, warmup_steps, measured_steps)?);
    }
    Ok(samples)
}

fn time_scene(
    scene: &'static str,
    id: SceneId,
    warmup_steps: u32,
    measured_steps: u32,
) -> Result<SceneSpotSample, SceneSpotError> {
    let mut session =
        SessionCore::create(id).map_err(|_error| SceneSpotError::SceneConstruction { scene })?;
    let start_particles = session
        .live_particle_count()
        .map_err(|_error| SceneSpotError::SceneConstruction { scene })?;

    let scene_started = Instant::now();
    for _ in 0..warmup_steps {
        session
            .advance(1)
            .map_err(|_error| SceneSpotError::StepFailed {
                scene,
                phase: "warmup",
            })?;
        if scene_started.elapsed() >= SCENE_WALL_TIMEOUT {
            return Err(SceneSpotError::TimedOut { scene });
        }
    }

    let measured_started = Instant::now();
    for _ in 0..measured_steps {
        session
            .advance(1)
            .map_err(|_error| SceneSpotError::StepFailed {
                scene,
                phase: "measured",
            })?;
        if scene_started.elapsed() >= SCENE_WALL_TIMEOUT {
            return Err(SceneSpotError::TimedOut { scene });
        }
    }
    let wall_ms = duration_as_millis(measured_started.elapsed());
    let end_particles = session
        .live_particle_count()
        .map_err(|_error| SceneSpotError::SceneConstruction { scene })?;

    Ok(SceneSpotSample {
        scene,
        warmup_steps,
        measured_steps,
        start_particles,
        end_particles,
        wall_ms,
        ms_per_step: wall_ms / f64::from(measured_steps),
        timed_out: false,
    })
}

fn duration_as_millis(elapsed: Duration) -> f64 {
    (elapsed.as_secs_f64() * 1000.0).max(f64::EPSILON)
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::SceneSpotSample;

    #[test]
    fn to_json_omits_ratio_and_reports_success() {
        // Arrange
        let sample = SceneSpotSample {
            scene: "fountain",
            warmup_steps: 60,
            measured_steps: 120,
            start_particles: 10,
            end_particles: 40,
            wall_ms: 12.5,
            ms_per_step: 0.104_166,
            timed_out: false,
        };

        // Act
        let json = sample.to_json();

        // Assert
        assert!(json.contains("\"scene\":\"fountain\""));
        assert!(json.contains("\"timed_out\":false"));
        assert!(!json.contains("rust_over_cpp_ratio"));
        assert!(!json.contains("pair.json"));
    }
}
