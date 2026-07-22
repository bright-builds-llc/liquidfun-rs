//! Live catalog/controller integration coverage for the interactive testbed model.

use std::time::Duration;

use liquidfun_differential::SessionState;
use liquidfun_test_protocol::{RunSettings, reviewed_scenario_catalog};
use liquidfun_testbed::interactive::{InteractiveTestbed, InteractiveTestbedError};

const CHECKPOINT_SCENARIO: &str = "rigid-non-colliding-lifecycle";
const CADENCE_SCENARIO: &str = "rigid-stack-stability";
const LAUNCHER_SOURCE: &str = include_str!("../src/bin/interactive.rs");

#[test]
fn production_launcher_wires_the_live_catalog_controller_and_renderer() {
    // Arrange
    let required_links = [
        "#[macroquad::main(window_conf)]",
        "InteractiveTestbed::new()",
        "resolve_key(",
        "ResponsiveLayout::for_window",
        "PanelBehavior::MutuallyExclusiveDrawers",
        "project_checkpoint(",
        "draw_protocol_frame(",
        "draw_protocol_comparison_frame(",
        "ProtocolComparisonBackend::Oracle",
        "DifferenceList::new(",
        "focused_difference_entry(",
        "KeyboardKey::QuestionMark",
        "draw_shortcut_help(",
    ];

    // Act
    let missing = required_links
        .into_iter()
        .filter(|link| !LAUNCHER_SOURCE.contains(link))
        .collect::<Vec<_>>();

    // Assert
    assert!(
        missing.is_empty(),
        "interactive production wiring is incomplete: {missing:?}"
    );
}

#[test]
fn production_launcher_wires_reachable_about_and_provenance_chrome() {
    // Arrange
    let required_links = [
        "OpenPanel::About",
        "About & provenance",
        "build_about_panel(",
        "ProvenanceInput {",
    ];

    // Act / Assert
    assert_launcher_contract("About and provenance", &required_links);
    assert_launcher_symbol_is_used("About and provenance", "build_about_panel");
    assert_launcher_symbol_is_used("About and provenance", "ProvenanceInput");
}

#[test]
fn production_launcher_wires_staged_settings_scenario_shortcuts_and_pause_labels() {
    // Arrange
    let required_links = [
        "SettingsEditor::new(",
        "SettingsField::Timestep",
        "SettingsField::VelocityIterations",
        "SettingsField::PositionIterations",
        "SettingsField::ParticleIterations",
        ".edit(",
        ".commit(",
        "ScenarioShortcut::new(",
        "KeyboardKey::Scenario(",
        "self.scenario_shortcuts().first()",
        "shortcut.label()",
        "SESSION_PAUSED_LABEL",
        "PARTICLE_PAUSE_ACTION_LABEL",
    ];

    // Act / Assert
    assert_launcher_contract("settings and scenario actions", &required_links);
    assert_launcher_symbol_is_used("settings and scenario actions", "SettingsEditor");
    assert_launcher_symbol_is_used("settings and scenario actions", "ScenarioShortcut");
    assert_launcher_symbol_is_used("settings and scenario actions", "SESSION_PAUSED_LABEL");
    assert_launcher_symbol_is_used(
        "settings and scenario actions",
        "PARTICLE_PAUSE_ACTION_LABEL",
    );
    assert!(
        !LAUNCHER_SOURCE.contains("scenario_shortcuts: &[]"),
        "settings and scenario actions contract still passes an empty scenario shortcut map"
    );
}

