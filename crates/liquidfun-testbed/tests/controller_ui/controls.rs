#[test]
fn controller_adapter_maps_every_run_control_to_a_closed_command() {
    // Arrange
    let checkpoint = CheckpointId::new("checkpoint-0001").expect("fixture ID is valid");
    let action = ScenarioActionId::new("action-0001").expect("fixture ID is valid");
    let replacement_settings = RunSettings::new(FloatBits::from_f32(1.0 / 60.0), 12, 3, 2)
        .expect("replacement settings are valid");
    let mut adapter = ControllerAdapter::default();

    // Act and Assert
    assert!(matches!(
        adapter
            .begin(
                SessionState::NoSelection,
                ControllerAction::Select(resolved(settings())),
            )
            .expect("selection is admitted"),
        SessionCommand::Select { .. }
    ));
    adapter.complete();
    assert!(matches!(
        adapter
            .begin(SessionState::ReadyPaused, ControllerAction::Run)
            .expect("run is admitted"),
        SessionCommand::Run
    ));
    adapter.complete();
    assert!(matches!(
        adapter
            .begin(
                SessionState::ReadyPaused,
                ControllerAction::ApplySettingsAndRestart {
                    settings: liquidfun_differential::RunSettingsInput::new(
                        replacement_settings.timestep_bits(),
                        replacement_settings.velocity_iterations(),
                        replacement_settings.position_iterations(),
                        replacement_settings.particle_iterations(),
                    ),
                    resolved: resolved(replacement_settings),
                },
            )
            .expect("settings restart is admitted"),
        SessionCommand::ApplySettingsAndRestart { .. }
    ));
    adapter.complete();
    assert!(matches!(
        adapter
            .begin(SessionState::Running, ControllerAction::Pause)
            .expect("pause is admitted"),
        SessionCommand::Pause
    ));
    adapter.complete();
    assert!(matches!(
        adapter
            .begin(SessionState::Running, ControllerAction::StepOnce)
            .expect("step is admitted"),
        SessionCommand::StepOnce
    ));
    adapter.complete();
    assert!(matches!(
        adapter
            .begin(SessionState::ReadyPaused, ControllerAction::Restart)
            .expect("restart is admitted"),
        SessionCommand::Restart
    ));
    adapter.complete();
    assert!(matches!(
        adapter
            .begin(
                SessionState::ReadyPaused,
                ControllerAction::CaptureCheckpoint(checkpoint.clone()),
            )
            .expect("capture is admitted"),
        SessionCommand::CaptureCheckpoint { checkpoint_id } if checkpoint_id == checkpoint
    ));
    adapter.complete();
    assert!(matches!(
        adapter
            .begin(
                SessionState::ReadyPaused,
                ControllerAction::ApplyScenarioAction(action.clone()),
            )
            .expect("scenario action is admitted"),
        SessionCommand::ApplyScenarioAction { action_id } if action_id == action
    ));
}

#[test]
fn controller_adapter_rejects_duplicate_and_invalid_submission() {
    // Arrange
    let mut adapter = ControllerAdapter::default();
    adapter
        .begin(SessionState::ReadyPaused, ControllerAction::Run)
        .expect("first command is admitted");

    // Act
    let duplicate = adapter.begin(SessionState::ReadyPaused, ControllerAction::Run);
    adapter.complete();
    let invalid = adapter.begin(SessionState::NoSelection, ControllerAction::StepOnce);

    // Assert
    assert_eq!(
        duplicate.expect_err("duplicate submission must fail"),
        ControllerAdapterError::CommandInFlight
    );
    assert_eq!(
        invalid.expect_err("invalid transition must fail"),
        ControllerAdapterError::InvalidTransition
    );
}

