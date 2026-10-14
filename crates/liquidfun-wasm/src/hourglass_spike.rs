//! Time-boxed sand hourglass feasibility spike (Phase 45 D-14..D-18,
//! D-23..D-27, PREP-03).
//!
//! Builds the fixed point-symmetric glass from `geometry`, runs the water and
//! powder controls and the three D-24 sand rules for 30 s at gravity 10,
//! judges each by the D-25 rules fixed in `45-HOURGLASS-SPIKE.md` before the
//! first measurement (`metrics`), then measures the D-26 flip cost. The output
//! is authoring evidence for the go or drop decision only (D-16).
//!
//! Run with
//! `cargo test -p liquidfun-wasm --release --lib hourglass_spike -- --ignored --nocapture --test-threads=1 2>&1 | tee target/phase45/hourglass-spike.log`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    reason = "spike metrics: particle counts, bins and ranks stay far below 2^23"
)]

mod flip;
mod geometry;
mod hooks;
mod metrics;

use std::collections::BTreeMap;
use std::error::Error;
use std::sync::OnceLock;
use std::time::Instant;

use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleGroupDestination, ParticleGroupFlags, ParticleGroupRecipe,
    ParticleGroupSource,
};
use liquidfun::{
    BodyDef, NoDecisionHook, ParticleFlags, ParticleSystemDef, ParticleSystemId, StepConfiguration,
    StepLimits, World,
};

use self::flip::measure_flip;
use self::geometry::RADIUS;
use self::hooks::{SpikeHook, hook_for};
use self::metrics::{JamTracker, Pile, nearest_rank, window_verdict};
use crate::scene::attach_basin_fixture;

type SpikeResult<T> = Result<T, Box<dyn Error>>;

const STEPS_PER_SECOND: u32 = 60;
/// One sample every 0.25 s.
const SAMPLE_STEPS: u32 = 15;
/// 30 s at 60 Hz before the flip.
const RUN_STEPS: u32 = 1_800;
/// 10 s after the flip.
const POST_FLIP_STEPS: u32 = 600;
/// Post-flip progress is printed once per second.
const POST_SAMPLE_STEPS: u32 = 60;
/// 25-30 s inclusive at 0.25 s.
const WINDOW_SAMPLES: usize = 21;
const PARTICLE_ITERATIONS: u32 = 2;
const GRAVITY: f32 = 10.0;
/// Gravity used to prove that a gravity change releases frozen grains.
const RELEASE_GRAVITY: f32 = 10.5;
const MAXIMUM_COUNT: usize = 4_000;
const PARTICLE_DENSITY: f32 = 1.0;
const SAND_COLOR: ParticleColor = ParticleColor::new(222, 184, 108, 255);
/// Post-flip pass mark (informational, SAND-02).
const RERUN_FRACTION: f32 = 0.9;
/// Order check tolerance for rebuilt member positions.
const ORDER_TOLERANCE: f32 = 1e-6;

static WATER_MEDIAN: OnceLock<f32> = OnceLock::new();

/// Sand rule layered on the particle material.
#[derive(Clone, Copy, Debug)]
enum Rule {
    Plain,
    StickSlip { v_slip: f32, k: f32 },
    Freeze { per_step: usize },
    Clumps,
}

/// One spike configuration.
#[derive(Clone, Debug)]
struct Config {
    label: String,
    flags: ParticleFlags,
    rule: Rule,
}

impl Config {
    fn new(label: &str, flags: ParticleFlags, rule: Rule) -> Self {
        Self {
            label: label.to_owned(),
            flags,
            rule,
        }
    }

    fn is_water_control(&self) -> bool {
        self.flags.is_empty() && matches!(self.rule, Rule::Plain)
    }

    fn group_flags(&self) -> ParticleGroupFlags {
        if matches!(self.rule, Rule::Clumps) {
            ParticleGroupFlags::RIGID
        } else {
            ParticleGroupFlags::empty()
        }
    }
}

fn sand_recipe(config: &Config, positions: Vec<Vec2>) -> SpikeResult<ParticleGroupRecipe<()>> {
    Ok(ParticleGroupRecipe::new(
        ParticleGroupSource::positions(positions)?,
        ParticleGroupDestination::New,
    )
    .with_particle_flags(config.flags)
    .with_group_flags(config.group_flags())
    .with_color(SAND_COLOR)
    .with_transform(Transform::IDENTITY)?)
}

