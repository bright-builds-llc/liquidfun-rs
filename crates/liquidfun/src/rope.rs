//! Checked standalone rope simulation translated from upstream `b2Rope`.
//!
//! [`Rope`] owns a renderer-independent pure simulation state. It is not a
//! world object, has no handle or body dependency, and is deliberately
//! separate from the world-owned [`crate::RopeJointDef`] constraint.

mod core;

use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::math::Vec2;

use self::core::RopeCore;

const MINIMUM_VERTEX_COUNT: usize = 3;
const MAXIMUM_VERTEX_COUNT: usize = 4_096;

/// A checked owned definition for a standalone [`Rope`].
#[derive(Clone, Debug, PartialEq)]
pub struct RopeDef {
    vertices: Vec<Vec2>,
    masses: Vec<f32>,
    gravity: Vec2,
    damping: f32,
    stretching_stiffness: f32,
    bending_stiffness: f32,
}

impl RopeDef {
    /// Maximum reviewed vertex count accepted by a standalone rope.
    pub const MAX_VERTICES: usize = MAXIMUM_VERTEX_COUNT;

    /// Creates an invariant-bearing owned rope definition.
    ///
    /// The vertex and mass lanes must have the same length between three and
    /// [`Self::MAX_VERTICES`]. All values must be finite. Masses, damping, and
    /// stiffness values must also be nonnegative; a zero mass fixes a vertex.
    ///
    /// # Errors
    ///
    /// Returns [`RopeError`] when a lane length or supplied value violates
    /// those invariants.
    pub fn new(
        vertices: Vec<Vec2>,
        masses: Vec<f32>,
        gravity: Vec2,
        damping: f32,
        stretching_stiffness: f32,
        bending_stiffness: f32,
    ) -> Result<Self, RopeError> {
        validate_lanes(&vertices, &masses)?;
        validate_definition_values(
            &vertices,
            &masses,
            gravity,
            damping,
            stretching_stiffness,
            bending_stiffness,
        )?;

        Ok(Self {
            vertices,
            masses,
            gravity,
            damping,
            stretching_stiffness,
            bending_stiffness,
        })
    }

    /// Returns the definition's owned initial vertices.
    #[must_use]
    pub fn vertices(&self) -> &[Vec2] {
        &self.vertices
    }

    /// Returns the definition's per-vertex masses.
    #[must_use]
    pub fn masses(&self) -> &[f32] {
        &self.masses
    }

    /// Returns the acceleration applied to every free vertex.
    #[must_use]
    pub const fn gravity(&self) -> Vec2 {
        self.gravity
    }

    /// Returns the exponential velocity damping coefficient.
    #[must_use]
    pub const fn damping(&self) -> f32 {
        self.damping
    }

    /// Returns the stretching constraint stiffness.
    #[must_use]
    pub const fn stretching_stiffness(&self) -> f32 {
        self.stretching_stiffness
    }

    /// Returns the bending constraint stiffness.
    #[must_use]
    pub const fn bending_stiffness(&self) -> f32 {
        self.bending_stiffness
    }
}

/// A reviewed bounded standalone-rope solver iteration count.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct RopeIterations(usize);

impl RopeIterations {
    /// Maximum number of constraint iterations accepted by one step.
    pub const MAX: usize = 1_024;

    /// Validates an iteration count. Zero is source-supported.
    ///
    /// # Errors
    ///
    /// Returns [`RopeError::IterationCountOutOfRange`] above [`Self::MAX`].
    pub const fn new(count: usize) -> Result<Self, RopeError> {
        if count > Self::MAX {
            return Err(RopeError::IterationCountOutOfRange {
                count,
                maximum: Self::MAX,
            });
        }

        Ok(Self(count))
    }

    /// Returns the checked count.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

/// A checked standalone rope simulation.
///
/// Stepping clones the compact pure state, evaluates the complete candidate,
/// and replaces the live state only after every derived value remains finite.
#[derive(Clone, Debug, PartialEq)]
pub struct Rope {
    core: RopeCore,
}

impl Rope {
    /// Initializes the owned rope state and source rest constraints.
    ///
    /// # Errors
    ///
    /// Returns [`RopeError::NonFiniteDerivedState`] if rest lengths, rest
    /// angles, or inverse masses overflow from otherwise finite inputs.
    pub fn new(definition: RopeDef) -> Result<Self, RopeError> {
        RopeCore::new(definition).map(|core| Self { core })
    }

