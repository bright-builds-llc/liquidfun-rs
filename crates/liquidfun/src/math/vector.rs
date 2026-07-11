use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use super::scalar::is_valid;

/// An initialized 2D column vector corresponding to upstream `b2Vec2`.
///
/// Coordinates normally represent meters, meters per second, or another MKS
/// quantity selected by the consuming API. Raw coordinates may contain any
/// IEEE-754 `f32` value; [`Self::is_valid`] classifies finite vectors.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec2 {
    /// The first coordinate.
    pub x: f32,
    /// The second coordinate.
    pub y: f32,
}

impl Vec2 {
    /// The initialized zero vector.
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// Creates a vector from two raw coordinates.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Returns the source-ordered dot product `self · other`.
    #[must_use]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Returns the scalar 2D cross product `self × other`.
    #[must_use]
    pub fn cross(self, other: Self) -> f32 {
        self.x * other.y - self.y * other.x
    }

    /// Returns `self × scalar` using the selected upstream operand order.
    #[must_use]
    pub fn cross_scalar(self, scalar: f32) -> Self {
        Self::new(scalar * self.y, -scalar * self.x)
    }

    /// Returns `scalar × self` using the selected upstream operand order.
    #[must_use]
    pub fn scalar_cross(scalar: f32, vector: Self) -> Self {
        Self::new(-scalar * vector.y, scalar * vector.x)
    }

    /// Returns the skew vector whose dot product reproduces the 2D cross product.
    #[must_use]
    pub fn skew(self) -> Self {
        Self::new(-self.y, self.x)
    }

    /// Returns the Euclidean length in the vector's MKS unit.
    #[must_use]
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Returns the source-ordered squared Euclidean length.
    #[must_use]
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Normalizes this vector and returns its original length.
    ///
    /// If the length is below [`f32::EPSILON`], this returns `0.0` and leaves
    /// the vector unchanged, matching upstream `b2Vec2::Normalize`.
    pub fn normalize(&mut self) -> f32 {
        let length = self.length();
        if length < f32::EPSILON {
            return 0.0;
        }

        let inverse_length = 1.0 / length;
        self.x *= inverse_length;
        self.y *= inverse_length;
        length
    }

    /// Returns whether both coordinates are finite.
    #[must_use]
    pub fn is_valid(self) -> bool {
        is_valid(self.x) && is_valid(self.y)
    }
}

/// An initialized 3D column vector corresponding to upstream `b2Vec3`.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec3 {
    /// The first coordinate.
    pub x: f32,
    /// The second coordinate.
    pub y: f32,
    /// The third coordinate.
    pub z: f32,
}

impl Vec3 {
    /// The initialized zero vector.
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    /// Creates a vector from three raw coordinates.
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Returns the source-ordered dot product `self · other`.
    #[must_use]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Returns the source-ordered 3D cross product `self × other`.
    #[must_use]
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    /// Returns the Euclidean length in the vector's MKS unit.
    #[must_use]
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Returns the source-ordered squared Euclidean length.
    #[must_use]
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Normalizes this vector and returns its original length.
    ///
    /// If the length is below [`f32::EPSILON`], this returns `0.0` and leaves
    /// the vector unchanged, matching upstream `b2Vec3::Normalize`.
    pub fn normalize(&mut self) -> f32 {
        let length = self.length();
        if length < f32::EPSILON {
            return 0.0;
        }

        let inverse_length = 1.0 / length;
        self.x *= inverse_length;
        self.y *= inverse_length;
        self.z *= inverse_length;
        length
    }

    /// Returns whether all coordinates are finite.
    #[must_use]
    pub fn is_valid(self) -> bool {
        is_valid(self.x) && is_valid(self.y) && is_valid(self.z)
    }
}

/// An initialized 4D column vector corresponding to upstream `b2Vec4`.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec4 {
    /// The first coordinate.
    pub x: f32,
    /// The second coordinate.
    pub y: f32,
    /// The third coordinate.
    pub z: f32,
    /// The fourth coordinate.
    pub w: f32,
}

impl Vec4 {
    /// The initialized zero vector.
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// Creates a vector from four raw coordinates.
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Returns whether all coordinates are finite.
    #[must_use]
    pub fn is_valid(self) -> bool {
        is_valid(self.x) && is_valid(self.y) && is_valid(self.z) && is_valid(self.w)
    }
}

macro_rules! impl_vector_operators {
    ($vector:ty, $($field:ident),+ $(,)?) => {
        impl Add for $vector {
            type Output = Self;

            fn add(self, other: Self) -> Self::Output {
                Self::new($(self.$field + other.$field),+)
            }
        }

        impl AddAssign for $vector {
            fn add_assign(&mut self, other: Self) {
                $(self.$field += other.$field;)+
            }
        }

        impl Sub for $vector {
            type Output = Self;

            fn sub(self, other: Self) -> Self::Output {
                Self::new($(self.$field - other.$field),+)
            }
        }

        impl SubAssign for $vector {
            fn sub_assign(&mut self, other: Self) {
                $(self.$field -= other.$field;)+
            }
        }

        impl Neg for $vector {
            type Output = Self;

            fn neg(self) -> Self::Output {
                Self::new($(-self.$field),+)
            }
        }

        impl Mul<f32> for $vector {
            type Output = Self;

            fn mul(self, scalar: f32) -> Self::Output {
                Self::new($(self.$field * scalar),+)
            }
        }

        impl Mul<$vector> for f32 {
            type Output = $vector;

            fn mul(self, vector: $vector) -> Self::Output {
                <$vector>::new($(self * vector.$field),+)
            }
        }

        impl MulAssign<f32> for $vector {
            fn mul_assign(&mut self, scalar: f32) {
                $(self.$field *= scalar;)+
            }
        }

        impl Div<f32> for $vector {
            type Output = Self;

            fn div(self, scalar: f32) -> Self::Output {
                Self::new($(self.$field / scalar),+)
            }
        }

        impl DivAssign<f32> for $vector {
            fn div_assign(&mut self, scalar: f32) {
                $(self.$field /= scalar;)+
            }
        }
    };
}

