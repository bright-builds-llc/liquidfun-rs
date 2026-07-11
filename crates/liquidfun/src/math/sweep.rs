use std::error::Error;
use std::fmt;

use super::settings::TAU;
use super::{Rotation, Transform, Vec2};

/// A field whose non-finite value prevented construction or advancement.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SweepField {
    /// The local center-of-mass x coordinate.
    LocalCenterX,
    /// The local center-of-mass y coordinate.
    LocalCenterY,
    /// The initial world-center x coordinate.
    InitialCenterX,
    /// The initial world-center y coordinate.
    InitialCenterY,
    /// The final world-center x coordinate.
    CenterX,
    /// The final world-center y coordinate.
    CenterY,
    /// The initial world angle in radians.
    InitialAngle,
    /// The final world angle in radians.
    Angle,
    /// A time fraction.
    Fraction,
}

impl fmt::Display for SweepField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::LocalCenterX => "local_center.x",
            Self::LocalCenterY => "local_center.y",
            Self::InitialCenterX => "initial_center.x",
            Self::InitialCenterY => "initial_center.y",
            Self::CenterX => "center.x",
            Self::CenterY => "center.y",
            Self::InitialAngle => "initial_angle",
            Self::Angle => "angle",
            Self::Fraction => "fraction",
        };
        formatter.write_str(name)
    }
}

/// A checked sweep construction or time-advance failure.
#[derive(Copy, Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum SweepError {
    /// A field was NaN or infinite.
    NonFinite {
        /// The rejected field.
        field: SweepField,
    },
    /// A time fraction was outside the inclusive `0.0..=1.0` interval.
    FractionOutOfRange {
        /// The rejected fraction.
        fraction: f32,
    },
    /// Advancement attempted to move backward in time.
    DecreasingFraction {
        /// The sweep's current initial fraction.
        current: f32,
        /// The rejected requested fraction.
        requested: f32,
    },
    /// A completed sweep cannot advance because its denominator would be zero.
    Complete,
}

impl fmt::Display for SweepError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite { field } => write!(formatter, "sweep field {field} must be finite"),
            Self::FractionOutOfRange { fraction } => write!(
                formatter,
                "sweep fraction {fraction} must be in the inclusive range 0.0..=1.0"
            ),
            Self::DecreasingFraction { current, requested } => write!(
                formatter,
                "sweep fraction cannot decrease from {current} to {requested}"
            ),
            Self::Complete => formatter.write_str("a completed sweep cannot advance"),
        }
    }
}

impl Error for SweepError {}

/// Initialized motion state corresponding to upstream `b2Sweep`.
///
/// Centers are measured in meters, angles are radians, and `initial_fraction`
/// is the fraction of the current time step represented by the initial state.
/// Construction and advancement keep all stored fields finite and the fraction
/// in `0.0..=1.0`. Unlike upstream's public struct, the representation is
/// private and invalid time transitions return [`SweepError`] without mutation.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Sweep {
    local_center: Vec2,
    c0: Vec2,
    c: Vec2,
    a0: f32,
    a: f32,
    alpha0: f32,
}

impl Sweep {
    /// Creates a checked sweep from its local center, endpoint states, and
    /// initial time fraction.
    ///
    /// # Errors
    ///
    /// Returns [`SweepError::NonFinite`] for any non-finite field and
    /// [`SweepError::FractionOutOfRange`] when `initial_fraction` is outside
    /// `0.0..=1.0`.
    #[must_use = "construction can fail when a field violates sweep invariants"]
    pub fn new(
        local_center: Vec2,
        initial_center: Vec2,
        center: Vec2,
        initial_angle: f32,
        angle: f32,
        initial_fraction: f32,
    ) -> Result<Self, SweepError> {
        validate_vector(
            local_center,
            SweepField::LocalCenterX,
            SweepField::LocalCenterY,
        )?;
        validate_vector(
            initial_center,
            SweepField::InitialCenterX,
            SweepField::InitialCenterY,
        )?;
        validate_vector(center, SweepField::CenterX, SweepField::CenterY)?;
        validate_scalar(initial_angle, SweepField::InitialAngle)?;
        validate_scalar(angle, SweepField::Angle)?;
        validate_fraction(initial_fraction)?;

        Ok(Self {
            local_center,
            c0: initial_center,
            c: center,
            a0: initial_angle,
            a: angle,
            alpha0: initial_fraction,
        })
    }

