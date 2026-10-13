//! Separating-axis test for convex quads with a clearance gap (D-04).
//!
//! Axes are unnormalized edge normals, and the gap is compared in squared
//! form, so the test needs no square root and no trigonometry; it reads only
//! the baked τ/24 vertices.

use liquidfun::math::Vec2;

/// Counter-clockwise corners of the axis-aligned rectangle `min..max`.
pub(crate) const fn rect_quad(min: Vec2, max: Vec2) -> [Vec2; 4] {
    [min, Vec2::new(max.x, min.y), max, Vec2::new(min.x, max.y)]
}

/// `true` iff some edge normal of `a` or `b` separates the convex quads by
/// strictly more than `gap` meters.
///
/// Touching, or any clearance at or below `gap`, is overlap. A non-finite
/// vertex or gap never separates, so bad input fails closed.
pub(crate) fn separated_by_more_than(a: &[Vec2; 4], b: &[Vec2; 4], gap: f32) -> bool {
    let all_finite = a
        .iter()
        .chain(b)
        .all(|vertex| vertex.x.is_finite() && vertex.y.is_finite());
    if !all_finite || !gap.is_finite() {
        return false;
    }
    edge_normals(a)
        .into_iter()
        .chain(edge_normals(b))
        .any(|normal| separated_on_axis(a, b, normal, gap))
}

fn edge_normals(quad: &[Vec2; 4]) -> [Vec2; 4] {
    let [v0, v1, v2, v3] = *quad;
    [(v0, v1), (v1, v2), (v2, v3), (v3, v0)].map(|(start, end)| {
        let edge = end - start;
        Vec2::new(edge.y, -edge.x)
    })
}

/// Compares `separation / |normal| > gap` as `separation² > gap² · |normal|²`.
fn separated_on_axis(a: &[Vec2; 4], b: &[Vec2; 4], normal: Vec2, gap: f32) -> bool {
    let (a_min, a_max) = project(a, normal);
    let (b_min, b_max) = project(b, normal);
    let separation = (b_min - a_max).max(a_min - b_max);
    separation > 0.0 && separation * separation > gap * gap * normal.length_squared()
}

fn project(quad: &[Vec2; 4], axis: Vec2) -> (f32, f32) {
    quad.iter()
        .map(|vertex| vertex.dot(axis))
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), value| {
            (min.min(value), max.max(value))
        })
}

#[cfg(test)]
mod tests {
    use liquidfun::math::Vec2;

    use super::{rect_quad, separated_by_more_than};
    use crate::puzzle::tools::pose_placement;

    fn square(min_x: f32) -> [Vec2; 4] {
        rect_quad(Vec2::new(min_x, 0.0), Vec2::new(min_x + 1.0, 1.0))
    }

    fn plank(pose: [i32; 4]) -> [Vec2; 4] {
        pose_placement(1, &pose)
            .expect("test pose describes a plank")
            .vertices()
    }

    #[test]
    fn clearance_equal_to_the_gap_is_overlap() {
        // Arrange
        let (a, b) = (square(0.0), square(1.125));

        // Act
        let separated = separated_by_more_than(&a, &b, 0.125);

        // Assert
        assert!(!separated);
    }

    #[test]
    fn clearance_beyond_the_gap_separates() {
        // Arrange
        let (a, b) = (square(0.0), square(1.25));

        // Act
        let separated = separated_by_more_than(&a, &b, 0.125);

        // Assert
        assert!(separated);
    }

    #[test]
    fn a_nan_vertex_never_separates() {
        // Arrange
        let mut far = square(100.0);
        far[2] = Vec2::new(f32::NAN, 1.0);

        // Act
        let separated = separated_by_more_than(&square(0.0), &far, 0.125);

        // Assert
        assert!(!separated);
    }

    #[test]
    fn a_nan_gap_never_separates() {
        // Arrange
        let (a, b) = (square(0.0), square(100.0));

        // Act
        let separated = separated_by_more_than(&a, &b, f32::NAN);

        // Assert
        assert!(!separated);
    }

    #[test]
    fn crossed_tau_24_planks_overlap() {
        // Arrange
        let (reference, flat) = (plank([8, 64, 23, 0]), plank([8, 64, 0, 0]));

        // Act
        let separated = separated_by_more_than(&reference, &flat, 0.08);

        // Assert
        assert!(!separated);
    }

    #[test]
    fn distant_tau_24_planks_separate() {
        // Arrange
        let (reference, upper) = (plank([8, 64, 23, 0]), plank([30, 84, 0, 0]));

        // Act
        let separated = separated_by_more_than(&reference, &upper, 0.08);

        // Assert
        assert!(separated);
    }
}
