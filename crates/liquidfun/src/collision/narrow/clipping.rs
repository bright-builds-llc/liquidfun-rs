use crate::collision::{ContactFeatureId, FeatureKind};
use crate::math::Vec2;

#[allow(
    dead_code,
    reason = "the private clip kernel is consumed by the next manifold task"
)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ClipVertex {
    pub(super) point: Vec2,
    pub(super) feature_id: ContactFeatureId,
}

#[allow(
    dead_code,
    reason = "the private clip kernel is consumed by the next manifold task"
)]
pub(super) fn clip_segment_to_line(
    input: [ClipVertex; 2],
    normal: Vec2,
    offset: f32,
    vertex_index_a: u8,
) -> Vec<ClipVertex> {
    let mut output = Vec::with_capacity(2);
    let distance0 = normal.dot(input[0].point) - offset;
    let distance1 = normal.dot(input[1].point) - offset;

    if distance0 <= 0.0 {
        output.push(input[0]);
    }
    if distance1 <= 0.0 {
        output.push(input[1]);
    }
    if distance0 * distance1 < 0.0 {
        let interpolation = distance0 / (distance0 - distance1);
        output.push(ClipVertex {
            point: input[0].point + interpolation * (input[1].point - input[0].point),
            feature_id: ContactFeatureId::new(
                vertex_index_a,
                input[0].feature_id.index_b(),
                FeatureKind::Vertex,
                FeatureKind::Face,
            ),
        });
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vertex(x: f32, index_b: u8) -> ClipVertex {
        ClipVertex {
            point: Vec2::new(x, 0.0),
            feature_id: ContactFeatureId::new(4, index_b, FeatureKind::Face, FeatureKind::Vertex),
        }
    }

    #[test]
    fn clipping_retains_inside_vertices_in_input_order() {
        // Arrange
        let input = [vertex(-2.0, 2), vertex(-1.0, 3)];

        // Act
        let output = clip_segment_to_line(input, Vec2::new(1.0, 0.0), 0.0, 7);

        // Assert
        assert_eq!(output, input);
    }

    #[test]
    fn clipping_appends_crossing_after_retained_vertex() {
        // Arrange
        let input = [vertex(-1.0, 2), vertex(1.0, 3)];

        // Act
        let output = clip_segment_to_line(input, Vec2::new(1.0, 0.0), 0.0, 7);

        // Assert
        assert_eq!(output.len(), 2);
        assert_eq!(output[0], input[0]);
        assert_eq!(output[1].point, Vec2::ZERO);
        assert_eq!(output[1].feature_id.index_a(), 7);
        assert_eq!(output[1].feature_id.index_b(), 2);
        assert_eq!(output[1].feature_id.kind_a(), FeatureKind::Vertex);
        assert_eq!(output[1].feature_id.kind_b(), FeatureKind::Face);
    }

    #[test]
    fn clipping_keeps_on_plane_points_without_duplicate_crossing() {
        // Arrange
        let input = [vertex(0.0, 2), vertex(1.0, 3)];

        // Act
        let output = clip_segment_to_line(input, Vec2::new(1.0, 0.0), 0.0, 7);

        // Assert
        assert_eq!(output, vec![input[0]]);
    }

    #[test]
    fn clipping_reversed_crossing_uses_input_zero_feature() {
        // Arrange
        let input = [vertex(1.0, 9), vertex(-1.0, 3)];

        // Act
        let output = clip_segment_to_line(input, Vec2::new(1.0, 0.0), 0.0, 7);

        // Assert
        assert_eq!(output[0], input[1]);
        assert_eq!(output[1].feature_id.index_b(), 9);
    }
}
