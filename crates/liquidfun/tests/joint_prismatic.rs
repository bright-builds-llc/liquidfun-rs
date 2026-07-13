//! Prismatic definition, query, mutation, and runtime contract coverage.

use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyType, JointDefError, JointLimitState, JointMutationError, JointSpecificSnapshot,
    PrismaticJointDef, World,
};

fn bodies(world: &mut World, position_b: Vec2) -> (liquidfun::BodyId, liquidfun::BodyId) {
    let definition_a = BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)
        .expect("body A")
        .with_linear_velocity(Vec2::new(1.0, 0.0))
        .expect("velocity A");
    let definition_b = BodyDef::new(BodyType::Dynamic, position_b, 0.0, true)
        .expect("body B")
        .with_linear_velocity(Vec2::new(4.0, 0.0))
        .expect("velocity B");
    let body_a = world.create_body(&definition_a).expect("body A storage");
    let body_b = world.create_body(&definition_b).expect("body B storage");
    (body_a, body_b)
}

#[test]
fn axis_is_normalized_and_zero_or_nonfinite_axes_are_rejected() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world, Vec2::ZERO);
    let definition = PrismaticJointDef::new(body_a, body_b).expect("joint");

    // Act
    let normalized = definition
        .with_frame(Vec2::ZERO, Vec2::ZERO, Vec2::new(2.0, 0.0), 0.0)
        .expect("axis");
    let zero = definition.with_frame(Vec2::ZERO, Vec2::ZERO, Vec2::ZERO, 0.0);
    let non_finite =
        definition.with_frame(Vec2::ZERO, Vec2::ZERO, Vec2::new(f32::INFINITY, 0.0), 0.0);

    // Assert
    assert_eq!(normalized.local_axis_a(), Vec2::new(1.0, 0.0));
    assert_eq!(zero, Err(JointDefError::InvalidAxis));
    assert_eq!(non_finite, Err(JointDefError::NonFiniteValue));
}

#[test]
fn gear_coordinate_speed_anchors_and_reactions_are_semantic() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world, Vec2::new(3.0, 2.0));
    let definition = PrismaticJointDef::new(body_a, body_b)
        .expect("joint")
        .with_frame(Vec2::ZERO, Vec2::ZERO, Vec2::new(2.0, 0.0), 0.0)
        .expect("frame");
    let joint = world
        .create_joint(definition.into())
        .expect("joint storage");

    // Act
    let snapshot = world.joint_snapshot(joint).expect("snapshot");

    // Assert
    assert_eq!(world.prismatic_joint_translation(joint), Ok(3.0));
    assert_eq!(world.prismatic_joint_speed(joint), Ok(3.0));
    assert_eq!(snapshot.anchor_a(), Vec2::ZERO);
    assert_eq!(snapshot.anchor_b(), Vec2::new(3.0, 2.0));
    assert_eq!(world.joint_reaction_force(joint, 60.0), Ok(Vec2::ZERO));
    assert_eq!(world.joint_reaction_torque(joint, 60.0), Ok(0.0));
    assert_eq!(world.prismatic_motor_force(joint, 60.0), Ok(0.0));
}

#[test]
fn inactive_lower_upper_and_equal_limits_are_classified() {
    // Arrange
    let mut world = World::new().expect("world");
    let cases = [
        (0.0, -1.0, 1.0),
        (-2.0, -1.0, 1.0),
        (2.0, -1.0, 1.0),
        (0.0, 0.0, 0.0),
    ];
    let mut states = Vec::new();
    for (translation, lower, upper) in cases {
        let (body_a, body_b) = bodies(&mut world, Vec2::new(translation, 0.0));
        let definition = PrismaticJointDef::new(body_a, body_b)
            .expect("joint")
            .with_limits(true, lower, upper)
            .expect("limits");
        let joint = world
            .create_joint(definition.into())
            .expect("joint storage");
        let JointSpecificSnapshot::Prismatic(state) =
            world.joint_snapshot(joint).expect("snapshot").specific()
        else {
            panic!("prismatic");
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
fn wake_cache_and_motor_setter_branches_match_source() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world, Vec2::ZERO);
    let definition = PrismaticJointDef::new(body_a, body_b)
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
        .set_prismatic_limits(joint, -1.0, 1.0)
        .expect("equal limits");
    let equal_awake = world.body_snapshot(body_a).expect("A").is_awake();
    world
        .set_prismatic_limits(joint, -2.0, 2.0)
        .expect("changed limits");
    let changed_awake = world.body_snapshot(body_a).expect("A").is_awake();
    world.set_body_awake(body_a, false).expect("sleep A");
    world.set_body_awake(body_b, false).expect("sleep B");
    world
        .set_prismatic_max_motor_force(joint, 0.0)
        .expect("source-unconditional setter");

    // Assert
    assert!(!equal_awake);
    assert!(changed_awake);
    assert!(world.body_snapshot(body_a).expect("A").is_awake());
    assert!(world.body_snapshot(body_b).expect("B").is_awake());
}

#[test]
fn invalid_motor_force_is_atomic_and_cold_cache_starts_zero() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world, Vec2::ZERO);
    let joint = world
        .create_joint(
            PrismaticJointDef::new(body_a, body_b)
                .expect("joint")
                .into(),
        )
        .expect("joint storage");
    world.set_body_awake(body_a, false).expect("sleep A");
    world.set_body_awake(body_b, false).expect("sleep B");

    // Act
    let result = world.set_prismatic_max_motor_force(joint, f32::MAX);
    let overflow = world.set_prismatic_motor_speed(joint, f32::INFINITY);
    let JointSpecificSnapshot::Prismatic(state) =
        world.joint_snapshot(joint).expect("snapshot").specific()
    else {
        panic!("prismatic");
    };

    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(overflow, Err(JointMutationError::InvalidValue));
    assert_eq!(state.motor_impulse().to_bits(), 0.0_f32.to_bits());
}
