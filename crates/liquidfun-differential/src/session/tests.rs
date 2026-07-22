use liquidfun_test_protocol::{
    CatalogSlug, FloatBits, Phase9ParticleAction, ResolveRequest, RigidWorldAction, RunSettings,
    resolve_catalog, reviewed_scenario_catalog,
    scenarios::{particles, rigid},
};

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
enum BackendEvent {
    Create {
        bytes: Vec<u8>,
        hash: String,
        settings: RunSettings,
    },
    Destroy,
    Execute(String),
    Capture(String),
}

#[derive(Debug, Default)]
struct RecordingBackend {
    events: Vec<BackendEvent>,
    maybe_next_failure: Option<SessionBackendError>,
}

impl RecordingBackend {
    fn fail_next(&mut self, failure: SessionBackendError) {
        self.maybe_next_failure = Some(failure);
    }

    fn maybe_fail(&mut self) -> Result<(), SessionBackendError> {
        let Some(failure) = self.maybe_next_failure.take() else {
            return Ok(());
        };
        Err(failure)
    }
}

impl SessionBackend for RecordingBackend {
    type Checkpoint = u32;

    fn create_session(
        &mut self,
        resolved: &liquidfun_test_protocol::ResolvedScenario,
    ) -> Result<(), SessionBackendError> {
        self.maybe_fail()?;
        self.events.push(BackendEvent::Create {
            bytes: resolved.canonical_bytes().to_vec(),
            hash: resolved.identity().content_sha256().as_str().to_owned(),
            settings: resolved.identity().settings(),
        });
        Ok(())
    }

    fn destroy_session(&mut self) {
        self.events.push(BackendEvent::Destroy);
    }

    fn execute_action(
        &mut self,
        action: &liquidfun_test_protocol::ScheduledAction,
    ) -> Result<(), SessionBackendError> {
        self.maybe_fail()?;
        self.events.push(BackendEvent::Execute(
            action.action_id().as_str().to_owned(),
        ));
        Ok(())
    }

    fn capture_checkpoint(
        &mut self,
        checkpoint: &SessionCheckpointIdentity,
    ) -> Result<Self::Checkpoint, SessionBackendError> {
        self.maybe_fail()?;
        self.events.push(BackendEvent::Capture(
            checkpoint.checkpoint_id().as_str().to_owned(),
        ));
        Ok(checkpoint.logical_step())
    }
}

fn settings() -> RunSettings {
    RunSettings::new(FloatBits::from_f32(1.0 / 60.0), 8, 3, 1)
        .expect("fixture run settings should validate")
}

fn resolved_rigid(settings: RunSettings) -> liquidfun_test_protocol::ResolvedScenario {
    let definitions = rigid::definitions().expect("rigid definitions should validate");
    let request = ResolveRequest::new(
        CatalogSlug::new("rigid-contact-lifecycle").expect("fixture slug should validate"),
        None,
        settings,
    );
    resolve_catalog(&definitions, &request).expect("fixture scenario should resolve")
}

fn selected_controller() -> SessionController<RecordingBackend> {
    let mut controller = SessionController::new(RecordingBackend::default());
    let command_id = controller
        .next_command_id()
        .expect("new controller should accept a command");
    controller
        .submit(
            command_id,
            SessionCommand::Select {
                resolved: resolved_rigid(settings()),
            },
        )
        .expect("fixture selection should succeed");
    controller
}

fn submit(
    controller: &mut SessionController<RecordingBackend>,
    command: SessionCommand,
) -> Result<SessionCommandOutcome, SessionControllerError> {
    let command_id = controller
        .next_command_id()
        .expect("fixture command counter should remain available");
    controller.submit(command_id, command)
}

#[test]
fn pause_transition_is_effect_free() {
    // Arrange
    let mut controller = selected_controller();
    submit(&mut controller, SessionCommand::Run).expect("run should be admitted");
    let event_count = controller.backend().events.len();

    // Act
    let outcome =
        submit(&mut controller, SessionCommand::Pause).expect("running session should pause");

    // Assert
    assert_eq!(outcome.state(), SessionState::ReadyPaused);
    assert_eq!(controller.backend().events.len(), event_count);
    assert!(controller.captures().is_empty());
}

#[test]
fn step_once_pauses_before_one_action_and_stays_paused() {
    // Arrange
    let mut controller = selected_controller();
    submit(&mut controller, SessionCommand::Run).expect("run should be admitted");

    // Act
    let outcome = submit(&mut controller, SessionCommand::StepOnce)
        .expect("step once should be admitted while running");

    // Assert
    assert_eq!(outcome.state(), SessionState::ReadyPaused);
    assert_eq!(controller.completed_logical_steps(), 1);
    assert_eq!(
        controller
            .backend()
            .events
            .iter()
            .filter(|event| matches!(event, BackendEvent::Execute(_)))
            .count(),
        1
    );
}

