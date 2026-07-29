use std::error::Error;

use super::{ParticleGroupId, Shape, Vec2, fmt, validate_count};

/// A non-empty owned union of shapes whose interiors are filled in source order.
///
/// Only circles and polygons have a fillable interior. The shape order is
/// retained and later sampling must evaluate the union in that order.
#[derive(Debug, Clone, PartialEq)]
pub struct FilledParticleGroupShapes {
    shapes: Box<[Shape]>,
}

impl FilledParticleGroupShapes {
    /// Creates a checked non-empty filled-shape union.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an empty or oversized collection, or when an
    /// edge or chain is supplied to an interior-filling source.
    pub fn new(shapes: Vec<Shape>) -> Result<Self, ParticleGroupRecipeError> {
        validate_count(shapes.len())?;
        if shapes.is_empty() {
            return Err(ParticleGroupRecipeError::EmptySource);
        }
        if shapes
            .iter()
            .any(|shape| matches!(shape, Shape::Edge(_) | Shape::Chain(_)))
        {
            return Err(ParticleGroupRecipeError::UnsupportedFilledShape);
        }
        Ok(Self {
            shapes: shapes.into_boxed_slice(),
        })
    }

    /// Returns the owned shapes in their fixed sampling order.
    #[must_use]
    pub fn shapes(&self) -> &[Shape] {
        &self.shapes
    }
}

/// One owned edge or chain sampled as a source-ordered stroke.
#[derive(Debug, Clone, PartialEq)]
pub struct ParticleGroupStrokeShape {
    shape: Shape,
}

impl ParticleGroupStrokeShape {
    /// Creates a checked stroke source.
    ///
    /// # Errors
    ///
    /// Returns [`ParticleGroupRecipeError::UnsupportedStrokeShape`] when a
    /// circle or polygon is supplied because those shapes select filled
    /// sampling rather than stroke sampling.
    pub fn new(shape: Shape) -> Result<Self, ParticleGroupRecipeError> {
        if matches!(shape, Shape::Circle(_) | Shape::Polygon(_)) {
            return Err(ParticleGroupRecipeError::UnsupportedStrokeShape);
        }
        Ok(Self { shape })
    }

    /// Returns the edge or chain sampled by this source.
    #[must_use]
    pub const fn shape(&self) -> &Shape {
        &self.shape
    }
}

/// Non-empty owned explicit particle positions in source order.
#[derive(Debug, Clone, PartialEq)]
pub struct ParticleGroupPositions {
    positions: Box<[Vec2]>,
}

impl ParticleGroupPositions {
    /// Creates checked explicit positions.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an empty or oversized collection or for the
    /// first non-finite position.
    pub fn new(positions: Vec<Vec2>) -> Result<Self, ParticleGroupRecipeError> {
        validate_count(positions.len())?;
        if positions.is_empty() {
            return Err(ParticleGroupRecipeError::EmptySource);
        }
        if let Some(index) = positions.iter().position(|position| !position.is_valid()) {
            return Err(ParticleGroupRecipeError::NonFinitePosition { index });
        }
        Ok(Self {
            positions: positions.into_boxed_slice(),
        })
    }

    /// Returns explicit positions in their fixed creation order.
    #[must_use]
    pub fn positions(&self) -> &[Vec2] {
        &self.positions
    }
}

/// Exactly one owned particle-group sampling source.
#[derive(Debug, Clone, PartialEq)]
pub enum ParticleGroupSource {
    /// Fill the union of one or more circles or polygons.
    FilledShapes(FilledParticleGroupShapes),
    /// Stroke one edge or chain in its child order.
    StrokeShape(ParticleGroupStrokeShape),
    /// Create one particle at each explicit position.
    Positions(ParticleGroupPositions),
}

