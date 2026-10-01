//! The deck fills the shaft, and the divider comes down to that deck.

#[test]
fn plate_spans_the_shaft_under_a_lower_divider() {
    // Arrange
    let corners = super::super::plate_local_corners();
    let left = corners
        .iter()
        .fold(f32::MAX, |bound, corner| bound.min(corner.x));
    let right = corners
        .iter()
        .fold(f32::MIN, |bound, corner| bound.max(corner.x));
    let deck_top = corners
        .iter()
        .fold(f32::MIN, |bound, corner| bound.max(corner.y));

    // Act
    let left_gap = left + super::super::PLATE_CENTER.x - (0.52 + super::super::WALL_HALF);
    let right_gap = (1.00 - super::super::WALL_HALF) - (right + super::super::PLATE_CENTER.x);

    // Assert
    assert!(
        left_gap > 0.0 && left_gap < super::super::PARTICLE_RADIUS * 2.0,
        "{left_gap}"
    );
    assert!(
        right_gap > 0.0 && right_gap < super::super::PARTICLE_RADIUS * 2.0,
        "{right_gap}"
    );
    assert!(super::super::DIVIDER_BOTTOM_Y > deck_top + super::super::PLATE_CENTER.y);
    assert!(super::super::DIVIDER_BOTTOM_Y < 0.22);
}
