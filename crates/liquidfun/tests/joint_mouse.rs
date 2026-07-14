//! Mouse definition, query, mutation, and runtime contract coverage.

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyType, FixtureDef, JointDefError, JointMutationError, JointSpecificSnapshot,
    MouseJointDef, StepConfiguration, StepHook, StepLimits, World,
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
        .create_body(&BodyDef::default())
        .expect("body A storage");
    let body_b = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::new(2.0, 3.0), 0.0, true).expect("body B"),
        )
        .expect("body B storage");
    (body_a, body_b)
}

#[test]
fn definition_rejects_invalid_target_force_and_tuning() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let definition = MouseJointDef::new(body_a, body_b).expect("definition");

    // Act
    let target = definition.with_target(Vec2::new(f32::NAN, 0.0));
    let force = definition.with_max_force(-1.0);
    let frequency = definition.with_frequency(-1.0);
    let damping = definition.with_damping_ratio(-1.0);

    // Assert
    assert_eq!(target, Err(JointDefError::NonFiniteValue));
    assert_eq!(force, Err(JointDefError::NegativeValue));
    assert_eq!(frequency, Err(JointDefError::NegativeValue));
    assert_eq!(damping, Err(JointDefError::NegativeValue));
}

#[test]
fn target_anchor_configuration_and_cold_reaction_are_semantic() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let definition = MouseJointDef::new(body_a, body_b)
        .expect("definition")
        .with_target(Vec2::new(4.0, 6.0))
        .expect("target")
        .with_max_force(20.0)
        .expect("force");
    let joint = world.create_joint(definition.into()).expect("joint");

    // Act
    let snapshot = world.joint_snapshot(joint).expect("snapshot");
    let JointSpecificSnapshot::Mouse(state) = snapshot.specific() else {
        panic!("mouse state expected");
    };

    // Assert
    assert_eq!(snapshot.anchor_a(), Vec2::new(4.0, 6.0));
    assert_eq!(snapshot.anchor_b(), Vec2::new(4.0, 6.0));
    assert_eq!(state.target(), Vec2::new(4.0, 6.0));
    assert_eq!(state.max_force().to_bits(), 20.0_f32.to_bits());
    assert_eq!(state.frequency().to_bits(), 5.0_f32.to_bits());
    assert_eq!(state.damping_ratio().to_bits(), 0.7_f32.to_bits());
    assert_eq!(world.joint_reaction_force(joint, 60.0), Ok(Vec2::ZERO));
    assert_eq!(world.joint_reaction_torque(joint, 60.0), Ok(0.0));
}

#[test]
fn target_wakes_only_body_b_and_tuning_setters_do_not_wake() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let joint = world
        .create_joint(
            MouseJointDef::new(body_a, body_b)
                .expect("definition")
                .into(),
        )
        .expect("joint");
    world.set_body_awake(body_a, false).expect("sleep A");
    world.set_body_awake(body_b, false).expect("sleep B");

    // Act
    world
        .set_mouse_target(joint, Vec2::new(5.0, 7.0))
        .expect("target");
    let after_target = [
        world.body_snapshot(body_a).expect("A").is_awake(),
        world.body_snapshot(body_b).expect("B").is_awake(),
    ];
    world.set_body_awake(body_b, false).expect("sleep B");
    world.set_mouse_max_force(joint, 10.0).expect("force");
    world.set_mouse_frequency(joint, 2.0).expect("frequency");
    world.set_mouse_damping_ratio(joint, 0.25).expect("damping");

    // Assert
    assert_eq!(after_target, [false, true]);
    assert!(!world.body_snapshot(body_b).expect("B").is_awake());
}

#[test]
fn invalid_mouse_mutation_is_atomic() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let joint = world
        .create_joint(
            MouseJointDef::new(body_a, body_b)
                .expect("definition")
                .into(),
        )
        .expect("joint");

    // Act
    let result = world.set_mouse_target(joint, Vec2::new(f32::INFINITY, 0.0));

    // Assert
    assert_eq!(result, Err(JointMutationError::InvalidValue));
    let JointSpecificSnapshot::Mouse(state) =
        world.joint_snapshot(joint).expect("snapshot").specific()
    else {
        panic!("mouse state expected");
    };
    assert_eq!(state.target(), Vec2::ZERO);
}

#[test]
fn live_target_force_cap_and_warm_start_affect_only_body_b() {
    // Arrange
    let mut world = World::new().expect("world");
    let body_a = world
        .create_body(
            &BodyDef::new(BodyType::Static, Vec2::new(-4.0, 1.0), 0.5, true).expect("body A"),
        )
        .expect("body A storage");
    let body_b = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::new(2.0, 3.0), -0.25, true)
                .expect("body B")
                .with_angular_velocity(4.0)
                .expect("angular velocity"),
        )
        .expect("body B storage");
    attach_mass(&mut world, body_b);
    let definition = MouseJointDef::new(body_a, body_b)
        .expect("mouse")
        .with_target(Vec2::new(2.5, 3.25))
        .expect("initial target")
        .with_max_force(3.0)
        .expect("force cap")
        .with_frequency(4.0)
        .expect("frequency")
        .with_damping_ratio(0.4)
        .expect("damping");
    let joint = world.create_joint(definition.into()).expect("joint");
    world
        .set_mouse_target(joint, Vec2::new(6.0, -1.0))
        .expect("target update");
    let body_a_before = world.body_snapshot(body_a).expect("body A snapshot");
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
    assert_eq!(world.body_snapshot(body_a).expect("body A"), body_a_before);
    assert_ne!(cold_reaction, Vec2::ZERO);
    assert!(
        cold_reaction.length() <= 3.0 + 8.0 * f32::EPSILON,
        "cold reaction exceeded cap: {cold_reaction:?}"
    );
    assert!(
        warm_reaction.length() <= 3.0 + 8.0 * f32::EPSILON,
        "warm reaction exceeded cap: {warm_reaction:?}"
    );
}
