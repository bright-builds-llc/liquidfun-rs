//! Test-only helpers shared by the Rayleigh–Taylor unit, spike, force-lane
//! and flow tests: the fixed step, the layer split, the D-07 finger,
//! overturn, speed, containment and rain metrics (49-01-PLAN "Metrics", as
//! amended by 49-CONTEXT D-06a: depths are measured from the settled
//! interface), the S4 stability recorder and the session drivers.
//!
//! Every metric reads heights along `frame.up` and bins along
//! `frame.across`, so a tilted-gravity test can reuse them. The finger bins
//! cover the inner tank in the gravity-down frame.

use liquidfun::math::Vec2;
use liquidfun::{
    NoDecisionHook, ParticleGroupId, ParticleSystemId, StepConfiguration, StepLimits, World,
};

use super::geometry::{FLOOR_TOP_Y, LID_BOTTOM_Y, TANK_HALF_WIDTH, TANK_INNER_HEIGHT};
use super::{RayleighTaylorParts, TUNING};
use crate::scene::particle_force::GravityFrame;
use crate::scene::{SceneHooks, SceneId, particle_iterations};
use crate::session::{MAX_ADVANCE_STEPS, SessionCore};

/// How far below the settled interface a finger must reach, as a share of
/// the inner tank height (D-06a).
const DEPTH_FRACTION: f32 = 0.2;
/// The fluid top is read at this height quantile, so a lone splash is ignored.
const FLUID_TOP_QUANTILE: f32 = 0.99;
/// Width of one finger bin across the tank: two particle diameters at r 0.04.
pub(super) const FINGER_BIN_WIDTH: f32 = 0.16;
/// Bins across the inner tank; the last partial strip folds into the last bin.
pub(super) const FINGER_BIN_COUNT: usize = 17;
/// Least heavy particles below the depth line for a bin to count as past it,
/// so single raining particles never make a finger (D-07).
pub(super) const MIN_FINGER_PARTICLES: usize = 4;
/// Largest particle speed a bounded run may reach at gravity 10 (S4).
pub(super) const MAX_SPEED: f32 = 6.0;
/// Speed and containment samples are taken every this many steps (S4).
pub(super) const STABILITY_SAMPLE_EVERY: u32 = 30;
/// Last step of the early speed window (S4).
const EARLY_WINDOW_END: u32 = 1_200;
/// First step of the late speed window (S4).
const LATE_WINDOW_START: u32 = 3_000;
/// A heavy particle below the depth line with fewer neighbors than this is rain.
const MIN_RAIN_NEIGHBORS: usize = 2;
/// Neighbor reach for the rain diagnostic, in particle radii.
const RAIN_NEIGHBOR_RADII: f32 = 3.0;

const _: () = assert!(
    FINGER_BIN_WIDTH * 17.0 <= 2.0 * TANK_HALF_WIDTH
        && FINGER_BIN_WIDTH * 18.0 > 2.0 * TANK_HALF_WIDTH
);

/// The session's fixed step: 1/60 s, 8 velocity and 3 position iterations,
/// and the scene's particle iterations.
pub(super) fn step_configuration() -> StepConfiguration {
    StepConfiguration::new(1.0 / 60.0, 8, 3)
        .expect("valid step")
        .with_particle_iterations(particle_iterations(SceneId::RayleighTaylor))
        .expect("particle iterations")
}

/// The 20% depth line: one fifth of the inner tank height below the
/// interface measured at the end of the settle window (D-06a).
pub(super) fn depth_line(settled_interface: f32) -> f32 {
    settled_interface - DEPTH_FRACTION * TANK_INNER_HEIGHT
}

/// The gravity-down frame every spike run uses.
pub(super) fn down_frame() -> GravityFrame {
    GravityFrame::from_gravity(Vec2::new(0.0, -10.0)).expect("gravity defines a frame")
}

/// Particle lanes split by group.
#[derive(Debug, Clone, Default)]
pub(super) struct Layers {
    pub(super) heavy: Vec<Vec2>,
    pub(super) light: Vec<Vec2>,
    pub(super) velocities: Vec<Vec2>,
}

