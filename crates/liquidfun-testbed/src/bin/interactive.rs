//! Optional private interactive catalog testbed.

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;

use liquidfun_differential::{
    ComparisonLimits, ComparisonModel, ComparisonState, compare_canonical_checkpoints,
};
use liquidfun_test_protocol::{
    CanonicalCheckpoint, CheckpointId, DebugLayerName, FloatBits, HarnessLimits,
    Phase4PolicyProfile, RunSettings, Sha256Hex, decode_canonical_checkpoint_jsonl,
};
use liquidfun_testbed::app::{AppShell, ShellRegion, status_copy, status_marker};
use liquidfun_testbed::input::{
    InputContext, InputEffect, KeyboardKey, PresentationAction, resolve_key,
};
use liquidfun_testbed::interactive::InteractiveTestbed;
use liquidfun_testbed::ui::differences::{BackendAvailability, ComparisonMode, DifferenceList};
use liquidfun_testbed::ui::inspector::{InspectorState, operational_copy};
use liquidfun_testbed::ui::layout::{PanelBehavior, ResponsiveLayout};
use liquidfun_testbed::ui::protocol_viewport::{
    ProtocolComparisonBackend, ProtocolLayerVisibility, ProtocolViewport,
    draw_protocol_comparison_frame, draw_protocol_frame, project_checkpoint,
};
use liquidfun_testbed::ui::viewport::Camera;
use macroquad::prelude::*;

const BACKGROUND: Color = Color::new(0.051, 0.067, 0.090, 1.0);
const PANEL: Color = Color::new(0.082, 0.102, 0.133, 1.0);
const PANEL_ALT: Color = Color::new(0.110, 0.137, 0.176, 1.0);
const BORDER: Color = Color::new(0.251, 0.286, 0.337, 1.0);
const TEXT: Color = Color::new(0.902, 0.929, 0.961, 1.0);
const MUTED: Color = Color::new(0.651, 0.690, 0.741, 1.0);
const ACCENT: Color = Color::new(0.345, 0.651, 1.0, 1.0);
const ERROR: Color = Color::from_rgba(248, 81, 73, 255);
const ROW_HEIGHT: f32 = 44.0;
const FONT: f32 = 18.0;
const SMALL_FONT: f32 = 14.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpenPanel {
    None,
    Scenario,
    Inspector,
    ShortcutHelp,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "liquidfun-rs semantic testbed".to_owned(),
        window_width: 1280,
        window_height: 800,
        high_dpi: true,
        sample_count: 4,
        ..Default::default()
    }
}

struct DesktopApp {
    shell: AppShell,
    testbed: InteractiveTestbed,
    focused_row: usize,
    query: String,
    editing_query: bool,
    layers: ProtocolLayerVisibility,
    layer_enabled: [bool; 9],
    pixels_per_meter: f32,
    center_x: f32,
    center_y: f32,
    maybe_drag_origin: Option<(f32, f32)>,
    maybe_oracle: Option<CanonicalCheckpoint>,
    maybe_comparison: Option<ComparisonModel>,
    maybe_compared_identity: Option<(Sha256Hex, CheckpointId)>,
    maybe_error: Option<String>,
    open_panel: OpenPanel,
    diagnostics_visible: bool,
    comparison_mode: ComparisonMode,
    focused_difference: usize,
}

impl DesktopApp {
    fn new(maybe_oracle_path: Option<PathBuf>) -> Result<Self, String> {
        let maybe_oracle = maybe_oracle_path.map(load_oracle_checkpoint).transpose()?;
        let mut testbed = InteractiveTestbed::new().map_err(bounded_error)?;
        let first_visual = testbed
            .visible_rows()
            .iter()
            .position(|row| row.eligibility().visual())
            .ok_or_else(|| "reviewed catalog has no visual scenario".to_owned())?;
        testbed
            .select_visible(first_visual)
            .map_err(bounded_error)?;
        testbed.step_once().map_err(bounded_error)?;
        testbed
            .capture_reachable_checkpoint()
            .map_err(bounded_error)?;

        Ok(Self {
            shell: AppShell::default(),
            testbed,
            focused_row: first_visual,
            query: String::new(),
            editing_query: false,
            layers: ProtocolLayerVisibility::all(),
            layer_enabled: [true; 9],
            pixels_per_meter: 42.0,
            center_x: 0.0,
            center_y: 0.0,
            maybe_drag_origin: None,
            maybe_oracle,
            maybe_comparison: None,
            maybe_compared_identity: None,
            maybe_error: None,
            open_panel: OpenPanel::None,
            diagnostics_visible: true,
            comparison_mode: ComparisonMode::Overlay,
            focused_difference: 0,
        })
    }

