//! Black-box coverage for checked rigid-body controls.

use liquidfun::collision::FilterData;
use liquidfun::collision::shape::{CircleShape, Shape};
use liquidfun::math::Vec2;
use liquidfun::{BodyControlError, BodyDef, BodyDefError, BodyType, FixtureDef, WakePolicy, World};

fn circle_fixture() -> FixtureDef {
    let shape = Shape::from(
        CircleShape::new(Vec2::ZERO, 1.0).expect("test circle should have valid geometry"),
    );
    FixtureDef::new(shape, 1.0, 0.2, 0.0, false, FilterData::default())
        .expect("test fixture should be valid")
}

#[test]
fn definitions_default_controls_match_pinned_values() {
    // Arrange
    let definition = BodyDef::default();

    // Act
    let snapshot = definition.snapshot();

    // Assert
    assert_eq!(definition.linear_velocity(), Vec2::ZERO);
    assert_eq!(definition.angular_velocity().to_bits(), 0.0_f32.to_bits());
    assert_eq!(definition.linear_damping().to_bits(), 0.0_f32.to_bits());
    assert_eq!(definition.angular_damping().to_bits(), 0.0_f32.to_bits());
    assert_eq!(definition.gravity_scale().to_bits(), 1.0_f32.to_bits());
    assert!(definition.is_sleeping_allowed());
    assert!(definition.is_awake());
    assert!(!definition.is_fixed_rotation());
    assert!(!definition.is_bullet());
    assert_eq!(snapshot.linear_velocity(), Vec2::ZERO);
    assert_eq!(snapshot.angular_velocity().to_bits(), 0.0_f32.to_bits());
    assert_eq!(snapshot.linear_damping().to_bits(), 0.0_f32.to_bits());
    assert_eq!(snapshot.angular_damping().to_bits(), 0.0_f32.to_bits());
    assert_eq!(snapshot.gravity_scale().to_bits(), 1.0_f32.to_bits());
    assert!(snapshot.is_sleeping_allowed());
    assert!(snapshot.is_awake());
    assert!(!snapshot.is_fixed_rotation());
    assert!(!snapshot.is_bullet());
}

#[test]
fn definitions_checked_builders_preserve_control_bits() {
    // Arrange
    let linear_velocity = Vec2::new(-3.25, 7.5);

    // Act
    let definition = BodyDef::new(BodyType::Dynamic, Vec2::new(2.0, -4.0), 0.25, true)
        .expect("finite definition should be accepted")
        .with_linear_velocity(linear_velocity)
        .expect("finite velocity should be accepted")
        .with_angular_velocity(-1.75)
        .expect("finite angular velocity should be accepted")
        .with_linear_damping(0.125)
        .expect("non-negative finite damping should be accepted")
        .with_angular_damping(0.375)
        .expect("non-negative finite damping should be accepted")
        .with_gravity_scale(-0.5)
        .expect("finite gravity scale should be accepted")
        .with_sleeping_allowed(false)
        .with_awake(false)
        .with_fixed_rotation(true)
        .with_bullet(true);
    let snapshot = definition.snapshot();

    // Assert
    assert_eq!(definition.linear_velocity(), linear_velocity);
    assert_eq!(
        definition.angular_velocity().to_bits(),
        (-1.75_f32).to_bits()
    );
    assert_eq!(definition.linear_damping().to_bits(), 0.125_f32.to_bits());
    assert_eq!(definition.angular_damping().to_bits(), 0.375_f32.to_bits());
    assert_eq!(definition.gravity_scale().to_bits(), (-0.5_f32).to_bits());
    assert!(!definition.is_sleeping_allowed());
    assert!(!definition.is_awake());
    assert!(definition.is_fixed_rotation());
    assert!(definition.is_bullet());
    assert_eq!(snapshot.linear_velocity(), linear_velocity);
    assert_eq!(snapshot.angular_velocity().to_bits(), (-1.75_f32).to_bits());
    assert_eq!(snapshot.linear_damping().to_bits(), 0.125_f32.to_bits());
    assert_eq!(snapshot.angular_damping().to_bits(), 0.375_f32.to_bits());
    assert_eq!(snapshot.gravity_scale().to_bits(), (-0.5_f32).to_bits());
    assert!(!snapshot.is_sleeping_allowed());
    assert!(!snapshot.is_awake());
    assert!(snapshot.is_fixed_rotation());
    assert!(snapshot.is_bullet());
}