/// Splits the system's positions by `heavy_group`, keeping every velocity.
pub(super) fn split_layers(
    world: &World,
    system: ParticleSystemId,
    heavy_group: ParticleGroupId,
) -> Layers {
    let view = world.particle_system_view(system).expect("liquid");
    let mut layers = Layers {
        velocities: view.velocities().to_vec(),
        ..Layers::default()
    };
    for (position, group) in view.positions().iter().zip(view.group_ids()) {
        if *group == Some(heavy_group) {
            layers.heavy.push(*position);
        } else {
            layers.light.push(*position);
        }
    }
    layers
}

/// The finger bin holding `position`, clamped into the bin range.
fn bin_index(position: Vec2, frame: GravityFrame) -> usize {
    let offset = ((position.dot(frame.across) + TANK_HALF_WIDTH) / FINGER_BIN_WIDTH).floor();
    let clamped = offset.clamp(0.0, 16.0);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "clamped is a whole number inside 0..FINGER_BIN_COUNT"
    )]
    let index = clamped as usize;
    index
}

/// Every particle height along `up`, sorted ascending.
fn sorted_heights(layers: &Layers, frame: GravityFrame) -> Vec<f32> {
    let mut heights: Vec<f32> = layers
        .light
        .iter()
        .chain(&layers.heavy)
        .map(|position| position.dot(frame.up))
        .collect();
    heights.sort_by(f32::total_cmp);
    heights
}

/// Mean heavy/light interface height along `up`: the height below which as
/// many particles lie as the light layer holds. Uniform compression moves
/// it with the column, and a ripple averages out.
pub(super) fn interface_height(layers: &Layers, frame: GravityFrame) -> f32 {
    let heights = sorted_heights(layers, frame);
    let Some(last) = heights.len().checked_sub(1) else {
        return 0.0;
    };
    heights[layers.light.len().min(last)]
}

/// Fluid top along `up`: the 99th-percentile particle height (S0).
pub(super) fn fluid_top(layers: &Layers, frame: GravityFrame) -> f32 {
    let heights = sorted_heights(layers, frame);
    let Some(last) = heights.len().checked_sub(1) else {
        return 0.0;
    };
    let last_height = f32::from(u16::try_from(last).expect("count fits u16"));
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the quantile index is a whole number inside 0..=last"
    )]
    let index = (last_height * FLUID_TOP_QUANTILE).floor() as usize;
    heights[index.min(last)]
}

/// Heavy particles below `depth_line` in each finger bin.
fn past_counts(layers: &Layers, frame: GravityFrame, depth_line: f32) -> [usize; FINGER_BIN_COUNT] {
    let mut counts = [0; FINGER_BIN_COUNT];
    for position in &layers.heavy {
        if position.dot(frame.up) < depth_line {
            counts[bin_index(*position, frame)] += 1;
        }
    }
    counts
}

/// Separate heavy fingers past `depth_line`: maximal runs of adjacent
/// interior bins that each hold at least `MIN_FINGER_PARTICLES` heavy
/// particles below the line. The bin touching each wall is skipped (D-07).
pub(super) fn finger_count(layers: &Layers, frame: GravityFrame, depth_line: f32) -> usize {
    let counts = past_counts(layers, frame, depth_line);
    let mut fingers = 0;
    let mut in_finger = false;
    for count in &counts[1..FINGER_BIN_COUNT - 1] {
        let past = *count >= MIN_FINGER_PARTICLES;
        if past && !in_finger {
            fingers += 1;
        }
        in_finger = past;
    }
    fingers
}

fn mean_height(positions: &[Vec2], frame: GravityFrame) -> f32 {
    let total: f32 = positions
        .iter()
        .map(|position| position.dot(frame.up))
        .sum();
    total / f32::from(u16::try_from(positions.len().max(1)).expect("count fits u16"))
}

/// Whether the heavy layer's mean height along `up` is below the light one's (D-06).
pub(super) fn is_overturned(layers: &Layers, frame: GravityFrame) -> bool {
    mean_height(&layers.heavy, frame) < mean_height(&layers.light, frame)
}

/// Largest particle speed.
pub(super) fn max_speed(velocities: &[Vec2]) -> f32 {
    velocities
        .iter()
        .map(|velocity| velocity.length())
        .fold(0.0, f32::max)
}

