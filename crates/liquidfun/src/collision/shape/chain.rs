use crate::collision::{Aabb, ChildIndex, CollisionError, MassData, RayCastHit, RayCastInput};
use crate::math::settings::{LINEAR_SLOP, POLYGON_RADIUS};
use crate::math::{Transform, Vec2};

use super::{EdgeShape, PointDistance, validate_density, validate_query, validate_vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChainTopology {
    Open,
    Closed,
}

/// An immutable owned open chain or closed loop.
///
/// Closed loops store each semantic point exactly once and derive their final
/// closing child. Optional ghost points exist only in the open topology.
#[derive(Debug, Clone, PartialEq)]
pub struct ChainShape {
    vertices: Vec<Vec2>,
    topology: ChainTopology,
    maybe_previous: Option<Vec2>,
    maybe_next: Option<Vec2>,
}

impl ChainShape {
    /// Creates an open chain with optional endpoint ghost points.
    ///
    /// # Errors
    ///
    /// Returns a typed error for fewer than two points, non-finite geometry,
    /// or adjacent points separated by at most the pinned linear slop.
    pub fn open(
        vertices: &[Vec2],
        maybe_previous: Option<Vec2>,
        maybe_next: Option<Vec2>,
    ) -> Result<Self, CollisionError> {
        validate_vertices(vertices, 2, false)?;
        validate_ghost(maybe_previous, vertices[0])?;
        validate_ghost(maybe_next, vertices[vertices.len() - 1])?;
        Ok(Self {
            vertices: vertices.to_vec(),
            topology: ChainTopology::Open,
            maybe_previous,
            maybe_next,
        })
    }

    /// Creates a closed loop without exposing a duplicate closing point.
    ///
    /// # Errors
    ///
    /// Returns a typed error for fewer than three points, non-finite geometry,
    /// or any adjacent pair separated by at most the pinned linear slop.
    pub fn closed(vertices: &[Vec2]) -> Result<Self, CollisionError> {
        validate_vertices(vertices, 3, true)?;
        Ok(Self {
            vertices: vertices.to_vec(),
            topology: ChainTopology::Closed,
            maybe_previous: None,
            maybe_next: None,
        })
    }

    /// Returns the canonical points, with no repeated closing point.
    #[must_use]
    pub fn vertices(&self) -> &[Vec2] {
        &self.vertices
    }

    /// Returns the canonical point count.
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Returns whether this chain is a closed loop.
    #[must_use]
    pub const fn is_closed(&self) -> bool {
        matches!(self.topology, ChainTopology::Closed)
    }

    /// Returns the pinned polygon skin radius.
    #[must_use]
    pub const fn radius(&self) -> f32 {
        POLYGON_RADIUS
    }

    /// Returns the number of source-ordered child segments.
    #[must_use]
    pub fn child_count(&self) -> usize {
        match self.topology {
            ChainTopology::Open => self.vertices.len() - 1,
            ChainTopology::Closed => self.vertices.len(),
        }
    }

    /// Checks and returns one child coordinate.
    ///
    /// # Errors
    ///
    /// Returns [`CollisionError::ChildIndexOutOfRange`] when absent.
    pub fn child_index(&self, requested: usize) -> Result<ChildIndex, CollisionError> {
        ChildIndex::new(requested, self.child_count())
    }

    /// Materializes one owned adjacency-bearing child edge.
    ///
    /// # Errors
    ///
    /// Returns a typed error if the child coordinate belongs to a different
    /// topology or if internal geometry cannot form a valid edge.
    pub fn child_edge(&self, child_index: ChildIndex) -> Result<EdgeShape, CollisionError> {
        let index = child_index.get();
        ChildIndex::new(index, self.child_count())?;
        let count = self.vertices.len();
        let next = (index + 1) % count;
        let maybe_previous = if index > 0 {
            Some(self.vertices[index - 1])
        } else {
            match self.topology {
                ChainTopology::Open => self.maybe_previous,
                ChainTopology::Closed => Some(self.vertices[count - 1]),
            }
        };
        let maybe_next = if index + 2 < count {
            Some(self.vertices[index + 2])
        } else {
            match self.topology {
                ChainTopology::Open => self.maybe_next,
                ChainTopology::Closed => Some(self.vertices[(index + 2) % count]),
            }
        };
        EdgeShape::with_adjacency(
            self.vertices[index],
            self.vertices[next],
            maybe_previous,
            maybe_next,
        )
    }

    /// Always returns false for finite input because a chain has no interior.
    ///
    /// # Errors
    ///
    /// Returns [`CollisionError::NonFiniteValue`] for invalid query geometry.
    pub fn test_point(&self, transform: Transform, point: Vec2) -> Result<bool, CollisionError> {
        validate_query(transform, point)?;
        Ok(false)
    }

    /// Delegates distance to the exact owned child edge.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid child or query geometry.
    pub fn distance_to_point(
        &self,
        transform: Transform,
        point: Vec2,
        child_index: ChildIndex,
    ) -> Result<PointDistance, CollisionError> {
        self.child_edge(child_index)?
            .distance_to_point(transform, point)
    }

    /// Delegates a two-sided ray cast to the exact owned child edge.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid child, transform, or derived hit.
    pub fn ray_cast(
        &self,
        input: RayCastInput,
        transform: Transform,
        child_index: ChildIndex,
    ) -> Result<Option<RayCastHit>, CollisionError> {
        self.child_edge(child_index)?.ray_cast(input, transform)
    }

    /// Delegates bounds to the exact owned child edge.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid child, transform, or derived bounds.
    pub fn compute_aabb(
        &self,
        transform: Transform,
        child_index: ChildIndex,
    ) -> Result<Aabb, CollisionError> {
        self.child_edge(child_index)?.compute_aabb(transform)
    }

    /// Returns the pinned zero chain mass at the local origin.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid density.
    pub fn compute_mass(&self, density: f32) -> Result<MassData, CollisionError> {
        validate_density(density)?;
        MassData::new(0.0, Vec2::ZERO, 0.0)
    }
}

fn validate_vertices(
    vertices: &[Vec2],
    minimum: usize,
    include_closing_pair: bool,
) -> Result<(), CollisionError> {
    if vertices.len() < minimum {
        return Err(CollisionError::InvalidGeometry);
    }
    for vertex in vertices {
        validate_vec2(*vertex)?;
    }
    for pair in vertices.windows(2) {
        validate_spacing(pair[0], pair[1])?;
    }
    if include_closing_pair {
        validate_spacing(vertices[vertices.len() - 1], vertices[0])?;
    }
    Ok(())
}

fn validate_ghost(maybe_ghost: Option<Vec2>, endpoint: Vec2) -> Result<(), CollisionError> {
    let Some(ghost) = maybe_ghost else {
        return Ok(());
    };
    validate_vec2(ghost)?;
    validate_spacing(ghost, endpoint)
}

fn validate_spacing(first: Vec2, second: Vec2) -> Result<(), CollisionError> {
    if (second - first).length_squared() <= LINEAR_SLOP * LINEAR_SLOP {
        return Err(CollisionError::InvalidGeometry);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_child_adjacency_wraps_both_ends() {
        // Arrange
        let points = [
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(0.0, 2.0),
        ];
        let chain = ChainShape::closed(&points).expect("loop should be valid");

        // Act
        let first = chain
            .child_edge(chain.child_index(0).expect("child should exist"))
            .expect("edge should be valid");

        // Assert
        assert_eq!(first.previous(), Some(points[3]));
        assert_eq!(first.next(), Some(points[2]));
    }
}
