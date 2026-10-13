//! Time-boxed powder spike for `sand-chute` (Phase 40 D-25).
//!
//! Each configuration runs a `POWDER` block and a plain water block in the
//! same geometry and particle radius and prints player-visible metrics. The
//! output is authoring evidence for the ship-or-drop decision only (D-18).
//!
//! Run with
//! `cargo test -p liquidfun-wasm --release --lib puzzle::powder_spike -- --ignored --nocapture --test-threads=1`.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "spike metrics: particle counts and ranks stay far below 2^23"
)]

use liquidfun::math::Vec2;
use liquidfun::particle::{ParticleColor, ParticleFlags};

use super::first_pour::FIRST_POUR;
use super::goal::GoalRect;
use super::spec::{LevelSpec, LiquidBlock, LiquidMaterial, SystemTuning};
use super::tools::Placements;
use crate::session::SessionCore;

/// Steps per simulated second.
const STEPS_PER_SECOND: u32 = 60;

const FREE_COLOR: ParticleColor = ParticleColor::new(220, 190, 120, 255);
const NEIGHBOUR_COLOR: ParticleColor = ParticleColor::new(200, 90, 160, 255);

/// Counter-clockwise corners of the axis-aligned rectangle `min..max`.
const fn quad(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> [Vec2; 4] {
    [
        Vec2::new(min_x, min_y),
        Vec2::new(max_x, min_y),
        Vec2::new(max_x, max_y),
        Vec2::new(min_x, max_y),
    ]
}

/// One spike world: its walls, liquid and tuning.
struct SpikeWorld {
    radius: f32,
    walls: Vec<[Vec2; 4]>,
    blocks: Vec<LiquidBlock>,
    tuning: SystemTuning,
}

impl SpikeWorld {
    /// A leaked, test-only spec; nothing counts toward a goal that matters.
    fn spec(self) -> &'static LevelSpec {
        let walls: &'static [[Vec2; 4]] = Box::leak(self.walls.into_boxed_slice());
        let blocks: &'static [LiquidBlock] = Box::leak(self.blocks.into_boxed_slice());
        Box::leak(Box::new(LevelSpec {
            particle_radius: self.radius,
            system_tuning: self.tuning,
            walls,
            cup_walls: &[],
            cup_interior: GoalRect::new(Vec2::new(-0.1, -0.1), Vec2::new(0.1, 0.1)),
            liquid_blocks: blocks,
            target_count: 1,
            brim_count: 2,
            full_count: 3,
            tool_budget: &[],
            par_tools: 0,
            ..FIRST_POUR
        }))
    }
}

/// A free block of `material` at the `0.75 × diameter` fill spacing.
fn free_block(min: Vec2, max: Vec2, radius: f32, material: LiquidMaterial) -> LiquidBlock {
    LiquidBlock {
        min,
        max,
        spacing: 1.5 * radius,
        color: FREE_COLOR,
        counted: true,
        material,
    }
}

/// Particle positions split into the free group and any viscous or elastic
/// neighbour.
struct Snapshot {
    free: Vec<Vec2>,
    neighbour: Vec<Vec2>,
}

fn snapshot(core: &SessionCore) -> Snapshot {
    core.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("live particle system has a view");
        let neighbour_flags = ParticleFlags::VISCOUS | ParticleFlags::ELASTIC;
        let (neighbour, free): (Vec<_>, Vec<_>) = view
            .positions()
            .iter()
            .zip(view.flags())
            .partition(|(_, flags)| flags.intersects(neighbour_flags));
        Snapshot {
            free: free.into_iter().map(|(position, _)| *position).collect(),
            neighbour: neighbour
                .into_iter()
                .map(|(position, _)| *position)
                .collect(),
        }
    })
}

fn advance_to(core: &mut SessionCore, step: u32) {
    while core.step_index() < step {
        core.advance(1).expect("spike world steps");
    }
}

