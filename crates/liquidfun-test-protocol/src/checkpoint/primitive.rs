//! Engine-neutral exact-bit mirror of the public debug primitive vocabulary.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    FloatBits, ScenarioId, TransformBits, Vec2Bits, codec::BoundedString, codec::BoundedVec,
};

/// Maximum vertices carried by one wire primitive.
pub const CHECKPOINT_MAXIMUM_PRIMITIVE_VERTICES: usize = 64;
/// Maximum bytes carried by one inert label.
pub const CHECKPOINT_MAXIMUM_LABEL_BYTES: usize = 256;

/// Stable semantic owner of one engine-neutral primitive.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "semantic_id", rename_all = "snake_case")]
pub enum DebugOwnerId {
    /// World-level semantic information.
    World,
    /// One rigid body.
    Body(ScenarioId),
    /// One fixture.
    Fixture(ScenarioId),
    /// One joint.
    Joint(ScenarioId),
    /// One source-significant rigid-contact occurrence.
    Contact(ScenarioId),
    /// One particle system.
    ParticleSystem(ScenarioId),
    /// One stable particle.
    Particle(ScenarioId),
    /// One source-significant particle-contact occurrence.
    ParticleContact(ScenarioId),
}

/// Closed semantic debug layer mirroring the public engine vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DebugLayerName {
    /// Collision shapes.
    Shapes,
    /// Joint constraints.
    Joints,
    /// Contact points.
    Contacts,
    /// Contact normal vectors.
    ContactNormals,
    /// Particle geometry.
    Particles,
    /// Particle contact geometry.
    ParticleContacts,
    /// Broad-phase bounds.
    BroadPhase,
    /// Body centers of mass.
    CentersOfMass,
    /// Inert semantic labels.
    Labels,
}

/// Closed debug primitive kind mirroring the public engine vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DebugPrimitiveKindName {
    /// Point marker.
    Point,
    /// Line segment.
    Segment,
    /// Open or closed polyline.
    Polyline,
    /// Circle outline or fill.
    Circle,
    /// Transform coordinate axes.
    TransformAxes,
    /// Axis-aligned bounding box.
    Aabb,
    /// Directed arrow.
    Arrow,
    /// Inert semantic text label.
    Label,
}

/// Stable semantic primitive key without renderer or storage coordinates.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebugPrimitiveKey {
    owner: DebugOwnerId,
    layer: DebugLayerName,
    kind: DebugPrimitiveKindName,
    child: u32,
    ordinal: u32,
}

impl DebugPrimitiveKey {
    /// Creates one stable semantic primitive key.
    #[must_use]
    pub const fn new(
        owner: DebugOwnerId,
        layer: DebugLayerName,
        kind: DebugPrimitiveKindName,
        child: u32,
        ordinal: u32,
    ) -> Self {
        Self {
            owner,
            layer,
            kind,
            child,
            ordinal,
        }
    }

    /// Returns the stable semantic owner.
    #[must_use]
    pub const fn owner(&self) -> &DebugOwnerId {
        &self.owner
    }

    /// Returns the closed debug layer.
    #[must_use]
    pub const fn layer(&self) -> DebugLayerName {
        self.layer
    }

    /// Returns the closed primitive kind.
    #[must_use]
    pub const fn kind(&self) -> DebugPrimitiveKindName {
        self.kind
    }

    /// Returns the stable child index within the semantic owner.
    #[must_use]
    pub const fn child(&self) -> u32 {
        self.child
    }

    /// Returns the stable occurrence ordinal within the owner and child.
    #[must_use]
    pub const fn ordinal(&self) -> u32 {
        self.ordinal
    }
}

/// Exact renderer-neutral RGBA color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DebugColorBits([u8; 4]);

impl DebugColorBits {
    /// Creates one exact RGBA color.
    #[must_use]
    pub const fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self([red, green, blue, alpha])
    }

    /// Returns red, green, blue, and alpha components.
    #[must_use]
    pub const fn components(self) -> [u8; 4] {
        self.0
    }
}