    fn update(&mut self) {
        self.handle_search_input();
        self.handle_panel_input();
        self.handle_scenario_navigation();
        self.handle_controller_input();
        self.handle_settings_input();
        self.handle_viewport_input();

        let elapsed = Duration::from_secs_f32(get_frame_time().clamp(0.0, 0.25));
        match self.testbed.update(elapsed) {
            Ok(ticks) if ticks > 0 && self.testbed.reachable_checkpoint_id().is_some() => {
                if let Err(error) = self.testbed.capture_reachable_checkpoint() {
                    self.set_error(error);
                }
            }
            Ok(_) => {}
            Err(error) => self.set_error(error),
        }
        self.refresh_comparison();
    }

    fn handle_search_input(&mut self) {
        let shifted = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
        if is_key_pressed(KeyCode::Slash) && !shifted && !self.editing_query {
            self.editing_query = true;
            self.open_panel = OpenPanel::Scenario;
            return;
        }
        if !self.editing_query {
            return;
        }
        while let Some(character) = get_char_pressed() {
            if !character.is_control() && self.query.len() < 128 {
                self.query.push(character);
            }
        }
        if is_key_pressed(KeyCode::Backspace) {
            self.query.pop();
        }
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Escape) {
            self.editing_query = false;
        }
        if let Err(error) = self.testbed.set_query(&self.query) {
            self.set_error(error);
        }
        self.focused_row = self
            .focused_row
            .min(self.testbed.visible_rows().len().saturating_sub(1));
    }

    fn handle_panel_input(&mut self) {
        if self.editing_query {
            return;
        }
        if is_key_pressed(KeyCode::B) {
            self.open_panel = if self.open_panel == OpenPanel::Scenario {
                OpenPanel::None
            } else {
                OpenPanel::Scenario
            };
        }
        if is_key_pressed(KeyCode::I) {
            self.open_panel = if self.open_panel == OpenPanel::Inspector {
                OpenPanel::None
            } else {
                OpenPanel::Inspector
            };
        }
        if is_key_pressed(KeyCode::O) && self.maybe_comparison.is_some() {
            self.comparison_mode = match self.comparison_mode {
                ComparisonMode::Overlay => ComparisonMode::SideBySide,
                ComparisonMode::SideBySide | ComparisonMode::SingleBackend => {
                    ComparisonMode::Overlay
                }
            };
        }
        if is_key_pressed(KeyCode::Escape) {
            self.open_panel = OpenPanel::None;
        }
    }

    fn handle_scenario_navigation(&mut self) {
        let responsive = ResponsiveLayout::for_window(
            bounded_screen_dimension(screen_width()),
            bounded_screen_dimension(screen_height()),
        );
        let scenario_visible = match responsive.panel_behavior() {
            PanelBehavior::BothVisible | PanelBehavior::InspectorDrawer => true,
            PanelBehavior::MutuallyExclusiveDrawers | PanelBehavior::FullWindowSheets => {
                self.open_panel == OpenPanel::Scenario
            }
            PanelBehavior::WindowTooSmall => false,
        };
        if !scenario_visible || self.editing_query || self.testbed.visible_rows().is_empty() {
            return;
        }
        if is_key_pressed(KeyCode::Down) {
            self.focused_row = (self.focused_row + 1) % self.testbed.visible_rows().len();
        }
        if is_key_pressed(KeyCode::Up) {
            self.focused_row = (self.focused_row + self.testbed.visible_rows().len() - 1)
                % self.testbed.visible_rows().len();
        }
        if is_key_pressed(KeyCode::Enter) {
            self.select_focused();
        }

        let scenario_region = responsive.shell().region(ShellRegion::ScenarioRail);
        let (region_x, region_y, region_width, _) = rect(scenario_region);
        let (mouse_x, mouse_y) = mouse_position();
        let maybe_clicked = self
            .testbed
            .visible_rows()
            .iter()
            .take(14)
            .enumerate()
            .find(|(index, _row)| {
                let Ok(row_index) = u16::try_from(*index) else {
                    return false;
                };
                let row_y = region_y + 44.0 + f32::from(row_index) * ROW_HEIGHT;
                (region_x..region_x + region_width).contains(&mouse_x)
                    && (row_y..row_y + ROW_HEIGHT).contains(&mouse_y)
            })
            .map(|(index, _row)| index);
        if is_mouse_button_pressed(MouseButton::Left)
            && let Some(index) = maybe_clicked
        {
            self.focused_row = index;
            self.select_focused();
        }
    }

    fn select_focused(&mut self) {
        match self.testbed.select_visible(self.focused_row) {
            Ok(()) => self.clear_comparison(),
            Err(error) => self.set_error(error),
        }
    }

    fn handle_controller_input(&mut self) {
        if self.editing_query {
            return;
        }
        let shifted = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
        let maybe_key = (is_key_pressed(KeyCode::Slash) && shifted)
            .then_some(KeyboardKey::QuestionMark)
            .or_else(|| {
                [
                    (KeyCode::Space, KeyboardKey::Space),
                    (KeyCode::Right, KeyboardKey::Right),
                    (KeyCode::R, KeyboardKey::R),
                    (KeyCode::C, KeyboardKey::C),
                    (KeyCode::Key1, KeyboardKey::Digit1),
                    (KeyCode::Key2, KeyboardKey::Digit2),
                    (KeyCode::Key3, KeyboardKey::Digit3),
                    (KeyCode::Key4, KeyboardKey::Digit4),
                    (KeyCode::F, KeyboardKey::F),
                    (KeyCode::LeftBracket, KeyboardKey::LeftBracket),
                    (KeyCode::RightBracket, KeyboardKey::RightBracket),
                    (KeyCode::Home, KeyboardKey::Home),
                ]
                .into_iter()
                .find_map(|(macroquad_key, semantic_key)| {
                    is_key_pressed(macroquad_key).then_some(semantic_key)
                })
            });
        let Some(key) = maybe_key else {
            return;
        };
        let checkpoint_id = self.testbed.reachable_checkpoint_id().cloned();
        let effect = resolve_key(
            key,
            InputContext {
                session_state: self.testbed.session_state(),
                editing_field: false,
                maybe_checkpoint_id: checkpoint_id.as_ref(),
                scenario_shortcuts: &[],
            },
        );
        let Some(effect) = effect else {
            return;
        };
        match effect {
            InputEffect::Controller(action) => {
                let clears_comparison = matches!(
                    &action,
                    liquidfun_testbed::controller_adapter::ControllerAction::Select(_)
                        | liquidfun_testbed::controller_adapter::ControllerAction::Restart
                        | liquidfun_testbed::controller_adapter::ControllerAction::ApplySettingsAndRestart { .. }
                );
                match self.testbed.perform(action) {
                    Ok(()) if clears_comparison => self.clear_comparison(),
                    Ok(()) => {}
                    Err(error) => self.set_error(error),
                }
            }
            InputEffect::Presentation(action) => self.apply_presentation(action),
        }
    }

    fn apply_presentation(&mut self, action: PresentationAction) {
        match action {
            PresentationAction::FocusScenarioSearch => {
                self.editing_query = true;
                self.open_panel = OpenPanel::Scenario;
            }
            PresentationAction::ToggleOverlayGroup(group) => self.toggle_group(group),
            PresentationAction::ResetCamera => {
                self.center_x = 0.0;
                self.center_y = 0.0;
                self.pixels_per_meter = 42.0;
            }
            PresentationAction::CloseTopmostOrClearFocus => {
                self.editing_query = false;
                self.open_panel = OpenPanel::None;
            }
            PresentationAction::FocusDifference => {
                self.open_panel = OpenPanel::Inspector;
                self.focused_difference = 0;
            }
            PresentationAction::PreviousDifference => self.move_difference(-1),
            PresentationAction::NextDifference => self.move_difference(1),
            PresentationAction::OpenShortcutHelp => self.open_panel = OpenPanel::ShortcutHelp,
        }
    }

    fn toggle_group(&mut self, group: u8) {
        if group == 4 {
            self.diagnostics_visible = !self.diagnostics_visible;
            return;
        }
        let layers: &[DebugLayerName] = match group {
            1 => &[DebugLayerName::Contacts, DebugLayerName::ContactNormals],
            2 => &[DebugLayerName::ParticleContacts],
            3 => &[DebugLayerName::BroadPhase],
            _ => &[],
        };
        for layer in layers {
            let index = layer_index(*layer);
            self.layer_enabled[index] = !self.layer_enabled[index];
            self.layers.set(*layer, self.layer_enabled[index]);
        }
    }

    fn move_difference(&mut self, direction: i8) {
        let count = self.maybe_comparison.as_ref().map_or(0, |model| {
            DifferenceList::new(model, Camera::default(), BackendAvailability::Both)
                .entries()
                .len()
        });
        if count == 0 {
            self.focused_difference = 0;
        } else if direction < 0 {
            self.focused_difference = (self.focused_difference + count - 1) % count;
        } else {
            self.focused_difference = (self.focused_difference + 1) % count;
        }
    }

    fn handle_settings_input(&mut self) {
        if self.editing_query {
            return;
        }
        let factor = if is_key_pressed(KeyCode::Minus) {
            Some(2.0)
        } else if is_key_pressed(KeyCode::Equal) {
            Some(0.5)
        } else {
            None
        };
        let Some(factor) = factor else {
            return;
        };
        let Some(settings) = self.testbed.selected_settings() else {
            return;
        };
        let candidate = RunSettings::new(
            FloatBits::from_f32(settings.timestep_bits().to_f32() * factor),
            settings.velocity_iterations(),
            settings.position_iterations(),
            settings.particle_iterations(),
        );
        match candidate {
            Ok(settings) => match self.testbed.apply_settings(settings) {
                Ok(()) => self.clear_comparison(),
                Err(error) => self.set_error(error),
            },
            Err(error) => self.set_error(error),
        }
    }

    fn handle_viewport_input(&mut self) {
        let (_, wheel_y) = mouse_wheel();
        if wheel_y != 0.0 {
            self.pixels_per_meter =
                (self.pixels_per_meter * 1.1_f32.powf(wheel_y)).clamp(5.0, 400.0);
        }
        let mouse = mouse_position();
        if is_mouse_button_pressed(MouseButton::Middle) {
            self.maybe_drag_origin = Some(mouse);
        }
        if is_mouse_button_down(MouseButton::Middle) {
            if let Some(previous) = self.maybe_drag_origin.replace(mouse) {
                self.center_x -= (mouse.0 - previous.0) / self.pixels_per_meter;
                self.center_y += (mouse.1 - previous.1) / self.pixels_per_meter;
            }
        } else {
            self.maybe_drag_origin = None;
        }
    }

    fn refresh_comparison(&mut self) {
        let Some(native) = self.testbed.latest_checkpoint() else {
            return;
        };
        let native_identity = (
            native.resolved_sha256().clone(),
            native.checkpoint_id().clone(),
        );
        if self.maybe_compared_identity.as_ref() == Some(&native_identity) {
            return;
        }
        let Some(oracle) = self.maybe_oracle.as_ref() else {
            self.maybe_comparison = None;
            return;
        };
        let policy = Phase4PolicyProfile::parse_toml(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../protocol/tolerances/phase4-v1.toml"
        )));
        let comparison = policy
            .map_err(|error| error.to_string())
            .and_then(|policy| {
                compare_canonical_checkpoints(
                    native,
                    oracle,
                    &policy,
                    ComparisonLimits::phase11_default(),
                )
                .map_err(|error| error.to_string())
            });
        self.maybe_compared_identity = Some(native_identity);
        match comparison {
            Ok(model) => {
                self.maybe_comparison = Some(model);
                self.focused_difference = 0;
            }
            Err(error) => {
                self.maybe_comparison = None;
                self.maybe_error = Some(bound_message(&error));
            }
        }
    }

    fn set_error(&mut self, error: impl std::fmt::Display) {
        self.maybe_error = Some(bound_message(&error.to_string()));
    }

    fn clear_comparison(&mut self) {
        self.maybe_comparison = None;
        self.maybe_compared_identity = None;
    }

    fn draw(&self) {
        clear_background(BACKGROUND);
        let width = bounded_screen_dimension(screen_width());
        let height = bounded_screen_dimension(screen_height());
        let responsive = ResponsiveLayout::for_window(width, height);
        if let Some((heading, body)) = responsive.minimum_window_copy() {
            draw_text(heading, 24.0, 52.0, 28.0, TEXT);
            draw_text(body, 24.0, 82.0, FONT, MUTED);
            return;
        }
        let layout = responsive.shell();
        self.draw_app_bar(layout.region(ShellRegion::AppBar));
        match responsive.panel_behavior() {
            PanelBehavior::BothVisible => {
                self.draw_scenarios(layout.region(ShellRegion::ScenarioRail));
                self.draw_viewport(layout.region(ShellRegion::Viewport));
                self.draw_inspector(layout.region(ShellRegion::Inspector));
            }
            PanelBehavior::InspectorDrawer => {
                self.draw_scenarios(layout.region(ShellRegion::ScenarioRail));
                self.draw_viewport(layout.region(ShellRegion::Viewport));
                if self.open_panel == OpenPanel::Inspector {
                    self.draw_inspector(layout.region(ShellRegion::Inspector));
                }
            }
            PanelBehavior::MutuallyExclusiveDrawers | PanelBehavior::FullWindowSheets => {
                self.draw_viewport(layout.region(ShellRegion::Viewport));
                if self.open_panel == OpenPanel::Scenario {
                    self.draw_scenarios(layout.region(ShellRegion::ScenarioRail));
                } else if self.open_panel == OpenPanel::Inspector {
                    self.draw_inspector(layout.region(ShellRegion::Inspector));
                }
            }
            PanelBehavior::WindowTooSmall => {}
        }
        Self::draw_controls(layout.region(ShellRegion::Controls));
        if self.open_panel == OpenPanel::ShortcutHelp {
            Self::draw_shortcut_help(width, height);
        }
    }

    fn draw_app_bar(&self, region: (u32, u32, u32, u32)) {
        fill_region(region, PANEL);
        draw_text("liquidfun-rs", 16.0, 31.0, 24.0, TEXT);
        let state = self.testbed.session_state();
        let comparison = self.maybe_comparison.as_ref().map(ComparisonModel::state);
        let status = format!(
            "{} {}",
            status_marker(state, comparison),
            status_copy(state, comparison)
        );
        draw_text(&status, 190.0, 30.0, FONT, state_color(comparison));
        draw_text(
            "Private diagnostic UI — pixels and timing are not compatibility authority",
            440.0,
            29.0,
            SMALL_FONT,
            MUTED,
        );
        let _presentation_state = self.shell.state();
    }

    fn draw_scenarios(&self, region: (u32, u32, u32, u32)) {
        fill_region(region, PANEL);
        let (x, y, width, _) = rect(region);
        let search_color = if self.editing_query { ACCENT } else { BORDER };
        draw_rectangle_lines(x + 8.0, y + 8.0, width - 16.0, 36.0, 2.0, search_color);
        let search = if self.query.is_empty() {
            "Search scenarios (/)"
        } else {
            &self.query
        };
        draw_text(search, x + 16.0, y + 32.0, SMALL_FONT, MUTED);
        for (index, row) in self.testbed.visible_rows().iter().take(14).enumerate() {
            let Ok(row_index) = u16::try_from(index) else {
                continue;
            };
            let row_y = y + 44.0 + f32::from(row_index) * ROW_HEIGHT;
            let selected = self
                .testbed
                .current_selection()
                .is_some_and(|selection| selection == row.selection());
            let focused = index == self.focused_row;
            if selected {
                draw_rectangle(x + 4.0, row_y, width - 8.0, ROW_HEIGHT, PANEL_ALT);
            }
            if focused {
                draw_rectangle_lines(x + 4.0, row_y, width - 8.0, ROW_HEIGHT, 2.0, ACCENT);
            }
            draw_text(
                row.display_title(),
                x + 12.0,
                row_y + 19.0,
                SMALL_FONT,
                TEXT,
            );
            let identity = format!(
                "{}@{}  R:{} O:{} V:{}",
                row.selection().catalog_slug(),
                row.selection().scenario_version(),
                yes_no(row.eligibility().rust()),
                yes_no(row.eligibility().oracle()),
                yes_no(row.eligibility().visual())
            );
            draw_text(&identity, x + 12.0, row_y + 37.0, 12.0, MUTED);
        }
    }

    fn draw_viewport(&self, region: (u32, u32, u32, u32)) {
        fill_region(region, BACKGROUND);
        let (x, y, width, height) = rect(region);
        draw_rectangle_lines(x, y, width, height, 1.0, BORDER);
        let Some(checkpoint) = self.testbed.latest_checkpoint() else {
            draw_text(
                "Step the scenario to render a semantic checkpoint",
                x + 20.0,
                y + 32.0,
                FONT,
                MUTED,
            );
            return;
        };
        let maybe_comparison_pair = self
            .maybe_comparison
            .as_ref()
            .zip(self.maybe_oracle.as_ref());
        match (self.comparison_mode, maybe_comparison_pair) {
            (ComparisonMode::SideBySide, Some((comparison, oracle))) => {
                let half_width = (width - 8.0) / 2.0;
                self.draw_checkpoint_viewport(
                    checkpoint,
                    (x, y, half_width, height),
                    "Rust — semantic checkpoint",
                    Some((comparison, ProtocolComparisonBackend::Rust)),
                );
                self.draw_checkpoint_viewport(
                    oracle,
                    (x + half_width + 8.0, y, half_width, height),
                    "Oracle — semantic checkpoint",
                    Some((comparison, ProtocolComparisonBackend::Oracle)),
                );
            }
            (ComparisonMode::Overlay, Some((comparison, oracle))) => {
                self.draw_checkpoint_viewport(
                    checkpoint,
                    (x, y, width, height),
                    "R — Rust solid orange",
                    Some((comparison, ProtocolComparisonBackend::Rust)),
                );
                self.draw_checkpoint_viewport(
                    oracle,
                    (x, y, width, height),
                    "O — Oracle dashed purple",
                    Some((comparison, ProtocolComparisonBackend::Oracle)),
                );
                draw_text(
                    "Shared camera; overlay pixels are diagnostic only",
                    x + 12.0,
                    y + height - 12.0,
                    12.0,
                    MUTED,
                );
            }
            (ComparisonMode::SingleBackend, _)
            | (ComparisonMode::SideBySide | ComparisonMode::Overlay, None) => {
                self.draw_checkpoint_viewport(
                    checkpoint,
                    (x, y, width, height),
                    "Rust — semantic checkpoint",
                    None,
                );
            }
        }
    }

    fn draw_checkpoint_viewport(
        &self,
        checkpoint: &CanonicalCheckpoint,
        bounds: (f32, f32, f32, f32),
        label: &str,
        maybe_comparison: Option<(&ComparisonModel, ProtocolComparisonBackend)>,
    ) {
        let (x, y, width, height) = bounds;
        let Some(viewport) = ProtocolViewport::new(
            x,
            y,
            width,
            height,
            self.center_x,
            self.center_y,
            self.pixels_per_meter,
        ) else {
            return;
        };
        match project_checkpoint(checkpoint, viewport, self.layers) {
            Ok(frame) => {
                if let Some((comparison, backend)) = maybe_comparison {
                    draw_protocol_comparison_frame(
                        &frame,
                        comparison,
                        backend,
                        self.focused_difference_entry(),
                    );
                } else {
                    draw_protocol_frame(&frame);
                }
                draw_text(label, x + 12.0, y + 20.0, 12.0, MUTED);
            }
            Err(_) => {
                draw_text(
                    "Semantic viewport rejected invalid geometry",
                    x + 20.0,
                    y + 32.0,
                    FONT,
                    ERROR,
                );
            }
        }
    }

    fn focused_difference_entry(&self) -> Option<&liquidfun_differential::ComparisonEntry> {
        let comparison = self.maybe_comparison.as_ref()?;
        let differences =
            DifferenceList::new(comparison, Camera::default(), BackendAvailability::Both);
        differences.entries().get(self.focused_difference).copied()
    }

    #[allow(
        clippy::too_many_lines,
        reason = "the inspector intentionally presents one bounded read-only semantic snapshot"
    )]
    fn draw_inspector(&self, region: (u32, u32, u32, u32)) {
        fill_region(region, PANEL);
        let (x, y, _, _) = rect(region);
        draw_text("Inspect", x + 16.0, y + 28.0, 22.0, TEXT);
        let (heading, body) = comparison_copy(self.maybe_comparison.as_ref());
        draw_text(heading, x + 16.0, y + 58.0, FONT, TEXT);
        draw_text(body, x + 16.0, y + 79.0, 12.0, MUTED);
        let mut line_y = y + 112.0;
        if let Some(selected) = self.testbed.selected() {
            let identity = selected.identity();
            line(
                &format!(
                    "Scenario: {}@{}",
                    identity.slug().as_str(),
                    identity.scenario_version().get()
                ),
                x,
                &mut line_y,
                TEXT,
            );
            line(
                &format!(
                    "Resolved: {}",
                    shorten(identity.content_sha256().as_str(), 18)
                ),
                x,
                &mut line_y,
                MUTED,
            );
        }
        line(
            &format!("State: {:?}", self.testbed.session_state()),
            x,
            &mut line_y,
            TEXT,
        );
        line(
            &format!("Logical steps: {}", self.testbed.completed_logical_steps()),
            x,
            &mut line_y,
            TEXT,
        );
        if let Some(checkpoint) = self.testbed.latest_checkpoint() {
            line(
                &format!("Checkpoint: {}", checkpoint.checkpoint_id().as_str()),
                x,
                &mut line_y,
                TEXT,
            );
            line(
                &format!("Primitives: {}", checkpoint.debug_primitives().len()),
                x,
                &mut line_y,
                TEXT,
            );
            line(
                &format!("Observations: {}", checkpoint.observations().len()),
                x,
                &mut line_y,
                TEXT,
            );
            line(
                &format!(
                    "Profiles: {} (names only)",
                    checkpoint.profile_names().len()
                ),
                x,
                &mut line_y,
                MUTED,
            );
            if self.diagnostics_visible {
                let layer_counts = debug_layer_counts(checkpoint);
                line(
                    &format!("Render FPS: {} (non-authority)", get_fps()),
                    x,
                    &mut line_y,
                    MUTED,
                );
                line(
                    &format!(
                        "Layer counts C:{} P:{} B:{}",
                        layer_counts[2], layer_counts[5], layer_counts[6]
                    ),
                    x,
                    &mut line_y,
                    MUTED,
                );
                for profile in checkpoint.profile_names().iter().take(3) {
                    line(
                        &format!("Profile name: {profile:?} (diagnostic)"),
                        x,
                        &mut line_y,
                        MUTED,
                    );
                }
            }
        }
        if let Some(settings) = self.testbed.selected_settings() {
            line(
                &format!("Timestep: {:.8}", settings.timestep_bits().to_f32()),
                x,
                &mut line_y,
                TEXT,
            );
            line(
                &format!(
                    "Iterations: {}/{}/{}",
                    settings.velocity_iterations(),
                    settings.position_iterations(),
                    settings.particle_iterations()
                ),
                x,
                &mut line_y,
                TEXT,
            );
        }
        if let Some(comparison) = self.maybe_comparison.as_ref() {
            let differences =
                DifferenceList::new(comparison, Camera::default(), BackendAvailability::Both);
            let entries = differences.entries();
            line(
                &format!(
                    "Comparison: {:?} ({:?})",
                    comparison.state(),
                    self.comparison_mode
                ),
                x,
                &mut line_y,
                state_color(Some(comparison.state())),
            );
            if entries.is_empty() {
                line("No differences at this checkpoint", x, &mut line_y, MUTED);
            } else {
                let focused = self.focused_difference.min(entries.len() - 1);
                let entry = entries[focused];
                line(
                    &format!("Difference {} of {}", focused + 1, entries.len()),
                    x,
                    &mut line_y,
                    MUTED,
                );
                line(entry.semantic_path(), x, &mut line_y, TEXT);
                line(
                    &format!("Rust: {}", entry.maybe_rust_value().unwrap_or("absent")),
                    x,
                    &mut line_y,
                    MUTED,
                );
                line(
                    &format!("Oracle: {}", entry.maybe_oracle_value().unwrap_or("absent")),
                    x,
                    &mut line_y,
                    MUTED,
                );
                line(
                    &format!("Policy: {:?}", entry.maybe_policy_path()),
                    x,
                    &mut line_y,
                    MUTED,
                );
            }
        }
        if let Some(error) = self.maybe_error.as_deref() {
            line_y += 8.0;
            line("Last bounded error:", x, &mut line_y, ERROR);
            line(error, x, &mut line_y, ERROR);
        }
    }

    fn draw_controls(region: (u32, u32, u32, u32)) {
        fill_region(region, PANEL_ALT);
        let (x, y, _, _) = rect(region);
        draw_text(
            "Space Run/Pause   Right Step   R Restart   C Capture   +/- Timestep",
            x + 16.0,
            y + 24.0,
            SMALL_FONT,
            TEXT,
        );
        draw_text(
            "B Scenarios   I Inspect   / Search   ? Help   1-3 Layers   4 Diagnostics   O View Mode",
            x + 16.0,
            y + 47.0,
            12.0,
            MUTED,
        );
        draw_text(
            "F Focus difference   [ / ] Navigate   Wheel Zoom   Middle-drag Pan",
            x + 620.0,
            y + 47.0,
            12.0,
            MUTED,
        );
    }

    fn draw_shortcut_help(width: u32, height: u32) {
        let panel_width = 520.0;
        let panel_height = 300.0;
        let width = u16::try_from(width).map_or(0.0, f32::from);
        let height = u16::try_from(height).map_or(0.0, f32::from);
        let x = (width - panel_width).max(0.0) / 2.0;
        let y = (height - panel_height).max(0.0) / 2.0;
        draw_rectangle(x, y, panel_width, panel_height, PANEL);
        draw_rectangle_lines(x, y, panel_width, panel_height, 2.0, ACCENT);
        draw_text("Keyboard shortcuts", x + 24.0, y + 38.0, 24.0, TEXT);
        let shortcuts = [
            "Space  Run/Pause    Right  Step once    R  Restart    C  Capture",
            "/  Search scenarios    B  Scenario panel    I  Inspector",
            "1  Contacts    2  Particle contacts    3  Broad phase",
            "4  Profiles/statistics    O  Overlay/Side by side",
            "F  Focus difference    [ / ]  Previous/Next difference",
            "Home  Reset camera    Escape  Close this help",
        ];
        for (index, shortcut) in shortcuts.iter().enumerate() {
            let row = u16::try_from(index).map_or(0.0, f32::from);
            draw_text(shortcut, x + 24.0, y + 78.0 + row * 32.0, SMALL_FONT, TEXT);
        }
        draw_text(
            "Presentation shortcuts never submit simulation commands.",
            x + 24.0,
            y + panel_height - 22.0,
            12.0,
            MUTED,
        );
    }
}

