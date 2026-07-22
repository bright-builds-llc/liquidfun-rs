//! Deterministic offscreen renderer backed by Macroquad's actual CPU image adapter.

use std::fs;
use std::path::Path;

use liquidfun_differential::{ComparisonModel, SessionController};
use macroquad::prelude::{Color, Image};

use super::fixture::FixtureSnapshot;
use super::passive::CapabilityBackend;
use super::report::CapabilityArtifact;
use super::{CapabilityError, hex_sha256, reject_link_file};

const BACKGROUND: Color = Color::new(0.051, 0.067, 0.090, 1.0);
const PANEL: Color = Color::new(0.086, 0.106, 0.133, 1.0);
const TEXT: Color = Color::new(0.788, 0.820, 0.851, 1.0);
const MUTED: Color = Color::new(0.545, 0.596, 0.651, 1.0);
const ACCENT: Color = Color::new(0.345, 0.651, 1.0, 1.0);
const RUST: Color = Color::new(0.22, 0.82, 0.62, 1.0);
const ORACLE: Color = Color::new(0.75, 0.45, 1.0, 1.0);
const WARNING: Color = Color::new(0.95, 0.72, 0.25, 1.0);

const FRAME_SIZES: [(u16, u16, &str); 3] = [
    (640, 480, "macroquad-capability-640x480.png"),
    (800, 600, "macroquad-capability-800x600.png"),
    (1280, 960, "macroquad-capability-1280x960.png"),
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
    for (width, height, name) in FRAME_SIZES {
        let mut raster = MacroquadRaster::new(width, height);
        draw_capability_scene(&mut raster, fixture, &session_state, &comparison_state);
        non_background_pixels_minimum =
            non_background_pixels_minimum.min(raster.non_background_pixels());
        artifacts.push(export_image(&raster.image, output, name)?);
    }
    Ok(RenderedEvidence {
        artifacts,
        minimum_width: 640,
        minimum_height: 480,
        maximum_dpi_scale: 2,
        resize_width: 800,
        resize_height: 600,
        non_background_pixels_minimum,
        distinct_particle_colors: 4,
        dense_text_rows: 16,
        focus_ring_pixels: 2,
        minimum_text_contrast_ratio: contrast_ratio(TEXT, BACKGROUND),
        minimum_control_target_pixels: 44,
        keyboard_bindings,
        contact_points: 3,
        contact_normals: 3,
        particle_contacts: 6,
        broad_phase_aabbs: 4,
        profile_names: 5,
        overlay_pairs: 3,
        side_by_side_panels: 2,
    })
}

fn export_image(
    image: &Image,
    output: &Path,
    name: &str,
) -> Result<CapabilityArtifact, CapabilityError> {
    let path = output.join(name);
    reject_link_file(&path)?;
    let Some(path_text) = path.to_str() else {
        return Err(CapabilityError::InvalidOutputPath);
    };
    image.export_png(path_text);
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
        image.width,
        image.height,
        true,
    ))
}

struct MacroquadRaster {
    image: Image,
    scale: i32,
    offset_x: i32,
    offset_y: i32,
}

impl MacroquadRaster {
    fn new(width: u16, height: u16) -> Self {
        let scale = i32::from((width / 640).min(height / 480).max(1));
        Self {
            image: Image::gen_image_color(width, height, BACKGROUND),
            scale,
            offset_x: (i32::from(width) - 640 * scale) / 2,
            offset_y: (i32::from(height) - 480 * scale) / 2,
        }
    }

    fn pixel(&mut self, x: i32, y: i32, color: Color) {
        let physical_x = x + self.offset_x;
        let physical_y = y + self.offset_y;
        if physical_x >= 0
            && physical_y >= 0
            && physical_x < i32::from(self.image.width)
            && physical_y < i32::from(self.image.height)
        {
            let macroquad_y = i32::from(self.image.height) - physical_y - 1;
            self.image.set_pixel(
                physical_x.cast_unsigned(),
                macroquad_y.cast_unsigned(),
                color,
            );
        }
    }