#[test]
fn restart_reuses_identical_resolved_bytes_and_hash() {
    // Arrange
    let mut controller = selected_controller();
    let first_create = controller.backend().events[0].clone();

    // Act
    submit(&mut controller, SessionCommand::Restart).expect("restart should succeed");

    // Assert
    assert_eq!(controller.state(), SessionState::ReadyPaused);
    assert_eq!(controller.backend().events[1], BackendEvent::Destroy);
    assert_eq!(controller.backend().events[2], first_create);
    assert_eq!(controller.completed_logical_steps(), 0);
}

#[test]
fn invalid_settings_are_rejected_before_backend_effects() {
    // Arrange
    let mut controller = selected_controller();
    let event_count = controller.backend().events.len();
    let replacement = resolved_rigid(settings());
    let invalid = RunSettingsInput::new(FloatBits::from_f32(0.0), 8, 3, 1);

    // Act
    let error = submit(
        &mut controller,
        SessionCommand::ApplySettingsAndRestart {
            settings: invalid,
            resolved: replacement,
        },
    )
    .expect_err("zero timestep must be rejected");

    // Assert
    assert_eq!(error.kind(), SessionControllerErrorKind::InvalidRunSettings);
    assert_eq!(controller.backend().events.len(), event_count);
    assert_eq!(controller.state(), SessionState::ReadyPaused);
}

#[test]
fn valid_settings_restart_accepts_only_resolver_materialized_step_changes() {
    // Arrange
    let mut controller = selected_controller();
    let replacement_settings = RunSettings::new(FloatBits::from_f32(1.0 / 120.0), 10, 4, 2)
        .expect("replacement settings should validate");
    let replacement = resolved_rigid(replacement_settings);

    // Act
    let outcome = submit(
        &mut controller,
        SessionCommand::ApplySettingsAndRestart {
            settings: RunSettingsInput::new(
                replacement_settings.timestep_bits(),
                replacement_settings.velocity_iterations(),
                replacement_settings.position_iterations(),
                replacement_settings.particle_iterations(),
            ),
            resolved: replacement,
        },
    )
    .expect("resolver-materialized settings replacement should succeed");

    // Assert
    assert_eq!(outcome.state(), SessionState::ReadyPaused);
    assert_eq!(
        controller
            .selected()
            .expect("replacement should remain selected")
            .identity()
            .settings(),
        replacement_settings
    );
    assert_eq!(controller.completed_logical_steps(), 0);
}

#[test]
fn valid_settings_restart_covers_every_reviewed_catalog_action_family() {
    // Arrange
    let catalog = reviewed_scenario_catalog().expect("reviewed catalog should validate");
    let original_settings = settings();
    let replacement_settings = RunSettings::new(FloatBits::from_f32(1.0 / 120.0), 10, 4, 2)
        .expect("replacement settings should validate");

    for definition in catalog.definitions() {
        let original =
            resolve_with_optional_seed(catalog.definitions(), definition.slug(), original_settings);
        let replacement = resolve_with_optional_seed(
            catalog.definitions(),
            definition.slug(),
            replacement_settings,
        );
        let mut controller = SessionController::new(RecordingBackend::default());
        submit(
            &mut controller,
            SessionCommand::Select { resolved: original },
        )
        .expect("reviewed scenario should select");

        // Act
        let outcome = submit(
            &mut controller,
            SessionCommand::ApplySettingsAndRestart {
                settings: RunSettingsInput::new(
                    replacement_settings.timestep_bits(),
                    replacement_settings.velocity_iterations(),
                    replacement_settings.position_iterations(),
                    replacement_settings.particle_iterations(),
                ),
                resolved: replacement,
            },
        )
        .unwrap_or_else(|error| {
            panic!(
                "settings replacement failed for {}: {error}",
                definition.slug().as_str()
            )
        });

        // Assert
        assert_eq!(outcome.state(), SessionState::ReadyPaused);
        assert_eq!(controller.completed_logical_steps(), 0);
    }
}

fn resolve_with_optional_seed(
    definitions: &[liquidfun_test_protocol::CatalogDefinition],
    slug: &CatalogSlug,
    settings: RunSettings,
) -> liquidfun_test_protocol::ResolvedScenario {
    let named = ResolveRequest::new(slug.clone(), None, settings);
    resolve_catalog(definitions, &named).unwrap_or_else(|_error| {
        let seeded = ResolveRequest::new(slug.clone(), Some(0), settings);
        resolve_catalog(definitions, &seeded).unwrap_or_else(|error| {
            panic!(
                "reviewed scenario {} did not resolve: {error}",
                slug.as_str()
            )
        })
    })
}