    /// Returns the local center of mass in meters.
    #[must_use]
    pub const fn local_center(self) -> Vec2 {
        self.local_center
    }

    /// Returns the initial world center in meters.
    #[must_use]
    pub const fn initial_center(self) -> Vec2 {
        self.c0
    }

    /// Returns the final world center in meters.
    #[must_use]
    pub const fn center(self) -> Vec2 {
        self.c
    }

    /// Returns the initial world angle in radians.
    #[must_use]
    pub const fn initial_angle(self) -> f32 {
        self.a0
    }

    /// Returns the final world angle in radians.
    #[must_use]
    pub const fn angle(self) -> f32 {
        self.a
    }

    /// Returns the initial time fraction in `0.0..=1.0`.
    #[must_use]
    pub const fn initial_fraction(self) -> f32 {
        self.alpha0
    }

    /// Returns the interpolated transform at `fraction` relative to the
    /// sweep's current initial state.
    ///
    /// Values in `0.0..=1.0` interpolate between the current endpoints. This
    /// pure compatibility kernel also permits raw fractions for probe and
    /// extrapolation use; it never changes the checked sweep state.
    #[must_use]
    pub fn transform_at(self, fraction: f32) -> Transform {
        let mut position = (1.0 - fraction) * self.c0 + fraction * self.c;
        let angle = (1.0 - fraction) * self.a0 + fraction * self.a;
        let rotation = Rotation::from_angle(angle);
        position -= rotation.apply(self.local_center);
        Transform::new(position, rotation)
    }

    /// Advances the initial state to a nondecreasing absolute time fraction.
    ///
    /// The exact upstream arithmetic kernel runs only after validation. An
    /// error leaves every field unchanged.
    ///
    /// # Errors
    ///
    /// Returns a typed [`SweepError`] when `fraction` is non-finite,
    /// out-of-range, decreasing, or requested after the sweep is complete.
    /// Arithmetic overflow is also rejected as a non-finite initial state.
    pub fn advance(&mut self, fraction: f32) -> Result<(), SweepError> {
        validate_fraction(fraction)?;
        if fraction < self.alpha0 {
            return Err(SweepError::DecreasingFraction {
                current: self.alpha0,
                requested: fraction,
            });
        }
        if self.alpha0 >= 1.0 {
            return Err(SweepError::Complete);
        }

        let beta = (fraction - self.alpha0) / (1.0 - self.alpha0);
        let initial_center = self.c0 + beta * (self.c - self.c0);
        let initial_angle = self.a0 + beta * (self.a - self.a0);
        validate_vector(
            initial_center,
            SweepField::InitialCenterX,
            SweepField::InitialCenterY,
        )?;
        validate_scalar(initial_angle, SweepField::InitialAngle)?;

        self.c0 = initial_center;
        self.a0 = initial_angle;
        self.alpha0 = fraction;
        Ok(())
    }

    /// Subtracts the same whole-turn offset from both endpoint angles.
    ///
    /// This preserves their angular difference and the selected upstream
    /// source order; it does not wrap each angle independently.
    pub fn normalize(&mut self) {
        let offset = TAU * (self.a0 / TAU).floor();
        self.a0 -= offset;
        self.a -= offset;
    }
}

fn validate_vector(
    vector: Vec2,
    x_field: SweepField,
    y_field: SweepField,
) -> Result<(), SweepError> {
    validate_scalar(vector.x, x_field)?;
    validate_scalar(vector.y, y_field)
}

fn validate_scalar(value: f32, field: SweepField) -> Result<(), SweepError> {
    if !value.is_finite() {
        return Err(SweepError::NonFinite { field });
    }
    Ok(())
}