#[test]
fn desktop_command_emission_is_passive_until_explicit_submission() {
    // Arrange
    let mut testbed = InteractiveTestbed::new().expect("reviewed catalog should load");
    let visual_index = testbed
        .visible_rows()
        .iter()
        .position(|row| row.eligibility().visual())
        .expect("reviewed catalog should retain a visual scenario");

    // Act
    let command = testbed
        .begin_select_visible(visual_index)
        .expect("selection should emit a typed command");
    let state_before_submission = testbed.session_state();
    testbed
        .submit_command(command.clone())
        .expect("emitted selection should submit");

    // Assert
    assert!(matches!(command, SessionCommand::Select { .. }));
    assert_eq!(state_before_submission, SessionState::NoSelection);
    assert_eq!(testbed.session_state(), SessionState::ReadyPaused);
}

#[test]
fn running_session_advances_only_when_the_logical_driver_is_called() {
    // Arrange
    let mut testbed = InteractiveTestbed::new().expect("reviewed catalog should load");
    let visual_index = testbed
        .visible_rows()
        .iter()
        .position(|row| row.eligibility().visual())
        .expect("reviewed catalog should retain a visual scenario");
    testbed
        .select_visible(visual_index)
        .expect("visual scenario should select");
    testbed.run().expect("selected session should enter Run");
    let timestep = testbed
        .selected_settings()
        .expect("selected scenario should expose settings")
        .timestep_bits()
        .to_f32();
    let before = testbed.completed_logical_steps();

    // Act
    let after_repaint_only = testbed.completed_logical_steps();
    let driven = testbed
        .drive_logical_time(std::time::Duration::from_secs_f32(timestep))
        .expect("explicit logical driver should advance");

    // Assert
    assert_eq!(after_repaint_only, before);
    assert_eq!(driven, 1);
    assert_eq!(testbed.completed_logical_steps(), before + 1);
}

#[test]
fn run_control_labels_and_enabledness_mirror_session_state() {
    // Arrange
    let ready = run_controls(SessionState::ReadyPaused, 0);
    let paused_after_step = run_controls(SessionState::ReadyPaused, 1);
    let running = run_controls(SessionState::Running, 1);
    let busy = run_controls(SessionState::Stepping, 1);

    // Act
    let ready_primary = ready[0];
    let resume_primary = paused_after_step[0];
    let pause_primary = running[0];

    // Assert
    assert_eq!(ready_primary.label, "Run Scenario");
    assert_eq!(ready_primary.control, RunControl::RunScenario);
    assert!(ready_primary.enabled && ready_primary.primary);
    assert_eq!(resume_primary.label, "Resume");
    assert_eq!(pause_primary.label, "Pause");
    assert!(running[1].enabled && running[2].enabled && running[3].enabled);
    assert!(busy.iter().all(|button| !button.enabled));
    assert_eq!(ready[1].label, "Step Once");
    assert_eq!(ready[2].tooltip, "Restart from step 0");
    assert_eq!(ready[3].label, "Capture Checkpoint");
}

#[test]
fn step_once_executes_exactly_one_tick_and_remains_paused() {
    // Arrange
    let mut controller = selected_controller();
    let before = controller.backend().ticks;

    // Act
    submit(&mut controller, SessionCommand::StepOnce);

    // Assert
    assert_eq!(controller.backend().ticks, before + 1);
    assert_eq!(controller.state(), SessionState::ReadyPaused);
    assert!(controller.captures().is_empty());
}

