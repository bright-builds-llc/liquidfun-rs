//! Integration coverage for the closed shared joint contract.

use liquidfun::{
    BodyDef, DestroyedId, DistanceJointDef, FrictionJointDef, GearJointDef, JointDef, JointKind,
    JointMutationError, JointQueryError, MotorJointDef, MouseJointDef, PrismaticJointDef,
    PulleyJointDef, RevoluteJointDef, RopeJointDef, WeldJointDef, WheelJointDef, World,
};

fn test_world_with_bodies() -> (World, liquidfun::BodyId, liquidfun::BodyId) {
    let mut world = World::new().expect("test world key should remain available");
    let body_a = world
        .create_body(&BodyDef::default())
        .expect("body A should fit");
    let body_b = world
        .create_body(&BodyDef::default())
        .expect("body B should fit");
    (world, body_a, body_b)
}

#[test]
fn closed_definition_contract_creates_all_eleven_joint_kinds() {
    // Arrange
    let (mut world, body_a, body_b) = test_world_with_bodies();
    let gear_body_a = world
        .create_body(&BodyDef::default())
        .expect("gear body A should fit");
    let gear_body_b = world
        .create_body(&BodyDef::default())
        .expect("gear body B should fit");
    let gear_source1 = world
        .create_joint(
            RevoluteJointDef::new(body_a, gear_body_a)
                .expect("gear source 1 endpoints")
                .into(),
        )
        .expect("gear source 1 should fit");
    let gear_source2 = world
        .create_joint(
            PrismaticJointDef::new(body_b, gear_body_b)
                .expect("gear source 2 endpoints")
                .into(),
        )
        .expect("gear source 2 should fit");
    let definitions = [
        JointDef::from(RevoluteJointDef::new(body_a, body_b).expect("valid endpoints")),
        JointDef::from(PrismaticJointDef::new(body_a, body_b).expect("valid endpoints")),
        JointDef::from(DistanceJointDef::new(body_a, body_b).expect("valid endpoints")),
        JointDef::from(PulleyJointDef::new(body_a, body_b).expect("valid endpoints")),
        JointDef::from(MouseJointDef::new(body_a, body_b).expect("valid endpoints")),
        JointDef::from(GearJointDef::new(gear_source1, gear_source2).expect("valid dependencies")),
        JointDef::from(WheelJointDef::new(body_a, body_b).expect("valid endpoints")),
        JointDef::from(WeldJointDef::new(body_a, body_b).expect("valid endpoints")),
        JointDef::from(FrictionJointDef::new(body_a, body_b).expect("valid endpoints")),
        JointDef::from(RopeJointDef::new(body_a, body_b).expect("valid endpoints")),
        JointDef::from(MotorJointDef::new(body_a, body_b).expect("valid endpoints")),
    ];
    let expected = [
        JointKind::Revolute,
        JointKind::Prismatic,
        JointKind::Distance,
        JointKind::Pulley,
        JointKind::Mouse,
        JointKind::Gear,
        JointKind::Wheel,
        JointKind::Weld,
        JointKind::Friction,
        JointKind::Rope,
        JointKind::Motor,
    ];

    // Act
    let actual = definitions
        .into_iter()
        .map(|definition| {
            let joint = world.create_joint(definition).expect("joint should fit");
            world
                .joint_snapshot(joint)
                .expect("created joint should remain live")
                .kind()
        })
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(actual, expected);
}

#[test]
fn stale_and_cross_world_joint_queries_preserve_live_state() {
    // Arrange
    let (mut world, body_a, body_b) = test_world_with_bodies();
    let joint = world
        .create_joint(
            RevoluteJointDef::new(body_a, body_b)
                .expect("valid endpoints")
                .into(),
        )
        .expect("joint should fit");
    let before = world.joint_snapshot(joint).expect("joint should be live");
    let mut other = World::new().expect("second world key should remain available");
    let other_a = other
        .create_body(&BodyDef::default())
        .expect("other body A should fit");
    let other_b = other
        .create_body(&BodyDef::default())
        .expect("other body B should fit");
    let foreign = other
        .create_joint(
            RevoluteJointDef::new(other_a, other_b)
                .expect("valid endpoints")
                .into(),
        )
        .expect("foreign joint should fit");

    // Act
    let cross_world = world.joint_snapshot(foreign);
    world.destroy_joint(joint).expect("joint should be live");
    let stale = world.joint_snapshot(joint);

    // Assert
    assert_eq!(
        cross_world,
        Err(JointQueryError::InvalidHandle(
            liquidfun::HandleError::WrongWorld
        ))
    );
    assert_eq!(
        stale,
        Err(JointQueryError::InvalidHandle(
            liquidfun::HandleError::StaleOrDestroyed
        ))
    );
    assert_eq!(before.kind(), JointKind::Revolute);
}

#[test]
fn kind_checked_queries_reject_mismatch_without_effects() {
    // Arrange
    let (mut world, body_a, body_b) = test_world_with_bodies();
    let joint = world
        .create_joint(
            DistanceJointDef::new(body_a, body_b)
                .expect("valid endpoints")
                .into(),
        )
        .expect("joint should fit");
    let before = world.joint_snapshot(joint).expect("joint should be live");

    // Act
    let result = world.joint_snapshot_of_kind(joint, JointKind::Revolute);

    // Assert
    assert_eq!(
        result,
        Err(JointQueryError::WrongKind {
            expected: JointKind::Revolute,
            actual: JointKind::Distance,
        })
    );
    assert_eq!(world.joint_snapshot(joint), Ok(before));
}

#[test]
fn body_cascade_preserves_newest_first_joint_order() {
    // Arrange
    let (mut world, body_a, body_b) = test_world_with_bodies();
    let first = world
        .create_joint(
            RevoluteJointDef::new(body_a, body_b)
                .expect("valid endpoints")
                .into(),
        )
        .expect("first joint should fit");
    let second = world
        .create_joint(
            DistanceJointDef::new(body_a, body_b)
                .expect("valid endpoints")
                .into(),
        )
        .expect("second joint should fit");

    // Act
    let records = world.destroy_body(body_a).expect("body should be live");

    // Assert
    assert_eq!(records[0].destroyed(), DestroyedId::Joint(second));
    assert_eq!(records[1].destroyed(), DestroyedId::Joint(first));
}

#[test]
fn invalid_reaction_timestep_and_repeated_destruction_are_no_effect_errors() {
    // Arrange
    let (mut world, body_a, body_b) = test_world_with_bodies();
    let joint = world
        .create_joint(
            MotorJointDef::new(body_a, body_b)
                .expect("valid endpoints")
                .into(),
        )
        .expect("joint should fit");
    let before = world.joint_snapshot(joint).expect("joint should be live");

    // Act
    let reaction = world.joint_reaction_force(joint, f32::NAN);
    world.destroy_joint(joint).expect("joint should be live");
    let repeated = world.destroy_joint(joint);

    // Assert
    assert_eq!(reaction, Err(JointQueryError::InvalidInverseTimestep));
    assert_eq!(before.kind(), JointKind::Motor);
    assert_eq!(
        repeated,
        Err(JointMutationError::InvalidHandle(
            liquidfun::HandleError::StaleOrDestroyed
        ))
    );
}