#[test]
fn production_launcher_wires_accessible_controls_pointer_gestures_and_minimum_window_actions() {
    // Arrange
    let required_links = [
        "KeyCode::Tab",
        "focus_return.open(",
        "self.modal_input_active(layout)",
        "self.handle_modal_focus_input()",
        "modal_focus_routed_this_frame",
        "focus_return.move_to(",
        "if let Some(returned_focus) = self.focus_return.close()",
        "control_for_focus(returned_focus)",
        "const CONTROL_TARGET: f32 = 44.0;",
        "scenario_search_bounds(",
        "clipboard_set(link.url())",
        "focused_about_link",
        "about_link_bounds(",
        "shortcut_close_bounds()",
        "MouseButton::Middle",
        "shift && is_mouse_button_down(MouseButton::Left)",
        "maybe_last_click",
        "if double_click {",
        "hit_test_frame(",
        "self.center_x = 0.0;",
        "self.center_y = 0.0;",
        "minimum_close_bounds()",
        "minimum_about_bounds()",
        "Close",
        "About & provenance",
    ];

    // Act / Assert
    assert_launcher_contract(
        "focus, controls, pointer gestures, and minimum-window actions",
        &required_links,
    );
    assert_launcher_symbol_is_used(
        "focus, controls, pointer gestures, and minimum-window actions",
        "FocusReturn",
    );
    assert_launcher_symbol_is_used(
        "focus, controls, pointer gestures, and minimum-window actions",
        "CONTROL_TARGET",
    );
    let pointer_centered_zoom = LAUNCHER_SOURCE.contains("zoom_about_pointer(")
        || ["old_scale", "new_scale", "world_x", "world_y"]
            .into_iter()
            .all(|link| LAUNCHER_SOURCE.contains(link));
    assert!(
        pointer_centered_zoom,
        "interactive production pointer gestures do not preserve the world point under zoom"
    );
    assert!(
        !LAUNCHER_SOURCE.contains("debug_primitives().first()"),
        "semantic pointer selection must hit-test the clicked primitive"
    );
    assert!(
        !LAUNCHER_SOURCE.contains("22.0, TEXT") && !LAUNCHER_SOURCE.contains("28.0, TEXT"),
        "production launcher uses typography outside the approved 12/14/18/24 scale"
    );
}

#[test]
fn selects_a_real_shared_catalog_definition_with_its_exact_defaults() {
    // Arrange
    let catalog = reviewed_scenario_catalog().expect("reviewed catalog should remain valid");
    let expected = catalog
        .definitions()
        .iter()
        .find(|definition| definition.slug().as_str() == CHECKPOINT_SCENARIO)
        .expect("shared scenario should remain registered");
    let expected_settings = expected
        .metadata()
        .expect("reviewed scenario should expose metadata")
        .default_settings();
    let mut testbed = InteractiveTestbed::new().expect("interactive model should load");
    let row_index = row_index(&testbed, CHECKPOINT_SCENARIO);

    // Act
    testbed
        .select_visible(row_index)
        .expect("shared scenario should select");

    // Assert
    let selected = testbed.selected().expect("selection should resolve");
    assert_eq!(selected.identity().slug(), expected.slug());
    assert_eq!(
        selected.identity().scenario_version(),
        expected.scenario_version()
    );
    assert_eq!(testbed.selected_settings(), Some(expected_settings));
    assert_eq!(testbed.session_state(), SessionState::ReadyPaused);
}

#[test]
fn captures_a_nonempty_canonical_checkpoint_after_one_logical_step() {
    // Arrange
    let mut testbed = selected_testbed(CHECKPOINT_SCENARIO);

    // Act
    testbed
        .step_once()
        .expect("one logical step should succeed");
    let reachable = testbed
        .reachable_checkpoint_id()
        .expect("step should make a declared checkpoint reachable")
        .clone();
    testbed
        .capture_reachable_checkpoint()
        .expect("reachable checkpoint should capture");

    // Assert
    let checkpoint = testbed
        .latest_checkpoint()
        .expect("canonical capture should be retained");
    assert_eq!(checkpoint.checkpoint_id(), &reachable);
    assert_eq!(testbed.completed_logical_steps(), 1);
    assert!(
        !checkpoint.observations().is_empty()
            || !checkpoint.numeric_observations().is_empty()
            || !checkpoint.ordered_occurrences().is_empty()
            || !checkpoint.unordered_sets().is_empty()
            || !checkpoint.debug_primitives().is_empty(),
        "a real native capture should contain semantic checkpoint data"
    );
}

