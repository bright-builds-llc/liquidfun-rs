use super::Vec2;

/// Returns whether `value` is finite according to upstream `b2IsValid`.
#[must_use]
pub fn is_valid(value: f32) -> bool {
    value.to_bits() & 0x7f80_0000 != 0x7f80_0000
}

/// Returns the upstream-ordered `b2Abs` result.
///
/// This intentionally does not use [`f32::abs`]. The pinned comparison branch
/// makes signed-zero and NaN behavior observable.
#[must_use]
pub fn abs(value: f32) -> f32 {
    if value > 0.0 { value } else { -value }
}

/// Returns the upstream-ordered `b2Min` result.
///
/// The second operand wins when the comparison is false, including equal
/// values, signed zeros, or an unordered comparison.
#[must_use]
pub fn min(a: f32, b: f32) -> f32 {
    if a < b { a } else { b }
}

/// Returns the upstream-ordered `b2Max` result.
///
/// The second operand wins when the comparison is false, including equal
/// values, signed zeros, or an unordered comparison.
#[must_use]
pub fn max(a: f32, b: f32) -> f32 {
    if a > b { a } else { b }
}

/// Restricts `value` to the inclusive upstream-ordered range `low..=high`.
///
/// This preserves the nested `b2Max(low, b2Min(value, high))` evaluation.
#[must_use]
pub fn clamp(value: f32, low: f32, high: f32) -> f32 {
    max(low, min(value, high))
}

/// Returns the next larger power of two using upstream `b2NextPowerOfTwo`.
///
/// An input that already is a power of two advances to the next power. Values
/// at or above `0x8000_0000` wrap to zero, matching unsigned 32-bit arithmetic.
#[must_use]
pub fn next_power_of_two(mut value: u32) -> u32 {
    value |= value >> 1;
    value |= value >> 2;
    value |= value >> 4;
    value |= value >> 8;
    value |= value >> 16;
    value.wrapping_add(1)
}

/// Returns whether `value` contains exactly one set bit.
#[must_use]
#[allow(clippy::manual_is_power_of_two)] // Preserve the selected b2IsPowerOfTwo branch.
pub fn is_power_of_two(value: u32) -> bool {
    value > 0 && value & (value - 1) == 0
}

/// Returns the Euclidean distance between two 2D positions in meters.
#[must_use]
pub fn distance(a: Vec2, b: Vec2) -> f32 {
    let delta = a - b;
    delta.length()
}

/// Returns the squared Euclidean distance between two 2D positions.
#[must_use]
pub fn distance_squared(a: Vec2, b: Vec2) -> f32 {
    let delta = a - b;
    delta.dot(delta)
}

