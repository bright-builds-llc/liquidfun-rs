//! Time-boxed siphon feasibility spike (Phase 45 D-14..D-22, PREP-03).
//!
//! Builds the fixed primed inverted-U geometry from `geometry`, runs each
//! D-20 candidate for 30 s at gravity 10 and judges it by the D-21 rules
//! recorded in `45-SIPHON-SPIKE.md` before the first measurement. The output
//! is authoring evidence for the go, reshape or drop decision only (D-16).
//!
//! Run with
//! `cargo test -p liquidfun-wasm --release --lib siphon_spike -- --ignored --nocapture --test-threads=1 2>&1 | tee target/phase45/siphon-spike.log`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    reason = "spike metrics: particle counts, bins and ranks stay far below 2^23"
)]

mod geometry;

use std::error::Error;
use std::time::Instant;

use liquidfun::collision::{FilterData, PolygonShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{ParticleGroupDestination, ParticleGroupRecipe, ParticleGroupSource};
use liquidfun::{
    BodyDef, BodyType, FixtureDef, NoDecisionHook, ParticleFlags, ParticleId, ParticleSystemDef,
    ParticleSystemId, StepConfiguration, StepLimits, World,
};

use self::geometry::{
    CREST, CREST_CHANNEL, DEST, INLET_LEG, INLET_MOUTH_TOP, OUTLET_LEG, RADIUS, Rect, SOURCE,
    SPACING, WALLS,
};
use crate::scene::attach_basin_fixture;

type SpikeResult<T> = Result<T, Box<dyn Error>>;

const STEPS_PER_SECOND: u32 = 60;
/// One sample every 0.25 s.
const SAMPLE_STEPS: u32 = 15;
const SAMPLES_PER_SECOND: usize = 4;
/// 30 s at 60 Hz.
const RUN_STEPS: u32 = 1_800;
const PARTICLE_ITERATIONS: u32 = 2;
const GRAVITY: f32 = 10.0;
const MAXIMUM_COUNT: usize = 4_000;
/// Particle density; the fill's areal mass density equals it.
const PARTICLE_DENSITY: f32 = 1.0;

/// Rule A starts once the source surface is this far below the crest.
const RULE_A_DROP: f32 = 0.3;
/// `t*` is never taken before 2 s, so the outlet leg's priming drain and the
/// column's compression settling cannot satisfy rule A by themselves.
const RULE_A_EARLIEST_SAMPLE: usize = 2 * SAMPLES_PER_SECOND;
/// 0.5 s windows.
const RULE_A_WINDOW_SAMPLES: usize = 2;
/// Four windows = 2 s.
const RULE_A_WINDOWS: usize = 4;
const RULE_A_MIN_TRANSFER: usize = 10;
const RULE_B_MAX_DIFFERENCE: f32 = 0.15;
/// 2 s without a `count_d` change.
const RULE_B_SETTLE_SAMPLES: usize = 8;
/// Surface bins are two diameters wide.
const SURFACE_BIN_WIDTH: f32 = 0.2;
const REPORT_SECONDS: [usize; 5] = [2, 5, 10, 20, 30];

/// Surface particles weigh less than this fraction of the tank median.
const SURFACE_WEIGHT_FRACTION: f32 = 0.5;
/// Lid size: tank width − 0.1 by 0.2 m.
const LID_CLEARANCE: f32 = 0.1;
const LID_HEIGHT: f32 = 0.2;
/// Water column whose weight per meter a β = 1 lid matches.
const LID_REFERENCE_COLUMN: f32 = 0.5;

/// Scene-level mechanism layered on the particle material.
#[derive(Clone, Copy, Debug)]
enum Mechanism {
    Natural,
    FakeAtmosphere { alpha: f32 },
    WeightedLids { beta: f32 },
    TubePull { alpha: f32 },
}

/// One spike configuration.
#[derive(Clone, Debug)]
struct Candidate {
    label: String,
    flags: ParticleFlags,
    maybe_viscous_strength: Option<f32>,
    mechanism: Mechanism,
}

impl Candidate {
    fn water(label: &str, mechanism: Mechanism) -> Self {
        Self {
            label: label.to_owned(),
            flags: ParticleFlags::empty(),
            maybe_viscous_strength: None,
            mechanism,
        }
    }
}

/// SceneHooks-shaped per-step hook so a passing stand-in can be lifted into
/// Phase 50.
trait SpikeHook {
    fn on_advance(&mut self, world: &mut World, system: ParticleSystemId) -> SpikeResult<()>;
}

struct NoHook;

impl SpikeHook for NoHook {
    fn on_advance(&mut self, _world: &mut World, _system: ParticleSystemId) -> SpikeResult<()> {
        Ok(())
    }
}

/// Candidate 4: extra acceleration `α·|g|` along gravity on low-weight
/// particles in `S ∪ D`.
struct FakeAtmosphere {
    alpha: f32,
}

impl SpikeHook for FakeAtmosphere {
    fn on_advance(&mut self, world: &mut World, system: ParticleSystemId) -> SpikeResult<()> {
        let gravity = world.gravity();
        let magnitude = gravity.length();
        if magnitude <= 0.0 {
            return Ok(());
        }
        let force = gravity * (force_per_acceleration() * self.alpha);
        let targets = {
            let view = world.particle_system_view(system)?;
            surface_indices(view.positions(), view.weights(), view.flags())
                .into_iter()
                .map(|index| view.particle_ids()[index])
                .collect::<Vec<ParticleId>>()
        };
        for particle in targets {
            world.apply_particle_force(particle, force)?;
        }
        Ok(())
    }
}

/// Candidate 6: while the source surface is above the inlet mouth top, pull
/// tube particles up the inlet leg, across the crest and down the outlet leg.
struct TubePull {
    alpha: f32,
}

impl SpikeHook for TubePull {
    fn on_advance(&mut self, world: &mut World, system: ParticleSystemId) -> SpikeResult<()> {
        let gravity = world.gravity();
        let magnitude = gravity.length();
        if magnitude <= 0.0 {
            return Ok(());
        }
        let down = gravity * (1.0 / magnitude);
        let scale = force_per_acceleration() * self.alpha * magnitude;
        let targets = {
            let view = world.particle_system_view(system)?;
            if surface(view.positions(), SOURCE) <= INLET_MOUTH_TOP {
                return Ok(());
            }
            view.positions()
                .iter()
                .zip(view.flags())
                .zip(view.particle_ids())
                .filter(|((_, flags), _)| !flags.contains(ParticleFlags::WALL))
                .filter_map(|((position, _), id)| {
                    tube_direction(*position, down).map(|direction| (*id, direction * scale))
                })
                .collect::<Vec<(ParticleId, Vec2)>>()
        };
        for (particle, force) in targets {
            world.apply_particle_force(particle, force)?;
        }
        Ok(())
    }
}

/// Pull direction for a tube particle, if it is inside the tube.
fn tube_direction(position: Vec2, down: Vec2) -> Option<Vec2> {
    if INLET_LEG.contains(position) {
        return Some(down * -1.0);
    }
    if CREST_CHANNEL.contains(position) {
        return Some(Vec2::new(1.0, 0.0));
    }
    if OUTLET_LEG.contains(position) {
        return Some(down);
    }
    None
}

/// Force per unit acceleration for one particle over one world step:
/// `m · particle_iterations`, `m = density · (0.75 · 2r)²` (research A2).
fn force_per_acceleration() -> f32 {
    PARTICLE_DENSITY * SPACING * SPACING * PARTICLE_ITERATIONS as f32
}

/// Indices of non-wall particles in `S ∪ D` whose weight is below half the
/// median weight of those particles.
fn surface_indices(positions: &[Vec2], weights: &[f32], flags: &[ParticleFlags]) -> Vec<usize> {
    let tank: Vec<usize> = (0..positions.len())
        .filter(|&index| !flags[index].contains(ParticleFlags::WALL))
        .filter(|&index| SOURCE.contains(positions[index]) || DEST.contains(positions[index]))
        .collect();
    let tank_weights: Vec<f32> = tank.iter().map(|&index| weights[index]).collect();
    let threshold = SURFACE_WEIGHT_FRACTION * percentile(&tank_weights, 0.5);
    tank.into_iter()
        .filter(|&index| weights[index] < threshold)
        .collect()
}

/// One built spike world.
struct SpikeWorld {
    world: World,
    system: ParticleSystemId,
    start_count: usize,
}

fn build_world(candidate: &Candidate) -> SpikeResult<SpikeWorld> {
    let mut world = World::new()?;
    world.set_gravity(Vec2::new(0.0, -GRAVITY))?;
    let ground = world.create_body(&BodyDef::default())?;
    for wall in WALLS {
        attach_basin_fixture(&mut world, ground, &wall.quad())
            .map_err(|error| format!("wall fixture: {error:?}"))?;
    }
    let mut definition = ParticleSystemDef::default()
        .with_density(PARTICLE_DENSITY)?
        .with_radius(RADIUS)?
        .with_maximum_count(MAXIMUM_COUNT)?;
    if let Some(strength) = candidate.maybe_viscous_strength {
        definition = definition.with_viscous_strength(strength)?;
    }
    let system = world.create_particle_system_with_def(&definition)?;
    let points = geometry::primed_fill();
    let start_count = points.len();
    let source_top = region_top(&points, SOURCE);
    let dest_top = region_top(&points, DEST);
    let recipe = ParticleGroupRecipe::new(
        ParticleGroupSource::positions(points)?,
        ParticleGroupDestination::New,
    )
    .with_particle_flags(candidate.flags)
    .with_transform(Transform::IDENTITY)?;
    world.create_particle_group(system, &recipe)?;
    if let Mechanism::WeightedLids { beta } = candidate.mechanism {
        add_lid(&mut world, SOURCE, source_top, beta)?;
        add_lid(&mut world, DEST, dest_top, beta)?;
    }
    Ok(SpikeWorld {
        world,
        system,
        start_count,
    })
}

/// Highest fill point inside `region`.
fn region_top(points: &[Vec2], region: Rect) -> f32 {
    points
        .iter()
        .filter(|point| region.contains(**point))
        .map(|point| point.y)
        .fold(region.min.y, f32::max)
}

/// Candidate 5: a fixed-rotation, frictionless dynamic lid resting on the
/// tank's initial surface, weighing β × a 0.5 m water column per meter.
fn add_lid(world: &mut World, tank: Rect, surface_top: f32, beta: f32) -> SpikeResult<()> {
    let half_width = 0.5 * (tank.width() - LID_CLEARANCE);
    let half_height = 0.5 * LID_HEIGHT;
    let center = Vec2::new(tank.center_x(), surface_top + RADIUS + half_height);
    let density = beta * PARTICLE_DENSITY * LID_REFERENCE_COLUMN / LID_HEIGHT;
    let body = world.create_body(&BodyDef::new(BodyType::Dynamic, center, 0.0, true)?)?;
    let shape = Shape::from(PolygonShape::box_shape(half_width, half_height)?);
    let fixture = FixtureDef::new(shape, density, 0.0, 0.0, false, FilterData::default())?;
    world.create_fixture(body, &fixture)?;
    world.set_body_fixed_rotation(body, true)?;
    Ok(())
}

fn hook_for(mechanism: Mechanism) -> Box<dyn SpikeHook> {
    match mechanism {
        Mechanism::Natural | Mechanism::WeightedLids { .. } => Box::new(NoHook),
        Mechanism::FakeAtmosphere { alpha } => Box::new(FakeAtmosphere { alpha }),
        Mechanism::TubePull { alpha } => Box::new(TubePull { alpha }),
    }
}

/// One 0.25 s measurement.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Sample {
    seconds: f32,
    source_surface: f32,
    dest_surface: f32,
    count_d: usize,
    /// Informational: particles inside `S`.
    count_s: usize,
    leaks: usize,
    lost: usize,
}

