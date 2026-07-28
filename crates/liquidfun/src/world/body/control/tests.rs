use crate::collision::MassData;
use crate::math::Vec2;

use super::{BodyState, BodyType, WakePolicy};
use crate::world::BodyDef;

fn body_state(body_type: BodyType, awake: bool) -> BodyState {
    let definition = BodyDef::new(body_type, Vec2::ZERO, 0.0, true)
        .expect("finite definition should be accepted")
        .with_awake(awake);
    BodyState::from_definition(&definition)
}

#[test]
fn non_dynamic_force_and_impulse_calls_are_successful_no_effects() {
    // Arrange
    let static_body = body_state(BodyType::Static, false);
    let kinematic_body = body_state(BodyType::Kinematic, false);

    // Act
    let static_candidate = static_body
        .candidate_apply_force(
            Vec2::new(f32::NAN, 0.0),
            Vec2::new(f32::INFINITY, 0.0),
            WakePolicy::Wake,
        )
        .expect("static force application should be ignored");
    let kinematic_candidate = kinematic_body
        .candidate_apply_linear_impulse_to_center(Vec2::new(f32::NAN, 0.0), WakePolicy::Wake)
        .expect("kinematic impulse application should be ignored");

    // Assert
    assert_eq!(static_candidate.snapshot(), static_body.snapshot());
    assert_eq!(kinematic_candidate.snapshot(), kinematic_body.snapshot());
}

#[test]
fn preserve_sleep_application_is_a_successful_no_effect() {
    // Arrange
    let body = body_state(BodyType::Dynamic, false);

    // Act
    let candidate = body
        .candidate_apply_torque(f32::NAN, WakePolicy::PreserveSleep)
        .expect("preserved sleeping application should be ignored");

    // Assert
    assert_eq!(candidate.snapshot(), body.snapshot());
    assert_eq!(candidate.force, body.force);
    assert_eq!(candidate.torque.to_bits(), body.torque.to_bits());
}

#[test]
fn wake_policy_wakes_before_a_valid_application() {
    // Arrange
    let body = body_state(BodyType::Dynamic, false);

    // Act
    let candidate = body
        .candidate_apply_force_to_center(Vec2::new(1.0, 0.0), WakePolicy::Wake)
        .expect("finite force should be accepted");

    // Assert
    assert!(candidate.snapshot().is_awake());
    assert_eq!(candidate.force, Vec2::new(1.0, 0.0));
}

#[test]
fn nonzero_velocity_wakes_while_zero_velocity_preserves_sleep() {
    // Arrange
    let body = body_state(BodyType::Dynamic, false);

    // Act
    let zero = body
        .candidate_set_linear_velocity(Vec2::ZERO)
        .expect("zero velocity should be accepted");
    let nonzero = body
        .candidate_set_angular_velocity(-1.0)
        .expect("finite angular velocity should be accepted");

    // Assert
    assert!(!zero.snapshot().is_awake());
    assert!(nonzero.snapshot().is_awake());
}

#[test]
fn sleeping_clears_motion_force_and_torque() {
    // Arrange
    let definition = BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)
        .expect("finite definition should be accepted")
        .with_linear_velocity(Vec2::new(2.0, -3.0))
        .expect("finite velocity should be accepted")
        .with_angular_velocity(4.0)
        .expect("finite angular velocity should be accepted");
    let body = BodyState::from_definition(&definition)
        .candidate_apply_force_to_center(Vec2::new(5.0, 6.0), WakePolicy::Wake)
        .expect("finite force should be accepted")
        .candidate_apply_torque(7.0, WakePolicy::Wake)
        .expect("finite torque should be accepted");

    // Act
    let sleeping = body.candidate_set_awake(false);

    // Assert
    assert!(!sleeping.snapshot().is_awake());
    assert_eq!(sleeping.snapshot().linear_velocity(), Vec2::ZERO);
    assert_eq!(
        sleeping.snapshot().angular_velocity().to_bits(),
        0.0_f32.to_bits()
    );
    assert_eq!(sleeping.force, Vec2::ZERO);
    assert_eq!(sleeping.torque.to_bits(), 0.0_f32.to_bits());
}

#[test]
fn disabling_sleep_wakes_an_asleep_body() {
    // Arrange
    let body = body_state(BodyType::Dynamic, false);

    // Act
    let candidate = body.candidate_set_sleeping_allowed(false);

    // Assert
    assert!(!candidate.snapshot().is_sleeping_allowed());
    assert!(candidate.snapshot().is_awake());
}

#[test]
fn passive_controls_preserve_sleep() {
    // Arrange
    let body = body_state(BodyType::Dynamic, false);

    // Act
    let candidate = body
        .candidate_set_linear_damping(0.25)
        .expect("finite damping should be accepted")
        .candidate_set_angular_damping(0.5)
        .expect("finite damping should be accepted")
        .candidate_set_gravity_scale(-1.0)
        .expect("finite gravity scale should be accepted")
        .candidate_set_bullet(true);

    // Assert
    assert!(!candidate.snapshot().is_awake());
    assert_eq!(
        candidate.snapshot().linear_damping().to_bits(),
        0.25_f32.to_bits()
    );
    assert_eq!(
        candidate.snapshot().angular_damping().to_bits(),
        0.5_f32.to_bits()
    );
    assert_eq!(
        candidate.snapshot().gravity_scale().to_bits(),
        (-1.0_f32).to_bits()
    );
    assert!(candidate.snapshot().is_bullet());
}

#[test]
fn fixed_rotation_clears_angular_velocity_and_recomputes_inertia() {
    // Arrange
    let definition = BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)
        .expect("finite definition should be accepted")
        .with_angular_velocity(3.0)
        .expect("finite angular velocity should be accepted");
    let mass_data =
        [MassData::new(2.0, Vec2::ZERO, 4.0)
            .expect("finite positive mass data should be accepted")];
    let body = BodyState::from_definition(&definition)
        .with_reset_mass_data(&mass_data)
        .expect("valid mass data should aggregate");

    // Act
    let fixed = body
        .candidate_set_fixed_rotation(true, &mass_data)
        .expect("fixed rotation should recompute valid mass data");
    let free = fixed
        .candidate_set_fixed_rotation(false, &mass_data)
        .expect("free rotation should recompute valid mass data");

    // Assert
    assert!(fixed.snapshot().is_fixed_rotation());
    assert_eq!(
        fixed.snapshot().angular_velocity().to_bits(),
        0.0_f32.to_bits()
    );
    assert_eq!(
        fixed.snapshot().rotational_inertia().to_bits(),
        0.0_f32.to_bits()
    );
    assert_eq!(fixed.inverse_inertia().to_bits(), 0.0_f32.to_bits());
    assert!(!free.snapshot().is_fixed_rotation());
    assert_eq!(
        free.snapshot().rotational_inertia().to_bits(),
        4.0_f32.to_bits()
    );
    assert_eq!(free.inverse_inertia().to_bits(), 0.25_f32.to_bits());
}

#[test]
fn derived_overflow_returns_error_without_a_candidate() {
    // Arrange
    let body = body_state(BodyType::Dynamic, true)
        .candidate_set_linear_velocity(Vec2::new(f32::MAX, 0.0))
        .expect("finite velocity should be accepted");

    // Act
    let maybe_candidate =
        body.candidate_apply_linear_impulse_to_center(Vec2::new(f32::MAX, 0.0), WakePolicy::Wake);

    // Assert
    assert!(maybe_candidate.is_err());
    assert_eq!(
        body.snapshot().linear_velocity().x.to_bits(),
        f32::MAX.to_bits()
    );
}
