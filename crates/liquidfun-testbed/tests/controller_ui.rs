//! Controller, input, settings, semantic viewport, and diagnostic screenshot contracts.

use std::path::Path;

use liquidfun::DebugLayer;
use liquidfun_differential::{SessionCommand, SessionState};
use liquidfun_test_protocol::{CheckpointId, FloatBits, RunSettings, ScenarioActionId};
use liquidfun_testbed::{
    controller_adapter::{
        ControlCapability, ControllerAction, ControllerAdapter, ControllerAdapterError,
        ControllerProjection, PARTICLE_PAUSE_ACTION_LABEL, SESSION_PAUSED_LABEL,
    },
    input::{
        InputContext, InputEffect, KeyboardKey, PresentationAction, ScenarioShortcut, resolve_key,
    },
    ui::{
        SCREENSHOT_CLARIFICATION,
        overlays::{DiagnosticProfile, OverlayKind, OverlayState},
        run_controls::{RunControl, run_controls},
        settings::{
            APPLY_LABEL, ITERATION_GUIDANCE, SettingsEditor, SettingsField, TIMESTEP_GUIDANCE,
        },
        viewport::{
            DiagnosticScreenshotPath, ScreenPoint, ScreenPrimitive, ScreenSize, SemanticViewport,
            SynchronizedCamera,
        },
    },
};

#[path = "controller_ui/support.rs"]
mod support;
use support::{arrow, point, resolved, selected_controller, settings, submit};

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

#[test]
fn viewport_projects_every_semantic_layer_and_preserves_stable_selection() {
    // Arrange
    let layers = [
        DebugLayer::Shapes,
        DebugLayer::Joints,
        DebugLayer::Contacts,
        DebugLayer::ContactNormals,
        DebugLayer::Particles,
        DebugLayer::ParticleContacts,
        DebugLayer::BroadPhase,
        DebugLayer::CentersOfMass,
        DebugLayer::Labels,
    ];
    let mut primitives = layers
        .into_iter()
        .zip(0_u32..)
        .map(|(layer, ordinal)| point(layer, ordinal))
        .collect::<Vec<_>>();
    primitives[3] = arrow(DebugLayer::ContactNormals, 3);
    let selected = primitives[3].key();
    let mut overlays = OverlayState::default();
    overlays.toggle(OverlayKind::BroadPhase);
    overlays.toggle(OverlayKind::CentersOfMass);
    let mut viewport = SemanticViewport::default();
    viewport.select(Some(selected));
    viewport.hover_for(Some(selected), 399);
    assert!(!viewport.tooltip_visible());
    viewport.hover_for(Some(selected), 400);
    assert!(viewport.tooltip_visible());
    let size = ScreenSize::new(800.0, 600.0).expect("fixture viewport is valid");

    // Act
    let frame = viewport
        .render_frame(&primitives, overlays, size)
        .expect("finite semantic primitives render");

    // Assert
    assert_eq!(frame.primitives().len(), layers.len());
    assert_eq!(frame.maybe_selected(), Some(selected));
    assert!(frame.primitives().iter().any(|item| item.key() == selected));
    let selected_primitive = frame
        .primitives()
        .iter()
        .find(|item| item.key() == selected)
        .expect("selected stable key remains present");
    assert!(matches!(
        selected_primitive,
        ScreenPrimitive::Arrow { style, .. }
            if style.stroke == [88, 166, 255, 255] && style.stroke_width >= 2.0
    ));
    assert_eq!(
        frame
            .primitives()
            .iter()
            .map(ScreenPrimitive::layer)
            .collect::<Vec<_>>(),
        layers
    );
}

#[test]
fn camera_zoom_is_pointer_anchored_and_comparison_views_stay_synchronized() {
    // Arrange
    let size = ScreenSize::new(800.0, 600.0).expect("fixture viewport is valid");
    let pointer = ScreenPoint::new(215.0, 117.0).expect("fixture pointer is finite");
    let mut synchronized = SynchronizedCamera::default();
    let world_before = synchronized.rust_camera().screen_to_world(pointer, size);

    // Act
    synchronized
        .camera_mut()
        .zoom_about_pointer(2.0, pointer, size);
    let world_after = synchronized.oracle_camera().screen_to_world(pointer, size);

    // Assert
    assert!((world_before.x - world_after.x).abs() < f32::EPSILON);
    assert!((world_before.y - world_after.y).abs() < f32::EPSILON);
    assert_eq!(synchronized.rust_camera(), synchronized.oracle_camera());
}

