//! Executable UI-SPEC contract for semantic differences and responsive diagnostics.

use std::path::PathBuf;

use liquidfun::math::Vec2;
use liquidfun_differential::{ComparisonLimits, compare_canonical_checkpoints};
use liquidfun_test_protocol::{
    CanonicalCheckpoint, CheckpointId, CheckpointPosition, FloatBits, Phase4PolicyProfile,
    RequestId, Sha256Hex,
};
use liquidfun_testbed::{
    screenshot::{VisualContractOptions, run_visual_contract_check},
    ui::{
        accessibility::{
            ACCESSIBILITY_CONTRACT, NORMAL_FOCUS_ORDER, SelectableValue,
            focused_difference_announcement,
        },
        differences::{
            BackendAvailability, ComparisonMode, DifferenceList, DifferenceSort, visual_cue,
        },
        inspector::{ErrorPanel, InspectorState, InspectorTab, operational_copy},
        layout::{
            CompactWindowNotice, FocusId, FocusReturn, PanelBehavior, ResponsiveLayout,
            ResponsivePresentation,
        },
        viewport::Camera,
    },
};

const RESOLVED_SHA256: &str = "ea5c1364ab3e2c50aafc2edb9aa09fe436e19f4b3fe8d48ff69ece5da1bd0860";

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("testbed package should be nested under the workspace root")
        .to_path_buf()
}

fn comparison_model() -> liquidfun_differential::ComparisonModel {
    comparison_model_with_oracle_time(1.0 / 60.0)
}

fn comparison_model_with_oracle_time(oracle_time: f32) -> liquidfun_differential::ComparisonModel {
    let checkpoint = CanonicalCheckpoint::new(
        RequestId::new("visual-contract").expect("request ID should be valid"),
        Sha256Hex::new(RESOLVED_SHA256.to_owned()).expect("SHA-256 should be valid"),
        CheckpointId::new("checkpoint-0001").expect("checkpoint ID should be valid"),
        CheckpointPosition::LogicalStep { ordinal: 1 },
        FloatBits::from_f32(1.0 / 60.0),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .expect("checkpoint should be valid");
    let oracle_checkpoint = CanonicalCheckpoint::new(
        RequestId::new("visual-contract").expect("request ID should be valid"),
        Sha256Hex::new(RESOLVED_SHA256.to_owned()).expect("SHA-256 should be valid"),
        CheckpointId::new("checkpoint-0001").expect("checkpoint ID should be valid"),
        CheckpointPosition::LogicalStep { ordinal: 1 },
        FloatBits::from_f32(oracle_time),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .expect("oracle checkpoint should be valid");
    let policy = Phase4PolicyProfile::parse_toml(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../protocol/tolerances/phase4-v1.toml"
    )))
    .expect("policy should be valid");
    compare_canonical_checkpoints(
        &oracle_checkpoint,
        &checkpoint,
        &policy,
        ComparisonLimits::phase11_default(),
    )
    .expect("comparison should be valid")
}

#[test]
fn overlay_and_side_by_side_share_canonical_semantic_projection() {
    // Arrange
    let comparison = comparison_model();
    let camera = Camera::new(Vec2::new(2.0, -3.0), 80.0).expect("camera should be valid");
    let overlay = DifferenceList::new(&comparison, camera, BackendAvailability::Both);
    let mut side_by_side = DifferenceList::new(&comparison, camera, BackendAvailability::Both);

    // Act
    side_by_side.set_mode(ComparisonMode::SideBySide);

    // Assert
    assert_eq!(overlay.mode(), ComparisonMode::Overlay);
    assert_eq!(
        overlay.semantic_projection(),
        side_by_side.semantic_projection()
    );
    assert_eq!(overlay.camera(), side_by_side.camera());
}

#[test]
fn canonical_difference_navigation_wraps_and_announces_ordinal() {
    // Arrange
    let comparison = comparison_model_with_oracle_time(1.0 / 30.0);
    let mut differences =
        DifferenceList::new(&comparison, Camera::default(), BackendAvailability::Both);

    // Act
    differences.focus_previous();
    let previous = differences.focused_announcement();
    differences.focus_next();
    let next = differences.focused_announcement();

    // Assert
    assert_eq!(differences.sort(), DifferenceSort::CanonicalSemanticPath);
    assert!(differences.is_canonical_path_ordered());
    assert!(previous.starts_with("Difference "));
    assert_eq!(next, "Difference 1 of 1");
}

#[test]
fn exact_model_keeps_overlay_entries_but_has_an_empty_difference_list() {
    // Arrange
    let comparison = comparison_model();

    // Act
    let differences =
        DifferenceList::new(&comparison, Camera::default(), BackendAvailability::Both);

    // Assert
    assert_eq!(differences.semantic_projection().paths().len(), 10);
    assert!(differences.entries().is_empty());
    assert_eq!(
        differences.focused_announcement(),
        "No differences at this checkpoint"
    );
}

