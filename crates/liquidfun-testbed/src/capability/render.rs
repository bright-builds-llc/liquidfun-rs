//! Deterministic offscreen capability capture through the replacement renderer contract.

use std::fs;
use std::path::Path;

use liquidfun_differential::{ComparisonModel, SessionController};

use super::fixture::FixtureSnapshot;
use super::passive::CapabilityBackend;
use super::report::CapabilityArtifact;
use super::{CapabilityError, hex_sha256, reject_link_file};
use crate::renderer::image::TinySkiaImageRenderer;
use crate::renderer::{
    Circle, DrawCommand, ImageRenderer, Line, LogicalPoint, LogicalSize, PhysicalSize,
    PresentationFrame, Rectangle, RenderedPixels, RgbaColor, Stroke,
};

const BACKGROUND: RgbaColor = RgbaColor::new(13, 17, 23, 255);
const PANEL: RgbaColor = RgbaColor::new(22, 27, 34, 255);
const TEXT: RgbaColor = RgbaColor::new(201, 209, 217, 255);
const MUTED: RgbaColor = RgbaColor::new(139, 152, 166, 255);
const ACCENT: RgbaColor = RgbaColor::new(88, 166, 255, 255);
const RUST: RgbaColor = RgbaColor::new(56, 209, 158, 255);
const ORACLE: RgbaColor = RgbaColor::new(191, 115, 255, 89);
const WARNING: RgbaColor = RgbaColor::new(242, 184, 64, 255);
const LOGICAL_WIDTH: u16 = 640;
const LOGICAL_HEIGHT: u16 = 480;

const FRAME_SIZES: [(u16, u16, &str); 3] = [
    (640, 480, "replacement-capability-640x480.png"),
    (800, 600, "replacement-capability-800x600.png"),
    (1280, 960, "replacement-capability-1280x960.png"),
];

pub(super) struct RenderedEvidence {
    pub(super) artifacts: Vec<CapabilityArtifact>,
    pub(super) minimum_width: u16,
    pub(super) minimum_height: u16,
    pub(super) maximum_dpi_scale: u16,
    pub(super) resize_width: u16,
    pub(super) resize_height: u16,
    pub(super) non_background_pixels_minimum: usize,
    pub(super) distinct_particle_colors: usize,
    pub(super) dense_text_rows: usize,
    pub(super) focus_ring_pixels: usize,
    pub(super) minimum_text_contrast_ratio: f32,
    pub(super) minimum_control_target_pixels: u16,
    pub(super) keyboard_bindings: usize,
    pub(super) contact_points: usize,
    pub(super) contact_normals: usize,
    pub(super) particle_contacts: usize,
    pub(super) broad_phase_aabbs: usize,
    pub(super) profile_names: usize,
    pub(super) overlay_pairs: usize,
    pub(super) side_by_side_panels: usize,
    pub(super) semantic_capture_acknowledgements: usize,
    pub(super) diagnostic_disclaimer_lines: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct SemanticDrawEvidence {
    particle_colors: [bool; 4],
    dense_text_rows: usize,
    focus_ring_pixels: usize,
    minimum_control_target_pixels: u16,
    contact_points: usize,
    contact_normals: usize,
    particle_contacts: usize,
    broad_phase_aabbs: usize,
    profile_names: usize,
    overlay_pairs: usize,
    side_by_side_panels: usize,
    semantic_capture_acknowledgements: usize,
    diagnostic_disclaimer_lines: usize,
}

impl SemanticDrawEvidence {
    fn minimum(self, other: Self) -> Self {
        Self {
            particle_colors: std::array::from_fn(|index| {
                self.particle_colors[index] && other.particle_colors[index]
            }),
            dense_text_rows: self.dense_text_rows.min(other.dense_text_rows),
            focus_ring_pixels: self.focus_ring_pixels.min(other.focus_ring_pixels),
            minimum_control_target_pixels: self
                .minimum_control_target_pixels
                .min(other.minimum_control_target_pixels),
            contact_points: self.contact_points.min(other.contact_points),
            contact_normals: self.contact_normals.min(other.contact_normals),
            particle_contacts: self.particle_contacts.min(other.particle_contacts),
            broad_phase_aabbs: self.broad_phase_aabbs.min(other.broad_phase_aabbs),
            profile_names: self.profile_names.min(other.profile_names),
            overlay_pairs: self.overlay_pairs.min(other.overlay_pairs),
            side_by_side_panels: self.side_by_side_panels.min(other.side_by_side_panels),
            semantic_capture_acknowledgements: self
                .semantic_capture_acknowledgements
                .min(other.semantic_capture_acknowledgements),
            diagnostic_disclaimer_lines: self
                .diagnostic_disclaimer_lines
                .min(other.diagnostic_disclaimer_lines),
        }
    }

