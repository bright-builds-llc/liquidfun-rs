//! Native-only all-catalog `SessionCore` stepper for the playground scene survey.

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::time::{Duration, Instant};

use crate::scene::SceneId;
use crate::session::SessionCore;

/// Untimed first-contact steps before the measured sample.
pub const DEFAULT_WARMUP_STEPS: u32 = 60;
/// Timed `advance(1)` count. Shorter than Dam Break's 600-step gate recipe.
pub const DEFAULT_MEASURED_STEPS: u32 = 120;
/// Repeated fresh-session runs per scene (D-05).
pub const DEFAULT_RUNS: u32 = 3;
/// Inclusive wall timeout covering warmup plus measured steps for one run of one scene.
pub const SCENE_WALL_TIMEOUT: Duration = Duration::from_mins(3);

/// Survey-only input applied to scenes that stay idle without interaction.
#[derive(Debug, Clone, Copy, PartialEq)]
enum SurveyCue {
    /// The scene runs as constructed.
    Default,
    /// A named scene action, as the README previews use.
    Action(&'static str),
    /// A pointer release at a world position.
    PointerUp { x: f32, y: f32 },
}

impl SurveyCue {
    const fn label(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Action(_) | Self::PointerUp { .. } => "scripted",
        }
    }
}

/// Every playground catalog scene, in `web/src/catalog/scenes.ts` order.
const SURVEY_SCENES: [(&str, SceneId, SurveyCue); 25] = [
    ("wave-machine", SceneId::WaveMachine, SurveyCue::Default),
    ("dam-break", SceneId::DamBreak, SurveyCue::Default),
    ("fountain", SceneId::Fountain, SurveyCue::Default),
    (
        "float-or-sink",
        SceneId::FloatOrSink,
        SurveyCue::Action("drop-body"),
    ),
    ("color-mixer", SceneId::ColorMixer, SurveyCue::Default),
    ("jelly-drop", SceneId::JellyDrop, SurveyCue::Default),
    ("water-wheel", SceneId::WaterWheel, SurveyCue::Default),
    ("particles", SceneId::Particles, SurveyCue::Default),
    ("liquid-timer", SceneId::LiquidTimer, SurveyCue::Default),
    (
        "surface-tension",
        SceneId::SurfaceTension,
        SurveyCue::Default,
    ),
    (
        "elastic-particles",
        SceneId::ElasticParticles,
        SurveyCue::Default,
    ),
    (
        "rigid-particles",
        SceneId::RigidParticles,
        SurveyCue::Default,
    ),
    ("soup", SceneId::Soup, SurveyCue::Default),
    ("soup-stirrer", SceneId::SoupStirrer, SurveyCue::Default),
    (
        "impulse",
        SceneId::Impulse,
        SurveyCue::PointerUp { x: 1.0, y: 2.0 },
    ),
    ("theo-jansen", SceneId::TheoJansen, SurveyCue::Default),
    ("liquid-tumbler", SceneId::LiquidTumbler, SurveyCue::Default),
    (
        "drawing-particles",
        SceneId::DrawingParticles,
        SurveyCue::PointerUp { x: 0.0, y: 2.0 },
    ),
    ("sparky", SceneId::Sparky, SurveyCue::Default),
    (
        "hydraulic-fountain",
        SceneId::HydraulicFountain,
        SurveyCue::Default,
    ),
    ("wave-tank", SceneId::WaveTank, SurveyCue::Default),
    ("liquid-bubbler", SceneId::LiquidBubbler, SurveyCue::Default),
    ("stacked-drip", SceneId::StackedDrip, SurveyCue::Default),
    (
        "washing-machine",
        SceneId::WashingMachine,
        SurveyCue::Default,
    ),
    ("tesla-valve", SceneId::TeslaValve, SurveyCue::Default),
];