fn debug_layer_counts(checkpoint: &CanonicalCheckpoint) -> [usize; 9] {
    let mut counts = [0; 9];
    for record in checkpoint.debug_primitives() {
        counts[layer_index(record.key().layer())] += 1;
    }
    counts
}

fn load_oracle_checkpoint(path: PathBuf) -> Result<CanonicalCheckpoint, String> {
    let limits = HarnessLimits::phase2_default_v1();
    let path_metadata = std::fs::symlink_metadata(&path)
        .map_err(|_| "oracle checkpoint metadata unavailable".to_owned())?;
    if path_metadata.file_type().is_symlink() {
        return Err("oracle checkpoint symlinks are not accepted".to_owned());
    }
    let file = File::open(path).map_err(|_| "oracle checkpoint could not be opened".to_owned())?;
    let metadata = file
        .metadata()
        .map_err(|_| "oracle checkpoint metadata unavailable".to_owned())?;
    if !metadata.is_file() {
        return Err("oracle checkpoint must be a regular file".to_owned());
    }
    let length =
        usize::try_from(metadata.len()).map_err(|_| "oracle checkpoint is oversized".to_owned())?;
    if length > limits.output_record_bytes() {
        return Err("oracle checkpoint is oversized".to_owned());
    }
    let maximum = u64::try_from(limits.output_record_bytes())
        .map_err(|_| "oracle checkpoint limit is invalid".to_owned())?;
    let mut bytes = Vec::new();
    file.take(maximum.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| "oracle checkpoint could not be read".to_owned())?;
    if bytes.len() > limits.output_record_bytes() {
        return Err("oracle checkpoint is oversized".to_owned());
    }
    decode_canonical_checkpoint_jsonl(&bytes, &limits)
        .map_err(|_| "oracle checkpoint is invalid".to_owned())
}