impl ParticleGroupSource {
    /// Creates a non-empty filled-shape source.
    ///
    /// # Errors
    ///
    /// Returns a typed source validation error.
    pub fn filled_shapes(shapes: Vec<Shape>) -> Result<Self, ParticleGroupRecipeError> {
        FilledParticleGroupShapes::new(shapes).map(Self::FilledShapes)
    }

    /// Creates one checked edge or chain stroke source.
    ///
    /// # Errors
    ///
    /// Returns a typed source validation error.
    pub fn stroke_shape(shape: Shape) -> Result<Self, ParticleGroupRecipeError> {
        ParticleGroupStrokeShape::new(shape).map(Self::StrokeShape)
    }

    /// Creates a non-empty explicit-position source.
    ///
    /// # Errors
    ///
    /// Returns a typed source validation error.
    pub fn positions(positions: Vec<Vec2>) -> Result<Self, ParticleGroupRecipeError> {
        ParticleGroupPositions::new(positions).map(Self::Positions)
    }
}

/// Whether a recipe creates a distinct group or appends to a live group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleGroupDestination {
    /// Create a new group identity.
    New,
    /// Append the sampled particles to this checked opaque group identity.
    AppendTo(ParticleGroupId),
}

/// A failure while constructing a checked [`crate::particle::ParticleGroupRecipe`] or source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParticleGroupRecipeError {
    /// A source collection contains no shapes or positions.
    EmptySource,
    /// A source collection exceeds the pinned signed 32-bit count range.
    SourceCountOutOfRange {
        /// Supplied collection length.
        count: usize,
    },
    /// A filled source contains an edge or chain without an interior.
    UnsupportedFilledShape,
    /// A stroke source contains a circle or polygon.
    UnsupportedStrokeShape,
    /// An explicit position contains a non-finite coordinate.
    NonFinitePosition {
        /// Source-order position of the invalid value.
        index: usize,
    },
    /// The transform contains a non-finite translation or rotation component.
    NonFiniteTransform,
    /// The linear velocity contains a non-finite coordinate.
    NonFiniteLinearVelocity,
    /// The angular velocity is not finite.
    NonFiniteAngularVelocity,
    /// The group strength is not finite.
    NonFiniteStrength,
    /// The group strength is negative.
    NegativeStrength,
    /// The explicit stride is not finite.
    NonFiniteStride,
    /// The explicit stride is zero or negative.
    NonPositiveStride,
    /// The lifetime is not finite.
    NonFiniteLifetime,
}

impl fmt::Display for ParticleGroupRecipeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySource => formatter.write_str("particle-group source must not be empty"),
            Self::SourceCountOutOfRange { count } => write!(
                formatter,
                "particle-group source count {count} exceeds the pinned signed 32-bit range"
            ),
            Self::UnsupportedFilledShape => formatter
                .write_str("particle-group filled sources support only circles and polygons"),
            Self::UnsupportedStrokeShape => {
                formatter.write_str("particle-group stroke sources support only edges and chains")
            }
            Self::NonFinitePosition { index } => {
                write!(formatter, "particle-group position {index} must be finite")
            }
            Self::NonFiniteTransform => {
                formatter.write_str("particle-group transform must be finite")
            }
            Self::NonFiniteLinearVelocity => {
                formatter.write_str("particle-group linear velocity must be finite")
            }
            Self::NonFiniteAngularVelocity => {
                formatter.write_str("particle-group angular velocity must be finite")
            }
            Self::NonFiniteStrength => {
                formatter.write_str("particle-group strength must be finite")
            }
            Self::NegativeStrength => {
                formatter.write_str("particle-group strength must be non-negative")
            }
            Self::NonFiniteStride => formatter.write_str("particle-group stride must be finite"),
            Self::NonPositiveStride => {
                formatter.write_str("particle-group stride must be positive")
            }
            Self::NonFiniteLifetime => {
                formatter.write_str("particle-group lifetime must be finite")
            }
        }
    }
}

impl Error for ParticleGroupRecipeError {}
