//! Bounded replacement-renderer projection for canonical protocol debug primitives.

#![allow(
    missing_docs,
    reason = "closed private-testbed display variants mirror the protocol vocabulary"
)]

use liquidfun_differential::{ComparisonEntry, ComparisonModel, ComparisonState};
use liquidfun_test_protocol::{
    CanonicalCheckpoint, DebugLayerName, DebugPrimitiveKey, DebugPrimitiveOrder, PrimitiveMetadata,
    Vec2Bits, WireDebugPrimitive,
};

use super::differences::visual_cue;
use crate::renderer::{
    Circle, DrawCommand, Line, LogicalPoint, LogicalSize, PresentationFrame, Rectangle, RgbaColor,
    Stroke, TextDrawing,
};

const LAYER_COUNT: usize = 9;
const MAXIMUM_VIEWPORT_EXTENT: f32 = 32_768.0;
const MAXIMUM_WORLD_COORDINATE: f32 = 1_000_000.0;
const MAXIMUM_PIXELS_PER_METER: f32 = 4_096.0;
const MAXIMUM_SCREEN_GEOMETRY: f32 = 1_048_576.0;
const LABEL_FONT_SIZE: f32 = 14.0;
const RUST_COMPARISON_COLOR: [u8; 4] = [255, 140, 66, 255];
const ORACLE_COMPARISON_COLOR: [u8; 4] = [163, 113, 247, 255];
const FOCUSED_HALO_COLOR: [u8; 4] = [248, 81, 73, 255];
const TRANSPARENT: RgbaColor = RgbaColor::new(0, 0, 0, 0);

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

/// Returns the topmost semantic primitive under one finite screen point.
///
/// Hit testing consumes only the immutable projected frame and never submits a simulation
/// command. Records are inspected in reverse draw order so overlapping geometry selects the
/// visible topmost semantic key.
#[must_use]
pub fn hit_test_frame(
    frame: &ProtocolFrame,
    point: ProtocolScreenPoint,
    tolerance: f32,
) -> Option<&DebugPrimitiveKey> {
    if !point.x.is_finite() || !point.y.is_finite() || !tolerance.is_finite() || tolerance < 0.0 {
        return None;
    }
    frame
        .primitives()
        .iter()
        .rev()
        .find(|record| hit_primitive(record.primitive(), point, tolerance))
        .map(ProtocolDisplayRecord::key)
}

fn hit_primitive(
    primitive: &ProtocolDisplayPrimitive,
    point: ProtocolScreenPoint,
    tolerance: f32,
) -> bool {
    match primitive {
        ProtocolDisplayPrimitive::Point { position, radius }
        | ProtocolDisplayPrimitive::Circle {
            center: position,
            radius,
        } => distance(*position, point) <= radius + tolerance,
        ProtocolDisplayPrimitive::Segment { start, end }
        | ProtocolDisplayPrimitive::Arrow { start, end } => {
            segment_distance(point, *start, *end) <= tolerance
        }
        ProtocolDisplayPrimitive::Polyline { vertices, closed } => {
            (*closed && point_in_polygon(point, vertices))
                || vertices
                    .windows(2)
                    .any(|edge| segment_distance(point, edge[0], edge[1]) <= tolerance)
                || (*closed
                    && vertices
                        .first()
                        .zip(vertices.last())
                        .is_some_and(|(first, last)| {
                            segment_distance(point, *last, *first) <= tolerance
                        }))
        }
        ProtocolDisplayPrimitive::TransformAxes {
            origin,
            x_end,
            y_end,
        } => {
            segment_distance(point, *origin, *x_end) <= tolerance
                || segment_distance(point, *origin, *y_end) <= tolerance
        }
        ProtocolDisplayPrimitive::Aabb {
            left,
            top,
            right,
            bottom,
        } => {
            (left - tolerance..=right + tolerance).contains(&point.x)
                && (top - tolerance..=bottom + tolerance).contains(&point.y)
        }
        ProtocolDisplayPrimitive::Label { position, .. } => {
            distance(*position, point) <= tolerance.max(8.0)
        }
    }
}

fn distance(left: ProtocolScreenPoint, right: ProtocolScreenPoint) -> f32 {
    (left.x - right.x).hypot(left.y - right.y)
}

fn segment_distance(
    point: ProtocolScreenPoint,
    start: ProtocolScreenPoint,
    end: ProtocolScreenPoint,
) -> f32 {
    let delta_x = end.x - start.x;
    let delta_y = end.y - start.y;
    let length_squared = delta_x.mul_add(delta_x, delta_y * delta_y);
    if length_squared <= f32::EPSILON {
        return distance(point, start);
    }
    let projection = ((point.x - start.x).mul_add(delta_x, (point.y - start.y) * delta_y)
        / length_squared)
        .clamp(0.0, 1.0);
    let closest = ProtocolScreenPoint {
        x: delta_x.mul_add(projection, start.x),
        y: delta_y.mul_add(projection, start.y),
    };
    distance(point, closest)
}

fn point_in_polygon(point: ProtocolScreenPoint, vertices: &[ProtocolScreenPoint]) -> bool {
    if vertices.len() < 3 {
        return false;
    }
    let mut inside = false;
    let mut previous = *vertices.last().unwrap_or(&vertices[0]);
    for current in vertices {
        let crosses = (current.y > point.y) != (previous.y > point.y)
            && point.x
                < (previous.x - current.x) * (point.y - current.y) / (previous.y - current.y)
                    + current.x;
        if crosses {
            inside = !inside;
        }
        previous = *current;
    }
    inside
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

include!("protocol_viewport/presentation.rs");

#[cfg(test)]
mod tests;
