//! Passive five-region shell over immutable semantic inputs.

#![allow(
    missing_docs,
    reason = "closed region variants are named by their UI-SPEC contract"
)]

use liquidfun_differential::{ComparisonModel, ComparisonState, SessionCommand, SessionState};
use liquidfun_test_protocol::RunIdentity;

mod state;
pub use state::*;

const APP_BAR_HEIGHT: u32 = 48;
const SCENARIO_RAIL_WIDTH: u32 = 280;
const NARROW_SCENARIO_RAIL_WIDTH: u32 = 240;
const INSPECTOR_WIDTH: u32 = 360;
const CONTROL_STRIP_HEIGHT: u32 = 64;
const MINIMUM_WINDOW_WIDTH: u32 = 640;
const MINIMUM_WINDOW_HEIGHT: u32 = 480;

type RegionRect = (u32, u32, u32, u32);

/// Five required regions in visual traversal order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellRegion {
    AppBar,
    ScenarioRail,
    Viewport,
    Inspector,
    Controls,
}

/// Responsive arrangement selected without changing semantic state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellLayoutMode {
    Standard,
    InspectorDrawer,
    ModalDrawers,
    FullWindowSheets,
    WindowTooSmall,
}

/// Pure logical layout for the five required shell regions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShellLayout {
    mode: ShellLayoutMode,
    app_bar: RegionRect,
    scenario_rail: RegionRect,
    viewport: RegionRect,
    inspector: RegionRect,
    controls: RegionRect,
}

impl ShellLayout {
    /// Computes a responsive logical layout without side effects.
    #[must_use]
    pub const fn for_window(width: u32, height: u32) -> Self {
        if width < MINIMUM_WINDOW_WIDTH || height < MINIMUM_WINDOW_HEIGHT {
            return too_small_layout(width, height);
        }
        if width < 720 {
            return full_window_sheet_layout(width, height);
        }
        if width < 960 {
            return modal_drawer_layout(width, height);
        }
        if width < 1280 {
            return inspector_drawer_layout(width, height);
        }
        standard_layout(width, height)
    }

    /// Returns the selected responsive arrangement.
    #[must_use]
    pub const fn mode(self) -> ShellLayoutMode {
        self.mode
    }

    /// Returns `(x, y, width, height)` for one required region.
    #[must_use]
    pub const fn region(self, region: ShellRegion) -> RegionRect {
        match region {
            ShellRegion::AppBar => self.app_bar,
            ShellRegion::ScenarioRail => self.scenario_rail,
            ShellRegion::Viewport => self.viewport,
            ShellRegion::Inspector => self.inspector,
            ShellRegion::Controls => self.controls,
        }
    }
}

const fn standard_layout(width: u32, height: u32) -> ShellLayout {
    let content_height = height - APP_BAR_HEIGHT - CONTROL_STRIP_HEIGHT;
    let viewport_width = width - SCENARIO_RAIL_WIDTH - INSPECTOR_WIDTH;
    ShellLayout {
        mode: ShellLayoutMode::Standard,
        app_bar: (0, 0, width, APP_BAR_HEIGHT),
        scenario_rail: (
            0,
            APP_BAR_HEIGHT,
            SCENARIO_RAIL_WIDTH,
            height - APP_BAR_HEIGHT,
        ),
        viewport: (
            SCENARIO_RAIL_WIDTH,
            APP_BAR_HEIGHT,
            viewport_width,
            content_height,
        ),
        inspector: (
            width - INSPECTOR_WIDTH,
            APP_BAR_HEIGHT,
            INSPECTOR_WIDTH,
            content_height,
        ),
        controls: (
            SCENARIO_RAIL_WIDTH,
            height - CONTROL_STRIP_HEIGHT,
            width - SCENARIO_RAIL_WIDTH,
            CONTROL_STRIP_HEIGHT,
        ),
    }
}

const fn inspector_drawer_layout(width: u32, height: u32) -> ShellLayout {
    let content_height = height - APP_BAR_HEIGHT - CONTROL_STRIP_HEIGHT;
    ShellLayout {
        mode: ShellLayoutMode::InspectorDrawer,
        app_bar: (0, 0, width, APP_BAR_HEIGHT),
        scenario_rail: (
            0,
            APP_BAR_HEIGHT,
            NARROW_SCENARIO_RAIL_WIDTH,
            height - APP_BAR_HEIGHT,
        ),
        viewport: (
            NARROW_SCENARIO_RAIL_WIDTH,
            APP_BAR_HEIGHT,
            width - NARROW_SCENARIO_RAIL_WIDTH,
            content_height,
        ),
        inspector: (
            width - INSPECTOR_WIDTH,
            APP_BAR_HEIGHT,
            INSPECTOR_WIDTH,
            content_height,
        ),
        controls: (
            NARROW_SCENARIO_RAIL_WIDTH,
            height - CONTROL_STRIP_HEIGHT,
            width - NARROW_SCENARIO_RAIL_WIDTH,
            CONTROL_STRIP_HEIGHT,
        ),
    }
}

const fn modal_drawer_layout(width: u32, height: u32) -> ShellLayout {
    let controls_height = 2 * 44;
    let content_height = height - APP_BAR_HEIGHT - controls_height;
    ShellLayout {
        mode: ShellLayoutMode::ModalDrawers,
        app_bar: (0, 0, width, APP_BAR_HEIGHT),
        scenario_rail: (0, APP_BAR_HEIGHT, width, content_height),
        viewport: (0, APP_BAR_HEIGHT, width, content_height),
        inspector: (0, APP_BAR_HEIGHT, width, content_height),
        controls: (0, height - controls_height, width, controls_height),
    }
}

