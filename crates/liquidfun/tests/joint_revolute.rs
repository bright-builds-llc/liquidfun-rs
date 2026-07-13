//! Revolute definition, query, mutation, and runtime contract coverage.

use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyType, JointDefError, JointLimitState, JointMutationError, JointSpecificSnapshot,
    RevoluteJointDef, World,
};

fn bodies(world: &mut World, angle_a: f32, angle_b: f32) -> (liquidfun::BodyId, liquidfun::BodyId) {
    let definition_a = BodyDef::new(BodyType::Dynamic, Vec2::new(1.0, 2.0), angle_a, true)
        .expect("body A definition")
        .with_angular_velocity(1.0)
        .expect("body A velocity");
    let definition_b = BodyDef::new(BodyType::Dynamic, Vec2::new(4.0, 6.0), angle_b, true)
        .expect("body B definition")
        .with_angular_velocity(3.0)
        .expect("body B velocity");
    let body_a = world.create_body(&definition_a).expect("body A");
    let body_b = world.create_body(&definition_b).expect("body B");
    (body_a, body_b)
}

#[test]
fn definition_rejects_invalid_frame_limits_and_motor_cap() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world, 0.0, 0.0);
    let definition = RevoluteJointDef::new(body_a, body_b).expect("base definition");

    // Act
    let non_finite = definition.with_frame(Vec2::new(f32::NAN, 0.0), Vec2::ZERO, 0.0);
    let inverted = definition.with_limits(true, 1.0, -1.0);
    let negative_cap = definition.with_motor(true, 0.0, -1.0);

    // Assert
    assert_eq!(non_finite, Err(JointDefError::NonFiniteValue));
    assert_eq!(inverted, Err(JointDefError::InvalidRange));
    assert_eq!(negative_cap, Err(JointDefError::NegativeValue));
}

#[test]
fn coordinate_speed_anchors_and_reactions_are_semantic() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world, 0.25, 1.5);
    let definition = RevoluteJointDef::new(body_a, body_b)
        .expect("joint definition")
        .with_frame(Vec2::ZERO, Vec2::ZERO, 0.25)
        .expect("frame");
    let joint = world.create_joint(definition.into()).expect("joint");

    // Act
    let snapshot = world.joint_snapshot(joint).expect("snapshot");

    // Assert
    assert_eq!(world.revolute_joint_angle(joint), Ok(1.0));
    assert_eq!(world.revolute_joint_speed(joint), Ok(2.0));
    assert_eq!(snapshot.anchor_a(), Vec2::new(1.0, 2.0));
    assert_eq!(snapshot.anchor_b(), Vec2::new(4.0, 6.0));
    assert_eq!(world.joint_reaction_force(joint, 60.0), Ok(Vec2::ZERO));
    assert_eq!(world.joint_reaction_torque(joint, 60.0), Ok(0.0));
    assert_eq!(world.revolute_motor_torque(joint, 60.0), Ok(0.0));
}

#[test]
fn inactive_lower_upper_and_equal_limits_are_classified() {
    // Arrange
    let mut world = World::new().expect("world");
    let angles = [
        (0.0, -1.0, 1.0),
        (-2.0, -1.0, 1.0),
        (2.0, -1.0, 1.0),
        (0.0, 0.0, 0.0),
    ];
    let mut states = Vec::new();
    for (angle, lower, upper) in angles {
        let (body_a, body_b) = bodies(&mut world, 0.0, angle);
        let definition = RevoluteJointDef::new(body_a, body_b)
            .expect("joint")
            .with_limits(true, lower, upper)
            .expect("limits");
        let joint = world
            .create_joint(definition.into())
            .expect("joint storage");
        let JointSpecificSnapshot::Revolute(state) =
            world.joint_snapshot(joint).expect("snapshot").specific()
        else {
            panic!("revolute state expected");
        };
        states.push(state.limit_state());
    }

    // Act / Assert
    assert_eq!(
        states,
        [
            JointLimitState::Inactive,
            JointLimitState::AtLower,
            JointLimitState::AtUpper,
            JointLimitState::Equal
        ]
    );
}

#[test]
fn changed_limit_wakes_equal_limit_does_not_and_motor_setter_always_wakes() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world, 0.0, 0.0);
    let definition = RevoluteJointDef::new(body_a, body_b)
        .expect("joint")
        .with_limits(true, -1.0, 1.0)
        .expect("limits");
    let joint = world
        .create_joint(definition.into())
        .expect("joint storage");
    world.set_body_awake(body_a, false).expect("sleep A");
    world.set_body_awake(body_b, false).expect("sleep B");

    // Act
    world
        .set_revolute_limits(joint, -1.0, 1.0)
        .expect("equal setter");
    let equal_awake = [
        world.body_snapshot(body_a).expect("A").is_awake(),
        world.body_snapshot(body_b).expect("B").is_awake(),
    ];
    world
        .set_revolute_limits(joint, -2.0, 2.0)
        .expect("changed setter");
    let changed_awake = [
        world.body_snapshot(body_a).expect("A").is_awake(),
        world.body_snapshot(body_b).expect("B").is_awake(),
    ];
    world.set_body_awake(body_a, false).expect("sleep A");
    world.set_body_awake(body_b, false).expect("sleep B");
    world
        .set_revolute_motor_speed(joint, 0.0)
        .expect("source-unconditional setter");

    // Assert
    assert_eq!(equal_awake, [false, false]);
    assert_eq!(changed_awake, [true, true]);
    assert!(world.body_snapshot(body_a).expect("A").is_awake());
    assert!(world.body_snapshot(body_b).expect("B").is_awake());
}

#[test]
fn invalid_limit_is_atomic_and_cold_cache_starts_zero() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world, 0.0, 0.0);
    let joint = world
        .create_joint(RevoluteJointDef::new(body_a, body_b).expect("joint").into())
        .expect("joint storage");
    world.set_body_awake(body_a, false).expect("sleep A");
    world.set_body_awake(body_b, false).expect("sleep B");

    // Act
    let result = world.set_revolute_limits(joint, f32::NAN, 1.0);
    let JointSpecificSnapshot::Revolute(state) =
        world.joint_snapshot(joint).expect("snapshot").specific()
    else {
        panic!("revolute");
    };

    // Assert
    assert_eq!(result, Err(JointMutationError::InvalidValue));
    assert!(!world.body_snapshot(body_a).expect("A").is_awake());
    assert!(!world.body_snapshot(body_b).expect("B").is_awake());
    assert_eq!(state.motor_impulse().to_bits(), 0.0_f32.to_bits());
}
