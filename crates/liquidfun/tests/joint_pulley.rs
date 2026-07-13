//! Pulley definition, query, and runtime contract coverage.

use liquidfun::math::Vec2;
use liquidfun::{BodyDef, BodyType, JointDefError, JointSpecificSnapshot, PulleyJointDef, World};

fn bodies(world: &mut World) -> (liquidfun::BodyId, liquidfun::BodyId) {
    let body_a = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::new(-2.0, 0.0), 0.0, true).expect("body A"),
        )
        .expect("body A storage");
    let body_b = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::new(2.0, 0.0), 0.0, true).expect("body B"),
        )
        .expect("body B storage");
    (body_a, body_b)
}

#[test]
fn definition_rejects_invalid_geometry_lengths_and_ratio() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let definition = PulleyJointDef::new(body_a, body_b).expect("definition");

    // Act
    let invalid_anchor = definition.with_geometry(
        Vec2::new(f32::NAN, 1.0),
        Vec2::new(1.0, 1.0),
        Vec2::ZERO,
        Vec2::ZERO,
        1.0,
        1.0,
        1.0,
    );
    let zero_length = definition.with_geometry(
        Vec2::ZERO,
        Vec2::ZERO,
        Vec2::ZERO,
        Vec2::ZERO,
        0.0,
        1.0,
        1.0,
    );
    let zero_ratio = definition.with_geometry(
        Vec2::ZERO,
        Vec2::ZERO,
        Vec2::ZERO,
        Vec2::ZERO,
        1.0,
        1.0,
        0.0,
    );

    // Assert
    assert_eq!(invalid_anchor, Err(JointDefError::NonFiniteValue));
    assert_eq!(zero_length, Err(JointDefError::NonPositiveValue));
    assert_eq!(zero_ratio, Err(JointDefError::NonPositiveValue));
}

#[test]
fn constant_lengths_anchors_and_reactions_are_semantic() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let definition = PulleyJointDef::new(body_a, body_b)
        .expect("definition")
        .with_geometry(
            Vec2::new(-2.0, 4.0),
            Vec2::new(2.0, 6.0),
            Vec2::ZERO,
            Vec2::ZERO,
            4.0,
            6.0,
            2.0,
        )
        .expect("geometry");
    let joint = world.create_joint(definition.into()).expect("joint");

    // Act
    let snapshot = world.joint_snapshot(joint).expect("snapshot");
    let JointSpecificSnapshot::Pulley(state) = snapshot.specific() else {
        panic!("pulley state expected");
    };

    // Assert
    assert_eq!(snapshot.anchor_a(), Vec2::new(-2.0, 0.0));
    assert_eq!(snapshot.anchor_b(), Vec2::new(2.0, 0.0));
    assert_eq!(state.ground_anchor_a(), Vec2::new(-2.0, 4.0));
    assert_eq!(state.ground_anchor_b(), Vec2::new(2.0, 6.0));
    assert_eq!(state.current_length_a().to_bits(), 4.0_f32.to_bits());
    assert_eq!(state.current_length_b().to_bits(), 6.0_f32.to_bits());
    assert_eq!(state.constant().to_bits(), 16.0_f32.to_bits());
    assert_eq!(state.ratio().to_bits(), 2.0_f32.to_bits());
    assert_eq!(world.joint_reaction_force(joint, 60.0), Ok(Vec2::ZERO));
    assert_eq!(world.joint_reaction_torque(joint, 60.0), Ok(0.0));
}
