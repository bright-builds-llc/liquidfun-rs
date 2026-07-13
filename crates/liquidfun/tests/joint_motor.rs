//! Motor-joint offsets, caps, correction, and wake coverage.

use liquidfun::math::Vec2;
use liquidfun::{BodyDef, BodyType, JointDefError, JointSpecificSnapshot, MotorJointDef, World};

fn bodies(world: &mut World) -> (liquidfun::BodyId, liquidfun::BodyId) {
    let body_a = world
        .create_body(&BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.25, true).expect("A"))
        .expect("A storage");
    let body_b = world
        .create_body(&BodyDef::new(BodyType::Dynamic, Vec2::new(2.0, 3.0), 0.75, true).expect("B"))
        .expect("B storage");
    (body_a, body_b)
}

#[test]
fn motor_definition_rejects_invalid_offsets_caps_and_correction() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let definition = MotorJointDef::new(body_a, body_b).expect("definition");

    // Act
    let offset = definition.with_offsets(Vec2::new(f32::NAN, 0.0), 0.0);
    let caps = definition.with_caps(-1.0, 1.0);
    let correction = definition.with_correction_factor(1.1);

    // Assert
    assert_eq!(offset, Err(JointDefError::NonFiniteValue));
    assert_eq!(caps, Err(JointDefError::NegativeValue));
    assert_eq!(correction, Err(JointDefError::InvalidRange));
}

#[test]
fn motor_changed_offsets_wake_but_equal_offsets_and_tuning_do_not() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let joint = world
        .create_joint(
            MotorJointDef::new(body_a, body_b)
                .expect("definition")
                .into(),
        )
        .expect("joint");
    world.set_body_awake(body_a, false).expect("sleep A");
    world.set_body_awake(body_b, false).expect("sleep B");

    // Act
    world
        .set_motor_linear_offset(joint, Vec2::ZERO)
        .expect("equal linear");
    world
        .set_motor_angular_offset(joint, 0.0)
        .expect("equal angular");
    world.set_motor_max_force(joint, 4.0).expect("force");
    world.set_motor_max_torque(joint, 5.0).expect("torque");
    world
        .set_motor_correction_factor(joint, 0.6)
        .expect("correction");
    let before_changed = [
        world.body_snapshot(body_a).expect("A").is_awake(),
        world.body_snapshot(body_b).expect("B").is_awake(),
    ];
    world
        .set_motor_linear_offset(joint, Vec2::new(1.0, 2.0))
        .expect("changed linear");

    // Assert
    assert_eq!(before_changed, [false, false]);
    assert!(world.body_snapshot(body_a).expect("A").is_awake());
    assert!(world.body_snapshot(body_b).expect("B").is_awake());
}

#[test]
fn motor_snapshot_reports_offsets_caps_and_correction() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let definition = MotorJointDef::new(body_a, body_b)
        .expect("definition")
        .with_offsets(Vec2::new(1.0, 2.0), 0.5)
        .expect("offsets")
        .with_caps(4.0, 5.0)
        .expect("caps")
        .with_correction_factor(0.6)
        .expect("correction");
    let joint = world.create_joint(definition.into()).expect("joint");

    // Act
    let snapshot = world.joint_snapshot(joint).expect("snapshot");
    let JointSpecificSnapshot::Motor(state) = snapshot.specific() else {
        panic!("motor state expected");
    };

    // Assert
    assert_eq!(snapshot.anchor_a(), Vec2::ZERO);
    assert_eq!(snapshot.anchor_b(), Vec2::new(2.0, 3.0));
    assert_eq!(state.linear_offset(), Vec2::new(1.0, 2.0));
    assert_eq!(state.angular_offset().to_bits(), 0.5_f32.to_bits());
    assert_eq!(state.correction_factor().to_bits(), 0.6_f32.to_bits());
}