#[test]
fn pause_camera_screenshot_and_panel_actions_execute_no_tick_or_capture() {
    // Arrange
    let mut controller = selected_controller();
    submit(&mut controller, SessionCommand::Run);
    let events_before = (controller.backend().ticks, controller.backend().captures);
    let mut camera = SynchronizedCamera::default();
    let size = ScreenSize::new(800.0, 600.0).expect("fixture viewport is valid");
    let pointer = ScreenPoint::new(300.0, 200.0).expect("fixture pointer is finite");

    // Act
    submit(&mut controller, SessionCommand::Pause);
    camera.camera_mut().zoom_about_pointer(1.25, pointer, size);
    camera.camera_mut().pan_pixels(pointer);
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let screenshot =
        DiagnosticScreenshotPath::new(&workspace, Path::new("target/testbed/diagnostic.png"))
            .expect("confined diagnostic destination is valid");
    let panel = resolve_key(
        KeyboardKey::Escape,
        InputContext {
            session_state: controller.state(),
            editing_field: false,
            maybe_checkpoint_id: None,
            scenario_shortcuts: &[],
        },
    );

    // Assert
    assert_eq!(
        (controller.backend().ticks, controller.backend().captures),
        events_before
    );
    assert!(controller.captures().is_empty());
    assert_eq!(screenshot.acknowledgement(), SCREENSHOT_CLARIFICATION);
    assert!(matches!(
        panel,
        Some(InputEffect::Presentation(
            PresentationAction::CloseTopmostOrClearFocus
        ))
    ));
}

#[test]
fn every_global_shortcut_maps_to_the_exact_typed_effect() {
    // Arrange
    let checkpoint = CheckpointId::new("checkpoint-0001").expect("fixture ID is valid");
    let context = InputContext {
        session_state: SessionState::ReadyPaused,
        editing_field: false,
        maybe_checkpoint_id: Some(&checkpoint),
        scenario_shortcuts: &[],
    };

    // Act and Assert
    assert!(matches!(
        resolve_key(KeyboardKey::Space, context),
        Some(InputEffect::Controller(ControllerAction::Run))
    ));
    assert!(matches!(
        resolve_key(KeyboardKey::Right, context),
        Some(InputEffect::Controller(ControllerAction::StepOnce))
    ));
    assert!(matches!(
        resolve_key(KeyboardKey::R, context),
        Some(InputEffect::Controller(ControllerAction::Restart))
    ));
    assert!(matches!(
        resolve_key(KeyboardKey::C, context),
        Some(InputEffect::Controller(
            ControllerAction::CaptureCheckpoint(_)
        ))
    ));
    let presentation_cases = [
        (KeyboardKey::Slash, PresentationAction::FocusScenarioSearch),
        (KeyboardKey::F, PresentationAction::FocusDifference),
        (
            KeyboardKey::LeftBracket,
            PresentationAction::PreviousDifference,
        ),
        (
            KeyboardKey::RightBracket,
            PresentationAction::NextDifference,
        ),
        (
            KeyboardKey::Digit1,
            PresentationAction::ToggleOverlayGroup(1),
        ),
        (
            KeyboardKey::Digit2,
            PresentationAction::ToggleOverlayGroup(2),
        ),
        (
            KeyboardKey::Digit3,
            PresentationAction::ToggleOverlayGroup(3),
        ),
        (
            KeyboardKey::Digit4,
            PresentationAction::ToggleOverlayGroup(4),
        ),
        (KeyboardKey::Home, PresentationAction::ResetCamera),
        (
            KeyboardKey::QuestionMark,
            PresentationAction::OpenShortcutHelp,
        ),
        (
            KeyboardKey::Escape,
            PresentationAction::CloseTopmostOrClearFocus,
        ),
    ];
    for (key, expected) in presentation_cases {
        let effect = resolve_key(key, context);
        assert!(matches!(
            effect,
            Some(InputEffect::Presentation(actual)) if actual == expected
        ));
    }
}

#[test]
fn global_shortcuts_are_suppressed_during_field_editing() {
    // Arrange
    let context = InputContext {
        session_state: SessionState::Running,
        editing_field: true,
        maybe_checkpoint_id: None,
        scenario_shortcuts: &[],
    };
    let keys = [
        KeyboardKey::Space,
        KeyboardKey::Right,
        KeyboardKey::R,
        KeyboardKey::C,
        KeyboardKey::Slash,
        KeyboardKey::F,
        KeyboardKey::LeftBracket,
        KeyboardKey::RightBracket,
        KeyboardKey::Digit1,
        KeyboardKey::Digit2,
        KeyboardKey::Digit3,
        KeyboardKey::Digit4,
        KeyboardKey::Home,
        KeyboardKey::QuestionMark,
        KeyboardKey::Escape,
    ];

    // Act
    let effects = keys.map(|key| resolve_key(key, context));

    // Assert
    assert!(effects.iter().all(Option::is_none));
}