    fn distinct_particle_colors(self) -> usize {
        self.particle_colors
            .into_iter()
            .filter(|seen| *seen)
            .count()
    }
}

pub(super) fn render_capability_frames(
    fixture: &FixtureSnapshot,
    controller: &SessionController<CapabilityBackend>,
    comparison: &ComparisonModel,
    keyboard_bindings: usize,
    output: &Path,
) -> Result<RenderedEvidence, CapabilityError> {
    let session_state = format!("SESSION {:?}", controller.state()).to_ascii_uppercase();
    let comparison_state = format!("COMPARE {:?}", comparison.state()).to_ascii_uppercase();
    let mut artifacts = Vec::with_capacity(FRAME_SIZES.len());
    let mut non_background_pixels_minimum = usize::MAX;
    let mut minimum_width = u16::MAX;
    let mut minimum_height = u16::MAX;
    let mut maximum_dpi_scale = 0_u16;
    let mut resize_width = 0_u16;
    let mut resize_height = 0_u16;
    let mut maybe_semantic_evidence: Option<SemanticDrawEvidence> = None;
    for (width, height, name) in FRAME_SIZES {
        let mut raster = RendererRaster::default();
        let semantic_evidence = draw_capability_scene(
            &mut raster,
            fixture,
            &session_state,
            &comparison_state,
            true,
        );
        maybe_semantic_evidence = Some(match maybe_semantic_evidence {
            Some(evidence) => evidence.minimum(semantic_evidence),
            None => semantic_evidence,
        });
        let pixels = raster.capture(width, height)?;
        non_background_pixels_minimum =
            non_background_pixels_minimum.min(non_background_pixels(&pixels));
        let physical_size = pixels.size();
        let physical_width =
            u16::try_from(physical_size.width()).map_err(|_| CapabilityError::CapabilityFailed)?;
        let physical_height =
            u16::try_from(physical_size.height()).map_err(|_| CapabilityError::CapabilityFailed)?;
        minimum_width = minimum_width.min(physical_width);
        minimum_height = minimum_height.min(physical_height);
        maximum_dpi_scale = maximum_dpi_scale.max(physical_width / LOGICAL_WIDTH);
        if width == 800 && height == 600 {
            resize_width = physical_width;
            resize_height = physical_height;
        }
        artifacts.push(export_image(&pixels, output, name)?);
    }
    let Some(semantic_evidence) = maybe_semantic_evidence else {
        return Err(CapabilityError::CapabilityFailed);
    };
    Ok(RenderedEvidence {
        artifacts,
        minimum_width,
        minimum_height,
        maximum_dpi_scale,
        resize_width,
        resize_height,
        non_background_pixels_minimum,
        distinct_particle_colors: semantic_evidence.distinct_particle_colors(),
        dense_text_rows: semantic_evidence.dense_text_rows,
        focus_ring_pixels: semantic_evidence.focus_ring_pixels,
        minimum_text_contrast_ratio: contrast_ratio(TEXT, BACKGROUND),
        minimum_control_target_pixels: semantic_evidence.minimum_control_target_pixels,
        keyboard_bindings,
        contact_points: semantic_evidence.contact_points,
        contact_normals: semantic_evidence.contact_normals,
        particle_contacts: semantic_evidence.particle_contacts,
        broad_phase_aabbs: semantic_evidence.broad_phase_aabbs,
        profile_names: semantic_evidence.profile_names,
        overlay_pairs: semantic_evidence.overlay_pairs,
        side_by_side_panels: semantic_evidence.side_by_side_panels,
        semantic_capture_acknowledgements: semantic_evidence.semantic_capture_acknowledgements,
        diagnostic_disclaimer_lines: semantic_evidence.diagnostic_disclaimer_lines,
    })
}

fn export_image(
    pixels: &RenderedPixels,
    output: &Path,
    name: &str,
) -> Result<CapabilityArtifact, CapabilityError> {
    let path = output.join(name);
    reject_link_file(&path)?;
    fs::write(&path, pixels.png_bytes()).map_err(|_| CapabilityError::Filesystem)?;
    let metadata = fs::symlink_metadata(&path).map_err(|_| CapabilityError::Filesystem)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(CapabilityError::InvalidOutputPath);
    }
    let bytes = fs::read(&path).map_err(|_| CapabilityError::Filesystem)?;
    if bytes.len() as u64 != metadata.len() || bytes.len() < 1_024 {
        return Err(CapabilityError::Filesystem);
    }
    Ok(CapabilityArtifact::new(
        name.to_owned(),
        hex_sha256(&bytes),
        metadata.len(),
        u16::try_from(pixels.size().width()).map_err(|_| CapabilityError::CapabilityFailed)?,
        u16::try_from(pixels.size().height()).map_err(|_| CapabilityError::CapabilityFailed)?,
        true,
    ))
}

