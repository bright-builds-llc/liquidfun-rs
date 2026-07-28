/// Projects a validated protocol display list through the replacement renderer contract.
pub fn draw_protocol_frame(frame: &ProtocolFrame) {
    let _maybe_presentation = protocol_presentation_frame(frame);
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
    let _maybe_presentation =
        comparison_presentation_frame(frame, comparison, backend, maybe_focused_entry);
}

pub(crate) fn protocol_presentation_frame(frame: &ProtocolFrame) -> Option<PresentationFrame> {
    let commands = frame
        .primitives()
        .iter()
        .flat_map(|record| record_commands(record, record.style, false))
        .collect();
    presentation(frame.viewport(), commands)
}

fn comparison_presentation_frame(
    frame: &ProtocolFrame,
    comparison: &ComparisonModel,
    backend: ProtocolComparisonBackend,
    maybe_focused_entry: Option<&ComparisonEntry>,
) -> Option<PresentationFrame> {
    let maybe_focused_key = maybe_focused_entry.and_then(ComparisonEntry::maybe_primitive_key);
    let mut commands = Vec::new();
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
            commands.extend(record_commands(record, halo, false));
        }
        let style = comparison_style(record.style(), state, backend);
        commands.extend(record_commands(
            record,
            style,
            backend == ProtocolComparisonBackend::Oracle && state != ComparisonState::ExactMatch,
        ));
        if focused && let Some(entry) = maybe_focused_entry {
            let anchor = primitive_anchor(record.primitive());
            let label = format!(
                "{} {}: {}",
                cue.marker(),
                cue.label(),
                entry.semantic_path()
            );
            commands.extend(text_command(
                ProtocolScreenPoint {
                    x: anchor.x + 8.0,
                    y: anchor.y - 8.0,
                },
                &label,
                FOCUSED_HALO_COLOR,
            ));
        }
    }
    presentation(frame.viewport(), commands)
}

fn presentation(
    viewport: ProtocolViewport,
    commands: Vec<DrawCommand>,
) -> Option<PresentationFrame> {
    let width = (viewport.x() + viewport.width()).max(1.0);
    let height = (viewport.y() + viewport.height()).max(1.0);
    Some(PresentationFrame::new(
        LogicalSize::new(width, height).ok()?,
        TRANSPARENT,
        commands,
    ))
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

fn record_commands(
    record: &ProtocolDisplayRecord,
    style: ProtocolScreenStyle,
    dashed: bool,
) -> Vec<DrawCommand> {
    match &record.primitive {
        ProtocolDisplayPrimitive::Point { position, radius }
        | ProtocolDisplayPrimitive::Circle {
            center: position,
            radius,
        } => circle_commands(*position, *radius, style, dashed),
        ProtocolDisplayPrimitive::Segment { start, end } => {
            segment_commands(*start, *end, style, dashed)
        }
        ProtocolDisplayPrimitive::Polyline { vertices, closed } => {
            polyline_commands(vertices, *closed, style, dashed)
        }
        ProtocolDisplayPrimitive::TransformAxes {
            origin,
            x_end,
            y_end,
        } => {
            let mut commands = segment_commands(*origin, *x_end, style, dashed);
            commands.extend(segment_commands(*origin, *y_end, style, dashed));
            commands
        }
        ProtocolDisplayPrimitive::Aabb {
            left,
            top,
            right,
            bottom,
        } => aabb_commands(*left, *top, *right, *bottom, style, dashed),
        ProtocolDisplayPrimitive::Arrow { start, end } => {
            arrow_commands(*start, *end, style, dashed)
        }
        ProtocolDisplayPrimitive::Label { position, text } => {
            text_command(*position, text, style.stroke)
                .into_iter()
                .collect()
        }
    }
}

fn circle_commands(
    center: ProtocolScreenPoint,
    radius: f32,
    style: ProtocolScreenStyle,
    dashed: bool,
) -> Vec<DrawCommand> {
    let mut commands = Vec::new();
    if let Some(fill) = style.maybe_fill {
        commands.extend(circle_command(center, radius, fill));
    }
    if style.stroke_width > 0.0 && dashed {
        commands.extend(dashed_circle_commands(center, radius, style));
    } else if style.stroke_width > 0.0 {
        commands.extend(circle_command(center, radius, style.stroke));
    }
    commands
}

fn polyline_commands(
    vertices: &[ProtocolScreenPoint],
    closed: bool,
    style: ProtocolScreenStyle,
    dashed: bool,
) -> Vec<DrawCommand> {
    let mut commands = Vec::new();
    if style.stroke_width <= 0.0 {
        return commands;
    }
    for pair in vertices.windows(2) {
        commands.extend(segment_commands(pair[0], pair[1], style, dashed));
    }
    if closed && vertices.len() > 2 {
        let Some(first) = vertices.first() else {
            return commands;
        };
        let Some(last) = vertices.last() else {
            return commands;
        };
        commands.extend(segment_commands(*last, *first, style, dashed));
    }
    commands
}

fn aabb_commands(
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    style: ProtocolScreenStyle,
    dashed: bool,
) -> Vec<DrawCommand> {
    let mut commands = Vec::new();
    let width = right - left;
    let height = bottom - top;
    if let Some(fill) = style.maybe_fill
        && let (Ok(origin), Ok(size)) = (
            LogicalPoint::new(left, top),
            LogicalSize::new(width, height),
        )
    {
        commands.push(DrawCommand::FillRectangle(Rectangle::new(
            origin,
            size,
            color(fill),
        )));
    }
    if style.stroke_width > 0.0 {
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
            commands.extend(segment_commands(
                corners[index],
                corners[(index + 1) % corners.len()],
                style,
                dashed,
            ));
        }
    }
    commands
}