/// Exact-bit stroke metadata in world-space meters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebugStrokeBits {
    color: DebugColorBits,
    width_bits: FloatBits,
}

impl DebugStrokeBits {
    /// Creates finite non-negative stroke metadata.
    ///
    /// # Errors
    ///
    /// Returns [`super::CheckpointValidationError`] for an invalid width.
    pub fn new(
        color: DebugColorBits,
        width_bits: FloatBits,
    ) -> Result<Self, super::CheckpointValidationError> {
        super::require_nonnegative_finite(width_bits)?;
        Ok(Self { color, width_bits })
    }

    /// Returns the exact stroke color.
    #[must_use]
    pub const fn color(self) -> DebugColorBits {
        self.color
    }

    /// Returns exact world-space stroke-width bits.
    #[must_use]
    pub const fn width_bits(self) -> FloatBits {
        self.width_bits
    }
}

/// Exact optional fill metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebugFillBits {
    color: DebugColorBits,
}

impl DebugFillBits {
    /// Creates exact fill metadata.
    #[must_use]
    pub const fn new(color: DebugColorBits) -> Self {
        Self { color }
    }

    /// Returns the exact fill color.
    #[must_use]
    pub const fn color(self) -> DebugColorBits {
        self.color
    }
}

/// Common stable key and closed style metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrimitiveMetadata {
    key: DebugPrimitiveKey,
    stroke: DebugStrokeBits,
    maybe_fill: Option<DebugFillBits>,
}

impl PrimitiveMetadata {
    /// Creates stable semantic key and style metadata.
    #[must_use]
    pub const fn new(
        key: DebugPrimitiveKey,
        stroke: DebugStrokeBits,
        maybe_fill: Option<DebugFillBits>,
    ) -> Self {
        Self {
            key,
            stroke,
            maybe_fill,
        }
    }

    /// Returns the stable semantic primitive key.
    #[must_use]
    pub const fn key(&self) -> &DebugPrimitiveKey {
        &self.key
    }

    /// Returns exact stroke metadata.
    #[must_use]
    pub const fn stroke(&self) -> DebugStrokeBits {
        self.stroke
    }

    /// Returns optional exact fill metadata.
    #[must_use]
    pub const fn maybe_fill(&self) -> Option<DebugFillBits> {
        self.maybe_fill
    }
}

macro_rules! primitive_record {
    ($name:ident, $doc:literal, { $($field:ident : $type:ty),* $(,)? }) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name {
            metadata: PrimitiveMetadata,
            $(
                $field: $type,
            )*
        }

        impl $name {
            /// Returns common stable key and style metadata.
            #[must_use]
            pub const fn metadata(&self) -> &PrimitiveMetadata {
                &self.metadata
            }

            $(
                #[doc = concat!("Returns exact semantic field `", stringify!($field), "`.")]
                #[must_use]
                pub const fn $field(&self) -> $type {
                    self.$field
                }
            )*
        }
    };
}

primitive_record!(PrimitivePoint, "One exact-bit point marker.", {
    position: Vec2Bits,
    radius_bits: FloatBits,
});
primitive_record!(PrimitiveSegment, "One exact-bit segment.", {
    start: Vec2Bits,
    end: Vec2Bits,
});
primitive_record!(PrimitiveCircle, "One exact-bit circle.", {
    center: Vec2Bits,
    radius_bits: FloatBits,
});
primitive_record!(PrimitiveTransformAxes, "One exact-bit coordinate frame.", {
    transform: TransformBits,
    scale_bits: FloatBits,
});
primitive_record!(PrimitiveAabb, "One exact-bit axis-aligned box.", {
    lower: Vec2Bits,
    upper: Vec2Bits,
});
primitive_record!(PrimitiveArrow, "One exact-bit directed arrow.", {
    start: Vec2Bits,
    end: Vec2Bits,
});

