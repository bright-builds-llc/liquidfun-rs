//! Bounded Macroquad projection for canonical protocol debug primitives.

#![allow(
    missing_docs,
    reason = "closed private-testbed display variants mirror the protocol vocabulary"
)]

use liquidfun_differential::{ComparisonEntry, ComparisonModel, ComparisonState};
use liquidfun_test_protocol::{
    CanonicalCheckpoint, DebugLayerName, DebugPrimitiveKey, DebugPrimitiveOrder, PrimitiveMetadata,
    Vec2Bits, WireDebugPrimitive,
};
use macroquad::prelude::{
    Color, Vec2, draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_rectangle_lines,
    draw_text, draw_triangle,
};

use super::differences::visual_cue;

const LAYER_COUNT: usize = 9;
const MAXIMUM_VIEWPORT_EXTENT: f32 = 32_768.0;
const MAXIMUM_WORLD_COORDINATE: f32 = 1_000_000.0;
const MAXIMUM_PIXELS_PER_METER: f32 = 4_096.0;
const MAXIMUM_SCREEN_GEOMETRY: f32 = 1_048_576.0;
const LABEL_FONT_SIZE: f32 = 14.0;
const RUST_COMPARISON_COLOR: [u8; 4] = [255, 140, 66, 255];
const ORACLE_COMPARISON_COLOR: [u8; 4] = [163, 113, 247, 255];
const FOCUSED_HALO_COLOR: [u8; 4] = [248, 81, 73, 255];

/// Backend-specific stroke treatment for a two-checkpoint visual comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolComparisonBackend {
    /// Native Rust uses a solid orange stroke.
    Rust,
    /// The C++ oracle uses a dashed purple stroke.
    Oracle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProtocolViewport {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    center_x: f32,
    center_y: f32,
    pixels_per_meter: f32,
}