#[test]
fn capture_uses_declared_identity_without_advancing() {
    // Arrange
    let mut controller = selected_controller();
    submit(&mut controller, SessionCommand::StepOnce).expect("step should succeed");
    let checkpoint_id = controller
        .selected()
        .expect("scenario should remain selected")
        .checkpoints()[0]
        .checkpoint_id()
        .clone();

    // Act
    submit(
        &mut controller,
        SessionCommand::CaptureCheckpoint { checkpoint_id },
    )
    .expect("current checkpoint should capture");

    // Assert
    assert_eq!(controller.completed_logical_steps(), 1);
    assert_eq!(controller.captures().len(), 1);
    assert_eq!(controller.captures()[0].identity().logical_step(), 1);
    assert_eq!(controller.captures()[0].value(), &1);
}

#[test]
fn duplicate_command_is_rejected_without_backend_effects() {
    // Arrange
    let mut controller = selected_controller();
    let command_id = controller
        .next_command_id()
        .expect("controller should accept another command");
    controller
        .submit(command_id, SessionCommand::Run)
        .expect("first command should succeed");
    let event_count = controller.backend().events.len();

    // Act
    let error = controller
        .submit(command_id, SessionCommand::Pause)
        .expect_err("duplicate command ID must fail");

    // Assert
    assert_eq!(error.kind(), SessionControllerErrorKind::StaleCommand);
    assert_eq!(controller.backend().events.len(), event_count);
    assert_eq!(controller.state(), SessionState::Running);
}

#[test]
fn backend_error_keeps_logical_ordinal_and_classifies_state() {
    // Arrange
    let mut controller = selected_controller();
    controller
        .backend_mut()
        .fail_next(SessionBackendError::recoverable(
            SessionBackendErrorCategory::Action,
        ));

    // Act
    let error = submit(&mut controller, SessionCommand::StepOnce)
        .expect_err("backend action failure should surface");

    // Assert
    assert_eq!(error.kind(), SessionControllerErrorKind::Backend);
    assert_eq!(controller.state(), SessionState::RecoverableError);
    assert_eq!(controller.completed_logical_steps(), 0);
}

#[test]
fn particle_pause_is_applied_only_as_a_scenario_action() {
    // Arrange
    let definitions = particles::definitions().expect("particle definitions should validate");
    let request = ResolveRequest::new(
        CatalogSlug::new("particle-system-pause-action").expect("fixture slug should validate"),
        None,
        settings(),
    );
    let resolved = resolve_catalog(&definitions, &request).expect("pause scenario should resolve");
    let action_id = resolved
        .actions()
        .iter()
        .find(|action| {
            matches!(
                action.action(),
                RigidWorldAction::Particle {
                    action: Phase9ParticleAction::SetPaused { paused: true, .. }
                }
            )
        })
        .map(|action| action.action_id().clone())
        .expect("fixture should contain particle pause");
    let mut controller = SessionController::new(RecordingBackend::default());
    submit(&mut controller, SessionCommand::Select { resolved })
        .expect("fixture selection should succeed");

    // Act
    submit(
        &mut controller,
        SessionCommand::ApplyScenarioAction {
            action_id: action_id.clone(),
        },
    )
    .expect("typed particle pause action should apply");

    // Assert
    assert_eq!(controller.state(), SessionState::ReadyPaused);
    assert!(
        controller
            .backend()
            .events
            .contains(&BackendEvent::Execute(action_id.as_str().to_owned()))
    );
}

#[test]
fn running_advances_to_completed_without_unchecked_replay() {
    // Arrange
    let mut controller = selected_controller();
    submit(&mut controller, SessionCommand::Run).expect("run should be admitted");

    // Act
    while controller.state() == SessionState::Running {
        controller
            .advance_running()
            .expect("fixture actions should execute");
    }

    // Assert
    assert_eq!(controller.state(), SessionState::Completed);
    assert_eq!(controller.completed_logical_steps(), 3);
    assert!(matches!(
        controller.advance_running(),
        Err(error) if error.kind() == SessionControllerErrorKind::InvalidTransition
    ));
}

#[test]
fn pure_transition_rejects_reentrant_stepping_commands() {
    // Arrange
    let command = SessionCommandKind::StepOnce;

    // Act
    let error = transition(SessionState::Stepping, command)
        .expect_err("stepping state must reject another command");

    // Assert
    assert_eq!(error.kind(), SessionTransitionErrorKind::InvalidTransition);
}
