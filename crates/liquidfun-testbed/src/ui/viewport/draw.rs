//! Renderer-contract projection over a validated semantic display list.

use crate::renderer::{
    Circle, DrawCommand, Line, LogicalPoint, LogicalSize, PresentationFrame, Rectangle, RgbaColor,
    Stroke, TextDrawing,
};

use super::{ScreenPoint, ScreenPrimitive, ScreenStyle, ViewportFrame};

const DEFAULT_VIEWPORT_WIDTH: f32 = 640.0;
const DEFAULT_VIEWPORT_HEIGHT: f32 = 480.0;
const TRANSPARENT: RgbaColor = RgbaColor::new(0, 0, 0, 0);

/// Projects a validated display list through the replacement renderer boundary.
///
/// The legacy shell intentionally ignores the resulting passive display list. The replacement
/// desktop shell consumes the same projection without gaining simulation authority.
pub fn draw_frame(frame: &ViewportFrame) {
    let _maybe_presentation = presentation_frame(frame);
}

pub(crate) fn presentation_frame(frame: &ViewportFrame) -> Option<PresentationFrame> {
    let commands = frame
        .primitives
        .iter()
        .flat_map(screen_primitive_commands)
        .collect();
    Some(PresentationFrame::new(
        LogicalSize::new(DEFAULT_VIEWPORT_WIDTH, DEFAULT_VIEWPORT_HEIGHT).ok()?,
        TRANSPARENT,
        commands,
    ))
}

fn screen_primitive_commands(primitive: &ScreenPrimitive) -> Vec<DrawCommand> {
    match primitive {
        ScreenPrimitive::Point {
            position,
            radius,
            style,
            ..
        }
        | ScreenPrimitive::Circle {
            center: position,
            radius,
            style,
            ..
        } => circle_command(*position, *radius, style.stroke)
            .into_iter()
            .collect(),
        ScreenPrimitive::Segment {
            start, end, style, ..
        } => line_command(*start, *end, *style).into_iter().collect(),
        ScreenPrimitive::Arrow {
            start, end, style, ..
        } => arrow_commands(*start, *end, *style),
        ScreenPrimitive::Polyline {
            vertices,
            closed,
            style,
            ..
        } => polyline_commands(vertices, *closed, *style),
        ScreenPrimitive::TransformAxes {
            origin,
            x_end,
            y_end,
            style,
            ..
        } => [(*origin, *x_end), (*origin, *y_end)]
            .into_iter()
            .filter_map(|(start, end)| line_command(start, end, *style))
            .collect(),
        ScreenPrimitive::Aabb {
            lower,
            upper,
            style,
            ..
        } => rectangle_commands(*lower, *upper, *style),
        ScreenPrimitive::Label {
            position,
            text,
            style,
            ..
        } => text_command(*position, text, style.stroke)
            .into_iter()
            .collect(),
    }
}

fn arrow_commands(start: ScreenPoint, end: ScreenPoint, style: ScreenStyle) -> Vec<DrawCommand> {
    let mut commands = line_command(start, end, style)
        .into_iter()
        .collect::<Vec<_>>();
    let delta_x = end.x - start.x;
    let delta_y = end.y - start.y;
    let length = delta_x.hypot(delta_y);
    if length <= f32::EPSILON {
        return commands;
    }
    let unit_x = delta_x / length;
    let unit_y = delta_y / length;
    for sign in [-1.0, 1.0] {
        let wing = ScreenPoint {
            x: end.x - 8.0 * unit_x + sign * 4.0 * unit_y,
            y: end.y - 8.0 * unit_y - sign * 4.0 * unit_x,
        };
        commands.extend(line_command(end, wing, style));
    }
    commands
}

fn polyline_commands(
    vertices: &[ScreenPoint],
    closed: bool,
    style: ScreenStyle,
) -> Vec<DrawCommand> {
    let mut commands = vertices
        .windows(2)
        .filter_map(|endpoints| line_command(endpoints[0], endpoints[1], style))
        .collect::<Vec<_>>();
    if closed
        && vertices.len() > 2
        && let (Some(first), Some(last)) = (vertices.first(), vertices.last())
    {
        commands.extend(line_command(*last, *first, style));
    }
    commands
}

