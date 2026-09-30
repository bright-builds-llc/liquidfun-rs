//! Washing Machine construction, live drum speed, and a short tumble.

use std::f32::consts::TAU;

use liquidfun::math::Vec2;
use liquidfun::{JointDef, World, WorldObservationLimits};

use super::super::SceneId;
use super::geometry::INNER_RADIUS;
use super::{DEFAULT_DRUM_RPM, build};
use crate::session::{MAX_ADVANCE_STEPS, SessionCore, SessionError};

#[test]
fn create_builds_a_partially_filled_drum() {
    // Arrange / Act
    let session =
        SessionCore::create(SceneId::WashingMachine).expect("Washing Machine should construct");
    let count = session.particle_count();

    // Assert
    session
        .capture_frame()
        .expect("the drum drawing must fit the frame lane");

    // Assert
    assert_eq!(
        count, 3000,
        "the lower drum must start with 3000 particles (got {count})"
    );
    assert_eq!(
        session.rigid_shape_count(),
        64,
        "inner and outer wall chords plus four ribs use the frame lane"
    );
}

#[test]
fn default_drum_speed_matches_twenty_rpm() {
    // Arrange
    let super::super::BuiltScene { world, .. } =
        build(&[]).expect("Washing Machine should construct");

    // Act
    let speed = revolute_motor_speed(&world);

    // Assert
    let expected = f32::from(DEFAULT_DRUM_RPM) * TAU / 60.0;
    assert!(
        (speed - expected).abs() < 1.0e-5,
        "the drum must start at {DEFAULT_DRUM_RPM} rpm (got {speed}, expected {expected})"
    );
}

#[test]
fn drum_speed_control_is_live() {
    // Arrange
    let mut session =
        SessionCore::create(SceneId::WashingMachine).expect("Washing Machine should construct");
    let before = session.particle_count();

    // Act
    let applied = session.apply_control("drum-speed", "36");

    // Assert
    assert_eq!(applied, Ok(false));
    assert_eq!(session.particle_count(), before);
}

#[test]
fn drum_speed_rejects_off_scale_tokens() {
    // Arrange
    let mut session =
        SessionCore::create(SceneId::WashingMachine).expect("Washing Machine should construct");

    // Act
    let rejected = ["01", "49", "-1", "20.0", "fast", ""]
        .map(|value| session.apply_control("drum-speed", value));

    // Assert
    assert!(
        rejected
            .iter()
            .all(|result| *result == Err(SessionError::UnknownControl))
    );
}

#[test]
fn stopped_drum_stays_level() {
    // Arrange
    let mut session =
        SessionCore::create(SceneId::WashingMachine).expect("Washing Machine should construct");
    session
        .apply_control("drum-speed", "0")
        .expect("0 rpm is on the spinner");

    // Act
    advance_steps(&mut session, 88);
    let angle = session
        .motor_angle()
        .expect("the drum should report its angle");

    // Assert
    assert!(angle.abs() < 0.08, "0 rpm must hold the drum (got {angle})");
}

#[test]
fn fast_spin_turns_the_drum_and_lifts_water() {
    // Arrange
    let mut session =
        SessionCore::create(SceneId::WashingMachine).expect("Washing Machine should construct");
    session
        .apply_control("drum-speed", "48")
        .expect("48 rpm is the spinner maximum");
    let starting_top = session.read_particles(highest_particle);

    // Act
    advance_steps(&mut session, 120);
    let angle = session
        .motor_angle()
        .expect("the drum should report its angle");
    let (highest, farthest) = session.read_particles(|world, system| {
        (
            highest_particle(world, system),
            farthest_particle(world, system),
        )
    });

    // Assert — 48 rpm for two seconds is 1.6 turns.
    assert!(
        angle > TAU,
        "48 rpm must turn the drum through at least one revolution (got {angle})"
    );
    assert!(
        highest > starting_top + 0.35,
        "ribs must lift water above the resting fill (start {starting_top}, later {highest})"
    );
    assert!(
        farthest < INNER_RADIUS + 0.05,
        "water must stay inside the drum (farthest {farthest}, inner {INNER_RADIUS})"
    );
}

fn advance_steps(session: &mut SessionCore, steps: u32) {
    let mut remaining = steps;
    while remaining > 0 {
        let batch = remaining.min(MAX_ADVANCE_STEPS);
        session
            .advance(batch)
            .expect("the drum should keep stepping");
        remaining -= batch;
    }
}

fn revolute_motor_speed(world: &World) -> f32 {
    let observation = world
        .world_observation(WorldObservationLimits::reviewed())
        .expect("reviewed observation should include the drum motor");
    let revolute: Vec<_> = observation
        .joints()
        .iter()
        .filter_map(|joint| match joint.snapshot().definition() {
            JointDef::Revolute(definition) => Some(definition),
            _ => None,
        })
        .collect();
    assert_eq!(revolute.len(), 1, "exactly one revolute joint");
    assert!(
        revolute[0].is_motor_enabled(),
        "the drum motor must be enabled"
    );
    revolute[0].motor_speed()
}

fn highest_particle(world: &World, system: liquidfun::ParticleSystemId) -> f32 {
    particle_positions(world, system)
        .into_iter()
        .map(|position| position.y)
        .fold(f32::MIN, f32::max)
}

fn farthest_particle(world: &World, system: liquidfun::ParticleSystemId) -> f32 {
    particle_positions(world, system)
        .into_iter()
        .map(Vec2::length)
        .fold(0.0, f32::max)
}

fn particle_positions(world: &World, system: liquidfun::ParticleSystemId) -> Vec<Vec2> {
    world
        .particle_system_view(system)
        .expect("the drum particle system should stay live")
        .positions()
        .to_vec()
}