#[test]
fn definitions_reject_non_finite_and_negative_controls() {
    // Arrange
    let definition = BodyDef::default();

    // Act
    let velocity_x = definition.with_linear_velocity(Vec2::new(f32::NAN, 0.0));
    let velocity_y = definition.with_linear_velocity(Vec2::new(0.0, f32::INFINITY));
    let angular_velocity = definition.with_angular_velocity(f32::NEG_INFINITY);
    let linear_damping = definition.with_linear_damping(-0.25);
    let angular_damping = definition.with_angular_damping(f32::NAN);
    let gravity_scale = definition.with_gravity_scale(f32::INFINITY);

    // Assert
    assert_eq!(velocity_x, Err(BodyDefError::NonFiniteLinearVelocityX));
    assert_eq!(velocity_y, Err(BodyDefError::NonFiniteLinearVelocityY));
    assert_eq!(
        angular_velocity,
        Err(BodyDefError::NonFiniteAngularVelocity)
    );
    assert_eq!(linear_damping, Err(BodyDefError::NegativeLinearDamping));
    assert_eq!(angular_damping, Err(BodyDefError::NonFiniteAngularDamping));
    assert_eq!(gravity_scale, Err(BodyDefError::NonFiniteGravityScale));
}

#[test]
fn world_api_exposes_every_granular_body_control() {
    // Arrange
    let definition = BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)
        .expect("finite definition should be accepted")
        .with_awake(false);
    let mut world = World::new().expect("world key should remain available");
    let body = world.create_body(&definition).expect("body should fit");
    world
        .create_fixture(body, &circle_fixture())
        .expect("fixture should fit");

    // Act
    world
        .set_body_linear_velocity(body, Vec2::ZERO)
        .expect("zero velocity should be accepted");
    let zero_velocity = world.body_snapshot(body).expect("body should remain live");
    world
        .set_body_angular_velocity(body, 2.0)
        .expect("finite angular velocity should be accepted");
    world
        .set_body_awake(body, false)
        .expect("body should remain live");
    world
        .apply_body_force_to_center(body, Vec2::new(1.0, 2.0), WakePolicy::PreserveSleep)
        .expect("preserved sleeping force should be ignored");
    world
        .apply_body_force(
            body,
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 1.0),
            WakePolicy::Wake,
        )
        .expect("finite force should be accepted");
    world
        .apply_body_torque(body, 0.5, WakePolicy::Wake)
        .expect("finite torque should be accepted");
    world
        .apply_body_linear_impulse_to_center(body, Vec2::new(3.0, -1.0), WakePolicy::Wake)
        .expect("finite impulse should be accepted");
    world
        .apply_body_linear_impulse(
            body,
            Vec2::new(0.5, 0.25),
            Vec2::new(0.0, 1.0),
            WakePolicy::Wake,
        )
        .expect("finite off-center impulse should be accepted");
    world
        .apply_body_angular_impulse(body, 0.75, WakePolicy::Wake)
        .expect("finite angular impulse should be accepted");
    world
        .set_body_linear_damping(body, 0.125)
        .expect("finite damping should be accepted");
    world
        .set_body_angular_damping(body, 0.25)
        .expect("finite damping should be accepted");
    world
        .set_body_gravity_scale(body, -0.5)
        .expect("finite gravity scale should be accepted");
    world
        .set_body_bullet(body, true)
        .expect("body should remain live");
    world
        .set_body_angular_velocity(body, 4.0)
        .expect("finite angular velocity should be accepted");
    world
        .set_body_fixed_rotation(body, true)
        .expect("mass reset should remain valid");
    let fixed = world.body_snapshot(body).expect("body should remain live");
    world
        .set_body_fixed_rotation(body, false)
        .expect("mass reset should remain valid");
    world
        .set_body_awake(body, false)
        .expect("body should remain live");
    world
        .set_body_sleeping_allowed(body, false)
        .expect("body should remain live");
    let final_snapshot = world.body_snapshot(body).expect("body should remain live");

    // Assert
    assert!(!zero_velocity.is_awake());
    assert!(fixed.is_fixed_rotation());
    assert_eq!(fixed.angular_velocity().to_bits(), 0.0_f32.to_bits());
    assert_eq!(fixed.rotational_inertia().to_bits(), 0.0_f32.to_bits());
    assert!(!final_snapshot.is_fixed_rotation());
    assert!(final_snapshot.rotational_inertia() > 0.0);
    assert_eq!(
        final_snapshot.linear_damping().to_bits(),
        0.125_f32.to_bits()
    );
    assert_eq!(
        final_snapshot.angular_damping().to_bits(),
        0.25_f32.to_bits()
    );
    assert_eq!(
        final_snapshot.gravity_scale().to_bits(),
        (-0.5_f32).to_bits()
    );
    assert!(final_snapshot.is_bullet());
    assert!(!final_snapshot.is_sleeping_allowed());
    assert!(final_snapshot.is_awake());
}

