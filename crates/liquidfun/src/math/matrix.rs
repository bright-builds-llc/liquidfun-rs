use super::{Vec2, Vec3, abs};

/// An initialized 2-by-2 matrix corresponding to upstream `b2Mat22`.
///
/// The matrix is mathematically column-major. Its representation is private
/// and is not an FFI or raw-layout contract.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mat22 {
    ex: Vec2,
    ey: Vec2,
}

impl Mat22 {
    /// The all-zero matrix.
    pub const ZERO: Self = Self::from_columns(Vec2::ZERO, Vec2::ZERO);

    /// The identity matrix.
    pub const IDENTITY: Self = Self::from_columns(Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0));

    /// Creates a matrix from its first and second columns.
    #[must_use]
    pub const fn from_columns(ex: Vec2, ey: Vec2) -> Self {
        Self { ex, ey }
    }

    /// Returns the first column.
    #[must_use]
    pub const fn first_column(self) -> Vec2 {
        self.ex
    }

    /// Returns the second column.
    #[must_use]
    pub const fn second_column(self) -> Vec2 {
        self.ey
    }

    /// Multiplies this matrix by a column vector.
    #[must_use]
    pub fn apply(self, vector: Vec2) -> Vec2 {
        Vec2::new(
            self.ex.x * vector.x + self.ey.x * vector.y,
            self.ex.y * vector.x + self.ey.y * vector.y,
        )
    }

    /// Multiplies this matrix's transpose by a column vector.
    #[must_use]
    pub fn inverse_apply(self, vector: Vec2) -> Vec2 {
        Vec2::new(vector.dot(self.ex), vector.dot(self.ey))
    }

    /// Returns `self * other` using the selected upstream column order.
    #[must_use]
    pub fn compose(self, other: Self) -> Self {
        Self::from_columns(self.apply(other.ex), self.apply(other.ey))
    }

    /// Returns `selfᵀ * other` using the selected upstream column order.
    #[must_use]
    pub fn transpose_compose(self, other: Self) -> Self {
        let ex = Vec2::new(self.ex.dot(other.ex), self.ey.dot(other.ex));
        let ey = Vec2::new(self.ex.dot(other.ey), self.ey.dot(other.ey));
        Self::from_columns(ex, ey)
    }

    /// Returns the component-wise upstream-ordered absolute value.
    #[must_use]
    pub fn abs(self) -> Self {
        Self::from_columns(
            Vec2::new(abs(self.ex.x), abs(self.ex.y)),
            Vec2::new(abs(self.ey.x), abs(self.ey.y)),
        )
    }

    /// Returns the inverse, or [`Self::ZERO`] when the determinant is zero.
    #[must_use]
    pub fn inverse(self) -> Self {
        let a = self.ex.x;
        let b = self.ey.x;
        let c = self.ex.y;
        let d = self.ey.y;
        let mut determinant = a * d - b * c;
        if determinant != 0.0 {
            determinant = 1.0 / determinant;
        }

        Self::from_columns(
            Vec2::new(determinant * d, -determinant * c),
            Vec2::new(-determinant * b, determinant * a),
        )
    }

    /// Solves `self * x = right`, returning zero when the determinant is zero.
    #[must_use]
    pub fn solve(self, right: Vec2) -> Vec2 {
        let a11 = self.ex.x;
        let a12 = self.ey.x;
        let a21 = self.ex.y;
        let a22 = self.ey.y;
        let mut determinant = a11 * a22 - a12 * a21;
        if determinant != 0.0 {
            determinant = 1.0 / determinant;
        }

        Vec2::new(
            determinant * (a22 * right.x - a12 * right.y),
            determinant * (a11 * right.y - a21 * right.x),
        )
    }
}

/// An initialized 3-by-3 matrix corresponding to upstream `b2Mat33`.
///
/// The matrix is mathematically column-major. Its representation is private
/// and is not an FFI or raw-layout contract.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mat33 {
    ex: Vec3,
    ey: Vec3,
    ez: Vec3,
}

