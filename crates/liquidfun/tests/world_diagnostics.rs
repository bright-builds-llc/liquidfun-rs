//! Exact bounded world-diagnostic metrics.

#![cfg(feature = "differential-internals")]

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyType, FixtureDef, NoDecisionHook, StepConfiguration, StepLimits, World,
};

fn fixture() -> FixtureDef {
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

fn create_body(world: &mut World, body_type: BodyType, position: Vec2) {
    let body = world
        .create_body(&BodyDef::new(body_type, position, 0.0, true).expect("body should be valid"))
        .expect("body should fit");
    world
        .create_fixture(body, &fixture())
        .expect("fixture should fit");
}

#[test]
fn empty_world_metrics_are_exact_zeroes() {
    // Arrange
    let world = World::new().expect("world key should remain available");

    // Act
    let diagnostics = world.world_diagnostics();

    // Assert
    assert_eq!(diagnostics.body_count(), 0);
    assert_eq!(diagnostics.fixture_count(), 0);
    assert_eq!(diagnostics.joint_count(), 0);
    assert_eq!(diagnostics.contact_count(), 0);
    assert_eq!(diagnostics.manifold_point_count(), 0);
    assert_eq!(diagnostics.proxy_count(), 0);
    assert_eq!(diagnostics.tree_height(), 0);
    assert_eq!(diagnostics.tree_balance(), 0);
    assert_eq!(diagnostics.tree_quality().to_bits(), 0.0_f32.to_bits());
}

#[test]
fn one_proxy_has_exact_leaf_tree_metrics() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    create_body(&mut world, BodyType::Static, Vec2::ZERO);

    // Act
    let diagnostics = world.world_diagnostics();

    // Assert
    assert_eq!(diagnostics.body_count(), 1);
    assert_eq!(diagnostics.fixture_count(), 1);
    assert_eq!(diagnostics.proxy_count(), 1);
    assert_eq!(diagnostics.tree_height(), 0);
    assert_eq!(diagnostics.tree_balance(), 0);
    assert_eq!(diagnostics.tree_quality().to_bits(), 1.0_f32.to_bits());
}

#[test]
fn touching_world_reports_exact_contact_and_manifold_counts() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    create_body(&mut world, BodyType::Static, Vec2::ZERO);
    create_body(&mut world, BodyType::Dynamic, Vec2::new(0.75, 0.0));
    let configuration = StepConfiguration::new(1.0 / 60.0, 8, 3).expect("step should be valid");

    // Act
    world
        .step(configuration, &mut NoDecisionHook, StepLimits::default())
        .expect("world should step");
    let before_shift = world.world_diagnostics();
    world
        .shift_origin(Vec2::new(4.0, -2.0))
        .expect("origin shift should succeed");
    let after_shift = world.world_diagnostics();

    // Assert
    assert_eq!(before_shift.body_count(), 2);
    assert_eq!(before_shift.fixture_count(), 2);
    assert_eq!(before_shift.contact_count(), 1);
    assert_eq!(before_shift.manifold_point_count(), 1);
    assert_eq!(before_shift.proxy_count(), 2);
    assert_eq!(before_shift, after_shift);
}
