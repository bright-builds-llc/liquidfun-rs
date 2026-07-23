//! Optional private interactive catalog testbed.

use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use eframe::egui::{
    self, Align2, Color32, FontId, Painter, PointerButton, Pos2, Rect, Sense, Stroke, StrokeKind,
    Vec2,
};
use liquidfun_differential::{
    ComparisonLimits, ComparisonModel, ComparisonState, SessionCommand, SessionState,
    compare_canonical_checkpoints,
};
use liquidfun_test_protocol::{
    ActionSchedule, CanonicalCheckpoint, CheckpointId, CheckpointPosition, DebugLayerName,
    DebugPrimitiveKey, HarnessLimits, Phase4PolicyProfile, RigidWorldAction, RunSettings,
    Sha256Hex, decode_canonical_checkpoint_jsonl,
};
use liquidfun_testbed::app::{AppEffect, AppShell, status_copy, status_marker};
use liquidfun_testbed::controller_adapter::{
    ControlCapability, ControllerAction, ControllerProjection, PARTICLE_PAUSE_ACTION_LABEL,
    SESSION_PAUSED_LABEL,
};
use liquidfun_testbed::input::{
    InputContext, InputEffect, KeyboardKey, PresentationAction, ScenarioShortcut, resolve_key,
};
use liquidfun_testbed::interactive::InteractiveTestbed;
use liquidfun_testbed::ui::differences::{BackendAvailability, ComparisonMode, DifferenceList};
use liquidfun_testbed::ui::inspector::{CheckpointDiagnostics, InspectorState, operational_copy};
use liquidfun_testbed::ui::layout::{PanelBehavior, ResponsiveLayout};
use liquidfun_testbed::ui::protocol_viewport::{
    ProtocolComparisonBackend, ProtocolDisplayPrimitive, ProtocolDisplayRecord,
    ProtocolLayerVisibility, ProtocolScreenPoint, ProtocolScreenStyle, ProtocolViewport,
    draw_protocol_comparison_frame, draw_protocol_frame, hit_test_frame, project_checkpoint,
};
use liquidfun_testbed::ui::settings::{SettingsEditor, SettingsField};
use liquidfun_testbed::ui::viewport::{Camera, DiagnosticScreenshotPath};
use liquidfun_testbed::ui::{AboutPanel, ProvenanceInput, build_about_panel};

const BACKGROUND: Color32 = Color32::from_rgb(13, 17, 23);
const PANEL: Color32 = Color32::from_rgb(21, 26, 34);
const PANEL_ALT: Color32 = Color32::from_rgb(28, 35, 45);
const BORDER: Color32 = Color32::from_rgb(64, 73, 86);
const MUTED: Color32 = Color32::from_rgb(166, 176, 189);
const ACCENT: Color32 = Color32::from_rgb(88, 166, 255);
const ERROR: Color32 = Color32::from_rgb(248, 81, 73);
const RUST_COMPARISON: Color32 = Color32::from_rgb(255, 140, 66);
const ORACLE_COMPARISON: Color32 = Color32::from_rgb(163, 113, 247);
const OVERLAY_OPACITY_PERCENT: u16 = 35;
const SETTINGS_FIELDS: [SettingsField; 4] = [
    SettingsField::Timestep,
    SettingsField::VelocityIterations,
    SettingsField::PositionIterations,
    SettingsField::ParticleIterations,
];

#[allow(
    clippy::struct_field_names,
    reason = "the maybe_ prefix is the repository convention for optional values"
)]
struct ComparisonLifecycle<Model, Identity> {
    maybe_model: Option<Model>,
    maybe_identity: Option<Identity>,
    maybe_error: Option<String>,
}

pub(crate) struct DesktopDiagnostics<Model, Identity> {
    comparison: ComparisonLifecycle<Model, Identity>,
    maybe_error: Option<String>,
}

impl<Model, Identity> Default for DesktopDiagnostics<Model, Identity> {
    fn default() -> Self {
        Self {
            comparison: ComparisonLifecycle {
                maybe_model: None,
                maybe_identity: None,
                maybe_error: None,
            },
            maybe_error: None,
        }
    }
}

impl<Model, Identity> DesktopDiagnostics<Model, Identity> {
    pub(crate) fn apply_comparison(&mut self, identity: Identity, result: Result<Model, String>) {
        self.comparison.maybe_identity = Some(identity);
        match result {
            Ok(model) => {
                self.comparison.maybe_model = Some(model);
                self.comparison.maybe_error = None;
            }
            Err(error) => {
                self.comparison.maybe_model = None;
                self.comparison.maybe_error = Some(bound_message(&error));
            }
        }
    }

    pub(crate) fn reset_comparison(&mut self) {
        self.comparison.maybe_model = None;
        self.comparison.maybe_identity = None;
        self.comparison.maybe_error = None;
    }

    pub(crate) fn set_error(&mut self, error: impl std::fmt::Display) {
        self.maybe_error = Some(bound_message(&error.to_string()));
    }

    pub(crate) fn maybe_comparison(&self) -> Option<&Model> {
        self.comparison.maybe_model.as_ref()
    }

    pub(crate) fn maybe_compared_identity(&self) -> Option<&Identity> {
        self.comparison.maybe_identity.as_ref()
    }

    pub(crate) fn maybe_comparison_error(&self) -> Option<&str> {
        self.comparison.maybe_error.as_deref()
    }