impl Sample {
    fn level_difference(self) -> f32 {
        (self.source_surface - self.dest_surface).abs()
    }
}

/// Free-surface height of the particles inside `region`: the median over
/// occupied 0.2 m x-bins of each bin's highest particle. An empty region
/// reads as its floor.
fn surface(positions: &[Vec2], region: Rect) -> f32 {
    let bins = (region.width() / SURFACE_BIN_WIDTH).ceil().max(1.0) as usize;
    let mut tops: Vec<Option<f32>> = vec![None; bins];
    for position in positions
        .iter()
        .filter(|position| region.contains(**position))
    {
        let bin = (((position.x - region.min.x) / SURFACE_BIN_WIDTH) as usize).min(bins - 1);
        let top = tops[bin].map_or(position.y, |top| top.max(position.y));
        tops[bin] = Some(top);
    }
    let tops: Vec<f32> = tops.into_iter().flatten().collect();
    if tops.is_empty() {
        return region.min.y;
    }
    percentile(&tops, 0.5)
}

fn sample(world: &World, system: ParticleSystemId, step: u32, start: usize) -> SpikeResult<Sample> {
    let view = world.particle_system_view(system)?;
    let positions = view.positions();
    Ok(Sample {
        seconds: step as f32 / STEPS_PER_SECOND as f32,
        source_surface: surface(positions, SOURCE),
        dest_surface: surface(positions, DEST),
        count_d: positions.iter().filter(|p| DEST.contains(**p)).count(),
        count_s: positions.iter().filter(|p| SOURCE.contains(**p)).count(),
        leaks: positions.iter().filter(|p| geometry::is_leak(**p)).count(),
        lost: start.saturating_sub(positions.len()),
    })
}