/// Particles outside the inner tank by more than one radius, or non-finite.
pub(super) fn out_of_tank_count(positions: &[Vec2], radius: f32) -> usize {
    positions
        .iter()
        .filter(|position| {
            let inside = position.x.abs() <= TANK_HALF_WIDTH + radius
                && position.y >= FLOOR_TOP_Y - radius
                && position.y <= LID_BOTTOM_Y + radius;
            !inside || !position.x.is_finite() || !position.y.is_finite()
        })
        .count()
}

/// Share of heavy particles below `depth_line` with fewer than two heavy
/// neighbors (among those particles) within three radii: a rain diagnostic.
pub(super) fn rain_share(
    layers: &Layers,
    frame: GravityFrame,
    radius: f32,
    depth_line: f32,
) -> f32 {
    let below: Vec<Vec2> = layers
        .heavy
        .iter()
        .copied()
        .filter(|position| position.dot(frame.up) < depth_line)
        .collect();
    if below.is_empty() {
        return 0.0;
    }
    let reach = RAIN_NEIGHBOR_RADII * radius;
    let isolated = below
        .iter()
        .enumerate()
        .filter(|(index, position)| {
            let neighbors = below
                .iter()
                .enumerate()
                .filter(|(other, neighbor)| {
                    other != index && (**neighbor - **position).length() <= reach
                })
                .count();
            neighbors < MIN_RAIN_NEIGHBORS
        })
        .count();
    let share = |count: usize| f32::from(u16::try_from(count).expect("count fits u16"));
    share(isolated) / share(below.len())
}

/// How far below `interface` the deepest interior bin reaches, measured at
/// each bin's fourth-deepest heavy particle (S6). Bins with fewer than four
/// heavy particles are skipped.
pub(super) fn ripple_depth(layers: &Layers, frame: GravityFrame, interface: f32) -> f32 {
    let mut bins: Vec<Vec<f32>> = vec![Vec::new(); FINGER_BIN_COUNT];
    for position in &layers.heavy {
        bins[bin_index(*position, frame)].push(position.dot(frame.up));
    }
    bins[1..FINGER_BIN_COUNT - 1]
        .iter_mut()
        .filter(|heights| heights.len() >= MIN_FINGER_PARTICLES)
        .map(|heights| {
            heights.sort_by(f32::total_cmp);
            interface - heights[MIN_FINGER_PARTICLES - 1]
        })
        .fold(f32::NEG_INFINITY, f32::max)
}

/// The particle force lane split by `heavy`: `(heavy rows, light rows)`,
/// each in row order. Read it right before and right after `on_advance`
/// and compare the two reads: wall reaction forces from the previous step
/// already sit in the lane, so its absolute values mean nothing here.
pub(super) fn force_rows(
    world: &World,
    system: ParticleSystemId,
    heavy: ParticleGroupId,
) -> (Vec<Vec2>, Vec<Vec2>) {
    let view = world.particle_system_view(system).expect("liquid");
    let mut rows = (Vec::new(), Vec::new());
    for (force, group) in view.forces().iter().zip(view.group_ids()) {
        if *group == Some(heavy) {
            rows.0.push(*force);
        } else {
            rows.1.push(*force);
        }
    }
    rows
}

/// Bit patterns of `vectors`, for exact comparisons.
pub(super) fn vector_bits(vectors: &[Vec2]) -> Vec<(u32, u32)> {
    vectors
        .iter()
        .map(|vector| (vector.x.to_bits(), vector.y.to_bits()))
        .collect()
}

/// Bit patterns of the whole particle velocity lane.
pub(super) fn velocity_bits(world: &World, system: ParticleSystemId) -> Vec<(u32, u32)> {
    vector_bits(
        world
            .particle_system_view(system)
            .expect("liquid")
            .velocities(),
    )
}

/// One session-order fixed step on bare parts: `on_advance`, then `world.step`.
pub(super) fn fixed_step(parts: &mut RayleighTaylorParts) {
    parts
        .hooks
        .on_advance(&mut parts.world, parts.particle_system)
        .expect("the extra weight applies");
    parts
        .world
        .step(
            step_configuration(),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("the tank world steps");
}

/// Plain mean of `positions`.
pub(super) fn centroid(positions: &[Vec2]) -> Vec2 {
    let total = positions
        .iter()
        .fold(Vec2::ZERO, |sum, position| sum + *position);
    total * (1.0 / f32::from(u16::try_from(positions.len().max(1)).expect("count fits u16")))
}

/// S4 speed and containment, sampled every `STABILITY_SAMPLE_EVERY` steps.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct Stability {
    /// Largest sampled speed over steps 0..=1,200.
    pub(super) max_speed_early: f32,
    /// Largest sampled speed from step 3,000, once a sample lands there.
    pub(super) maybe_max_speed_late: Option<f32>,
    pub(super) max_speed_overall: f32,
    pub(super) max_out_of_tank: usize,
}

