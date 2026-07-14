#![cfg(feature = "differential-internals")]

//! Joint collision suppression and selective origin-shift regressions.

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyType, FixtureDef, JointDef, JointSpecificSnapshot, MouseJointDef,
    OriginShiftError, PulleyJointDef, RevoluteJointDef, StepConfiguration, StepHook, StepLimits,
    World,
};

struct NoopHook;

impl StepHook for NoopHook {}

fn body(body_type: BodyType, position: Vec2) -> BodyDef {
    BodyDef::new(body_type, position, 0.0, true).expect("body definition should be valid")
}

fn circle() -> FixtureDef {
    FixtureDef::new(
        Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("circle should be valid")),
        1.0,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .expect("fixture definition should be valid")
}

fn step(world: &mut World) {
    let mut hook = NoopHook;
    world
        .step(
            StepConfiguration::new(0.0, 1, 1).expect("zero step should be valid"),
            &mut hook,
            StepLimits::default(),
        )
        .expect("contact maintenance should succeed");
}

#[test]
fn collision_suppression_refilters_existing_and_future_pairs_until_last_joint_is_removed() {
    // Arrange
    let mut world = World::new().expect("test world should be available");
    let body_a = world
        .create_body(&body(BodyType::Dynamic, Vec2::ZERO))
        .expect("body A should fit");
    let body_b = world
        .create_body(&body(BodyType::Dynamic, Vec2::new(1.0, 0.0)))
        .expect("body B should fit");
    world.create_fixture(body_a, &circle()).expect("fixture A");
    world.create_fixture(body_b, &circle()).expect("fixture B");
    step(&mut world);
    assert_eq!(world.rigid_contact_diagnostics().len(), 1);

    // Act
    let definition =
        JointDef::from(RevoluteJointDef::new(body_a, body_b).expect("joint should be valid"));
    let first = world.create_joint(definition).expect("first suppressor");
    let second = world.create_joint(definition).expect("second suppressor");
    step(&mut world);
    let while_suppressed = world.rigid_contact_diagnostics().len();
    world
        .destroy_joint(second)
        .expect("second suppressor should be live");
    step(&mut world);
    let after_one = world.rigid_contact_diagnostics().len();
    world
        .destroy_joint(first)
        .expect("first suppressor should be live");
    step(&mut world);

    // Assert
    assert_eq!(while_suppressed, 0);
    assert_eq!(after_one, 0);
    assert_eq!(world.rigid_contact_diagnostics().len(), 1);
}

#[test]
fn origin_shift_changes_only_joint_world_space_coordinates() {
    // Arrange
    let mut world = World::new().expect("test world should be available");
    let body_a = world
        .create_body(&body(BodyType::Static, Vec2::new(-2.0, 1.0)))
        .expect("body A should fit");
    let body_b = world
        .create_body(&body(BodyType::Dynamic, Vec2::new(3.0, -1.0)))
        .expect("body B should fit");
    let pulley_definition = PulleyJointDef::new(body_a, body_b)
        .expect("pulley should be valid")
        .with_geometry(
            Vec2::new(-4.0, 5.0),
            Vec2::new(6.0, 5.0),
            Vec2::ZERO,
            Vec2::ZERO,
            1.0,
            1.0,
            1.0,
        )
        .expect("pulley geometry should be valid");
    let mouse_definition = MouseJointDef::new(body_a, body_b)
        .expect("mouse should be valid")
        .with_target(Vec2::new(7.0, 8.0))
        .expect("mouse target should be valid");
    let pulley = world
        .create_joint(JointDef::from(pulley_definition))
        .expect("pulley should fit");
    let mouse = world
        .create_joint(JointDef::from(mouse_definition))
        .expect("mouse should fit");
    let shift = Vec2::new(2.0, -3.0);

    // Act
    world
        .shift_origin(shift)
        .expect("origin shift should succeed");

    // Assert
    let JointSpecificSnapshot::Pulley(pulley_state) = world
        .joint_snapshot(pulley)
        .expect("pulley should remain live")
        .specific()
    else {
        panic!("expected pulley snapshot");
    };
    let JointSpecificSnapshot::Mouse(mouse_state) = world
        .joint_snapshot(mouse)
        .expect("mouse should remain live")
        .specific()
    else {
        panic!("expected mouse snapshot");
    };
    assert_eq!(pulley_state.ground_anchor_a(), Vec2::new(-4.0, 5.0) - shift);
    assert_eq!(pulley_state.ground_anchor_b(), Vec2::new(6.0, 5.0) - shift);
    assert_eq!(mouse_state.target(), Vec2::new(7.0, 8.0) - shift);
}

#[test]
fn overflowing_joint_origin_candidate_rejects_the_complete_shift() {
    // Arrange
    let mut world = World::new().expect("test world should be available");
    let body_a = world
        .create_body(&body(BodyType::Static, Vec2::ZERO))
        .expect("body A should fit");
    let body_b = world
        .create_body(&body(BodyType::Dynamic, Vec2::ZERO))
        .expect("body B should fit");
    let joint = world
        .create_joint(
            MouseJointDef::new(body_a, body_b)
                .expect("mouse should be valid")
                .with_target(Vec2::new(f32::MAX, 0.0))
                .expect("finite target should be valid")
                .into(),
        )
        .expect("mouse should fit");
    let body_before = world.body_snapshot(body_b).expect("body should be live");
    let joint_before = world.joint_snapshot(joint).expect("joint should be live");

    // Act
    let result = world.shift_origin(Vec2::new(-f32::MAX, 0.0));

    // Assert
    assert_eq!(result, Err(OriginShiftError::NonFiniteJointState));
    assert_eq!(world.body_snapshot(body_b), Ok(body_before));
    assert_eq!(world.joint_snapshot(joint), Ok(joint_before));
}