fn rectangle_commands(
    lower: ScreenPoint,
    upper: ScreenPoint,
    style: ScreenStyle,
) -> Vec<DrawCommand> {
    let left = lower.x.min(upper.x);
    let right = lower.x.max(upper.x);
    let top = lower.y.min(upper.y);
    let bottom = lower.y.max(upper.y);
    let corners = [
        ScreenPoint { x: left, y: top },
        ScreenPoint { x: right, y: top },
        ScreenPoint {
            x: right,
            y: bottom,
        },
        ScreenPoint { x: left, y: bottom },
    ];
    let mut commands = Vec::new();
    if let Some(fill) = style.maybe_fill
        && let (Ok(origin), Ok(size)) = (
            LogicalPoint::new(left, top),
            LogicalSize::new(right - left, bottom - top),
        )
    {
        commands.push(DrawCommand::FillRectangle(Rectangle::new(
            origin,
            size,
            color(fill),
        )));
    }
    for index in 0..corners.len() {
        commands.extend(line_command(
            corners[index],
            corners[(index + 1) % corners.len()],
            style,
        ));
    }
    commands
}

fn line_command(start: ScreenPoint, end: ScreenPoint, style: ScreenStyle) -> Option<DrawCommand> {
    let stroke = Stroke::new(color(style.stroke), style.stroke_width).ok()?;
    Some(DrawCommand::StrokeLine(Line::new(
        point(start)?,
        point(end)?,
        stroke,
    )))
}

fn circle_command(position: ScreenPoint, radius: f32, components: [u8; 4]) -> Option<DrawCommand> {
    Circle::new(point(position)?, radius.max(1.0), color(components))
        .ok()
        .map(DrawCommand::FillCircle)
}

fn text_command(position: ScreenPoint, text: &str, components: [u8; 4]) -> Option<DrawCommand> {
    TextDrawing::new(point(position)?, text.to_owned(), 14.0, color(components))
        .ok()
        .map(DrawCommand::Text)
}

fn point(value: ScreenPoint) -> Option<LogicalPoint> {
    LogicalPoint::new(value.x, value.y).ok()
}

const fn color(components: [u8; 4]) -> RgbaColor {
    RgbaColor::new(components[0], components[1], components[2], components[3])
}

#[cfg(test)]
mod tests {
    use liquidfun::{DebugLayer, DebugOwnerKey, DebugPrimitiveKey, DebugPrimitiveKind};

    use super::*;

    #[test]
    fn projection_preserves_source_order_and_overlay_alpha() {
        // Arrange
        let style = ScreenStyle {
            stroke: [10, 20, 30, 89],
            stroke_width: 2.0,
            maybe_fill: None,
        };
        let key = DebugPrimitiveKey::new(
            DebugOwnerKey::World,
            DebugLayer::Shapes,
            DebugPrimitiveKind::Segment,
            0,
            0,
        );
        let frame = ViewportFrame {
            primitives: vec![
                ScreenPrimitive::Segment {
                    key,
                    layer: DebugLayer::Shapes,
                    start: ScreenPoint { x: 1.0, y: 2.0 },
                    end: ScreenPoint { x: 3.0, y: 4.0 },
                    style,
                },
                ScreenPrimitive::Point {
                    key,
                    layer: DebugLayer::Shapes,
                    position: ScreenPoint { x: 5.0, y: 6.0 },
                    radius: 2.0,
                    style,
                },
            ],
            maybe_selected: None,
            maybe_hovered: None,
            zoom_percent: 100.0,
        };

        // Act
        let presentation = presentation_frame(&frame).expect("bounded display list should project");

        // Assert
        assert!(matches!(
            presentation.commands()[0],
            DrawCommand::StrokeLine(_)
        ));
        assert!(matches!(
            presentation.commands()[1],
            DrawCommand::FillCircle(_)
        ));
        let DrawCommand::StrokeLine(line) = &presentation.commands()[0] else {
            panic!("first primitive should remain a line");
        };
        assert_eq!(line.stroke().color().channels()[3], 89);
    }
}
