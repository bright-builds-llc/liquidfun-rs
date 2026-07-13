use super::{ContinuousScanControl, ContinuousStepKey, ContinuousStepKind, ContinuousStepState};
use crate::collision::{CircleShape, FilterData, Shape};
use crate::math::Vec2;
use crate::math::settings::MAX_SUB_STEPS;
use crate::{BodyDef, BodyType, FixtureDef, StepConfiguration, StepHook, StepLimits, World};

#[derive(Default)]
struct NoopHook;

impl StepHook for NoopHook {}

fn world_with_contact() -> (World, crate::BodyId) {
    let mut world = World::new().expect("test world key should remain available");
    let static_body = world
        .create_body(
            &BodyDef::new(BodyType::Static, Vec2::ZERO, 0.0, true)
                .expect("test static body definition should be valid"),
        )
        .expect("test static body should fit");
    let dynamic_body = world
        .create_body(
            &BodyDef::new(BodyType::Dynamic, Vec2::new(0.5, 0.0), 0.0, true)
                .expect("test dynamic body definition should be valid"),
        )
        .expect("test dynamic body should fit");
    let fixture = FixtureDef::new(
        Shape::from(CircleShape::new(Vec2::ZERO, 1.0).expect("test circle should be valid")),
        1.0,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .expect("test fixture definition should be valid");
    world
        .create_fixture(static_body, &fixture)
        .expect("test static fixture should fit");
    world
        .create_fixture(dynamic_body, &fixture)
        .expect("test dynamic fixture should fit");
    world.find_new_contacts();
    world.update_contacts();
    (world, dynamic_body)
}

fn step_key(time_step: f32) -> ContinuousStepKey {
    let configuration =
        StepConfiguration::new(time_step, 8, 3).expect("test step configuration should be valid");
    ContinuousStepKey::from_configuration(configuration)
}

fn world_with_swept_contact() -> (World, crate::BodyId, crate::BodyId) {
    let mut world = World::new().expect("test world key should remain available");
    let moving_definition = BodyDef::new(BodyType::Dynamic, Vec2::new(-2.0, 0.0), 0.0, true)
        .expect("test moving body definition should be valid")
        .with_linear_velocity(Vec2::new(2.0, 0.0))
        .expect("test moving velocity should be valid")
        .with_bullet(true);
    let moving = world
        .create_body(&moving_definition)
        .expect("test moving body should fit");
    let target = world
        .create_body(
            &BodyDef::new(BodyType::Static, Vec2::ZERO, 0.0, true)
                .expect("test target definition should be valid"),
        )
        .expect("test target body should fit");
    let fixture = FixtureDef::new(
        Shape::from(CircleShape::new(Vec2::ZERO, 0.25).expect("test circle should be valid")),
        1.0,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .expect("test fixture definition should be valid");
    world
        .create_fixture(moving, &fixture)
        .expect("test moving fixture should fit");
    world
        .create_fixture(target, &fixture)
        .expect("test target fixture should fit");
    world
        .step(
            StepConfiguration::new(1.0, 8, 3).expect("test step configuration should be valid"),
            &mut NoopHook,
            StepLimits::default(),
        )
        .expect("test discrete step should succeed");
    (world, moving, target)
}

#[test]
fn ccd_cache_is_invalidated_by_contact_and_sweep_changes() {
    // Arrange
    let (mut world, dynamic_body) = world_with_contact();
    let ordinal = world.contact_manager.contacts()[0].ordinal;
    world
        .contact_manager
        .seed_toi_state_for_test(ordinal, 0.25, MAX_SUB_STEPS + 1)
        .expect("bounded test TOI state should be accepted");

    // Act
    world.contact_manager.set_hook_enabled(ordinal, false);

    // Assert
    assert_eq!(
        world.contact_manager.toi_state_for_test(ordinal),
        Some((None, MAX_SUB_STEPS + 1))
    );
    assert!(
        world
            .contact_manager
            .increment_toi_count_for_test(ordinal)
            .is_err(),
        "the checked count must reject values above the strict upstream guard"
    );

    // Arrange
    world
        .contact_manager
        .seed_toi_state_for_test(ordinal, 0.5, 1)
        .expect("bounded test TOI state should be accepted");

    // Act
    world.contact_manager.invalidate_toi_for_body(dynamic_body);

    // Assert
    assert_eq!(
        world.contact_manager.toi_state_for_test(ordinal),
        Some((None, 1))
    );
}

#[test]
fn pending_ccd_state_survives_only_the_matching_step() {
    // Arrange
    let (mut world, _dynamic_body) = world_with_contact();
    let ordinal = world.contact_manager.contacts()[0].ordinal;
    let mut state = ContinuousStepState::new();
    let matching = step_key(1.0 / 60.0);
    let different = step_key(1.0 / 30.0);
    world
        .contact_manager
        .seed_toi_state_for_test(ordinal, 0.25, 1)
        .expect("bounded test TOI state should be accepted");
    state.mark_pending(matching);

    // Act
    let matching_kind = state.begin_step(matching, &mut world.contact_manager);
    let retained_state = world.contact_manager.toi_state_for_test(ordinal);
    state.mark_pending(matching);
    let different_kind = state.begin_step(different, &mut world.contact_manager);
    let reset_state = world.contact_manager.toi_state_for_test(ordinal);
    let stale_kind = state.begin_step(matching, &mut world.contact_manager);

    // Assert
    assert_eq!(matching_kind, ContinuousStepKind::Resumed);
    assert_eq!(retained_state, Some((Some(0.25), 1)));
    assert_eq!(different_kind, ContinuousStepKind::Fresh);
    assert_eq!(reset_state, Some((None, 0)));
    assert_eq!(stale_kind, ContinuousStepKind::Fresh);
}

#[test]
fn rejected_ccd_candidate_restores_internal_body_state() {
    // Arrange
    let (mut world, moving, target) = world_with_swept_contact();
    let ordinal = world.contact_manager.contacts()[0].ordinal;
    let moving_before = world
        .bodies
        .get(moving)
        .expect("test moving body should remain live")
        .state;
    let target_before = world
        .bodies
        .get(target)
        .expect("test target body should remain live")
        .state;

    // Act
    let candidate = world
        .select_continuous_candidate_with_control(ContinuousScanControl {
            maybe_reject_ordinal: Some(ordinal),
        })
        .expect("test rejected scan should remain coherent");

    // Assert
    assert!(candidate.is_none());
    assert_eq!(
        world
            .bodies
            .get(moving)
            .expect("test moving body should remain live")
            .state,
        moving_before
    );
    assert_eq!(
        world
            .bodies
            .get(target)
            .expect("test target body should remain live")
            .state,
        target_before
    );
}
