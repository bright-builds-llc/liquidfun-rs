//! Test-only helpers shared by the Rayleigh–Taylor unit and spike tests: the
//! fixed step, the layer split and the D-07 finger, overturn, speed,
//! containment and rain metrics (49-01-PLAN "Metrics").
//!
//! Every metric reads heights along `frame.up` and bins along
//! `frame.across`, so a tilted-gravity test can reuse them. The finger bins
//! cover the inner tank in the gravity-down frame.

use liquidfun::math::Vec2;
use liquidfun::{ParticleGroupId, ParticleSystemId, StepConfiguration, World};

use super::geometry::{
    FLOOR_TOP_Y, INTERFACE_MEAN_Y, LID_BOTTOM_Y, TANK_HALF_WIDTH, TANK_INNER_HEIGHT,
};
use crate::scene::particle_force::GravityFrame;
use crate::scene::{SceneId, particle_iterations};

/// The 20% depth line: a finger must reach this far below the initial
/// interface, one fifth of the inner tank height (D-06).
pub(super) const DEPTH_LINE: f32 = INTERFACE_MEAN_Y - 0.2 * TANK_INNER_HEIGHT;
/// Width of one finger bin across the tank: two particle diameters at r 0.04.
pub(super) const FINGER_BIN_WIDTH: f32 = 0.16;
/// Bins across the inner tank; the last partial strip folds into the last bin.
pub(super) const FINGER_BIN_COUNT: usize = 17;
/// Least heavy particles below the depth line for a bin to count as past it,
/// so single raining particles never make a finger (D-07).
pub(super) const MIN_FINGER_PARTICLES: usize = 4;
/// Largest particle speed a bounded run may reach at gravity 10 (S4).
pub(super) const MAX_SPEED: f32 = 6.0;
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

/// Heavy particles below the depth line in each finger bin.
fn past_counts(layers: &Layers, frame: GravityFrame) -> [usize; FINGER_BIN_COUNT] {
    let mut counts = [0; FINGER_BIN_COUNT];
    for position in &layers.heavy {
        if position.dot(frame.up) < DEPTH_LINE {
            counts[bin_index(*position, frame)] += 1;
        }
    }
    counts
}

/// Separate heavy fingers past the depth line: maximal runs of adjacent
/// interior bins that each hold at least `MIN_FINGER_PARTICLES` heavy
/// particles below the line. The bin touching each wall is skipped (D-07).
pub(super) fn finger_count(layers: &Layers, frame: GravityFrame) -> usize {
    let counts = past_counts(layers, frame);
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

/// Share of heavy particles below the depth line with fewer than two heavy
/// neighbors (among those particles) within three radii: a rain diagnostic.
pub(super) fn rain_share(layers: &Layers, frame: GravityFrame, radius: f32) -> f32 {
    let below: Vec<Vec2> = layers
        .heavy
        .iter()
        .copied()
        .filter(|position| position.dot(frame.up) < DEPTH_LINE)
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

/// How far below `INTERFACE_MEAN_Y` the deepest interior bin reaches,
/// measured at each bin's fourth-deepest heavy particle (S6). Bins with
/// fewer than four heavy particles are skipped.
pub(super) fn seed_depth(layers: &Layers, frame: GravityFrame) -> f32 {
    let mut bins: Vec<Vec<f32>> = vec![Vec::new(); FINGER_BIN_COUNT];
    for position in &layers.heavy {
        bins[bin_index(*position, frame)].push(position.dot(frame.up));
    }
    bins[1..FINGER_BIN_COUNT - 1]
        .iter_mut()
        .filter(|heights| heights.len() >= MIN_FINGER_PARTICLES)
        .map(|heights| {
            heights.sort_by(f32::total_cmp);
            INTERFACE_MEAN_Y - heights[MIN_FINGER_PARTICLES - 1]
        })
        .fold(f32::NEG_INFINITY, f32::max)
}