    /// Returns the current vertices through a borrow scoped to this rope.
    #[must_use]
    pub fn vertices(&self) -> &[Vec2] {
        self.core.vertices()
    }

    /// Returns the number of vertices in this rope.
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.vertices().len()
    }

    /// Advances the standalone rope with pinned source ordering.
    ///
    /// A zero timestep is a bit-identical no-op. A positive step integrates in
    /// vertex order, solves stretch, bend, then stretch per iteration, and
    /// reconstructs velocity in vertex order. Invalid input or derived
    /// arithmetic leaves the live rope unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`RopeError`] for an invalid timestep or any non-finite or
    /// unbounded source-ordered derived operation.
    pub fn step(&mut self, time_step: f32, iterations: RopeIterations) -> Result<(), RopeError> {
        if !time_step.is_finite() {
            return Err(RopeError::NonFiniteTimeStep);
        }
        if time_step < 0.0 {
            return Err(RopeError::NegativeTimeStep);
        }
        if time_step == 0.0 {
            return Ok(());
        }

        let mut candidate = self.core.clone();
        candidate.step(time_step, iterations)?;
        self.core = candidate;
        Ok(())
    }

    /// Sets the rest angle for every bending constraint.
    ///
    /// The pinned solver wraps the later angular error across `±PI`; this
    /// method changes only the finite target and leaves vertices untouched.
    ///
    /// # Errors
    ///
    /// Returns [`RopeError::NonFiniteAngle`] for a non-finite target.
    pub fn set_angle(&mut self, angle: f32) -> Result<(), RopeError> {
        if !angle.is_finite() {
            return Err(RopeError::NonFiniteAngle);
        }

        self.core.set_angle(angle);
        Ok(())
    }
}

/// Failure from checked standalone rope construction or stepping.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RopeError {
    /// Vertex and mass lanes have different lengths.
    VertexMassLengthMismatch {
        /// Number of supplied vertices.
        vertices: usize,
        /// Number of supplied masses.
        masses: usize,
    },
    /// Fewer than the source-required three vertices were provided.
    TooFewVertices {
        /// Supplied vertex count.
        count: usize,
        /// Source-required minimum.
        minimum: usize,
    },
    /// The reviewed vertex bound was exceeded.
    TooManyVertices {
        /// Supplied vertex count.
        count: usize,
        /// Reviewed maximum.
        maximum: usize,
    },
    /// An initial vertex is non-finite.
    NonFiniteVertex {
        /// Index of the rejected vertex.
        index: usize,
    },
    /// A mass is non-finite.
    NonFiniteMass {
        /// Index of the rejected mass.
        index: usize,
    },
    /// A mass is negative.
    NegativeMass {
        /// Index of the rejected mass.
        index: usize,
    },
    /// Gravity is non-finite.
    NonFiniteGravity,
    /// Damping is non-finite.
    NonFiniteDamping,
    /// Damping is negative.
    NegativeDamping,
    /// Stretch stiffness is non-finite.
    NonFiniteStretchingStiffness,
    /// Stretch stiffness is negative.
    NegativeStretchingStiffness,
    /// Bend stiffness is non-finite.
    NonFiniteBendingStiffness,
    /// Bend stiffness is negative.
    NegativeBendingStiffness,
    /// A timestep is non-finite.
    NonFiniteTimeStep,
    /// A timestep is negative.
    NegativeTimeStep,
    /// An angle is non-finite.
    NonFiniteAngle,
    /// An iteration count exceeds the reviewed bound.
    IterationCountOutOfRange {
        /// Supplied iteration count.
        count: usize,
        /// Reviewed maximum.
        maximum: usize,
    },
    /// Source-ordered derived arithmetic became non-finite.
    NonFiniteDerivedState {
        /// Vertex or constraint index at which arithmetic failed.
        index: usize,
    },
    /// Source angle wrapping did not converge within the reviewed bound.
    AngleWrapLimitExceeded {
        /// Bending-constraint index whose wrapping did not converge.
        constraint: usize,
    },
}