/// Nearest-rank percentile `fraction` in `0..=1`.
fn percentile(values: &[f32], fraction: f32) -> f32 {
    if values.is_empty() {
        return f32::NAN;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f32::total_cmp);
    let last = sorted.len() - 1;
    let rank = (fraction * last as f32).round() as usize;
    sorted[rank.min(last)]
}

/// Rule A outcome: the `t*` sample index, if reached, and the verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RuleA {
    maybe_t_star: Option<usize>,
    pass: bool,
}

/// First sample at or after 2 s whose source surface is at or below
/// `crest − 0.3`.
fn t_star_index(samples: &[Sample], crest: f32) -> Option<usize> {
    samples
        .iter()
        .enumerate()
        .skip(RULE_A_EARLIEST_SAMPLE)
        .find(|(_, sample)| sample.source_surface <= crest - RULE_A_DROP)
        .map(|(index, _)| index)
}

/// D-21 transfer: over `[t*, t* + 2 s]` every 0.5 s window adds to `D` and
/// the total is at least 10 particles.
fn evaluate_rule_a(samples: &[Sample], crest: f32) -> RuleA {
    let Some(start) = t_star_index(samples, crest) else {
        return RuleA {
            maybe_t_star: None,
            pass: false,
        };
    };
    let end = start + RULE_A_WINDOWS * RULE_A_WINDOW_SAMPLES;
    let Some(window) = samples.get(start..=end) else {
        return RuleA {
            maybe_t_star: Some(start),
            pass: false,
        };
    };
    let each_positive = (0..RULE_A_WINDOWS).all(|index| {
        window[(index + 1) * RULE_A_WINDOW_SAMPLES].count_d
            > window[index * RULE_A_WINDOW_SAMPLES].count_d
    });
    let total = window[window.len() - 1]
        .count_d
        .saturating_sub(window[0].count_d);
    RuleA {
        maybe_t_star: Some(start),
        pass: each_positive && total >= RULE_A_MIN_TRANSFER,
    }
}

