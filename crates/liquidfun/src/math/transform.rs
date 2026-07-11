use super::Vec2;

/// An initialized planar rotation corresponding to upstream `b2Rot`.
///
/// Angles are radians. Composing rotations preserves the selected upstream
/// multiplication order and does not silently renormalize the result.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rotation {
    sine: f32,
    cosine: f32,
}

impl Rotation {
    /// The identity rotation.
    pub const IDENTITY: Self = Self {
        sine: 0.0,
        cosine: 1.0,
    };

    /// Creates a rotation from an angle in radians.
    #[must_use]
    pub fn from_angle(angle: f32) -> Self {
        let sine = angle.sin();
        let cosine = angle.cos();
        Self { sine, cosine }
    }

    /// Returns the represented angle in radians.
    #[must_use]
    pub fn angle(self) -> f32 {
        self.sine.atan2(self.cosine)
    }

    /// Returns the sine component.
    #[must_use]
    pub const fn sine(self) -> f32 {
        self.sine
    }

    /// Returns the cosine component.
    #[must_use]
    pub const fn cosine(self) -> f32 {
        self.cosine
    }

    /// Returns the rotated x-axis.
    #[must_use]
    pub const fn x_axis(self) -> Vec2 {
        Vec2::new(self.cosine, self.sine)
    }

    /// Returns the rotated y-axis.
    #[must_use]
    pub const fn y_axis(self) -> Vec2 {
        Vec2::new(-self.sine, self.cosine)
    }

    /// Rotates a vector into the parent frame.
    #[must_use]
    pub fn apply(self, vector: Vec2) -> Vec2 {
        Vec2::new(
            self.cosine * vector.x - self.sine * vector.y,
            self.sine * vector.x + self.cosine * vector.y,
        )
    }

    /// Applies the transpose rotation to a vector.
    #[must_use]
    pub fn inverse_apply(self, vector: Vec2) -> Vec2 {
        Vec2::new(
            self.cosine * vector.x + self.sine * vector.y,
            -self.sine * vector.x + self.cosine * vector.y,
        )
    }

    /// Returns `self * other` in upstream composition order.
    #[must_use]
    pub fn compose(self, other: Self) -> Self {
        Self {
            sine: self.sine * other.cosine + self.cosine * other.sine,
            cosine: self.cosine * other.cosine - self.sine * other.sine,
        }
    }

    /// Returns `selfᵀ * other` in upstream composition order.
    #[must_use]
    pub fn inverse_compose(self, other: Self) -> Self {
        Self {
            sine: self.cosine * other.sine - self.sine * other.cosine,
            cosine: self.cosine * other.cosine + self.sine * other.sine,
        }
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// A translation and rotation corresponding to upstream `b2Transform`.
///
/// Applying a transform rotates a local point and then translates it into the
/// parent frame. Its representation is private and has no raw-layout promise.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Transform {
    position: Vec2,
    rotation: Rotation,
}

impl Transform {
    /// The identity transform.
    pub const IDENTITY: Self = Self::new(Vec2::ZERO, Rotation::IDENTITY);

    /// Creates a transform from a position and rotation.
    #[must_use]
    pub const fn new(position: Vec2, rotation: Rotation) -> Self {
        Self { position, rotation }
    }

    /// Creates a transform from a position and angle in radians.
    #[must_use]
    pub fn from_position_angle(position: Vec2, angle: f32) -> Self {
        Self::new(position, Rotation::from_angle(angle))
    }

    /// Returns the translation in meters.
    #[must_use]
    pub const fn position(self) -> Vec2 {
        self.position
    }

    /// Returns the rotation.
    #[must_use]
    pub const fn rotation(self) -> Rotation {
        self.rotation
    }

    /// Maps a local point into the parent frame.
    #[must_use]
    pub fn apply(self, point: Vec2) -> Vec2 {
        let x = (self.rotation.cosine * point.x - self.rotation.sine * point.y) + self.position.x;
        let y = (self.rotation.sine * point.x + self.rotation.cosine * point.y) + self.position.y;
        Vec2::new(x, y)
    }

    /// Maps a parent-frame point back into the local frame.
    #[must_use]
    pub fn inverse_apply(self, point: Vec2) -> Vec2 {
        let px = point.x - self.position.x;
        let py = point.y - self.position.y;
        let x = self.rotation.cosine * px + self.rotation.sine * py;
        let y = -self.rotation.sine * px + self.rotation.cosine * py;
        Vec2::new(x, y)
    }

    /// Returns the transform that applies `other` and then `self`.
    #[must_use]
    pub fn compose(self, other: Self) -> Self {
        Self {
            rotation: self.rotation.compose(other.rotation),
            position: self.rotation.apply(other.position) + self.position,
        }
    }

    /// Returns `self⁻¹ * other` using the pinned transpose composition.
    #[must_use]
    pub fn inverse_compose(self, other: Self) -> Self {
        Self {
            rotation: self.rotation.inverse_compose(other.rotation),
            position: self.rotation.inverse_apply(other.position - self.position),
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::settings::TAU;

    #[test]
    fn identity_rotation_and_transform_leave_vectors_unchanged() {
        // Arrange
        let point = Vec2::new(2.0, -3.0);

        // Act
        let rotated = Rotation::IDENTITY.apply(point);
        let transformed = Transform::IDENTITY.apply(point);

        // Assert
        assert_eq!(rotated, point);
        assert_eq!(transformed, point);
    }

    #[test]
    fn rotation_preserves_signed_zero_angle() {
        // Arrange
        let angle = -0.0_f32;

        // Act
        let rotation = Rotation::from_angle(angle);
        let recovered = rotation.angle();

        // Assert
        assert_eq!(rotation.sine().to_bits(), angle.to_bits());
        assert_eq!(recovered.to_bits(), angle.to_bits());
    }

    #[test]
    fn rotation_axes_follow_pinned_direction() {
        // Arrange
        let rotation = Rotation::from_angle(TAU / 4.0);

        // Act
        let x_axis = rotation.x_axis();
        let y_axis = rotation.y_axis();

        // Assert
        assert_eq!(x_axis, Vec2::new(rotation.cosine(), rotation.sine()));
        assert_eq!(y_axis, Vec2::new(-rotation.sine(), rotation.cosine()));
    }

    #[test]
    fn transform_composition_applies_right_operand_first() {
        // Arrange
        let outer = Transform::from_position_angle(Vec2::new(5.0, 0.0), TAU / 4.0);
        let inner = Transform::from_position_angle(Vec2::new(2.0, 0.0), 0.0);
        let point = Vec2::new(1.0, 0.0);

        // Act
        let sequential = outer.apply(inner.apply(point));
        let composed = outer.compose(inner).apply(point);

        // Assert
        assert_eq!(composed, sequential);
    }

    #[test]
    fn inverse_application_subtracts_position_before_rotation() {
        // Arrange
        let transform = Transform::from_position_angle(Vec2::new(2.0, -4.0), 0.0);
        let point = Vec2::new(3.0, 5.0);

        // Act
        let local = transform.inverse_apply(point);

        // Assert
        assert_eq!(local, Vec2::new(1.0, 9.0));
    }
}
