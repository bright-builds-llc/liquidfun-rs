use crate::collision::shape::{CircleShape, PolygonShape};
use crate::collision::{CollisionError, ContactFeatureId, FeatureKind, Manifold, ManifoldPoint};
use crate::math::Transform;
use crate::math::settings::EPSILON;

pub(super) fn collide_circles(
    circle_a: &CircleShape,
    transform_a: Transform,
    circle_b: &CircleShape,
    transform_b: Transform,
) -> Result<Option<Manifold>, CollisionError> {
    let point_a = transform_a.apply(circle_a.center());
    let point_b = transform_b.apply(circle_b.center());
    let difference = point_b - point_a;
    let distance_squared = difference.dot(difference);
    let radius = circle_a.radius() + circle_b.radius();
    if distance_squared > radius * radius {
        return Ok(None);
    }

    let point = ManifoldPoint::new(circle_b.center(), zero_feature())?;
    Ok(Some(Manifold::circles(circle_a.center(), point)?))
}

pub(super) fn collide_polygon_circle(
    polygon_a: &PolygonShape,
    transform_a: Transform,
    circle_b: &CircleShape,
    transform_b: Transform,
) -> Result<Option<Manifold>, CollisionError> {
    let center = transform_b.apply(circle_b.center());
    let local_center = transform_a.inverse_apply(center);
    let radius = polygon_a.radius() + circle_b.radius();
    let mut normal_index = 0;
    let mut separation = -f32::MAX;

    for (index, (normal, vertex)) in polygon_a
        .normals()
        .iter()
        .zip(polygon_a.vertices())
        .enumerate()
    {
        let candidate = normal.dot(local_center - *vertex);
        if candidate > radius {
            return Ok(None);
        }
        if candidate > separation {
            separation = candidate;
            normal_index = index;
        }
    }

    let vertex_index1 = normal_index;
    let vertex_index2 = if vertex_index1 + 1 < polygon_a.vertex_count() {
        vertex_index1 + 1
    } else {
        0
    };
    let vertex1 = polygon_a.vertices()[vertex_index1];
    let vertex2 = polygon_a.vertices()[vertex_index2];
    let point = ManifoldPoint::new(circle_b.center(), zero_feature())?;

    if separation < EPSILON {
        return Ok(Some(Manifold::face_a(
            polygon_a.normals()[normal_index],
            0.5 * (vertex1 + vertex2),
            &[point],
        )?));
    }

    let coordinate1 = (local_center - vertex1).dot(vertex2 - vertex1);
    let coordinate2 = (local_center - vertex2).dot(vertex1 - vertex2);
    if coordinate1 <= 0.0 {
        if (local_center - vertex1).length_squared() > radius * radius {
            return Ok(None);
        }
        let mut normal = local_center - vertex1;
        normal.normalize();
        return Ok(Some(Manifold::face_a(normal, vertex1, &[point])?));
    }
    if coordinate2 <= 0.0 {
        if (local_center - vertex2).length_squared() > radius * radius {
            return Ok(None);
        }
        let mut normal = local_center - vertex2;
        normal.normalize();
        return Ok(Some(Manifold::face_a(normal, vertex2, &[point])?));
    }

    let face_center = 0.5 * (vertex1 + vertex2);
    let face_separation = (local_center - face_center).dot(polygon_a.normals()[vertex_index1]);
    if face_separation > radius {
        return Ok(None);
    }
    Ok(Some(Manifold::face_a(
        polygon_a.normals()[vertex_index1],
        face_center,
        &[point],
    )?))
}

const fn zero_feature() -> ContactFeatureId {
    ContactFeatureId::new(0, 0, FeatureKind::Vertex, FeatureKind::Vertex)
}
