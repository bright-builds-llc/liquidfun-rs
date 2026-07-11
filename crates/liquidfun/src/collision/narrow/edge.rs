use crate::collision::shape::{CircleShape, EdgeShape, PolygonShape};
use crate::collision::{CollisionError, ContactFeatureId, FeatureKind, Manifold, ManifoldPoint};
use crate::math::settings::ANGULAR_SLOP;
use crate::math::{Transform, Vec2, min};

use super::clipping::{ClipVertex, clip_segment_to_line};

pub(super) fn collide_edge_circle(
    edge_a: &EdgeShape,
    transform_a: Transform,
    circle_b: &CircleShape,
    transform_b: Transform,
) -> Result<Option<Manifold>, CollisionError> {
    let query = transform_a.inverse_apply(transform_b.apply(circle_b.center()));
    let start = edge_a.start();
    let end = edge_a.end();
    let edge = end - start;
    let coordinate_u = edge.dot(end - query);
    let coordinate_v = edge.dot(query - start);
    let radius = edge_a.radius() + circle_b.radius();

    if coordinate_v <= 0.0 {
        let difference = query - start;
        if difference.dot(difference) > radius * radius {
            return Ok(None);
        }
        if let Some(previous) = edge_a.previous() {
            let previous_edge = start - previous;
            if previous_edge.dot(start - query) > 0.0 {
                return Ok(None);
            }
        }
        let feature = ContactFeatureId::new(0, 0, FeatureKind::Vertex, FeatureKind::Vertex);
        let point = ManifoldPoint::new(circle_b.center(), feature)?;
        return Ok(Some(Manifold::circles(start, point)?));
    }

    if coordinate_u <= 0.0 {
        let difference = query - end;
        if difference.dot(difference) > radius * radius {
            return Ok(None);
        }
        if let Some(next) = edge_a.next() {
            let next_edge = next - end;
            if next_edge.dot(query - end) > 0.0 {
                return Ok(None);
            }
        }
        let feature = ContactFeatureId::new(1, 0, FeatureKind::Vertex, FeatureKind::Vertex);
        let point = ManifoldPoint::new(circle_b.center(), feature)?;
        return Ok(Some(Manifold::circles(end, point)?));
    }

    let denominator = edge.dot(edge);
    let projected = (1.0 / denominator) * (coordinate_u * start + coordinate_v * end);
    let difference = query - projected;
    if difference.dot(difference) > radius * radius {
        return Ok(None);
    }
    let mut normal = Vec2::new(-edge.y, edge.x);
    if normal.dot(query - start) < 0.0 {
        normal = -normal;
    }
    normal.normalize();
    let feature = ContactFeatureId::new(0, 0, FeatureKind::Face, FeatureKind::Vertex);
    let point = ManifoldPoint::new(circle_b.center(), feature)?;
    Ok(Some(Manifold::face_a(normal, start, &[point])?))
}

