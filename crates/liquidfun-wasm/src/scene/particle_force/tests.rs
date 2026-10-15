//! Debug-safe tests for the shared particle force helper (D-12, D-13, D-14).

use std::f32::consts::FRAC_1_SQRT_2;

use liquidfun::collision::{PolygonShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleFlags, ParticleGroupDestination, ParticleGroupRecipe, ParticleGroupSource,
};
use liquidfun::{
    NoDecisionHook, ParticleGroupId, ParticleSystemDef, ParticleSystemId, StepConfiguration,
    StepLimits, World,
};

use super::{
    ForceScale, GravityFrame, ParticleScratch, apply_accelerations, apply_group_acceleration,
};

const RADIUS: f32 = 0.04;
const DENSITY: f32 = 1.0;
const SETTLE_STEPS: u32 = 30;
const STEP_SECONDS: f32 = 1.0 / 60.0;
/// Relative error allowed between the measured and requested velocity change (D-13).
const CALIBRATION_TOLERANCE: f32 = 1e-3;

/// A zero-gravity world holding one settled 5 x 25 box of liquid particles.
fn small_world(iterations: u32) -> (World, ParticleSystemId, ParticleGroupId) {
    let mut world = World::new().expect("world");
    world.set_gravity(Vec2::ZERO).expect("zero gravity");
    let definition = ParticleSystemDef::default()
        .with_radius(RADIUS)
        .expect("radius")
        .with_density(DENSITY)
        .expect("density");
    let system = world
        .create_particle_system_with_def(&definition)
        .expect("system");
    let stride = 0.75 * 2.0 * RADIUS;
    let filled = Shape::from(
        PolygonShape::oriented_box(12.5 * stride, 2.5 * stride, Vec2::ZERO, 0.0).expect("box"),
    );
    let source = ParticleGroupSource::filled_shapes(vec![filled]).expect("source");
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
        .with_particle_flags(ParticleFlags::WATER)
        .with_transform(Transform::IDENTITY)
        .expect("recipe");
    let group = world.create_particle_group(system, &recipe).expect("group");
    for _ in 0..SETTLE_STEPS {
        step(&mut world, iterations);
    }
    (world, system, group)
}

fn step(world: &mut World, iterations: u32) {
    let configuration = StepConfiguration::new(STEP_SECONDS, 8, 3)
        .expect("valid step")
        .with_particle_iterations(iterations)
        .expect("particle iterations");
    world
        .step(configuration, &mut NoDecisionHook, StepLimits::default())
        .expect("world steps");
}

fn velocities(world: &World, system: ParticleSystemId) -> Vec<Vec2> {
    world
        .particle_system_view(system)
        .expect("view")
        .velocities()
        .to_vec()
}

/// Mean of `forced - twin` over aligned velocity lanes.
fn mean_difference(forced: &[Vec2], twin: &[Vec2]) -> Vec2 {
    assert_eq!(forced.len(), twin.len());
    let sum = forced
        .iter()
        .zip(twin)
        .fold(Vec2::ZERO, |sum, (forced, twin)| sum + (*forced - *twin));
    sum / f32::from(u16::try_from(forced.len()).expect("small count"))
}

fn relative_error(measured: Vec2, expected: Vec2) -> f32 {
    (measured - expected).length() / expected.length()
}

/// Steps a forced world and its untouched twin once and returns the mean velocity change.
fn measured_change(iterations: u32, acceleration: Vec2) -> Vec2 {
    let (mut forced, system, _group) = small_world(iterations);
    let (mut twin, twin_system, _twin_group) = small_world(iterations);
    let scale = ForceScale::for_system(&forced, system, iterations).expect("scale");
    let mut scratch = ParticleScratch::default();
    apply_accelerations(&mut forced, system, scale, &mut scratch, |_, _| {
        acceleration
    })
    .expect("forces apply");
    step(&mut forced, iterations);
    step(&mut twin, iterations);
    mean_difference(
        &velocities(&forced, system),
        &velocities(&twin, twin_system),
    )
}

#[test]
fn straight_down_gravity_builds_the_unit_frame() {
    // Act
    let frame = GravityFrame::from_gravity(Vec2::new(0.0, -10.0)).expect("frame");

    // Assert
    assert_eq!(frame.magnitude.to_bits(), 10.0_f32.to_bits());
    assert_eq!(frame.up, Vec2::new(0.0, 1.0));
    assert_eq!(frame.across, Vec2::new(1.0, 0.0));
}