impl Stability {
    /// Folds in the sample taken after `step`, for particles of `radius`.
    pub(super) fn record(
        &mut self,
        step: u32,
        positions: &[Vec2],
        velocities: &[Vec2],
        radius: f32,
    ) {
        let speed = max_speed(velocities);
        self.max_speed_overall = self.max_speed_overall.max(speed);
        if step <= EARLY_WINDOW_END {
            self.max_speed_early = self.max_speed_early.max(speed);
        }
        if step >= LATE_WINDOW_START {
            self.maybe_max_speed_late = Some(
                self.maybe_max_speed_late
                    .map_or(speed, |late| late.max(speed)),
            );
        }
        self.max_out_of_tank = self
            .max_out_of_tank
            .max(out_of_tank_count(positions, radius));
    }
}

/// S4: nothing left the tank, every sampled speed is within `MAX_SPEED`, and
/// the late window is no faster than the early one. `context` is printed on
/// failure.
pub(super) fn assert_bounded(stability: &Stability, context: &impl std::fmt::Debug) {
    assert_eq!(stability.max_out_of_tank, 0, "{context:?}");
    assert!(stability.max_speed_overall <= MAX_SPEED, "{context:?}");
    let late = stability.maybe_max_speed_late.expect("late window sampled");
    assert!(late <= stability.max_speed_early, "{context:?}");
}

/// Advances a session `steps` fixed steps in the largest allowed chunks;
/// every chunk must step.
pub(super) fn advance(session: &mut SessionCore, steps: u32) {
    let mut remaining = steps;
    while remaining > 0 {
        let chunk = remaining.min(MAX_ADVANCE_STEPS);
        session
            .advance(chunk)
            .expect("the rayleigh-taylor scene steps");
        remaining -= chunk;
    }
}

/// The heavy group: the group of the highest particle. The heavy layer sits
/// on top at load, so call this before advancing.
pub(super) fn heavy_group_of(session: &SessionCore) -> ParticleGroupId {
    session.read_particles(|world, system| {
        let view = world.particle_system_view(system).expect("liquid");
        let (top, _height) = view.positions().iter().enumerate().fold(
            (0, f32::NEG_INFINITY),
            |best, (index, position)| {
                if position.y > best.1 {
                    (index, position.y)
                } else {
                    best
                }
            },
        );
        view.group_ids()[top].expect("every particle is in a layer group")
    })
}

/// The session's particles split by `heavy`.
pub(super) fn session_layers(session: &SessionCore, heavy: ParticleGroupId) -> Layers {
    session.read_particles(|world, system| split_layers(world, system, heavy))
}

/// Bit patterns of the session's positions, then its velocities.
pub(super) fn session_bits(session: &SessionCore) -> Vec<(u32, u32)> {
    session.read_particles(|world, system| {
        let view = world.particle_system_view(system).expect("liquid");
        let mut bits = vector_bits(view.positions());
        bits.extend(vector_bits(view.velocities()));
        bits
    })
}

/// Stability of a session run, with its built and live particle counts.
#[derive(Debug, Clone, Copy)]
pub(super) struct StabilityReport {
    pub(super) stability: Stability,
    pub(super) built: usize,
    pub(super) live: usize,
}

/// Advances `session` `steps` steps, sampling S4 stability every
/// `STABILITY_SAMPLE_EVERY` steps; every step must succeed.
pub(super) fn run_session_stability(session: &mut SessionCore, steps: u32) -> StabilityReport {
    let mut stability = Stability::default();
    let mut step = 0;
    while step < steps {
        let chunk = STABILITY_SAMPLE_EVERY.min(steps - step);
        advance(session, chunk);
        step += chunk;
        session.read_particles(|world, system| {
            let view = world.particle_system_view(system).expect("liquid");
            stability.record(step, view.positions(), view.velocities(), TUNING.radius);
        });
    }
    StabilityReport {
        stability,
        built: session.particle_count(),
        live: session.live_particle_count().expect("live particle count"),
    }
}
