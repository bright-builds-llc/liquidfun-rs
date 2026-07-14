//! Distance definition, query, mutation, and runtime contract coverage.

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyType, DistanceJointDef, FixtureDef, JointDefError, JointMutationError,
    JointSpecificSnapshot, StepConfiguration, StepHook, StepLimits, World,
};

struct NoopHook;

impl StepHook for NoopHook {}

fn attach_mass(world: &mut World, body: liquidfun::BodyId) {
    let shape = Shape::from(CircleShape::new(Vec2::new(0.25, 0.0), 0.5).expect("circle"));
    let fixture = FixtureDef::new(shape, 1.0, 0.0, 0.0, false, FilterData::default())
        .expect("fixture definition");
    world.create_fixture(body, &fixture).expect("fixture");
}

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

#[test]
fn live_rigid_and_soft_off_center_steps_populate_warm_reactions() {
    for frequency in [0.0, 3.0] {
        // Arrange
        let mut world = World::new().expect("world");
        let body_a = world
            .create_body(
                &BodyDef::new(BodyType::Static, Vec2::ZERO, 0.25, true).expect("static body"),
            )
            .expect("body A");
        let body_b = world
            .create_body(
                &BodyDef::new(BodyType::Dynamic, Vec2::new(2.0, 0.5), -0.5, true)
                    .expect("dynamic body")
                    .with_linear_velocity(Vec2::new(-3.0, 1.0))
                    .expect("linear velocity")
                    .with_angular_velocity(2.0)
                    .expect("angular velocity"),
            )
            .expect("body B");
        attach_mass(&mut world, body_b);
        let definition = DistanceJointDef::new(body_a, body_b)
            .expect("distance")
            .with_anchors(Vec2::new(0.5, -0.25), Vec2::new(-0.5, 0.5))
            .expect("anchors")
            .with_length(1.0)
            .expect("length")
            .with_frequency(frequency)
            .expect("frequency")
            .with_damping_ratio(0.5)
            .expect("damping");
        let joint = world.create_joint(definition.into()).expect("joint");
        let step = StepConfiguration::new(1.0 / 60.0, 8, 3).expect("step");
        let mut hook = NoopHook;

        // Act
        world
            .step(step, &mut hook, StepLimits::default())
            .expect("cold step");
        let cold_reaction = world.joint_reaction_force(joint, 60.0).expect("reaction");
        world
            .step(step, &mut hook, StepLimits::default())
            .expect("warm step");
        let warm_reaction = world.joint_reaction_force(joint, 60.0).expect("reaction");

        // Assert
        assert_ne!(cold_reaction, Vec2::ZERO, "frequency {frequency}");
        assert!(cold_reaction.is_valid(), "frequency {frequency}");
        assert!(warm_reaction.is_valid(), "frequency {frequency}");
    }
}