fn build_world(config: &Config) -> SpikeResult<(World, ParticleSystemId, usize)> {
    let mut world = World::new()?;
    world.set_gravity(Vec2::new(0.0, -GRAVITY))?;
    let ground = world.create_body(&BodyDef::default())?;
    for quad in geometry::wall_quads() {
        attach_basin_fixture(&mut world, ground, &quad)
            .map_err(|error| format!("wall fixture: {error:?}"))?;
    }
    let definition = ParticleSystemDef::default()
        .with_density(PARTICLE_DENSITY)?
        .with_radius(RADIUS)?
        .with_maximum_count(MAXIMUM_COUNT)?;
    let system = world.create_particle_system_with_def(&definition)?;
    let fill = geometry::sand_fill();
    let batches: Vec<Vec<Vec2>> = if matches!(config.rule, Rule::Clumps) {
        let mut clumps: BTreeMap<(i32, i32), Vec<Vec2>> = BTreeMap::new();
        for point in &fill {
            clumps.entry(point.clump).or_default().push(point.position);
        }
        clumps.into_values().collect()
    } else {
        vec![fill.iter().map(|point| point.position).collect()]
    };
    let full_clumps = batches.iter().filter(|batch| batch.len() == 4).count();
    println!(
        "BUILD hourglass {} particles {} groups {} full_clumps {full_clumps} partial_clumps {}",
        config.label,
        fill.len(),
        batches.len(),
        batches.len() - full_clumps
    );
    for batch in batches {
        world.create_particle_group(system, &sand_recipe(config, batch)?)?;
    }
    Ok((world, system, fill.len()))
}

/// `(upper, lower)` = particles above and below the neck.
fn counts(positions: &[Vec2]) -> (usize, usize) {
    let upper = positions.iter().filter(|position| position.y > 0.0).count();
    let lower = positions.iter().filter(|position| position.y < 0.0).count();
    (upper, lower)
}

/// `(frozen count, max frozen speed)` over `WALL` particles.
fn frozen(world: &World, system: ParticleSystemId) -> SpikeResult<(usize, f32)> {
    let view = world.particle_system_view(system)?;
    Ok(view
        .flags()
        .iter()
        .zip(view.velocities())
        .filter(|(flags, _)| flags.contains(ParticleFlags::WALL))
        .fold((0, 0.0_f32), |(count, speed), (_, velocity)| {
            (count + 1, speed.max(velocity.length()))
        }))
}

/// One 0.25 s measurement.
#[derive(Clone, Copy, Debug)]
struct Sample {
    seconds: f32,
    upper: usize,
    lower: usize,
    pile: Pile,
    frozen: usize,
}

fn sample(world: &World, system: ParticleSystemId, step: u32) -> SpikeResult<Sample> {
    let view = world.particle_system_view(system)?;
    let (upper, lower) = counts(view.positions());
    let frozen = view
        .flags()
        .iter()
        .filter(|flags| flags.contains(ParticleFlags::WALL))
        .count();
    Ok(Sample {
        seconds: step as f32 / STEPS_PER_SECOND as f32,
        upper,
        lower,
        pile: metrics::pile(view.positions(), view.velocities()),
        frozen,
    })
}

fn print_sample(label: &str, sample: Sample) {
    println!(
        "SPIKE hourglass {label} t {:.2} upper {} lower {} slope {:.2} literal {:.2} side {:.2} \
         frozen {}",
        sample.seconds,
        sample.upper,
        sample.lower,
        sample.pile.slope,
        sample.pile.literal,
        sample.pile.side,
        sample.frozen
    );
}

/// A finished 30 s run, still live for the release proof and the flip.
struct Run {
    world: World,
    system: ParticleSystemId,
    hook: Box<dyn SpikeHook>,
    step_config: StepConfiguration,
    particles: usize,
    ms_per_step: f64,
    samples: Vec<Sample>,
    maybe_jam_step: Option<u32>,
    max_frozen_speed: f32,
}