impl_vector_operators!(Vec2, x, y);
impl_vector_operators!(Vec3, x, y, z);
impl_vector_operators!(Vec4, x, y, z, w);

impl Add<f32> for Vec2 {
    type Output = Self;

    fn add(self, scalar: f32) -> Self::Output {
        Self::new(self.x + scalar, self.y + scalar)
    }
}

impl Sub<f32> for Vec2 {
    type Output = Self;

    fn sub(self, scalar: f32) -> Self::Output {
        Self::new(self.x - scalar, self.y - scalar)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec2_ordinary_arithmetic_is_component_wise() {
        // Arrange
        let a = Vec2::new(1.0, -2.0);
        let b = Vec2::new(3.0, 4.0);

        // Act
        let sum = a + b;
        let difference = a - b;
        let scaled = 2.0 * a;
        let divided = b / 2.0;

        // Assert
        assert_eq!(sum, Vec2::new(4.0, 2.0));
        assert_eq!(difference, Vec2::new(-2.0, -6.0));
        assert_eq!(scaled, Vec2::new(2.0, -4.0));
        assert_eq!(divided, Vec2::new(1.5, 2.0));
    }

    #[test]
    fn vec2_dot_cross_and_skew_preserve_source_order() {
        // Arrange
        let a = Vec2::new(2.0, 3.0);
        let b = Vec2::new(5.0, 7.0);

        // Act
        let dot = a.dot(b);
        let cross = a.cross(b);
        let vector_cross_scalar = a.cross_scalar(4.0);
        let scalar_cross_vector = Vec2::scalar_cross(4.0, a);
        let skew = a.skew();

        // Assert
        assert_eq!(dot.to_bits(), 31.0_f32.to_bits());
        assert_eq!(cross.to_bits(), (-1.0_f32).to_bits());
        assert_eq!(vector_cross_scalar, Vec2::new(12.0, -8.0));
        assert_eq!(scalar_cross_vector, Vec2::new(-12.0, 8.0));
        assert_eq!(skew.dot(b).to_bits(), cross.to_bits());
    }

    #[test]
    fn vec3_dot_and_cross_preserve_source_order() {
        // Arrange
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);

        // Act
        let dot = a.dot(b);
        let cross = a.cross(b);

        // Assert
        assert_eq!(dot.to_bits(), 32.0_f32.to_bits());
        assert_eq!(cross, Vec3::new(-3.0, 6.0, -3.0));
    }

    #[test]
    fn vec2_normalize_leaves_below_epsilon_value_unchanged() {
        // Arrange
        let original = Vec2::new(f32::EPSILON / 2.0, 0.0);
        let mut vector = original;

        // Act
        let length = vector.normalize();

        // Assert
        assert_eq!(length.to_bits(), 0.0_f32.to_bits());
        assert_eq!(vector, original);
    }

    #[test]
    fn vec2_normalize_accepts_epsilon_boundary() {
        // Arrange
        let mut vector = Vec2::new(f32::EPSILON, 0.0);

        // Act
        let length = vector.normalize();

        // Assert
        assert_eq!(length.to_bits(), f32::EPSILON.to_bits());
        assert_eq!(vector, Vec2::new(1.0, 0.0));
    }

    #[test]
    fn vec3_normalize_returns_original_length() {
        // Arrange
        let mut vector = Vec3::new(2.0, 3.0, 6.0);

        // Act
        let length = vector.normalize();

        // Assert
        assert_eq!(length.to_bits(), 7.0_f32.to_bits());
        let inverse_length = 1.0_f32 / 7.0;
        assert_eq!(
            vector,
            Vec3::new(
                2.0 * inverse_length,
                3.0 * inverse_length,
                6.0 * inverse_length,
            )
        );
    }

    #[test]
    fn vector_validity_accepts_subnormals_and_rejects_non_finite_coordinates() {
        // Arrange
        let subnormal = f32::from_bits(1);
        let finite = Vec4::new(subnormal, -subnormal, 0.0, -0.0);
        let infinite = Vec4::new(0.0, f32::INFINITY, 0.0, 0.0);
        let nan = Vec4::new(0.0, 0.0, f32::NAN, 0.0);

        // Act
        let results = [finite.is_valid(), infinite.is_valid(), nan.is_valid()];

        // Assert
        assert_eq!(results, [true, false, false]);
    }

    #[test]
    fn initialized_defaults_are_zero_vectors() {
        // Arrange / Act
        let vectors = (Vec2::default(), Vec3::default(), Vec4::default());

        // Assert
        assert_eq!(vectors, (Vec2::ZERO, Vec3::ZERO, Vec4::ZERO));
    }
}