#[test]
fn world_api_rejects_foreign_and_stale_handles_without_mutation() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let local = world
        .create_body(&BodyDef::default())
        .expect("local body should fit");
    let before = world
        .body_snapshot(local)
        .expect("local body should remain live");
    let mut other = World::new().expect("world key should remain available");
    let foreign = other
        .create_body(&BodyDef::default())
        .expect("foreign body should fit");
    let stale = world
        .create_body(&BodyDef::default())
        .expect("temporary body should fit");
    world
        .destroy_body(stale)
        .expect("temporary body should be live");

    // Act
    let foreign_result = world.set_body_linear_velocity(foreign, Vec2::new(1.0, 0.0));
    let stale_result = world.set_body_angular_damping(stale, 0.5);
    let after = world
        .body_snapshot(local)
        .expect("local body should remain live");

    // Assert
    assert!(matches!(
        foreign_result,
        Err(BodyControlError::InvalidHandle(_))
    ));
    assert!(matches!(
        stale_result,
        Err(BodyControlError::InvalidHandle(_))
    ));
    assert_eq!(after, before);
}

#[test]
fn world_api_candidate_overflow_is_atomic() {
    // Arrange
    let definition = BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)
        .expect("finite definition should be accepted")
        .with_linear_velocity(Vec2::new(f32::MAX, 0.0))
        .expect("finite velocity should be accepted");
    let mut world = World::new().expect("world key should remain available");
    let body = world.create_body(&definition).expect("body should fit");
    world
        .create_fixture(body, &circle_fixture())
        .expect("fixture should fit");
    world
        .apply_body_force_to_center(body, Vec2::new(f32::MAX, 0.0), WakePolicy::Wake)
        .expect("first finite force should fit");
    let before = world.body_snapshot(body).expect("body should remain live");
    let entries_before = world.broad_phase_entry_count();

    // Act
    let force_result =
        world.apply_body_force_to_center(body, Vec2::new(f32::MAX, 0.0), WakePolicy::Wake);
    let impulse_result =
        world.apply_body_linear_impulse_to_center(body, Vec2::new(f32::MAX, 0.0), WakePolicy::Wake);
    let damping_result = world.set_body_linear_damping(body, f32::NAN);
    let after = world.body_snapshot(body).expect("body should remain live");

    // Assert
    assert_eq!(force_result, Err(BodyControlError::NonFiniteDerivedForceX));
    assert_eq!(
        impulse_result,
        Err(BodyControlError::NonFiniteDerivedLinearVelocityX)
    );
    assert_eq!(
        damping_result,
        Err(BodyControlError::NonFiniteLinearDamping)
    );
    assert_eq!(after, before);
    assert_eq!(world.broad_phase_entry_count(), entries_before);
}