impl Mat33 {
    /// The all-zero matrix.
    pub const ZERO: Self = Self::from_columns(Vec3::ZERO, Vec3::ZERO, Vec3::ZERO);

    /// The identity matrix.
    pub const IDENTITY: Self = Self::from_columns(
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
    );

    /// Creates a matrix from its first, second, and third columns.
    #[must_use]
    pub const fn from_columns(ex: Vec3, ey: Vec3, ez: Vec3) -> Self {
        Self { ex, ey, ez }
    }

    /// Returns the first column.
    #[must_use]
    pub const fn first_column(self) -> Vec3 {
        self.ex
    }

    /// Returns the second column.
    #[must_use]
    pub const fn second_column(self) -> Vec3 {
        self.ey
    }

    /// Returns the third column.
    #[must_use]
    pub const fn third_column(self) -> Vec3 {
        self.ez
    }

    /// Multiplies this matrix by a 3D column vector.
    #[must_use]
    pub fn apply(self, vector: Vec3) -> Vec3 {
        vector.x * self.ex + vector.y * self.ey + vector.z * self.ez
    }

    /// Multiplies this matrix's upper 2-by-2 block by a column vector.
    #[must_use]
    pub fn apply22(self, vector: Vec2) -> Vec2 {
        Vec2::new(
            self.ex.x * vector.x + self.ey.x * vector.y,
            self.ex.y * vector.x + self.ey.y * vector.y,
        )
    }

    /// Solves the full `self * x = right` equation.
    ///
    /// A singular determinant returns [`Vec3::ZERO`].
    #[must_use]
    pub fn solve33(self, right: Vec3) -> Vec3 {
        let mut determinant = self.ex.dot(self.ey.cross(self.ez));
        if determinant != 0.0 {
            determinant = 1.0 / determinant;
        }

        Vec3::new(
            determinant * right.dot(self.ey.cross(self.ez)),
            determinant * self.ex.dot(right.cross(self.ez)),
            determinant * self.ex.dot(self.ey.cross(right)),
        )
    }

    /// Solves the upper 2-by-2 block of `self * x = right`.
    ///
    /// A singular determinant returns [`Vec2::ZERO`].
    #[must_use]
    pub fn solve22(self, right: Vec2) -> Vec2 {
        let a11 = self.ex.x;
        let a12 = self.ey.x;
        let a21 = self.ex.y;
        let a22 = self.ey.y;
        let mut determinant = a11 * a22 - a12 * a21;
        if determinant != 0.0 {
            determinant = 1.0 / determinant;
        }

        Vec2::new(
            determinant * (a22 * right.x - a12 * right.y),
            determinant * (a11 * right.y - a21 * right.x),
        )
    }

    /// Returns the inverse of the upper 2-by-2 block in a 3-by-3 matrix.
    ///
    /// A singular determinant returns [`Self::ZERO`].
    #[must_use]
    pub fn inverse22(self) -> Self {
        let a = self.ex.x;
        let b = self.ey.x;
        let c = self.ex.y;
        let d = self.ey.y;
        let mut determinant = a * d - b * c;
        if determinant != 0.0 {
            determinant = 1.0 / determinant;
        }

        Self::from_columns(
            Vec3::new(determinant * d, -determinant * c, 0.0),
            Vec3::new(-determinant * b, determinant * a, 0.0),
            Vec3::ZERO,
        )
    }

