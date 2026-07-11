use crate::math::settings::{EPSILON, PI};
use crate::math::{Transform, Vec2};

use super::{
    PointDistance, validate_density, validate_query, validate_scalar, validate_transform,
    validate_vec2,
};
use crate::collision::{Aabb, CollisionError, MassData, RayCastHit, RayCastInput};

/// An immutable owned circle corresponding to pinned `b2CircleShape` geometry.
#[derive(Debug, Clone, PartialEq)]
pub struct CircleShape {
    center: Vec2,
    radius: f32,
}

impl CircleShape {
    /// Creates a finite circle with a non-negative radius.
    ///
    /// # Errors
    ///
    /// Returns a typed error for non-finite geometry or a negative radius.
    pub fn new(center: Vec2, radius: f32) -> Result<Self, CollisionError> {
        validate_vec2(center)?;
        validate_scalar(radius)?;
        if radius < 0.0 {
            return Err(CollisionError::InvalidGeometry);
        }
        Ok(Self { center, radius })
    }

    /// Returns the local center.
    #[must_use]
    pub const fn center(&self) -> Vec2 {
        self.center
    }

    /// Returns the circle radius.
    #[must_use]
    pub const fn radius(&self) -> f32 {
        self.radius
    }

    /// Returns the single child count.
    #[must_use]
    pub const fn child_count(&self) -> usize {
        1
    }

    /// Tests a finite world point for containment, including the boundary.
    ///
    /// # Errors
    ///
    /// Returns [`CollisionError::NonFiniteValue`] for invalid query geometry.
    pub fn test_point(&self, transform: Transform, point: Vec2) -> Result<bool, CollisionError> {
        validate_query(transform, point)?;
        let center = transform.position() + transform.rotation().apply(self.center);
        let offset = point - center;
        Ok(offset.dot(offset) <= self.radius * self.radius)
    }

    /// Computes signed distance and a finite outward normal.
    ///
    /// At the exact center the pinned C++ kernel divides by zero. This safe
    /// API returns the signed distance with a zero normal instead.
    ///
    /// # Errors
    ///
    /// Returns [`CollisionError::NonFiniteValue`] for invalid query geometry
    /// or a non-finite derived result.
    pub fn distance_to_point(
        &self,
        transform: Transform,
        point: Vec2,
    ) -> Result<PointDistance, CollisionError> {
        validate_query(transform, point)?;
        let center = transform.position() + transform.rotation().apply(self.center);
        let offset = point - center;
        let length = offset.length();
        let normal = if length > 0.0 {
            (1.0 / length) * offset
        } else {
            Vec2::ZERO
        };
        PointDistance::new(length - self.radius, normal)
    }

    /// Casts a ray using the pinned quadratic expression and interval checks.
    ///
    /// # Errors
    ///
    /// Returns [`CollisionError::NonFiniteValue`] for an invalid transform or
    /// non-finite intermediate result.
    pub fn ray_cast(
        &self,
        input: RayCastInput,
        transform: Transform,
    ) -> Result<Option<RayCastHit>, CollisionError> {
        validate_transform(transform)?;
        let position = transform.position() + transform.rotation().apply(self.center);
        let s = input.start() - position;
        let b = s.dot(s) - self.radius * self.radius;
        let ray = input.end() - input.start();
        let c = s.dot(ray);
        let ray_length_squared = ray.dot(ray);
        let sigma = c * c - ray_length_squared * b;
        if !sigma.is_finite() || !ray_length_squared.is_finite() {
            return Err(CollisionError::NonFiniteValue);
        }
        if sigma < 0.0 || ray_length_squared < EPSILON {
            return Ok(None);
        }

        let numerator = -(c + sigma.sqrt());
        if numerator < 0.0 || input.max_fraction() * ray_length_squared < numerator {
            return Ok(None);
        }

        let fraction = numerator / ray_length_squared;
        let mut normal = s + fraction * ray;
        if normal.normalize() == 0.0 {
            return Ok(None);
        }
        Ok(Some(RayCastHit::new(normal, fraction)?))
    }

    /// Computes the transformed circle AABB.
    ///
    /// # Errors
    ///
    /// Returns a typed error if the transform or derived bounds are invalid.
    pub fn compute_aabb(&self, transform: Transform) -> Result<Aabb, CollisionError> {
        validate_transform(transform)?;
        let center = transform.position() + transform.rotation().apply(self.center);
        let radius = Vec2::new(self.radius, self.radius);
        Aabb::new(center - radius, center + radius)
    }

    /// Computes mass and inertia with pinned expression grouping.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid density or non-finite derived data.
    pub fn compute_mass(&self, density: f32) -> Result<MassData, CollisionError> {
        validate_density(density)?;
        let mass = density * PI * self.radius * self.radius;
        let inertia = mass * (0.5 * self.radius * self.radius + self.center.dot(self.center));
        MassData::new(mass, self.center, inertia)
    }
}
