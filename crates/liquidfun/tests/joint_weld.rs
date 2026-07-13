//! Weld-joint definition, mutation, and semantic query coverage.

use liquidfun::math::Vec2;
use liquidfun::{BodyDef, BodyType, JointDefError, JointSpecificSnapshot, WeldJointDef, World};

fn bodies(world: &mut World) -> (liquidfun::BodyId, liquidfun::BodyId) {
    let body_a = world
        .create_body(&BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.25, true).expect("body A"))
        .expect("body A storage");
    let body_b = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::new(1.0, 2.0), 0.75, true).expect("body B"),
        )
        .expect("body B storage");
    (body_a, body_b)
}

#[test]
fn weld_definition_rejects_invalid_frame_and_softness() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let definition = WeldJointDef::new(body_a, body_b).expect("definition");

    // Act
    let frame = definition.with_frame(Vec2::new(f32::NAN, 0.0), Vec2::ZERO, 0.0);
    let frequency = definition.with_frequency(-1.0);
    let damping = definition.with_damping_ratio(-1.0);

    // Assert
    assert_eq!(frame, Err(JointDefError::NonFiniteValue));
    assert_eq!(frequency, Err(JointDefError::NegativeValue));
    assert_eq!(damping, Err(JointDefError::NegativeValue));
}

#[test]
fn weld_snapshot_reports_frame_and_softness() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let definition = WeldJointDef::new(body_a, body_b)
        .expect("definition")
        .with_frame(Vec2::ZERO, Vec2::ZERO, 0.5)
        .expect("frame")
        .with_frequency(2.0)
        .expect("frequency")
        .with_damping_ratio(0.4)
        .expect("damping");
    let joint = world.create_joint(definition.into()).expect("joint");

    // Act
    let snapshot = world.joint_snapshot(joint).expect("snapshot");
    let JointSpecificSnapshot::Weld(state) = snapshot.specific() else {
        panic!("weld state expected");
    };

    // Assert
    assert_eq!(snapshot.anchor_a(), Vec2::ZERO);
    assert_eq!(snapshot.anchor_b(), Vec2::new(1.0, 2.0));
    assert_eq!(state.reference_angle().to_bits(), 0.5_f32.to_bits());
    assert_eq!(state.frequency().to_bits(), 2.0_f32.to_bits());
    assert_eq!(state.damping_ratio().to_bits(), 0.4_f32.to_bits());
}

#[test]
fn weld_softness_setters_follow_source_no_wake_behavior() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let joint = world
        .create_joint(
            WeldJointDef::new(body_a, body_b)
                .expect("definition")
                .into(),
        )
        .expect("joint");
    world.set_body_awake(body_a, false).expect("sleep A");
    world.set_body_awake(body_b, false).expect("sleep B");

    // Act
    world.set_weld_frequency(joint, 3.0).expect("frequency");
    world.set_weld_damping_ratio(joint, 0.6).expect("damping");

    // Assert
    assert!(!world.body_snapshot(body_a).expect("A").is_awake());
    assert!(!world.body_snapshot(body_b).expect("B").is_awake());
    assert_eq!(world.joint_reaction_force(joint, 60.0), Ok(Vec2::ZERO));
    assert_eq!(world.joint_reaction_torque(joint, 60.0), Ok(0.0));
}
