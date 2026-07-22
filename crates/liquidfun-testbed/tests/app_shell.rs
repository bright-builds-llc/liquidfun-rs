//! Pure shell, catalog, provenance, and accessibility contracts.

use liquidfun_differential::{ComparisonState, SessionCommand, SessionState};
use liquidfun_test_protocol::{ResolveRequest, resolve_catalog, reviewed_scenario_catalog};
use liquidfun_testbed::{
    app::{AppEffect, AppShell, ShellLayout, ShellLayoutMode, ShellRegion, status_copy},
    theme::{DARK_THEME, FONT_SIZES, FONT_WEIGHTS, SPACING, SPACING_VALUES, TYPOGRAPHY_STYLES},
    ui::{
        ABOUT_LABEL, EMPTY_CAPTURE_BODY, EMPTY_CAPTURE_HEADING, EMPTY_DIFFERENCES_BODY,
        EMPTY_DIFFERENCES_HEADING, EMPTY_SCENARIO_BODY, EMPTY_SCENARIO_HEADING, LOADING_COMPARISON,
        LOADING_ORACLE, LOADING_SCENARIO, ORACLE_UNAVAILABLE, PRIMARY_CTA, ProvenanceInput,
        SCREENSHOT_CLARIFICATION, ScenarioBrowser, build_about_panel,
    },
};

#[test]
fn dark_theme_matches_the_approved_visual_and_accessibility_tokens() {
    // Arrange
    let theme = DARK_THEME;

    // Act
    let palette = theme.palette;

    // Assert
    assert_eq!(SPACING_VALUES, [4, 8, 16, 24, 32, 48, 64]);
    assert_eq!(FONT_SIZES, [12, 14, 18, 24]);
    assert_eq!(FONT_WEIGHTS, [400, 600]);
    assert_eq!(
        [
            SPACING.xs,
            SPACING.sm,
            SPACING.md,
            SPACING.lg,
            SPACING.xl,
            SPACING.two_xl,
            SPACING.three_xl,
        ],
        SPACING_VALUES
    );
    assert_eq!(TYPOGRAPHY_STYLES.map(|style| style.size), FONT_SIZES);
    assert_eq!(
        TYPOGRAPHY_STYLES.map(|style| style.weight),
        [400, 400, 600, 600]
    );
    assert_eq!(theme.field_radius, 4);
    assert_eq!(theme.panel_radius, 8);
    assert_eq!(theme.minimum_target, 44);
    assert_eq!(theme.focus_ring, 2);
    assert!((theme.normal_text_minimum_contrast - 4.5).abs() < f32::EPSILON);
    assert!((theme.large_text_minimum_contrast - 3.0).abs() < f32::EPSILON);
    assert_eq!(palette.dominant, "#0D1117");
    assert_eq!(palette.secondary, "#161B22");
    assert_eq!(palette.accent, "#58A6FF");
    assert_eq!(palette.destructive, "#F85149");
    assert_eq!(palette.primary_text, "#F0F6FC");
    assert_eq!(palette.secondary_text, "#B1BAC4");
    assert_eq!(palette.muted_text, "#8B949E");
    assert_eq!(palette.border, "#30363D");
    assert_eq!(palette.hover, "#21262D");
    assert_eq!(palette.success, "#3FB950");
    assert_eq!(palette.within_policy, "#D29922");
    assert_eq!(palette.rust_only, "#FF8C42");
    assert_eq!(palette.oracle_only, "#A371F7");
    assert_eq!(palette.informational, "#39C5CF");
}

#[test]
fn standard_shell_has_five_exact_regions_at_1280_pixels() {
    // Arrange
    let width = 1280;
    let height = 720;

    // Act
    let layout = ShellLayout::for_window(width, height);

    // Assert
    assert_eq!(layout.mode(), ShellLayoutMode::Standard);
    assert_eq!(layout.region(ShellRegion::AppBar), (0, 0, 1280, 48));
    assert_eq!(layout.region(ShellRegion::ScenarioRail), (0, 48, 280, 672));
    assert_eq!(layout.region(ShellRegion::Viewport), (280, 48, 640, 608));
    assert_eq!(layout.region(ShellRegion::Inspector), (920, 48, 360, 608));
    assert_eq!(layout.region(ShellRegion::Controls), (280, 656, 1000, 64));
    assert!(layout.region(ShellRegion::Viewport).2 >= 480);
}

