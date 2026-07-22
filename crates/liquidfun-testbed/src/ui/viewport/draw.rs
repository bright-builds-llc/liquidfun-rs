//! Macroquad imperative shell over a validated semantic display list.

use macroquad::prelude::{Color, draw_circle, draw_line, draw_rectangle_lines, draw_text};

use super::{ScreenPoint, ScreenPrimitive, ScreenStyle, ViewportFrame};

/// Draws a previously validated display list through the selected private adapter.
pub fn draw_frame(frame: &ViewportFrame) {
    for primitive in &frame.primitives {
        draw_screen_primitive(primitive);
    }
}

fn draw_screen_primitive(primitive: &ScreenPrimitive) {
    match primitive {
        ScreenPrimitive::Point {
            position,
            radius,
            style,
            ..
        } => draw_circle(position.x, position.y, *radius, color(style.stroke)),
        ScreenPrimitive::Segment {
            start, end, style, ..
        } => draw_line(
            start.x,
            start.y,
            end.x,
            end.y,
            style.stroke_width,
            color(style.stroke),
        ),
        ScreenPrimitive::Arrow {
            start, end, style, ..
        } => draw_arrow(*start, *end, *style),
        ScreenPrimitive::Polyline {
            vertices,
            closed,
            style,
            ..
        } => draw_polyline(vertices, *closed, *style),
        ScreenPrimitive::Circle {
            center,
            radius,
            style,
            ..
        } => draw_circle(center.x, center.y, *radius, color(style.stroke)),
        ScreenPrimitive::TransformAxes {
            origin,
            x_end,
            y_end,
            style,
            ..
        } => {
            draw_segment(*origin, *x_end, *style);
            draw_segment(*origin, *y_end, *style);
        }
        ScreenPrimitive::Aabb {
            lower,
            upper,
            style,
            ..
        } => draw_rectangle_lines(
            lower.x,
            upper.y,
            upper.x - lower.x,
            lower.y - upper.y,
            style.stroke_width,
            color(style.stroke),
        ),
        ScreenPrimitive::Label {
            position,
            text,
            style,
            ..
        } => {
            draw_text(text, position.x, position.y, 14.0, color(style.stroke));
        }
    }
}

fn draw_arrow(start: ScreenPoint, end: ScreenPoint, style: ScreenStyle) {
    draw_segment(start, end, style);
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let length = dx.hypot(dy);
    if length <= f32::EPSILON {
        return;
    }
    let unit_x = dx / length;
    let unit_y = dy / length;
    for sign in [-1.0, 1.0] {
        let wing = ScreenPoint {
            x: end.x - 8.0 * unit_x + sign * 4.0 * unit_y,
            y: end.y - 8.0 * unit_y - sign * 4.0 * unit_x,
        };
        draw_segment(end, wing, style);
    }
}

fn draw_polyline(vertices: &[ScreenPoint], closed: bool, style: ScreenStyle) {
    for endpoints in vertices.windows(2) {
        draw_segment(endpoints[0], endpoints[1], style);
    }
    if closed && vertices.len() > 2 {
        let Some(first) = vertices.first() else {
            return;
        };
        let Some(last) = vertices.last() else {
            return;
        };
        draw_segment(*last, *first, style);
    }
}

fn draw_segment(start: ScreenPoint, end: ScreenPoint, style: ScreenStyle) {
    draw_line(
        start.x,
        start.y,
        end.x,
        end.y,
        style.stroke_width,
        color(style.stroke),
    );
}

const fn color(components: [u8; 4]) -> Color {
    Color::from_rgba(components[0], components[1], components[2], components[3])
}
