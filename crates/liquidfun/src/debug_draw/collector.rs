//! Authoritative owned semantic observation-to-primitive collector.

mod layers;
mod support;

use std::error::Error;
use std::fmt;

use crate::{World, WorldObservationError, WorldObservationLimits};

use super::primitive::{DebugLayer, DebugPrimitive};
use layers::Collector;
use support::layer_index;

const REVIEWED_MAX_PRIMITIVES: usize = 131_072;
const REVIEWED_MAX_VERTICES: usize = 1_048_576;
const REVIEWED_MAX_TEXT_BYTES: usize = 262_144;
const REVIEWED_MAX_VERTICES_PER_PRIMITIVE: usize = 4_096;
const REVIEWED_MAX_LABEL_BYTES: usize = 256;
const CONTACT_POINT_RADIUS: f32 = 0.04;
const PARTICLE_CONTACT_RADIUS: f32 = 0.03;
const NORMAL_LENGTH: f32 = 0.3;
const AXIS_LENGTH: f32 = 0.4;

/// Reviewed finite capacities for one owned primitive collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugDrawLimits {
    primitives: usize,
    vertices: usize,
    text_bytes: usize,
}

impl DebugDrawLimits {
    /// Returns repository-reviewed production limits.
    #[must_use]
    pub const fn reviewed() -> Self {
        Self {
            primitives: REVIEWED_MAX_PRIMITIVES,
            vertices: REVIEWED_MAX_VERTICES,
            text_bytes: REVIEWED_MAX_TEXT_BYTES,
        }
    }

    /// Creates limits no larger than the reviewed maxima.
    #[must_use]
    pub const fn new(primitives: usize, vertices: usize, text_bytes: usize) -> Option<Self> {
        if primitives <= REVIEWED_MAX_PRIMITIVES
            && vertices <= REVIEWED_MAX_VERTICES
            && text_bytes <= REVIEWED_MAX_TEXT_BYTES
        {
            Some(Self {
                primitives,
                vertices,
                text_bytes,
            })
        } else {
            None
        }
    }

    /// Returns the primitive-record limit.
    #[must_use]
    pub const fn max_primitives(self) -> usize {
        self.primitives
    }

    /// Returns the aggregate explicit-vertex limit.
    #[must_use]
    pub const fn max_vertices(self) -> usize {
        self.vertices
    }

    /// Returns the aggregate UTF-8 label-byte limit.
    #[must_use]
    pub const fn max_text_bytes(self) -> usize {
        self.text_bytes
    }
}

/// Closed layer selection and finite collection limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugDrawOptions {
    layers: [bool; 9],
    limits: DebugDrawLimits,
}

impl DebugDrawOptions {
    /// Enables every semantic layer at reviewed limits.
    #[must_use]
    pub const fn all() -> Self {
        Self {
            layers: [true; 9],
            limits: DebugDrawLimits::reviewed(),
        }
    }

    /// Selects or clears one closed layer.
    #[must_use]
    pub const fn with_layer(mut self, layer: DebugLayer, enabled: bool) -> Self {
        self.layers[layer_index(layer)] = enabled;
        self
    }

    /// Applies checked collection limits.
    #[must_use]
    pub const fn with_limits(mut self, limits: DebugDrawLimits) -> Self {
        self.limits = limits;
        self
    }

    /// Returns whether one layer is selected.
    #[must_use]
    pub const fn includes(self, layer: DebugLayer) -> bool {
        self.layers[layer_index(layer)]
    }

    /// Returns finite collection limits.
    #[must_use]
    pub const fn limits(self) -> DebugDrawLimits {
        self.limits
    }
}

impl Default for DebugDrawOptions {
    fn default() -> Self {
        Self::all()
    }
}

/// A bounded primitive collection failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DebugCollectionError {
    /// The underlying semantic observation could not be collected.
    Observation(WorldObservationError),
    /// A reviewed collection bound would be exceeded.
    CapacityExceeded {
        /// Stable bounded resource category.
        resource: DebugCollectionResource,
        /// Configured limit.
        limit: usize,
    },
    /// Semantic input could not produce finite explicit geometry.
    InvalidGeometry {
        /// Layer whose geometry was invalid.
        layer: DebugLayer,
    },
    /// A semantic owner required by an observation was absent.
    IncompleteOwner,
}

/// Closed resource categories for debug collection limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugCollectionResource {
    /// Primitive records.
    Primitives,
    /// Explicit polyline vertices.
    Vertices,
    /// UTF-8 label bytes.
    TextBytes,
    /// Vertices in one primitive.
    PrimitiveVertices,
    /// Bytes in one label.
    LabelBytes,
}

impl fmt::Display for DebugCollectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Observation(error) => write!(formatter, "semantic observation failed: {error}"),
            Self::CapacityExceeded { resource, limit } => {
                write!(formatter, "debug {resource:?} exceed limit {limit}")
            }
            Self::InvalidGeometry { layer } => {
                write!(formatter, "{layer:?} produced invalid finite geometry")
            }
            Self::IncompleteOwner => formatter.write_str("debug owner could not be resolved"),
        }
    }
}

impl Error for DebugCollectionError {}

impl From<WorldObservationError> for DebugCollectionError {
    fn from(error: WorldObservationError) -> Self {
        Self::Observation(error)
    }
}

/// Narrow passive adapter over an already-collected authoritative model.
pub trait DebugPrimitiveSink {
    /// Adapter error.
    type Error;

    /// Consumes one borrowed primitive without changing semantic order or identity.
    ///
    /// # Errors
    ///
    /// Returns the adapter's error without consuming any later primitive.
    fn push(&mut self, primitive: &DebugPrimitive) -> Result<(), Self::Error>;
}

/// One authoritative owned primitive collection.
#[derive(Debug, Clone, PartialEq)]
pub struct DebugPrimitiveCollection {
    primitives: Vec<DebugPrimitive>,
}

impl DebugPrimitiveCollection {
    /// Returns primitives in semantic source order with declared unordered sets canonicalized.
    #[must_use]
    pub fn primitives(&self) -> &[DebugPrimitive] {
        &self.primitives
    }

    /// Replays this exact collection through a narrow passive sink.
    ///
    /// # Errors
    ///
    /// Returns the first adapter error without replaying any later primitive.
    pub fn emit_to<S: DebugPrimitiveSink>(&self, sink: &mut S) -> Result<(), S::Error> {
        for primitive in &self.primitives {
            sink.push(primitive)?;
        }
        Ok(())
    }
}

impl World {
    /// Collects renderer-neutral primitives from one owned public semantic observation.
    ///
    /// Every call starts an empty owned collection, so a failed or later frame cannot
    /// leak records from an earlier collection. Geometry never reads arena, proxy,
    /// contact-manager, joint-storage, or dense-particle coordinates.
    ///
    /// # Errors
    ///
    /// Returns before publishing output when semantic observation, owner resolution,
    /// finite-geometry validation, or a reviewed collection bound fails.
    pub fn collect_debug_primitives(
        &self,
        options: DebugDrawOptions,
    ) -> Result<DebugPrimitiveCollection, DebugCollectionError> {
        let observation = self.world_observation(WorldObservationLimits::reviewed())?;
        Collector::new(options).collect(&observation)
    }
}
