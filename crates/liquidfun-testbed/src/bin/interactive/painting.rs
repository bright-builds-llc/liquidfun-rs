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