impl PrimitivePoint {
    /// Creates one exact-bit point primitive.
    #[must_use]
    pub const fn new(
        key: DebugPrimitiveKey,
        stroke: DebugStrokeBits,
        maybe_fill: Option<DebugFillBits>,
        position: Vec2Bits,
        radius_bits: FloatBits,
    ) -> Self {
        Self {
            metadata: PrimitiveMetadata::new(key, stroke, maybe_fill),
            position,
            radius_bits,
        }
    }
}

/// One source-ordered open or closed polyline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PrimitivePolyline {
    metadata: PrimitiveMetadata,
    vertices: Box<[Vec2Bits]>,
    closed: bool,
}

impl PrimitivePolyline {
    /// Returns common stable key and style metadata.
    #[must_use]
    pub const fn metadata(&self) -> &PrimitiveMetadata {
        &self.metadata
    }

    /// Returns source-ordered exact-bit vertices.
    #[must_use]
    pub fn vertices(&self) -> &[Vec2Bits] {
        &self.vertices
    }

    /// Returns whether the source-ordered polyline is closed.
    #[must_use]
    pub const fn closed(&self) -> bool {
        self.closed
    }
}

impl<'de> Deserialize<'de> for PrimitivePolyline {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            metadata: PrimitiveMetadata,
            vertices: BoundedVec<Vec2Bits, CHECKPOINT_MAXIMUM_PRIMITIVE_VERTICES>,
            closed: bool,
        }
        let raw = Raw::deserialize(deserializer)?;
        Ok(Self {
            metadata: raw.metadata,
            vertices: raw.vertices.into_vec().into_boxed_slice(),
            closed: raw.closed,
        })
    }
}

/// One bounded inert semantic label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PrimitiveLabel {
    metadata: PrimitiveMetadata,
    position: Vec2Bits,
    text: Box<str>,
}

impl PrimitiveLabel {
    /// Returns common stable key and style metadata.
    #[must_use]
    pub const fn metadata(&self) -> &PrimitiveMetadata {
        &self.metadata
    }

    /// Returns the exact-bit label anchor.
    #[must_use]
    pub const fn position(&self) -> Vec2Bits {
        self.position
    }

    /// Returns the bounded inert label text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

impl<'de> Deserialize<'de> for PrimitiveLabel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            metadata: PrimitiveMetadata,
            position: Vec2Bits,
            text: BoundedString<CHECKPOINT_MAXIMUM_LABEL_BYTES>,
        }
        let raw = Raw::deserialize(deserializer)?;
        Ok(Self {
            metadata: raw.metadata,
            position: raw.position,
            text: raw.text.into_string().into_boxed_str(),
        })
    }
}

/// Closed renderer-neutral primitive vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum WireDebugPrimitive {
    /// Point marker.
    Point(PrimitivePoint),
    /// Line segment.
    Segment(PrimitiveSegment),
    /// Open or closed polyline.
    Polyline(PrimitivePolyline),
    /// Circle outline or fill.
    Circle(PrimitiveCircle),
    /// Transform coordinate axes.
    TransformAxes(PrimitiveTransformAxes),
    /// Axis-aligned bounding box.
    Aabb(PrimitiveAabb),
    /// Directed arrow.
    Arrow(PrimitiveArrow),
    /// Inert semantic label.
    Label(PrimitiveLabel),
}

impl WireDebugPrimitive {
    /// Returns common stable key and style metadata.
    #[must_use]
    pub const fn metadata(&self) -> &PrimitiveMetadata {
        match self {
            Self::Point(value) => value.metadata(),
            Self::Segment(value) => value.metadata(),
            Self::Polyline(value) => value.metadata(),
            Self::Circle(value) => value.metadata(),
            Self::TransformAxes(value) => value.metadata(),
            Self::Aabb(value) => value.metadata(),
            Self::Arrow(value) => value.metadata(),
            Self::Label(value) => value.metadata(),
        }
    }

