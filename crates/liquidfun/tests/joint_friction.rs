//! Friction-joint checked configuration and cap coverage.

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyType, FixtureDef, FrictionJointDef, JointDefError, JointSpecificSnapshot,
    StepConfiguration, StepHook, StepLimits, World,
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

#[test]
fn friction_world_step_caps_live_linear_and_angular_impulses() {
    // Arrange
    let mut world = World::new().expect("world");
    let (body_a, body_b) = bodies(&mut world);
    attach_mass(&mut world, body_a);
    attach_mass(&mut world, body_b);
    let joint = world
        .create_joint(
            FrictionJointDef::new(body_a, body_b)
                .expect("definition")
                .with_anchors(Vec2::new(0.5, 0.0), Vec2::new(-0.5, 0.25))
                .expect("anchors")
                .with_max_force(3.0)
                .expect("force")
                .with_max_torque(2.0)
                .expect("torque")
                .into(),
        )
        .expect("joint");
    world
        .set_body_linear_velocity(body_b, Vec2::new(100.0, -50.0))
        .expect("linear velocity");
    world
        .set_body_angular_velocity(body_b, 100.0)
        .expect("angular velocity");
    let mut hook = NoopHook;

    // Act
    world
        .step(
            StepConfiguration::new(1.0 / 60.0, 1, 1).expect("configuration"),
            &mut hook,
            StepLimits::default(),
        )
        .expect("step");
    let force = world.joint_reaction_force(joint, 60.0).expect("force");
    let torque = world.joint_reaction_torque(joint, 60.0).expect("torque");

    // Assert
    assert!(
        force.length() > 0.0 && force.length() <= 3.0 + f32::EPSILON * 4.0,
        "unexpected friction force {force:?}"
    );
    assert!(
        torque.abs() > 0.0 && torque.abs() <= 2.0,
        "unexpected friction torque {torque}"
    );
}