#[test]
fn visual_cues_redundantly_encode_every_comparison_state() {
    // Arrange
    let states = [
        liquidfun_differential::ComparisonState::ExactMatch,
        liquidfun_differential::ComparisonState::WithinPolicy,
        liquidfun_differential::ComparisonState::PhysicsMismatch,
        liquidfun_differential::ComparisonState::RustOnly,
        liquidfun_differential::ComparisonState::OracleOnly,
    ];

    // Act
    let cues = states.map(visual_cue);

    // Assert
    assert_eq!(cues[0].opacity_percent(), 35);
    assert_eq!(cues[1].marker(), "◇");
    assert_eq!(cues[2].marker(), "×");
    assert!(cues[2].focused_halo());
    assert_eq!(cues[3].marker(), "R");
    assert_eq!(cues[3].stroke(), "solid");
    assert_eq!(cues[4].marker(), "O");
    assert_eq!(cues[4].stroke(), "dashed");
}

#[test]
fn responsive_layout_matches_every_approved_breakpoint() {
    // Arrange
    let windows = [(1280, 720), (1100, 720), (800, 600), (700, 600), (639, 479)];

    // Act
    let layouts = windows.map(|(width, height)| ResponsiveLayout::for_window(width, height));

    // Assert
    assert_eq!(layouts[0].panel_behavior(), PanelBehavior::BothVisible);
    assert_eq!(layouts[1].panel_behavior(), PanelBehavior::InspectorDrawer);
    assert_eq!(
        layouts[2].panel_behavior(),
        PanelBehavior::MutuallyExclusiveDrawers
    );
    assert_eq!(layouts[2].control_rows(), 2);
    assert_eq!(layouts[3].panel_behavior(), PanelBehavior::FullWindowSheets);
    assert_eq!(
        layouts[3].compact_notice(),
        Some("Compact window — panels open one at a time")
    );
    assert_eq!(layouts[4].panel_behavior(), PanelBehavior::WindowTooSmall);
    assert_eq!(
        layouts[4].minimum_window_copy(),
        Some(("Window too small", "Resize to at least 640 × 480"))
    );
    assert_eq!(
        layouts[4].minimum_window_affordances(),
        ["Close", "About & provenance"]
    );
}

#[test]
fn compact_window_notice_is_announced_once_per_session() {
    // Arrange
    let layout = ResponsiveLayout::for_window(700, 600);
    let mut notice = CompactWindowNotice::default();

    // Act
    let first = notice.take(layout);
    let repeated = notice.take(layout);

    // Assert
    assert_eq!(first, Some("Compact window — panels open one at a time"));
    assert_eq!(repeated, None);
}

#[test]
fn resize_preserves_camera_selection_checkpoint_and_controller_identity() {
    // Arrange
    let camera = Camera::new(Vec2::new(7.0, 11.0), 125.0).expect("camera should be valid");
    let mut presentation = ResponsivePresentation::new(
        camera,
        "shape/body-0001/fixture-0001/child-0000",
        "checkpoint-0007",
        "visual-contract",
    )
    .expect("presentation identity should be valid");
    let before = presentation.identity_snapshot();

    // Act
    presentation.resize(800, 600, 2.0);

    // Assert
    assert_eq!(presentation.identity_snapshot(), before);
}

#[test]
fn modal_focus_moves_inside_and_returns_to_invoker() {
    // Arrange
    let mut focus = FocusReturn::default();

    // Act
    focus.open(FocusId::InspectorButton, FocusId::InspectorHeading);
    let opened = focus.current();
    let returned = focus.close();

    // Assert
    assert_eq!(opened, Some(FocusId::InspectorHeading));
    assert_eq!(returned, Some(FocusId::InspectorButton));
    assert_eq!(focus.current(), Some(FocusId::InspectorButton));

    focus.open(FocusId::ScenarioButton, FocusId::ScenarioSearch);
    focus.move_to(FocusId::ScenarioRow);
    assert_eq!(focus.current(), Some(FocusId::ScenarioRow));
    assert_eq!(focus.close(), Some(FocusId::ScenarioButton));

    focus.open(FocusId::SettingsButton, FocusId::SettingsHeading);
    focus.move_to(FocusId::SettingsField);
    assert_eq!(focus.current(), Some(FocusId::SettingsField));
    focus.move_to(FocusId::SettingsApply);
    assert_eq!(focus.current(), Some(FocusId::SettingsApply));
    assert_eq!(focus.close(), Some(FocusId::SettingsButton));
    assert_eq!(focus.current(), Some(FocusId::SettingsButton));
}