#[derive(Debug, Clone, Copy)]
enum RasterCommand {
    Rectangle {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        color: RgbaColor,
    },
    Line {
        start: (i32, i32),
        end: (i32, i32),
        color: RgbaColor,
        width: i32,
    },
    Circle {
        center: (i32, i32),
        radius: i32,
        color: RgbaColor,
    },
}

#[derive(Debug, Default)]
struct RendererRaster {
    commands: Vec<RasterCommand>,
}

impl RendererRaster {
    fn rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: RgbaColor) {
        self.commands.push(RasterCommand::Rectangle {
            x,
            y,
            width,
            height,
            color,
        });
    }

    fn outline(&mut self, x: i32, y: i32, width: i32, height: i32, color: RgbaColor, line: i32) {
        self.rect(x, y, width, line, color);
        self.rect(x, y + height - line, width, line, color);
        self.rect(x, y, line, height, color);
        self.rect(x + width - line, y, line, height, color);
    }

    fn line(&mut self, start: (i32, i32), end: (i32, i32), color: RgbaColor, width: i32) {
        self.commands.push(RasterCommand::Line {
            start,
            end,
            color,
            width,
        });
    }

    fn dashed_line(&mut self, start: (i32, i32), end: (i32, i32), color: RgbaColor) {
        let segments = 8;
        for index in 0..segments {
            if index % 2 == 0 {
                let a = interpolate(start, end, index, segments);
                let b = interpolate(start, end, index + 1, segments);
                self.line(a, b, color, 1);
            }
        }
    }

    fn circle(&mut self, center: (i32, i32), radius: i32, color: RgbaColor) {
        self.commands.push(RasterCommand::Circle {
            center,
            radius,
            color,
        });
    }

    fn text(&mut self, x: i32, y: i32, value: &str, color: RgbaColor, size: i32) {
        let mut cursor = x;
        for character in value.chars() {
            let rows = glyph(character.to_ascii_uppercase());
            for (row, bits) in rows.iter().enumerate() {
                for column in 0..5 {
                    if bits & (1 << (4 - column)) != 0 {
                        let Ok(row) = i32::try_from(row) else {
                            continue;
                        };
                        self.rect(cursor + column * size, y + row * size, size, size, color);
                    }
                }
            }
            cursor += 6 * size;
        }
    }

    fn capture(self, width: u16, height: u16) -> Result<RenderedPixels, CapabilityError> {
        let commands = self
            .commands
            .into_iter()
            .map(renderer_command)
            .collect::<Result<Vec<_>, _>>()?;
        let logical_size = LogicalSize::new(f32::from(LOGICAL_WIDTH), f32::from(LOGICAL_HEIGHT))
            .map_err(|_| CapabilityError::CapabilityFailed)?;
        let physical_size = PhysicalSize::new(u32::from(width), u32::from(height))
            .map_err(|_| CapabilityError::CapabilityFailed)?;
        let presentation = PresentationFrame::new(logical_size, BACKGROUND, commands);
        TinySkiaImageRenderer
            .capture(physical_size, presentation)
            .map_err(|_| CapabilityError::CapabilityFailed)
    }
}