#[test]
fn fixed_time_updates_are_independent_of_render_cadence() {
    // Arrange
    let mut coarse = selected_testbed(CADENCE_SCENARIO);
    let mut split = selected_testbed(CADENCE_SCENARIO);
    coarse.run().expect("coarse session should run");
    split.run().expect("split session should run");
    let timestep = timestep(&coarse);
    let first_part = timestep / 3;
    let Some(second_part) = timestep.checked_sub(first_part) else {
        panic!("fixture timestep should exceed its first bounded fragment");
    };

    // Act
    let coarse_ticks = coarse
        .update(timestep.saturating_mul(2))
        .expect("coarse update should advance");
    let split_ticks = [first_part, second_part, first_part, second_part]
        .into_iter()
        .try_fold(0, |total, elapsed| {
            split.update(elapsed).map(|ticks| total + ticks)
        })
        .expect("split updates should advance");

    // Assert
    assert_eq!(coarse_ticks, 2);
    assert_eq!(split_ticks, coarse_ticks);
    assert_eq!(
        split.completed_logical_steps(),
        coarse.completed_logical_steps()
    );
    assert_eq!(split.session_state(), coarse.session_state());
}

#[test]
fn settings_changes_re_resolve_and_restart_the_selected_catalog_identity() {
    // Arrange
    let mut testbed = selected_testbed(CHECKPOINT_SCENARIO);
    testbed
        .step_once()
        .expect("fixture should advance before restart");
    let original = testbed
        .selected_settings()
        .expect("selected scenario should expose settings");
    let changed = RunSettings::new(
        liquidfun_test_protocol::FloatBits::from_f32(1.0 / 120.0),
        original.velocity_iterations() + 1,
        original.position_iterations() + 1,
        original.particle_iterations() + 1,
    )
    .expect("changed settings should validate");
    let original_identity = testbed
        .current_selection()
        .expect("fixture should remain selected")
        .clone();

    // Act
    testbed
        .apply_settings(changed)
        .expect("settings should re-resolve and restart");

    // Assert
    assert_eq!(testbed.current_selection(), Some(&original_identity));
    assert_eq!(testbed.selected_settings(), Some(changed));
    assert_eq!(testbed.completed_logical_steps(), 0);
    assert_eq!(testbed.session_state(), SessionState::ReadyPaused);
}

#[test]
fn sub_clock_resolution_timestep_fails_without_advancing() {
    // Arrange
    let mut testbed = selected_testbed(CHECKPOINT_SCENARIO);
    let original = testbed
        .selected_settings()
        .expect("selected scenario should expose settings");
    let tiny = RunSettings::new(
        liquidfun_test_protocol::FloatBits::from_f32(f32::from_bits(1)),
        original.velocity_iterations(),
        original.position_iterations(),
        original.particle_iterations(),
    )
    .expect("smallest positive finite timestep should validate at the protocol boundary");
    testbed
        .apply_settings(tiny)
        .expect("catalog should re-resolve the reviewed settings change");
    testbed.run().expect("session should enter running state");

    // Act
    let error = testbed
        .update(Duration::ZERO)
        .expect_err("an unrepresentable clock duration must fail closed");

    // Assert
    assert_eq!(error, InteractiveTestbedError::TimestepBelowClockResolution);
    assert_eq!(testbed.completed_logical_steps(), 0);
    assert_eq!(testbed.session_state(), SessionState::Running);
}

fn selected_testbed(slug: &str) -> InteractiveTestbed {
    let mut testbed = InteractiveTestbed::new().expect("interactive model should load");
    let index = row_index(&testbed, slug);
    testbed
        .select_visible(index)
        .expect("reviewed scenario should select");
    testbed
}

fn row_index(testbed: &InteractiveTestbed, slug: &str) -> usize {
    testbed
        .visible_rows()
        .iter()
        .position(|row| row.selection().catalog_slug() == slug)
        .expect("reviewed scenario should have a browser row")
}

fn timestep(testbed: &InteractiveTestbed) -> Duration {
    let settings: RunSettings = testbed
        .selected_settings()
        .expect("selected scenario should have settings");
    Duration::from_secs_f64(f64::from(settings.timestep_bits().to_f32()))
}

fn assert_launcher_contract(contract: &str, required_links: &[&str]) {
    let missing = required_links
        .iter()
        .copied()
        .filter(|link| !LAUNCHER_SOURCE.contains(link))
        .collect::<Vec<_>>();

    assert!(
        missing.is_empty(),
        "interactive production {contract} wiring is incomplete: {missing:?}"
    );
}

fn assert_launcher_symbol_is_used(contract: &str, symbol: &str) {
    let occurrences = LAUNCHER_SOURCE.match_indices(symbol).count();

    assert!(
        occurrences >= 2,
        "interactive production {contract} imports or declares {symbol} without using it"
    );
}