/// Closed error for the native all-catalog scene survey timer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneSpotError {
    /// Scene construction failed.
    SceneConstruction {
        /// Hyphenated scene id.
        scene: &'static str,
    },
    /// The survey cue for an idle scene failed to apply.
    CueFailed {
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
    /// One run of one scene exceeded [`SCENE_WALL_TIMEOUT`].
    TimedOut {
        /// Hyphenated scene id.
        scene: &'static str,
    },
    /// The caller asked for zero runs per scene.
    ZeroRuns,
}

impl Display for SceneSpotError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::SceneConstruction { scene } => {
                write!(formatter, "{scene} scene construction failed")
            }
            Self::CueFailed { scene } => write!(formatter, "{scene} survey cue failed"),
            Self::StepFailed { scene, phase } => {
                write!(formatter, "{scene} {phase} SessionCore::advance(1) failed")
            }
            Self::TimedOut { scene } => write!(
                formatter,
                "{scene} exceeded {}s wall timeout",
                SCENE_WALL_TIMEOUT.as_secs()
            ),
            Self::ZeroRuns => write!(formatter, "--runs must be greater than 0"),
        }
    }
}

impl Error for SceneSpotError {}

/// One native playground scene survey result, aggregated across runs.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneSpotSample {
    /// Hyphenated scene id.
    pub scene: &'static str,
    /// `default` when the scene runs as constructed, `scripted` when a survey cue applies.
    pub interaction: &'static str,
    /// Fresh-session runs behind the spread.
    pub runs: u32,
    /// Untimed steps taken before each timed window.
    pub warmup_steps: u32,
    /// Timed `advance(1)` count per run.
    pub measured_steps: u32,
    /// Live particles after construction, before any cue or warmup, in the last run.
    pub start_particles: usize,
    /// Live particles after warmup plus measured steps, in the last run.
    pub end_particles: usize,
    /// Median wall milliseconds per timed step across runs.
    pub median_ms_per_step: f64,
    /// Fastest run's wall milliseconds per timed step.
    pub min_ms_per_step: f64,
    /// Slowest run's wall milliseconds per timed step.
    pub max_ms_per_step: f64,
    /// Always false on success; timeouts fail the process instead.
    pub timed_out: bool,
}

impl SceneSpotSample {
    /// Renders one JSON object for the xtask driver.
    #[must_use]
    pub fn to_json(&self) -> String {
        format!(
            "{{\"scene\":\"{}\",\"interaction\":\"{}\",\"runs\":{},\"warmup_steps\":{},\"measured_steps\":{},\"start_particles\":{},\"end_particles\":{},\"median_ms_per_step\":{:.6},\"min_ms_per_step\":{:.6},\"max_ms_per_step\":{:.6},\"timed_out\":{}}}",
            json_escape(self.scene),
            json_escape(self.interaction),
            self.runs,
            self.warmup_steps,
            self.measured_steps,
            self.start_particles,
            self.end_particles,
            self.median_ms_per_step,
            self.min_ms_per_step,
            self.max_ms_per_step,
            self.timed_out,
        )
    }
}

/// Builds every playground catalog scene `runs` times and times `advance(1)`.
///
/// # Errors
///
/// Returns a closed error when `runs` or `measured_steps` is zero, or when
/// construction, a survey cue, a step, a live particle snapshot, or the
/// per-run wall timeout fails.
pub fn run_scene_spot(
    warmup_steps: u32,
    measured_steps: u32,
    runs: u32,
) -> Result<Vec<SceneSpotSample>, SceneSpotError> {
    if runs == 0 {
        return Err(SceneSpotError::ZeroRuns);
    }
    if measured_steps == 0 {
        return Err(SceneSpotError::StepFailed {
            scene: "all",
            phase: "measured",
        });
    }

    let mut samples = Vec::with_capacity(SURVEY_SCENES.len());
    for (scene, id, cue) in SURVEY_SCENES {
        samples.push(survey_scene(
            scene,
            id,
            cue,
            warmup_steps,
            measured_steps,
            runs,
        )?);
    }
    Ok(samples)
}