/// First sample after `t*` that closes 2 s without a `count_d` change.
fn settle_index(samples: &[Sample], start: usize) -> Option<usize> {
    (start + RULE_B_SETTLE_SAMPLES..samples.len()).find(|&index| {
        samples[index - RULE_B_SETTLE_SAMPLES..=index]
            .iter()
            .all(|sample| sample.count_d == samples[index].count_d)
    })
}

/// D-21 equal levels: `|source − dest| < 0.15` at the end of the run, or at
/// the first settle after `t*`.
fn evaluate_rule_b(samples: &[Sample], crest: f32) -> bool {
    let Some(last) = samples.last() else {
        return false;
    };
    if last.level_difference() < RULE_B_MAX_DIFFERENCE {
        return true;
    }
    let Some(start) = t_star_index(samples, crest) else {
        return false;
    };
    settle_index(samples, start)
        .is_some_and(|index| samples[index].level_difference() < RULE_B_MAX_DIFFERENCE)
}

fn print_sample(label: &str, sample: Sample) {
    println!(
        "SPIKE siphon {label} t {:.2} src {:.3} dst {:.3} count_d {} leaks {} lost {} count_s {}",
        sample.seconds,
        sample.source_surface,
        sample.dest_surface,
        sample.count_d,
        sample.leaks,
        sample.lost,
        sample.count_s
    );
}

