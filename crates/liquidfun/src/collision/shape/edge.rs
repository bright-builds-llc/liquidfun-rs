use crate::math::settings::POLYGON_RADIUS;
use crate::math::{Transform, Vec2};

use super::{PointDistance, validate_density, validate_query, validate_transform, validate_vec2};
use crate::collision::{Aabb, CollisionError, MassData, RayCastHit, RayCastInput};

/// An immutable owned segment with optional smooth-collision adjacency.
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeShape {
    start: Vec2,
    end: Vec2,
    maybe_previous: Option<Vec2>,
    maybe_next: Option<Vec2>,
}

impl EdgeShape {
    /// Creates an isolated finite non-degenerate edge.
    ///
    /// # Errors
    ///
    /// Returns a typed error for non-finite or equal endpoints.
    pub fn new(start: Vec2, end: Vec2) -> Result<Self, CollisionError> {
        Self::with_adjacency(start, end, None, None)
    }

    /// Creates an edge with optional previous and next adjacency.
    ///
    /// Adjacency equal to the endpoint it extends is rejected so a zero-length
    /// neighboring segment cannot enter later edge collision kernels.
    ///
    /// # Errors
    ///
    /// Returns a typed error for non-finite points, equal endpoints, or a
    /// ghost point equal to the endpoint it extends.
    pub fn with_adjacency(
        start: Vec2,
        end: Vec2,
        maybe_previous: Option<Vec2>,
        maybe_next: Option<Vec2>,
    ) -> Result<Self, CollisionError> {
        validate_vec2(start)?;
        validate_vec2(end)?;
        if start == end {
            return Err(CollisionError::InvalidGeometry);
        }
        if let Some(previous) = maybe_previous {
            validate_vec2(previous)?;
            if previous == start {
                return Err(CollisionError::InvalidGeometry);
            }
        }
        if let Some(next) = maybe_next {
            validate_vec2(next)?;
            if next == end {
                return Err(CollisionError::InvalidGeometry);
            }
        }
        Ok(Self {
            start,
            end,
            maybe_previous,
            maybe_next,
        })
    }

    /// Returns the first endpoint.
    #[must_use]
    pub const fn start(&self) -> Vec2 {
        self.start
    }

    /// Returns the second endpoint.
    #[must_use]
    pub const fn end(&self) -> Vec2 {
        self.end
    }

    /// Returns the optional preceding point used for smooth adjacency.
    #[must_use]
    pub const fn previous(&self) -> Option<Vec2> {
        self.maybe_previous
    }

    /// Returns the optional following point used for smooth adjacency.
    #[must_use]
    pub const fn next(&self) -> Option<Vec2> {
        self.maybe_next
    }

    /// Returns the pinned polygon skin radius.
    #[must_use]
    pub const fn radius(&self) -> f32 {
        POLYGON_RADIUS
    }

    /// Returns the single child count.
    #[must_use]
    pub const fn child_count(&self) -> usize {
        1
    }

    /// Always returns false for finite input because an edge has no interior.
    ///
    /// # Errors
    ///
    /// Returns [`CollisionError::NonFiniteValue`] for invalid query geometry.
    pub fn test_point(&self, transform: Transform, point: Vec2) -> Result<bool, CollisionError> {
        validate_query(transform, point)?;
        Ok(false)
    }

    /// Computes unsigned distance to the transformed segment.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid query geometry or derived data.
    pub fn distance_to_point(
        &self,
        transform: Transform,
        point: Vec2,
    ) -> Result<PointDistance, CollisionError> {
        validate_query(transform, point)?;
        let start = transform.apply(self.start);
        let end = transform.apply(self.end);
        let mut offset = point - start;
        let segment = end - start;
        let projection = offset.dot(segment);
        if projection > 0.0 {
            let segment_length_squared = segment.dot(segment);
            if projection > segment_length_squared {
                offset = point - end;
            } else {
                offset -= (projection / segment_length_squared) * segment;
            }
        }
        let length = offset.length();
        let normal = if length > 0.0 {
            (1.0 / length) * offset
        } else {
            Vec2::ZERO
        };
        PointDistance::new(length, normal)
    }

    /// Casts a two-sided ray against this segment.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid transform or derived hit.
    pub fn ray_cast(
        &self,
        input: RayCastInput,
        transform: Transform,
    ) -> Result<Option<RayCastHit>, CollisionError> {
        validate_transform(transform)?;
        let first = transform
            .rotation()
            .inverse_apply(input.start() - transform.position());
        let second = transform
            .rotation()
            .inverse_apply(input.end() - transform.position());
        let direction = second - first;
        let edge = self.end - self.start;
        let mut normal = Vec2::new(edge.y, -edge.x);
        if normal.normalize() == 0.0 {
            return Err(CollisionError::InvalidGeometry);
        }
        let numerator = normal.dot(self.start - first);
        let denominator = normal.dot(direction);
        if denominator == 0.0 {
            return Ok(None);
        }
        let fraction = numerator / denominator;
        if fraction < 0.0 || input.max_fraction() < fraction {
            return Ok(None);
        }
        let intersection = first + fraction * direction;
        let segment_length_squared = edge.dot(edge);
        let segment_fraction = (intersection - self.start).dot(edge) / segment_length_squared;
        if !(0.0..=1.0).contains(&segment_fraction) {
            return Ok(None);
        }
        let world_normal = transform.rotation().apply(normal);
        let outward_normal = if numerator > 0.0 {
            -world_normal
        } else {
            world_normal
        };
        Ok(Some(RayCastHit::new(outward_normal, fraction)?))
    }

    /// Computes the transformed edge AABB expanded by the polygon radius.
    ///
    /// # Errors
    ///
    /// Returns a typed error if the transform or derived bounds are invalid.
    pub fn compute_aabb(&self, transform: Transform) -> Result<Aabb, CollisionError> {
        validate_transform(transform)?;
        let start = transform.apply(self.start);
        let end = transform.apply(self.end);
        let lower = Vec2::new(start.x.min(end.x), start.y.min(end.y));
        let upper = Vec2::new(start.x.max(end.x), start.y.max(end.y));
        let radius = Vec2::new(POLYGON_RADIUS, POLYGON_RADIUS);
        Aabb::new(lower - radius, upper + radius)
    }

    /// Computes the pinned zero edge mass with midpoint center.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid density or derived midpoint.
    pub fn compute_mass(&self, density: f32) -> Result<MassData, CollisionError> {
        validate_density(density)?;
        MassData::new(0.0, 0.5 * (self.start + self.end), 0.0)
    }
}
