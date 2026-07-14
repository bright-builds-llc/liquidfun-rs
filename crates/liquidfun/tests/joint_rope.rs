//! `RopeJoint` contract coverage, deliberately separate from standalone rope.

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyType, FixtureDef, JointDefError, JointLimitState, JointSpecificSnapshot,
    RopeJointDef, StepConfiguration, StepHook, StepLimits, World,
};

struct NoopHook;
impl StepHook for NoopHook {}

fn attach_mass(world: &mut World, body: liquidfun::BodyId) {
    let fixture = FixtureDef::new(
        Shape::from(CircleShape::new(Vec2::ZERO, 0.5).expect("circle")),
        1.0,
        0.0,
        0.0,
        false,
        FilterData::default(),
    )
    .expect("fixture");
    world.create_fixture(body, &fixture).expect("mass fixture");
}

fn bodies(world: &mut World) -> (liquidfun::BodyId, liquidfun::BodyId) {
    let body_a = world
        .create_body(&BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true).expect("A"))
        .expect("A storage");
    let body_b = world
        .create_body(&BodyDef::new(BodyType::Dynamic, Vec2::new(4.0, 0.0), 0.0, true).expect("B"))
        .expect("B storage");
    (body_a, body_b)
}

#[test]
fn rope_joint_definition_requires_finite_anchors_and_positive_length() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let definition = RopeJointDef::new(body_a, body_b).expect("definition");

    // Act
    let anchors = definition.with_anchors(Vec2::new(f32::NAN, 0.0), Vec2::ZERO);
    let length = definition.with_max_length(0.0);

    // Assert
    assert_eq!(anchors, Err(JointDefError::NonFiniteValue));
    assert_eq!(length, Err(JointDefError::NonPositiveValue));
}

#[test]
fn rope_joint_classifies_inactive_and_upper_limit_states() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let long_joint = world
        .create_joint(
            RopeJointDef::new(body_a, body_b)
                .expect("definition")
                .with_max_length(10.0)
                .expect("long")
                .into(),
        )
        .expect("long joint");
    let short_joint = world
        .create_joint(
            RopeJointDef::new(body_a, body_b)
                .expect("definition")
                .with_max_length(2.0)
                .expect("short")
                .into(),
        )
        .expect("short joint");

    // Act
    let JointSpecificSnapshot::Rope(long_state) = world
        .joint_snapshot(long_joint)
        .expect("long snapshot")
        .specific()
    else {
        panic!("rope state expected");
    };
    let JointSpecificSnapshot::Rope(short_state) = world
        .joint_snapshot(short_joint)
        .expect("short snapshot")
        .specific()
    else {
        panic!("rope state expected");
    };

    // Assert
    assert_eq!(long_state.limit_state(), JointLimitState::Inactive);
    assert_eq!(short_state.limit_state(), JointLimitState::AtUpper);
    assert_eq!(short_state.current_length().to_bits(), 6.0_f32.to_bits());
}

#[test]
fn rope_joint_max_length_setter_is_no_wake_and_name_is_distinct() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    let joint = world
        .create_joint(
            RopeJointDef::new(body_a, body_b)
                .expect("definition")
                .into(),
        )
        .expect("joint");
    world.set_body_awake(body_a, false).expect("sleep A");
    world.set_body_awake(body_b, false).expect("sleep B");

    // Act
    world.set_rope_joint_max_length(joint, 8.0).expect("length");

    // Assert
    assert!(!world.body_snapshot(body_a).expect("A").is_awake());
    assert!(!world.body_snapshot(body_b).expect("B").is_awake());
    assert_eq!(
        std::any::type_name::<RopeJointDef>().rsplit("::").next(),
        Some("RopeJointDef")
    );
}

#[test]
fn rope_world_step_applies_only_an_upper_limit_reaction() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    attach_mass(&mut world, body_a);
    attach_mass(&mut world, body_b);
    let inactive = world
        .create_joint(
            RopeJointDef::new(body_a, body_b)
                .expect("definition")
                .with_max_length(10.0)
                .expect("inactive length")
                .into(),
        )
        .expect("inactive joint");
    let upper = world
        .create_joint(
            RopeJointDef::new(body_a, body_b)
                .expect("definition")
                .with_max_length(2.0)
                .expect("upper length")
                .into(),
        )
        .expect("upper joint");
    world
        .set_body_linear_velocity(body_b, Vec2::new(3.0, 0.0))
        .expect("velocity");
    let mut hook = NoopHook;

    // Act
    world
        .step(
            StepConfiguration::new(1.0 / 60.0, 8, 3).expect("configuration"),
            &mut hook,
            StepLimits::default(),
        )
        .expect("step");

    // Assert
    assert_eq!(world.joint_reaction_force(inactive, 60.0), Ok(Vec2::ZERO));
    assert_ne!(world.joint_reaction_force(upper, 60.0), Ok(Vec2::ZERO));
}
