//! Semantic primitive viewport with presentation-only camera and stable selection.

#![allow(
    missing_docs,
    reason = "closed render-command variants are named by their primitive contract"
)]

use std::path::{Component, Path, PathBuf};

use super::SCREENSHOT_CLARIFICATION;
use super::overlays::OverlayState;
use liquidfun::{
    DebugLayer, DebugPrimitive, DebugPrimitiveKey, DebugPrimitiveMetadata, math::Vec2,
};

mod draw;
pub use draw::draw_frame;

pub const MODULE_NAME: &str = "viewport";
const MINIMUM_PIXELS_PER_METER: f32 = 5.0;
const MAXIMUM_PIXELS_PER_METER: f32 = 400.0;
const MAXIMUM_VIEWPORT_PRIMITIVES: usize = 100_000;
const MAXIMUM_POLYLINE_VERTICES: usize = 4_096;
const MAXIMUM_SCREENSHOT_PATH_BYTES: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenSize {
    pub width: f32,
    pub height: f32,
}

impl ScreenSize {
    #[must_use]
    pub const fn new(width: f32, height: f32) -> Option<Self> {
        if width.is_finite()
            && height.is_finite()
            && width > 0.0
            && height > 0.0
            && width <= 16_384.0
            && height <= 16_384.0
        {
            Some(Self { width, height })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenPoint {
    pub x: f32,
    pub y: f32,
}

impl ScreenPoint {
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Option<Self> {
        if x.is_finite() && y.is_finite() {
            Some(Self { x, y })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    center: Vec2,
    pixels_per_meter: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            center: Vec2::ZERO,
            pixels_per_meter: 50.0,
        }
    }
}

impl Camera {
    #[must_use]
    pub fn new(center: Vec2, pixels_per_meter: f32) -> Option<Self> {
        if !center.is_valid()
            || !pixels_per_meter.is_finite()
            || !(MINIMUM_PIXELS_PER_METER..=MAXIMUM_PIXELS_PER_METER).contains(&pixels_per_meter)
        {
            return None;
        }
        Some(Self {
            center,
            pixels_per_meter,
        })
    }

    #[must_use]
    pub const fn center(self) -> Vec2 {
        self.center
    }

    #[must_use]
    pub const fn pixels_per_meter(self) -> f32 {
        self.pixels_per_meter
    }

    #[must_use]
    pub fn zoom_percent(self) -> f32 {
        ((self.pixels_per_meter / 50.0) * 100.0)
            .round()
            .clamp(10.0, 800.0)
    }

    #[must_use]
    pub fn world_to_screen(self, world: Vec2, size: ScreenSize) -> ScreenPoint {
        ScreenPoint {
            x: size
                .width
                .mul_add(0.5, (world.x - self.center.x) * self.pixels_per_meter),
            y: size
                .height
                .mul_add(0.5, (self.center.y - world.y) * self.pixels_per_meter),
        }
    }

    #[must_use]
    pub fn screen_to_world(self, screen: ScreenPoint, size: ScreenSize) -> Vec2 {
        Vec2::new(
            self.center.x + (screen.x - size.width * 0.5) / self.pixels_per_meter,
            self.center.y - (screen.y - size.height * 0.5) / self.pixels_per_meter,
        )
    }

    /// Zooms around the pointer while preserving the world coordinate under it.
    pub fn zoom_about_pointer(&mut self, factor: f32, pointer: ScreenPoint, size: ScreenSize) {
        if !factor.is_finite() || factor <= 0.0 {
            return;
        }
        let world_before = self.screen_to_world(pointer, size);
        self.pixels_per_meter = (self.pixels_per_meter * factor)
            .clamp(MINIMUM_PIXELS_PER_METER, MAXIMUM_PIXELS_PER_METER);
        let offset_x = (pointer.x - size.width * 0.5) / self.pixels_per_meter;
        let offset_y = (pointer.y - size.height * 0.5) / self.pixels_per_meter;
        self.center = Vec2::new(world_before.x - offset_x, world_before.y + offset_y);
    }

    /// Pans from middle-drag or Shift+primary-drag pixels without a controller effect.
    pub fn pan_pixels(&mut self, delta: ScreenPoint) {
        self.center.x -= delta.x / self.pixels_per_meter;
        self.center.y += delta.y / self.pixels_per_meter;
    }

    /// Fits the declared scenario bounds for Home or empty-space double click.
    pub fn reset_to_bounds(&mut self, lower: Vec2, upper: Vec2, size: ScreenSize) {
        if !lower.is_valid() || !upper.is_valid() || lower.x > upper.x || lower.y > upper.y {
            return;
        }
        self.center = 0.5 * (lower + upper);
        let span_x = (upper.x - lower.x).max(0.01);
        let span_y = (upper.y - lower.y).max(0.01);
        self.pixels_per_meter = (0.9 * (size.width / span_x).min(size.height / span_y))
            .clamp(MINIMUM_PIXELS_PER_METER, MAXIMUM_PIXELS_PER_METER);
    }
}

/// One shared transform keeps side-by-side comparison viewports synchronized.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SynchronizedCamera {
    camera: Camera,
}

impl SynchronizedCamera {
    #[must_use]
    pub const fn rust_camera(self) -> Camera {
        self.camera
    }

    #[must_use]
    pub const fn oracle_camera(self) -> Camera {
        self.camera
    }

    pub const fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenStyle {
    pub stroke: [u8; 4],
    pub stroke_width: f32,
    pub maybe_fill: Option<[u8; 4]>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScreenPrimitive {
    Point {
        key: DebugPrimitiveKey,
        layer: DebugLayer,
        position: ScreenPoint,
        radius: f32,
        style: ScreenStyle,
    },
    Segment {
        key: DebugPrimitiveKey,
        layer: DebugLayer,
        start: ScreenPoint,
        end: ScreenPoint,
        style: ScreenStyle,
    },
    Arrow {
        key: DebugPrimitiveKey,
        layer: DebugLayer,
        start: ScreenPoint,
        end: ScreenPoint,
        style: ScreenStyle,
    },
    Polyline {
        key: DebugPrimitiveKey,
        layer: DebugLayer,
        vertices: Vec<ScreenPoint>,
        closed: bool,
        style: ScreenStyle,
    },
    Circle {
        key: DebugPrimitiveKey,
        layer: DebugLayer,
        center: ScreenPoint,
        radius: f32,
        style: ScreenStyle,
    },
    TransformAxes {
        key: DebugPrimitiveKey,
        layer: DebugLayer,
        origin: ScreenPoint,
        x_end: ScreenPoint,
        y_end: ScreenPoint,
        style: ScreenStyle,
    },
    Aabb {
        key: DebugPrimitiveKey,
        layer: DebugLayer,
        lower: ScreenPoint,
        upper: ScreenPoint,
        style: ScreenStyle,
    },
    Label {
        key: DebugPrimitiveKey,
        layer: DebugLayer,
        position: ScreenPoint,
        text: Box<str>,
        style: ScreenStyle,
    },
}

impl ScreenPrimitive {
    #[must_use]
    pub const fn key(&self) -> DebugPrimitiveKey {
        match self {
            Self::Point { key, .. }
            | Self::Segment { key, .. }
            | Self::Arrow { key, .. }
            | Self::Polyline { key, .. }
            | Self::Circle { key, .. }
            | Self::TransformAxes { key, .. }
            | Self::Aabb { key, .. }
            | Self::Label { key, .. } => *key,
        }
    }

    #[must_use]
    pub const fn layer(&self) -> DebugLayer {
        match self {
            Self::Point { layer, .. }
            | Self::Segment { layer, .. }
            | Self::Arrow { layer, .. }
            | Self::Polyline { layer, .. }
            | Self::Circle { layer, .. }
            | Self::TransformAxes { layer, .. }
            | Self::Aabb { layer, .. }
            | Self::Label { layer, .. } => *layer,
        }
    }

    fn highlight_selection(&mut self) {
        let style = match self {
            Self::Point { style, .. }
            | Self::Segment { style, .. }
            | Self::Arrow { style, .. }
            | Self::Polyline { style, .. }
            | Self::Circle { style, .. }
            | Self::TransformAxes { style, .. }
            | Self::Aabb { style, .. }
            | Self::Label { style, .. } => style,
        };
        style.stroke = [88, 166, 255, 255];
        style.stroke_width = style.stroke_width.max(2.0);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewportFrame {
    primitives: Vec<ScreenPrimitive>,
    maybe_selected: Option<DebugPrimitiveKey>,
    maybe_hovered: Option<DebugPrimitiveKey>,
    zoom_percent: f32,
}

impl ViewportFrame {
    #[must_use]
    pub fn primitives(&self) -> &[ScreenPrimitive] {
        &self.primitives
    }

    #[must_use]
    pub const fn maybe_selected(&self) -> Option<DebugPrimitiveKey> {
        self.maybe_selected
    }

    #[must_use]
    pub const fn maybe_hovered(&self) -> Option<DebugPrimitiveKey> {
        self.maybe_hovered
    }

    #[must_use]
    pub const fn zoom_percent(&self) -> f32 {
        self.zoom_percent
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ViewportError {
    #[error("viewport primitive limit exceeded")]
    PrimitiveLimit,
    #[error("viewport received non-finite or out-of-range semantic geometry")]
    InvalidGeometry,
    #[error("viewport received invalid bounded label text")]
    InvalidLabel,
    #[error("viewport primitive kind is unsupported")]
    UnsupportedPrimitive,
}

/// Mutable state remains selection, hover, and camera presentation only.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SemanticViewport {
    camera: Camera,
    maybe_selected: Option<DebugPrimitiveKey>,
    maybe_hovered: Option<DebugPrimitiveKey>,
    hover_elapsed_millis: u16,
}

impl SemanticViewport {
    #[must_use]
    pub const fn camera(&self) -> Camera {
        self.camera
    }

    pub const fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub const fn select(&mut self, maybe_key: Option<DebugPrimitiveKey>) {
        self.maybe_selected = maybe_key;
    }

    pub const fn hover(&mut self, maybe_key: Option<DebugPrimitiveKey>) {
        self.maybe_hovered = maybe_key;
        self.hover_elapsed_millis = 0;
    }

    /// Records bounded pointer dwell for the supplementary 400ms tooltip threshold.
    pub fn hover_for(&mut self, maybe_key: Option<DebugPrimitiveKey>, elapsed_millis: u32) {
        self.maybe_hovered = maybe_key;
        self.hover_elapsed_millis = u16::try_from(elapsed_millis).unwrap_or(u16::MAX);
    }

    #[must_use]
    pub const fn tooltip_visible(&self) -> bool {
        self.maybe_hovered.is_some() && self.hover_elapsed_millis >= 400
    }

    /// Converts owned semantic primitives to a bounded display list without controller effects.
    ///
    /// # Errors
    ///
    /// Returns a closed error for excessive, non-finite, unsupported, or unsafe semantic input.
    pub fn render_frame(
        &self,
        primitives: &[DebugPrimitive],
        overlays: OverlayState,
        size: ScreenSize,
    ) -> Result<ViewportFrame, ViewportError> {
        if primitives.len() > MAXIMUM_VIEWPORT_PRIMITIVES {
            return Err(ViewportError::PrimitiveLimit);
        }
        let projected = primitives
            .iter()
            .filter(|primitive| overlays.layer_visible(primitive.layer()))
            .map(|primitive| {
                let mut projected = project_primitive(primitive, self.camera, size)?;
                if self.maybe_selected == Some(projected.key()) {
                    projected.highlight_selection();
                }
                Ok(projected)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ViewportFrame {
            primitives: projected,
            maybe_selected: self.maybe_selected,
            maybe_hovered: self.maybe_hovered,
            zoom_percent: self.camera.zoom_percent(),
        })
    }
}

fn project_primitive(
    primitive: &DebugPrimitive,
    camera: Camera,
    size: ScreenSize,
) -> Result<ScreenPrimitive, ViewportError> {
    let metadata = primitive.metadata();
    let key = metadata.key();
    let layer = key.layer();
    let style = project_style(metadata, camera)?;
    let screen = match primitive {
        DebugPrimitive::Point {
            position, radius, ..
        } => ScreenPrimitive::Point {
            key,
            layer,
            position: project_point(*position, camera, size)?,
            radius: project_radius(*radius, camera)?,
            style,
        },
        DebugPrimitive::Segment { start, end, .. } => ScreenPrimitive::Segment {
            key,
            layer,
            start: project_point(*start, camera, size)?,
            end: project_point(*end, camera, size)?,
            style,
        },
        DebugPrimitive::Arrow { start, end, .. } => ScreenPrimitive::Arrow {
            key,
            layer,
            start: project_point(*start, camera, size)?,
            end: project_point(*end, camera, size)?,
            style,
        },
        DebugPrimitive::Polyline {
            vertices, closed, ..
        } => {
            if vertices.len() > MAXIMUM_POLYLINE_VERTICES {
                return Err(ViewportError::PrimitiveLimit);
            }
            let vertices = vertices
                .iter()
                .map(|vertex| project_point(*vertex, camera, size))
                .collect::<Result<Vec<_>, _>>()?;
            ScreenPrimitive::Polyline {
                key,
                layer,
                vertices,
                closed: *closed,
                style,
            }
        }
        DebugPrimitive::Circle { center, radius, .. } => ScreenPrimitive::Circle {
            key,
            layer,
            center: project_point(*center, camera, size)?,
            radius: project_radius(*radius, camera)?,
            style,
        },
        DebugPrimitive::TransformAxes {
            transform, scale, ..
        } => {
            let origin = transform.position();
            let x_end = origin + *scale * transform.rotation().x_axis();
            let y_end = origin + *scale * transform.rotation().y_axis();
            ScreenPrimitive::TransformAxes {
                key,
                layer,
                origin: project_point(origin, camera, size)?,
                x_end: project_point(x_end, camera, size)?,
                y_end: project_point(y_end, camera, size)?,
                style,
            }
        }
        DebugPrimitive::Aabb { bounds, .. } => ScreenPrimitive::Aabb {
            key,
            layer,
            lower: project_point(bounds.lower_bound(), camera, size)?,
            upper: project_point(bounds.upper_bound(), camera, size)?,
            style,
        },
        DebugPrimitive::Label { position, text, .. } => {
            if text.is_empty()
                || text.len() > 256
                || !text.is_ascii()
                || text.bytes().any(|byte| byte.is_ascii_control())
            {
                return Err(ViewportError::InvalidLabel);
            }
            ScreenPrimitive::Label {
                key,
                layer,
                position: project_point(*position, camera, size)?,
                text: text.clone().into_boxed_str(),
                style,
            }
        }
        _ => return Err(ViewportError::UnsupportedPrimitive),
    };
    Ok(screen)
}

fn project_style(
    metadata: DebugPrimitiveMetadata,
    camera: Camera,
) -> Result<ScreenStyle, ViewportError> {
    let style = ScreenStyle {
        stroke: metadata.stroke().color().components(),
        stroke_width: (metadata.stroke().width() * camera.pixels_per_meter()).max(1.0),
        maybe_fill: metadata.maybe_fill().map(|fill| fill.color().components()),
    };
    if !style.stroke_width.is_finite() {
        return Err(ViewportError::InvalidGeometry);
    }
    Ok(style)
}

fn project_point(
    point: Vec2,
    camera: Camera,
    size: ScreenSize,
) -> Result<ScreenPoint, ViewportError> {
    if !point.is_valid() || point.x.abs() > 1_000_000.0 || point.y.abs() > 1_000_000.0 {
        return Err(ViewportError::InvalidGeometry);
    }
    let projected = camera.world_to_screen(point, size);
    if !projected.x.is_finite() || !projected.y.is_finite() {
        return Err(ViewportError::InvalidGeometry);
    }
    Ok(projected)
}

fn project_radius(radius: f32, camera: Camera) -> Result<f32, ViewportError> {
    if !radius.is_finite() || !(0.0..=1_000_000.0).contains(&radius) {
        return Err(ViewportError::InvalidGeometry);
    }
    Ok(radius * camera.pixels_per_meter())
}

/// Validated screenshot destination confined below the workspace `target/` tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticScreenshotPath(PathBuf);

impl DiagnosticScreenshotPath {
    /// Validates a regular `.png` destination without creating a file.
    ///
    /// # Errors
    ///
    /// Rejects absolute, traversing, linked, oversized, non-target, or non-PNG destinations.
    pub fn new(workspace_root: &Path, relative: &Path) -> Result<Self, ScreenshotPathError> {
        if relative.as_os_str().len() > MAXIMUM_SCREENSHOT_PATH_BYTES
            || relative.is_absolute()
            || relative.extension().and_then(|value| value.to_str()) != Some("png")
        {
            return Err(ScreenshotPathError);
        }
        let mut components = relative.components();
        if components.next() != Some(Component::Normal("target".as_ref()))
            || components.any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(ScreenshotPathError);
        }
        let mut cursor = workspace_root.to_path_buf();
        for component in relative.components() {
            let Component::Normal(segment) = component else {
                return Err(ScreenshotPathError);
            };
            cursor.push(segment);
            let Ok(metadata) = std::fs::symlink_metadata(&cursor) else {
                continue;
            };
            if metadata.file_type().is_symlink() {
                return Err(ScreenshotPathError);
            }
        }
        if cursor.exists() && !cursor.is_file() {
            return Err(ScreenshotPathError);
        }
        Ok(Self(relative.to_path_buf()))
    }

    #[must_use]
    pub fn relative(&self) -> &Path {
        &self.0
    }

    #[must_use]
    pub const fn acknowledgement(&self) -> &'static str {
        SCREENSHOT_CLARIFICATION
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("screenshot output must be a confined regular PNG below target")]
pub struct ScreenshotPathError;