fn count_d_at(samples: &[Sample], second: usize) -> String {
    samples
        .get(second * SAMPLES_PER_SECOND)
        .map_or_else(|| "none".to_owned(), |sample| sample.count_d.to_string())
}

/// Runs one configuration for 30 s and prints its SPIKE and RESULT lines.
fn run(candidate: &Candidate) -> SpikeResult<()> {
    let SpikeWorld {
        mut world,
        system,
        start_count,
    } = build_world(candidate)?;
    let mut hook = hook_for(candidate.mechanism);
    let config = StepConfiguration::new(1.0 / STEPS_PER_SECOND as f32, 8, 3)?
        .with_particle_iterations(PARTICLE_ITERATIONS)?;
    let first = sample(&world, system, 0, start_count)?;
    print_sample(&candidate.label, first);
    let mut samples = vec![first];
    let mut busy_ms = 0.0_f64;
    for step in 1..=RUN_STEPS {
        let started = Instant::now();
        hook.on_advance(&mut world, system)?;
        world.step(config, &mut NoDecisionHook, StepLimits::default())?;
        busy_ms += started.elapsed().as_secs_f64() * 1e3;
        if step % SAMPLE_STEPS == 0 {
            let next = sample(&world, system, step, start_count)?;
            print_sample(&candidate.label, next);
            samples.push(next);
        }
    }
    report(
        candidate,
        start_count,
        busy_ms / f64::from(RUN_STEPS),
        &samples,
    );
    Ok(())
}

fn report(candidate: &Candidate, particles: usize, ms_per_step: f64, samples: &[Sample]) {
    let rule_a = evaluate_rule_a(samples, CREST);
    let rule_b = evaluate_rule_b(samples, CREST);
    let leaks_max = samples.iter().map(|s| s.leaks).max().unwrap_or(0);
    let lost_max = samples.iter().map(|s| s.lost).max().unwrap_or(0);
    let diff_end = samples.last().map_or(f32::NAN, |s| s.level_difference());
    let pass = rule_a.pass && rule_b && leaks_max == 0 && lost_max == 0;
    let t_star = rule_a.maybe_t_star.map_or_else(
        || "none".to_owned(),
        |index| format!("{:.2}", samples[index].seconds),
    );
    let verdict = |ok: bool| if ok { "pass" } else { "fail" };
    println!(
        "RESULT siphon {} particles {particles} ms_per_step {ms_per_step:.3} t_star {t_star} \
         rule_a {} rule_b {} diff_end {diff_end:.3} leaks_max {leaks_max} lost_max {lost_max} \
         verdict {}",
        candidate.label,
        verdict(rule_a.pass),
        verdict(rule_b),
        if pass { "PASS" } else { "FAIL" }
    );
    let counts: Vec<String> = REPORT_SECONDS
        .iter()
        .map(|&second| format!("count_d_{second}s {}", count_d_at(samples, second)))
        .collect();
    let transfer_0_2 = samples
        .get(2 * SAMPLES_PER_SECOND)
        .map_or(0, |s| s.count_d as i64 - samples[0].count_d as i64);
    let (src_end, dst_end) = samples
        .last()
        .map_or((f32::NAN, f32::NAN), |s| (s.source_surface, s.dest_surface));
    println!(
        "DETAIL siphon {} {} transfer_0_2s {transfer_0_2} src_end {src_end:.3} dst_end {dst_end:.3}",
        candidate.label,
        counts.join(" ")
    );
}