    /// Returns the selected upstream symmetric inverse.
    ///
    /// The caller is responsible for supplying a symmetric matrix. A singular
    /// determinant returns [`Self::ZERO`].
    #[must_use]
    #[allow(clippy::similar_names)] // Symmetric matrix entries differ only by axis pair.
    pub fn symmetric_inverse33(self) -> Self {
        let mut determinant = self.ex.dot(self.ey.cross(self.ez));
        if determinant != 0.0 {
            determinant = 1.0 / determinant;
        }

        let a11 = self.ex.x;
        let a12 = self.ey.x;
        let a13 = self.ez.x;
        let a22 = self.ey.y;
        let a23 = self.ez.y;
        let a33 = self.ez.z;

        let diagonal_x = determinant * (a22 * a33 - a23 * a23);
        let off_diagonal_xy = determinant * (a13 * a23 - a12 * a33);
        let off_diagonal_xz = determinant * (a12 * a23 - a13 * a22);
        let diagonal_y = determinant * (a11 * a33 - a13 * a13);
        let off_diagonal_yz = determinant * (a13 * a12 - a11 * a23);
        let diagonal_z = determinant * (a11 * a22 - a12 * a12);

        Self::from_columns(
            Vec3::new(diagonal_x, off_diagonal_xy, off_diagonal_xz),
            Vec3::new(off_diagonal_xy, diagonal_y, off_diagonal_yz),
            Vec3::new(off_diagonal_xz, off_diagonal_yz, diagonal_z),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_matrices_leave_vectors_unchanged() {
        // Arrange
        let vector2 = Vec2::new(2.0, -3.0);
        let vector3 = Vec3::new(2.0, -3.0, 4.0);

        // Act
        let result2 = Mat22::IDENTITY.apply(vector2);
        let result3 = Mat33::IDENTITY.apply(vector3);

        // Assert
        assert_eq!(result2, vector2);
        assert_eq!(result3, vector3);
    }

    #[test]
    fn mat22_singular_inverse_and_solve_return_zero() {
        // Arrange
        let matrix = Mat22::from_columns(Vec2::new(1.0, 2.0), Vec2::new(2.0, 4.0));

        // Act
        let inverse = matrix.inverse();
        let solution = matrix.solve(Vec2::new(3.0, 6.0));

        // Assert
        assert_eq!(inverse, Mat22::ZERO);
        assert_eq!(solution, Vec2::ZERO);
    }

    #[test]
    fn mat22_near_singular_values_take_nonzero_branch() {
        // Arrange
        let next_after_one = f32::from_bits(1.0_f32.to_bits() + 1);
        let matrix = Mat22::from_columns(Vec2::new(1.0, 1.0), Vec2::new(1.0, next_after_one));

        // Act
        let inverse = matrix.inverse();
        let solution = matrix.solve(Vec2::new(1.0, next_after_one));

        // Assert
        assert_ne!(inverse, Mat22::ZERO);
        assert_eq!(solution, Vec2::new(0.0, 1.0));
    }

    #[test]
    fn mat33_singular_operations_return_zero() {
        // Arrange
        let matrix = Mat33::from_columns(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
        );

        // Act
        let solved33 = matrix.solve33(Vec3::new(1.0, 2.0, 3.0));
        let solved22 = matrix.solve22(Vec2::new(1.0, 2.0));
        let inverse22 = matrix.inverse22();
        let symmetric_inverse = matrix.symmetric_inverse33();

        // Assert
        assert_eq!(solved33, Vec3::ZERO);
        assert_eq!(solved22, Vec2::ZERO);
        assert_eq!(inverse22, Mat33::ZERO);
        assert_eq!(symmetric_inverse, Mat33::ZERO);
    }

    #[test]
    fn symmetric_inverse_mirrors_off_diagonal_values() {
        // Arrange
        let matrix = Mat33::from_columns(
            Vec3::new(4.0, 1.0, 1.0),
            Vec3::new(1.0, 3.0, 0.0),
            Vec3::new(1.0, 0.0, 2.0),
        );

        // Act
        let inverse = matrix.symmetric_inverse33();

        // Assert
        assert_eq!(
            inverse.first_column().y.to_bits(),
            inverse.second_column().x.to_bits()
        );
        assert_eq!(
            inverse.first_column().z.to_bits(),
            inverse.third_column().x.to_bits()
        );
        assert_eq!(
            inverse.second_column().z.to_bits(),
            inverse.third_column().y.to_bits()
        );
    }
}
