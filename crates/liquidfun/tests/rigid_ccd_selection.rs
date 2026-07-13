//! Semantic witnesses for private continuous-collision candidate selection.

#![cfg(feature = "differential-internals")]

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::rigid_differential::RigidCcdFailureInjection;
use liquidfun::{
    BodyDef, BodyId, BodyType, FixtureDef, StepConfiguration, StepHook, StepLimits, World,
};

#[derive(Default)]
struct NoopHook;

impl StepHook for NoopHook {}

fn body_definition(body_type: BodyType, position: Vec2, velocity: Vec2, bullet: bool) -> BodyDef {
    BodyDef::new(body_type, position, 0.0, true)
        .expect("test body definition should be valid")
        .with_linear_velocity(velocity)
        .expect("test velocity should be valid")
        .with_bullet(bullet)
}

fn circle_fixture(sensor: bool, density: f32) -> FixtureDef {
    FixtureDef::new(
        Shape::from(CircleShape::new(Vec2::ZERO, 0.25).expect("test circle should be valid")),
        density,
        0.2,
        0.0,
        sensor,
        FilterData::default(),
    )
    .expect("test fixture definition should be valid")
}

fn create_body_with_circle(
    world: &mut World,
    body_type: BodyType,
    position: Vec2,
    velocity: Vec2,
    bullet: bool,
    sensor: bool,
) -> BodyId {
    let body = world
        .create_body(&body_definition(body_type, position, velocity, bullet))
        .expect("test body should fit");
    let density = if body_type == BodyType::Dynamic {
        1.0
    } else {
        0.0
    };
    world
        .create_fixture(body, &circle_fixture(sensor, density))
        .expect("test fixture should fit");
    body
}

fn advance_discretely(world: &mut World) {
    let configuration =
        StepConfiguration::new(1.0, 8, 3).expect("test step configuration should be valid");
    world
        .step(configuration, &mut NoopHook, StepLimits::default())
        .expect("test discrete step should succeed");
}

fn single_target_world(target_type: BodyType, sensor: bool) -> (World, BodyId) {
    let mut world = World::new().expect("test world key should remain available");
    let moving = create_body_with_circle(
        &mut world,
        BodyType::Dynamic,
        Vec2::new(-2.0, 0.0),
        Vec2::new(2.0, 0.0),
        true,
        false,
    );
    create_body_with_circle(
        &mut world,
        target_type,
        Vec2::ZERO,
        Vec2::ZERO,
        false,
        sensor,
    );
    advance_discretely(&mut world);
    (world, moving)
}

#[test]
fn ccd_selects_strictly_earliest_contact_and_preserves_equal_time_order() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    let moving = create_body_with_circle(
        &mut world,
        BodyType::Dynamic,
        Vec2::new(-2.0, 0.0),
        Vec2::new(2.0, 0.0),
        true,
        false,
    );
    create_body_with_circle(
        &mut world,
        BodyType::Static,
        Vec2::ZERO,
        Vec2::ZERO,
        false,
        false,
    );
    create_body_with_circle(
        &mut world,
        BodyType::Static,
        Vec2::ZERO,
        Vec2::ZERO,
        false,
        false,
    );
    advance_discretely(&mut world);
    let contacts = world.rigid_contact_diagnostics();
    assert_eq!(contacts.len(), 2);
    let expected_occurrence = contacts[0].occurrence();

    // Act
    let candidate = world
        .rigid_ccd_candidate_diagnostic(None)
        .expect("bounded CCD scan should succeed")
        .expect("the swept bullet should produce one candidate");

    // Assert
    assert_eq!(candidate.occurrence(), expected_occurrence);
    assert!(candidate.alpha() > 0.0 && candidate.alpha() < 1.0);
    assert!(
        world
            .body_snapshot(moving)
            .expect("moving body should remain live")
            .is_awake()
    );
    assert!(candidate.contact().is_touching());
}

#[test]
fn rejected_candidate_restores_complete_body_state() {
    // Arrange
    let (mut world, moving) = single_target_world(BodyType::Static, false);
    let occurrence = world.rigid_contact_diagnostics()[0].occurrence();
    let moving_before = world
        .rigid_body_diagnostic(moving)
        .expect("moving body should remain live");

    // Act
    let candidate = world
        .rigid_ccd_candidate_diagnostic(Some(RigidCcdFailureInjection::RejectCandidate {
            occurrence,
        }))
        .expect("bounded rejected scan should succeed");
    let moving_after = world
        .rigid_body_diagnostic(moving)
        .expect("moving body should remain live");

    // Assert
    assert!(candidate.is_none());
    assert_eq!(moving_after.snapshot(), moving_before.snapshot());
    assert_eq!(
        moving_after.linear_velocity(),
        moving_before.linear_velocity()
    );
    assert_eq!(
        moving_after.angular_velocity().to_bits(),
        moving_before.angular_velocity().to_bits()
    );
}

#[test]
fn ccd_applies_sensor_activity_bullet_and_substep_exclusions() {
    // Arrange
    let (mut sensor_world, _sensor_moving) = single_target_world(BodyType::Static, true);
    let (mut sleeping_world, sleeping_moving) = single_target_world(BodyType::Static, false);
    sleeping_world
        .set_body_awake(sleeping_moving, false)
        .expect("test moving body should remain live");
    let (mut non_bullet_world, non_bullet_moving) = single_target_world(BodyType::Dynamic, false);
    non_bullet_world
        .set_body_bullet(non_bullet_moving, false)
        .expect("test moving body should remain live");
    let (mut exhausted_world, _exhausted_moving) = single_target_world(BodyType::Static, false);
    let exhausted_occurrence = exhausted_world.rigid_contact_diagnostics()[0].occurrence();

    // Act
    let sensor_candidate = sensor_world
        .rigid_ccd_candidate_diagnostic(None)
        .expect("sensor scan should succeed");
    let sleeping_candidate = sleeping_world
        .rigid_ccd_candidate_diagnostic(None)
        .expect("sleeping scan should succeed");
    let non_bullet_candidate = non_bullet_world
        .rigid_ccd_candidate_diagnostic(None)
        .expect("non-bullet scan should succeed");
    let exhausted_candidate = exhausted_world
        .rigid_ccd_candidate_diagnostic(Some(RigidCcdFailureInjection::ExhaustSubStepBudget {
            occurrence: exhausted_occurrence,
        }))
        .expect("exhausted scan should succeed");

    // Assert
    assert!(sensor_candidate.is_none());
    assert!(sleeping_candidate.is_none());
    assert!(non_bullet_candidate.is_none());
    assert!(exhausted_candidate.is_none());
}