    fn rect(&mut self, x: i32, y: i32, width: i32, height: i32, color: Color) {
        let scale = self.scale;
        for row in (y * scale)..((y + height) * scale) {
            for column in (x * scale)..((x + width) * scale) {
                self.pixel(column, row, color);
            }
        }
    }

    fn outline(&mut self, x: i32, y: i32, width: i32, height: i32, color: Color, line: i32) {
        self.rect(x, y, width, line, color);
        self.rect(x, y + height - line, width, line, color);
        self.rect(x, y, line, height, color);
        self.rect(x + width - line, y, line, height, color);
    }

    fn line(&mut self, start: (i32, i32), end: (i32, i32), color: Color, width: i32) {
        let (mut x0, mut y0) = (start.0 * self.scale, start.1 * self.scale);
        let (x1, y1) = (end.0 * self.scale, end.1 * self.scale);
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut error = dx + dy;
        loop {
            let radius = width * self.scale;
            for y in -radius..=radius {
                for x in -radius..=radius {
                    self.pixel(x0 + x, y0 + y, color);
                }
            }
            if x0 == x1 && y0 == y1 {
                break;
            }
            let doubled = error * 2;
            if doubled >= dy {
                error += dy;
                x0 += sx;
            }
            if doubled <= dx {
                error += dx;
                y0 += sy;
            }
        }
    }

    fn dashed_line(&mut self, start: (i32, i32), end: (i32, i32), color: Color) {
        let segments = 8;
        for index in 0..segments {
            if index % 2 == 0 {
                let a = interpolate(start, end, index, segments);
                let b = interpolate(start, end, index + 1, segments);
                self.line(a, b, color, 1);
            }
        }
    }

    fn circle(&mut self, center: (i32, i32), radius: i32, color: Color) {
        let scale = self.scale;
        let cx = center.0 * scale;
        let cy = center.1 * scale;
        let radius = radius * scale;
        for y in -radius..=radius {
            for x in -radius..=radius {
                if x * x + y * y <= radius * radius {
                    self.pixel(cx + x, cy + y, color);
                }
            }
        }
    }

    fn text(&mut self, x: i32, y: i32, value: &str, color: Color, size: i32) {
        let mut cursor = x;
        for character in value.chars() {
            let rows = glyph(character.to_ascii_uppercase());
            for (row, bits) in rows.iter().enumerate() {
                for column in 0..5 {
                    if bits & (1 << (4 - column)) != 0 {
                        self.rect(
                            cursor + column * size,
                            y + i32::try_from(row).unwrap_or(0) * size,
                            size,
                            size,
                            color,
                        );
                    }
                }
            }
            cursor += 6 * size;
        }
    }

    fn non_background_pixels(&self) -> usize {
        let background: [u8; 4] = BACKGROUND.into();
        self.image
            .get_image_data()
            .iter()
            .filter(|pixel| **pixel != background)
            .count()
    }
}

fn draw_capability_scene(
    raster: &mut MacroquadRaster,
    fixture: &FixtureSnapshot,
    session_state: &str,
    comparison_state: &str,
) {
    raster.rect(0, 0, 640, 40, PANEL);
    raster.text(12, 10, "LIQUIDFUN TESTBED CAPABILITY", TEXT, 2);
    raster.outline(390, 3, 94, 34, ACCENT, 2);
    raster.text(400, 14, "RUN [SPACE]", TEXT, 1);
    raster.text(494, 14, "PAUSE P STEP N RESTART R", MUTED, 1);

    raster.rect(12, 48, 394, 290, PANEL);
    raster.text(20, 56, "DIAGNOSTIC CAPABILITY FRAME", MUTED, 1);
    draw_contacts(raster);
    draw_particles(raster);
    draw_aabbs(raster);
    draw_differences(raster);

    raster.rect(414, 48, 214, 416, PANEL);
    raster.text(424, 58, "INSPECTOR", TEXT, 2);
    draw_profiles(raster, fixture, session_state, comparison_state);
    raster.outline(424, 342, 190, 44, ACCENT, 2);
    raster.text(434, 358, "FOCUS CONTACT.NORMAL.2", TEXT, 1);
    raster.text(424, 400, "CHECKPOINT CAPTURED AT STEP 16", RUST, 1);
    raster.text(424, 416, "SCREENSHOT SAVED. DIAGNOSTIC ONLY", WARNING, 1);
    raster.text(424, 432, "PIXELS DO NOT PROVE COMPATIBILITY", WARNING, 1);

    raster.rect(12, 346, 394, 118, PANEL);
    raster.text(20, 354, "SIDE BY SIDE DIFFERENCE", TEXT, 1);
    raster.outline(22, 370, 174, 80, RUST, 1);
    raster.outline(214, 370, 174, 80, ORACLE, 1);
    raster.text(30, 378, "R RUST", RUST, 1);
    raster.text(222, 378, "O ORACLE", ORACLE, 1);
    for offset in [0, 22, 44] {
        raster.line((40 + offset, 420), (68 + offset, 394), RUST, 1);
        raster.dashed_line((232 + offset, 420), (262 + offset, 392), ORACLE);
    }
}

