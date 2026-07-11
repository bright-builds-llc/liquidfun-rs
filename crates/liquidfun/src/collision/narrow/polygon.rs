use crate::collision::shape::PolygonShape;
use crate::collision::{CollisionError, ContactFeatureId, FeatureKind, Manifold, ManifoldPoint};
use crate::math::settings::LINEAR_SLOP;
use crate::math::{Transform, Vec2};

use super::clipping::{ClipVertex, clip_segment_to_line};

pub(super) fn collide_polygons(
    polygon_a: &PolygonShape,
    transform_a: Transform,
    polygon_b: &PolygonShape,
    transform_b: Transform,
) -> Result<Option<Manifold>, CollisionError> {
    let total_radius = polygon_a.radius() + polygon_b.radius();
    let (edge_a, separation_a) =
        find_max_separation(polygon_a, transform_a, polygon_b, transform_b);
    if separation_a > total_radius {
        return Ok(None);
    }
    let (edge_b, separation_b) =
        find_max_separation(polygon_b, transform_b, polygon_a, transform_a);
    if separation_b > total_radius {
        return Ok(None);
    }

    let (reference, incident, reference_transform, incident_transform, reference_edge, flip) =
        if uses_polygon_b_reference(separation_a, separation_b) {
            (polygon_b, polygon_a, transform_b, transform_a, edge_b, true)
        } else {
            (
                polygon_a,
                polygon_b,
                transform_a,
                transform_b,
                edge_a,
                false,
            )
        };

    let incident_edge = find_incident_edge(
        reference,
        reference_transform,
        reference_edge,
        incident,
        incident_transform,
    );
    let vertex_index1 = reference_edge;
    let vertex_index2 = if reference_edge + 1 < reference.vertex_count() {
        reference_edge + 1
    } else {
        0
    };
    let local_vertex1 = reference.vertices()[vertex_index1];
    let local_vertex2 = reference.vertices()[vertex_index2];
    let mut local_tangent = local_vertex2 - local_vertex1;
    local_tangent.normalize();
    let local_normal = local_tangent.cross_scalar(1.0);
    let plane_point = 0.5 * (local_vertex1 + local_vertex2);
    let tangent = reference_transform.rotation().apply(local_tangent);
    let normal = tangent.cross_scalar(1.0);
    let vertex1 = reference_transform.apply(local_vertex1);
    let vertex2 = reference_transform.apply(local_vertex2);
    let front_offset = normal.dot(vertex1);
    let side_offset1 = -tangent.dot(vertex1) + total_radius;
    let side_offset2 = tangent.dot(vertex2) + total_radius;

    let clip_points1 = clip_segment_to_line(
        incident_edge,
        -tangent,
        side_offset1,
        to_feature_index(vertex_index1),
    );
    let [first, second] = clip_points1.as_slice() else {
        return Ok(None);
    };
    let clip_points2 = clip_segment_to_line(
        [*first, *second],
        tangent,
        side_offset2,
        to_feature_index(vertex_index2),
    );
    let [first, second] = clip_points2.as_slice() else {
        return Ok(None);
    };

    let mut points = Vec::with_capacity(2);
    for clip_point in [*first, *second] {
        let separation = normal.dot(clip_point.point) - front_offset;
        if separation <= total_radius {
            let feature_id = if flip {
                swap_feature(clip_point.feature_id)
            } else {
                clip_point.feature_id
            };
            points.push(ManifoldPoint::new(
                incident_transform.inverse_apply(clip_point.point),
                feature_id,
            )?);
        }
    }
    if points.is_empty() {
        return Ok(None);
    }
    if flip {
        Ok(Some(Manifold::face_b(local_normal, plane_point, &points)?))
    } else {
        Ok(Some(Manifold::face_a(local_normal, plane_point, &points)?))
    }
}

fn find_max_separation(
    polygon1: &PolygonShape,
    transform1: Transform,
    polygon2: &PolygonShape,
    transform2: Transform,
) -> (usize, f32) {
    let transform = transform2.inverse_compose(transform1);
    let mut best_index = 0;
    let mut maximum_separation = -f32::MAX;
    for (index, (normal1, vertex1)) in polygon1
        .normals()
        .iter()
        .zip(polygon1.vertices())
        .enumerate()
    {
        let normal = transform.rotation().apply(*normal1);
        let vertex = transform.apply(*vertex1);
        let mut separation = f32::MAX;
        for vertex2 in polygon2.vertices() {
            let candidate = normal.dot(*vertex2 - vertex);
            if candidate < separation {
                separation = candidate;
            }
        }
        if separation > maximum_separation {
            maximum_separation = separation;
            best_index = index;
        }
    }
    (best_index, maximum_separation)
}

fn find_incident_edge(
    reference: &PolygonShape,
    reference_transform: Transform,
    reference_edge: usize,
    incident: &PolygonShape,
    incident_transform: Transform,
) -> [ClipVertex; 2] {
    let reference_normal = incident_transform.rotation().inverse_apply(
        reference_transform
            .rotation()
            .apply(reference.normals()[reference_edge]),
    );
    let mut incident_index = 0;
    let mut minimum_dot = f32::MAX;
    for (index, normal) in incident.normals().iter().enumerate() {
        let dot = reference_normal.dot(*normal);
        if dot < minimum_dot {
            minimum_dot = dot;
            incident_index = index;
        }
    }
    let next_index = if incident_index + 1 < incident.vertex_count() {
        incident_index + 1
    } else {
        0
    };
    [
        incident_vertex(
            incident_transform.apply(incident.vertices()[incident_index]),
            reference_edge,
            incident_index,
        ),
        incident_vertex(
            incident_transform.apply(incident.vertices()[next_index]),
            reference_edge,
            next_index,
        ),
    ]
}

fn incident_vertex(point: Vec2, reference_edge: usize, incident_vertex: usize) -> ClipVertex {
    ClipVertex {
        point,
        feature_id: ContactFeatureId::new(
            to_feature_index(reference_edge),
            to_feature_index(incident_vertex),
            FeatureKind::Face,
            FeatureKind::Vertex,
        ),
    }
}

fn swap_feature(feature: ContactFeatureId) -> ContactFeatureId {
    ContactFeatureId::new(
        feature.index_b(),
        feature.index_a(),
        feature.kind_b(),
        feature.kind_a(),
    )
}

fn to_feature_index(index: usize) -> u8 {
    u8::try_from(index).expect("validated polygons have at most eight vertices")
}

fn uses_polygon_b_reference(separation_a: f32, separation_b: f32) -> bool {
    separation_b > separation_a + 0.1 * LINEAR_SLOP
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_hysteresis_keeps_a_at_equality_and_selects_b_strictly_above() {
        // Arrange
        let separation_a = 0.25;
        let boundary = separation_a + 0.1 * LINEAR_SLOP;
        let above_boundary = f32::from_bits(boundary.to_bits() + 1);

        // Act
        let below_uses_b = uses_polygon_b_reference(separation_a, separation_a);
        let equality_uses_b = uses_polygon_b_reference(separation_a, boundary);
        let above_uses_b = uses_polygon_b_reference(separation_a, above_boundary);

        // Assert
        assert!(!below_uses_b);
        assert!(!equality_uses_b);
        assert!(above_uses_b);
    }
}
