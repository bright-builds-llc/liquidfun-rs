//! Closed owned primitive vocabulary shared by headless and visual consumers.

use crate::collision::Aabb;
use crate::math::{Transform, Vec2};
use crate::{BodyId, FixtureId, JointId, ParticleId, ParticleSystemId};

/// Stable semantic owner of one debug primitive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DebugOwnerKey {
    /// World-level semantic information.
    World,
    /// One rigid body.
    Body(BodyId),
    /// One fixture.
    Fixture(FixtureId),
    /// One joint.
    Joint(JointId),
    /// One source-ordered rigid contact occurrence.
    Contact {
        /// Oriented fixture pair.
        fixtures: [FixtureId; 2],
        /// Source occurrence within the current observation.
        occurrence: u32,
    },
    /// One particle system.
    ParticleSystem(ParticleSystemId),
    /// One stable particle.
    Particle(ParticleId),
    /// One source-ordered particle contact occurrence.
    ParticleContact {
        /// Owning particle system.
        system: ParticleSystemId,
        /// Stable particle pair in stored order.
        particles: [ParticleId; 2],
        /// Source occurrence within the current observation.
        occurrence: u32,
    },
}

/// Closed semantic overlay layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DebugLayer {
    /// Fixture geometry.
    Shapes,
    /// Joint connections.
    Joints,
    /// Rigid contact points.
    Contacts,
    /// Rigid contact normals.
    ContactNormals,
    /// Particle discs.
    Particles,
    /// Particle contact geometry.
    ParticleContacts,
    /// Tight current fixture-child AABBs.
    BroadPhase,
    /// Body center-of-mass frames.
    CentersOfMass,
    /// Bounded semantic labels.
    Labels,
}

/// Closed primitive variant tag used in stable keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DebugPrimitiveKind {
    /// One point marker.
    Point,
    /// One line segment.
    Segment,
    /// One open or closed polyline.
    Polyline,
    /// One circle.
    Circle,
    /// One transform-axis pair.
    TransformAxes,
    /// One axis-aligned box.
    Aabb,
    /// One directed arrow.
    Arrow,
    /// One inert semantic text label.
    Label,
}

/// Stable identity of one primitive within an owned semantic observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DebugPrimitiveKey {
    owner: DebugOwnerKey,
    layer: DebugLayer,
    kind: DebugPrimitiveKind,
    child: u32,
    ordinal: u32,
}

impl DebugPrimitiveKey {
    /// Creates a key from semantic owner, layer, primitive kind, child, and occurrence ordinal.
    #[must_use]
    pub const fn new(
        owner: DebugOwnerKey,
        layer: DebugLayer,
        kind: DebugPrimitiveKind,
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
    pub const fn owner(self) -> DebugOwnerKey {
        self.owner
    }

    /// Returns the named overlay layer.
    #[must_use]
    pub const fn layer(self) -> DebugLayer {
        self.layer
    }

    /// Returns the closed primitive kind.
    #[must_use]
    pub const fn kind(self) -> DebugPrimitiveKind {
        self.kind
    }

    /// Returns the semantic child coordinate.
    #[must_use]
    pub const fn child(self) -> u32 {
        self.child
    }

    /// Returns the source occurrence or declared canonical ordinal.
    #[must_use]
    pub const fn ordinal(self) -> u32 {
        self.ordinal
    }
}

/// Exact renderer-neutral RGBA color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DebugColor([u8; 4]);

impl DebugColor {
    /// Creates an exact color from red, green, blue, and alpha bytes.
    #[must_use]
    pub const fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self([red, green, blue, alpha])
    }

    /// Returns exact RGBA components.
    #[must_use]
    pub const fn components(self) -> [u8; 4] {
        self.0
    }
}

/// Stroke metadata in world-space meters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DebugStroke {
    color: DebugColor,
    width: f32,
}

impl DebugStroke {
    /// Creates a stroke. Width must be finite and non-negative.
    #[must_use]
    pub const fn new(color: DebugColor, width: f32) -> Option<Self> {
        if width.is_finite() && width >= 0.0 {
            Some(Self { color, width })
        } else {
            None
        }
    }

    /// Returns the exact color.
    #[must_use]
    pub const fn color(self) -> DebugColor {
        self.color
    }