fn renderer_command(command: RasterCommand) -> Result<DrawCommand, CapabilityError> {
    match command {
        RasterCommand::Rectangle {
            x,
            y,
            width,
            height,
            color,
        } => {
            let origin = renderer_point((x, y))?;
            let size = LogicalSize::new(width as f32, height as f32)
                .map_err(|_| CapabilityError::CapabilityFailed)?;
            Ok(DrawCommand::FillRectangle(Rectangle::new(
                origin, size, color,
            )))
        }
        RasterCommand::Line {
            start,
            end,
            color,
            width,
        } => {
            let stroke =
                Stroke::new(color, width as f32).map_err(|_| CapabilityError::CapabilityFailed)?;
            Ok(DrawCommand::StrokeLine(Line::new(
                renderer_point(start)?,
                renderer_point(end)?,
                stroke,
            )))
        }
        RasterCommand::Circle {
            center,
            radius,
            color,
        } => Ok(DrawCommand::FillCircle(
            Circle::new(renderer_point(center)?, radius as f32, color)
                .map_err(|_| CapabilityError::CapabilityFailed)?,
        )),
    }
}

fn renderer_point(point: (i32, i32)) -> Result<LogicalPoint, CapabilityError> {
    LogicalPoint::new(point.0 as f32, point.1 as f32).map_err(|_| CapabilityError::CapabilityFailed)
}

fn non_background_pixels(pixels: &RenderedPixels) -> usize {
    let background = BACKGROUND.channels();
    pixels
        .rgba_bytes()
        .chunks_exact(background.len())
        .filter(|pixel| *pixel != background)
        .count()
}

fn draw_capability_scene(
    raster: &mut RendererRaster,
    fixture: &FixtureSnapshot,
    session_state: &str,
    comparison_state: &str,
    emit_contact_normals: bool,
) -> SemanticDrawEvidence {
    let mut evidence = SemanticDrawEvidence::default();
    raster.rect(0, 0, 640, 40, PANEL);
    raster.text(12, 10, "LIQUIDFUN TESTBED CAPABILITY", TEXT, 2);
    raster.outline(390, 3, 94, 34, ACCENT, 2);
    raster.text(400, 14, "RUN [SPACE]", TEXT, 1);
    raster.text(494, 14, "PAUSE P STEP N RESTART R", MUTED, 1);

    raster.rect(12, 48, 394, 290, PANEL);
    raster.text(20, 56, "DIAGNOSTIC CAPABILITY FRAME", MUTED, 1);
    draw_contacts(raster, &mut evidence, emit_contact_normals);
    draw_particles(raster, &mut evidence);
    draw_aabbs(raster, &mut evidence);
    draw_differences(raster, &mut evidence);

    raster.rect(414, 48, 214, 416, PANEL);
    raster.text(424, 58, "INSPECTOR", TEXT, 2);
    draw_profiles(
        raster,
        fixture,
        session_state,
        comparison_state,
        &mut evidence,
    );
    raster.outline(424, 342, 190, 44, ACCENT, 2);
    evidence.focus_ring_pixels = 2;
    evidence.minimum_control_target_pixels = 44;
    raster.text(434, 358, "FOCUS CONTACT.NORMAL.2", TEXT, 1);
    raster.text(424, 400, "CHECKPOINT CAPTURED AT STEP 16", RUST, 1);
    evidence.semantic_capture_acknowledgements += 1;
    raster.text(424, 416, "SCREENSHOT SAVED. DIAGNOSTIC ONLY", WARNING, 1);
    evidence.diagnostic_disclaimer_lines += 1;
    raster.text(424, 432, "PIXELS DO NOT PROVE COMPATIBILITY", WARNING, 1);
    evidence.diagnostic_disclaimer_lines += 1;

    raster.rect(12, 346, 394, 118, PANEL);
    raster.text(20, 354, "SIDE BY SIDE DIFFERENCE", TEXT, 1);
    raster.outline(22, 370, 174, 80, RUST, 1);
    evidence.side_by_side_panels += 1;
    raster.outline(214, 370, 174, 80, ORACLE, 1);
    evidence.side_by_side_panels += 1;
    raster.text(30, 378, "R RUST", RUST, 1);
    raster.text(222, 378, "O ORACLE", ORACLE, 1);
    for offset in [0, 22, 44] {
        raster.line((40 + offset, 420), (68 + offset, 394), RUST, 1);
        raster.dashed_line((232 + offset, 420), (262 + offset, 392), ORACLE);
    }
    evidence
}