/// Runs a whole fixed sweep; a failing configuration is recorded and the
/// sweep continues, then any harness error fails the test.
fn run_sweep(candidates: &[Candidate]) -> SpikeResult<()> {
    let mut failures = Vec::new();
    for candidate in candidates {
        if let Err(error) = run(candidate) {
            println!(
                "RESULT siphon {} error {error} verdict FAIL",
                candidate.label
            );
            failures.push(format!("{}: {error}", candidate.label));
        }
    }
    if failures.is_empty() {
        return Ok(());
    }
    Err(failures.join("; ").into())
}

#[test]
#[ignore = "D-20 siphon spike"]
fn candidate_1_plain_water() -> SpikeResult<()> {
    run_sweep(&[Candidate::water("c1-plain-water", Mechanism::Natural)])
}

#[test]
#[ignore = "D-20 siphon spike"]
fn candidate_2_viscous() -> SpikeResult<()> {
    let sweep: Vec<Candidate> = [0.5_f32, 1.0, 1.5]
        .into_iter()
        .map(|strength| Candidate {
            label: format!("c2-viscous-{strength:.1}"),
            flags: ParticleFlags::VISCOUS,
            maybe_viscous_strength: Some(strength),
            mechanism: Mechanism::Natural,
        })
        .collect();
    run_sweep(&sweep)
}

#[test]
#[ignore = "D-20 siphon spike"]
fn candidate_3_tensile_viscous() -> SpikeResult<()> {
    run_sweep(&[Candidate {
        label: "c3-tensile-viscous".to_owned(),
        flags: ParticleFlags::TENSILE | ParticleFlags::VISCOUS,
        maybe_viscous_strength: None,
        mechanism: Mechanism::Natural,
    }])
}

#[test]
#[ignore = "D-20 siphon spike"]
fn candidate_4_fake_atmosphere() -> SpikeResult<()> {
    let sweep: Vec<Candidate> = [0.5_f32, 1.0, 2.0]
        .into_iter()
        .map(|alpha| {
            Candidate::water(
                &format!("c4-fake-atmosphere-{alpha:.1}"),
                Mechanism::FakeAtmosphere { alpha },
            )
        })
        .collect();
    run_sweep(&sweep)
}

#[test]
#[ignore = "D-20 siphon spike"]
fn candidate_5_weighted_lids() -> SpikeResult<()> {
    let sweep: Vec<Candidate> = [0.5_f32, 1.0, 2.0]
        .into_iter()
        .map(|beta| {
            Candidate::water(
                &format!("c5-weighted-lids-{beta:.1}"),
                Mechanism::WeightedLids { beta },
            )
        })
        .collect();
    run_sweep(&sweep)
}

#[test]
#[ignore = "D-20 siphon spike"]
fn candidate_6_tube_pull() -> SpikeResult<()> {
    let sweep: Vec<Candidate> = [0.25_f32, 0.5, 1.0]
        .into_iter()
        .map(|alpha| {
            Candidate::water(
                &format!("c6-tube-pull-{alpha:.2}"),
                Mechanism::TubePull { alpha },
            )
        })
        .collect();
    run_sweep(&sweep)
}

mod tests;