fn run_config(config: &Config, verbose: bool) -> SpikeResult<Run> {
    let (mut world, system, particles) = build_world(config)?;
    let mut hook = hook_for(config);
    let step_config = StepConfiguration::new(1.0 / STEPS_PER_SECOND as f32, 8, 3)?
        .with_particle_iterations(PARTICLE_ITERATIONS)?;
    let mut jam = JamTracker::default();
    let first = sample(&world, system, 0)?;
    jam.observe(0, first.upper, first.lower);
    if verbose {
        print_sample(&config.label, first);
    }
    let mut samples = vec![first];
    let mut busy_ms = 0.0_f64;
    let mut max_frozen_speed = 0.0_f32;
    for step in 1..=RUN_STEPS {
        let started = Instant::now();
        hook.on_advance(&mut world, system)?;
        world.step(step_config, &mut NoDecisionHook, StepLimits::default())?;
        busy_ms += started.elapsed().as_secs_f64() * 1e3;
        let (upper, lower) = counts(world.particle_system_view(system)?.positions());
        jam.observe(step, upper, lower);
        max_frozen_speed = max_frozen_speed.max(frozen(&world, system)?.1);
        if step % SAMPLE_STEPS == 0 {
            let next = sample(&world, system, step)?;
            if verbose {
                print_sample(&config.label, next);
            }
            samples.push(next);
        }
    }
    Ok(Run {
        world,
        system,
        hook,
        step_config,
        particles,
        ms_per_step: busy_ms / f64::from(RUN_STEPS),
        samples,
        maybe_jam_step: jam.maybe_jam_step(),
        max_frozen_speed,
    })
}

fn window(samples: &[Sample]) -> SpikeResult<&[Sample]> {
    let start = samples
        .len()
        .checked_sub(WINDOW_SAMPLES)
        .ok_or("run shorter than the 5 s window")?;
    Ok(&samples[start..])
}

fn slope_median(run: &Run) -> SpikeResult<f32> {
    let slopes: Vec<f32> = window(&run.samples)?
        .iter()
        .map(|sample| sample.pile.slope)
        .collect();
    Ok(nearest_rank(&slopes, 0.5))
}

/// Median water-control slope over the window, computed once.
fn water_median() -> SpikeResult<f32> {
    if let Some(median) = WATER_MEDIAN.get() {
        return Ok(*median);
    }
    println!("NOTE hourglass water control computed silently for the margin");
    let run = run_config(&water(), false)?;
    let median = slope_median(&run)?;
    Ok(*WATER_MEDIAN.get_or_init(|| median))
}

fn verdict(pass: bool) -> &'static str {
    if pass { "PASS" } else { "FAIL" }
}

fn report(config: &Config, run: &Run, water: f32) -> SpikeResult<()> {
    let span = window(&run.samples)?;
    let slopes: Vec<f32> = span.iter().map(|s| s.pile.slope).collect();
    let literals: Vec<f32> = span.iter().map(|s| s.pile.literal).collect();
    let sides: Vec<f32> = span.iter().map(|s| s.pile.side).collect();
    let jammed = run.maybe_jam_step.is_some();
    let edge = window_verdict(&slopes, jammed, Some(water));
    let literal = window_verdict(&literals, jammed, None);
    let jam = run.maybe_jam_step.map_or_else(
        || "no".to_owned(),
        |step| format!("yes@{:.2}", step as f32 / STEPS_PER_SECOND as f32),
    );
    println!(
        "RESULT hourglass {} particles {} ms_per_step {:.3} median_slope {:.2} min {:.2} \
         max {:.2} jam {jam} water_margin {:.2} verdict {}",
        config.label,
        run.particles,
        run.ms_per_step,
        edge.median,
        edge.min,
        edge.max,
        edge.median - water,
        verdict(edge.pass)
    );
    let last = span.last().ok_or("empty window")?;
    let stats = run.hook.freeze_stats();
    let per_step = if stats.steps == 0 {
        0.0
    } else {
        stats.total as f32 / stats.steps as f32
    };
    println!(
        "DETAIL hourglass {} literal_median {:.2} literal_min {:.2} literal_max {:.2} \
         literal_verdict {} side_median {:.2} upper_end {} lower_end {} frozen_end {} \
         freezes_total {} freezes_per_step {per_step:.3} freezes_max_step {} \
         max_frozen_speed {:.6}",
        config.label,
        literal.median,
        literal.min,
        literal.max,
        verdict(literal.pass),
        nearest_rank(&sides, 0.5),
        last.upper,
        last.lower,
        last.frozen,
        stats.total,
        stats.max_per_step,
        run.max_frozen_speed
    );
    Ok(())
}