fn parse_args() -> Result<Option<PathBuf>, String> {
    let mut arguments = env::args().skip(1);
    let Some(first) = arguments.next() else {
        return Ok(None);
    };
    if first != "--oracle-checkpoint" {
        return Err("expected --oracle-checkpoint PATH".to_owned());
    }
    let path = arguments
        .next()
        .ok_or_else(|| "missing oracle checkpoint path".to_owned())?;
    if arguments.next().is_some() {
        return Err("unexpected interactive testbed argument".to_owned());
    }
    Ok(Some(PathBuf::from(path)))
}

fn layer_index(layer: DebugLayerName) -> usize {
    match layer {
        DebugLayerName::Shapes => 0,
        DebugLayerName::Joints => 1,
        DebugLayerName::Contacts => 2,
        DebugLayerName::ContactNormals => 3,
        DebugLayerName::Particles => 4,
        DebugLayerName::ParticleContacts => 5,
        DebugLayerName::BroadPhase => 6,
        DebugLayerName::CentersOfMass => 7,
        DebugLayerName::Labels => 8,
    }
}

fn fill_region(region: (u32, u32, u32, u32), color: Color) {
    let (x, y, width, height) = rect(region);
    draw_rectangle(x, y, width, height, color);
}

fn rect(region: (u32, u32, u32, u32)) -> (f32, f32, f32, f32) {
    (
        u16::try_from(region.0).map_or(0.0, f32::from),
        u16::try_from(region.1).map_or(0.0, f32::from),
        u16::try_from(region.2).map_or(0.0, f32::from),
        u16::try_from(region.3).map_or(0.0, f32::from),
    )
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "Macroquad reports finite pixel dimensions as f32; the UI contract caps them first"
)]
fn bounded_screen_dimension(value: f32) -> u32 {
    value.clamp(1.0, 16_384.0).round() as u32
}