    /// Returns world-space stroke width in meters.
    #[must_use]
    pub const fn width(self) -> f32 {
        self.width
    }
}

/// Optional fill metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DebugFill {
    color: DebugColor,
}

impl DebugFill {
    /// Creates a fill from an exact color.
    #[must_use]
    pub const fn new(color: DebugColor) -> Self {
        Self { color }
    }

    /// Returns the exact color.
    #[must_use]
    pub const fn color(self) -> DebugColor {
        self.color
    }
}

/// Common stable key and closed style metadata.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DebugPrimitiveMetadata {
    key: DebugPrimitiveKey,
    stroke: DebugStroke,
    maybe_fill: Option<DebugFill>,
}

impl DebugPrimitiveMetadata {
    /// Creates primitive metadata.
    #[must_use]
    pub const fn new(
        key: DebugPrimitiveKey,
        stroke: DebugStroke,
        maybe_fill: Option<DebugFill>,
    ) -> Self {
        Self {
            key,
            stroke,
            maybe_fill,
        }
    }

    /// Returns the stable primitive key.
    #[must_use]
    pub const fn key(self) -> DebugPrimitiveKey {
        self.key
    }

    /// Returns stroke metadata.
    #[must_use]
    pub const fn stroke(self) -> DebugStroke {
        self.stroke
    }

    /// Returns optional fill metadata.
    #[must_use]
    pub const fn maybe_fill(self) -> Option<DebugFill> {
        self.maybe_fill
    }
}

/// Closed renderer-neutral primitive vocabulary.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum DebugPrimitive {
    /// One point marker.
    Point {
        /// Stable semantic metadata.
        metadata: DebugPrimitiveMetadata,
        /// World-space point in meters.
        position: Vec2,
        /// Marker radius in meters.
        radius: f32,
    },
    /// One segment.
    Segment {
        /// Stable semantic metadata.
        metadata: DebugPrimitiveMetadata,
        /// World-space start.
        start: Vec2,
        /// World-space end.
        end: Vec2,
    },
    /// One source-ordered open or closed polyline.
    Polyline {
        /// Stable semantic metadata.
        metadata: DebugPrimitiveMetadata,
        /// World-space vertices.
        vertices: Vec<Vec2>,
        /// Whether renderers close the final edge.
        closed: bool,
    },
    /// One circle.
    Circle {
        /// Stable semantic metadata.
        metadata: DebugPrimitiveMetadata,
        /// World-space center.
        center: Vec2,
        /// Radius in meters.
        radius: f32,
    },
    /// One coordinate frame.
    TransformAxes {
        /// Stable semantic metadata.
        metadata: DebugPrimitiveMetadata,
        /// World-space transform.
        transform: Transform,
        /// Axis length in meters.
        scale: f32,
    },
    /// One tight current AABB.
    Aabb {
        /// Stable semantic metadata.
        metadata: DebugPrimitiveMetadata,
        /// World-space bounds.
        bounds: Aabb,
    },
    /// One directed arrow.
    Arrow {
        /// Stable semantic metadata.
        metadata: DebugPrimitiveMetadata,
        /// World-space start.
        start: Vec2,
        /// World-space end.
        end: Vec2,
    },
    /// One inert semantic label without markup or renderer commands.
    Label {
        /// Stable semantic metadata.
        metadata: DebugPrimitiveMetadata,
        /// World-space anchor.
        position: Vec2,
        /// Bounded plain text.
        text: String,
    },
}

impl DebugPrimitive {
    /// Returns common semantic metadata.
    #[must_use]
    pub const fn metadata(&self) -> DebugPrimitiveMetadata {
        match self {
            Self::Point { metadata, .. }
            | Self::Segment { metadata, .. }
            | Self::Polyline { metadata, .. }
            | Self::Circle { metadata, .. }
            | Self::TransformAxes { metadata, .. }
            | Self::Aabb { metadata, .. }
            | Self::Arrow { metadata, .. }
            | Self::Label { metadata, .. } => *metadata,
        }
    }

    /// Returns the stable primitive key.
    #[must_use]
    pub const fn key(&self) -> DebugPrimitiveKey {
        self.metadata().key()
    }

    /// Returns the named layer.
    #[must_use]
    pub const fn layer(&self) -> DebugLayer {
        self.key().layer()
    }
}