fn draw_contacts(
    raster: &mut RendererRaster,
    evidence: &mut SemanticDrawEvidence,
    emit_normals: bool,
) {
    raster.text(24, 78, "CONTACTS AND NORMALS", TEXT, 1);
    for (x, y) in [(52, 116), (86, 126), (120, 112)] {
        raster.circle((x, y), 4, RUST);
        evidence.contact_points += 1;
        if !emit_normals {
            continue;
        }
        raster.line((x, y), (x + 10, y - 18), ACCENT, 1);
        raster.line((x + 10, y - 18), (x + 4, y - 15), ACCENT, 1);
        raster.line((x + 10, y - 18), (x + 9, y - 11), ACCENT, 1);
        evidence.contact_normals += 1;
    }
}

fn draw_particles(raster: &mut RendererRaster, evidence: &mut SemanticDrawEvidence) {
    raster.text(24, 150, "PARTICLES COLORS CONTACTS", TEXT, 1);
    let colors = [RUST, ACCENT, WARNING, ORACLE];
    let mut centers = Vec::with_capacity(12);
    for (row, y) in [176, 198, 220].into_iter().enumerate() {
        for (column, x) in [40, 64, 88, 112].into_iter().enumerate() {
            let center = (x, y);
            centers.push(center);
            let color_index = (row * 4 + column) % colors.len();
            raster.circle(center, 6, colors[color_index]);
            evidence.particle_colors[color_index] = true;
        }
    }
    for (left, right) in [(0, 1), (1, 5), (5, 6), (6, 10), (10, 11), (3, 7)] {
        raster.line(centers[left], centers[right], MUTED, 1);
        evidence.particle_contacts += 1;
    }
}

fn draw_aabbs(raster: &mut RendererRaster, evidence: &mut SemanticDrawEvidence) {
    raster.text(164, 78, "BROAD PHASE AABBS", TEXT, 1);
    for (x, y, width, height) in [
        (166, 96, 42, 30),
        (218, 102, 54, 44),
        (178, 154, 74, 36),
        (266, 148, 50, 58),
    ] {
        raster.outline(x, y, width, height, WARNING, 1);
        evidence.broad_phase_aabbs += 1;
    }
}

fn draw_differences(raster: &mut RendererRaster, evidence: &mut SemanticDrawEvidence) {
    raster.text(164, 224, "SYNCHRONIZED OVERLAY R O", TEXT, 1);
    for offset in [0, 48, 96] {
        raster.line((176 + offset, 282), (204 + offset, 248), RUST, 1);
        raster.dashed_line((178 + offset, 282), (208 + offset, 246), ORACLE);
        evidence.overlay_pairs += 1;
    }
    raster.circle((304, 268), 14, ACCENT);
    raster.circle((304, 268), 10, PANEL);
    raster.text(324, 264, "FOCUS BODY-1 CONTACT-2", ACCENT, 1);
}

fn draw_profiles(
    raster: &mut RendererRaster,
    fixture: &FixtureSnapshot,
    session_state: &str,
    comparison_state: &str,
    evidence: &mut SemanticDrawEvidence,
) {
    let rows = [
        "PROFILE NAMES",
        "COLLIDE",
        "SOLVE",
        "PARTICLES",
        "BROAD PHASE",
        "CAPTURE",
        session_state,
        comparison_state,
        "MODE OVERLAY",
        "MODE SIDE BY SIDE",
        "KEYBOARD FOCUS VISIBLE",
        "TARGET 44 PX",
        "DPI 100 PERCENT",
        "RESIZE RESPONSIVE",
        "SOURCE GITHUB",
        "ORACLE UNAVAILABLE",
    ];
    for (index, row) in rows.iter().enumerate() {
        let y = 88 + i32::try_from(index).unwrap_or(0) * 14;
        raster.text(424, y, row, if index < 6 { TEXT } else { MUTED }, 1);
        evidence.dense_text_rows += 1;
        if (1..=5).contains(&index) {
            evidence.profile_names += 1;
        }
    }
    let cases = format!("FIXTURE {} CASES", fixture.case_ids.len());
    raster.text(424, 316, &cases, MUTED, 1);
}