impl Display for RopeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VertexMassLengthMismatch { vertices, masses } => write!(
                formatter,
                "rope vertex and mass lane lengths differ ({vertices} versus {masses})"
            ),
            Self::TooFewVertices { count, minimum } => {
                write!(formatter, "rope has {count} vertices; minimum is {minimum}")
            }
            Self::TooManyVertices { count, maximum } => {
                write!(formatter, "rope has {count} vertices; maximum is {maximum}")
            }
            Self::NonFiniteVertex { index } => {
                write!(formatter, "rope vertex {index} must be finite")
            }
            Self::NonFiniteMass { index } => {
                write!(formatter, "rope mass {index} must be finite")
            }
            Self::NegativeMass { index } => {
                write!(formatter, "rope mass {index} must be nonnegative")
            }
            Self::NonFiniteGravity => formatter.write_str("rope gravity must be finite"),
            Self::NonFiniteDamping => formatter.write_str("rope damping must be finite"),
            Self::NegativeDamping => formatter.write_str("rope damping must be nonnegative"),
            Self::NonFiniteStretchingStiffness => {
                formatter.write_str("rope stretching stiffness must be finite")
            }
            Self::NegativeStretchingStiffness => {
                formatter.write_str("rope stretching stiffness must be nonnegative")
            }
            Self::NonFiniteBendingStiffness => {
                formatter.write_str("rope bending stiffness must be finite")
            }
            Self::NegativeBendingStiffness => {
                formatter.write_str("rope bending stiffness must be nonnegative")
            }
            Self::NonFiniteTimeStep => formatter.write_str("rope timestep must be finite"),
            Self::NegativeTimeStep => formatter.write_str("rope timestep must be nonnegative"),
            Self::NonFiniteAngle => formatter.write_str("rope angle must be finite"),
            Self::IterationCountOutOfRange { count, maximum } => write!(
                formatter,
                "rope iteration count {count} exceeds maximum {maximum}"
            ),
            Self::NonFiniteDerivedState { index } => {
                write!(
                    formatter,
                    "rope derived state at index {index} is non-finite"
                )
            }
            Self::AngleWrapLimitExceeded { constraint } => write!(
                formatter,
                "rope angle wrapping exceeded its bound at constraint {constraint}"
            ),
        }
    }
}

impl Error for RopeError {}

fn validate_lanes(vertices: &[Vec2], masses: &[f32]) -> Result<(), RopeError> {
    if vertices.len() != masses.len() {
        return Err(RopeError::VertexMassLengthMismatch {
            vertices: vertices.len(),
            masses: masses.len(),
        });
    }
    if vertices.len() < MINIMUM_VERTEX_COUNT {
        return Err(RopeError::TooFewVertices {
            count: vertices.len(),
            minimum: MINIMUM_VERTEX_COUNT,
        });
    }
    if vertices.len() > MAXIMUM_VERTEX_COUNT {
        return Err(RopeError::TooManyVertices {
            count: vertices.len(),
            maximum: MAXIMUM_VERTEX_COUNT,
        });
    }

    Ok(())
}

fn validate_definition_values(
    vertices: &[Vec2],
    masses: &[f32],
    gravity: Vec2,
    damping: f32,
    stretching_stiffness: f32,
    bending_stiffness: f32,
) -> Result<(), RopeError> {
    for (index, vertex) in vertices.iter().copied().enumerate() {
        if !vertex.is_valid() {
            return Err(RopeError::NonFiniteVertex { index });
        }
    }
    for (index, mass) in masses.iter().copied().enumerate() {
        if !mass.is_finite() {
            return Err(RopeError::NonFiniteMass { index });
        }
        if mass < 0.0 {
            return Err(RopeError::NegativeMass { index });
        }
    }
    if !gravity.is_valid() {
        return Err(RopeError::NonFiniteGravity);
    }
    validate_nonnegative_coefficients(damping, stretching_stiffness, bending_stiffness)
}

fn validate_nonnegative_coefficients(
    damping: f32,
    stretching_stiffness: f32,
    bending_stiffness: f32,
) -> Result<(), RopeError> {
    if !damping.is_finite() {
        return Err(RopeError::NonFiniteDamping);
    }
    if damping < 0.0 {
        return Err(RopeError::NegativeDamping);
    }
    if !stretching_stiffness.is_finite() {
        return Err(RopeError::NonFiniteStretchingStiffness);
    }
    if stretching_stiffness < 0.0 {
        return Err(RopeError::NegativeStretchingStiffness);
    }
    if !bending_stiffness.is_finite() {
        return Err(RopeError::NonFiniteBendingStiffness);
    }
    if bending_stiffness < 0.0 {
        return Err(RopeError::NegativeBendingStiffness);
    }

    Ok(())
}
