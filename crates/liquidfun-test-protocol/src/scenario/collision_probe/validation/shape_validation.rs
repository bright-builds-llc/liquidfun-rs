use super::{
    CollisionCacheBits, CollisionProbeDecodeError, CollisionProbeErrorKind,
    CollisionProxyFingerprint, CollisionRejectionCategory, CollisionRejectionField,
    CollisionShapeDefinition, CollisionShapeKind, RawCache, RawProxyFingerprint, RawShape,
    Vec2Bits, validate_child, validate_finite, validate_id, validate_vec2, validation,
};

pub(super) fn validate_shape(
    raw: RawShape,
) -> Result<CollisionShapeDefinition, CollisionProbeDecodeError> {
    let shape = raw_shape_definition(raw)?;
    validate_production_shape(&shape)?;
    Ok(shape)
}

fn raw_shape_definition(
    raw: RawShape,
) -> Result<CollisionShapeDefinition, CollisionProbeDecodeError> {
    let shape = match raw {
        RawShape::Circle {
            shape_id,
            center,
            radius_bits,
        } => CollisionShapeDefinition::Circle {
            shape_id: validate_id(shape_id)?,
            center,
            radius_bits,
        },
        RawShape::Edge {
            shape_id,
            start,
            end,
            maybe_previous,
            maybe_next,
        } => CollisionShapeDefinition::Edge {
            shape_id: validate_id(shape_id)?,
            start,
            end,
            maybe_previous,
            maybe_next,
        },
        RawShape::Polygon { shape_id, vertices } => {
            let vertices = vertices.into_vec();
            CollisionShapeDefinition::Polygon {
                shape_id: validate_id(shape_id)?,
                vertices: vertices.into_boxed_slice(),
            }
        }
        RawShape::Chain {
            shape_id,
            vertices,
            closed,
            maybe_previous,
            maybe_next,
        } => {
            let vertices = vertices.into_vec();
            CollisionShapeDefinition::Chain {
                shape_id: validate_id(shape_id)?,
                vertices: vertices.into_boxed_slice(),
                closed,
                maybe_previous,
                maybe_next,
            }
        }
    };
    Ok(shape)
}

fn validate_production_shape(
    shape: &CollisionShapeDefinition,
) -> Result<(), CollisionProbeDecodeError> {
    use liquidfun::collision::{ChainShape, CircleShape, EdgeShape, PolygonShape};

    let result = match shape {
        CollisionShapeDefinition::Circle {
            center,
            radius_bits,
            ..
        } => CircleShape::new(production_vec2(*center), radius_bits.to_f32()).map(|_| ()),
        CollisionShapeDefinition::Edge {
            start,
            end,
            maybe_previous,
            maybe_next,
            ..
        } => EdgeShape::with_adjacency(
            production_vec2(*start),
            production_vec2(*end),
            maybe_previous.map(production_vec2),
            maybe_next.map(production_vec2),
        )
        .map(|_| ()),
        CollisionShapeDefinition::Polygon { vertices, .. } => PolygonShape::new(
            &vertices
                .iter()
                .copied()
                .map(production_vec2)
                .collect::<Vec<_>>(),
        )
        .map(|_| ()),
        CollisionShapeDefinition::Chain {
            vertices,
            closed,
            maybe_previous,
            maybe_next,
            ..
        } => {
            let vertices = vertices
                .iter()
                .copied()
                .map(production_vec2)
                .collect::<Vec<_>>();
            if *closed {
                if maybe_previous.is_some() || maybe_next.is_some() {
                    return Err(validation(CollisionProbeErrorKind::InvalidGeometry));
                }
                ChainShape::closed(&vertices)
            } else {
                ChainShape::open(
                    &vertices,
                    maybe_previous.map(production_vec2),
                    maybe_next.map(production_vec2),
                )
            }
            .map(|_| ())
        }
    };
    result.map_err(|_| validation(CollisionProbeErrorKind::InvalidGeometry))
}

pub(super) fn validate_rejected_shape(
    raw: RawShape,
    child_index: u32,
    category: CollisionRejectionCategory,
    field: CollisionRejectionField,
) -> Result<CollisionShapeDefinition, CollisionProbeDecodeError> {
    let shape = raw_shape_definition(raw)?;
    let classified = if validate_production_shape(&shape).is_ok() {
        validate_child(&shape, child_index).is_err().then_some((
            CollisionRejectionCategory::InvalidChildIndex,
            CollisionRejectionField::ChildIndex,
        ))
    } else {
        classify_rejected_shape(&shape)
    };
    if classified != Some((category, field)) {
        return Err(validation(CollisionProbeErrorKind::InvalidGeometry));
    }
    Ok(shape)
}