#[test]
fn tilted_gravity_turns_the_frame_with_it() {
    // Act
    let frame = GravityFrame::from_gravity(Vec2::new(7.071_068, -7.071_068)).expect("frame");

    // Assert
    assert!((frame.up - Vec2::new(-FRAC_1_SQRT_2, FRAC_1_SQRT_2)).length() < 1e-5);
    assert_eq!(frame.across, Vec2::new(frame.up.y, -frame.up.x));
}

#[test]
fn zero_tiny_or_non_finite_gravity_has_no_frame() {
    // Arrange
    let gravities = [
        Vec2::ZERO,
        Vec2::new(f32::NAN, -10.0),
        Vec2::new(0.0, -1e-6),
    ];

    for gravity in gravities {
        // Act
        let maybe_frame = GravityFrame::from_gravity(gravity);

        // Assert
        assert!(maybe_frame.is_none(), "{gravity:?}");
    }
}

#[test]
fn force_scale_is_particle_mass_times_iterations() {
    // Arrange
    let (world, system, _group) = small_world(2);
    let expected = DENSITY * (0.75 * 0.08) * (0.75 * 0.08) * 2.0;

    // Act
    let scale = ForceScale::for_system(&world, system, 2).expect("scale");

    // Assert
    let force = scale.force(Vec2::new(1.0, 0.0));
    assert!((force.x - expected).abs() < 1e-7, "{force:?} vs {expected}");
}

#[test]
fn requested_acceleration_changes_velocity_by_a_dt_at_two_iterations() {
    // Arrange
    let acceleration = Vec2::new(3.0, -2.0);

    // Act
    let change = measured_change(2, acceleration);

    // Assert
    let expected = acceleration * STEP_SECONDS;
    assert!(
        relative_error(change, expected) <= CALIBRATION_TOLERANCE,
        "{change:?} vs {expected:?}"
    );
}

#[test]
fn requested_acceleration_changes_velocity_by_a_dt_at_three_iterations() {
    // Arrange
    let acceleration = Vec2::new(3.0, -2.0);

    // Act
    let change = measured_change(3, acceleration);

    // Assert
    let expected = acceleration * STEP_SECONDS;
    assert!(
        relative_error(change, expected) <= CALIBRATION_TOLERANCE,
        "{change:?} vs {expected:?}"
    );
}

#[test]
fn group_acceleration_reaches_every_member_once() {
    // Arrange
    let acceleration = Vec2::new(0.0, 4.0);
    let (mut forced, system, group) = small_world(2);
    let (mut twin, twin_system, _twin_group) = small_world(2);
    let scale = ForceScale::for_system(&forced, system, 2).expect("scale");

    // Act
    apply_group_acceleration(&mut forced, system, group, scale, acceleration)
        .expect("group force applies");
    step(&mut forced, 2);
    step(&mut twin, 2);

    // Assert
    let change = mean_difference(
        &velocities(&forced, system),
        &velocities(&twin, twin_system),
    );
    let expected = acceleration * STEP_SECONDS;
    assert!(
        relative_error(change, expected) <= CALIBRATION_TOLERANCE,
        "{change:?} vs {expected:?}"
    );
}

#[test]
fn zero_accelerations_leave_velocities_bit_identical() {
    // Arrange
    let (mut forced, system, _group) = small_world(2);
    let (mut twin, twin_system, _twin_group) = small_world(2);
    let scale = ForceScale::for_system(&forced, system, 2).expect("scale");
    let mut scratch = ParticleScratch::default();

    // Act
    apply_accelerations(&mut forced, system, scale, &mut scratch, |_, _| Vec2::ZERO)
        .expect("nothing to apply");
    step(&mut forced, 2);
    step(&mut twin, 2);

    // Assert
    let forced_bits = velocities(&forced, system)
        .iter()
        .map(|velocity| (velocity.x.to_bits(), velocity.y.to_bits()))
        .collect::<Vec<_>>();
    let twin_bits = velocities(&twin, twin_system)
        .iter()
        .map(|velocity| (velocity.x.to_bits(), velocity.y.to_bits()))
        .collect::<Vec<_>>();
    assert_eq!(forced_bits, twin_bits);
}
