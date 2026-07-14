//! Explicit and implicit destruction-listener timing.

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyId, BodyType, DestroyedId, DestructionCause, FixtureDef, GearJointDef,
    LifecycleEvent, ObjectSnapshot, PrismaticJointDef, RevoluteJointDef, World,
};

fn fixture_definition() -> FixtureDef {
    FixtureDef::new(
        Shape::from(CircleShape::new(Vec2::ZERO, 0.5).expect("circle should be valid")),
        1.0,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .expect("fixture should be valid")
}

fn dynamic_body(world: &mut World, position: Vec2) -> BodyId {
    world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, position, 0.0, true)
                .expect("dynamic body should be valid"),
        )
        .expect("dynamic body should fit")
}

#[test]
fn explicit_fixture_destruction_does_not_fabricate_goodbye() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&BodyDef::default())
        .expect("body should fit");
    let fixture = world
        .create_fixture(body, &fixture_definition())
        .expect("fixture should fit");

    // Act
    let report = world
        .destroy_fixture(fixture)
        .expect("fixture should remain live");

    // Assert
    assert!(matches!(
        report.lifecycle(),
        [LifecycleEvent::Destruction(record)]
            if record.destroyed() == DestroyedId::Fixture(fixture)
                && record.cause() == DestructionCause::Explicit
    ));
    assert_eq!(report.records().len(), 1);
    assert!(matches!(
        report.records()[0].snapshot(),
        ObjectSnapshot::Fixture { body: owner, .. } if *owner == body
    ));
}

#[test]
fn explicit_joint_destruction_does_not_fabricate_goodbye() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body_a = dynamic_body(&mut world, Vec2::ZERO);
    let body_b = dynamic_body(&mut world, Vec2::new(1.0, 0.0));
    let joint = world
        .create_joint(
            RevoluteJointDef::new(body_a, body_b)
                .expect("distinct bodies should form a joint")
                .into(),
        )
        .expect("joint should fit");

    // Act
    let report = world
        .destroy_joint(joint)
        .expect("joint should remain live");

    // Assert
    assert!(matches!(
        report.lifecycle(),
        [LifecycleEvent::Destruction(record)]
            if record.destroyed() == DestroyedId::Joint(joint)
                && record.cause() == DestructionCause::Explicit
    ));
    assert_eq!(report.records().len(), 1);
}

#[test]
fn body_cascade_emits_joint_and_fixture_goodbyes_before_root() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let root = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)
                .expect("dynamic body should be valid"),
        )
        .expect("root body should fit");
    let survivor = world
        .create_body(&BodyDef::default())
        .expect("survivor body should fit");
    let fixture = world
        .create_fixture(root, &fixture_definition())
        .expect("fixture should fit");
    let joint = world
        .create_joint(
            RevoluteJointDef::new(root, survivor)
                .expect("distinct bodies should form a joint")
                .into(),
        )
        .expect("joint should fit");

    // Act
    let report = world.destroy_body(root).expect("root should remain live");

    // Assert
    assert!(matches!(
        report.lifecycle(),
        [
            LifecycleEvent::JointGoodbye(joint_record),
            LifecycleEvent::FixtureGoodbye(fixture_record),
            LifecycleEvent::Destruction(body_record),
        ] if joint_record.destroyed() == DestroyedId::Joint(joint)
            && fixture_record.destroyed() == DestroyedId::Fixture(fixture)
            && body_record.destroyed() == DestroyedId::Body(root)
    ));
    assert_eq!(report.records().len(), 3);
    assert!(!world.contains_body(root));
    assert!(world.contains_body(survivor));
}

#[test]
fn destruction_report_remains_owned_after_slot_reuse() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let body = world
        .create_body(&BodyDef::default())
        .expect("body should fit");

    // Act
    let report = world.destroy_body(body).expect("body should remain live");
    let replacement = world
        .create_body(&BodyDef::default())
        .expect("replacement should fit");

    // Assert
    assert_ne!(body, replacement);
    assert_eq!(report.records()[0].destroyed(), DestroyedId::Body(body));
    assert_eq!(
        report.lifecycle(),
        [LifecycleEvent::Destruction(report.records()[0].clone())]
    );
}

#[test]
fn body_gear_cascade_reports_dependency_before_source_joint_goodbyes() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    let root = dynamic_body(&mut world, Vec2::ZERO);
    let body_a = dynamic_body(&mut world, Vec2::new(1.0, 0.0));
    let body_b = dynamic_body(&mut world, Vec2::new(0.0, 1.0));
    let source1 = world
        .create_joint(
            RevoluteJointDef::new(root, body_a)
                .expect("distinct bodies should form a revolute joint")
                .into(),
        )
        .expect("revolute joint should fit");
    let source2 = world
        .create_joint(
            PrismaticJointDef::new(root, body_b)
                .expect("distinct bodies should form a prismatic joint")
                .into(),
        )
        .expect("prismatic joint should fit");
    let gear = world
        .create_joint(
            GearJointDef::new(source1, source2)
                .expect("distinct source joints should form a gear")
                .into(),
        )
        .expect("gear should fit");

    // Act
    let report = world.destroy_body(root).expect("root should remain live");
    let goodbye_ids = report
        .lifecycle()
        .iter()
        .filter_map(|event| match event {
            LifecycleEvent::JointGoodbye(record) => Some(record.destroyed()),
            _ => None,
        })
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(
        goodbye_ids,
        vec![
            DestroyedId::Joint(gear),
            DestroyedId::Joint(source2),
            DestroyedId::Joint(source1),
        ]
    );
    assert!(matches!(
        report.lifecycle().last(),
        Some(LifecycleEvent::Destruction(record))
            if record.destroyed() == DestroyedId::Body(root)
    ));
}