impl ProtocolViewport {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        center_x: f32,
        center_y: f32,
        pixels_per_meter: f32,
    ) -> Option<Self> {
        if !x.is_finite()
            || !y.is_finite()
            || !width.is_finite()
            || !height.is_finite()
            || !center_x.is_finite()
            || !center_y.is_finite()
            || !pixels_per_meter.is_finite()
            || x.abs() > MAXIMUM_VIEWPORT_EXTENT
            || y.abs() > MAXIMUM_VIEWPORT_EXTENT
            || width <= 0.0
            || height <= 0.0
            || width > MAXIMUM_VIEWPORT_EXTENT
            || height > MAXIMUM_VIEWPORT_EXTENT
            || center_x.abs() > MAXIMUM_WORLD_COORDINATE
            || center_y.abs() > MAXIMUM_WORLD_COORDINATE
            || pixels_per_meter <= 0.0
            || pixels_per_meter > MAXIMUM_PIXELS_PER_METER
        {
            return None;
        }
        Some(Self {
            x,
            y,
            width,
            height,
            center_x,
            center_y,
            pixels_per_meter,
        })
    }

    #[must_use]
    pub const fn x(self) -> f32 {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> f32 {
        self.y
    }

    #[must_use]
    pub const fn width(self) -> f32 {
        self.width
    }

    #[must_use]
    pub const fn height(self) -> f32 {
        self.height
    }

    #[must_use]
    pub const fn center_x(self) -> f32 {
        self.center_x
    }

    #[must_use]
    pub const fn center_y(self) -> f32 {
        self.center_y
    }

    #[must_use]
    pub const fn pixels_per_meter(self) -> f32 {
        self.pixels_per_meter
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtocolLayerVisibility {
    visible: [bool; LAYER_COUNT],
}

impl Default for ProtocolLayerVisibility {
    fn default() -> Self {
        Self::all()
    }
}

impl ProtocolLayerVisibility {
    #[must_use]
    pub const fn all() -> Self {
        Self {
            visible: [true; LAYER_COUNT],
        }
    }

    #[must_use]
    pub const fn none() -> Self {
        Self {
            visible: [false; LAYER_COUNT],
        }
    }

    pub const fn set(&mut self, layer: DebugLayerName, visible: bool) {
        self.visible[layer_index(layer)] = visible;
    }

    #[must_use]
    pub const fn is_visible(self, layer: DebugLayerName) -> bool {
        self.visible[layer_index(layer)]
    }
}

const fn layer_index(layer: DebugLayerName) -> usize {
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProtocolScreenPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProtocolScreenStyle {
    pub stroke: [u8; 4],
    pub stroke_width: f32,
    pub maybe_fill: Option<[u8; 4]>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolDisplayPrimitive {
    Point {
        position: ProtocolScreenPoint,
        radius: f32,
    },
    Segment {
        start: ProtocolScreenPoint,
        end: ProtocolScreenPoint,
    },
    Polyline {
        vertices: Box<[ProtocolScreenPoint]>,
        closed: bool,
    },
    Circle {
        center: ProtocolScreenPoint,
        radius: f32,
    },
    TransformAxes {
        origin: ProtocolScreenPoint,
        x_end: ProtocolScreenPoint,
        y_end: ProtocolScreenPoint,
    },
    Aabb {
        left: f32,
        top: f32,
        right: f32,
        bottom: f32,
    },
    Arrow {
        start: ProtocolScreenPoint,
        end: ProtocolScreenPoint,
    },
    Label {
        position: ProtocolScreenPoint,
        text: Box<str>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProtocolDisplayRecord {
    ordering: DebugPrimitiveOrder,
    key: DebugPrimitiveKey,
    layer: DebugLayerName,
    style: ProtocolScreenStyle,
    primitive: ProtocolDisplayPrimitive,
}

impl ProtocolDisplayRecord {
    #[must_use]
    pub const fn ordering(&self) -> DebugPrimitiveOrder {
        self.ordering
    }

    #[must_use]
    pub const fn key(&self) -> &DebugPrimitiveKey {
        &self.key
    }

    #[must_use]
    pub const fn layer(&self) -> DebugLayerName {
        self.layer
    }

    #[must_use]
    pub const fn style(&self) -> ProtocolScreenStyle {
        self.style
    }

    #[must_use]
    pub const fn primitive(&self) -> &ProtocolDisplayPrimitive {
        &self.primitive
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProtocolFrame {
    viewport: ProtocolViewport,
    primitives: Box<[ProtocolDisplayRecord]>,
}

impl ProtocolFrame {
    #[must_use]
    pub const fn viewport(&self) -> ProtocolViewport {
        self.viewport
    }

    #[must_use]
    pub fn primitives(&self) -> &[ProtocolDisplayRecord] {
        &self.primitives
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ProtocolViewportError {
    #[error("protocol viewport received out-of-range geometry")]
    GeometryOutOfRange,
}

/// Projects one validated checkpoint into a bounded, source-ordered display list.
///
/// # Errors
///
/// Returns [`ProtocolViewportError`] if finite wire values would produce geometry outside the
/// renderer's reviewed coordinate bounds.
pub fn project_checkpoint(
    checkpoint: &CanonicalCheckpoint,
    viewport: ProtocolViewport,
    visibility: ProtocolLayerVisibility,
) -> Result<ProtocolFrame, ProtocolViewportError> {
    let primitives = checkpoint
        .debug_primitives()
        .iter()
        .filter(|record| visibility.is_visible(record.key().layer()))
        .map(|record| {
            let primitive = record.primitive();
            Ok(ProtocolDisplayRecord {
                ordering: record.ordering(),
                key: record.key().clone(),
                layer: record.key().layer(),
                style: project_style(primitive.metadata(), viewport)?,
                primitive: project_primitive(primitive, viewport)?,
            })
        })
        .collect::<Result<Vec<_>, ProtocolViewportError>>()?;
    Ok(ProtocolFrame {
        viewport,
        primitives: primitives.into_boxed_slice(),
    })
}

fn project_style(
    metadata: &PrimitiveMetadata,
    viewport: ProtocolViewport,
) -> Result<ProtocolScreenStyle, ProtocolViewportError> {
    let stroke_width = project_length(
        metadata.stroke().width_bits().to_f32(),
        viewport.pixels_per_meter,
    )?;
    Ok(ProtocolScreenStyle {
        stroke: metadata.stroke().color().components(),
        stroke_width,
        maybe_fill: metadata.maybe_fill().map(|fill| fill.color().components()),
    })
}

fn project_primitive(
    primitive: &WireDebugPrimitive,
    viewport: ProtocolViewport,
) -> Result<ProtocolDisplayPrimitive, ProtocolViewportError> {
    match primitive {
        WireDebugPrimitive::Point(value) => Ok(ProtocolDisplayPrimitive::Point {
            position: project_point(value.position(), viewport)?,
            radius: project_length(value.radius_bits().to_f32(), viewport.pixels_per_meter)?,
        }),
        WireDebugPrimitive::Segment(value) => Ok(ProtocolDisplayPrimitive::Segment {
            start: project_point(value.start(), viewport)?,
            end: project_point(value.end(), viewport)?,
        }),
        WireDebugPrimitive::Polyline(value) => Ok(ProtocolDisplayPrimitive::Polyline {
            vertices: value
                .vertices()
                .iter()
                .copied()
                .map(|point| project_point(point, viewport))
                .collect::<Result<Vec<_>, _>>()?
                .into_boxed_slice(),
            closed: value.closed(),
        }),
        WireDebugPrimitive::Circle(value) => Ok(ProtocolDisplayPrimitive::Circle {
            center: project_point(value.center(), viewport)?,
            radius: project_length(value.radius_bits().to_f32(), viewport.pixels_per_meter)?,
        }),
        WireDebugPrimitive::TransformAxes(value) => {
            let transform = value.transform();
            let origin_x = transform.position.x_bits.to_f32();
            let origin_y = transform.position.y_bits.to_f32();
            let scale = value.scale_bits().to_f32();
            let (sin, cos) = transform.angle_bits.to_f32().sin_cos();
            let origin = point_bits(origin_x, origin_y);
            let x_end = point_bits(origin_x + scale * cos, origin_y + scale * sin);
            let y_end = point_bits(origin_x - scale * sin, origin_y + scale * cos);
            Ok(ProtocolDisplayPrimitive::TransformAxes {
                origin: project_point(origin, viewport)?,
                x_end: project_point(x_end, viewport)?,
                y_end: project_point(y_end, viewport)?,
            })
        }
        WireDebugPrimitive::Aabb(value) => {
            let lower = project_point(value.lower(), viewport)?;
            let upper = project_point(value.upper(), viewport)?;
            Ok(ProtocolDisplayPrimitive::Aabb {
                left: lower.x,
                top: upper.y,
                right: upper.x,
                bottom: lower.y,
            })
        }
        WireDebugPrimitive::Arrow(value) => Ok(ProtocolDisplayPrimitive::Arrow {
            start: project_point(value.start(), viewport)?,
            end: project_point(value.end(), viewport)?,
        }),
        WireDebugPrimitive::Label(value) => Ok(ProtocolDisplayPrimitive::Label {
            position: project_point(value.position(), viewport)?,
            text: value.text().into(),
        }),
    }
}

fn point_bits(x: f32, y: f32) -> Vec2Bits {
    Vec2Bits {
        x_bits: liquidfun_test_protocol::FloatBits::from_f32(x),
        y_bits: liquidfun_test_protocol::FloatBits::from_f32(y),
    }
}

fn project_point(
    point: Vec2Bits,
    viewport: ProtocolViewport,
) -> Result<ProtocolScreenPoint, ProtocolViewportError> {
    let world_x = point.x_bits.to_f32();
    let world_y = point.y_bits.to_f32();
    if !world_x.is_finite()
        || !world_y.is_finite()
        || world_x.abs() > MAXIMUM_WORLD_COORDINATE
        || world_y.abs() > MAXIMUM_WORLD_COORDINATE
    {
        return Err(ProtocolViewportError::GeometryOutOfRange);
    }
    let x = (world_x - viewport.center_x).mul_add(
        viewport.pixels_per_meter,
        viewport.width.mul_add(0.5, viewport.x),
    );
    let y = (viewport.center_y - world_y).mul_add(
        viewport.pixels_per_meter,
        viewport.height.mul_add(0.5, viewport.y),
    );
    if !bounded_screen_value(x) || !bounded_screen_value(y) {
        return Err(ProtocolViewportError::GeometryOutOfRange);
    }
    Ok(ProtocolScreenPoint { x, y })
}

fn project_length(world_length: f32, pixels_per_meter: f32) -> Result<f32, ProtocolViewportError> {
    let length = world_length * pixels_per_meter;
    if !world_length.is_finite()
        || world_length < 0.0
        || !length.is_finite()
        || length > MAXIMUM_SCREEN_GEOMETRY
    {
        return Err(ProtocolViewportError::GeometryOutOfRange);
    }
    Ok(length)
}

fn bounded_screen_value(value: f32) -> bool {
    value.is_finite() && value.abs() <= MAXIMUM_SCREEN_GEOMETRY
}

/// Draws a previously validated protocol display list through Macroquad.
pub fn draw_protocol_frame(frame: &ProtocolFrame) {
    for record in frame.primitives() {
        draw_record(record);
    }
}

/// Draws one backend of an authoritative comparison with redundant visual encoding.
///
/// Matching primitives fade to 35% opacity. Differences use a solid orange Rust stroke or a
/// dashed purple oracle stroke. A focused physics mismatch gains a red halo and bounded semantic
/// path label; these presentation effects never mutate the comparison or checkpoint.
pub fn draw_protocol_comparison_frame(
    frame: &ProtocolFrame,
    comparison: &ComparisonModel,
    backend: ProtocolComparisonBackend,
    maybe_focused_entry: Option<&ComparisonEntry>,
) {
    let maybe_focused_key = maybe_focused_entry.and_then(ComparisonEntry::maybe_primitive_key);
    for record in frame.primitives() {
        let state = primitive_comparison_state(comparison, record.key());
        if (backend == ProtocolComparisonBackend::Rust && state == ComparisonState::OracleOnly)
            || (backend == ProtocolComparisonBackend::Oracle && state == ComparisonState::RustOnly)
        {
            continue;
        }
        let focused = maybe_focused_key == Some(record.key());
        let cue = visual_cue(state);
        if focused && cue.focused_halo() {
            let halo = ProtocolScreenStyle {
                stroke: FOCUSED_HALO_COLOR,
                stroke_width: record.style().stroke_width.max(2.0) + 4.0,
                maybe_fill: None,
            };
            draw_record_styled(record, halo, false);
        }
        let style = comparison_style(record.style(), state, backend);
        draw_record_styled(
            record,
            style,
            backend == ProtocolComparisonBackend::Oracle && state != ComparisonState::ExactMatch,
        );
        if focused && let Some(entry) = maybe_focused_entry {
            let anchor = primitive_anchor(record.primitive());
            let label = format!(
                "{} {}: {}",
                cue.marker(),
                cue.label(),
                entry.semantic_path()
            );
            draw_text(
                &label,
                anchor.x + 8.0,
                anchor.y - 8.0,
                LABEL_FONT_SIZE,
                color(FOCUSED_HALO_COLOR),
            );
        }
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
        .map(ComparisonEntry::state)
        .max_by_key(|state| comparison_state_rank(*state))
        .unwrap_or(ComparisonState::ExactMatch)
}

const fn comparison_state_rank(state: ComparisonState) -> u8 {
    match state {
        ComparisonState::ExactMatch => 0,
        ComparisonState::WithinPolicy => 1,
        ComparisonState::RustOnly | ComparisonState::OracleOnly => 2,
        ComparisonState::PhysicsMismatch => 3,
    }
}

fn comparison_style(
    original: ProtocolScreenStyle,
    state: ComparisonState,
    backend: ProtocolComparisonBackend,
) -> ProtocolScreenStyle {
    let cue = visual_cue(state);
    if state == ComparisonState::ExactMatch {
        return ProtocolScreenStyle {
            stroke: with_alpha(original.stroke, cue.opacity_percent()),
            stroke_width: original.stroke_width,
            maybe_fill: original
                .maybe_fill
                .map(|fill| with_alpha(fill, cue.opacity_percent())),
        };
    }
    ProtocolScreenStyle {
        stroke: match backend {
            ProtocolComparisonBackend::Rust => RUST_COMPARISON_COLOR,
            ProtocolComparisonBackend::Oracle => ORACLE_COMPARISON_COLOR,
        },
        stroke_width: original.stroke_width.max(2.0),
        maybe_fill: None,
    }
}

fn with_alpha(mut color: [u8; 4], opacity_percent: u8) -> [u8; 4] {
    let scaled = u16::from(color[3]) * u16::from(opacity_percent) / 100;
    color[3] = u8::try_from(scaled).unwrap_or(u8::MAX);
    color
}

fn draw_record(record: &ProtocolDisplayRecord) {
    draw_record_styled(record, record.style, false);
}

fn draw_record_styled(record: &ProtocolDisplayRecord, style: ProtocolScreenStyle, dashed: bool) {
    match &record.primitive {
        ProtocolDisplayPrimitive::Point { position, radius }
        | ProtocolDisplayPrimitive::Circle {
            center: position,
            radius,
        } => draw_circle_primitive(*position, *radius, style, dashed),
        ProtocolDisplayPrimitive::Segment { start, end } => {
            draw_segment(*start, *end, style, dashed);
        }
        ProtocolDisplayPrimitive::Polyline { vertices, closed } => {
            draw_polyline(vertices, *closed, style, dashed);
        }
        ProtocolDisplayPrimitive::TransformAxes {
            origin,
            x_end,
            y_end,
        } => {
            draw_segment(*origin, *x_end, style, dashed);
            draw_segment(*origin, *y_end, style, dashed);
        }
        ProtocolDisplayPrimitive::Aabb {
            left,
            top,
            right,
            bottom,
        } => draw_aabb(*left, *top, *right, *bottom, style, dashed),
        ProtocolDisplayPrimitive::Arrow { start, end } => {
            draw_arrow(*start, *end, style, dashed);
        }
        ProtocolDisplayPrimitive::Label { position, text } => {
            draw_text(
                text,
                position.x,
                position.y,
                LABEL_FONT_SIZE,
                color(style.stroke),
            );
        }
    }
}

fn draw_circle_primitive(
    center: ProtocolScreenPoint,
    radius: f32,
    style: ProtocolScreenStyle,
    dashed: bool,
) {
    if let Some(fill) = style.maybe_fill {
        draw_circle(center.x, center.y, radius, color(fill));
    }
    if style.stroke_width > 0.0 && dashed {
        draw_dashed_circle(center, radius, style);
    } else if style.stroke_width > 0.0 {
        draw_circle_lines(
            center.x,
            center.y,
            radius,
            style.stroke_width,
            color(style.stroke),
        );
    }
}

fn draw_polyline(
    vertices: &[ProtocolScreenPoint],
    closed: bool,
    style: ProtocolScreenStyle,
    dashed: bool,
) {
    if closed
        && vertices.len() >= 3
        && let Some(fill) = style.maybe_fill
    {
        let origin = screen_vec(vertices[0]);
        for pair in vertices[1..].windows(2) {
            draw_triangle(
                origin,
                screen_vec(pair[0]),
                screen_vec(pair[1]),
                color(fill),
            );
        }
    }
    if style.stroke_width <= 0.0 {
        return;
    }
    for pair in vertices.windows(2) {
        draw_segment(pair[0], pair[1], style, dashed);
    }
    if closed && vertices.len() > 2 {
        let Some(first) = vertices.first() else {
            return;
        };
        let Some(last) = vertices.last() else {
            return;
        };
        draw_segment(*last, *first, style, dashed);
    }
}

fn draw_aabb(
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    style: ProtocolScreenStyle,
    dashed: bool,
) {
    let width = right - left;
    let height = bottom - top;
    if let Some(fill) = style.maybe_fill {
        draw_rectangle(left, top, width, height, color(fill));
    }
    if style.stroke_width > 0.0 && dashed {
        let corners = [
            ProtocolScreenPoint { x: left, y: top },
            ProtocolScreenPoint { x: right, y: top },
            ProtocolScreenPoint {
                x: right,
                y: bottom,
            },
            ProtocolScreenPoint { x: left, y: bottom },
        ];
        for index in 0..corners.len() {
            draw_segment(
                corners[index],
                corners[(index + 1) % corners.len()],
                style,
                true,
            );
        }
    } else if style.stroke_width > 0.0 {
        draw_rectangle_lines(
            left,
            top,
            width,
            height,
            style.stroke_width,
            color(style.stroke),
        );
    }
}

fn draw_arrow(
    start: ProtocolScreenPoint,
    end: ProtocolScreenPoint,
    style: ProtocolScreenStyle,
    dashed: bool,
) {
    draw_segment(start, end, style, dashed);
    if style.stroke_width <= 0.0 {
        return;
    }
    let delta_x = end.x - start.x;
    let delta_y = end.y - start.y;
    let length = delta_x.hypot(delta_y);
    if length <= f32::EPSILON {
        return;
    }
    let unit_x = delta_x / length;
    let unit_y = delta_y / length;
    for sign in [-1.0, 1.0] {
        let wing = ProtocolScreenPoint {
            x: end.x - 8.0 * unit_x + sign * 4.0 * unit_y,
            y: end.y - 8.0 * unit_y - sign * 4.0 * unit_x,
        };
        draw_segment(end, wing, style, dashed);
    }
}

fn draw_segment(
    start: ProtocolScreenPoint,
    end: ProtocolScreenPoint,
    style: ProtocolScreenStyle,
    dashed: bool,
) {
    if style.stroke_width <= 0.0 {
        return;
    }
    if dashed {
        draw_dashed_segment(start, end, style);
        return;
    }
    draw_line(
        start.x,
        start.y,
        end.x,
        end.y,
        style.stroke_width,
        color(style.stroke),
    );
}

fn draw_dashed_segment(
    start: ProtocolScreenPoint,
    end: ProtocolScreenPoint,
    style: ProtocolScreenStyle,
) {
    let delta_x = end.x - start.x;
    let delta_y = end.y - start.y;
    let length = delta_x.hypot(delta_y);
    if length <= f32::EPSILON {
        return;
    }
    let unit_x = delta_x / length;
    let unit_y = delta_y / length;
    let mut offset = 0.0;
    while offset < length {
        let dash_end = (offset + 6.0).min(length);
        draw_line(
            start.x + unit_x * offset,
            start.y + unit_y * offset,
            start.x + unit_x * dash_end,
            start.y + unit_y * dash_end,
            style.stroke_width,
            color(style.stroke),
        );
        offset += 10.0;
    }
}

fn draw_dashed_circle(center: ProtocolScreenPoint, radius: f32, style: ProtocolScreenStyle) {
    const SEGMENTS: u8 = 32;
    for index in (0..SEGMENTS).step_by(2) {
        let start_angle = f32::from(index) * std::f32::consts::TAU / f32::from(SEGMENTS);
        let end_angle = f32::from(index + 1) * std::f32::consts::TAU / f32::from(SEGMENTS);
        let (start_sin, start_cos) = start_angle.sin_cos();
        let (end_sin, end_cos) = end_angle.sin_cos();
        draw_line(
            center.x + radius * start_cos,
            center.y + radius * start_sin,
            center.x + radius * end_cos,
            center.y + radius * end_sin,
            style.stroke_width,
            color(style.stroke),
        );
    }
}

fn primitive_anchor(primitive: &ProtocolDisplayPrimitive) -> ProtocolScreenPoint {
    match primitive {
        ProtocolDisplayPrimitive::Point { position, .. }
        | ProtocolDisplayPrimitive::Label { position, .. } => *position,
        ProtocolDisplayPrimitive::Segment { start, .. }
        | ProtocolDisplayPrimitive::Arrow { start, .. } => *start,
        ProtocolDisplayPrimitive::Polyline { vertices, .. } => vertices
            .first()
            .copied()
            .unwrap_or(ProtocolScreenPoint { x: 0.0, y: 0.0 }),
        ProtocolDisplayPrimitive::Circle { center, .. } => *center,
        ProtocolDisplayPrimitive::TransformAxes { origin, .. } => *origin,
        ProtocolDisplayPrimitive::Aabb { left, top, .. } => {
            ProtocolScreenPoint { x: *left, y: *top }
        }
    }
}

const fn color(components: [u8; 4]) -> Color {
    Color::from_rgba(components[0], components[1], components[2], components[3])
}

const fn screen_vec(point: ProtocolScreenPoint) -> Vec2 {
    Vec2::new(point.x, point.y)
}

#[cfg(test)]
mod tests {
    use liquidfun_test_protocol::{
        CanonicalCheckpoint, CheckpointId, CheckpointPosition, DebugPrimitiveOrder,
        DebugPrimitiveRecord, FloatBits, RequestId, Sha256Hex,
    };
    use serde_json::{Value, json};

    use super::{
        ORACLE_COMPARISON_COLOR, ProtocolComparisonBackend, ProtocolDisplayPrimitive,
        ProtocolLayerVisibility, ProtocolScreenStyle, ProtocolViewport, ProtocolViewportError,
        RUST_COMPARISON_COLOR, comparison_style, project_checkpoint,
    };

    fn bits(value: f32) -> u32 {
        value.to_bits()
    }

    fn metadata(kind: &str, layer: &str, ordinal: u32) -> Value {
        json!({
            "key": {
                "owner": { "kind": "world" },
                "layer": layer,
                "kind": kind,
                "child": 0,
                "ordinal": ordinal
            },
            "stroke": {
                "color": [1, 2, 3, 4],
                "width_bits": bits(0.5)
            },
            "maybe_fill": { "color": [5, 6, 7, 8] }
        })
    }

    #[allow(
        clippy::needless_pass_by_value,
        reason = "the JSON fixture builder consumes each temporary value exactly once"
    )]
    fn record(kind: &str, _layer: &str, _ordinal: u32, value: Value) -> DebugPrimitiveRecord {
        serde_json::from_value(json!({
            "ordering": "source_significant",
            "primitive": {
                "kind": kind,
                "value": value
            }
        }))
        .expect("test primitive fixture should satisfy the wire shape")
    }

    fn all_primitive_records() -> Vec<DebugPrimitiveRecord> {
        vec![
            record(
                "point",
                "shapes",
                0,
                json!({
                    "metadata": metadata("point", "shapes", 0),
                    "position": { "x_bits": bits(1.0), "y_bits": bits(2.0) },
                    "radius_bits": bits(0.25)
                }),
            ),
            record(
                "segment",
                "joints",
                1,
                json!({
                    "metadata": metadata("segment", "joints", 1),
                    "start": { "x_bits": bits(-1.0), "y_bits": bits(0.0) },
                    "end": { "x_bits": bits(1.0), "y_bits": bits(0.0) }
                }),
            ),
            record(
                "polyline",
                "contacts",
                2,
                json!({
                    "metadata": metadata("polyline", "contacts", 2),
                    "vertices": [
                        { "x_bits": bits(-1.0), "y_bits": bits(-1.0) },
                        { "x_bits": bits(1.0), "y_bits": bits(-1.0) },
                        { "x_bits": bits(0.0), "y_bits": bits(1.0) }
                    ],
                    "closed": true
                }),
            ),
            record(
                "circle",
                "contact_normals",
                3,
                json!({
                    "metadata": metadata("circle", "contact_normals", 3),
                    "center": { "x_bits": bits(0.0), "y_bits": bits(0.0) },
                    "radius_bits": bits(2.0)
                }),
            ),
            record(
                "transform_axes",
                "particles",
                4,
                json!({
                    "metadata": metadata("transform_axes", "particles", 4),
                    "transform": {
                        "position": { "x_bits": bits(0.0), "y_bits": bits(0.0) },
                        "angle_bits": bits(0.0)
                    },
                    "scale_bits": bits(1.0)
                }),
            ),
            record(
                "aabb",
                "particle_contacts",
                5,
                json!({
                    "metadata": metadata("aabb", "particle_contacts", 5),
                    "lower": { "x_bits": bits(-2.0), "y_bits": bits(-1.0) },
                    "upper": { "x_bits": bits(2.0), "y_bits": bits(1.0) }
                }),
            ),
            record(
                "arrow",
                "broad_phase",
                6,
                json!({
                    "metadata": metadata("arrow", "broad_phase", 6),
                    "start": { "x_bits": bits(0.0), "y_bits": bits(0.0) },
                    "end": { "x_bits": bits(2.0), "y_bits": bits(1.0) }
                }),
            ),
            record(
                "label",
                "labels",
                7,
                json!({
                    "metadata": metadata("label", "labels", 7),
                    "position": { "x_bits": bits(0.5), "y_bits": bits(-0.5) },
                    "text": "fixture-a"
                }),
            ),
        ]
    }

    fn checkpoint(records: Vec<DebugPrimitiveRecord>) -> CanonicalCheckpoint {
        CanonicalCheckpoint::new(
            RequestId::new("request-1").expect("static request ID should be valid"),
            Sha256Hex::new("0".repeat(64)).expect("static digest should be valid"),
            CheckpointId::new("checkpoint-0001").expect("static checkpoint ID should be valid"),
            CheckpointPosition::LogicalStep { ordinal: 1 },
            FloatBits::from_f32(0.0),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            records,
            Vec::new(),
        )
        .expect("test checkpoint should satisfy the canonical contract")
    }

    fn viewport() -> ProtocolViewport {
        ProtocolViewport::new(100.0, 50.0, 400.0, 200.0, 0.0, 0.0, 10.0)
            .expect("static viewport should be valid")
    }

    #[test]
    fn comparison_styles_fade_matches_and_distinguish_backends() {
        // Arrange
        let original = ProtocolScreenStyle {
            stroke: [10, 20, 30, 200],
            stroke_width: 1.0,
            maybe_fill: Some([40, 50, 60, 100]),
        };

        // Act
        let exact = comparison_style(
            original,
            liquidfun_differential::ComparisonState::ExactMatch,
            ProtocolComparisonBackend::Rust,
        );
        let rust_difference = comparison_style(
            original,
            liquidfun_differential::ComparisonState::PhysicsMismatch,
            ProtocolComparisonBackend::Rust,
        );
        let oracle_difference = comparison_style(
            original,
            liquidfun_differential::ComparisonState::PhysicsMismatch,
            ProtocolComparisonBackend::Oracle,
        );

        // Assert
        assert_eq!(exact.stroke, [10, 20, 30, 70]);
        assert_eq!(exact.maybe_fill, Some([40, 50, 60, 35]));
        assert_eq!(rust_difference.stroke, RUST_COMPARISON_COLOR);
        assert_eq!(oracle_difference.stroke, ORACLE_COMPARISON_COLOR);
        assert!(rust_difference.stroke_width >= 2.0);
        assert!(oracle_difference.maybe_fill.is_none());
    }

    #[test]
    fn projects_all_wire_variants_in_source_order_with_exact_style() {
        // Arrange
        let checkpoint = checkpoint(all_primitive_records());

        // Act
        let frame = project_checkpoint(&checkpoint, viewport(), ProtocolLayerVisibility::all())
            .expect("bounded fixture should project");

        // Assert
        let primitives = frame.primitives();
        assert_eq!(primitives.len(), 8);
        assert!(matches!(
            primitives[0].primitive(),
            ProtocolDisplayPrimitive::Point { .. }
        ));
        assert!(matches!(
            primitives[1].primitive(),
            ProtocolDisplayPrimitive::Segment { .. }
        ));
        assert!(matches!(
            primitives[2].primitive(),
            ProtocolDisplayPrimitive::Polyline { .. }
        ));
        assert!(matches!(
            primitives[3].primitive(),
            ProtocolDisplayPrimitive::Circle { .. }
        ));
        assert!(matches!(
            primitives[4].primitive(),
            ProtocolDisplayPrimitive::TransformAxes { .. }
        ));
        assert!(matches!(
            primitives[5].primitive(),
            ProtocolDisplayPrimitive::Aabb { .. }
        ));
        assert!(matches!(
            primitives[6].primitive(),
            ProtocolDisplayPrimitive::Arrow { .. }
        ));
        assert!(matches!(
            primitives[7].primitive(),
            ProtocolDisplayPrimitive::Label { .. }
        ));
        assert_eq!(
            primitives[0].ordering(),
            DebugPrimitiveOrder::SourceSignificant
        );
        assert_eq!(primitives[0].style().stroke, [1, 2, 3, 4]);
        assert_eq!(
            primitives[0].style().stroke_width.to_bits(),
            5.0_f32.to_bits()
        );
        assert_eq!(primitives[0].style().maybe_fill, Some([5, 6, 7, 8]));
        let ProtocolDisplayPrimitive::Point { position, radius } = primitives[0].primitive() else {
            panic!("first fixture primitive should remain a point");
        };
        assert_eq!(*position, super::ProtocolScreenPoint { x: 310.0, y: 130.0 });
        assert_eq!(radius.to_bits(), 2.5_f32.to_bits());
    }

    #[test]
    fn filters_by_protocol_semantic_layer_without_reordering() {
        // Arrange
        let checkpoint = checkpoint(all_primitive_records());
        let mut visibility = ProtocolLayerVisibility::none();
        visibility.set(liquidfun_test_protocol::DebugLayerName::Joints, true);
        visibility.set(liquidfun_test_protocol::DebugLayerName::Labels, true);

        // Act
        let frame = project_checkpoint(&checkpoint, viewport(), visibility)
            .expect("visible fixture layers should project");

        // Assert
        assert_eq!(frame.primitives().len(), 2);
        assert_eq!(frame.primitives()[0].key().ordinal(), 1);
        assert_eq!(frame.primitives()[1].key().ordinal(), 7);
    }

    #[test]
    fn rejects_geometry_outside_reviewed_projection_bounds() {
        // Arrange
        let record = record(
            "point",
            "shapes",
            0,
            json!({
                "metadata": metadata("point", "shapes", 0),
                "position": { "x_bits": bits(1_000_001.0), "y_bits": bits(0.0) },
                "radius_bits": bits(1.0)
            }),
        );
        let checkpoint = checkpoint(vec![record]);

        // Act
        let result = project_checkpoint(&checkpoint, viewport(), ProtocolLayerVisibility::all());

        // Assert
        assert_eq!(result, Err(ProtocolViewportError::GeometryOutOfRange));
    }

    #[test]
    fn rejects_nonfinite_or_unbounded_viewport_inputs() {
        // Arrange
        let invalid = [
            (f32::NAN, 0.0, 640.0, 480.0, 0.0, 0.0, 50.0),
            (0.0, 0.0, 0.0, 480.0, 0.0, 0.0, 50.0),
            (0.0, 0.0, 640.0, 480.0, 0.0, 0.0, 0.0),
            (0.0, 0.0, 640.0, 480.0, 0.0, 0.0, 4_097.0),
        ];

        // Act
        let viewports = invalid.map(|values| {
            ProtocolViewport::new(
                values.0, values.1, values.2, values.3, values.4, values.5, values.6,
            )
        });

        // Assert
        assert!(viewports.into_iter().all(|value| value.is_none()));
    }
}