#[test]
fn scenario_shortcut_routes_a_stable_typed_action_and_pause_labels_stay_distinct() {
    // Arrange
    let action_id = ScenarioActionId::new("particle-pause").expect("fixture ID is valid");
    let shortcut = ScenarioShortcut::new('p', action_id.clone(), "Pause particle system")
        .expect("scenario shortcut is bounded and not reserved");
    let shortcuts = [shortcut];

    // Act
    let effect = resolve_key(
        KeyboardKey::Scenario('P'),
        InputContext {
            session_state: SessionState::ReadyPaused,
            editing_field: false,
            maybe_checkpoint_id: None,
            scenario_shortcuts: &shortcuts,
        },
    );

    // Assert
    assert!(matches!(
        effect,
        Some(InputEffect::Controller(ControllerAction::ApplyScenarioAction(
            actual
        ))) if actual == action_id
    ));
    assert_eq!(SESSION_PAUSED_LABEL, "Session paused");
    assert_eq!(PARTICLE_PAUSE_ACTION_LABEL, "Particle system pause action");
    assert_ne!(SESSION_PAUSED_LABEL, PARTICLE_PAUSE_ACTION_LABEL);
}

#[test]
fn invalid_settings_keep_the_previous_accepted_values_and_exact_guidance() {
    // Arrange
    let active = settings();
    let invalid_timestep = ["0", "-1", "NaN", "inf", "text"];
    let invalid_iterations = ["0", "1025", "1.5", "-1"];

    // Act and Assert
    for text in invalid_timestep {
        let mut editor = SettingsEditor::new(active);
        editor.edit(SettingsField::Timestep, text);
        editor.commit(SettingsField::Timestep);
        assert_eq!(editor.accepted(), active);
        assert_eq!(
            editor.maybe_error(SettingsField::Timestep),
            Some(TIMESTEP_GUIDANCE)
        );
        assert!(!editor.apply_enabled());
    }
    for field in [
        SettingsField::VelocityIterations,
        SettingsField::PositionIterations,
        SettingsField::ParticleIterations,
    ] {
        for text in invalid_iterations {
            let mut editor = SettingsEditor::new(active);
            editor.edit(field, text);
            editor.commit(field);
            assert_eq!(editor.accepted(), active);
            assert_eq!(editor.maybe_error(field), Some(ITERATION_GUIDANCE));
            assert!(!editor.apply_enabled());
        }
    }
}

#[test]
fn valid_settings_enable_only_apply_and_restart_controller_command() {
    // Arrange
    let active = settings();
    let replacement_settings =
        RunSettings::new(FloatBits::from_f32(0.02), 12, 5, 4).expect("replacement is valid");
    let mut editor = SettingsEditor::new(active);

    // Act
    editor.edit(SettingsField::VelocityIterations, "12");
    editor.commit(SettingsField::VelocityIterations);
    editor.edit(SettingsField::PositionIterations, "5");
    editor.commit(SettingsField::PositionIterations);
    editor.edit(SettingsField::ParticleIterations, "4");
    editor.commit(SettingsField::ParticleIterations);
    editor.edit(SettingsField::Timestep, "0.02");
    editor.commit(SettingsField::Timestep);
    let maybe_action = editor.maybe_apply_action(resolved(replacement_settings));

    // Assert
    assert_eq!(APPLY_LABEL, "Apply & Restart");
    assert!(editor.apply_enabled());
    assert_eq!(editor.accepted(), replacement_settings);
    assert!(matches!(
        maybe_action,
        Some(ControllerAction::ApplySettingsAndRestart { .. })
    ));
}
