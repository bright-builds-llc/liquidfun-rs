//! Distance definition, query, mutation, and runtime contract coverage.

use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyType, DistanceJointDef, JointDefError, JointMutationError, JointSpecificSnapshot,
    World,
};

fn bodies(world: &mut World) -> (liquidfun::BodyId, liquidfun::BodyId) {
    let body_a = world
        .create_body(&BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true).expect("body A"))
        .expect("body A storage");
    let body_b = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::new(3.0, 4.0), 0.0, true).expect("body B"),
        )
        .expect("body B storage");
    (body_a, body_b)
}

#[test]
fn definition_rejects_invalid_length_anchors_and_tuning() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let definition = DistanceJointDef::new(body_a, body_b).expect("definition");

    // Act
    let zero_length = definition.with_length(0.0);
    let invalid_anchor = definition.with_anchors(Vec2::new(f32::NAN, 0.0), Vec2::ZERO);
    let negative_frequency = definition.with_frequency(-1.0);
    let negative_damping = definition.with_damping_ratio(-1.0);

    // Assert
    assert_eq!(zero_length, Err(JointDefError::NonPositiveValue));
    assert_eq!(invalid_anchor, Err(JointDefError::NonFiniteValue));
    assert_eq!(negative_frequency, Err(JointDefError::NegativeValue));
    assert_eq!(negative_damping, Err(JointDefError::NegativeValue));
}

#[test]
fn current_length_anchors_and_cold_reaction_are_semantic() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let definition = DistanceJointDef::new(body_a, body_b)
        .expect("definition")
        .with_length(5.0)
        .expect("length");
    let joint = world.create_joint(definition.into()).expect("joint");

    // Act
    let snapshot = world.joint_snapshot(joint).expect("snapshot");
    let JointSpecificSnapshot::Distance(state) = snapshot.specific() else {
        panic!("distance state expected");
    };

    // Assert
    assert_eq!(snapshot.anchor_a(), Vec2::ZERO);
    assert_eq!(snapshot.anchor_b(), Vec2::new(3.0, 4.0));
    assert_eq!(state.current_length().to_bits(), 5.0_f32.to_bits());
    assert_eq!(state.length().to_bits(), 5.0_f32.to_bits());
    assert_eq!(world.joint_reaction_force(joint, 60.0), Ok(Vec2::ZERO));
    assert_eq!(world.joint_reaction_torque(joint, 60.0), Ok(0.0));
}

#[test]
fn distance_setters_preserve_source_no_wake_behavior() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let joint = world
        .create_joint(
            DistanceJointDef::new(body_a, body_b)
                .expect("definition")
                .into(),
        )
        .expect("joint");
    world.set_body_awake(body_a, false).expect("sleep A");
    world.set_body_awake(body_b, false).expect("sleep B");

    // Act
    world.set_distance_length(joint, 2.0).expect("length");
    world.set_distance_frequency(joint, 3.0).expect("frequency");
    world
        .set_distance_damping_ratio(joint, 0.5)
        .expect("damping");

    // Assert
    assert!(!world.body_snapshot(body_a).expect("A").is_awake());
    assert!(!world.body_snapshot(body_b).expect("B").is_awake());
    let JointSpecificSnapshot::Distance(state) =
        world.joint_snapshot(joint).expect("snapshot").specific()
    else {
        panic!("distance state expected");
    };
    assert_eq!(state.length().to_bits(), 2.0_f32.to_bits());
    assert_eq!(state.frequency().to_bits(), 3.0_f32.to_bits());
    assert_eq!(state.damping_ratio().to_bits(), 0.5_f32.to_bits());
}

#[test]
fn invalid_distance_mutation_is_atomic() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let joint = world
        .create_joint(
            DistanceJointDef::new(body_a, body_b)
                .expect("definition")
                .into(),
        )
        .expect("joint");

    // Act
    let result = world.set_distance_length(joint, f32::INFINITY);

    // Assert
    assert_eq!(result, Err(JointMutationError::InvalidValue));
    let JointSpecificSnapshot::Distance(state) =
        world.joint_snapshot(joint).expect("snapshot").specific()
    else {
        panic!("distance state expected");
    };
    assert_eq!(state.length().to_bits(), 1.0_f32.to_bits());
}