#[test]
fn app_state_is_presentation_only_and_effects_are_typed_commands() {
    // Arrange
    let source = include_str!("../src/app/state.rs");
    let shell = AppShell::default();

    // Act
    let effect = shell.submit(SessionCommand::Run);

    // Assert
    assert!(!source.contains("SessionController"));
    assert!(!source.contains("ComparisonModel"));
    assert!(!source.contains("SessionBackend"));
    assert!(!source.contains("World"));
    assert!(!source.contains("frame_time"));
    assert!(!source.contains("logical_tick"));
    assert!(matches!(effect, AppEffect::Submit(SessionCommand::Run)));
    assert_eq!(shell.state().camera().zoom_percent(), 100);
    assert!(shell.state().selected_catalog().is_none());
}

#[test]
fn scenario_browser_searches_and_selects_stable_run_identity() {
    // Arrange
    let catalog = reviewed_scenario_catalog().expect("reviewed catalog must remain valid");
    let mut browser = ScenarioBrowser::from_catalog(&catalog)
        .expect("reviewed catalog must project to bounded browser rows");
    let expected = catalog.definitions()[0].slug().as_str().to_owned();

    // Act
    browser
        .set_query(&expected)
        .expect("stable slug is bounded");
    browser.focus_first();
    let selected = browser.select_focused().expect("one row matches its slug");

    // Assert
    assert_eq!(selected.catalog_slug(), expected);
    assert_eq!(selected.scenario_version(), 1);
    assert_eq!(browser.visible_rows().len(), 1);
    assert_eq!(browser.visible_rows()[0].minimum_target_height(), 44);
    assert!(browser.visible_rows()[0].eligibility().rust());
    assert!(browser.visible_rows()[0].eligibility().oracle());
    assert!(browser.visible_rows()[0].eligibility().visual());
}

#[test]
fn resolved_identity_uses_slug_version_seed_and_content_hash() {
    // Arrange
    let catalog = reviewed_scenario_catalog().expect("reviewed catalog must remain valid");
    let definition = &catalog.definitions()[0];
    let settings = definition
        .metadata()
        .expect("reviewed definitions have metadata")
        .default_settings();
    let request = ResolveRequest::new(definition.slug().clone(), None, settings);
    let resolved =
        resolve_catalog(catalog.definitions(), &request).expect("reviewed definition must resolve");

    // Act
    let view = liquidfun_testbed::ui::RunIdentityView::from_resolved(&resolved);

    // Assert
    assert_eq!(view.catalog_slug(), definition.slug().as_str());
    assert_eq!(view.catalog_schema_version(), 1);
    assert_eq!(view.scenario_version(), 1);
    assert_eq!(view.seed_label(), "Unavailable");
    assert_eq!(view.content_sha256().len(), 64);
    assert!(!view.generator_id().is_empty());
    assert_eq!(view.generator_version(), 1);
}