fn line(text: &str, x: f32, y: &mut f32, color: Color) {
    draw_text(text, x + 16.0, *y, SMALL_FONT, color);
    *y += 21.0;
}

fn shorten(value: &str, maximum: usize) -> &str {
    value.get(..maximum).unwrap_or(value)
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn state_color(maybe_comparison: Option<ComparisonState>) -> Color {
    match maybe_comparison {
        Some(
            ComparisonState::PhysicsMismatch
            | ComparisonState::RustOnly
            | ComparisonState::OracleOnly,
        ) => ERROR,
        Some(ComparisonState::WithinPolicy) => Color::new(0.824, 0.600, 0.133, 1.0),
        Some(ComparisonState::ExactMatch) => Color::new(0.247, 0.725, 0.314, 1.0),
        None => MUTED,
    }
}

fn comparison_copy(maybe_comparison: Option<&ComparisonModel>) -> (&'static str, &'static str) {
    let Some(comparison) = maybe_comparison else {
        let copy = operational_copy(InspectorState::OracleUnavailable);
        return (copy.heading(), copy.body());
    };
    match comparison.state() {
        ComparisonState::ExactMatch => {
            let copy = operational_copy(InspectorState::ExactMatch);
            (copy.heading(), copy.body())
        }
        ComparisonState::WithinPolicy => (
            "Within reviewed policy",
            "Rust and oracle differ only within the named numeric policies.",
        ),
        ComparisonState::PhysicsMismatch => (
            "Physics mismatch",
            "Rust and oracle disagree outside the reviewed compatibility policy.",
        ),
        ComparisonState::RustOnly => (
            "Rust-only observations",
            "Some semantic observations are absent from the oracle checkpoint.",
        ),
        ComparisonState::OracleOnly => (
            "Oracle-only observations",
            "Some semantic observations are absent from the Rust checkpoint.",
        ),
    }
}

fn bound_message(message: &str) -> String {
    message.chars().take(160).collect()
}

fn bounded_error(error: impl std::fmt::Display) -> String {
    bound_message(&error.to_string())
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut maybe_app = parse_args().and_then(DesktopApp::new);
    loop {
        match maybe_app.as_mut() {
            Ok(app) => {
                app.update();
                app.draw();
            }
            Err(error) => {
                clear_background(BACKGROUND);
                draw_text(
                    "Interactive testbed could not start",
                    24.0,
                    48.0,
                    28.0,
                    ERROR,
                );
                draw_text(error, 24.0, 80.0, FONT, TEXT);
            }
        }
        next_frame().await;
    }
}
