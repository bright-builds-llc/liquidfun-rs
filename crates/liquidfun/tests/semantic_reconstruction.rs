//! Typed semantic-world reconstruction evidence.

#![cfg(feature = "differential-internals")]

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::rigid_differential::{
    JointReconstruction, ReconstructionSupport, ReconstructionUnsupported,
    WorldReconstructionLimits,
};
use liquidfun::{
    BodyDef, BodyType, DistanceJointDef, FixtureDef, FrictionJointDef, GearJointDef, JointKind,
    MotorJointDef, MouseJointDef, PrismaticJointDef, PulleyJointDef, RevoluteJointDef,
    RopeJointDef, WeldJointDef, WheelJointDef, World,
};

fn circle_fixture() -> FixtureDef {
    FixtureDef::new(
        Shape::from(CircleShape::new(Vec2::ZERO, 0.5).expect("circle should be valid")),
        1.0,
        0.2,
        0.1,
        false,
        FilterData::new(0x0002, 0x00f0, -2),
    )
    .expect("fixture should be valid")
}

fn body(world: &mut World, position: Vec2) -> liquidfun::BodyId {
    world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, position, 0.0, true)
                .expect("body definition should be valid"),
        )
        .expect("body should fit")
}

#[test]
fn empty_reconstruction_is_owned_and_deterministic() {
    // Arrange
    let world = World::new().expect("world key should remain available");

    // Act
    let reconstruction = world
        .semantic_reconstruction()
        .expect("empty reconstruction should fit");
    let first = reconstruction.to_diagnostic_text();
    let second = reconstruction.to_diagnostic_text();

    // Assert
    assert!(reconstruction.bodies().is_empty());
    assert!(reconstruction.joints().is_empty());
    assert_eq!(first, second);
    assert!(first.contains("diagnostic-only"));
}

#[test]
fn reconstruction_covers_all_joint_kinds_and_emits_gear_last() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body_a = body(&mut world, Vec2::new(-2.0, 0.0));
    let body_b = body(&mut world, Vec2::new(2.0, 0.0));
    let body_c = body(&mut world, Vec2::new(-2.0, 2.0));
    let body_d = body(&mut world, Vec2::new(2.0, 2.0));
    let revolute = world
        .create_joint(
            RevoluteJointDef::new(body_a, body_c)
                .expect("revolute")
                .into(),
        )
        .expect("revolute should fit");
    let prismatic = world
        .create_joint(
            PrismaticJointDef::new(body_b, body_d)
                .expect("prismatic")
                .into(),
        )
        .expect("prismatic should fit");
    for definition in [
        DistanceJointDef::new(body_a, body_b)
            .expect("distance")
            .into(),
        PulleyJointDef::new(body_a, body_b).expect("pulley").into(),
        MouseJointDef::new(body_a, body_b).expect("mouse").into(),
        WheelJointDef::new(body_a, body_b).expect("wheel").into(),
        WeldJointDef::new(body_a, body_b).expect("weld").into(),
        FrictionJointDef::new(body_a, body_b)
            .expect("friction")
            .into(),
        RopeJointDef::new(body_a, body_b).expect("rope").into(),
        MotorJointDef::new(body_a, body_b).expect("motor").into(),
    ] {
        world.create_joint(definition).expect("joint should fit");
    }
    world
        .create_joint(
            GearJointDef::new(revolute, prismatic)
                .expect("gear sources")
                .into(),
        )
        .expect("gear should fit");

    // Act
    let reconstruction = world
        .semantic_reconstruction()
        .expect("bounded reconstruction should fit");

    // Assert
    let joints = reconstruction.joints();
    assert_eq!(joints.len(), 11);
    assert_eq!(
        joints.last().map(JointReconstruction::kind),
        Some(JointKind::Gear)
    );
    for kind in [
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
    ] {
        assert!(joints.iter().any(|joint| joint.kind() == kind));
    }
    let mouse = joints
        .iter()
        .find(|joint| joint.kind() == JointKind::Mouse)
        .expect("mouse record should exist");
    assert_eq!(
        mouse.support(),
        &ReconstructionSupport::Unsupported(ReconstructionUnsupported::MouseJoint)
    );
    let gear = joints.last().expect("gear should be last");
    let source_indices = gear
        .maybe_source_joint_indices()
        .expect("gear should reference both sources");
    assert!(
        joints
            .iter()
            .any(|joint| joint.index() == source_indices[0])
    );
    assert!(
        joints
            .iter()
            .any(|joint| joint.index() == source_indices[1])
    );
}

#[test]
fn bodies_and_fixtures_precede_joints_and_follow_origin_shift() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body_a = body(&mut world, Vec2::new(4.0, 3.0));
    let body_b = body(&mut world, Vec2::new(5.0, 3.0));
    world
        .create_fixture(body_a, &circle_fixture())
        .expect("fixture should fit");
    world
        .create_joint(
            PulleyJointDef::new(body_a, body_b)
                .expect("pulley")
                .with_geometry(
                    Vec2::new(4.0, 8.0),
                    Vec2::new(5.0, 8.0),
                    Vec2::ZERO,
                    Vec2::ZERO,
                    5.0,
                    5.0,
                    1.0,
                )
                .expect("pulley geometry")
                .into(),
        )
        .expect("pulley should fit");

    // Act
    let before = world
        .semantic_reconstruction()
        .expect("reconstruction should fit");
    world
        .shift_origin(Vec2::new(3.0, 2.0))
        .expect("origin shift should succeed");
    let after = world
        .semantic_reconstruction()
        .expect("shifted reconstruction should fit");

    // Assert
    assert_eq!(before.bodies().len(), 2);
    assert_eq!(before.bodies()[1].fixtures().len(), 1);
    assert_eq!(
        before.bodies()[1].fixtures()[0].snapshot(),
        &circle_fixture().snapshot()
    );
    assert_ne!(before.to_diagnostic_text(), after.to_diagnostic_text());
    assert_eq!(after.bodies()[1].snapshot().position(), Vec2::new(1.0, 1.0));
}

#[test]
fn destroyed_and_reused_storage_does_not_leak_stale_records() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let stale = body(&mut world, Vec2::new(1.0, 0.0));
    world.destroy_body(stale).expect("body should be live");
    let replacement = body(&mut world, Vec2::new(7.0, 0.0));

    // Act
    let reconstruction = world
        .semantic_reconstruction()
        .expect("reconstruction should fit");

    // Assert
    assert_ne!(stale, replacement);
    assert_eq!(reconstruction.bodies().len(), 1);
    assert_eq!(
        reconstruction.bodies()[0].snapshot().position().x.to_bits(),
        7.0_f32.to_bits()
    );
    assert_eq!(reconstruction.bodies()[0].index().get(), 0);
}

#[test]
fn reconstruction_publishes_finite_reviewed_record_bounds() {
    // Arrange
    let limits = WorldReconstructionLimits::reviewed();

    // Act
    let capacities = [
        limits.max_bodies(),
        limits.max_fixtures(),
        limits.max_joints(),
    ];

    // Assert
    assert_eq!(capacities, [4_096, 8_192, 8_192]);
}