#[test]
fn overlay_groups_and_profiles_are_independent_diagnostic_presentation() {
    // Arrange
    let mut overlays = OverlayState::default();
    let profile = DiagnosticProfile::new("solve_velocity", 42.5)
        .expect("bounded finite diagnostic profile is valid");

    // Act
    overlays.toggle_shortcut_group(1);
    overlays.toggle_shortcut_group(2);
    overlays.toggle_shortcut_group(3);
    overlays.toggle_shortcut_group(4);

    // Assert
    assert!(!overlays.enabled(OverlayKind::Contacts));
    assert!(!overlays.enabled(OverlayKind::ParticleContacts));
    assert!(overlays.enabled(OverlayKind::BroadPhase));
    assert!(!overlays.enabled(OverlayKind::Statistics));
    assert!(!overlays.enabled(OverlayKind::Profiles));
    assert_eq!(profile.name(), "solve_velocity");
    assert_eq!(
        profile.authority_label(),
        "Diagnostic timing — excluded from compatibility authority"
    );
}

#[test]
fn screenshot_paths_reject_escape_and_keep_exact_diagnostic_copy() {
    // Arrange
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");

    // Act
    let valid = DiagnosticScreenshotPath::new(&workspace, Path::new("target/testbed/capture.png"))
        .expect("confined PNG path is valid");
    let traversal =
        DiagnosticScreenshotPath::new(&workspace, Path::new("target/testbed/../../../capture.png"));
    let outside = DiagnosticScreenshotPath::new(&workspace, Path::new("capture.png"));

    // Assert
    assert_eq!(valid.relative(), Path::new("target/testbed/capture.png"));
    assert_eq!(valid.acknowledgement(), SCREENSHOT_CLARIFICATION);
    assert!(traversal.is_err());
    assert!(outside.is_err());
}

#[test]
fn all_closed_controller_states_have_an_explicit_enabledness_projection() {
    // Arrange
    let states = [
        SessionState::NoSelection,
        SessionState::Resolving,
        SessionState::ReadyPaused,
        SessionState::Running,
        SessionState::Stepping,
        SessionState::Comparing,
        SessionState::Completed,
        SessionState::RecoverableError,
        SessionState::HarnessFailure,
    ];

    // Act
    let projections = states.map(ControllerProjection::from_state);

    // Assert
    assert_eq!(projections.len(), states.len());
    assert!(projections[0].enabled(ControlCapability::SelectScenario));
    assert!(
        projections[2].enabled(ControlCapability::Run)
            && projections[2].enabled(ControlCapability::StepOnce)
    );
    assert!(
        projections[3].enabled(ControlCapability::Pause)
            && projections[3].enabled(ControlCapability::Capture)
    );
    assert!(!projections[4].enabled(ControlCapability::StepOnce));
    assert!(!projections[5].enabled(ControlCapability::Capture));
    assert!(
        projections[6].enabled(ControlCapability::Restart)
            && projections[6].enabled(ControlCapability::Capture)
    );
    assert!(
        projections[7].enabled(ControlCapability::Restart)
            && projections[7].enabled(ControlCapability::ApplySettings)
    );
    assert!(
        projections[8].enabled(ControlCapability::Restart)
            && projections[8].enabled(ControlCapability::ApplySettings)
    );
}

#[test]
fn completed_projection_rejects_actions_that_would_surface_invalid_transition_errors() {
    // Arrange
    let projection = ControllerProjection::from_state(SessionState::Completed);
    let action_id = ScenarioActionId::new("action-0001").expect("fixture ID is valid");

    // Act
    let run_enabled = projection.admits(&ControllerAction::Run);
    let step_enabled = projection.admits(&ControllerAction::StepOnce);
    let scenario_action_enabled =
        projection.admits(&ControllerAction::ApplyScenarioAction(action_id));
    let restart_enabled = projection.admits(&ControllerAction::Restart);

    // Assert
    assert!(!run_enabled);
    assert!(!step_enabled);
    assert!(!scenario_action_enabled);
    assert!(restart_enabled);
}