    pub(super) fn validate(&self) -> Result<usize, super::CheckpointValidationError> {
        let metadata = self.metadata();
        super::require_nonnegative_finite(metadata.stroke().width_bits())?;
        let (kind, vertices) = match self {
            Self::Point(value) => {
                validate_vec2(value.position)?;
                super::require_nonnegative_finite(value.radius_bits)?;
                (DebugPrimitiveKindName::Point, 1)
            }
            Self::Segment(value) => {
                validate_vec2(value.start)?;
                validate_vec2(value.end)?;
                (DebugPrimitiveKindName::Segment, 2)
            }
            Self::Polyline(value) => {
                if value.vertices.is_empty()
                    || value.vertices.len() > CHECKPOINT_MAXIMUM_PRIMITIVE_VERTICES
                {
                    return Err(super::validation(
                        super::CheckpointErrorKind::BoundaryLimitExceeded,
                    ));
                }
                for vertex in &value.vertices {
                    validate_vec2(*vertex)?;
                }
                (DebugPrimitiveKindName::Polyline, value.vertices.len())
            }
            Self::Circle(value) => {
                validate_vec2(value.center)?;
                super::require_nonnegative_finite(value.radius_bits)?;
                (DebugPrimitiveKindName::Circle, 1)
            }
            Self::TransformAxes(value) => {
                validate_vec2(value.transform.position)?;
                super::require_finite(value.transform.angle_bits)?;
                super::require_nonnegative_finite(value.scale_bits)?;
                (DebugPrimitiveKindName::TransformAxes, 1)
            }
            Self::Aabb(value) => {
                validate_vec2(value.lower)?;
                validate_vec2(value.upper)?;
                if value.lower.x_bits.to_f32() > value.upper.x_bits.to_f32()
                    || value.lower.y_bits.to_f32() > value.upper.y_bits.to_f32()
                {
                    return Err(super::validation(
                        super::CheckpointErrorKind::InvalidPrimitive,
                    ));
                }
                (DebugPrimitiveKindName::Aabb, 2)
            }
            Self::Arrow(value) => {
                validate_vec2(value.start)?;
                validate_vec2(value.end)?;
                (DebugPrimitiveKindName::Arrow, 2)
            }
            Self::Label(value) => {
                validate_vec2(value.position)?;
                if value.text.is_empty()
                    || value.text.len() > CHECKPOINT_MAXIMUM_LABEL_BYTES
                    || value.text.chars().any(char::is_control)
                {
                    return Err(super::validation(
                        super::CheckpointErrorKind::InvalidPrimitive,
                    ));
                }
                (DebugPrimitiveKindName::Label, 1)
            }
        };
        if metadata.key().kind() != kind {
            return Err(super::validation(
                super::CheckpointErrorKind::InvalidPrimitive,
            ));
        }
        Ok(vertices)
    }
}

fn validate_vec2(value: Vec2Bits) -> Result<(), super::CheckpointValidationError> {
    super::require_finite(value.x_bits)?;
    super::require_finite(value.y_bits)
}

/// Declares whether primitive order is source-significant or explicitly canonicalized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DebugPrimitiveOrder {
    /// Preserve producer emission order exactly.
    SourceSignificant,
    /// Require stable semantic-key order within a declared unordered subset.
    Canonicalized,
}

/// One primitive with an explicit ordering declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebugPrimitiveRecord {
    ordering: DebugPrimitiveOrder,
    primitive: WireDebugPrimitive,
}

impl DebugPrimitiveRecord {
    /// Creates one primitive with an explicit ordering declaration.
    #[must_use]
    pub const fn new(ordering: DebugPrimitiveOrder, primitive: WireDebugPrimitive) -> Self {
        Self {
            ordering,
            primitive,
        }
    }

    /// Returns the explicit ordering declaration.
    #[must_use]
    pub const fn ordering(&self) -> DebugPrimitiveOrder {
        self.ordering
    }

    /// Returns the engine-neutral primitive.
    #[must_use]
    pub const fn primitive(&self) -> &WireDebugPrimitive {
        &self.primitive
    }

    /// Returns the primitive's stable semantic key.
    #[must_use]
    pub const fn key(&self) -> &DebugPrimitiveKey {
        self.primitive.metadata().key()
    }
}