fn classify_rejected_shape(
    shape: &CollisionShapeDefinition,
) -> Option<(CollisionRejectionCategory, CollisionRejectionField)> {
    use CollisionRejectionCategory::{InvalidGeometry, NonFiniteValue};
    use CollisionRejectionField::{
        ChainVertices, CircleCenter, CircleRadius, EdgeEnd, EdgeNext, EdgePrevious, EdgeStart,
        PolygonVertices,
    };
    match shape {
        CollisionShapeDefinition::Circle {
            center,
            radius_bits,
            ..
        } => {
            if !vec2_is_finite(*center) {
                Some((NonFiniteValue, CircleCenter))
            } else if !radius_bits.to_f32().is_finite() {
                Some((NonFiniteValue, CircleRadius))
            } else if radius_bits.to_f32() < 0.0 {
                Some((InvalidGeometry, CircleRadius))
            } else {
                None
            }
        }
        CollisionShapeDefinition::Edge {
            start,
            end,
            maybe_previous,
            maybe_next,
            ..
        } => {
            if !vec2_is_finite(*start) {
                Some((NonFiniteValue, EdgeStart))
            } else if !vec2_is_finite(*end) {
                Some((NonFiniteValue, EdgeEnd))
            } else if start == end {
                Some((InvalidGeometry, EdgeEnd))
            } else if maybe_previous.is_some_and(|point| !vec2_is_finite(point)) {
                Some((NonFiniteValue, EdgePrevious))
            } else if maybe_previous == &Some(*start) {
                Some((InvalidGeometry, EdgePrevious))
            } else if maybe_next.is_some_and(|point| !vec2_is_finite(point)) {
                Some((NonFiniteValue, EdgeNext))
            } else if maybe_next == &Some(*end) {
                Some((InvalidGeometry, EdgeNext))
            } else {
                None
            }
        }
        CollisionShapeDefinition::Polygon { vertices, .. } => {
            (!vertices.iter().copied().all(vec2_is_finite))
                .then_some((NonFiniteValue, PolygonVertices))
                .or(Some((InvalidGeometry, PolygonVertices)))
        }
        CollisionShapeDefinition::Chain {
            vertices,
            closed,
            maybe_previous,
            maybe_next,
            ..
        } => {
            if !vertices.iter().copied().all(vec2_is_finite) {
                Some((NonFiniteValue, ChainVertices))
            } else if vertices.len() < if *closed { 3 } else { 2 }
                || vertices
                    .windows(2)
                    .any(|pair| points_too_close(pair[0], pair[1]))
                || (*closed
                    && points_too_close(vertices[vertices.len().saturating_sub(1)], vertices[0]))
            {
                Some((InvalidGeometry, ChainVertices))
            } else if maybe_previous.is_some_and(|point| !vec2_is_finite(point)) {
                Some((NonFiniteValue, EdgePrevious))
            } else if maybe_previous
                .is_some_and(|point| *closed || points_too_close(point, vertices[0]))
            {
                Some((InvalidGeometry, EdgePrevious))
            } else if maybe_next.is_some_and(|point| !vec2_is_finite(point)) {
                Some((NonFiniteValue, EdgeNext))
            } else if maybe_next.is_some_and(|point| {
                *closed || points_too_close(point, vertices[vertices.len() - 1])
            }) {
                Some((InvalidGeometry, EdgeNext))
            } else {
                None
            }
        }
    }
}

fn production_vec2(value: Vec2Bits) -> liquidfun::math::Vec2 {
    liquidfun::math::Vec2::new(value.x_bits.to_f32(), value.y_bits.to_f32())
}

fn vec2_is_finite(value: Vec2Bits) -> bool {
    value.x_bits.to_f32().is_finite() && value.y_bits.to_f32().is_finite()
}

fn points_too_close(first: Vec2Bits, second: Vec2Bits) -> bool {
    let offset = production_vec2(second) - production_vec2(first);
    offset.length_squared()
        <= liquidfun::math::settings::LINEAR_SLOP * liquidfun::math::settings::LINEAR_SLOP
}

pub(super) fn validate_cache(
    raw: RawCache,
) -> Result<CollisionCacheBits, CollisionProbeDecodeError> {
    let proxy_a = validate_proxy_fingerprint(raw.proxy_a)?;
    let proxy_b = validate_proxy_fingerprint(raw.proxy_b)?;
    Ok(CollisionCacheBits {
        proxy_a,
        proxy_b,
        support_pairs: raw.support_pairs.into_vec().into_boxed_slice(),
        metric_bits: raw.metric_bits,
    })
}

fn validate_proxy_fingerprint(
    raw: RawProxyFingerprint,
) -> Result<CollisionProxyFingerprint, CollisionProbeDecodeError> {
    validate_finite(raw.radius_bits)?;
    if raw.radius_bits.to_f32() < 0.0 {
        return Err(validation(CollisionProbeErrorKind::InvalidGeometry));
    }
    let vertices = raw.vertices.into_vec();
    for vertex in &vertices {
        validate_vec2(*vertex)?;
    }
    let valid_topology = match raw.shape_kind {
        CollisionShapeKind::Circle => vertices.len() == 1 && raw.child_index == 0,
        CollisionShapeKind::Edge => vertices.len() == 2 && raw.child_index == 0,
        CollisionShapeKind::Polygon => (3..=8).contains(&vertices.len()) && raw.child_index == 0,
        CollisionShapeKind::Chain => {
            vertices.len() >= 2
                && usize::try_from(raw.child_index)
                    .ok()
                    .is_some_and(|index| index + 1 < vertices.len())
        }
    };
    if !valid_topology {
        return Err(validation(CollisionProbeErrorKind::InvalidChildIndex));
    }
    Ok(CollisionProxyFingerprint {
        shape_kind: raw.shape_kind,
        child_index: raw.child_index,
        radius_bits: raw.radius_bits,
        vertices: vertices.into_boxed_slice(),
    })
}
