//! Pure product-chrome models and exact operational copy.

#![allow(
    missing_docs,
    reason = "public copy constants and identity accessors are self-describing UI contracts"
)]

mod about;
pub mod overlays;
pub mod run_controls;
mod scenario_browser;
pub mod settings;
pub mod viewport;

pub use about::*;
pub use scenario_browser::*;

use crate::app::{status_copy, status_marker};
use liquidfun_differential::{ComparisonState, SessionState};
use liquidfun_test_protocol::ResolvedScenario;

pub const PRIMARY_CTA: &str = "Run Scenario";
pub const ABOUT_LABEL: &str = "About & provenance";
pub const EMPTY_SCENARIO_HEADING: &str = "Select a scenario";
pub const EMPTY_SCENARIO_BODY: &str = "Choose a reviewed catalog scenario to resolve its run plan and inspect it headlessly or visually.";
pub const EMPTY_DIFFERENCES_HEADING: &str = "No differences at this checkpoint";
pub const EMPTY_DIFFERENCES_BODY: &str =
    "Rust and oracle observations match under the selected policies.";
pub const EMPTY_CAPTURE_HEADING: &str = "No checkpoint captured";
pub const EMPTY_CAPTURE_BODY: &str =
    "Run or step the scenario, then capture a deterministic semantic checkpoint.";
pub const LOADING_SCENARIO: &str = "Resolving scenario…";
pub const LOADING_ORACLE: &str = "Starting the pinned oracle…";
pub const LOADING_COMPARISON: &str = "Comparing semantic checkpoints…";
pub const VALIDATION_ERROR: &str =
    "This setting is not valid. Correct the highlighted value and try again.";
pub const SCENARIO_ERROR: &str =
    "Scenario could not start. Review the run details, correct the issue, and try again.";
pub const ORACLE_UNAVAILABLE: &str =
    "Oracle unavailable. Continue with Rust-only diagnostics or configure the pinned oracle.";
pub const SCREENSHOT_CLARIFICATION: &str =
    "Screenshot saved. Screenshots are diagnostic and do not prove compatibility.";
pub const WINDOW_TOO_SMALL_HEADING: &str = "Window too small";
pub const WINDOW_TOO_SMALL_BODY: &str = "Resize to at least 640 × 480";
pub const COMPACT_WINDOW_NOTICE: &str = "Compact window — panels open one at a time";
pub const UNAVAILABLE: &str = "Unavailable";

/// Compact normal-chrome app bar projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppBarModel<'a> {
    active_scenario_title: &'a str,
    status: &'static str,
    status_marker: &'static str,
}

impl<'a> AppBarModel<'a> {
    /// Builds an app bar from immutable status and optional presentation title.
    #[must_use]
    pub fn new(
        maybe_active_scenario_title: Option<&'a str>,
        session_state: SessionState,
        maybe_comparison: Option<ComparisonState>,
    ) -> Self {
        Self {
            active_scenario_title: maybe_active_scenario_title.unwrap_or(EMPTY_SCENARIO_HEADING),
            status: status_copy(session_state, maybe_comparison),
            status_marker: status_marker(session_state, maybe_comparison),
        }
    }

    /// Returns the fixed project identity.
    #[must_use]
    pub const fn project_name(&self) -> &'static str {
        "liquidfun-rs"
    }

    /// Returns the active presentation title or empty-state title.
    #[must_use]
    pub const fn active_scenario_title(&self) -> &str {
        self.active_scenario_title
    }

    /// Returns exact compact session/comparison status copy.
    #[must_use]
    pub const fn status(&self) -> &'static str {
        self.status
    }

    /// Returns the shape or glyph paired with status color and text.
    #[must_use]
    pub const fn status_marker(&self) -> &'static str {
        self.status_marker
    }

    /// Returns the accessible About affordance label.
    #[must_use]
    pub const fn about_label(&self) -> &'static str {
        ABOUT_LABEL
    }
}

/// Copyable presentation of the exact resolved run identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunIdentityView {
    catalog_schema_version: u32,
    catalog_slug: Box<str>,
    scenario_version: u32,
    generator_id: Box<str>,
    generator_version: u32,
    seed_label: Box<str>,
    timestep_bits: u32,
    velocity_iterations: u32,
    position_iterations: u32,
    particle_iterations: u32,
    content_sha256: Box<str>,
}

impl RunIdentityView {
    /// Projects only stable semantic identity fields from immutable resolved input.
    #[must_use]
    pub fn from_resolved(resolved: &ResolvedScenario) -> Self {
        let identity = resolved.identity();
        let settings = identity.settings();
        let seed_label = identity
            .maybe_seed()
            .map_or_else(|| UNAVAILABLE.to_owned(), |seed| seed.to_string());
        Self {
            catalog_schema_version: identity.catalog_schema_version().get(),
            catalog_slug: identity.slug().as_str().into(),
            scenario_version: identity.scenario_version().get(),
            generator_id: identity.generator_id().as_str().into(),
            generator_version: identity.generator_version().get(),
            seed_label: seed_label.into_boxed_str(),
            timestep_bits: settings.timestep_bits().bits(),
            velocity_iterations: settings.velocity_iterations(),
            position_iterations: settings.position_iterations(),
            particle_iterations: settings.particle_iterations(),
            content_sha256: identity.content_sha256().as_str().into(),
        }
    }

    #[must_use]
    pub const fn catalog_schema_version(&self) -> u32 {
        self.catalog_schema_version
    }

    #[must_use]
    pub fn catalog_slug(&self) -> &str {
        &self.catalog_slug
    }

    #[must_use]
    pub const fn scenario_version(&self) -> u32 {
        self.scenario_version
    }

    #[must_use]
    pub fn generator_id(&self) -> &str {
        &self.generator_id
    }

    #[must_use]
    pub const fn generator_version(&self) -> u32 {
        self.generator_version
    }

    #[must_use]
    pub fn seed_label(&self) -> &str {
        &self.seed_label
    }

    #[must_use]
    pub const fn timestep_bits(&self) -> u32 {
        self.timestep_bits
    }

    #[must_use]
    pub const fn velocity_iterations(&self) -> u32 {
        self.velocity_iterations
    }

    #[must_use]
    pub const fn position_iterations(&self) -> u32 {
        self.position_iterations
    }

    #[must_use]
    pub const fn particle_iterations(&self) -> u32 {
        self.particle_iterations
    }

    #[must_use]
    pub fn content_sha256(&self) -> &str {
        &self.content_sha256
    }
}
