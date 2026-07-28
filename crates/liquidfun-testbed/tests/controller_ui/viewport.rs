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
