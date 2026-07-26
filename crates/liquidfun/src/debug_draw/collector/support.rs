//! Shared finite-geometry, identity, and style helpers for debug collection.

use crate::collision::Shape;
use crate::math::Vec2;
use crate::{BodySnapshot, BodyType, FixtureObservation, ParticleObservation, WorldObservation};

use super::{DebugCollectionError, DebugCollectionResource};
use crate::debug_draw::primitive::{
    DebugColor, DebugFill, DebugLayer, DebugOwnerKey, DebugPrimitive, DebugPrimitiveKey,
    DebugPrimitiveKind, DebugPrimitiveMetadata, DebugStroke,
};

pub(super) const fn layer_index(layer: DebugLayer) -> usize {
    match layer {
        DebugLayer::Shapes => 0,
        DebugLayer::Joints => 1,
        DebugLayer::Contacts => 2,
        DebugLayer::ContactNormals => 3,
        DebugLayer::Particles => 4,
        DebugLayer::ParticleContacts => 5,
        DebugLayer::BroadPhase => 6,
        DebugLayer::CentersOfMass => 7,
        DebugLayer::Labels => 8,
    }
}

pub(super) fn shape_kind(shape: &Shape) -> DebugPrimitiveKind {
    match shape {
        Shape::Circle(_) => DebugPrimitiveKind::Circle,
        Shape::Edge(_) => DebugPrimitiveKind::Segment,
        Shape::Polygon(_) | Shape::Chain(_) => DebugPrimitiveKind::Polyline,
    }
}

pub(super) fn metadata(
    key: DebugPrimitiveKey,
    color: DebugColor,
    filled: bool,
) -> DebugPrimitiveMetadata {
    let stroke = DebugStroke::new(color, 0.01).expect("constant debug stroke is valid");
    let maybe_fill = filled.then(|| DebugFill::new(color));
    DebugPrimitiveMetadata::new(key, stroke, maybe_fill)
}

pub(super) fn segment(
    owner: DebugOwnerKey,
    layer: DebugLayer,
    ordinal: usize,
    start: Vec2,
    end: Vec2,
    color: DebugColor,
) -> Result<DebugPrimitive, DebugCollectionError> {
    Ok(DebugPrimitive::Segment {
        metadata: metadata(
            DebugPrimitiveKey::new(
                owner,
                layer,
                DebugPrimitiveKind::Segment,
                0,
                checked_u32(ordinal)?,
            ),
            color,
            false,
        ),
        start,
        end,
    })
}

pub(super) fn body_snapshot(
    observation: &WorldObservation,
    id: crate::BodyId,
) -> Result<BodySnapshot, DebugCollectionError> {
    observation
        .bodies()
        .iter()
        .find(|body| body.id() == id)
        .map(|body| body.snapshot())
        .ok_or(DebugCollectionError::IncompleteOwner)
}

pub(super) fn fixture_observation(
    observation: &WorldObservation,
    id: crate::FixtureId,
) -> Result<&FixtureObservation, DebugCollectionError> {
    observation
        .fixtures()
        .iter()
        .find(|fixture| fixture.id() == id)
        .ok_or(DebugCollectionError::IncompleteOwner)
}

pub(super) fn particle_observation(
    observation: &WorldObservation,
    id: crate::ParticleId,
) -> Result<ParticleObservation, DebugCollectionError> {
    observation
        .particles()
        .iter()
        .copied()
        .find(|particle| particle.particle() == id)
        .ok_or(DebugCollectionError::IncompleteOwner)
}

pub(super) fn body_color(body: BodySnapshot) -> DebugColor {
    if !body.is_active() {
        return DebugColor::rgba(128, 128, 77, 255);
    }
    match body.body_type() {
        BodyType::Static => DebugColor::rgba(128, 230, 128, 255),
        BodyType::Kinematic => DebugColor::rgba(128, 128, 230, 255),
        BodyType::Dynamic if !body.is_awake() => DebugColor::rgba(153, 153, 153, 255),
        BodyType::Dynamic => DebugColor::rgba(230, 179, 179, 255),
    }
}

pub(super) fn validate_primitive(primitive: &DebugPrimitive) -> Result<(), DebugCollectionError> {
    let valid = match primitive {
        DebugPrimitive::Point {
            position, radius, ..
        } => position.is_valid() && radius.is_finite() && *radius >= 0.0,
        DebugPrimitive::Segment { start, end, .. } | DebugPrimitive::Arrow { start, end, .. } => {
            start.is_valid() && end.is_valid()
        }
        DebugPrimitive::Polyline { vertices, .. } => {
            !vertices.is_empty() && vertices.iter().all(|vertex| vertex.is_valid())
        }
        DebugPrimitive::Circle { center, radius, .. } => {
            center.is_valid() && radius.is_finite() && *radius >= 0.0
        }
        DebugPrimitive::TransformAxes {
            transform, scale, ..
        } => {
            transform.position().is_valid()
                && transform.rotation().sine().is_finite()
                && transform.rotation().cosine().is_finite()
                && scale.is_finite()
                && *scale >= 0.0
        }
        DebugPrimitive::Aabb { bounds, .. } => {
            bounds.lower_bound().is_valid() && bounds.upper_bound().is_valid()
        }
        DebugPrimitive::Label { position, text, .. } => {
            position.is_valid()
                && !text.is_empty()
                && text.chars().all(|character| !character.is_control())
        }
    };
    if valid {
        return Ok(());
    }
    Err(DebugCollectionError::InvalidGeometry {
        layer: primitive.layer(),
    })
}

pub(super) fn primitive_vertex_count(primitive: &DebugPrimitive) -> usize {
    match primitive {
        DebugPrimitive::Point { .. }
        | DebugPrimitive::Circle { .. }
        | DebugPrimitive::TransformAxes { .. }
        | DebugPrimitive::Aabb { .. }
        | DebugPrimitive::Label { .. } => 0,
        DebugPrimitive::Segment { .. } | DebugPrimitive::Arrow { .. } => 2,
        DebugPrimitive::Polyline { vertices, .. } => vertices.len(),
    }
}

pub(super) fn primitive_text_bytes(primitive: &DebugPrimitive) -> usize {
    match primitive {
        DebugPrimitive::Label { text, .. } => text.len(),
        _ => 0,
    }
}

pub(super) fn check_limit(
    resource: DebugCollectionResource,
    count: usize,
    limit: usize,
) -> Result<(), DebugCollectionError> {
    if count > limit {
        return Err(DebugCollectionError::CapacityExceeded { resource, limit });
    }
    Ok(())
}

pub(super) fn checked_u32(value: usize) -> Result<u32, DebugCollectionError> {
    u32::try_from(value).map_err(|_error| DebugCollectionError::InvalidGeometry {
        layer: DebugLayer::Labels,
    })
}
