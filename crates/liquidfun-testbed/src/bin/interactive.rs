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

include!("interactive/session.rs");
include!("interactive/presentation.rs");

impl DesktopApp {
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

include!("interactive/painting.rs");

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