fn mean(values: &[f32]) -> f32 {
    if values.is_empty() {
        return f32::NAN;
    }
    values.iter().sum::<f32>() / values.len() as f32
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

fn xs(points: &[Vec2]) -> Vec<f32> {
    points.iter().map(|point| point.x).collect()
}

fn ys(points: &[Vec2]) -> Vec<f32> {
    points.iter().map(|point| point.y).collect()
}

/// Runs `world` with a water free block and a powder free block and returns
/// each run's metrics row per sample second.
fn compare(
    label: &str,
    seconds: &[u32],
    build: impl Fn(LiquidMaterial) -> SpikeWorld,
    metrics: impl Fn(&Snapshot) -> Vec<(&'static str, f32)>,
) {
    let mut rows = Vec::new();
    let mut particles = Vec::new();
    for material in [LiquidMaterial::Water, LiquidMaterial::Powder] {
        let spec = build(material).spec();
        let mut core =
            SessionCore::create_puzzle_from_spec(spec, Placements::empty()).expect("spike builds");
        particles.push(core.particle_count());
        let mut per_second = Vec::new();
        for &second in seconds {
            advance_to(&mut core, second * STEPS_PER_SECOND);
            per_second.push(metrics(&snapshot(&core)));
        }
        rows.push(per_second);
    }
    println!(
        "SPIKE {label} particles water {} powder {}",
        particles[0], particles[1]
    );
    for (index, second) in seconds.iter().enumerate() {
        for ((name, water), (_, powder)) in rows[0][index].iter().zip(&rows[1][index]) {
            let delta = if water.abs() > f32::EPSILON {
                100.0 * (powder - water) / water.abs()
            } else {
                f32::NAN
            };
            println!(
                "SPIKE {label} t {second}s {name} water {water:.3} powder {powder:.3} delta {delta:+.1}%"
            );
        }
    }
}

/// A reservoir over a centred slit `slit` wide, inside an 8 m box.
fn slit_world(radius: f32, slit: f32, material: LiquidMaterial) -> SpikeWorld {
    let half = slit / 2.0;
    SpikeWorld {
        radius,
        walls: vec![
            quad(-4.0, -0.5, 4.0, 0.0),
            quad(-4.2, 0.0, -4.0, 8.0),
            quad(4.0, 0.0, 4.2, 8.0),
            quad(-1.8, 3.0, -1.6, 6.5),
            quad(1.6, 3.0, 1.8, 6.5),
            quad(-1.6, 3.0, -half, 3.2),
            quad(half, 3.0, 1.6, 3.2),
        ],
        blocks: vec![free_block(
            Vec2::new(-1.56, 3.24),
            Vec2::new(1.56, 5.4),
            radius,
            material,
        )],
        tuning: SystemTuning::NONE,
    }
}

/// Drained count (below the reservoir floor) and the deposit's height and
/// width on the box floor.
fn slit_metrics(snapshot: &Snapshot) -> Vec<(&'static str, f32)> {
    let drained: Vec<Vec2> = snapshot
        .free
        .iter()
        .copied()
        .filter(|point| point.y < 3.0)
        .collect();
    let drained_xs = xs(&drained);
    vec![
        ("drained", drained.len() as f32),
        ("deposit_p95_height", percentile(&ys(&drained), 0.95)),
        (
            "deposit_p5_p95_width",
            percentile(&drained_xs, 0.95) - percentile(&drained_xs, 0.05),
        ),
    ]
}

/// A 15° slab ramp falling from (-3, 2) to (3, 0.392) above a long run-out.
fn ramp_world(radius: f32, material: LiquidMaterial) -> SpikeWorld {
    SpikeWorld {
        radius,
        walls: vec![
            quad(-3.4, -0.5, 8.2, 0.0),
            quad(-3.4, 0.0, -3.2, 6.0),
            quad(8.0, 0.0, 8.2, 6.0),
            // tan(15°) = 0.26795: the top face drops 1.608 m over 6 m.
            [
                Vec2::new(-3.2, 1.8),
                Vec2::new(3.0, 0.1923),
                Vec2::new(3.0, 0.3923),
                Vec2::new(-3.2, 2.0),
            ],
        ],
        blocks: vec![free_block(
            Vec2::new(-3.1, 2.1),
            Vec2::new(-1.9, 3.1),
            radius,
            material,
        )],
        tuning: SystemTuning::NONE,
    }
}

fn ramp_metrics(snapshot: &Snapshot) -> Vec<(&'static str, f32)> {
    let free_xs = xs(&snapshot.free);
    vec![
        ("mean_x", mean(&free_xs)),
        ("front_p95_x", percentile(&free_xs, 0.95)),
        ("tail_p5_x", percentile(&free_xs, 0.05)),
    ]
}

/// A viscous (strength 1.5) block with the free block beside it on its
/// right, both collapsing to the right across a flat floor.
fn viscous_neighbour_world(material: LiquidMaterial) -> SpikeWorld {
    let radius = 0.04;
    SpikeWorld {
        radius,
        walls: vec![
            quad(-3.2, -0.5, 5.2, 0.0),
            quad(-3.2, 0.0, -3.0, 4.0),
            quad(5.0, 0.0, 5.2, 4.0),
        ],
        blocks: vec![
            LiquidBlock {
                color: NEIGHBOUR_COLOR,
                ..free_block(
                    Vec2::new(-2.96, 0.04),
                    Vec2::new(-1.52, 1.5),
                    radius,
                    LiquidMaterial::Viscous,
                )
            },
            free_block(
                Vec2::new(-1.46, 0.04),
                Vec2::new(-0.02, 1.5),
                radius,
                material,
            ),
        ],
        tuning: SystemTuning {
            maybe_viscous_strength: Some(1.5),
            ..SystemTuning::NONE
        },
    }
}

/// Free-group travel and height, the neighbour's travel, and how many free
/// particles end up over or inside the neighbour's footprint.
fn neighbour_metrics(snapshot: &Snapshot) -> Vec<(&'static str, f32)> {
    let free_xs = xs(&snapshot.free);
    let neighbour_xs = xs(&snapshot.neighbour);
    let neighbour_front = percentile(&neighbour_xs, 0.95);
    let over_neighbour = free_xs.iter().filter(|x| **x < neighbour_front).count();
    vec![
        ("free_mean_x", mean(&free_xs)),
        ("free_front_p95_x", percentile(&free_xs, 0.95)),
        ("free_mean_y", mean(&ys(&snapshot.free))),
        ("neighbour_mean_x", mean(&neighbour_xs)),
        ("free_behind_neighbour_front", over_neighbour as f32),
    ]
}

/// An uncounted solid jelly slab on the floor with the free block dropped
/// onto it from 1.6 m.
fn jelly_neighbour_world(material: LiquidMaterial) -> SpikeWorld {
    let radius = 0.04;
    SpikeWorld {
        radius,
        walls: vec![
            quad(-4.2, -0.5, 4.2, 0.0),
            quad(-4.2, 0.0, -4.0, 6.0),
            quad(4.0, 0.0, 4.2, 6.0),
            // A shallow cradle that keeps the pad from sliding.
            quad(-1.3, 0.0, -1.1, 0.3),
            quad(1.1, 0.0, 1.3, 0.3),
        ],
        blocks: vec![
            LiquidBlock {
                color: NEIGHBOUR_COLOR,
                counted: false,
                ..free_block(
                    Vec2::new(-1.06, 0.04),
                    Vec2::new(1.06, 0.4),
                    radius,
                    LiquidMaterial::Elastic,
                )
            },
            free_block(Vec2::new(-0.5, 2.0), Vec2::new(0.5, 3.0), radius, material),
        ],
        tuning: SystemTuning::NONE,
    }
}

/// Free particles inside the pad's start box, the splash width, the highest
/// free particle and the pad's centroid height.
fn jelly_metrics(snapshot: &Snapshot) -> Vec<(&'static str, f32)> {
    let pad_box = GoalRect::new(Vec2::new(-1.06, 0.0), Vec2::new(1.06, 0.4));
    let soaked = snapshot
        .free
        .iter()
        .filter(|point| pad_box.contains_strictly(**point))
        .count();
    let free_xs = xs(&snapshot.free);
    vec![
        ("free_in_pad_box", soaked as f32),
        (
            "free_p5_p95_width",
            percentile(&free_xs, 0.95) - percentile(&free_xs, 0.05),
        ),
        ("free_p99_height", percentile(&ys(&snapshot.free), 0.99)),
        ("pad_mean_y", mean(&ys(&snapshot.neighbour))),
    ]
}

/// A block dropped from 1 m onto a static plank sloped at -30° (top face
/// from (-0.866, 2.0) to (0.866, 1.0)).
fn plank_world(radius: f32, material: LiquidMaterial) -> SpikeWorld {
    SpikeWorld {
        radius,
        walls: vec![
            quad(-5.2, -0.5, 5.2, 0.0),
            quad(-5.2, 0.0, -5.0, 6.0),
            quad(5.0, 0.0, 5.2, 6.0),
            [
                Vec2::new(-0.866, 1.85),
                Vec2::new(0.866, 0.85),
                Vec2::new(0.866, 1.0),
                Vec2::new(-0.866, 2.0),
            ],
        ],
        blocks: vec![free_block(
            Vec2::new(-0.6, 3.0),
            Vec2::new(0.6, 4.2),
            radius,
            material,
        )],
        tuning: SystemTuning::NONE,
    }
}

/// Lateral spread of the splash: x standard deviation and p5–p95 width,
/// plus the airborne count above the plank's top.
fn plank_metrics(snapshot: &Snapshot) -> Vec<(&'static str, f32)> {
    let free_xs = xs(&snapshot.free);
    let centre = mean(&free_xs);
    let variance = free_xs
        .iter()
        .map(|x| (x - centre) * (x - centre))
        .sum::<f32>()
        / free_xs.len().max(1) as f32;
    let airborne = snapshot.free.iter().filter(|point| point.y > 2.1).count();
    vec![
        ("mean_x", centre),
        ("x_std_dev", variance.sqrt()),
        (
            "p5_p95_width",
            percentile(&free_xs, 0.95) - percentile(&free_xs, 0.05),
        ),
        ("above_plank_top", airborne as f32),
    ]
}

#[test]
#[ignore = "D-25 powder spike"]
fn spike_coarse_slit() {
    for radius in [0.06_f32, 0.08] {
        // A slit 1.75 particle diameters wide.
        let slit = 3.5 * radius;
        compare(
            &format!("a-slit r{radius} slit{slit:.2}"),
            &[2, 4, 8],
            |material| slit_world(radius, slit, material),
            slit_metrics,
        );
    }
}

#[test]
#[ignore = "D-25 powder spike"]
fn spike_coarse_ramp() {
    for radius in [0.06_f32, 0.08] {
        compare(
            &format!("a-ramp15 r{radius}"),
            &[1, 2, 4],
            |material| ramp_world(radius, material),
            ramp_metrics,
        );
    }
}

#[test]
#[ignore = "D-25 powder spike"]
fn spike_viscous_neighbour() {
    compare(
        "b-viscous-neighbour r0.04",
        &[1, 2, 4],
        viscous_neighbour_world,
        neighbour_metrics,
    );
}

#[test]
#[ignore = "D-25 powder spike"]
fn spike_jelly_neighbour() {
    compare(
        "b-jelly-neighbour r0.04",
        &[1, 2, 4],
        jelly_neighbour_world,
        jelly_metrics,
    );
}

#[test]
#[ignore = "D-25 powder spike"]
fn spike_plank_scatter() {
    for radius in [0.04_f32, 0.08] {
        compare(
            &format!("c-plank-scatter r{radius}"),
            &[1, 2],
            |material| plank_world(radius, material),
            plank_metrics,
        );
    }
}

/// Robustness check for the one candidate past 25% (coarse grains through a
/// slit): the same reservoir at radius 0.08 over a range of slit widths and
/// a slit shifted off centre.
#[test]
#[ignore = "D-25 powder spike"]
fn spike_coarse_slit_width_sweep() {
    let radius = 0.08_f32;
    for slit in [0.20_f32, 0.24, 0.28, 0.32, 0.40, 0.56] {
        compare(
            &format!("d-slit-sweep r{radius} slit{slit:.2}"),
            &[4, 8, 12],
            |material| slit_world(radius, slit, material),
            |snapshot| slit_metrics(snapshot).into_iter().take(1).collect(),
        );
    }
    compare(
        "d-slit-sweep r0.08 slit0.28 shifted+0.05",
        &[4, 8, 12],
        |material| shifted_slit_world(radius, 0.28, 0.05, material),
        |snapshot| slit_metrics(snapshot).into_iter().take(1).collect(),
    );
}

/// `slit_world` with the slit centred at `offset` instead of 0.
fn shifted_slit_world(radius: f32, slit: f32, offset: f32, material: LiquidMaterial) -> SpikeWorld {
    let mut world = slit_world(radius, slit, material);
    let half = slit / 2.0;
    world.walls[5] = quad(-1.6, 3.0, offset - half, 3.2);
    world.walls[6] = quad(offset + half, 3.0, 1.6, 3.2);
    world
}
