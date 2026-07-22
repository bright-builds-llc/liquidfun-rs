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
    ActionSchedule, CanonicalCheckpoint, CheckpointId, DebugLayerName, HarnessLimits,
    Phase4PolicyProfile, RigidWorldAction, Sha256Hex, decode_canonical_checkpoint_jsonl,
};
use liquidfun_testbed::app::{AppShell, ShellRegion, status_copy, status_marker};
use liquidfun_testbed::controller_adapter::{
    ControllerAction, PARTICLE_PAUSE_ACTION_LABEL, SESSION_PAUSED_LABEL,
};
use liquidfun_testbed::input::{
    InputContext, InputEffect, KeyboardKey, PresentationAction, ScenarioShortcut, resolve_key,
};
use liquidfun_testbed::interactive::InteractiveTestbed;
use liquidfun_testbed::ui::differences::{BackendAvailability, ComparisonMode, DifferenceList};
use liquidfun_testbed::ui::inspector::{InspectorState, operational_copy};
use liquidfun_testbed::ui::layout::{
    CompactWindowNotice, FocusId, FocusReturn, PanelBehavior, ResponsiveLayout,
};
use liquidfun_testbed::ui::protocol_viewport::{
    ProtocolComparisonBackend, ProtocolLayerVisibility, ProtocolScreenPoint, ProtocolViewport,
    draw_protocol_comparison_frame, draw_protocol_frame, hit_test_frame, project_checkpoint,
};
use liquidfun_testbed::ui::settings::{SettingsEditor, SettingsField};
use liquidfun_testbed::ui::viewport::Camera;
use liquidfun_testbed::ui::{AboutPanel, ProvenanceInput, build_about_panel};
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
const SMALL_FONT_SIZE: u16 = 14;
const CONTROL_TARGET: f32 = 44.0;
const SETTINGS_FIELDS: [SettingsField; 4] = [
    SettingsField::Timestep,
    SettingsField::VelocityIterations,
    SettingsField::PositionIterations,
    SettingsField::ParticleIterations,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpenPanel {
    None,
    Scenario,
    Inspector,
    Settings,
    About,
    ShortcutHelp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControlFocus {
    Scenario,
    Inspector,
    RunPause,
    Step,
    Restart,
    Capture,
    Settings,
    Overlay,
    PreviousDifference,
    NextDifference,
    About,
}

const FOCUS_ORDER: [ControlFocus; 11] = [
    ControlFocus::Scenario,
    ControlFocus::Inspector,
    ControlFocus::RunPause,
    ControlFocus::Step,
    ControlFocus::Restart,
    ControlFocus::Capture,
    ControlFocus::Settings,
    ControlFocus::Overlay,
    ControlFocus::PreviousDifference,
    ControlFocus::NextDifference,
    ControlFocus::About,
];

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
    focus_index: usize,
    focus_return: FocusReturn,
    compact_notice: CompactWindowNotice,
    maybe_compact_notice: Option<&'static str>,
    settings: SettingsEditor,
    maybe_editing_setting: Option<SettingsField>,
    maybe_last_click: Option<(f64, f32, f32)>,
    maybe_selected_primitive: Option<String>,
    maybe_last_scenario_action_label: Option<&'static str>,
    diagnostics_visible: bool,
    comparison_mode: ComparisonMode,
    focused_difference: usize,
    focused_about_link: usize,
    maybe_link_status: Option<String>,
    keyboard_input_consumed_this_frame: bool,
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
        let settings = SettingsEditor::new(
            testbed
                .selected_settings()
                .ok_or_else(|| "selected scenario has no settings".to_owned())?,
        );

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
            focus_index: 0,
            focus_return: FocusReturn::default(),
            compact_notice: CompactWindowNotice::default(),
            maybe_compact_notice: None,
            settings,
            maybe_editing_setting: None,
            maybe_last_click: None,
            maybe_selected_primitive: None,
            maybe_last_scenario_action_label: None,
            diagnostics_visible: true,
            comparison_mode: ComparisonMode::Overlay,
            focused_difference: 0,
            focused_about_link: 0,
            maybe_link_status: None,
            keyboard_input_consumed_this_frame: false,
        })
    }

    fn update(&mut self) {
        self.keyboard_input_consumed_this_frame = false;
        self.handle_search_input();
        self.handle_focus_input();
        self.handle_panel_input();
        self.handle_about_input();
        self.handle_shortcut_help_input();
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
        let layout = ResponsiveLayout::for_window(
            bounded_screen_dimension(screen_width()),
            bounded_screen_dimension(screen_height()),
        );
        if let Some(notice) = self.compact_notice.take(layout) {
            self.maybe_compact_notice = Some(notice);
        }
    }

    fn handle_focus_input(&mut self) {
        if self.keyboard_input_consumed_this_frame
            || self.editing_query
            || self.maybe_editing_setting.is_some()
        {
            return;
        }
        let layout = ResponsiveLayout::for_window(
            bounded_screen_dimension(screen_width()),
            bounded_screen_dimension(screen_height()),
        );
        if layout.panel_behavior() == PanelBehavior::WindowTooSmall {
            return;
        }
        if self.modal_input_active(layout) {
            self.handle_modal_focus_input();
            return;
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            let mouse = mouse_position();
            if let Some((index, _)) = FOCUS_ORDER
                .iter()
                .enumerate()
                .find(|(_, focus)| point_in_rect(mouse, control_bounds(**focus, layout)))
            {
                self.focus_index = index;
                self.activate_focus();
                return;
            }
        }
        if is_key_pressed(KeyCode::Tab) {
            let backwards = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
            self.focus_index = if backwards {
                (self.focus_index + FOCUS_ORDER.len() - 1) % FOCUS_ORDER.len()
            } else {
                (self.focus_index + 1) % FOCUS_ORDER.len()
            };
        }
        if is_key_pressed(KeyCode::Enter) {
            self.activate_focus();
        }
    }

    fn handle_modal_focus_input(&mut self) {
        if !is_key_pressed(KeyCode::Tab) && !is_key_pressed(KeyCode::Enter) {
            return;
        }
        self.keyboard_input_consumed_this_frame = true;
        match self.open_panel {
            OpenPanel::Scenario => {
                self.focus_return.move_to(FocusId::ScenarioSearch);
                self.editing_query = true;
            }
            OpenPanel::Inspector => {
                self.focus_return.move_to(FocusId::InspectorDifference);
                if is_key_pressed(KeyCode::Tab) {
                    let backwards =
                        is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
                    self.move_difference(if backwards { -1 } else { 1 });
                }
            }
            OpenPanel::Settings => {
                let backwards = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
                if is_key_pressed(KeyCode::Enter)
                    && self.focus_return.current() == Some(FocusId::SettingsApply)
                {
                    self.apply_settings_and_restart();
                } else if backwards && self.focus_return.current() == Some(FocusId::SettingsHeading)
                {
                    self.maybe_editing_setting = None;
                    self.focus_return.move_to(FocusId::SettingsApply);
                } else {
                    self.focus_return.move_to(FocusId::SettingsField);
                    self.maybe_editing_setting = Some(if backwards {
                        SettingsField::ParticleIterations
                    } else {
                        SettingsField::Timestep
                    });
                }
            }
            OpenPanel::About => {
                let link_count = self.about_panel().links().len();
                if link_count == 0 {
                    return;
                }
                if is_key_pressed(KeyCode::Tab) {
                    let backwards =
                        is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
                    self.focused_about_link =
                        if self.focus_return.current() == Some(FocusId::AboutHeading) {
                            if backwards { link_count - 1 } else { 0 }
                        } else if backwards {
                            (self.focused_about_link + link_count - 1) % link_count
                        } else {
                            (self.focused_about_link + 1) % link_count
                        };
                    self.focus_return.move_to(FocusId::AboutLink);
                } else if self.focus_return.current() == Some(FocusId::AboutLink) {
                    self.copy_focused_about_link();
                } else {
                    self.focused_about_link = 0;
                    self.focus_return.move_to(FocusId::AboutLink);
                }
            }
            OpenPanel::ShortcutHelp => {
                self.focus_return.move_to(FocusId::ShortcutHelp);
                if is_key_pressed(KeyCode::Enter) {
                    self.close_modal();
                }
            }
            OpenPanel::None => {}
        }
    }

    fn modal_input_active(&self, layout: ResponsiveLayout) -> bool {
        match self.open_panel {
            OpenPanel::Settings | OpenPanel::About | OpenPanel::ShortcutHelp => true,
            OpenPanel::Scenario => matches!(
                layout.panel_behavior(),
                PanelBehavior::MutuallyExclusiveDrawers | PanelBehavior::FullWindowSheets
            ),
            OpenPanel::Inspector => matches!(
                layout.panel_behavior(),
                PanelBehavior::InspectorDrawer
                    | PanelBehavior::MutuallyExclusiveDrawers
                    | PanelBehavior::FullWindowSheets
            ),
            OpenPanel::None => false,
        }
    }

    fn activate_focus(&mut self) {
        match FOCUS_ORDER[self.focus_index] {
            ControlFocus::Scenario => self.open_modal(
                OpenPanel::Scenario,
                FocusId::ScenarioButton,
                FocusId::ScenarioHeading,
            ),
            ControlFocus::Inspector => self.open_modal(
                OpenPanel::Inspector,
                FocusId::InspectorButton,
                FocusId::InspectorHeading,
            ),
            ControlFocus::RunPause => {
                let action = if self.testbed.session_state()
                    == liquidfun_differential::SessionState::Running
                {
                    ControllerAction::Pause
                } else {
                    ControllerAction::Run
                };
                self.perform_controller(action);
            }
            ControlFocus::Step => self.perform_controller(ControllerAction::StepOnce),
            ControlFocus::Restart => self.perform_controller(ControllerAction::Restart),
            ControlFocus::Capture => {
                if let Some(checkpoint_id) = self.testbed.reachable_checkpoint_id().cloned() {
                    self.perform_controller(ControllerAction::CaptureCheckpoint(checkpoint_id));
                }
            }
            ControlFocus::Settings => self.open_settings(),
            ControlFocus::Overlay => {
                if self.maybe_comparison.is_some() {
                    self.comparison_mode = match self.comparison_mode {
                        ComparisonMode::Overlay => ComparisonMode::SideBySide,
                        ComparisonMode::SideBySide | ComparisonMode::SingleBackend => {
                            ComparisonMode::Overlay
                        }
                    };
                }
            }
            ControlFocus::PreviousDifference => self.move_difference(-1),
            ControlFocus::NextDifference => self.move_difference(1),
            ControlFocus::About => self.open_about(),
        }
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
        let submit = is_key_pressed(KeyCode::Enter);
        if submit || is_key_pressed(KeyCode::Escape) {
            self.editing_query = false;
            self.keyboard_input_consumed_this_frame = true;
        }
        if let Err(error) = self.testbed.set_query(&self.query) {
            self.set_error(error);
        }
        self.focused_row = self
            .focused_row
            .min(self.testbed.visible_rows().len().saturating_sub(1));
        if submit && !self.testbed.visible_rows().is_empty() {
            self.select_focused();
        }
    }

    fn handle_panel_input(&mut self) {
        if self.editing_query || self.maybe_editing_setting.is_some() {
            return;
        }
        let layout = ResponsiveLayout::for_window(
            bounded_screen_dimension(screen_width()),
            bounded_screen_dimension(screen_height()),
        );
        if self.modal_input_active(layout) {
            if is_key_pressed(KeyCode::Escape) {
                self.close_modal();
            }
            return;
        }
        if is_key_pressed(KeyCode::B) {
            if self.open_panel == OpenPanel::Scenario {
                self.close_modal();
            } else {
                self.open_modal(
                    OpenPanel::Scenario,
                    FocusId::ScenarioButton,
                    FocusId::ScenarioHeading,
                );
            }
        }
        if is_key_pressed(KeyCode::I) {
            if self.open_panel == OpenPanel::Inspector {
                self.close_modal();
            } else {
                self.open_modal(
                    OpenPanel::Inspector,
                    FocusId::InspectorButton,
                    FocusId::InspectorHeading,
                );
            }
        }
        if is_key_pressed(KeyCode::S) {
            self.open_settings();
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
            self.close_modal();
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            let width = screen_width();
            let (mouse_x, mouse_y) = mouse_position();
            if mouse_y <= 48.0 && mouse_x >= width - 196.0 {
                self.open_about();
                self.focus_index = focus_index(ControlFocus::About);
            }
            let layout = ResponsiveLayout::for_window(
                bounded_screen_dimension(screen_width()),
                bounded_screen_dimension(screen_height()),
            );
            if layout.panel_behavior() == PanelBehavior::WindowTooSmall {
                if point_in_rect((mouse_x, mouse_y), minimum_close_bounds()) {
                    macroquad::miniquad::window::request_quit();
                } else if point_in_rect((mouse_x, mouse_y), minimum_about_bounds()) {
                    self.open_about();
                }
            }
        }
    }

    fn open_modal(&mut self, panel: OpenPanel, invoker: FocusId, first: FocusId) {
        self.open_panel = panel;
        self.focus_return.open(invoker, first);
        self.focus_index = focus_index(control_for_focus(first));
    }

    fn close_modal(&mut self) {
        self.open_panel = OpenPanel::None;
        self.maybe_editing_setting = None;
        if let Some(returned_focus) = self.focus_return.close() {
            self.focus_index = focus_index(control_for_focus(returned_focus));
        }
    }

    fn open_settings(&mut self) {
        if let Some(settings) = self.testbed.selected_settings() {
            self.settings = SettingsEditor::new(settings);
        }
        self.open_modal(
            OpenPanel::Settings,
            FocusId::SettingsButton,
            FocusId::SettingsHeading,
        );
    }

    fn open_about(&mut self) {
        self.focused_about_link = 0;
        self.maybe_link_status = None;
        self.open_modal(
            OpenPanel::About,
            FocusId::AboutButton,
            FocusId::AboutHeading,
        );
    }

    fn about_panel(&self) -> AboutPanel {
        let target = format!("{}-{}", env::consts::ARCH, env::consts::OS);
        let maybe_run_identity = self
            .testbed
            .selected()
            .map(|selected| selected.identity().content_sha256().as_str());
        let maybe_oracle_identity = self
            .maybe_oracle
            .as_ref()
            .map(|checkpoint| checkpoint.resolved_sha256().as_str());
        build_about_panel(ProvenanceInput {
            version: Some(env!("CARGO_PKG_VERSION")),
            commit: option_env!("LIQUIDFUN_BUILD_COMMIT"),
            profile: option_env!("PROFILE"),
            target: Some(&target),
            rust_toolchain: Some("Rust 1.97.0"),
            protocol_version: Some("phase11-v1"),
            adapter_version: Some(env!("CARGO_PKG_VERSION")),
            run_identity: maybe_run_identity,
            oracle_revision: maybe_oracle_identity,
            oracle_compiler: None,
            oracle_preset: None,
            evidence_tier: self
                .maybe_oracle
                .as_ref()
                .map(|_| "diagnostic comparison; not compatibility authority"),
        })
    }

    fn copy_focused_about_link(&mut self) {
        let about = self.about_panel();
        let Some(link) = about.links().get(self.focused_about_link) else {
            return;
        };
        macroquad::miniquad::window::clipboard_set(link.url());
        self.maybe_link_status = Some(format!("Copied {} URL", link.label()));
    }

    fn handle_about_input(&mut self) {
        if self.open_panel != OpenPanel::About || !is_mouse_button_pressed(MouseButton::Left) {
            return;
        }
        let mouse = mouse_position();
        let link_count = self.about_panel().links().len();
        let maybe_index =
            (0..link_count).find(|index| point_in_rect(mouse, about_link_bounds(*index)));
        if let Some(index) = maybe_index {
            self.focused_about_link = index;
            self.focus_return.move_to(FocusId::AboutLink);
            self.copy_focused_about_link();
        }
    }

    fn handle_shortcut_help_input(&mut self) {
        if self.open_panel == OpenPanel::ShortcutHelp
            && is_mouse_button_pressed(MouseButton::Left)
            && point_in_rect(mouse_position(), shortcut_close_bounds())
        {
            self.close_modal();
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
        if self.modal_input_active(responsive) && self.open_panel != OpenPanel::Scenario {
            return;
        }
        if self.keyboard_input_consumed_this_frame
            || !scenario_visible
            || self.editing_query
            || self.testbed.visible_rows().is_empty()
        {
            return;
        }
        if is_key_pressed(KeyCode::Down) {
            self.focused_row = (self.focused_row + 1) % self.testbed.visible_rows().len();
        }
        if is_key_pressed(KeyCode::Up) {
            self.focused_row = (self.focused_row + self.testbed.visible_rows().len() - 1)
                % self.testbed.visible_rows().len();
        }
        if is_key_pressed(KeyCode::Enter) && FOCUS_ORDER[self.focus_index] == ControlFocus::Scenario
        {
            self.select_focused();
        }

        let scenario_region = responsive.shell().region(ShellRegion::ScenarioRail);
        let (region_x, region_y, region_width, _) = rect(scenario_region);
        let (mouse_x, mouse_y) = mouse_position();
        if is_mouse_button_pressed(MouseButton::Left)
            && point_in_rect((mouse_x, mouse_y), scenario_search_bounds(scenario_region))
        {
            self.editing_query = true;
            self.focus_return.move_to(FocusId::ScenarioSearch);
            return;
        }
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
                let row_y = region_y + 56.0 + f32::from(row_index) * ROW_HEIGHT;
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
            Ok(()) => {
                self.clear_comparison();
                if let Some(settings) = self.testbed.selected_settings() {
                    self.settings = SettingsEditor::new(settings);
                }
            }
            Err(error) => self.set_error(error),
        }
    }

    fn handle_controller_input(&mut self) {
        if self.editing_query || self.maybe_editing_setting.is_some() {
            return;
        }
        let layout = ResponsiveLayout::for_window(
            bounded_screen_dimension(screen_width()),
            bounded_screen_dimension(screen_height()),
        );
        if self.modal_input_active(layout) {
            return;
        }
        let scenario_shortcuts = self.scenario_shortcuts();
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
                    (KeyCode::A, KeyboardKey::Scenario('a')),
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
                scenario_shortcuts: &scenario_shortcuts,
            },
        );
        let Some(effect) = effect else {
            return;
        };
        match effect {
            InputEffect::Controller(action) => self.perform_controller(action),
            InputEffect::Presentation(action) => self.apply_presentation(action),
        }
    }

    fn scenario_shortcuts(&self) -> Vec<ScenarioShortcut> {
        let next_ordinal = self.testbed.completed_logical_steps().saturating_add(1);
        self.testbed
            .selected()
            .and_then(|resolved| {
                resolved.actions().iter().find(|action| {
                    action.schedule()
                        == ActionSchedule::LogicalStep {
                            ordinal: next_ordinal,
                        }
                })
            })
            .and_then(|action| {
                let label = if matches!(
                    action.action(),
                    RigidWorldAction::Particle {
                        action: liquidfun_test_protocol::Phase9ParticleAction::SetPaused { .. }
                    }
                ) {
                    PARTICLE_PAUSE_ACTION_LABEL
                } else {
                    "Apply next typed scenario action"
                };
                ScenarioShortcut::new('a', action.action_id().clone(), label)
            })
            .into_iter()
            .collect()
    }

    fn perform_controller(&mut self, action: ControllerAction) {
        let clears_comparison = matches!(
            &action,
            ControllerAction::Select(_)
                | ControllerAction::Restart
                | ControllerAction::ApplySettingsAndRestart { .. }
        );
        let maybe_action_label = match &action {
            ControllerAction::ApplyScenarioAction(action_id) => self
                .scenario_shortcuts()
                .into_iter()
                .find(|shortcut| shortcut.action_id() == action_id)
                .map(|shortcut| {
                    if shortcut.label() == PARTICLE_PAUSE_ACTION_LABEL {
                        PARTICLE_PAUSE_ACTION_LABEL
                    } else {
                        "Scenario action applied"
                    }
                }),
            _ => None,
        };
        match self.testbed.perform(action) {
            Ok(()) => {
                if clears_comparison {
                    self.clear_comparison();
                }
                self.maybe_last_scenario_action_label = maybe_action_label;
            }
            Err(error) => self.set_error(error),
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
            PresentationAction::OpenShortcutHelp => self.open_modal(
                OpenPanel::ShortcutHelp,
                FocusId::AboutButton,
                FocusId::ShortcutHelp,
            ),
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
        if self.open_panel != OpenPanel::Settings {
            return;
        }
        if self.keyboard_input_consumed_this_frame {
            return;
        }
        if is_key_pressed(KeyCode::Escape) {
            self.close_modal();
            return;
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            let mouse = mouse_position();
            let maybe_field = SETTINGS_FIELDS
                .into_iter()
                .find(|field| point_in_rect(mouse, settings_field_bounds(*field)));
            if let Some(field) = maybe_field {
                if let Some(previous) = self.maybe_editing_setting.replace(field) {
                    self.settings.commit(previous);
                }
                self.focus_return.move_to(FocusId::SettingsField);
            } else {
                if let Some(previous) = self.maybe_editing_setting.take() {
                    self.settings.commit(previous);
                }
                if point_in_rect(mouse, settings_apply_bounds()) {
                    self.focus_return.move_to(FocusId::SettingsApply);
                    self.apply_settings_and_restart();
                }
            }
        }

        let Some(field) = self.maybe_editing_setting else {
            return;
        };
        let mut text = self.settings.text(field).to_owned();
        while let Some(character) = get_char_pressed() {
            let valid = character.is_ascii_digit()
                || (field == SettingsField::Timestep
                    && matches!(character, '.' | '-' | '+' | 'e' | 'E'));
            if valid && text.len() < 32 {
                text.push(character);
            }
        }
        if is_key_pressed(KeyCode::Backspace) {
            text.pop();
        }
        self.settings.edit(field, text);
        if is_key_pressed(KeyCode::Enter) {
            self.settings.commit(field);
        } else if is_key_pressed(KeyCode::Tab) {
            self.settings.commit(field);
            let index = setting_index(field);
            let backwards = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
            let moves_to_apply =
                (backwards && index == 0) || (!backwards && index + 1 == SETTINGS_FIELDS.len());
            if moves_to_apply {
                self.maybe_editing_setting = None;
                self.focus_return.move_to(FocusId::SettingsApply);
            } else {
                let next = if backwards { index - 1 } else { index + 1 };
                self.maybe_editing_setting = Some(SETTINGS_FIELDS[next]);
                self.focus_return.move_to(FocusId::SettingsField);
            }
        }
    }

    fn apply_settings_and_restart(&mut self) {
        if !self.settings.apply_enabled() {
            return;
        }
        let accepted = self.settings.accepted();
        match self.testbed.apply_settings(accepted) {
            Ok(()) => {
                self.clear_comparison();
                self.settings = SettingsEditor::new(accepted);
                self.close_modal();
            }
            Err(error) => self.set_error(error),
        }
    }

    fn handle_viewport_input(&mut self) {
        let layout = ResponsiveLayout::for_window(
            bounded_screen_dimension(screen_width()),
            bounded_screen_dimension(screen_height()),
        );
        if layout.panel_behavior() == PanelBehavior::WindowTooSmall {
            return;
        }
        if self.modal_input_active(layout) {
            self.maybe_drag_origin = None;
            return;
        }
        let viewport = rect(layout.shell().region(ShellRegion::Viewport));
        let mouse = mouse_position();
        if !point_in_rect(mouse, viewport) {
            self.maybe_drag_origin = None;
            return;
        }
        let (_, wheel_y) = mouse_wheel();
        if wheel_y != 0.0 {
            let old_scale = self.pixels_per_meter;
            let new_scale = (old_scale * 1.1_f32.powf(wheel_y)).clamp(5.0, 400.0);
            let screen_center_x = viewport.0 + viewport.2 * 0.5;
            let screen_center_y = viewport.1 + viewport.3 * 0.5;
            let offset_x = mouse.0 - screen_center_x;
            let offset_y = mouse.1 - screen_center_y;
            let world_x = self.center_x + offset_x / old_scale;
            let world_y = self.center_y - offset_y / old_scale;
            self.center_x = world_x - offset_x / new_scale;
            self.center_y = world_y + offset_y / new_scale;
            self.pixels_per_meter = new_scale;
        }
        let shift = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
        let pan_pressed = is_mouse_button_pressed(MouseButton::Middle)
            || (shift && is_mouse_button_pressed(MouseButton::Left));
        let pan_down = is_mouse_button_down(MouseButton::Middle)
            || (shift && is_mouse_button_down(MouseButton::Left));
        if pan_pressed {
            self.maybe_drag_origin = Some(mouse);
        }
        if pan_down {
            if let Some(previous) = self.maybe_drag_origin.replace(mouse) {
                self.center_x -= (mouse.0 - previous.0) / self.pixels_per_meter;
                self.center_y += (mouse.1 - previous.1) / self.pixels_per_meter;
            }
        } else {
            self.maybe_drag_origin = None;
        }

        if is_mouse_button_pressed(MouseButton::Left) && !shift {
            let now = get_time();
            let double_click = self.maybe_last_click.is_some_and(|(last, x, y)| {
                now - last <= 0.35 && (mouse.0 - x).abs() <= 6.0 && (mouse.1 - y).abs() <= 6.0
            });
            self.maybe_last_click = Some((now, mouse.0, mouse.1));
            if double_click {
                self.center_x = 0.0;
                self.center_y = 0.0;
                self.pixels_per_meter = 42.0;
                self.maybe_selected_primitive = None;
            } else {
                let maybe_key = self.testbed.latest_checkpoint().and_then(|checkpoint| {
                    let projected_viewport = ProtocolViewport::new(
                        viewport.0,
                        viewport.1,
                        viewport.2,
                        viewport.3,
                        self.center_x,
                        self.center_y,
                        self.pixels_per_meter,
                    )?;
                    let frame =
                        project_checkpoint(checkpoint, projected_viewport, self.layers).ok()?;
                    hit_test_frame(
                        &frame,
                        ProtocolScreenPoint {
                            x: mouse.0,
                            y: mouse.1,
                        },
                        6.0,
                    )
                    .cloned()
                });
                self.maybe_selected_primitive = maybe_key.map(|key| format!("{key:?}"));
            }
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
            if self.open_panel == OpenPanel::About {
                self.draw_about_panel();
                return;
            }
            draw_text(heading, 24.0, 52.0, 24.0, TEXT);
            draw_text(body, 24.0, 82.0, FONT, MUTED);
            draw_accessible_button(minimum_close_bounds(), "Close", false, true);
            draw_accessible_button(minimum_about_bounds(), "About & provenance", false, true);
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
        self.draw_controls(responsive);
        match self.open_panel {
            OpenPanel::Settings => self.draw_settings_panel(),
            OpenPanel::About => self.draw_about_panel(),
            OpenPanel::ShortcutHelp => self.draw_shortcut_help(width, height),
            OpenPanel::None | OpenPanel::Scenario | OpenPanel::Inspector => {}
        }
        if let Some(notice) = self.maybe_compact_notice {
            let notice_width = measure_text(notice, None, SMALL_FONT_SIZE, 1.0).width;
            draw_rectangle(12.0, 54.0, notice_width + 24.0, 36.0, PANEL_ALT);
            draw_text(notice, 24.0, 78.0, SMALL_FONT, TEXT);
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
        let about = (screen_width() - 196.0, 2.0, 194.0, CONTROL_TARGET);
        draw_accessible_button(
            about,
            "About & provenance",
            FOCUS_ORDER[self.focus_index] == ControlFocus::About
                && !focus_is_modal_heading(self.focus_return.current()),
            self.open_panel == OpenPanel::About,
        );
        let _presentation_state = self.shell.state();
    }

    fn draw_scenarios(&self, region: (u32, u32, u32, u32)) {
        fill_region(region, PANEL);
        let (x, y, width, _) = rect(region);
        let search_color = if self.editing_query
            || matches!(
                self.focus_return.current(),
                Some(FocusId::ScenarioHeading | FocusId::ScenarioSearch)
            ) {
            ACCENT
        } else {
            BORDER
        };
        let search_bounds = scenario_search_bounds(region);
        draw_rectangle_lines(
            search_bounds.0,
            search_bounds.1,
            search_bounds.2,
            search_bounds.3,
            2.0,
            search_color,
        );
        let search = if self.query.is_empty() {
            "Search scenarios (/)"
        } else {
            &self.query
        };
        draw_text(search, x + 16.0, y + 36.0, SMALL_FONT, MUTED);
        for (index, row) in self.testbed.visible_rows().iter().take(14).enumerate() {
            let Ok(row_index) = u16::try_from(index) else {
                continue;
            };
            let row_y = y + 56.0 + f32::from(row_index) * ROW_HEIGHT;
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
        let (x, y, width, _) = rect(region);
        if matches!(
            self.focus_return.current(),
            Some(FocusId::InspectorHeading | FocusId::InspectorDifference)
        ) {
            draw_rectangle_lines(x + 4.0, y + 2.0, width - 8.0, CONTROL_TARGET, 2.0, ACCENT);
        }
        draw_text("Inspect", x + 16.0, y + 28.0, 24.0, TEXT);
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
            &format!(
                "State: {}",
                if self.testbed.session_state() == liquidfun_differential::SessionState::ReadyPaused
                {
                    SESSION_PAUSED_LABEL.to_owned()
                } else {
                    format!("{:?}", self.testbed.session_state())
                }
            ),
            x,
            &mut line_y,
            TEXT,
        );
        if let Some(label) = self.maybe_last_scenario_action_label {
            line(label, x, &mut line_y, ACCENT);
        }
        if let Some(key) = self.maybe_selected_primitive.as_deref() {
            line(
                &format!("Selected semantic primitive: {}", shorten(key, 42)),
                x,
                &mut line_y,
                ACCENT,
            );
        }
        line(
            &format!("Zoom: {:.0}%", self.pixels_per_meter / 42.0 * 100.0),
            x,
            &mut line_y,
            MUTED,
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

    fn draw_controls(&self, layout: ResponsiveLayout) {
        fill_region(layout.shell().region(ShellRegion::Controls), PANEL_ALT);
        let state = self.testbed.session_state();
        for (index, focus) in FOCUS_ORDER.iter().copied().enumerate() {
            let label = match focus {
                ControlFocus::Scenario => "Scenarios",
                ControlFocus::Inspector => "Inspect",
                ControlFocus::RunPause
                    if state == liquidfun_differential::SessionState::Running =>
                {
                    "Pause"
                }
                ControlFocus::RunPause => "Run",
                ControlFocus::Step => "Step",
                ControlFocus::Restart => "Restart",
                ControlFocus::Capture => "Capture",
                ControlFocus::Settings => "Settings",
                ControlFocus::Overlay => match self.comparison_mode {
                    ComparisonMode::Overlay => "Overlay",
                    ComparisonMode::SideBySide => "Side by side",
                    ComparisonMode::SingleBackend => "Rust view",
                },
                ControlFocus::PreviousDifference => "Prev diff",
                ControlFocus::NextDifference => "Next diff",
                ControlFocus::About => "About",
            };
            let active = match focus {
                ControlFocus::Scenario => self.open_panel == OpenPanel::Scenario,
                ControlFocus::Inspector => self.open_panel == OpenPanel::Inspector,
                ControlFocus::Settings => self.open_panel == OpenPanel::Settings,
                ControlFocus::About => self.open_panel == OpenPanel::About,
                ControlFocus::Overlay => self.maybe_comparison.is_some(),
                ControlFocus::RunPause
                | ControlFocus::Step
                | ControlFocus::Restart
                | ControlFocus::Capture
                | ControlFocus::PreviousDifference
                | ControlFocus::NextDifference => false,
            };
            draw_accessible_button(
                control_bounds(focus, layout),
                label,
                index == self.focus_index && !focus_is_modal_heading(self.focus_return.current()),
                active,
            );
        }
    }

    fn draw_settings_panel(&self) {
        let panel = centered_panel(560.0, 430.0);
        draw_rectangle(panel.0, panel.1, panel.2, panel.3, PANEL);
        draw_rectangle_lines(
            panel.0,
            panel.1,
            panel.2,
            panel.3,
            if self.focus_return.current() == Some(FocusId::SettingsHeading) {
                2.0
            } else {
                1.0
            },
            ACCENT,
        );
        draw_text("Run settings", panel.0 + 24.0, panel.1 + 34.0, 24.0, TEXT);
        draw_text(
            "Validated values apply only through Apply & Restart",
            panel.0 + 24.0,
            panel.1 + 58.0,
            12.0,
            MUTED,
        );
        for field in SETTINGS_FIELDS {
            let bounds = settings_field_bounds(field);
            draw_text(
                setting_label(field),
                bounds.0,
                bounds.1 - 6.0,
                SMALL_FONT,
                TEXT,
            );
            draw_rectangle(bounds.0, bounds.1, bounds.2, bounds.3, PANEL_ALT);
            let focused = self.maybe_editing_setting == Some(field);
            draw_rectangle_lines(
                bounds.0,
                bounds.1,
                bounds.2,
                bounds.3,
                if focused { 2.0 } else { 1.0 },
                if focused { ACCENT } else { BORDER },
            );
            draw_text(
                self.settings.text(field),
                bounds.0 + 12.0,
                bounds.1 + 28.0,
                FONT,
                TEXT,
            );
            if let Some(error) = self.settings.maybe_error(field) {
                draw_text(
                    error,
                    bounds.0 + bounds.2 + 12.0,
                    bounds.1 + 28.0,
                    12.0,
                    ERROR,
                );
            }
        }
        draw_accessible_button(
            settings_apply_bounds(),
            "Apply & Restart",
            self.focus_return.current() == Some(FocusId::SettingsApply),
            self.settings.apply_enabled(),
        );
        draw_text(
            "Escape closes without applying",
            panel.0 + 24.0,
            panel.1 + panel.3 - 18.0,
            12.0,
            MUTED,
        );
    }

    #[allow(
        clippy::too_many_lines,
        reason = "the About surface intentionally renders one bounded complete provenance record"
    )]
    fn draw_about_panel(&self) {
        let about = self.about_panel();
        let panel = centered_panel(720.0, 610.0);
        draw_rectangle(panel.0, panel.1, panel.2, panel.3, PANEL);
        draw_rectangle_lines(
            panel.0,
            panel.1,
            panel.2,
            panel.3,
            if matches!(
                self.focus_return.current(),
                Some(FocusId::AboutHeading | FocusId::AboutLink)
            ) {
                2.0
            } else {
                1.0
            },
            ACCENT,
        );
        let mut y = panel.1 + 36.0;
        line(about.project_name(), panel.0 + 8.0, &mut y, TEXT);
        line(about.maintainer(), panel.0 + 8.0, &mut y, TEXT);
        line(about.license_summary(), panel.0 + 8.0, &mut y, TEXT);
        line(about.upstream_summary(), panel.0 + 8.0, &mut y, MUTED);
        line(about.version_label(), panel.0 + 8.0, &mut y, TEXT);
        line(about.commit_label(), panel.0 + 8.0, &mut y, TEXT);
        line(
            &format!("Profile: {}", about.profile()),
            panel.0 + 8.0,
            &mut y,
            MUTED,
        );
        line(
            &format!("Target: {}", about.target()),
            panel.0 + 8.0,
            &mut y,
            MUTED,
        );
        line(
            &format!("Toolchain: {}", about.rust_toolchain()),
            panel.0 + 8.0,
            &mut y,
            MUTED,
        );
        line(
            &format!(
                "Protocol: {}; adapter: {}",
                about.protocol_version(),
                about.adapter_version()
            ),
            panel.0 + 8.0,
            &mut y,
            MUTED,
        );
        line(
            &format!("Run identity: {}", shorten(about.run_identity(), 36)),
            panel.0 + 8.0,
            &mut y,
            MUTED,
        );
        line(
            &format!("Oracle: {}", shorten(about.oracle_identity(), 40)),
            panel.0 + 8.0,
            &mut y,
            MUTED,
        );
        line(
            &format!("Evidence tier: {}", about.evidence_tier()),
            panel.0 + 8.0,
            &mut y,
            MUTED,
        );
        for (index, link) in about.links().iter().enumerate() {
            let bounds = about_link_bounds(index);
            draw_accessible_button(
                bounds,
                link.label(),
                self.focus_return.current() == Some(FocusId::AboutLink)
                    && self.focused_about_link == index,
                false,
            );
            draw_text(
                shorten(link.url(), 42),
                bounds.0 + 4.0,
                bounds.1 + bounds.3 + 16.0,
                12.0,
                ACCENT,
            );
        }
        if let Some(status) = self.maybe_link_status.as_deref() {
            draw_text(
                status,
                panel.0 + panel.2 * 0.5,
                panel.1 + panel.3 - 18.0,
                12.0,
                TEXT,
            );
        }
        draw_text(
            "Tab selects a link; Enter or click copies its visible URL; Escape closes",
            panel.0 + 24.0,
            panel.1 + panel.3 - 18.0,
            12.0,
            MUTED,
        );
    }

    fn draw_shortcut_help(&self, _width: u32, _height: u32) {
        let panel = shortcut_help_panel();
        let (x, y, panel_width, panel_height) = panel;
        draw_rectangle(x, y, panel_width, panel_height, PANEL);
        draw_rectangle_lines(x, y, panel_width, panel_height, 2.0, ACCENT);
        draw_text("Keyboard shortcuts", x + 24.0, y + 38.0, 24.0, TEXT);
        let shortcuts = [
            "Space  Run/Pause    Right  Step once    R  Restart    C  Capture",
            "/  Search    B  Scenarios    I  Inspector    S  Validated settings",
            "1  Contacts    2  Particle contacts    3  Broad phase",
            "4  Profiles/statistics    O  Overlay/Side by side",
            "F  Focus difference    [ / ]  Previous/Next difference",
            "A  Apply next typed scenario action (shown when available)",
            "Home / double-click  Reset camera    Escape  Close this help",
            "Wheel  Zoom about pointer    Middle or Shift+primary drag  Pan",
        ];
        for (index, shortcut) in shortcuts.iter().enumerate() {
            let row = u16::try_from(index).map_or(0.0, f32::from);
            draw_text(shortcut, x + 24.0, y + 78.0 + row * 32.0, SMALL_FONT, TEXT);
        }
        if let Some(shortcut) = self.scenario_shortcuts().first() {
            draw_text(
                format!(
                    "Scenario {} — {} ({})",
                    shortcut.key().to_ascii_uppercase(),
                    shortcut.label(),
                    shortcut.action_id().as_str()
                ),
                x + 24.0,
                y + panel_height - 46.0,
                12.0,
                ACCENT,
            );
        }
        draw_text(
            "Presentation shortcuts never submit simulation commands.",
            x + 24.0,
            y + panel_height - 82.0,
            12.0,
            MUTED,
        );
        draw_accessible_button(
            shortcut_close_bounds(),
            "Close",
            self.focus_return.current() == Some(FocusId::ShortcutHelp),
            false,
        );
    }
}

fn focus_index(focus: ControlFocus) -> usize {
    FOCUS_ORDER
        .iter()
        .position(|candidate| *candidate == focus)
        .unwrap_or(0)
}

const fn control_for_focus(focus: FocusId) -> ControlFocus {
    match focus {
        FocusId::ScenarioButton | FocusId::ScenarioHeading | FocusId::ScenarioSearch => {
            ControlFocus::Scenario
        }
        FocusId::InspectorButton | FocusId::InspectorHeading | FocusId::InspectorDifference => {
            ControlFocus::Inspector
        }
        FocusId::SettingsButton
        | FocusId::SettingsHeading
        | FocusId::SettingsField
        | FocusId::SettingsApply => ControlFocus::Settings,
        FocusId::AboutButton
        | FocusId::AboutHeading
        | FocusId::AboutLink
        | FocusId::ShortcutHelp => ControlFocus::About,
    }
}

const fn focus_is_modal_heading(maybe_focus: Option<FocusId>) -> bool {
    matches!(
        maybe_focus,
        Some(
            FocusId::ScenarioHeading
                | FocusId::ScenarioSearch
                | FocusId::InspectorHeading
                | FocusId::InspectorDifference
                | FocusId::SettingsHeading
                | FocusId::SettingsField
                | FocusId::SettingsApply
                | FocusId::AboutHeading
                | FocusId::AboutLink
                | FocusId::ShortcutHelp
        )
    )
}

fn control_bounds(focus: ControlFocus, layout: ResponsiveLayout) -> (f32, f32, f32, f32) {
    let index = focus_index(focus);
    let rows = usize::from(layout.control_rows().max(1));
    let columns = FOCUS_ORDER.len().div_ceil(rows);
    let row = index / columns;
    let column = index % columns;
    let region = rect(layout.shell().region(ShellRegion::Controls));
    let cell_width = region.2 / u16::try_from(columns).map_or(1.0, f32::from);
    let cell_height = region.3 / u16::try_from(rows).map_or(1.0, f32::from);
    (
        region.0 + u16::try_from(column).map_or(0.0, f32::from) * cell_width + 2.0,
        region.1 + u16::try_from(row).map_or(0.0, f32::from) * cell_height + 2.0,
        (cell_width - 4.0).max(CONTROL_TARGET),
        (cell_height - 4.0).max(CONTROL_TARGET),
    )
}

fn centered_panel(width: f32, height: f32) -> (f32, f32, f32, f32) {
    let available_width = (screen_width() - 32.0).max(CONTROL_TARGET);
    let available_height = (screen_height() - 32.0).max(CONTROL_TARGET);
    let width = width.min(available_width);
    let height = height.min(available_height);
    (
        (screen_width() - width).max(0.0) * 0.5,
        (screen_height() - height).max(0.0) * 0.5,
        width,
        height,
    )
}

fn scenario_search_bounds(region: (u32, u32, u32, u32)) -> (f32, f32, f32, f32) {
    let (x, y, width, _) = rect(region);
    (x + 8.0, y + 8.0, width - 16.0, CONTROL_TARGET)
}

fn about_link_bounds(index: usize) -> (f32, f32, f32, f32) {
    let panel = centered_panel(720.0, 610.0);
    (
        panel.0 + panel.2 * 0.5,
        panel.1 + 52.0 + u16::try_from(index).map_or(0.0, f32::from) * 76.0,
        panel.2 * 0.5 - 16.0,
        CONTROL_TARGET,
    )
}

fn shortcut_help_panel() -> (f32, f32, f32, f32) {
    centered_panel(640.0, 500.0)
}

fn shortcut_close_bounds() -> (f32, f32, f32, f32) {
    let panel = shortcut_help_panel();
    (
        panel.0 + panel.2 - 136.0,
        panel.1 + panel.3 - 60.0,
        112.0,
        CONTROL_TARGET,
    )
}

fn settings_field_bounds(field: SettingsField) -> (f32, f32, f32, f32) {
    let panel = centered_panel(560.0, 430.0);
    let index = setting_index(field);
    (
        panel.0 + 24.0,
        panel.1 + 88.0 + u16::try_from(index).map_or(0.0, f32::from) * 68.0,
        220.0,
        CONTROL_TARGET,
    )
}

fn settings_apply_bounds() -> (f32, f32, f32, f32) {
    let panel = centered_panel(560.0, 430.0);
    (
        panel.0 + panel.2 - 204.0,
        panel.1 + panel.3 - 68.0,
        180.0,
        CONTROL_TARGET,
    )
}

fn setting_index(field: SettingsField) -> usize {
    SETTINGS_FIELDS
        .iter()
        .position(|candidate| *candidate == field)
        .unwrap_or(0)
}

const fn setting_label(field: SettingsField) -> &'static str {
    match field {
        SettingsField::Timestep => "Timestep seconds",
        SettingsField::VelocityIterations => "Velocity iterations",
        SettingsField::PositionIterations => "Position iterations",
        SettingsField::ParticleIterations => "Particle iterations",
    }
}

fn minimum_close_bounds() -> (f32, f32, f32, f32) {
    (
        24.0,
        (screen_height() - 68.0).max(104.0),
        96.0,
        CONTROL_TARGET,
    )
}

fn minimum_about_bounds() -> (f32, f32, f32, f32) {
    (
        136.0,
        (screen_height() - 68.0).max(104.0),
        220.0,
        CONTROL_TARGET,
    )
}

fn point_in_rect(point: (f32, f32), bounds: (f32, f32, f32, f32)) -> bool {
    (bounds.0..=bounds.0 + bounds.2).contains(&point.0)
        && (bounds.1..=bounds.1 + bounds.3).contains(&point.1)
}

fn draw_accessible_button(
    bounds: (f32, f32, f32, f32),
    label: &str,
    focused: bool,
    selected: bool,
) {
    draw_rectangle(
        bounds.0,
        bounds.1,
        bounds.2,
        bounds.3,
        if selected { PANEL_ALT } else { PANEL },
    );
    draw_rectangle_lines(
        bounds.0,
        bounds.1,
        bounds.2,
        bounds.3,
        if focused { 2.0 } else { 1.0 },
        if focused { ACCENT } else { BORDER },
    );
    let measured = measure_text(label, None, SMALL_FONT_SIZE, 1.0);
    let text_x = bounds.0 + ((bounds.2 - measured.width) * 0.5).max(6.0);
    let text_y = bounds.1 + (bounds.3 + measured.height) * 0.5 - 2.0;
    draw_text(label, text_x, text_y, SMALL_FONT, TEXT);
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
                    24.0,
                    ERROR,
                );
                draw_text(error, 24.0, 80.0, FONT, TEXT);
            }
        }
        next_frame().await;
    }
}