/// Candidate 2 release proof: a gravity change releases every frozen grain.
fn release_proof(config: &Config, run: &mut Run) -> SpikeResult<()> {
    run.world.set_gravity(Vec2::new(0.0, -RELEASE_GRAVITY))?;
    let released = run.hook.release_all(&mut run.world, run.system)?;
    let (left, _) = frozen(&run.world, run.system)?;
    if left != 0 {
        return Err(format!("{left} grains still frozen after the gravity change").into());
    }
    run.world.set_gravity(Vec2::new(0.0, -GRAVITY))?;
    let released_again = run.hook.release_all(&mut run.world, run.system)?;
    println!(
        "RELEASE hourglass {} released {released} frozen_after 0 restore_released {released_again}",
        config.label
    );
    Ok(())
}

fn water() -> Config {
    Config::new("w-water", ParticleFlags::empty(), Rule::Plain)
}

/// Runs one configuration end to end: 30 s, verdict, release proof, flip.
fn run_full(config: &Config) -> SpikeResult<()> {
    let mut run = run_config(config, true)?;
    let water = if config.is_water_control() {
        let own = slope_median(&run)?;
        *WATER_MEDIAN.get_or_init(|| own)
    } else {
        water_median()?
    };
    report(config, &run, water)?;
    if matches!(config.rule, Rule::Freeze { .. }) {
        release_proof(config, &mut run)?;
    }
    measure_flip(config, &mut run)
}

/// Runs a fixed sweep; a failing configuration is recorded and the sweep
/// continues, then any harness error fails the test.
fn run_sweep(configs: &[Config]) -> SpikeResult<()> {
    let mut failures = Vec::new();
    for config in configs {
        if let Err(error) = run_full(config) {
            println!(
                "RESULT hourglass {} error {error} verdict FAIL",
                config.label
            );
            failures.push(format!("{}: {error}", config.label));
        }
    }
    if failures.is_empty() {
        return Ok(());
    }
    Err(failures.join("; ").into())
}

#[test]
#[ignore = "D-24 hourglass spike"]
fn c0_control_a_water() -> SpikeResult<()> {
    run_sweep(&[water()])
}

#[test]
#[ignore = "D-24 hourglass spike"]
fn c0_control_b_powder() -> SpikeResult<()> {
    run_sweep(&[Config::new("p-powder", ParticleFlags::POWDER, Rule::Plain)])
}

#[test]
#[ignore = "D-24 hourglass spike"]
fn c1_stick_slip() -> SpikeResult<()> {
    let sweep: Vec<Config> = [(0.2_f32, 0.5_f32), (0.2, 1.0), (0.5, 0.5), (0.5, 1.0)]
        .into_iter()
        .map(|(v_slip, k)| {
            Config::new(
                &format!("c1-stick-slip-v{v_slip:.1}-k{k:.1}"),
                ParticleFlags::POWDER,
                Rule::StickSlip { v_slip, k },
            )
        })
        .collect();
    run_sweep(&sweep)
}

#[test]
#[ignore = "D-24 hourglass spike"]
fn c2_freeze_to_wall() -> SpikeResult<()> {
    let sweep: Vec<Config> = [8_usize, 32]
        .into_iter()
        .map(|per_step| {
            Config::new(
                &format!("c2-freeze-k{per_step}"),
                ParticleFlags::POWDER,
                Rule::Freeze { per_step },
            )
        })
        .collect();
    run_sweep(&sweep)
}

#[test]
#[ignore = "D-24 hourglass spike"]
fn c3_rigid_clumps() -> SpikeResult<()> {
    run_sweep(&[Config::new(
        "c3-rigid-clumps",
        ParticleFlags::empty(),
        Rule::Clumps,
    )])
}

#[test]
fn sand_fill_fits_the_budget_and_starts_in_the_upper_bulb() {
    // Arrange
    let walls = geometry::wall_quads();

    // Act
    let fill = geometry::sand_fill();

    // Assert
    assert!((400..=3_000).contains(&fill.len()), "{}", fill.len());
    assert!(fill.iter().all(|point| point.position.y > 0.0));
    assert!(fill.iter().all(|point| {
        walls
            .iter()
            .all(|quad| geometry::distance_to_quad(point.position, quad) >= RADIUS)
    }));
}

#[test]
fn reflected_walls_build_a_world() -> SpikeResult<()> {
    // Arrange
    let config = water();

    // Act
    let (world, system, particles) = build_world(&config)?;

    // Assert
    assert_eq!(
        world.particle_system_view(system)?.particle_ids().len(),
        particles
    );
    Ok(())
}