#[test]
fn shell_copy_matches_the_approved_operational_language() {
    // Assert
    assert_eq!(PRIMARY_CTA, "Run Scenario");
    assert_eq!(ABOUT_LABEL, "About & provenance");
    assert_eq!(EMPTY_SCENARIO_HEADING, "Select a scenario");
    assert_eq!(
        EMPTY_SCENARIO_BODY,
        "Choose a reviewed catalog scenario to resolve its run plan and inspect it headlessly or visually."
    );
    assert_eq!(
        EMPTY_DIFFERENCES_HEADING,
        "No differences at this checkpoint"
    );
    assert_eq!(
        EMPTY_DIFFERENCES_BODY,
        "Rust and oracle observations match under the selected policies."
    );
    assert_eq!(EMPTY_CAPTURE_HEADING, "No checkpoint captured");
    assert_eq!(
        EMPTY_CAPTURE_BODY,
        "Run or step the scenario, then capture a deterministic semantic checkpoint."
    );
    assert_eq!(LOADING_SCENARIO, "Resolving scenario…");
    assert_eq!(LOADING_ORACLE, "Starting the pinned oracle…");
    assert_eq!(LOADING_COMPARISON, "Comparing semantic checkpoints…");
    assert_eq!(
        ORACLE_UNAVAILABLE,
        "Oracle unavailable. Continue with Rust-only diagnostics or configure the pinned oracle."
    );
    assert_eq!(
        SCREENSHOT_CLARIFICATION,
        "Screenshot saved. Screenshots are diagnostic and do not prove compatibility."
    );
    assert_eq!(status_copy(SessionState::Running, None), "Running");
    assert_eq!(status_copy(SessionState::ReadyPaused, None), "Paused");
    assert_eq!(status_copy(SessionState::Stepping, None), "Stepping");
    assert_eq!(status_copy(SessionState::Comparing, None), "Comparing");
    assert_eq!(status_copy(SessionState::HarnessFailure, None), "Error");
    assert_eq!(status_copy(SessionState::NoSelection, None), "Unavailable");
    assert_eq!(
        status_copy(SessionState::Completed, Some(ComparisonState::ExactMatch)),
        "Exact match"
    );
}

#[test]
fn provenance_uses_safe_allowlisted_links_and_literal_fallbacks() {
    // Arrange
    let input = ProvenanceInput {
        version: Some("0.1.0"),
        commit: Some("0123456789abcdef0123456789abcdef01234567"),
        profile: Some("release"),
        target: Some("x86_64-unknown-linux-gnu"),
        rust_toolchain: Some("1.97.0"),
        protocol_version: Some("1"),
        adapter_version: Some("1"),
        run_identity: None,
        oracle_revision: None,
        oracle_compiler: None,
        oracle_preset: None,
        evidence_tier: None,
    };

    // Act
    let about = build_about_panel(input);

    // Assert
    assert_eq!(about.project_name(), "liquidfun-rs");
    assert_eq!(about.maintainer(), "By Peter Ryszkiewicz");
    assert_eq!(
        about.license_summary(),
        "MIT-licensed open-source Rust project"
    );
    assert_eq!(
        about.upstream_summary(),
        "LiquidFun/Box2D provenance and notices"
    );
    assert_eq!(
        about.source_url(),
        "https://github.com/bright-builds-llc/liquidfun-rs"
    );
    assert_eq!(about.openlinks_url(), "https://openlinks.us/");
    assert_eq!(about.version_label(), "Version 0.1.0");
    assert_eq!(about.commit_label(), "Commit 0123456789ab");
    assert!(
        about
            .commit_url()
            .starts_with("https://github.com/bright-builds-llc/liquidfun-rs/commit/")
    );
    assert_eq!(about.run_identity(), "Unavailable");
    assert_eq!(about.oracle_identity(), "Oracle Unavailable");
    assert!(
        about
            .links()
            .iter()
            .all(|link| link.url().starts_with("https://"))
    );
    assert!(
        about
            .links()
            .iter()
            .all(liquidfun_testbed::ui::SafeExternalLink::copyable_fallback)
    );
}

#[test]
fn provenance_sanitizes_untrusted_text_and_rejects_untrusted_commit_links() {
    // Arrange
    let long_profile = "x".repeat(400);
    let input = ProvenanceInput {
        version: Some("bad\nversion"),
        commit: Some("../../not-a-commit"),
        profile: Some(&long_profile),
        target: None,
        rust_toolchain: None,
        protocol_version: None,
        adapter_version: None,
        run_identity: None,
        oracle_revision: None,
        oracle_compiler: None,
        oracle_preset: None,
        evidence_tier: None,
    };

    // Act
    let about = build_about_panel(input);

    // Assert
    assert_eq!(about.version_label(), "Version Unavailable");
    assert_eq!(about.commit_label(), "Commit Unavailable");
    assert_eq!(about.commit_url(), "Unavailable");
    assert_eq!(about.profile(), "Unavailable");
    assert_eq!(about.target(), "Unavailable");
}
