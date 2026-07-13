//! Wheel-joint definition, mutation, and semantic query coverage.

use liquidfun::math::Vec2;
use liquidfun::{BodyDef, BodyType, JointDefError, JointSpecificSnapshot, WheelJointDef, World};

fn bodies(world: &mut World) -> (liquidfun::BodyId, liquidfun::BodyId) {
    let body_a = world
        .create_body(&BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true).expect("body A"))
        .expect("body A storage");
    let body_b = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::new(2.0, 1.0), 0.5, true).expect("body B"),
        )
        .expect("body B storage");
    (body_a, body_b)
}

#[test]
fn wheel_definition_rejects_invalid_frame_and_configuration() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let definition = WheelJointDef::new(body_a, body_b).expect("definition");

    // Act
    let axis = definition.with_frame(Vec2::ZERO, Vec2::ZERO, Vec2::ZERO);
    let torque = definition.with_motor(true, 1.0, -1.0);
    let frequency = definition.with_spring(-1.0, 0.7);

    // Assert
    assert_eq!(axis, Err(JointDefError::InvalidAxis));
    assert_eq!(torque, Err(JointDefError::NegativeValue));
    assert_eq!(frequency, Err(JointDefError::NegativeValue));
}

#[test]
fn wheel_snapshot_reports_translation_speed_and_configuration() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let definition = WheelJointDef::new(body_a, body_b)
        .expect("definition")
        .with_frame(Vec2::ZERO, Vec2::ZERO, Vec2::new(1.0, 0.0))
        .expect("frame")
        .with_motor(true, 3.0, 4.0)
        .expect("motor")
        .with_spring(2.0, 0.5)
        .expect("spring");
    let joint = world.create_joint(definition.into()).expect("joint");

    // Act
    let snapshot = world.joint_snapshot(joint).expect("snapshot");
    let JointSpecificSnapshot::Wheel(state) = snapshot.specific() else {
        panic!("wheel state expected");
    };

    // Assert
    assert_eq!(state.translation().to_bits(), 2.0_f32.to_bits());
    assert_eq!(state.speed().to_bits(), 0.0_f32.to_bits());
    assert!(state.is_motor_enabled());
    assert_eq!(state.motor_speed().to_bits(), 3.0_f32.to_bits());
    assert_eq!(state.max_motor_torque().to_bits(), 4.0_f32.to_bits());
    assert_eq!(state.frequency().to_bits(), 2.0_f32.to_bits());
    assert_eq!(state.damping_ratio().to_bits(), 0.5_f32.to_bits());
}

#[test]
fn wheel_setters_wake_both_bodies_and_update_configuration() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let joint = world
        .create_joint(
            WheelJointDef::new(body_a, body_b)
                .expect("definition")
                .into(),
        )
        .expect("joint");
    world.set_body_awake(body_a, false).expect("sleep A");
    world.set_body_awake(body_b, false).expect("sleep B");

    // Act
    world.set_wheel_motor_enabled(joint, true).expect("enabled");
    world.set_wheel_motor_speed(joint, 5.0).expect("speed");
    world
        .set_wheel_max_motor_torque(joint, 6.0)
        .expect("torque");
    world.set_wheel_frequency(joint, 7.0).expect("frequency");
    world.set_wheel_damping_ratio(joint, 0.25).expect("damping");

    // Assert
    assert!(world.body_snapshot(body_a).expect("A").is_awake());
    assert!(world.body_snapshot(body_b).expect("B").is_awake());
    assert_eq!(world.wheel_motor_torque(joint, 60.0), Ok(0.0));
}
