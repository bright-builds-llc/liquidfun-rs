//! Public integration coverage for transactional native catalog execution.

use liquidfun_differential::{
    NativeCatalogBackend, SessionBackend, SessionBackendErrorCategory, SessionCommand,
    SessionController,
};
use liquidfun_test_protocol::{
    ActionSchedule, CatalogSlug, FloatBits, ResolveRequest, RunSettings, resolve_catalog,
    scenarios::scenario_definitions,
};

fn resolved(slug: &str) -> liquidfun_test_protocol::ResolvedScenario {
    let definitions = scenario_definitions().expect("native definitions should be valid");
    let settings = RunSettings::new(FloatBits::from_f32(1.0 / 60.0), 8, 3, 8)
        .expect("reviewed settings should be valid");
    resolve_catalog(
        &definitions,
        &ResolveRequest::new(
            CatalogSlug::new(slug).expect("test slug should be valid"),
            None,
            settings,
        ),
    )
    .expect("test scenario should resolve")
}

fn submit(controller: &mut SessionController<NativeCatalogBackend>, command: SessionCommand) {
    let command_id = controller
        .next_command_id()
        .expect("test command identity should remain available");
    controller
        .submit(command_id, command)
        .expect("native catalog command should succeed");
}

#[test]
fn representative_catalog_families_execute_and_capture() {
    // Arrange
    let slugs = [
        "rigid-runtime-mutation",
        "joint-revolute-behavior",
        "standalone-rope-evolution",
        "particle-system-pause-action",
        "particle-group-construction-append",
        "particle-aabb-query-controls",
        "rigid-callback-timing",
        "particle-mutations",
    ];

    // Act
    for slug in slugs {
        let resolved = resolved(slug);
        let checkpoint_id = resolved.checkpoints()[0].checkpoint_id().clone();
        let mut controller = SessionController::new(NativeCatalogBackend::new());
        submit(&mut controller, SessionCommand::Select { resolved });
        submit(&mut controller, SessionCommand::StepOnce);
        submit(
            &mut controller,
            SessionCommand::CaptureCheckpoint { checkpoint_id },
        );

        // Assert
        assert_eq!(controller.captures().len(), 1, "slug {slug}");
        assert!(controller.backend().is_session_active(), "slug {slug}");
    }
}

#[test]
fn checkpoint_replay_is_byte_identical() {
    // Arrange
    let selected = resolved("rigid-runtime-mutation");
    let checkpoint_id = selected.checkpoints()[0].checkpoint_id().clone();
    let run = |resolved, checkpoint_id| {
        let mut controller = SessionController::new(NativeCatalogBackend::new());

        // Act
        submit(&mut controller, SessionCommand::Select { resolved });
        submit(&mut controller, SessionCommand::StepOnce);
        submit(
            &mut controller,
            SessionCommand::CaptureCheckpoint { checkpoint_id },
        );
        serde_json::to_vec(controller.captures()[0].value())
            .expect("checkpoint should serialize canonically")
    };

    // Assert
    assert_eq!(
        run(selected.clone(), checkpoint_id.clone()),
        run(selected, checkpoint_id)
    );
}

#[test]
fn foreign_action_fails_closed_without_a_partial_session() {
    // Arrange
    let selected = resolved("rigid-runtime-mutation");
    let foreign = resolved("standalone-rope-evolution");
    let mut backend = NativeCatalogBackend::new();
    backend
        .create_session(&selected)
        .expect("selected scenario should initialize");

    // Act
    let error = backend
        .execute_action(&foreign.actions()[0])
        .expect_err("foreign action should be rejected before effects");

    // Assert
    assert_eq!(error.category(), SessionBackendErrorCategory::Protocol);
    assert!(!backend.is_session_active());
}

#[test]
fn wrong_resolved_hash_is_rejected_before_a_session_exists() {
    // Arrange
    let selected = resolved("rigid-runtime-mutation");
    let wrong_hash = resolved("standalone-rope-evolution")
        .identity()
        .content_sha256()
        .clone();
    let mut backend = NativeCatalogBackend::new();

    // Act
    let error = backend
        .create_canonical(selected.canonical_bytes(), &wrong_hash)
        .expect_err("wrong resolved hash should fail before world creation");

    // Assert
    assert_eq!(error.category(), SessionBackendErrorCategory::Protocol);
    assert!(!backend.is_session_active());
}

#[test]
fn every_closed_catalog_action_executes_through_the_native_backend() {
    // Arrange
    let definitions = scenario_definitions().expect("native definitions should be valid");

    // Act
    for definition in &definitions {
        let settings = definition
            .metadata()
            .expect("native catalog metadata should be present")
            .default_settings();
        let resolved = resolve_catalog(
            &definitions,
            &ResolveRequest::new(definition.slug().clone(), None, settings),
        )
        .expect("native definition should resolve");
        let mut backend = NativeCatalogBackend::new();
        backend
            .create_session(&resolved)
            .unwrap_or_else(|error| panic!("{} setup failed: {error}", definition.slug().as_str()));
        for action in resolved
            .actions()
            .iter()
            .filter(|action| matches!(action.schedule(), ActionSchedule::LogicalStep { .. }))
        {
            backend.execute_action(action).unwrap_or_else(|error| {
                panic!(
                    "{} action {} failed: {error}",
                    definition.slug().as_str(),
                    action.action_id().as_str()
                )
            });
        }

        // Assert
        assert!(
            backend.is_session_active(),
            "slug {}",
            definition.slug().as_str()
        );
    }
}

#[test]
fn replay_action_limit_fails_closed_without_a_partial_session() {
    // Arrange
    let resolved = resolved("rigid-runtime-mutation");
    let action = resolved
        .actions()
        .iter()
        .find(|action| matches!(action.schedule(), ActionSchedule::LogicalStep { .. }))
        .expect("one logical action should exist");
    let mut backend = NativeCatalogBackend::new();
    backend
        .create_session(&resolved)
        .expect("selected scenario should initialize");
    for _ in 0..128 {
        backend
            .execute_action(action)
            .expect("reviewed replay budget should execute");
    }

    // Act
    let error = backend
        .execute_action(action)
        .expect_err("the next action must exceed the reviewed replay budget");

    // Assert
    assert_eq!(error.category(), SessionBackendErrorCategory::ResourceLimit);
    assert!(!backend.is_session_active());
}