pub(super) fn collide_edge_polygon(
    edge_a: &EdgeShape,
    transform_a: Transform,
    polygon_b: &PolygonShape,
    transform_b: Transform,
) -> Result<Option<Manifold>, CollisionError> {
    EdgePolygonCollider::new(edge_a, transform_a, polygon_b, transform_b).collide(polygon_b)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AxisKind {
    Unknown,
    EdgeA,
    EdgeB,
}

#[derive(Debug, Clone, Copy)]
struct Axis {
    kind: AxisKind,
    index: usize,
    separation: f32,
}

struct ReferenceFace {
    index1: usize,
    index2: usize,
    vertex1: Vec2,
    vertex2: Vec2,
    normal: Vec2,
    side_normal1: Vec2,
    side_normal2: Vec2,
    side_offset1: f32,
    side_offset2: f32,
}

struct EdgePolygonCollider {
    transform: Transform,
    vertices: Vec<Vec2>,
    normals: Vec<Vec2>,
    vertex1: Vec2,
    vertex2: Vec2,
    normal1: Vec2,
    normal: Vec2,
    lower_limit: Vec2,
    upper_limit: Vec2,
    radius: f32,
    front: bool,
}

impl EdgePolygonCollider {
    fn new(
        edge_a: &EdgeShape,
        transform_a: Transform,
        polygon_b: &PolygonShape,
        transform_b: Transform,
    ) -> Self {
        let transform = transform_a.inverse_compose(transform_b);
        let centroid = transform.apply(polygon_b.centroid());
        let vertex1 = edge_a.start();
        let vertex2 = edge_a.end();
        let mut edge1 = vertex2 - vertex1;
        edge1.normalize();
        let normal1 = edge1.cross_scalar(1.0);
        let offset1 = normal1.dot(centroid - vertex1);

        let maybe_preceding = edge_a.previous().map(|vertex0| {
            let mut edge0 = vertex1 - vertex0;
            edge0.normalize();
            let normal0 = edge0.cross_scalar(1.0);
            (
                normal0,
                edge0.cross(edge1) >= 0.0,
                normal0.dot(centroid - vertex0),
            )
        });
        let maybe_following = edge_a.next().map(|vertex3| {
            let mut edge2 = vertex3 - vertex2;
            edge2.normalize();
            let normal2 = edge2.cross_scalar(1.0);
            (
                normal2,
                edge1.cross(edge2) > 0.0,
                normal2.dot(centroid - vertex2),
            )
        });
        let (front, normal, lower_limit, upper_limit) =
            classify_normals(normal1, offset1, maybe_preceding, maybe_following);
        let vertices = polygon_b
            .vertices()
            .iter()
            .map(|vertex| transform.apply(*vertex))
            .collect();
        let polygon_normals = polygon_b
            .normals()
            .iter()
            .map(|normal| transform.rotation().apply(*normal))
            .collect();
        Self {
            transform,
            vertices,
            normals: polygon_normals,
            vertex1,
            vertex2,
            normal1,
            normal,
            lower_limit,
            upper_limit,
            radius: edge_a.radius() + polygon_b.radius(),
            front,
        }
    }

    fn collide(self, polygon_b: &PolygonShape) -> Result<Option<Manifold>, CollisionError> {
        let edge_axis = self.edge_separation();
        if edge_axis.separation > self.radius {
            return Ok(None);
        }
        let polygon_axis = self.polygon_separation();
        if polygon_axis.kind != AxisKind::Unknown && polygon_axis.separation > self.radius {
            return Ok(None);
        }
        let primary_axis = if polygon_axis.kind == AxisKind::Unknown {
            edge_axis
        } else if uses_polygon_axis(edge_axis.separation, polygon_axis.separation) {
            polygon_axis
        } else {
            edge_axis
        };

        let (incident_edge, mut reference_face) = match primary_axis.kind {
            AxisKind::EdgeA => self.edge_reference(),
            AxisKind::EdgeB => self.polygon_reference(primary_axis),
            AxisKind::Unknown => return Ok(None),
        };
        reference_face.side_normal1 = reference_face.normal.cross_scalar(1.0);
        reference_face.side_normal2 = -reference_face.side_normal1;
        reference_face.side_offset1 = reference_face.side_normal1.dot(reference_face.vertex1);
        reference_face.side_offset2 = reference_face.side_normal2.dot(reference_face.vertex2);

        let clip_points1 = clip_segment_to_line(
            incident_edge,
            reference_face.side_normal1,
            reference_face.side_offset1,
            feature_index(reference_face.index1),
        );
        let [first, second] = clip_points1.as_slice() else {
            return Ok(None);
        };
        let clip_points2 = clip_segment_to_line(
            [*first, *second],
            reference_face.side_normal2,
            reference_face.side_offset2,
            feature_index(reference_face.index2),
        );
        let [first, second] = clip_points2.as_slice() else {
            return Ok(None);
        };

        let mut points = Vec::with_capacity(2);
        for clip_point in [*first, *second] {
            let separation = reference_face
                .normal
                .dot(clip_point.point - reference_face.vertex1);
            if separation <= self.radius {
                let (local_point, feature_id) = if primary_axis.kind == AxisKind::EdgeA {
                    (
                        self.transform.inverse_apply(clip_point.point),
                        clip_point.feature_id,
                    )
                } else {
                    (clip_point.point, swap_feature(clip_point.feature_id))
                };
                points.push(ManifoldPoint::new(local_point, feature_id)?);
            }
        }
        if points.is_empty() {
            return Ok(None);
        }
        if primary_axis.kind == AxisKind::EdgeA {
            Ok(Some(Manifold::face_a(
                reference_face.normal,
                reference_face.vertex1,
                &points,
            )?))
        } else {
            Ok(Some(Manifold::face_b(
                polygon_b.normals()[reference_face.index1],
                polygon_b.vertices()[reference_face.index1],
                &points,
            )?))
        }
    }

    fn edge_separation(&self) -> Axis {
        let mut separation = f32::MAX;
        for vertex in &self.vertices {
            let candidate = self.normal.dot(*vertex - self.vertex1);
            if candidate < separation {
                separation = candidate;
            }
        }
        Axis {
            kind: AxisKind::EdgeA,
            index: usize::from(!self.front),
            separation,
        }
    }

    fn polygon_separation(&self) -> Axis {
        let mut axis = Axis {
            kind: AxisKind::Unknown,
            index: 0,
            separation: -f32::MAX,
        };
        let perpendicular = Vec2::new(-self.normal.y, self.normal.x);
        for (index, (vertex, polygon_normal)) in self.vertices.iter().zip(&self.normals).enumerate()
        {
            let normal = -*polygon_normal;
            let separation1 = normal.dot(*vertex - self.vertex1);
            let separation2 = normal.dot(*vertex - self.vertex2);
            let separation = min(separation1, separation2);
            if separation > self.radius {
                return Axis {
                    kind: AxisKind::EdgeB,
                    index,
                    separation,
                };
            }
            if normal.dot(perpendicular) >= 0.0 {
                if (normal - self.upper_limit).dot(self.normal) < -ANGULAR_SLOP {
                    continue;
                }
            } else if (normal - self.lower_limit).dot(self.normal) < -ANGULAR_SLOP {
                continue;
            }
            if separation > axis.separation {
                axis = Axis {
                    kind: AxisKind::EdgeB,
                    index,
                    separation,
                };
            }
        }
        axis
    }

    fn edge_reference(&self) -> ([ClipVertex; 2], ReferenceFace) {
        let mut best_index = 0;
        let mut best_value = self.normal.dot(self.normals[0]);
        for (index, normal) in self.normals.iter().enumerate().skip(1) {
            let value = self.normal.dot(*normal);
            if value < best_value {
                best_value = value;
                best_index = index;
            }
        }
        let next_index = if best_index + 1 < self.vertices.len() {
            best_index + 1
        } else {
            0
        };
        let incident_edge = [
            clip_vertex(
                0,
                best_index,
                FeatureKind::Face,
                FeatureKind::Vertex,
                self.vertices[best_index],
            ),
            clip_vertex(
                0,
                next_index,
                FeatureKind::Face,
                FeatureKind::Vertex,
                self.vertices[next_index],
            ),
        ];
        let (index1, index2, vertex1, vertex2, normal) = if self.front {
            (0, 1, self.vertex1, self.vertex2, self.normal1)
        } else {
            (1, 0, self.vertex2, self.vertex1, -self.normal1)
        };
        (
            incident_edge,
            ReferenceFace::new(index1, index2, vertex1, vertex2, normal),
        )
    }

    fn polygon_reference(&self, primary_axis: Axis) -> ([ClipVertex; 2], ReferenceFace) {
        let incident_edge = [
            clip_vertex(
                0,
                primary_axis.index,
                FeatureKind::Vertex,
                FeatureKind::Face,
                self.vertex1,
            ),
            clip_vertex(
                0,
                primary_axis.index,
                FeatureKind::Vertex,
                FeatureKind::Face,
                self.vertex2,
            ),
        ];
        let index1 = primary_axis.index;
        let index2 = if index1 + 1 < self.vertices.len() {
            index1 + 1
        } else {
            0
        };
        (
            incident_edge,
            ReferenceFace::new(
                index1,
                index2,
                self.vertices[index1],
                self.vertices[index2],
                self.normals[index1],
            ),
        )
    }
}

impl ReferenceFace {
    fn new(index1: usize, index2: usize, vertex1: Vec2, vertex2: Vec2, normal: Vec2) -> Self {
        Self {
            index1,
            index2,
            vertex1,
            vertex2,
            normal,
            side_normal1: Vec2::ZERO,
            side_normal2: Vec2::ZERO,
            side_offset1: 0.0,
            side_offset2: 0.0,
        }
    }
}

fn classify_normals(
    normal1: Vec2,
    offset1: f32,
    maybe_preceding: Option<(Vec2, bool, f32)>,
    maybe_following: Option<(Vec2, bool, f32)>,
) -> (bool, Vec2, Vec2, Vec2) {
    match (maybe_preceding, maybe_following) {
        (Some((normal0, convex1, offset0)), Some((normal2, convex2, offset2))) => classify_both(
            normal0, normal1, normal2, convex1, convex2, offset0, offset1, offset2,
        ),
        (Some((normal0, convex1, offset0)), None) => {
            classify_preceding(normal0, normal1, convex1, offset0, offset1)
        }
        (None, Some((normal2, convex2, offset2))) => {
            classify_following(normal1, normal2, convex2, offset1, offset2)
        }
        (None, None) => {
            let front = offset1 >= 0.0;
            if front {
                (front, normal1, -normal1, -normal1)
            } else {
                (front, -normal1, normal1, normal1)
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn classify_both(
    normal0: Vec2,
    normal1: Vec2,
    normal2: Vec2,
    convex1: bool,
    convex2: bool,
    offset0: f32,
    offset1: f32,
    offset2: f32,
) -> (bool, Vec2, Vec2, Vec2) {
    if convex1 && convex2 {
        let front = offset0 >= 0.0 || offset1 >= 0.0 || offset2 >= 0.0;
        if front {
            (front, normal1, normal0, normal2)
        } else {
            (front, -normal1, -normal1, -normal1)
        }
    } else if convex1 {
        let front = offset0 >= 0.0 || (offset1 >= 0.0 && offset2 >= 0.0);
        if front {
            (front, normal1, normal0, normal1)
        } else {
            (front, -normal1, -normal2, -normal1)
        }
    } else if convex2 {
        let front = offset2 >= 0.0 || (offset0 >= 0.0 && offset1 >= 0.0);
        if front {
            (front, normal1, normal1, normal2)
        } else {
            (front, -normal1, -normal1, -normal0)
        }
    } else {
        let front = offset0 >= 0.0 && offset1 >= 0.0 && offset2 >= 0.0;
        if front {
            (front, normal1, normal1, normal1)
        } else {
            (front, -normal1, -normal2, -normal0)
        }
    }
}

fn classify_preceding(
    normal0: Vec2,
    normal1: Vec2,
    convex1: bool,
    offset0: f32,
    offset1: f32,
) -> (bool, Vec2, Vec2, Vec2) {
    if convex1 {
        let front = offset0 >= 0.0 || offset1 >= 0.0;
        if front {
            (front, normal1, normal0, -normal1)
        } else {
            (front, -normal1, normal1, -normal1)
        }
    } else {
        let front = offset0 >= 0.0 && offset1 >= 0.0;
        if front {
            (front, normal1, normal1, -normal1)
        } else {
            (front, -normal1, normal1, -normal0)
        }
    }
}

fn classify_following(
    normal1: Vec2,
    normal2: Vec2,
    convex2: bool,
    offset1: f32,
    offset2: f32,
) -> (bool, Vec2, Vec2, Vec2) {
    if convex2 {
        let front = offset1 >= 0.0 || offset2 >= 0.0;
        if front {
            (front, normal1, -normal1, normal2)
        } else {
            (front, -normal1, -normal1, normal1)
        }
    } else {
        let front = offset1 >= 0.0 && offset2 >= 0.0;
        if front {
            (front, normal1, -normal1, normal1)
        } else {
            (front, -normal1, -normal2, normal1)
        }
    }
}

fn clip_vertex(
    index_a: usize,
    index_b: usize,
    kind_a: FeatureKind,
    kind_b: FeatureKind,
    point: Vec2,
) -> ClipVertex {
    ClipVertex {
        point,
        feature_id: ContactFeatureId::new(
            feature_index(index_a),
            feature_index(index_b),
            kind_a,
            kind_b,
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

fn feature_index(index: usize) -> u8 {
    u8::try_from(index).expect("validated collision features fit in eight bits")
}

fn uses_polygon_axis(edge_separation: f32, polygon_separation: f32) -> bool {
    polygon_separation > 0.98 * edge_separation + 0.001
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_hysteresis_keeps_edge_at_equality_and_selects_polygon_above() {
        // Arrange
        let edge_separation = 0.25_f32;
        let boundary = 0.98_f32 * edge_separation + 0.001_f32;
        let below_boundary = f32::from_bits(boundary.to_bits() - 1);
        let above_boundary = f32::from_bits(boundary.to_bits() + 1);

        // Act
        let below_uses_polygon = uses_polygon_axis(edge_separation, below_boundary);
        let equality_uses_polygon = uses_polygon_axis(edge_separation, boundary);
        let above_uses_polygon = uses_polygon_axis(edge_separation, above_boundary);

        // Assert
        assert!(!below_uses_polygon);
        assert!(!equality_uses_polygon);
        assert!(above_uses_polygon);
    }

    #[test]
    fn isolated_edge_classification_flips_normal_across_the_edge() {
        // Arrange
        let normal = Vec2::new(0.0, -1.0);

        // Act
        let front = classify_normals(normal, 0.5, None, None);
        let back = classify_normals(normal, -0.5, None, None);

        // Assert
        assert_eq!(front, (true, normal, -normal, -normal));
        assert_eq!(back, (false, -normal, normal, normal));
    }
}
