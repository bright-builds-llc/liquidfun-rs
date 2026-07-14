//! Pulley definition, query, and runtime contract coverage.

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyType, FixtureDef, JointDefError, JointSpecificSnapshot, PulleyJointDef,
    StepConfiguration, StepHook, StepLimits, World,
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

#[test]
fn live_non_unit_ratio_and_rotating_anchors_populate_warm_reaction() {
    // Arrange
    let mut world = World::new().expect("world");
    let body_a = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::new(-2.0, 0.0), 0.25, true)
                .expect("body A")
                .with_angular_velocity(2.0)
                .expect("angular velocity A"),
        )
        .expect("body A storage");
    let body_b = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::new(2.0, 0.5), -0.5, true)
                .expect("body B")
                .with_angular_velocity(-3.0)
                .expect("angular velocity B"),
        )
        .expect("body B storage");
    attach_mass(&mut world, body_a);
    attach_mass(&mut world, body_b);
    let definition = PulleyJointDef::new(body_a, body_b)
        .expect("pulley")
        .with_geometry(
            Vec2::new(-3.0, 4.0),
            Vec2::new(3.0, 5.0),
            Vec2::new(0.5, -0.25),
            Vec2::new(-0.25, 0.5),
            4.0,
            5.0,
            2.0,
        )
        .expect("geometry");
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
    assert_ne!(cold_reaction, Vec2::ZERO);
    assert!(cold_reaction.is_valid());
    assert!(warm_reaction.is_valid());
}