const fn full_window_sheet_layout(width: u32, height: u32) -> ShellLayout {
    let content_height = height - APP_BAR_HEIGHT - CONTROL_STRIP_HEIGHT;
    ShellLayout {
        mode: ShellLayoutMode::FullWindowSheets,
        app_bar: (0, 0, width, APP_BAR_HEIGHT),
        scenario_rail: (0, APP_BAR_HEIGHT, width, content_height),
        viewport: (0, APP_BAR_HEIGHT, width, content_height),
        inspector: (0, APP_BAR_HEIGHT, width, content_height),
        controls: (
            0,
            height - CONTROL_STRIP_HEIGHT,
            width,
            CONTROL_STRIP_HEIGHT,
        ),
    }
}

const fn too_small_layout(width: u32, height: u32) -> ShellLayout {
    let app_bar_height = if height < APP_BAR_HEIGHT {
        height
    } else {
        APP_BAR_HEIGHT
    };
    ShellLayout {
        mode: ShellLayoutMode::WindowTooSmall,
        app_bar: (0, 0, width, app_bar_height),
        scenario_rail: (0, 0, 0, 0),
        viewport: (0, app_bar_height, width, height - app_bar_height),
        inspector: (0, 0, 0, 0),
        controls: (0, 0, 0, 0),
    }
}

/// Immutable semantic values supplied by controller and comparator owners.
#[derive(Debug, Clone, Copy)]
pub struct ReadOnlyAppInputs<'a> {
    session_state: SessionState,
    maybe_comparison: Option<&'a ComparisonModel>,
    maybe_run_identity: Option<&'a RunIdentity>,
}

impl<'a> ReadOnlyAppInputs<'a> {
    /// Creates one borrowed shell input snapshot.
    #[must_use]
    pub const fn new(
        session_state: SessionState,
        maybe_comparison: Option<&'a ComparisonModel>,
        maybe_run_identity: Option<&'a RunIdentity>,
    ) -> Self {
        Self {
            session_state,
            maybe_comparison,
            maybe_run_identity,
        }
    }

    /// Returns the closed session status supplied to the shell.
    #[must_use]
    pub const fn session_state(self) -> SessionState {
        self.session_state
    }

    /// Returns the borrowed semantic comparison model.
    #[must_use]
    pub const fn comparison(self) -> Option<&'a ComparisonModel> {
        self.maybe_comparison
    }

    /// Returns the borrowed exact run identity.
    #[must_use]
    pub const fn run_identity(self) -> Option<&'a RunIdentity> {
        self.maybe_run_identity
    }
}

/// The sole effectful output permitted from the shell.
#[derive(Debug, Clone)]
pub enum AppEffect {
    /// Submit one closed command to the external session owner.
    Submit(SessionCommand),
}

/// Pure shell state and typed effect translation.
#[derive(Debug, Clone, Default)]
pub struct AppShell {
    state: AppState,
}

impl AppShell {
    /// Returns presentation-only local state.
    #[must_use]
    pub const fn state(&self) -> &AppState {
        &self.state
    }

    /// Translates an admitted interaction into the only permitted effect type.
    #[must_use]
    pub const fn submit(&self, command: SessionCommand) -> AppEffect {
        AppEffect::Submit(command)
    }
}

/// Returns exact compact status copy for the app bar.
#[must_use]
pub const fn status_copy(
    state: SessionState,
    maybe_comparison: Option<ComparisonState>,
) -> &'static str {
    if let Some(comparison) = maybe_comparison {
        return match comparison {
            ComparisonState::ExactMatch => "Exact match",
            ComparisonState::WithinPolicy => "Within policy",
            ComparisonState::PhysicsMismatch => "Physics mismatch",
            ComparisonState::RustOnly => "Rust-only",
            ComparisonState::OracleOnly => "Oracle-only",
        };
    }
    match state {
        SessionState::Running => "Running",
        SessionState::ReadyPaused | SessionState::Completed => "Paused",
        SessionState::Stepping => "Stepping",
        SessionState::Resolving => "Resolving",
        SessionState::Comparing => "Comparing",
        SessionState::RecoverableError | SessionState::HarnessFailure => "Error",
        SessionState::NoSelection => "Unavailable",
    }
}

/// Returns the non-color marker paired with compact status text.
#[must_use]
pub const fn status_marker(
    state: SessionState,
    maybe_comparison: Option<ComparisonState>,
) -> &'static str {
    if let Some(comparison) = maybe_comparison {
        return match comparison {
            ComparisonState::ExactMatch => "✓",
            ComparisonState::WithinPolicy => "◇",
            ComparisonState::PhysicsMismatch => "×",
            ComparisonState::RustOnly => "R",
            ComparisonState::OracleOnly => "O",
        };
    }
    match state {
        SessionState::Running => "▶",
        SessionState::ReadyPaused | SessionState::Completed => "‖",
        SessionState::Stepping => "→",
        SessionState::Resolving | SessionState::NoSelection => "○",
        SessionState::Comparing => "◇",
        SessionState::RecoverableError | SessionState::HarnessFailure => "×",
    }
}
