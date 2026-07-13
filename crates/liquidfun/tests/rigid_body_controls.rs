//! Black-box coverage for checked rigid-body controls.

use liquidfun::math::Vec2;
use liquidfun::{BodyDef, BodyDefError, BodyType};

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