/// Returns the selected upstream's approximate inverse square root.
///
/// This is the safe `to_bits`/`from_bits` equivalent of `b2InvSqrt`, including
/// its magic constant and single Newton refinement in source order.
#[must_use]
pub fn inverse_sqrt(value: f32) -> f32 {
    let half = 0.5 * value;
    let signed_bits = value.to_bits().cast_signed();
    let approximation_bits = 0x5f37_59df_i32
        .wrapping_sub(signed_bits >> 1)
        .cast_unsigned();
    let approximation = f32::from_bits(approximation_bits);
    approximation * (1.5 - half * approximation * approximation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validity_accepts_subnormal_values() {
        // Arrange
        let positive = f32::from_bits(1);
        let negative = f32::from_bits(0x8000_0001);

        // Act
        let positive_is_valid = is_valid(positive);
        let negative_is_valid = is_valid(negative);

        // Assert
        assert!(positive_is_valid);
        assert!(negative_is_valid);
    }

    #[test]
    fn validity_rejects_non_finite_values() {
        // Arrange
        let values = [f32::INFINITY, f32::NEG_INFINITY, f32::NAN];

        // Act
        let results = values.map(is_valid);

        // Assert
        assert_eq!(results, [false, false, false]);
    }

    #[test]
    fn abs_preserves_upstream_signed_zero_directions() {
        // Arrange
        let positive_zero = 0.0_f32;
        let negative_zero = -0.0_f32;

        // Act
        let positive_result = abs(positive_zero);
        let negative_result = abs(negative_zero);

        // Assert
        assert_eq!(positive_result.to_bits(), negative_zero.to_bits());
        assert_eq!(negative_result.to_bits(), positive_zero.to_bits());
    }

    #[test]
    fn min_uses_second_signed_zero_operand() {
        // Arrange
        let positive_zero = 0.0_f32;
        let negative_zero = -0.0_f32;

        // Act
        let positive_then_negative = min(positive_zero, negative_zero);
        let negative_then_positive = min(negative_zero, positive_zero);

        // Assert
        assert_eq!(positive_then_negative.to_bits(), negative_zero.to_bits());
        assert_eq!(negative_then_positive.to_bits(), positive_zero.to_bits());
    }

    #[test]
    fn max_uses_second_signed_zero_operand() {
        // Arrange
        let positive_zero = 0.0_f32;
        let negative_zero = -0.0_f32;

        // Act
        let positive_then_negative = max(positive_zero, negative_zero);
        let negative_then_positive = max(negative_zero, positive_zero);

        // Assert
        assert_eq!(positive_then_negative.to_bits(), negative_zero.to_bits());
        assert_eq!(negative_then_positive.to_bits(), positive_zero.to_bits());
    }

    #[test]
    fn min_preserves_upstream_nan_operand_order() {
        // Arrange
        let first_nan = f32::from_bits(0x7fc0_0001);
        let second_nan = f32::from_bits(0x7fc0_0010);
        let number = 2.0_f32;

        // Act
        let nan_then_number = min(first_nan, number);
        let number_then_nan = min(number, second_nan);
        let nan_then_nan = min(first_nan, second_nan);

        // Assert
        assert_eq!(nan_then_number.to_bits(), number.to_bits());
        assert_eq!(number_then_nan.to_bits(), second_nan.to_bits());
        assert_eq!(nan_then_nan.to_bits(), second_nan.to_bits());
    }

    #[test]
    fn max_preserves_upstream_nan_operand_order() {
        // Arrange
        let first_nan = f32::from_bits(0x7fc0_0001);
        let second_nan = f32::from_bits(0x7fc0_0010);
        let number = 2.0_f32;

        // Act
        let nan_then_number = max(first_nan, number);
        let number_then_nan = max(number, second_nan);
        let nan_then_nan = max(first_nan, second_nan);

        // Assert
        assert_eq!(nan_then_number.to_bits(), number.to_bits());
        assert_eq!(number_then_nan.to_bits(), second_nan.to_bits());
        assert_eq!(nan_then_nan.to_bits(), second_nan.to_bits());
    }

    #[test]
    fn clamp_preserves_nested_upstream_order() {
        // Arrange
        let high_nan = f32::from_bits(0x7fc0_0020);

        // Act
        let ordinary = clamp(3.0, 1.0, 2.0);
        let unordered_high = clamp(3.0, 1.0, high_nan);

        // Assert
        assert_eq!(ordinary.to_bits(), 2.0_f32.to_bits());
        assert_eq!(unordered_high.to_bits(), high_nan.to_bits());
    }

    #[test]
    fn power_of_two_helpers_match_upstream_boundaries() {
        // Arrange
        let values = [0_u32, 1, 3, 8, 0x8000_0000];

        // Act
        let next = values.map(next_power_of_two);
        let exact = values.map(is_power_of_two);

        // Assert
        assert_eq!(next, [1, 2, 4, 16, 0]);
        assert_eq!(exact, [false, true, false, true, true]);
    }

    #[test]
    fn distance_helpers_preserve_vector_operation_order() {
        // Arrange
        let a = Vec2::new(4.0, 6.0);
        let b = Vec2::new(1.0, 2.0);

        // Act
        let length = distance(a, b);
        let squared = distance_squared(a, b);

        // Assert
        assert_eq!(length.to_bits(), 5.0_f32.to_bits());
        assert_eq!(squared.to_bits(), 25.0_f32.to_bits());
    }

    #[test]
    fn inverse_sqrt_matches_pinned_witness_bits() {
        // Arrange
        let values = [1.0_f32, 4.0_f32, 9.0_f32];

        // Act
        let result_bits = values.map(|value| inverse_sqrt(value).to_bits());

        // Assert
        assert_eq!(result_bits, [0x3f7f_910f, 0x3eff_910f, 0x3eaa_78d8]);
    }
}