#[test]
fn accessibility_contract_has_targets_focus_contrast_and_reduced_motion() {
    // Arrange
    let contract = ACCESSIBILITY_CONTRACT;

    // Act
    let announcement = focused_difference_announcement(
        "Physics mismatch",
        3,
        12,
        "primitives/shape/body-0001/position.x",
        "position-absolute-v1",
        true,
        true,
    )
    .expect("bounded announcement should be valid");

    // Assert
    assert_eq!(contract.minimum_target_pixels(), 44);
    assert_eq!(contract.focus_ring_pixels(), 2);
    assert!(contract.focus_contrast_ratio() >= 3.0);
    assert!(contract.normal_text_contrast_ratio() >= 4.5);
    assert!(contract.maximum_transition_millis() <= 200);
    assert!(!contract.flashing_allowed());
    assert_eq!(
        NORMAL_FOCUS_ORDER,
        [
            "App bar",
            "Scenario browser",
            "Simulation viewport",
            "Run controls",
            "Inspector",
            "About & provenance",
        ]
    );
    let selectable = SelectableValue::new("primitives/shape/body-0001/position.x");
    assert!(selectable.is_selectable());
    assert_eq!(
        selectable.copy_text(),
        "primitives/shape/body-0001/position.x"
    );
    assert_eq!(
        announcement,
        "Physics mismatch, difference 3 of 12, primitives/shape/body-0001/position.x, policy position-absolute-v1, Rust value present, oracle value present."
    );
}

#[test]
fn inspector_exposes_exact_tabs_copy_and_bounded_retained_errors() {
    // Arrange
    let tabs = InspectorTab::ALL;
    let error = ErrorPanel::new(
        InspectorState::HarnessFailure,
        "Oracle process did not start.",
        "Configure the pinned oracle and retry.",
        "bounded category=process_start",
        Some("checkpoint-0004"),
    )
    .expect("bounded error should be valid");

    // Act
    let oracle = operational_copy(InspectorState::OracleUnavailable);

    // Assert
    assert_eq!(
        tabs.map(InspectorTab::label),
        ["Run", "Observe", "Differences", "Provenance"]
    );
    assert_eq!(
        oracle.body(),
        "Oracle unavailable. Continue with Rust-only diagnostics or configure the pinned oracle."
    );
    assert_eq!(error.heading(), "Harness failure");
    assert_eq!(error.retained_checkpoint(), Some("checkpoint-0004"));
    assert_eq!(error.details(), "bounded category=process_start");
}

#[test]
fn inspector_copy_distinguishes_loading_mismatch_and_error_authority() {
    // Arrange
    let states = [
        InspectorState::NoSelection,
        InspectorState::Resolving,
        InspectorState::Comparing,
        InspectorState::ExactMatch,
        InspectorState::RecoverableError,
        InspectorState::HarnessFailure,
        InspectorState::OracleUnavailable,
    ];

    // Act
    let headings = states.map(|state| operational_copy(state).heading());

    // Assert
    assert_eq!(headings[1], "Resolving scenario…");
    assert_eq!(headings[2], "Comparing semantic checkpoints…");
    assert_eq!(headings[3], "No differences at this checkpoint");
    assert_eq!(headings[4], "Scenario could not start");
    assert_eq!(headings[5], "Harness failure");
    assert_eq!(headings[6], "Oracle unavailable");
}

#[test]
fn inspector_rejects_unbounded_diagnostic_details() {
    // Arrange
    let oversized = "x".repeat(513);

    // Act
    let result = ErrorPanel::new(
        InspectorState::RecoverableError,
        "Scenario could not start.",
        "Correct the issue and retry.",
        &oversized,
        Some("checkpoint-0004"),
    );

    // Assert
    assert!(result.is_err());
}

#[test]
fn diagnostic_capture_records_provenance_hash_and_non_authority() {
    // Arrange
    let root = repository_root();
    let output = PathBuf::from("target/testbed-visual-contract-tests/capture");
    let options = VisualContractOptions::new(
        root.join("crates/liquidfun-differential/tests/fixtures/catalog/phase11-v1.json"),
        output,
        "c595f0e",
    )
    .expect("visual contract options should be valid");

    // Act
    let report = run_visual_contract_check(&options).expect("visual contract should pass");
    let replay = run_visual_contract_check(&options).expect("visual contract replay should pass");

    // Assert
    assert!(report.all_passed());
    assert_eq!(report.resolved_sha256(), RESOLVED_SHA256);
    assert_eq!(report.commit(), "c595f0e");
    assert_eq!(
        report.authority(),
        "Diagnostic only — screenshots do not prove compatibility."
    );
    assert!(!report.contributes_to_comparison());
    assert!(!report.contributes_to_evidence());
    assert!(
        report
            .artifacts()
            .iter()
            .all(liquidfun_testbed::screenshot::DiagnosticArtifact::is_regular)
    );
    assert_eq!(report, replay);
}