fn interpolate(start: (i32, i32), end: (i32, i32), index: i32, count: i32) -> (i32, i32) {
    (
        start.0 + (end.0 - start.0) * index / count,
        start.1 + (end.1 - start.1) * index / count,
    )
}

fn contrast_ratio(foreground: RgbaColor, background: RgbaColor) -> f32 {
    let light = relative_luminance(foreground);
    let dark = relative_luminance(background);
    (light.max(dark) + 0.05) / (light.min(dark) + 0.05)
}

fn relative_luminance(color: RgbaColor) -> f32 {
    let [red, green, blue, _alpha] = color.channels();
    0.2126 * channel_luminance(f32::from(red) / 255.0)
        + 0.7152 * channel_luminance(f32::from(green) / 255.0)
        + 0.0722 * channel_luminance(f32::from(blue) / 255.0)
}

fn channel_luminance(value: f32) -> f32 {
    if value <= 0.039_28 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
pub(super) fn rendered_evidence_with_contact_normals(
    emit_contact_normals: bool,
) -> RenderedEvidence {
    let fixture = FixtureSnapshot {
        sha256: "0".repeat(64),
        profile: "phase11-v1".to_owned(),
        upstream_revision: "0".repeat(40),
        case_ids: vec!["test-case".to_owned()],
        families: vec!["rigid".to_owned()],
        verified_artifacts: 1,
    };
    let mut raster = RendererRaster::default();
    let semantic_evidence = draw_capability_scene(
        &mut raster,
        &fixture,
        "SESSION READY",
        "COMPARE EXACT",
        emit_contact_normals,
    );
    let artifacts = FRAME_SIZES
        .iter()
        .map(|(width, height, name)| {
            CapabilityArtifact::new(
                (*name).to_owned(),
                "0".repeat(64),
                1_024,
                *width,
                *height,
                true,
            )
        })
        .collect();
    RenderedEvidence {
        artifacts,
        minimum_width: 640,
        minimum_height: 480,
        maximum_dpi_scale: 2,
        resize_width: 800,
        resize_height: 600,
        non_background_pixels_minimum: raster.commands.len(),
        distinct_particle_colors: semantic_evidence.distinct_particle_colors(),
        dense_text_rows: semantic_evidence.dense_text_rows,
        focus_ring_pixels: semantic_evidence.focus_ring_pixels,
        minimum_text_contrast_ratio: contrast_ratio(TEXT, BACKGROUND),
        minimum_control_target_pixels: semantic_evidence.minimum_control_target_pixels,
        keyboard_bindings: 6,
        contact_points: semantic_evidence.contact_points,
        contact_normals: semantic_evidence.contact_normals,
        particle_contacts: semantic_evidence.particle_contacts,
        broad_phase_aabbs: semantic_evidence.broad_phase_aabbs,
        profile_names: semantic_evidence.profile_names,
        overlay_pairs: semantic_evidence.overlay_pairs,
        side_by_side_panels: semantic_evidence.side_by_side_panels,
        semantic_capture_acknowledgements: semantic_evidence.semantic_capture_acknowledgements,
        diagnostic_disclaimer_lines: semantic_evidence.diagnostic_disclaimer_lines,
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "the closed 5x7 capability font stays auditable"
)]
const fn glyph(character: char) -> [u8; 7] {
    match character {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 15],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [14, 4, 4, 4, 4, 4, 14],
        'J' => [7, 2, 2, 2, 18, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        '.' => [0, 0, 0, 0, 0, 12, 12],
        '[' => [14, 8, 8, 8, 8, 8, 14],
        ']' => [14, 2, 2, 2, 2, 2, 14],
        ':' => [0, 12, 12, 0, 12, 12, 0],
        '/' => [1, 2, 2, 4, 8, 8, 16],
        _ => [0; 7],
    }
}