    pub(crate) fn maybe_error(&self) -> Option<&str> {
        self.maybe_error.as_deref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpenPanel {
    None,
    Scenario,
    Inspector,
    Settings,
    About,
    ShortcutHelp,
}

enum PendingEffect {
    Controller(ControllerAction),
    Select(usize),
    ApplySettings(RunSettings),
}

struct DesktopApp {
    shell: AppShell,
    testbed: InteractiveTestbed,
    query: String,
    layers: ProtocolLayerVisibility,
    layer_enabled: [bool; 9],
    pixels_per_meter: f32,
    center_x: f32,
    center_y: f32,
    maybe_oracle: Option<CanonicalCheckpoint>,
    diagnostics: DesktopDiagnostics<ComparisonModel, (Sha256Hex, CheckpointId)>,
    open_panel: OpenPanel,
    settings: SettingsEditor,
    settings_drafts: [String; 4],
    comparison_mode: ComparisonMode,
    focused_difference: usize,
    maybe_selected_primitive: Option<String>,
    maybe_last_scenario_action_label: Option<&'static str>,
    maybe_pending_effect: Option<PendingEffect>,
    maybe_driver_tick: Option<Instant>,
    maybe_screenshot_status: Option<String>,
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
        let settings_drafts = settings_drafts(&settings);
        Ok(Self {
            shell: AppShell::default(),
            testbed,
            query: String::new(),
            layers: ProtocolLayerVisibility::all(),
            layer_enabled: [true; 9],
            pixels_per_meter: 42.0,
            center_x: 0.0,
            center_y: 0.0,
            maybe_oracle,
            diagnostics: DesktopDiagnostics::default(),
            open_panel: OpenPanel::None,
            settings,
            settings_drafts,
            comparison_mode: ComparisonMode::Overlay,
            focused_difference: 0,
            maybe_selected_primitive: None,
            maybe_last_scenario_action_label: None,
            maybe_pending_effect: None,
            maybe_driver_tick: None,
            maybe_screenshot_status: None,
        })
    }

    fn drive_running_session(&mut self, ctx: &egui::Context) {
        self.handle_screenshot_result(ctx);
        if self.testbed.session_state() != SessionState::Running {
            self.maybe_driver_tick = None;
            self.refresh_comparison();
            return;
        }
        let now = Instant::now();
        let elapsed = self
            .maybe_driver_tick
            .replace(now)
            .map_or(Duration::ZERO, |previous| {
                now.saturating_duration_since(previous)
                    .min(Duration::from_millis(250))
            });
        match self.testbed.drive_logical_time(elapsed) {
            Ok(ticks) if ticks > 0 && self.testbed.reachable_checkpoint_id().is_some() => {
                if let Err(error) = self.testbed.capture_reachable_checkpoint() {
                    self.diagnostics.set_error(error);
                }
            }
            Ok(_) => {}
            Err(error) => self.diagnostics.set_error(error),
        }
        self.refresh_comparison();
        ctx.request_repaint_after(Duration::from_millis(8));
    }

    fn queue(&mut self, effect: PendingEffect) {
        if self.maybe_pending_effect.is_none() {
            self.maybe_pending_effect = Some(effect);
        }
    }

    fn dispatch_pending(&mut self) {
        let Some(effect) = self.maybe_pending_effect.take() else {
            return;
        };
        let clears_comparison = matches!(
            effect,
            PendingEffect::Select(_)
                | PendingEffect::ApplySettings(_)
                | PendingEffect::Controller(ControllerAction::Restart)
        );
        let command = match effect {
            PendingEffect::Controller(action) => self.testbed.begin_action(action),
            PendingEffect::Select(index) => self.testbed.begin_select_visible(index),
            PendingEffect::ApplySettings(settings) => self.testbed.begin_settings(settings),
        };
        let result = command.and_then(|command: SessionCommand| {
            let AppEffect::Submit(command) = self.shell.submit(command);
            self.testbed.submit_command(command)
        });
        match result {
            Ok(()) => {
                if clears_comparison {
                    self.diagnostics.reset_comparison();
                }
                if let Some(settings) = self.testbed.selected_settings() {
                    self.settings = SettingsEditor::new(settings);
                    self.settings_drafts = settings_drafts(&self.settings);
                }
            }
            Err(error) => self.diagnostics.set_error(error),
        }
    }

    fn refresh_comparison(&mut self) {
        let Some(native) = self.maybe_display_checkpoint() else {
            return;
        };
        let native_identity = (
            native.resolved_sha256().clone(),
            native.checkpoint_id().clone(),
        );
        if self.diagnostics.maybe_compared_identity() == Some(&native_identity) {
            return;
        }
        let Some(oracle) = self.maybe_oracle.as_ref() else {
            self.diagnostics.reset_comparison();
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
        self.diagnostics
            .apply_comparison(native_identity, comparison);
        if self.diagnostics.maybe_comparison().is_some() {
            self.focused_difference = 0;
        }
    }

    fn maybe_display_checkpoint(&self) -> Option<&CanonicalCheckpoint> {
        if self.maybe_oracle.is_some() {
            return self.testbed.latest_checkpoint();
        }
        self.testbed.presentation_checkpoint()
    }

    fn render_app_bar(&mut self, root: &mut egui::Ui) {
        egui::Panel::top("app_bar")
            .exact_size(48.0)
            .frame(egui::Frame::new().fill(PANEL))
            .show(root, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.heading("liquidfun-rs");
                    let comparison = self
                        .diagnostics
                        .maybe_comparison()
                        .map(ComparisonModel::state);
                    ui.colored_label(
                        state_color(comparison),
                        format!(
                            "{} {}",
                            status_marker(self.testbed.session_state(), comparison),
                            status_copy(self.testbed.session_state(), comparison)
                        ),
                    );
                    ui.label(
                        egui::RichText::new(
                            "Private diagnostic UI — pixels and timing are not compatibility authority",
                        )
                        .color(MUTED),
                    );
                    if ui.button("About & provenance").clicked() {
                        self.open_panel = OpenPanel::About;
                    }
                });
            });
    }

    fn render_scenarios(&mut self, root: &mut egui::Ui) {
        egui::Panel::left("scenario_rail")
            .resizable(true)
            .default_size(280.0)
            .frame(egui::Frame::new().fill(PANEL))
            .show(root, |ui| {
                ui.heading("Scenarios");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.query)
                        .hint_text("Search scenarios (/)")
                        .desired_width(f32::INFINITY),
                );
                if response.changed()
                    && let Err(error) = self.testbed.set_query(&self.query)
                {
                    self.diagnostics.set_error(error);
                }
                let current = self.testbed.current_selection().cloned();
                let mut maybe_selected = None;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (index, row) in self.testbed.visible_rows().iter().enumerate() {
                        let identity = format!(
                            "{}@{}  R:{} O:{} V:{}",
                            row.selection().catalog_slug(),
                            row.selection().scenario_version(),
                            yes_no(row.eligibility().rust()),
                            yes_no(row.eligibility().oracle()),
                            yes_no(row.eligibility().visual())
                        );
                        let selected = current
                            .as_ref()
                            .is_some_and(|selection| selection == row.selection());
                        if ui
                            .selectable_label(
                                selected,
                                format!("{}\n{identity}", row.display_title()),
                            )
                            .clicked()
                        {
                            maybe_selected = Some(index);
                        }
                    }
                });
                if let Some(index) = maybe_selected {
                    self.queue(PendingEffect::Select(index));
                }
            });
    }

    fn render_inspector(&mut self, root: &mut egui::Ui) {
        egui::Panel::right("inspector")
            .resizable(true)
            .default_size(360.0)
            .frame(egui::Frame::new().fill(PANEL))
            .show(root, |ui| {
                ui.heading("Inspect");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let (heading, body) = comparison_copy(self.diagnostics.maybe_comparison());
                    ui.label(egui::RichText::new(heading).strong());
                    ui.colored_label(MUTED, body);
                    if let Some(selected) = self.testbed.selected() {
                        let identity = selected.identity();
                        ui.label(format!(
                            "Scenario: {}@{}",
                            identity.slug().as_str(),
                            identity.scenario_version().get()
                        ));
                        ui.colored_label(
                            MUTED,
                            format!(
                                "Resolved: {}",
                                shorten(identity.content_sha256().as_str(), 18)
                            ),
                        );
                    }
                    let state = if self.testbed.session_state() == SessionState::ReadyPaused {
                        SESSION_PAUSED_LABEL.to_owned()
                    } else {
                        format!("{:?}", self.testbed.session_state())
                    };
                    ui.label(format!("State: {state}"));
                    ui.label(format!(
                        "Logical steps: {}",
                        self.testbed.completed_logical_steps()
                    ));
                    if let Some(label) = self.maybe_last_scenario_action_label {
                        ui.colored_label(ACCENT, label);
                    }
                    if let Some(key) = self.maybe_selected_primitive.as_deref() {
                        ui.colored_label(
                            ACCENT,
                            format!("Selected primitive: {}", shorten(key, 42)),
                        );
                    }
                    self.render_checkpoint_diagnostics(ui);
                    self.render_difference(ui);
                    if let Some(error) = self.diagnostics.maybe_comparison_error() {
                        ui.colored_label(ERROR, format!("Comparison error: {error}"));
                    }
                    if let Some(error) = self.diagnostics.maybe_error() {
                        ui.colored_label(ERROR, format!("Last bounded error: {error}"));
                    }
                    if let Some(status) = self.maybe_screenshot_status.as_deref() {
                        ui.colored_label(ACCENT, status);
                    }
                });
            });
    }

    fn render_checkpoint_diagnostics(&self, ui: &mut egui::Ui) {
        let Some(displayed) = self.maybe_display_checkpoint() else {
            ui.colored_label(MUTED, "Captured checkpoints: 0");
            return;
        };
        let diagnostics = CheckpointDiagnostics::from_checkpoint(displayed);
        ui.label(format!(
            "Captured checkpoints: {}",
            self.testbed.captured_checkpoint_count()
        ));
        let showing_history = self
            .testbed
            .latest_checkpoint()
            .is_some_and(|latest| latest.checkpoint_id() != displayed.checkpoint_id());
        ui.label(format!(
            "Displayed: {}{}",
            displayed.checkpoint_id().as_str(),
            if showing_history {
                " (last drawable)"
            } else {
                ""
            }
        ));
        if showing_history {
            ui.colored_label(MUTED, "Latest capture is empty after teardown");
        }
        let boundary = match displayed.position() {
            CheckpointPosition::Action { ordinal, .. } => format!("action {ordinal}"),
            CheckpointPosition::LogicalStep { ordinal } => format!("logical step {ordinal}"),
        };
        ui.colored_label(
            MUTED,
            format!(
                "Boundary: {boundary} | sim {:.5}s",
                displayed.simulation_time_bits().to_f32()
            ),
        );
        ui.label(format!(
            "World B:{} F:{} J:{} C:{} P:{}",
            count_text(diagnostics.maybe_body_count()),
            count_text(diagnostics.maybe_fixture_count()),
            count_text(diagnostics.maybe_joint_count()),
            count_text(diagnostics.maybe_contact_count()),
            count_text(diagnostics.maybe_particle_count())
        ));
        ui.colored_label(
            MUTED,
            format!(
                "Draw shapes:{} joints:{} particles:{}",
                diagnostics.layer_count(DebugLayerName::Shapes),
                diagnostics.layer_count(DebugLayerName::Joints),
                diagnostics.layer_count(DebugLayerName::Particles)
            ),
        );
    }

    fn render_difference(&mut self, ui: &mut egui::Ui) {
        let Some(comparison) = self.diagnostics.maybe_comparison() else {
            return;
        };
        let differences =
            DifferenceList::new(comparison, Camera::default(), BackendAvailability::Both);
        let entries = differences.entries();
        ui.separator();
        ui.label(format!("Comparison: {:?}", comparison.state()));
        if entries.is_empty() {
            ui.colored_label(MUTED, "No differences at this checkpoint");
            return;
        }
        self.focused_difference = self.focused_difference.min(entries.len() - 1);
        ui.horizontal(|ui| {
            if ui.button("Previous").clicked() {
                self.focused_difference =
                    (self.focused_difference + entries.len() - 1) % entries.len();
            }
            if ui.button("Next").clicked() {
                self.focused_difference = (self.focused_difference + 1) % entries.len();
            }
        });
        let entry = entries[self.focused_difference];
        ui.label(format!(
            "Difference {} of {}",
            self.focused_difference + 1,
            entries.len()
        ));
        ui.label(entry.semantic_path());
        ui.colored_label(
            MUTED,
            format!("Rust: {}", entry.maybe_rust_value().unwrap_or("absent")),
        );
        ui.colored_label(
            MUTED,
            format!("Oracle: {}", entry.maybe_oracle_value().unwrap_or("absent")),
        );
    }

    fn render_controls(&mut self, root: &mut egui::Ui) {
        let ctx = root.ctx().clone();
        egui::Panel::bottom("controls")
            .frame(egui::Frame::new().fill(PANEL_ALT))
            .show(root, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Scenarios").clicked() {
                        self.open_panel = OpenPanel::Scenario;
                    }
                    if ui.button("Inspect").clicked() {
                        self.open_panel = OpenPanel::Inspector;
                    }
                    let projection = ControllerProjection::from_state(self.testbed.session_state());
                    if self.testbed.session_state() == SessionState::Running {
                        if ui
                            .add_enabled(
                                projection.enabled(ControlCapability::Pause),
                                egui::Button::new("Pause"),
                            )
                            .clicked()
                        {
                            self.queue(PendingEffect::Controller(ControllerAction::Pause));
                        }
                    } else if ui
                        .add_enabled(
                            projection.enabled(ControlCapability::Run),
                            egui::Button::new("Run"),
                        )
                        .clicked()
                    {
                        self.queue(PendingEffect::Controller(ControllerAction::Run));
                    }
                    if ui
                        .add_enabled(
                            projection.enabled(ControlCapability::StepOnce),
                            egui::Button::new("Step"),
                        )
                        .clicked()
                    {
                        self.queue(PendingEffect::Controller(ControllerAction::StepOnce));
                    }
                    if ui
                        .add_enabled(
                            projection.enabled(ControlCapability::Restart),
                            egui::Button::new("Restart"),
                        )
                        .clicked()
                    {
                        self.queue(PendingEffect::Controller(ControllerAction::Restart));
                    }
                    let maybe_checkpoint = self.testbed.reachable_checkpoint_id().cloned();
                    if ui
                        .add_enabled(
                            projection.enabled(ControlCapability::Capture)
                                && maybe_checkpoint.is_some(),
                            egui::Button::new("Capture"),
                        )
                        .clicked()
                        && let Some(checkpoint_id) = maybe_checkpoint
                    {
                        self.queue(PendingEffect::Controller(
                            ControllerAction::CaptureCheckpoint(checkpoint_id),
                        ));
                    }
                    if ui.button("Settings").clicked() {
                        self.open_panel = OpenPanel::Settings;
                    }
                    if ui
                        .add_enabled(
                            self.diagnostics.maybe_comparison().is_some(),
                            egui::Button::new(match self.comparison_mode {
                                ComparisonMode::Overlay => "Overlay",
                                ComparisonMode::SideBySide => "Side by side",
                                ComparisonMode::SingleBackend => "Rust view",
                            }),
                        )
                        .clicked()
                    {
                        self.comparison_mode = match self.comparison_mode {
                            ComparisonMode::Overlay => ComparisonMode::SideBySide,
                            ComparisonMode::SideBySide | ComparisonMode::SingleBackend => {
                                ComparisonMode::Overlay
                            }
                        };
                    }
                    if ui.button("Screenshot").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                            egui::UserData::default(),
                        ));
                    }
                    if ui.button("Shortcuts").clicked() {
                        self.open_panel = OpenPanel::ShortcutHelp;
                    }
                });
            });
    }

    fn render_viewport(&mut self, root: &mut egui::Ui) {
        let ctx = root.ctx().clone();
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BACKGROUND))
            .show(root, |ui| {
                let (response, painter) =
                    ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
                painter.rect_stroke(
                    response.rect,
                    0.0,
                    Stroke::new(1.0, BORDER),
                    StrokeKind::Inside,
                );
                self.handle_viewport_gestures(&ctx, &response);
                let Some(checkpoint) = self.maybe_display_checkpoint() else {
                    painter.text(
                        response.rect.left_top() + Vec2::new(20.0, 20.0),
                        Align2::LEFT_TOP,
                        "Step the scenario to render a semantic checkpoint",
                        FontId::proportional(18.0),
                        MUTED,
                    );
                    return;
                };
                let maybe_pair = self
                    .diagnostics
                    .maybe_comparison()
                    .zip(self.maybe_oracle.as_ref());
                match (self.comparison_mode, maybe_pair) {
                    (ComparisonMode::SideBySide, Some((comparison, oracle))) => {
                        let gap = 8.0;
                        let width = (response.rect.width() - gap) * 0.5;
                        let rust_rect = Rect::from_min_size(
                            response.rect.min,
                            Vec2::new(width, response.rect.height()),
                        );
                        let oracle_rect = Rect::from_min_size(
                            Pos2::new(rust_rect.max.x + gap, response.rect.min.y),
                            rust_rect.size(),
                        );
                        paint_checkpoint(
                            &painter,
                            checkpoint,
                            rust_rect,
                            self.camera(),
                            self.layers,
                            Some((comparison, ProtocolComparisonBackend::Rust)),
                        );
                        paint_checkpoint(
                            &painter,
                            oracle,
                            oracle_rect,
                            self.camera(),
                            self.layers,
                            Some((comparison, ProtocolComparisonBackend::Oracle)),
                        );
                    }
                    (ComparisonMode::Overlay, Some((comparison, oracle))) => {
                        paint_checkpoint(
                            &painter,
                            checkpoint,
                            response.rect,
                            self.camera(),
                            self.layers,
                            Some((comparison, ProtocolComparisonBackend::Rust)),
                        );
                        paint_checkpoint(
                            &painter,
                            oracle,
                            response.rect,
                            self.camera(),
                            self.layers,
                            Some((comparison, ProtocolComparisonBackend::Oracle)),
                        );
                    }
                    (ComparisonMode::SingleBackend, _)
                    | (ComparisonMode::Overlay | ComparisonMode::SideBySide, None) => {
                        paint_checkpoint(
                            &painter,
                            checkpoint,
                            response.rect,
                            self.camera(),
                            self.layers,
                            None,
                        );
                    }
                }
            });
    }

    fn handle_viewport_gestures(&mut self, ctx: &egui::Context, response: &egui::Response) {
        if !response.hovered() {
            return;
        }
        let scroll = ctx.input(|input| input.smooth_scroll_delta.y);
        if scroll != 0.0
            && let Some(pointer) = response.hover_pos()
        {
            let old_scale = self.pixels_per_meter;
            let new_scale = (old_scale * 1.1_f32.powf(scroll / 40.0)).clamp(5.0, 400.0);
            let offset = pointer - response.rect.center();
            let world_x = self.center_x + offset.x / old_scale;
            let world_y = self.center_y - offset.y / old_scale;
            self.center_x = world_x - offset.x / new_scale;
            self.center_y = world_y + offset.y / new_scale;
            self.pixels_per_meter = new_scale;
        }
        let shift = ctx.input(|input| input.modifiers.shift);
        let panning = response.dragged_by(PointerButton::Middle)
            || (shift && response.dragged_by(PointerButton::Primary));
        if panning {
            let delta = ctx.input(|input| input.pointer.delta());
            self.center_x -= delta.x / self.pixels_per_meter;
            self.center_y += delta.y / self.pixels_per_meter;
        }
        if response.double_clicked() {
            self.center_x = 0.0;
            self.center_y = 0.0;
            self.pixels_per_meter = 42.0;
            self.maybe_selected_primitive = None;
        } else if response.clicked()
            && let Some(pointer) = response.interact_pointer_pos()
        {
            self.maybe_selected_primitive = self.hit_test(response.rect, pointer);
        }
    }

    fn hit_test(&self, rect: Rect, pointer: Pos2) -> Option<String> {
        let checkpoint = self.maybe_display_checkpoint()?;
        let viewport = protocol_viewport(rect, self.camera())?;
        let frame = project_checkpoint(checkpoint, viewport, self.layers).ok()?;
        hit_test_frame(
            &frame,
            ProtocolScreenPoint {
                x: pointer.x,
                y: pointer.y,
            },
            6.0,
        )
        .map(|key| format!("{key:?}"))
    }

    const fn camera(&self) -> (f32, f32, f32) {
        (self.center_x, self.center_y, self.pixels_per_meter)
    }

    fn render_settings(&mut self, ctx: &egui::Context) {
        if self.open_panel != OpenPanel::Settings {
            return;
        }
        let mut open = true;
        egui::Window::new("Run settings")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.colored_label(MUTED, "Validated values apply only through Apply & Restart");
                for field in SETTINGS_FIELDS {
                    ui.label(setting_label(field));
                    let index = setting_index(field);
                    let response = ui.text_edit_singleline(&mut self.settings_drafts[index]);
                    if response.lost_focus() || response.changed() {
                        self.settings
                            .edit(field, self.settings_drafts[index].clone());
                        self.settings.commit(field);
                    }
                    if let Some(error) = self.settings.maybe_error(field) {
                        ui.colored_label(ERROR, error);
                    }
                }
                if ui
                    .add_enabled(
                        self.settings.apply_enabled(),
                        egui::Button::new("Apply & Restart"),
                    )
                    .clicked()
                {
                    self.queue(PendingEffect::ApplySettings(self.settings.accepted()));
                    self.open_panel = OpenPanel::None;
                }
            });
        if !open {
            self.open_panel = OpenPanel::None;
        }
    }

    fn render_about(&mut self, ctx: &egui::Context) {
        if self.open_panel != OpenPanel::About {
            return;
        }
        let about = self.about_panel();
        let mut open = true;
        egui::Window::new("About & provenance")
            .open(&mut open)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading(about.project_name());
                ui.label(about.maintainer());
                ui.label(about.license_summary());
                ui.colored_label(MUTED, about.upstream_summary());
                ui.separator();
                for value in [
                    about.version_label(),
                    about.commit_label(),
                    about.profile(),
                    about.target(),
                    about.rust_toolchain(),
                    about.protocol_version(),
                    about.adapter_version(),
                    about.run_identity(),
                    about.oracle_identity(),
                    about.evidence_tier(),
                ] {
                    ui.label(value);
                }
                ui.separator();
                for link in about.links() {
                    ui.horizontal(|ui| {
                        if ui.link(link.label()).clicked() {
                            ctx.open_url(egui::OpenUrl::new_tab(link.url()));
                        }
                        if ui.small_button("Copy URL").clicked() {
                            ctx.copy_text(link.url().to_owned());
                        }
                        ui.colored_label(MUTED, link.url());
                    });
                }
            });
        if !open {
            self.open_panel = OpenPanel::None;
        }
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

    fn render_shortcuts(&mut self, ctx: &egui::Context) {
        if self.open_panel != OpenPanel::ShortcutHelp {
            return;
        }
        let mut open = true;
        egui::Window::new("Keyboard shortcuts")
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                for shortcut in [
                    "Space Run/Pause · Right Step · R Restart · C Capture",
                    "/ Search · 1–4 overlay groups · O Overlay/Side by side",
                    "F Focus difference · [ / ] Previous/Next difference",
                    "A Apply next typed scenario action",
                    "Home/double-click Reset camera · Wheel Zoom · Shift-drag Pan",
                ] {
                    ui.label(shortcut);
                }
                ui.colored_label(
                    MUTED,
                    "Presentation shortcuts never submit simulation commands.",
                );
                if let Some(shortcut) = self.scenario_shortcuts().first() {
                    ui.colored_label(
                        ACCENT,
                        format!(
                            "{} — {} ({})",
                            shortcut.key().to_ascii_uppercase(),
                            shortcut.label(),
                            shortcut.action_id().as_str()
                        ),
                    );
                }
            });
        if !open {
            self.open_panel = OpenPanel::None;
        }
    }

    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        let editing = ctx.egui_wants_keyboard_input();
        let scenario_shortcuts = self.scenario_shortcuts();
        let maybe_key = ctx.input(|input| {
            [
                (egui::Key::Space, KeyboardKey::Space),
                (egui::Key::ArrowRight, KeyboardKey::Right),
                (egui::Key::R, KeyboardKey::R),
                (egui::Key::C, KeyboardKey::C),
                (egui::Key::Slash, KeyboardKey::Slash),
                (egui::Key::F, KeyboardKey::F),
                (egui::Key::OpenBracket, KeyboardKey::LeftBracket),
                (egui::Key::CloseBracket, KeyboardKey::RightBracket),
                (egui::Key::Num1, KeyboardKey::Digit1),
                (egui::Key::Num2, KeyboardKey::Digit2),
                (egui::Key::Num3, KeyboardKey::Digit3),
                (egui::Key::Num4, KeyboardKey::Digit4),
                (egui::Key::Home, KeyboardKey::Home),
                (egui::Key::Questionmark, KeyboardKey::QuestionMark),
                (egui::Key::Escape, KeyboardKey::Escape),
                (egui::Key::A, KeyboardKey::Scenario('a')),
            ]
            .into_iter()
            .find_map(|(egui_key, semantic_key)| {
                input.key_pressed(egui_key).then_some(semantic_key)
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
                editing_field: editing,
                maybe_checkpoint_id: checkpoint_id.as_ref(),
                scenario_shortcuts: &scenario_shortcuts,
            },
        );
        match effect {
            Some(InputEffect::Controller(action)) => {
                if matches!(action, ControllerAction::ApplyScenarioAction(_)) {
                    self.maybe_last_scenario_action_label = Some(PARTICLE_PAUSE_ACTION_LABEL);
                }
                self.queue(PendingEffect::Controller(action));
            }
            Some(InputEffect::Presentation(action)) => self.apply_presentation(action),
            None => {}
        }
    }

    fn apply_presentation(&mut self, action: PresentationAction) {
        match action {
            PresentationAction::FocusScenarioSearch => self.open_panel = OpenPanel::Scenario,
            PresentationAction::FocusDifference => {
                self.open_panel = OpenPanel::Inspector;
                self.focused_difference = 0;
            }
            PresentationAction::PreviousDifference => self.move_difference(-1),
            PresentationAction::NextDifference => self.move_difference(1),
            PresentationAction::ToggleOverlayGroup(group) => self.toggle_group(group),
            PresentationAction::ResetCamera => {
                self.center_x = 0.0;
                self.center_y = 0.0;
                self.pixels_per_meter = 42.0;
            }
            PresentationAction::OpenShortcutHelp => self.open_panel = OpenPanel::ShortcutHelp,
            PresentationAction::CloseTopmostOrClearFocus => self.open_panel = OpenPanel::None,
        }
    }

    fn move_difference(&mut self, direction: i8) {
        let count = self.diagnostics.maybe_comparison().map_or(0, |model| {
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

    fn toggle_group(&mut self, group: u8) {
        let layers: &[DebugLayerName] = match group {
            1 => &[DebugLayerName::Contacts, DebugLayerName::ContactNormals],
            2 => &[DebugLayerName::ParticleContacts],
            3 => &[DebugLayerName::BroadPhase],
            4 => &[DebugLayerName::Labels],
            _ => &[],
        };
        for layer in layers {
            let index = layer_index(*layer);
            self.layer_enabled[index] = !self.layer_enabled[index];
            self.layers.set(*layer, self.layer_enabled[index]);
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

    fn handle_screenshot_result(&mut self, ctx: &egui::Context) {
        let maybe_image = ctx.input(|input| {
            input.events.iter().find_map(|event| {
                let egui::Event::Screenshot { image, .. } = event else {
                    return None;
                };
                Some(Arc::clone(image))
            })
        });
        let Some(image) = maybe_image else {
            return;
        };
        self.maybe_screenshot_status = Some(match save_screenshot(&image) {
            Ok(path) => format!(
                "Saved {} — diagnostic only, not compatibility evidence",
                path.display()
            ),
            Err(error) => error,
        });
    }
}

impl eframe::App for DesktopApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drive_running_session(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_keyboard(&ctx);
        let size = ctx.input(|input| input.content_rect().size());
        let layout =
            ResponsiveLayout::for_window(bounded_dimension(size.x), bounded_dimension(size.y));
        self.render_app_bar(ui);
        self.render_controls(ui);
        if layout.panel_behavior() == PanelBehavior::WindowTooSmall {
            egui::CentralPanel::default().show(ui, |ui| {
                ui.heading("Window too small");
                ui.label("Resize to at least 640 × 480 to use the semantic testbed.");
                if ui.button("Close").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                if ui.button("About & provenance").clicked() {
                    self.open_panel = OpenPanel::About;
                }
            });
        } else {
            if matches!(
                layout.panel_behavior(),
                PanelBehavior::BothVisible | PanelBehavior::InspectorDrawer
            ) || self.open_panel == OpenPanel::Scenario
            {
                self.render_scenarios(ui);
            }
            if layout.panel_behavior() == PanelBehavior::BothVisible
                || self.open_panel == OpenPanel::Inspector
            {
                self.render_inspector(ui);
            }
            self.render_viewport(ui);
        }
        self.render_settings(&ctx);
        self.render_about(&ctx);
        self.render_shortcuts(&ctx);
        self.dispatch_pending();
    }
}

fn paint_checkpoint(
    painter: &Painter,
    checkpoint: &CanonicalCheckpoint,
    rect: Rect,
    camera: (f32, f32, f32),
    layers: ProtocolLayerVisibility,
    maybe_comparison: Option<(&ComparisonModel, ProtocolComparisonBackend)>,
) {
    let Some(viewport) = protocol_viewport(rect, camera) else {
        return;
    };
    let Ok(frame) = project_checkpoint(checkpoint, viewport, layers) else {
        painter.text(
            rect.left_top() + Vec2::splat(20.0),
            Align2::LEFT_TOP,
            "Semantic viewport rejected invalid geometry",
            FontId::proportional(18.0),
            ERROR,
        );
        return;
    };
    if let Some((comparison, backend)) = maybe_comparison {
        draw_protocol_comparison_frame(&frame, comparison, backend, None);
        for record in frame.primitives() {
            let state = primitive_comparison_state(comparison, record.key());
            if should_skip(state, backend) {
                continue;
            }
            let style = comparison_style(record.style(), state, backend);
            paint_record(painter, record, style);
        }
    } else {
        draw_protocol_frame(&frame);
        for record in frame.primitives() {
            paint_record(painter, record, record.style());
        }
    }
    if checkpoint.debug_primitives().is_empty() {
        painter.text(
            rect.left_top() + Vec2::new(20.0, 48.0),
            Align2::LEFT_TOP,
            "Checkpoint has no drawable primitives",
            FontId::proportional(18.0),
            MUTED,
        );
    } else if frame.primitives().is_empty() {
        painter.text(
            rect.left_top() + Vec2::new(20.0, 48.0),
            Align2::LEFT_TOP,
            "No primitives in enabled debug layers",
            FontId::proportional(18.0),
            MUTED,
        );
    }
}

fn paint_record(painter: &Painter, record: &ProtocolDisplayRecord, style: ProtocolScreenStyle) {
    let stroke = Stroke::new(style.stroke_width.max(1.0), color(style.stroke));
    if let Some(fill) = style.maybe_fill {
        paint_fill(painter, record.primitive(), color(fill));
    }
    match record.primitive() {
        ProtocolDisplayPrimitive::Point { position, radius }
        | ProtocolDisplayPrimitive::Circle {
            center: position,
            radius,
        } => {
            painter.circle_stroke(point(*position), *radius, stroke);
        }
        ProtocolDisplayPrimitive::Segment { start, end } => {
            painter.line_segment([point(*start), point(*end)], stroke);
        }
        ProtocolDisplayPrimitive::Polyline { vertices, closed } => {
            let mut points = vertices.iter().copied().map(point).collect::<Vec<_>>();
            if *closed && let Some(first) = points.first().copied() {
                points.push(first);
            }
            painter.add(egui::Shape::line(points, stroke));
        }
        ProtocolDisplayPrimitive::TransformAxes {
            origin,
            x_end,
            y_end,
        } => {
            painter.line_segment([point(*origin), point(*x_end)], stroke);
            painter.line_segment([point(*origin), point(*y_end)], stroke);
        }
        ProtocolDisplayPrimitive::Aabb {
            left,
            top,
            right,
            bottom,
        } => {
            painter.rect_stroke(
                Rect::from_min_max(Pos2::new(*left, *top), Pos2::new(*right, *bottom)),
                0.0,
                stroke,
                StrokeKind::Middle,
            );
        }
        ProtocolDisplayPrimitive::Arrow { start, end } => {
            paint_arrow(painter, *start, *end, stroke);
        }
        ProtocolDisplayPrimitive::Label { position, text } => {
            painter.text(
                point(*position),
                Align2::LEFT_BOTTOM,
                text,
                FontId::proportional(14.0),
                stroke.color,
            );
        }
    }
}

fn paint_fill(painter: &Painter, primitive: &ProtocolDisplayPrimitive, fill: Color32) {
    match primitive {
        ProtocolDisplayPrimitive::Point { position, radius }
        | ProtocolDisplayPrimitive::Circle {
            center: position,
            radius,
        } => {
            painter.circle_filled(point(*position), *radius, fill);
        }
        ProtocolDisplayPrimitive::Polyline { vertices, closed } if *closed => {
            painter.add(egui::Shape::convex_polygon(
                vertices.iter().copied().map(point).collect(),
                fill,
                Stroke::NONE,
            ));
        }
        ProtocolDisplayPrimitive::Aabb {
            left,
            top,
            right,
            bottom,
        } => {
            painter.rect_filled(
                Rect::from_min_max(Pos2::new(*left, *top), Pos2::new(*right, *bottom)),
                0.0,
                fill,
            );
        }
        ProtocolDisplayPrimitive::Segment { .. }
        | ProtocolDisplayPrimitive::Polyline { .. }
        | ProtocolDisplayPrimitive::TransformAxes { .. }
        | ProtocolDisplayPrimitive::Arrow { .. }
        | ProtocolDisplayPrimitive::Label { .. } => {}
    }
}

fn paint_arrow(
    painter: &Painter,
    start: ProtocolScreenPoint,
    end: ProtocolScreenPoint,
    stroke: Stroke,
) {
    let start = point(start);
    let end = point(end);
    painter.line_segment([start, end], stroke);
    let delta = end - start;
    let length = delta.length();
    if length <= f32::EPSILON {
        return;
    }
    let direction = delta / length;
    let perpendicular = Vec2::new(-direction.y, direction.x);
    for sign in [-1.0, 1.0] {
        let wing = end - direction * 8.0 + perpendicular * (sign * 4.0);
        painter.line_segment([end, wing], stroke);
    }
}

fn primitive_comparison_state(
    comparison: &ComparisonModel,
    key: &DebugPrimitiveKey,
) -> ComparisonState {
    comparison
        .entries()
        .iter()
        .filter(|entry| entry.maybe_primitive_key() == Some(key))
        .map(liquidfun_differential::ComparisonEntry::state)
        .max_by_key(|state| match state {
            ComparisonState::ExactMatch => 0,
            ComparisonState::WithinPolicy => 1,
            ComparisonState::RustOnly | ComparisonState::OracleOnly => 2,
            ComparisonState::PhysicsMismatch => 3,
        })
        .unwrap_or(ComparisonState::ExactMatch)
}

const fn should_skip(state: ComparisonState, backend: ProtocolComparisonBackend) -> bool {
    matches!(
        (state, backend),
        (ComparisonState::OracleOnly, ProtocolComparisonBackend::Rust)
            | (ComparisonState::RustOnly, ProtocolComparisonBackend::Oracle)
    )
}

fn comparison_style(
    original: ProtocolScreenStyle,
    state: ComparisonState,
    backend: ProtocolComparisonBackend,
) -> ProtocolScreenStyle {
    if state == ComparisonState::ExactMatch {
        return ProtocolScreenStyle {
            stroke: scaled_alpha(original.stroke),
            stroke_width: original.stroke_width,
            maybe_fill: original.maybe_fill.map(scaled_alpha),
        };
    }
    let tint = match backend {
        ProtocolComparisonBackend::Rust => RUST_COMPARISON,
        ProtocolComparisonBackend::Oracle => ORACLE_COMPARISON,
    };
    ProtocolScreenStyle {
        stroke: tint.to_array(),
        stroke_width: original.stroke_width.max(2.0),
        maybe_fill: None,
    }
}

fn scaled_alpha(mut components: [u8; 4]) -> [u8; 4] {
    let scaled = u16::from(components[3]) * OVERLAY_OPACITY_PERCENT / 100;
    components[3] = u8::try_from(scaled).unwrap_or(u8::MAX);
    components
}

fn protocol_viewport(
    rect: Rect,
    (center_x, center_y, pixels_per_meter): (f32, f32, f32),
) -> Option<ProtocolViewport> {
    ProtocolViewport::new(
        rect.min.x,
        rect.min.y,
        rect.width(),
        rect.height(),
        center_x,
        center_y,
        pixels_per_meter,
    )
}

const fn point(value: ProtocolScreenPoint) -> Pos2 {
    Pos2::new(value.x, value.y)
}

const fn color(components: [u8; 4]) -> Color32 {
    Color32::from_rgba_premultiplied(components[0], components[1], components[2], components[3])
}

fn save_screenshot(image: &egui::ColorImage) -> Result<PathBuf, String> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let relative = Path::new("target/testbed/interactive.png");
    let confined =
        DiagnosticScreenshotPath::new(&workspace, relative).map_err(|error| error.to_string())?;
    let destination = workspace.join(confined.relative());
    let parent = destination
        .parent()
        .ok_or_else(|| "screenshot destination has no parent".to_owned())?;
    fs::create_dir_all(parent)
        .map_err(|_| "screenshot directory could not be created".to_owned())?;
    let width =
        u32::try_from(image.size[0]).map_err(|_| "screenshot width is oversized".to_owned())?;
    let height =
        u32::try_from(image.size[1]).map_err(|_| "screenshot height is oversized".to_owned())?;
    let size = tiny_skia::IntSize::from_wh(width, height)
        .ok_or_else(|| "screenshot dimensions are invalid".to_owned())?;
    let bytes = image
        .pixels
        .iter()
        .flat_map(Color32::to_array)
        .collect::<Vec<_>>();
    let pixmap = tiny_skia::Pixmap::from_vec(bytes, size)
        .ok_or_else(|| "screenshot pixels are invalid".to_owned())?;
    let png = pixmap
        .encode_png()
        .map_err(|_| "screenshot PNG encoding failed".to_owned())?;
    fs::write(&destination, png).map_err(|_| "screenshot could not be written".to_owned())?;
    Ok(confined.relative().to_path_buf())
}

fn load_oracle_checkpoint(path: PathBuf) -> Result<CanonicalCheckpoint, String> {
    let limits = HarnessLimits::phase2_default_v1();
    let path_metadata = fs::symlink_metadata(&path)
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

const fn state_color(maybe_comparison: Option<ComparisonState>) -> Color32 {
    match maybe_comparison {
        Some(
            ComparisonState::PhysicsMismatch
            | ComparisonState::RustOnly
            | ComparisonState::OracleOnly,
        ) => ERROR,
        Some(ComparisonState::WithinPolicy) => Color32::from_rgb(210, 153, 34),
        Some(ComparisonState::ExactMatch) => Color32::from_rgb(63, 185, 80),
        None => MUTED,
    }
}

fn count_text(maybe_count: Option<u64>) -> String {
    maybe_count.map_or_else(|| "—".to_owned(), |count| count.to_string())
}

fn shorten(value: &str, maximum: usize) -> &str {
    value.get(..maximum).unwrap_or(value)
}

const fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

const fn setting_label(field: SettingsField) -> &'static str {
    match field {
        SettingsField::Timestep => "Timestep seconds",
        SettingsField::VelocityIterations => "Velocity iterations",
        SettingsField::PositionIterations => "Position iterations",
        SettingsField::ParticleIterations => "Particle iterations",
    }
}

fn setting_index(field: SettingsField) -> usize {
    match field {
        SettingsField::Timestep => 0,
        SettingsField::VelocityIterations => 1,
        SettingsField::PositionIterations => 2,
        SettingsField::ParticleIterations => 3,
    }
}

fn settings_drafts(settings: &SettingsEditor) -> [String; 4] {
    SETTINGS_FIELDS.map(|field| settings.text(field).to_owned())
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

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "finite logical window dimensions are clamped before conversion"
)]
fn bounded_dimension(value: f32) -> u32 {
    value.clamp(1.0, 16_384.0).round() as u32
}

fn bound_message(message: &str) -> String {
    message.chars().take(160).collect()
}

fn bounded_error(error: impl std::fmt::Display) -> String {
    bound_message(&error.to_string())
}

#[cfg(not(test))]
fn main() -> eframe::Result {
    let maybe_app = parse_args().and_then(DesktopApp::new);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("liquidfun-rs semantic testbed")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([480.0, 360.0]),
        multisampling: 4,
        ..Default::default()
    };
    eframe::run_native(
        "liquidfun-rs semantic testbed",
        options,
        Box::new(move |_creation_context| {
            let app = maybe_app.map_err(std::io::Error::other)?;
            Ok(Box::new(app))
        }),
    )
}