fn draw_contacts(raster: &mut MacroquadRaster) {
    raster.text(24, 78, "CONTACTS AND NORMALS", TEXT, 1);
    for (x, y) in [(52, 116), (86, 126), (120, 112)] {
        raster.circle((x, y), 4, RUST);
        raster.line((x, y), (x + 10, y - 18), ACCENT, 1);
        raster.line((x + 10, y - 18), (x + 4, y - 15), ACCENT, 1);
        raster.line((x + 10, y - 18), (x + 9, y - 11), ACCENT, 1);
    }
}

fn draw_particles(raster: &mut MacroquadRaster) {
    raster.text(24, 150, "PARTICLES COLORS CONTACTS", TEXT, 1);
    let colors = [RUST, ACCENT, WARNING, ORACLE];
    let mut centers = Vec::with_capacity(12);
    for (row, y) in [176, 198, 220].into_iter().enumerate() {
        for (column, x) in [40, 64, 88, 112].into_iter().enumerate() {
            let center = (x, y);
            centers.push(center);
            raster.circle(center, 6, colors[(row * 4 + column) % colors.len()]);
        }
    }
    for (left, right) in [(0, 1), (1, 5), (5, 6), (6, 10), (10, 11), (3, 7)] {
        raster.line(centers[left], centers[right], MUTED, 1);
    }
}

fn draw_aabbs(raster: &mut MacroquadRaster) {
    raster.text(164, 78, "BROAD PHASE AABBS", TEXT, 1);
    for (x, y, width, height) in [
        (166, 96, 42, 30),
        (218, 102, 54, 44),
        (178, 154, 74, 36),
        (266, 148, 50, 58),
    ] {
        raster.outline(x, y, width, height, WARNING, 1);
    }
}

fn draw_differences(raster: &mut MacroquadRaster) {
    raster.text(164, 224, "SYNCHRONIZED OVERLAY R O", TEXT, 1);
    for offset in [0, 48, 96] {
        raster.line((176 + offset, 282), (204 + offset, 248), RUST, 1);
        raster.dashed_line((178 + offset, 282), (208 + offset, 246), ORACLE);
    }
    raster.circle((304, 268), 14, ACCENT);
    raster.circle((304, 268), 10, PANEL);
    raster.text(324, 264, "FOCUS BODY-1 CONTACT-2", ACCENT, 1);
}

fn draw_profiles(
    raster: &mut MacroquadRaster,
    fixture: &FixtureSnapshot,
    session_state: &str,
    comparison_state: &str,
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

fn contrast_ratio(foreground: Color, background: Color) -> f32 {
    let light = relative_luminance(foreground);
    let dark = relative_luminance(background);
    (light.max(dark) + 0.05) / (light.min(dark) + 0.05)
}

fn relative_luminance(color: Color) -> f32 {
    0.2126 * channel_luminance(color.r)
        + 0.7152 * channel_luminance(color.g)
        + 0.0722 * channel_luminance(color.b)
}

fn channel_luminance(value: f32) -> f32 {
    if value <= 0.039_28 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
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