fn arrow_commands(
    start: ProtocolScreenPoint,
    end: ProtocolScreenPoint,
    style: ProtocolScreenStyle,
    dashed: bool,
) -> Vec<DrawCommand> {
    let mut commands = segment_commands(start, end, style, dashed);
    if style.stroke_width <= 0.0 {
        return commands;
    }
    let delta_x = end.x - start.x;
    let delta_y = end.y - start.y;
    let length = delta_x.hypot(delta_y);
    if length <= f32::EPSILON {
        return commands;
    }
    let unit_x = delta_x / length;
    let unit_y = delta_y / length;
    for sign in [-1.0, 1.0] {
        let wing = ProtocolScreenPoint {
            x: end.x - 8.0 * unit_x + sign * 4.0 * unit_y,
            y: end.y - 8.0 * unit_y - sign * 4.0 * unit_x,
        };
        commands.extend(segment_commands(end, wing, style, dashed));
    }
    commands
}

fn segment_commands(
    start: ProtocolScreenPoint,
    end: ProtocolScreenPoint,
    style: ProtocolScreenStyle,
    dashed: bool,
) -> Vec<DrawCommand> {
    if style.stroke_width <= 0.0 {
        return Vec::new();
    }
    if dashed {
        return dashed_segment_commands(start, end, style);
    }
    line_command(start, end, style).into_iter().collect()
}

fn dashed_segment_commands(
    start: ProtocolScreenPoint,
    end: ProtocolScreenPoint,
    style: ProtocolScreenStyle,
) -> Vec<DrawCommand> {
    let mut commands = Vec::new();
    let delta_x = end.x - start.x;
    let delta_y = end.y - start.y;
    let length = delta_x.hypot(delta_y);
    if length <= f32::EPSILON {
        return commands;
    }
    let unit_x = delta_x / length;
    let unit_y = delta_y / length;
    let mut offset = 0.0;
    while offset < length {
        let dash_end = (offset + 6.0).min(length);
        commands.extend(line_command(
            ProtocolScreenPoint {
                x: start.x + unit_x * offset,
                y: start.y + unit_y * offset,
            },
            ProtocolScreenPoint {
                x: start.x + unit_x * dash_end,
                y: start.y + unit_y * dash_end,
            },
            style,
        ));
        offset += 10.0;
    }
    commands
}

fn dashed_circle_commands(
    center: ProtocolScreenPoint,
    radius: f32,
    style: ProtocolScreenStyle,
) -> Vec<DrawCommand> {
    const SEGMENTS: u8 = 32;
    let mut commands = Vec::new();
    for index in (0..SEGMENTS).step_by(2) {
        let start_angle = f32::from(index) * std::f32::consts::TAU / f32::from(SEGMENTS);
        let end_angle = f32::from(index + 1) * std::f32::consts::TAU / f32::from(SEGMENTS);
        let (start_sin, start_cos) = start_angle.sin_cos();
        let (end_sin, end_cos) = end_angle.sin_cos();
        commands.extend(line_command(
            ProtocolScreenPoint {
                x: center.x + radius * start_cos,
                y: center.y + radius * start_sin,
            },
            ProtocolScreenPoint {
                x: center.x + radius * end_cos,
                y: center.y + radius * end_sin,
            },
            style,
        ));
    }
    commands
}

fn line_command(
    start: ProtocolScreenPoint,
    end: ProtocolScreenPoint,
    style: ProtocolScreenStyle,
) -> Option<DrawCommand> {
    let stroke = Stroke::new(color(style.stroke), style.stroke_width).ok()?;
    Some(DrawCommand::StrokeLine(Line::new(
        point(start)?,
        point(end)?,
        stroke,
    )))
}

fn circle_command(
    center: ProtocolScreenPoint,
    radius: f32,
    components: [u8; 4],
) -> Option<DrawCommand> {
    Circle::new(point(center)?, radius.max(1.0), color(components))
        .ok()
        .map(DrawCommand::FillCircle)
}

fn text_command(
    position: ProtocolScreenPoint,
    text: &str,
    components: [u8; 4],
) -> Option<DrawCommand> {
    TextDrawing::new(
        point(position)?,
        text.to_owned(),
        LABEL_FONT_SIZE,
        color(components),
    )
    .ok()
    .map(DrawCommand::Text)
}

fn point(value: ProtocolScreenPoint) -> Option<LogicalPoint> {
    LogicalPoint::new(value.x, value.y).ok()
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

const fn color(components: [u8; 4]) -> RgbaColor {
    RgbaColor::new(components[0], components[1], components[2], components[3])
}
