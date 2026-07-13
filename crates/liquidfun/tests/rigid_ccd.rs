//! Semantic witnesses for bounded TOI-island construction and solving.

#![cfg(feature = "differential-internals")]

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::rigid_differential::{RigidToiFailureInjection, RigidToiIslandLimits};
use liquidfun::{
    BodyDef, BodyId, BodyType, FixtureDef, StepCompletion, StepConfiguration, StepError, StepHook,
    StepLimits, World,
};

#[derive(Default)]
struct NoopHook;

impl StepHook for NoopHook {}

fn circle_fixture(body_type: BodyType) -> FixtureDef {
    FixtureDef::new(
        Shape::from(CircleShape::new(Vec2::ZERO, 0.25).expect("test circle should be valid")),
        if body_type == BodyType::Dynamic {
            1.0
        } else {
            0.0
        },
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .expect("test fixture definition should be valid")
}

fn create_circle_body(
    world: &mut World,
    body_type: BodyType,
    position: Vec2,
    velocity: Vec2,
    bullet: bool,
) -> BodyId {
    let definition = BodyDef::new(body_type, position, 0.0, true)
        .expect("test body definition should be valid")
        .with_linear_velocity(velocity)
        .expect("test velocity should be valid")
        .with_bullet(bullet);
    let body = world
        .create_body(&definition)
        .expect("test body should fit");
    world
        .create_fixture(body, &circle_fixture(body_type))
        .expect("test fixture should fit");
    body
}

fn create_unit_circle_body(world: &mut World, body_type: BodyType, position: Vec2) -> BodyId {
    let body = world
        .create_body(
            &BodyDef::new(body_type, position, 0.0, true)
                .expect("test body definition should be valid"),
        )
        .expect("test body should fit");
    let fixture = FixtureDef::new(
        Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("test circle should be valid")),
        if body_type == BodyType::Dynamic {
            1.0
        } else {
            0.0
        },
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .expect("test fixture definition should be valid");
    world
        .create_fixture(body, &fixture)
        .expect("test fixture should fit");
    body
}

fn swept_world(target_count: usize) -> (World, BodyId, Vec<BodyId>, StepConfiguration) {
    let mut world = World::new().expect("test world key should remain available");
    let moving = create_circle_body(
        &mut world,
        BodyType::Dynamic,
        Vec2::new(-2.0, 0.0),
        Vec2::new(2.0, 0.0),
        true,
    );
    let mut targets = Vec::new();
    for _ in 0..target_count {
        targets.push(create_circle_body(
            &mut world,
            BodyType::Static,
            Vec2::ZERO,
            Vec2::ZERO,
            false,
        ));
    }
    let configuration =
        StepConfiguration::new(1.0, 8, 3).expect("test step configuration should be valid");
    world
        .set_continuous_physics_enabled(false)
        .expect("test configuration should remain mutable");
    world
        .step(configuration, &mut NoopHook, StepLimits::default())
        .expect("discrete sweep preparation should succeed");
    world
        .set_continuous_physics_enabled(true)
        .expect("test configuration should remain mutable");
    (world, moving, targets, configuration)
}

#[test]
fn accepted_substep_preserves_symmetric_multi_contact_center() {
    // Arrange
    let mut world = World::new().expect("test world key should remain available");
    world
        .set_sub_stepping_enabled(true)
        .expect("sub-stepping control should remain mutable");
    let _left = create_unit_circle_body(&mut world, BodyType::Static, Vec2::new(-1.0, 0.0));
    let dynamic = create_unit_circle_body(&mut world, BodyType::Dynamic, Vec2::ZERO);
    let _right = create_unit_circle_body(&mut world, BodyType::Static, Vec2::new(1.0, 0.0));
    let configuration =
        StepConfiguration::new(1.0 / 60.0, 8, 3).expect("test step should be valid");

    // Act
    let report = world
        .step(configuration, &mut NoopHook, StepLimits::default())
        .expect("one accepted TOI event should remain coherent");
    let snapshot = world
        .body_snapshot(dynamic)
        .expect("dynamic body should remain live");

    // Assert
    assert_eq!(report.completion(), StepCompletion::ContinuousPending);
    assert_eq!(snapshot.position().x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(snapshot.position().y.to_bits(), 0.0_f32.to_bits());
}

#[test]
fn toi_island_respects_source_order_and_capacity_bounds() {
    // Arrange
    let (mut world, moving, targets, configuration) = swept_world(2);
    let expected_occurrences = world
        .rigid_contact_diagnostics()
        .into_iter()
        .map(|contact| contact.occurrence())
        .collect::<Vec<_>>();

    // Act
    let diagnostic = world
        .rigid_toi_event_diagnostic(configuration, RigidToiIslandLimits::reviewed(), None)
        .expect("bounded TOI event should solve")
        .expect("the swept bullet should produce an event");

    // Assert
    assert_eq!(diagnostic.contact_occurrences(), expected_occurrences);
    assert_eq!(diagnostic.body_ids().len(), 3);
    assert!(diagnostic.body_ids().contains(&moving));
    assert!(
        targets
            .iter()
            .all(|target| diagnostic.body_ids().contains(target))
    );
    assert!(diagnostic.body_ids().len() <= RigidToiIslandLimits::reviewed().max_bodies());
    assert!(
        diagnostic.contact_occurrences().len() <= RigidToiIslandLimits::reviewed().max_contacts()
    );
}

#[test]
fn toi_solver_does_not_warm_start_or_store_continuous_impulses() {
    // Arrange
    let (mut world, _moving, _targets, configuration) = swept_world(1);

    // Act
    let diagnostic = world
        .rigid_toi_event_diagnostic(configuration, RigidToiIslandLimits::reviewed(), None)
        .expect("bounded TOI event should solve")
        .expect("the swept bullet should produce an event");
    let persistent_contacts = world.rigid_contact_diagnostics();

    // Assert
    assert!(diagnostic.transient_normal_impulse_sum() > 0.0);
    assert!(
        persistent_contacts
            .iter()
            .flat_map(|contact| contact.contact().points())
            .all(
                |point| point.normal_impulse().to_bits() == 0.0_f32.to_bits()
                    && point.tangent_impulse().to_bits() == 0.0_f32.to_bits()
            )
    );
}

#[test]
fn failed_toi_event_is_atomic() {
    // Arrange
    let (mut world, moving, targets, configuration) = swept_world(1);
    let moving_before = world
        .rigid_body_diagnostic(moving)
        .expect("moving body should remain live");
    let target_before = world
        .rigid_body_diagnostic(targets[0])
        .expect("target body should remain live");
    let contacts_before = world.rigid_contact_diagnostics();

    // Act
    let result = world.rigid_toi_event_diagnostic(
        configuration,
        RigidToiIslandLimits::reviewed(),
        Some(RigidToiFailureInjection::AfterSolve),
    );

    // Assert
    assert!(result.is_err());
    assert_eq!(
        world
            .rigid_body_diagnostic(moving)
            .expect("moving body should remain live"),
        moving_before
    );
    assert_eq!(
        world
            .rigid_body_diagnostic(targets[0])
            .expect("target body should remain live"),
        target_before
    );
    assert_eq!(world.rigid_contact_diagnostics(), contacts_before);
}

fn automatic_ccd_world(target_type: BodyType) -> (World, BodyId, BodyId, StepConfiguration) {
    let mut world = World::new().expect("test world key should remain available");
    let moving = create_circle_body(
        &mut world,
        BodyType::Dynamic,
        Vec2::new(-2.0, 0.0),
        Vec2::new(2.0, 0.0),
        true,
    );
    let target = create_circle_body(&mut world, target_type, Vec2::ZERO, Vec2::ZERO, false);
    let configuration =
        StepConfiguration::new(1.0, 8, 3).expect("test step configuration should be valid");
    (world, moving, target, configuration)
}

#[test]
fn substepping_accepts_one_toi_event_and_resumes_without_repeating_discrete_work() {
    // Arrange
    let (mut world, moving, _target, configuration) = automatic_ccd_world(BodyType::Static);
    world
        .set_sub_stepping_enabled(true)
        .expect("test configuration should remain mutable");

    // Act
    let pending = world
        .step(configuration, &mut NoopHook, StepLimits::default())
        .expect("first sub-step should succeed");
    let position_after_pending = world
        .body_snapshot(moving)
        .expect("moving body should remain live")
        .position();
    let stored_after_pending = world.rigid_contact_diagnostics();
    let complete = world
        .step(configuration, &mut NoopHook, StepLimits::default())
        .expect("matching continuation should succeed");
    let position_after_resume = world
        .body_snapshot(moving)
        .expect("moving body should remain live")
        .position();

    // Assert
    assert_eq!(pending.completion(), StepCompletion::ContinuousPending);
    assert_eq!(pending.continuous_contact_solves().len(), 1);
    assert!(
        pending.continuous_contact_solves()[0]
            .contact()
            .points()
            .iter()
            .any(|point| point.normal_impulse() > 0.0)
    );
    assert!(
        stored_after_pending
            .iter()
            .flat_map(|contact| contact.contact().points())
            .all(
                |point| point.normal_impulse().to_bits() == 0.0_f32.to_bits()
                    && point.tangent_impulse().to_bits() == 0.0_f32.to_bits()
            )
    );
    assert_eq!(complete.completion(), StepCompletion::Complete);
    assert_eq!(position_after_resume, position_after_pending);
    assert_eq!(complete.contact_transitions().len(), 1);
    assert_eq!(complete.events().len(), 1);
    assert!(complete.events()[0].maybe_pre_solve().is_some());
    assert!(complete.contact_solves().is_empty());
    assert!(complete.continuous_contact_solves().is_empty());
}

#[test]
fn exhausted_budget_returns_coherent_partial_evidence_and_can_resume() {
    // Arrange
    let (mut world, moving, _target, configuration) = automatic_ccd_world(BodyType::Static);
    let exhausted_limits = StepLimits::default()
        .with_continuous_work_limit(0)
        .expect("zero is a coherent pre-event work boundary");

    // Act
    let exhausted = world.step(configuration, &mut NoopHook, exhausted_limits);
    let position_after_discrete = world
        .body_snapshot(moving)
        .expect("moving body should remain live")
        .position();
    let resumed = world
        .step(configuration, &mut NoopHook, StepLimits::default())
        .expect("same world should resume continuous work");

    // Assert
    let Err(StepError::ContinuousWorkLimitExceeded { limit, progress }) = exhausted else {
        panic!("zero continuous work must return typed partial evidence");
    };
    assert_eq!(limit, 0);
    assert!(progress.discrete_completed());
    assert_eq!(progress.completed_events(), 0);
    assert_eq!(resumed.completion(), StepCompletion::Complete);
    assert!(
        world
            .body_snapshot(moving)
            .expect("moving body should remain live")
            .position()
            .x
            < position_after_discrete.x
    );
}

#[test]
fn anti_tunneling_witnesses_stop_bullets_without_corrupting_contacts() {
    for target_type in [BodyType::Static, BodyType::Kinematic, BodyType::Dynamic] {
        // Arrange
        let (mut world, moving, target, configuration) = automatic_ccd_world(target_type);

        // Act
        let report = world
            .step(configuration, &mut NoopHook, StepLimits::default())
            .expect("bounded continuous step should succeed");
        let moving_position = world
            .body_snapshot(moving)
            .expect("moving body should remain live")
            .position();
        let target_position = world
            .body_snapshot(target)
            .expect("target body should remain live")
            .position();

        // Assert
        assert_eq!(report.completion(), StepCompletion::Complete);
        assert!(moving_position.x <= target_position.x);
        assert_eq!(world.contact_count(), 1);
        assert!(world.rigid_contact_diagnostics()[0].contact().is_touching());
    }
}
