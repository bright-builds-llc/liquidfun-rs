//! Consumer-facing contract tests for the initialized public math surface.

use liquidfun::math::{
    Mat22, Mat33, Rotation, Sweep, SweepError, SweepField, Transform, Vec2, Vec3, settings::TAU,
};
use proptest::prelude::*;

fn ordinary_sweep(initial_fraction: f32) -> Sweep {
    Sweep::new(
        Vec2::ZERO,
        Vec2::new(1.0, 2.0),
        Vec2::new(5.0, 6.0),
        0.0,
        TAU / 4.0,
        initial_fraction,
    )
    .expect("ordinary finite sweep should be valid")
}

fn coordinate_is_close(actual: f32, expected: f32) -> bool {
    let scale = expected.abs().max(1.0);
    (actual - expected).abs() <= 32.0 * f32::EPSILON * scale
}

#[test]
fn public_math_types_are_constructible_from_curated_exports() {
    // Arrange / Act
    let matrix22 = Mat22::from_columns(Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0));
    let matrix33 = Mat33::from_columns(
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
    );
    let rotation = Rotation::from_angle(0.0);
    let transform = Transform::new(Vec2::ZERO, rotation);
    let sweep = ordinary_sweep(0.0);

    // Assert
    assert_eq!(matrix22, Mat22::IDENTITY);
    assert_eq!(matrix33, Mat33::IDENTITY);
    assert_eq!(rotation, Rotation::IDENTITY);
    assert_eq!(transform, Transform::IDENTITY);
    assert_eq!(sweep.initial_fraction().to_bits(), 0.0_f32.to_bits());
}

#[test]
fn sweep_endpoints_match_exact_consumer_state_bits() {
    // Arrange
    let sweep = ordinary_sweep(0.0);

    // Act
    let initial = sweep.transform_at(0.0);
    let final_state = sweep.transform_at(1.0);

    // Assert
    assert_eq!(initial.position().x.to_bits(), 1.0_f32.to_bits());
    assert_eq!(initial.position().y.to_bits(), 2.0_f32.to_bits());
    assert_eq!(final_state.position().x.to_bits(), 5.0_f32.to_bits());
    assert_eq!(final_state.position().y.to_bits(), 6.0_f32.to_bits());
}

#[test]
fn sweep_advance_is_monotonic_and_updates_the_initial_endpoint() {
    // Arrange
    let mut sweep = ordinary_sweep(0.0);

    // Act
    let first = sweep.advance(0.25);
    let first_center = sweep.initial_center();
    let second = sweep.advance(0.75);

    // Assert
    assert_eq!(first, Ok(()));
    assert_eq!(second, Ok(()));
    assert_eq!(first_center, Vec2::new(2.0, 3.0));
    assert_eq!(sweep.initial_center(), Vec2::new(4.0, 5.0));
    assert_eq!(sweep.initial_fraction().to_bits(), 0.75_f32.to_bits());
}

#[test]
fn rejected_sweep_transition_preserves_public_state() {
    // Arrange
    let mut sweep = ordinary_sweep(0.5);
    let before = sweep;

    // Act
    let result = sweep.advance(0.25);

    // Assert
    assert_eq!(
        result,
        Err(SweepError::DecreasingFraction {
            current: 0.5,
            requested: 0.25,
        })
    );
    assert_eq!(sweep, before);
}

#[test]
fn non_finite_public_sweep_input_reports_the_field() {
    // Arrange / Act
    let result = Sweep::new(Vec2::ZERO, Vec2::ZERO, Vec2::ZERO, 0.0, f32::NAN, 0.0);

    // Assert
    assert_eq!(
        result,
        Err(SweepError::NonFinite {
            field: SweepField::Angle,
        })
    );
}

#[test]
fn sweep_normalization_retains_endpoint_angle_difference() {
    // Arrange
    let mut sweep = Sweep::new(
        Vec2::ZERO,
        Vec2::ZERO,
        Vec2::ZERO,
        TAU + 0.25,
        TAU + TAU + 0.5,
        0.0,
    )
    .expect("finite angles should produce a valid sweep");
    let difference_before = sweep.angle() - sweep.initial_angle();

    // Act
    sweep.normalize();

    // Assert
    assert_eq!(
        (sweep.angle() - sweep.initial_angle()).to_bits(),
        difference_before.to_bits()
    );
    assert!(sweep.angle() > TAU);
}

proptest! {
    #[test]
    fn well_conditioned_matrix_round_trip_is_bounded(
        a in 1.0_f32..4.0,
        b in -0.25_f32..0.25,
        c in -0.25_f32..0.25,
        d in 1.0_f32..4.0,
        x in -10.0_f32..10.0,
        y in -10.0_f32..10.0,
    ) {
        // Arrange
        let matrix = Mat22::from_columns(Vec2::new(a, c), Vec2::new(b, d));
        let vector = Vec2::new(x, y);

        // Act
        let restored = matrix.inverse().apply(matrix.apply(vector));

        // Assert
        prop_assert!(coordinate_is_close(restored.x, vector.x));
        prop_assert!(coordinate_is_close(restored.y, vector.y));
    }

    #[test]
    fn well_conditioned_transform_round_trip_is_bounded(
        px in -10.0_f32..10.0,
        py in -10.0_f32..10.0,
        tx in -10.0_f32..10.0,
        ty in -10.0_f32..10.0,
        angle in (-TAU / 4.0)..(TAU / 4.0),
    ) {
        // Arrange
        let point = Vec2::new(px, py);
        let transform = Transform::from_position_angle(Vec2::new(tx, ty), angle);

        // Act
        let restored = transform.inverse_apply(transform.apply(point));

        // Assert
        prop_assert!(coordinate_is_close(restored.x, point.x));
        prop_assert!(coordinate_is_close(restored.y, point.y));
    }
}
