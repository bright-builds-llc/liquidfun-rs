use serde::{Deserialize, Serialize};

/// An authoritative `f32` value represented by its exact IEEE-754 bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FloatBits(u32);

impl FloatBits {
    /// Creates a value from an authoritative bit pattern.
    #[must_use]
    pub const fn new(bits: u32) -> Self {
        Self(bits)
    }

    /// Captures the exact bits of a diagnostic floating-point value.
    #[must_use]
    pub fn from_f32(value: f32) -> Self {
        Self(value.to_bits())
    }

    /// Returns the authoritative bit pattern.
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Reconstructs an `f32` for diagnostics, never for wire authority.
    #[must_use]
    pub fn to_f32(self) -> f32 {
        f32::from_bits(self.0)
    }
}

impl From<u32> for FloatBits {
    fn from(bits: u32) -> Self {
        Self::new(bits)
    }
}

impl From<FloatBits> for u32 {
    fn from(value: FloatBits) -> Self {
        value.bits()
    }
}

#[cfg(test)]
mod tests {
    use super::FloatBits;

    #[test]
    fn float_bits_round_trip_authoritative_bit_patterns() {
        // Arrange
        let bit_patterns = [
            0x0000_0000,
            0x8000_0000,
            1.5_f32.to_bits(),
            f32::INFINITY.to_bits(),
            f32::NEG_INFINITY.to_bits(),
            0x7fc0_0042,
        ];

        // Act
        let round_tripped =
            bit_patterns.map(|bits| FloatBits::from_f32(FloatBits::new(bits).to_f32()));

        // Assert
        assert_eq!(round_tripped.map(FloatBits::bits), bit_patterns);
    }
}
