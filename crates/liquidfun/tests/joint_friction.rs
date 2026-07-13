//! Friction-joint checked configuration and cap coverage.

use liquidfun::math::Vec2;
use liquidfun::{BodyDef, BodyType, FrictionJointDef, JointDefError, JointSpecificSnapshot, World};

fn bodies(world: &mut World) -> (liquidfun::BodyId, liquidfun::BodyId) {
    let body_a = world
        .create_body(&BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true).expect("A"))
        .expect("A storage");
    let body_b = world
        .create_body(&BodyDef::new(BodyType::Dynamic, Vec2::new(2.0, 3.0), 0.0, true).expect("B"))
        .expect("B storage");
    (body_a, body_b)
}

#[test]
fn friction_definition_rejects_invalid_anchors_and_caps() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let definition = FrictionJointDef::new(body_a, body_b).expect("definition");

    // Act
    let anchors = definition.with_anchors(Vec2::new(f32::NAN, 0.0), Vec2::ZERO);
    let force = definition.with_max_force(-1.0);
    let torque = definition.with_max_torque(-1.0);

    // Assert
    assert_eq!(anchors, Err(JointDefError::NonFiniteValue));
    assert_eq!(force, Err(JointDefError::NegativeValue));
    assert_eq!(torque, Err(JointDefError::NegativeValue));
}

#[test]
fn friction_snapshot_and_no_wake_setters_are_semantic() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let joint = world
        .create_joint(
            FrictionJointDef::new(body_a, body_b)
                .expect("definition")
                .into(),
        )
        .expect("joint");
    world.set_body_awake(body_a, false).expect("sleep A");
    world.set_body_awake(body_b, false).expect("sleep B");

    // Act
    world.set_friction_max_force(joint, 4.0).expect("force");
    world.set_friction_max_torque(joint, 5.0).expect("torque");
    let snapshot = world.joint_snapshot(joint).expect("snapshot");
    let JointSpecificSnapshot::Friction(state) = snapshot.specific() else {
        panic!("friction state expected");
    };

    // Assert
    assert_eq!(snapshot.anchor_a(), Vec2::ZERO);
    assert_eq!(snapshot.anchor_b(), Vec2::new(2.0, 3.0));
    assert_eq!(state.max_force().to_bits(), 4.0_f32.to_bits());
    assert_eq!(state.max_torque().to_bits(), 5.0_f32.to_bits());
    assert!(!world.body_snapshot(body_a).expect("A").is_awake());
    assert!(!world.body_snapshot(body_b).expect("B").is_awake());
}
