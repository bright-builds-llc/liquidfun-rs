//! Owned immutable shape values and checked unary shape operations.

mod circle;
mod edge;

pub use circle::CircleShape;
pub use edge::EdgeShape;

use crate::math::{Transform, Vec2};

use super::{Aabb, ChildIndex, CollisionError, MassData, RayCastHit, RayCastInput};

/// A finite distance and outward direction from a shape to a point.
///
/// Circle queries at the exact center use [`Vec2::ZERO`] for the normal. The
/// pinned C++ division by zero produces arithmetic NaN there; returning a
/// named initialized result is this crate's deliberate safe-Rust difference.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointDistance {
    distance: f32,
    normal: Vec2,
}

impl PointDistance {
    pub(super) fn new(distance: f32, normal: Vec2) -> Result<Self, CollisionError> {
        validate_scalar(distance)?;
        validate_vec2(normal)?;
        Ok(Self { distance, normal })
    }

    /// Returns the signed distance. Circle interior distances are negative.
    #[must_use]
    pub const fn distance(self) -> f32 {
        self.distance
    }

    /// Returns the finite direction in which distance increases.
    #[must_use]
    pub const fn normal(self) -> Vec2 {
        self.normal
    }
}

/// The closed set of owned shape values supported by the pinned engine.
///
/// Polygon and chain variants are added in the same plan after their checked
/// constructors establish the required invariants.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    /// One owned circle.
    Circle(CircleShape),
    /// One owned line segment with optional adjacency.
    Edge(EdgeShape),
}

impl Shape {
    /// Returns the collision skin radius.
    #[must_use]
    pub const fn radius(&self) -> f32 {
        match self {
            Self::Circle(shape) => shape.radius(),
            Self::Edge(shape) => shape.radius(),
        }
    }

    /// Returns the number of selectable child primitives.
    #[must_use]
    pub const fn child_count(&self) -> usize {
        1
    }

    /// Checks and returns one child coordinate for this shape.
    ///
    /// # Errors
    ///
    /// Returns [`CollisionError::ChildIndexOutOfRange`] when absent.
    pub const fn child_index(&self, requested: usize) -> Result<ChildIndex, CollisionError> {
        ChildIndex::new(requested, self.child_count())
    }

    /// Tests a finite world point for containment.
    ///
    /// # Errors
    ///
    /// Returns [`CollisionError::NonFiniteValue`] for invalid query geometry.
    pub fn test_point(&self, transform: Transform, point: Vec2) -> Result<bool, CollisionError> {
        match self {
            Self::Circle(shape) => shape.test_point(transform, point),
            Self::Edge(shape) => shape.test_point(transform, point),
        }
    }

    /// Computes distance to a finite world point for a checked child.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid query geometry or child selection.
    pub fn distance_to_point(
        &self,
        transform: Transform,
        point: Vec2,
        child_index: ChildIndex,
    ) -> Result<PointDistance, CollisionError> {
        validate_child(child_index, self.child_count())?;
        match self {
            Self::Circle(shape) => shape.distance_to_point(transform, point),
            Self::Edge(shape) => shape.distance_to_point(transform, point),
        }
    }

    /// Casts a finite clipped ray against one checked child.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid transform results or child selection.
    pub fn ray_cast(
        &self,
        input: RayCastInput,
        transform: Transform,
        child_index: ChildIndex,
    ) -> Result<Option<RayCastHit>, CollisionError> {
        validate_child(child_index, self.child_count())?;
        match self {
            Self::Circle(shape) => shape.ray_cast(input, transform),
            Self::Edge(shape) => shape.ray_cast(input, transform),
        }
    }

    /// Computes one checked child's finite world AABB.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid transform results or child selection.
    pub fn compute_aabb(
        &self,
        transform: Transform,
        child_index: ChildIndex,
    ) -> Result<Aabb, CollisionError> {
        validate_child(child_index, self.child_count())?;
        match self {
            Self::Circle(shape) => shape.compute_aabb(transform),
            Self::Edge(shape) => shape.compute_aabb(transform),
        }
    }

    /// Computes initialized mass properties at a finite non-negative density.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid density or derived mass value.
    pub fn compute_mass(&self, density: f32) -> Result<MassData, CollisionError> {
        match self {
            Self::Circle(shape) => shape.compute_mass(density),
            Self::Edge(shape) => shape.compute_mass(density),
        }
    }
}

impl From<CircleShape> for Shape {
    fn from(shape: CircleShape) -> Self {
        Self::Circle(shape)
    }
}

impl From<EdgeShape> for Shape {
    fn from(shape: EdgeShape) -> Self {
        Self::Edge(shape)
    }
}

pub(super) fn validate_scalar(value: f32) -> Result<(), CollisionError> {
    if !value.is_finite() {
        return Err(CollisionError::NonFiniteValue);
    }
    Ok(())
}

pub(super) fn validate_vec2(value: Vec2) -> Result<(), CollisionError> {
    if !value.is_valid() {
        return Err(CollisionError::NonFiniteValue);
    }
    Ok(())
}

pub(super) fn validate_transform(transform: Transform) -> Result<(), CollisionError> {
    validate_vec2(transform.position())?;
    let rotation = transform.rotation();
    validate_scalar(rotation.sine())?;
    validate_scalar(rotation.cosine())
}

pub(super) fn validate_query(transform: Transform, point: Vec2) -> Result<(), CollisionError> {
    validate_transform(transform)?;
    validate_vec2(point)
}

pub(super) fn validate_density(density: f32) -> Result<(), CollisionError> {
    validate_scalar(density)?;
    if density < 0.0 {
        return Err(CollisionError::InvalidGeometry);
    }
    Ok(())
}

fn validate_child(child_index: ChildIndex, child_count: usize) -> Result<(), CollisionError> {
    ChildIndex::new(child_index.get(), child_count).map(|_| ())
}