fn validate_fraction(fraction: f32) -> Result<(), SweepError> {
    validate_scalar(fraction, SweepField::Fraction)?;
    if !(0.0..=1.0).contains(&fraction) {
        return Err(SweepError::FractionOutOfRange { fraction });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ordinary_sweep(initial_fraction: f32) -> Sweep {
        Sweep::new(
            Vec2::ZERO,
            Vec2::new(2.0, 4.0),
            Vec2::new(6.0, 8.0),
            0.25,
            0.75,
            initial_fraction,
        )
        .expect("ordinary finite sweep should be valid")
    }

    #[test]
    fn constructor_rejects_each_non_finite_field() {
        // Arrange
        let invalid_cases = [
            (
                Vec2::new(f32::NAN, 0.0),
                Vec2::ZERO,
                Vec2::ZERO,
                0.0,
                0.0,
                0.0,
                SweepField::LocalCenterX,
            ),
            (
                Vec2::new(0.0, f32::INFINITY),
                Vec2::ZERO,
                Vec2::ZERO,
                0.0,
                0.0,
                0.0,
                SweepField::LocalCenterY,
            ),
            (
                Vec2::ZERO,
                Vec2::new(f32::NEG_INFINITY, 0.0),
                Vec2::ZERO,
                0.0,
                0.0,
                0.0,
                SweepField::InitialCenterX,
            ),
            (
                Vec2::ZERO,
                Vec2::new(0.0, f32::NAN),
                Vec2::ZERO,
                0.0,
                0.0,
                0.0,
                SweepField::InitialCenterY,
            ),
            (
                Vec2::ZERO,
                Vec2::ZERO,
                Vec2::new(f32::INFINITY, 0.0),
                0.0,
                0.0,
                0.0,
                SweepField::CenterX,
            ),
            (
                Vec2::ZERO,
                Vec2::ZERO,
                Vec2::new(0.0, f32::NEG_INFINITY),
                0.0,
                0.0,
                0.0,
                SweepField::CenterY,
            ),
            (
                Vec2::ZERO,
                Vec2::ZERO,
                Vec2::ZERO,
                f32::NAN,
                0.0,
                0.0,
                SweepField::InitialAngle,
            ),
            (
                Vec2::ZERO,
                Vec2::ZERO,
                Vec2::ZERO,
                0.0,
                f32::INFINITY,
                0.0,
                SweepField::Angle,
            ),
            (
                Vec2::ZERO,
                Vec2::ZERO,
                Vec2::ZERO,
                0.0,
                0.0,
                f32::NAN,
                SweepField::Fraction,
            ),
        ];

        // Act / Assert
        for (local_center, initial_center, center, initial_angle, angle, fraction, field) in
            invalid_cases
        {
            let result = Sweep::new(
                local_center,
                initial_center,
                center,
                initial_angle,
                angle,
                fraction,
            );
            assert_eq!(result, Err(SweepError::NonFinite { field }));
        }
    }

    #[test]
    fn constructor_rejects_out_of_range_fraction() {
        // Arrange / Act
        let below = Sweep::new(Vec2::ZERO, Vec2::ZERO, Vec2::ZERO, 0.0, 0.0, -0.25);
        let above = Sweep::new(Vec2::ZERO, Vec2::ZERO, Vec2::ZERO, 0.0, 0.0, 1.25);

        // Assert
        assert_eq!(
            below,
            Err(SweepError::FractionOutOfRange { fraction: -0.25 })
        );
        assert_eq!(
            above,
            Err(SweepError::FractionOutOfRange { fraction: 1.25 })
        );
    }

    #[test]
    fn transform_endpoints_preserve_exact_expected_bits() {
        // Arrange
        let sweep = ordinary_sweep(0.0);

        // Act
        let initial = sweep.transform_at(0.0);
        let final_state = sweep.transform_at(1.0);

        // Assert
        assert_eq!(initial.position().x.to_bits(), 2.0_f32.to_bits());
        assert_eq!(initial.position().y.to_bits(), 4.0_f32.to_bits());
        assert_eq!(
            initial.rotation().sine().to_bits(),
            0.25_f32.sin().to_bits()
        );
        assert_eq!(
            initial.rotation().cosine().to_bits(),
            0.25_f32.cos().to_bits()
        );
        assert_eq!(final_state.position().x.to_bits(), 6.0_f32.to_bits());
        assert_eq!(final_state.position().y.to_bits(), 8.0_f32.to_bits());
        assert_eq!(
            final_state.rotation().sine().to_bits(),
            0.75_f32.sin().to_bits()
        );
        assert_eq!(
            final_state.rotation().cosine().to_bits(),
            0.75_f32.cos().to_bits()
        );
    }

    #[test]
    fn advance_updates_initial_state_with_pinned_kernel() {
        // Arrange
        let mut sweep = ordinary_sweep(0.0);

        // Act
        let result = sweep.advance(0.5);

        // Assert
        assert_eq!(result, Ok(()));
        assert_eq!(sweep.initial_center(), Vec2::new(4.0, 6.0));
        assert_eq!(sweep.initial_angle().to_bits(), 0.5_f32.to_bits());
        assert_eq!(sweep.initial_fraction().to_bits(), 0.5_f32.to_bits());
    }

    #[test]
    fn decreasing_advance_reports_typed_error_without_mutation() {
        // Arrange
        let mut sweep = ordinary_sweep(0.5);
        let original = sweep;

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
        assert_eq!(sweep, original);
    }

    #[test]
    fn out_of_range_advance_reports_typed_error_without_mutation() {
        // Arrange
        let mut sweep = ordinary_sweep(0.5);
        let original = sweep;

        // Act
        let result = sweep.advance(1.25);

        // Assert
        assert_eq!(
            result,
            Err(SweepError::FractionOutOfRange { fraction: 1.25 })
        );
        assert_eq!(sweep, original);
    }

    #[test]
    fn non_finite_advance_reports_typed_error_without_mutation() {
        // Arrange
        let mut sweep = ordinary_sweep(0.5);
        let original = sweep;

        // Act
        let result = sweep.advance(f32::NAN);

        // Assert
        assert_eq!(
            result,
            Err(SweepError::NonFinite {
                field: SweepField::Fraction,
            })
        );
        assert_eq!(sweep, original);
    }

    #[test]
    fn completed_sweep_rejects_advance_without_mutation() {
        // Arrange
        let mut sweep = ordinary_sweep(1.0);
        let original = sweep;

        // Act
        let result = sweep.advance(1.0);

        // Assert
        assert_eq!(result, Err(SweepError::Complete));
        assert_eq!(sweep, original);
    }

    #[test]
    fn overflowing_advance_rejects_non_finite_result_without_mutation() {
        // Arrange
        let mut sweep = Sweep::new(
            Vec2::ZERO,
            Vec2::new(f32::MAX, 0.0),
            Vec2::new(-f32::MAX, 0.0),
            0.0,
            0.0,
            0.0,
        )
        .expect("finite endpoints should produce a valid sweep");
        let original = sweep;

        // Act
        let result = sweep.advance(0.5);

        // Assert
        assert_eq!(
            result,
            Err(SweepError::NonFinite {
                field: SweepField::InitialCenterX,
            })
        );
        assert_eq!(sweep, original);
    }

    #[test]
    fn normalize_subtracts_one_shared_tau_multiple() {
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
        let offset = TAU * ((TAU + 0.25) / TAU).floor();
        let expected_initial = (TAU + 0.25) - offset;
        let expected_final = (TAU + TAU + 0.5) - offset;

        // Act
        sweep.normalize();

        // Assert
        assert_eq!(sweep.initial_angle().to_bits(), expected_initial.to_bits());
        assert_eq!(sweep.angle().to_bits(), expected_final.to_bits());
        assert!(sweep.angle() > TAU);
    }
}