fn survey_scene(
    scene: &'static str,
    id: SceneId,
    cue: SurveyCue,
    warmup_steps: u32,
    measured_steps: u32,
    runs: u32,
) -> Result<SceneSpotSample, SceneSpotError> {
    let mut ms_per_step = Vec::with_capacity(runs as usize);
    let mut maybe_last_run = None;
    for _ in 0..runs {
        let run = time_run(scene, id, cue, warmup_steps, measured_steps)?;
        ms_per_step.push(run.ms_per_step);
        maybe_last_run = Some(run);
    }
    let (Some(spread), Some(last_run)) = (summarize(&mut ms_per_step), maybe_last_run) else {
        return Err(SceneSpotError::ZeroRuns);
    };

    Ok(SceneSpotSample {
        scene,
        interaction: cue.label(),
        runs,
        warmup_steps,
        measured_steps,
        start_particles: last_run.start_particles,
        end_particles: last_run.end_particles,
        median_ms_per_step: spread.median,
        min_ms_per_step: spread.min,
        max_ms_per_step: spread.max,
        timed_out: false,
    })
}

/// One fresh-session timed run.
struct RunSample {
    ms_per_step: f64,
    start_particles: usize,
    end_particles: usize,
}

fn time_run(
    scene: &'static str,
    id: SceneId,
    cue: SurveyCue,
    warmup_steps: u32,
    measured_steps: u32,
) -> Result<RunSample, SceneSpotError> {
    let mut session =
        SessionCore::create(id).map_err(|_error| SceneSpotError::SceneConstruction { scene })?;
    let start_particles = session
        .live_particle_count()
        .map_err(|_error| SceneSpotError::SceneConstruction { scene })?;
    apply_cue(&mut session, scene, cue)?;

    let run_started = Instant::now();
    for _ in 0..warmup_steps {
        session
            .advance(1)
            .map_err(|_error| SceneSpotError::StepFailed {
                scene,
                phase: "warmup",
            })?;
        if run_started.elapsed() >= SCENE_WALL_TIMEOUT {
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
        if run_started.elapsed() >= SCENE_WALL_TIMEOUT {
            return Err(SceneSpotError::TimedOut { scene });
        }
    }
    let timed_ms = duration_as_millis(measured_started.elapsed());
    let end_particles = session
        .live_particle_count()
        .map_err(|_error| SceneSpotError::SceneConstruction { scene })?;

    Ok(RunSample {
        ms_per_step: timed_ms / f64::from(measured_steps),
        start_particles,
        end_particles,
    })
}

fn apply_cue(
    session: &mut SessionCore,
    scene: &'static str,
    cue: SurveyCue,
) -> Result<(), SceneSpotError> {
    let result = match cue {
        SurveyCue::Default => return Ok(()),
        SurveyCue::Action(name) => session.apply_action(name),
        SurveyCue::PointerUp { x, y } => session.apply_pointer("up", x, y),
    };
    result.map_err(|_error| SceneSpotError::CueFailed { scene })
}

/// Median, fastest and slowest ms/step across runs.
#[derive(Debug, Clone, Copy, PartialEq)]
struct RunSpread {
    median: f64,
    min: f64,
    max: f64,
}

/// Sorts `values` and reduces them to a spread; an even count averages the two middle values.
fn summarize(values: &mut [f64]) -> Option<RunSpread> {
    values.sort_by(f64::total_cmp);
    let (&min, &max) = (values.first()?, values.last()?);
    let middle = values.len() / 2;
    let median = if values.len() % 2 == 1 {
        values[middle]
    } else {
        f64::midpoint(values[middle - 1], values[middle])
    };
    Some(RunSpread { median, min, max })
}

fn duration_as_millis(elapsed: Duration) -> f64 {
    (elapsed.as_secs_f64() * 1000.0).max(f64::EPSILON)
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests;
