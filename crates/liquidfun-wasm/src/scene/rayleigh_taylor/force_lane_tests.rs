//! Debug-safe Rayleigh–Taylor extra-weight tests (D-02, D-03, D-18): twin
//! worlds for the settle window, and the particle force lane read right
//! after `on_advance` and before `world.step`.
//!
//! The lane is not zero between steps: wall collisions leave reaction forces
//! in it until the next Force pass consumes them, and `on_advance` adds onto
//! them. So every force-lane test compares the lane after `on_advance` with
//! a snapshot taken right before it. Post-step motion is coupled through the
//! pressure, damping and wall passes, so it is proved by the release
//! sideways-gravity flow test instead.

use liquidfun::math::Vec2;
use liquidfun::{NoDecisionHook, StepLimits};

use super::test_support::{force_rows, step_configuration, vector_bits, velocity_bits};
use super::{RayleighTaylorParts, TUNING, WeightRatio, build_parts, ramp_factor};
use crate::scene::SceneHooks;
use crate::scene::particle_force::GravityFrame;

/// Gravity 10 tilted 45 degrees toward `+x`.
const TILTED_GRAVITY: Vec2 = Vec2::new(7.071_068, -7.071_068);
/// Relative tolerance on a heavy row's added force: the helper divides the
/// group total by the member count and adds it onto the wall reaction force.
const FORCE_TOLERANCE: f32 = 1e-5;

fn new_parts(ratio: WeightRatio) -> RayleighTaylorParts {
    build_parts(ratio, &TUNING).expect("rayleigh-taylor parts")
}

/// Two identical builds: one steps with `on_advance`, the other without.
fn twins(ratio: WeightRatio) -> (RayleighTaylorParts, RayleighTaylorParts) {
    (new_parts(ratio), new_parts(ratio))
}

fn on_advance(parts: &mut RayleighTaylorParts) {
    parts
        .hooks
        .on_advance(&mut parts.world, parts.particle_system)
        .expect("the extra weight applies");
}

fn world_step(parts: &mut RayleighTaylorParts) {
    parts
        .world
        .step(
            step_configuration(),
            &mut NoDecisionHook,
            StepLimits::default(),
        )
        .expect("the tank world steps");
}

fn forces(parts: &RayleighTaylorParts) -> (Vec<Vec2>, Vec<Vec2>) {
    force_rows(&parts.world, parts.particle_system, parts.hooks.heavy_group)
}

/// The whole lane, heavy rows first, as bits.
fn lane_bits(parts: &RayleighTaylorParts) -> Vec<(u32, u32)> {
    let (heavy, light) = forces(parts);
    let mut bits = vector_bits(&heavy);
    bits.extend(vector_bits(&light));
    bits
}

/// A ratio-2 world stepped through the settle and the ramp without
/// `on_advance`, then told the ramp is complete, so the next `on_advance`
/// applies the full extra weight under whatever gravity is set.
fn ramped_parts(gravity: Vec2) -> RayleighTaylorParts {
    let mut parts = new_parts(WeightRatio::Two);
    let ramped = TUNING.settle_steps + TUNING.ramp_steps;
    for _step in 0..ramped {
        world_step(&mut parts);
    }
    parts.world.set_gravity(gravity).expect("finite gravity");
    parts.hooks.steps_advanced = ramped;
    parts
}

/// Asserts every heavy row's `after - before` equals `expected` within
/// `FORCE_TOLERANCE` of its length.
fn assert_heavy_rows_gain(before: &[Vec2], after: &[Vec2], expected: Vec2) {
    assert_eq!(before.len(), after.len());
    assert!(!after.is_empty());
    let tolerance = FORCE_TOLERANCE * expected.length();
    for (row, (old, new)) in before.iter().zip(after).enumerate() {
        let change = *new - *old;
        let error = change - expected;
        assert!(
            error.x.abs() <= tolerance && error.y.abs() <= tolerance,
            "row {row}: {change:?} vs {expected:?}"
        );
    }
}

#[test]
fn settle_window_applies_no_extra_weight() {
    // Arrange
    let (mut hooked, mut twin) = twins(WeightRatio::Three);

    for step in 0..TUNING.settle_steps {
        // Act
        on_advance(&mut hooked);
        world_step(&mut hooked);
        world_step(&mut twin);

        // Assert
        assert_eq!(
            velocity_bits(&hooked.world, hooked.particle_system),
            velocity_bits(&twin.world, twin.particle_system),
            "step {step}"
        );
    }
}

#[test]
fn settle_window_writes_no_force() {
    // Arrange
    let mut parts = new_parts(WeightRatio::Three);

    for step in 0..TUNING.settle_steps {
        let before = lane_bits(&parts);

        // Act
        on_advance(&mut parts);

        // Assert
        assert_eq!(lane_bits(&parts), before, "step {step}");
        world_step(&mut parts);
    }
}

#[test]
fn extra_weight_starts_after_the_settle_window() {
    // Arrange
    let mut parts = new_parts(WeightRatio::Two);
    for _step in 0..TUNING.settle_steps {
        on_advance(&mut parts);
        world_step(&mut parts);
    }
    let frame = GravityFrame::from_gravity(parts.world.gravity()).expect("gravity 10");
    let ramp = ramp_factor(TUNING.settle_steps, TUNING.settle_steps, TUNING.ramp_steps);
    let extra = WeightRatio::Two.extra_weight() * frame.magnitude * ramp;
    let expected = parts.hooks.scale.force(frame.up * -extra);
    let (before, _light) = forces(&parts);

    // Act
    on_advance(&mut parts);

    // Assert
    let (after, _light) = forces(&parts);
    assert!(expected.dot(frame.up) < 0.0, "{expected:?}");
    assert_heavy_rows_gain(&before, &after, expected);
}

#[test]
fn extra_weight_follows_tilted_gravity() {
    // Arrange
    let mut parts = ramped_parts(TILTED_GRAVITY);
    let frame = GravityFrame::from_gravity(TILTED_GRAVITY).expect("tilted gravity");
    let expected = parts
        .hooks
        .scale
        .force(frame.up * -(WeightRatio::Two.extra_weight() * frame.magnitude));
    let (before, _light) = forces(&parts);

    // Act
    on_advance(&mut parts);

    // Assert
    let (after, _light) = forces(&parts);
    assert_heavy_rows_gain(&before, &after, expected);
}

#[test]
fn extra_weight_leaves_the_light_layer_alone() {
    // Arrange
    let mut parts = ramped_parts(TILTED_GRAVITY);
    let (_heavy, before) = forces(&parts);

    // Act
    on_advance(&mut parts);

    // Assert
    let (_heavy, after) = forces(&parts);
    assert!(!after.is_empty());
    assert_eq!(vector_bits(&after), vector_bits(&before));
}

#[test]
fn zero_gravity_applies_no_extra_weight() {
    // Arrange
    let mut parts = ramped_parts(Vec2::ZERO);
    let before = lane_bits(&parts);

    // Act
    let result = parts
        .hooks
        .on_advance(&mut parts.world, parts.particle_system);

    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(lane_bits(&parts), before);
}
