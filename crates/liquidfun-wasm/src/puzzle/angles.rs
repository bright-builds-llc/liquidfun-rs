//! Literal (cos, sin) for each τ/24 step, so static tools never call sin or cos (D-03).

use std::f32::consts::FRAC_1_SQRT_2;

/// f32-rounded `(cos, sin)` of `k · τ/24` for `k = 0..24`.
///
/// The values are literals so native and wasm builds read identical bits.
pub(crate) const TAU_24: [(f32, f32); 24] = [
    (1.0, 0.0),
    (0.965_925_8, 0.258_819_04),
    (0.866_025_4, 0.5),
    (FRAC_1_SQRT_2, FRAC_1_SQRT_2),
    (0.5, 0.866_025_4),
    (0.258_819_04, 0.965_925_8),
    (0.0, 1.0),
    (-0.258_819_04, 0.965_925_8),
    (-0.5, 0.866_025_4),
    (-FRAC_1_SQRT_2, FRAC_1_SQRT_2),
    (-0.866_025_4, 0.5),
    (-0.965_925_8, 0.258_819_04),
    (-1.0, 0.0),
    (-0.965_925_8, -0.258_819_04),
    (-0.866_025_4, -0.5),
    (-FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
    (-0.5, -0.866_025_4),
    (-0.258_819_04, -0.965_925_8),
    (0.0, -1.0),
    (0.258_819_04, -0.965_925_8),
    (0.5, -0.866_025_4),
    (FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
    (0.866_025_4, -0.5),
    (0.965_925_8, -0.258_819_04),
];

#[cfg(test)]
mod tests {
    use std::f64::consts::TAU;

    use super::TAU_24;

    #[test]
    fn tau_24_entries_match_cos_and_sin_of_each_step() {
        // Arrange
        let entries = TAU_24.iter().zip(0_u8..);

        for (&(cos, sin), step) in entries {
            // Act
            let angle = TAU * f64::from(step) / 24.0;

            // Assert
            assert!((f64::from(cos) - angle.cos()).abs() <= 1e-7, "cos {step}");
            assert!((f64::from(sin) - angle.sin()).abs() <= 1e-7, "sin {step}");
        }
    }

    #[test]
    fn tau_24_entries_have_unit_length() {
        // Arrange
        let entries = TAU_24;

        for (cos, sin) in entries {
            // Act
            let length_squared = f64::from(cos).powi(2) + f64::from(sin).powi(2);

            // Assert
            assert!((length_squared - 1.0).abs() <= 1e-6);
        }
    }

    #[test]
    fn tau_24_quarter_turns_are_exact() {
        // Arrange
        let quarter_turns = [0, 6, 12, 18];

        // Act
        let entries = quarter_turns.map(|step| TAU_24[step]);

        // Assert
        assert_eq!(entries, [(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)]);
    }
}
