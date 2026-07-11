use std::error::Error;
use std::fmt;

/// A failure at a checked collision-domain boundary.
///
/// Variants describe semantic input categories and deliberately omit private
/// node slots, simplex storage, packed feature keys, and other implementation
/// coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CollisionError {
    /// A geometry, transform, ray, mass, or sweep input is not finite.
    NonFiniteValue,
    /// Shape geometry is finite but degenerate or otherwise unsupported.
    InvalidGeometry,
    /// An AABB has a lower coordinate greater than its corresponding upper coordinate.
    InvalidBounds,
    /// A normalized fraction lies outside the inclusive `0.0..=1.0` interval.
    FractionOutOfRange,
    /// A child selection does not name an existing shape child.
    ChildIndexOutOfRange {
        /// The requested public child coordinate.
        requested: usize,
        /// The number of available children.
        child_count: usize,
    },
    /// Reusable distance state does not match the supplied shape topology.
    IncompatibleDistanceCache,
    /// A shape-child distance proxy does not match the supplied shape or child.
    IncompatibleShapeProxy,
    /// The requested shape pair has no manifold kernel in the pinned registry.
    UnsupportedShapePair,
}

impl fmt::Display for CollisionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteValue => formatter.write_str("collision input must be finite"),
            Self::InvalidGeometry => {
                formatter.write_str("shape geometry is degenerate or unsupported")
            }
            Self::InvalidBounds => formatter.write_str("AABB bounds are reversed"),
            Self::FractionOutOfRange => {
                formatter.write_str("fraction must be in the inclusive interval 0.0..=1.0")
            }
            Self::ChildIndexOutOfRange {
                requested,
                child_count,
            } => write!(
                formatter,
                "shape child {requested} is outside the available child count {child_count}",
            ),
            Self::IncompatibleDistanceCache => {
                formatter.write_str("distance cache does not match the supplied shape topology")
            }
            Self::IncompatibleShapeProxy => {
                formatter.write_str("distance proxy does not match the supplied shape child")
            }
            Self::UnsupportedShapePair => {
                formatter.write_str("shape pair is unsupported by the pinned manifold registry")
            }
        }
    }
}

impl Error for CollisionError {}
