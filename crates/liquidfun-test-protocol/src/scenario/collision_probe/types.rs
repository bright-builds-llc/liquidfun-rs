use serde::{Deserialize, Serialize};

use super::witness_registry::CollisionWitnessFamily;
use crate::{
    CodecError, FloatBits, ProtocolVersion, RequestId, ScenarioId, ScenarioSchemaVersion,
    ScenarioSource, Sha256Hex, SweepBits, ToleranceProfileVersion, TraceSchemaVersion,
    TransformBits, Vec2Bits, tolerance::CollectionPolicy,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionProbeErrorKind {
    NoCases,
    DuplicateCaseId,
    InvalidIdentifier,
    InvalidSource,
    InvalidGeometry,
    InvalidChildIndex,
    OperationInputMismatch,
    PolicyPathMismatch,
    HorizonMismatch,
    CollectionPolicyMismatch,
    AggregateLimitExceeded,
    DuplicateSetPayload,
    MissingWitnessFamily,
    WitnessFamilyMismatch,
}

#[derive(Debug, thiserror::Error)]
pub enum CollisionProbeDecodeError {
    #[error(transparent)]
    Codec(#[from] CodecError),
    #[error("collision probe validation failed: {0:?}")]
    Validation(CollisionProbeErrorKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollisionProbeOperation {
    ShapeConstruction,
    ShapeUnaryQuery,
    Distance,
    Overlap,
    Clip,
    Manifold,
    PairDispatch,
    FeatureTransition,
    TreeLifecycle,
    TreeQuery,
    TreeRay,
    TreeMetrics,
    BroadPhaseMoveTouch,
    BroadPhasePairs,
    BroadPhaseFilter,
    BroadPhaseRefilter,
    TimeOfImpact,
}

impl CollisionProbeOperation {
    pub const ALL: [Self; 17] = [
        Self::ShapeConstruction,
        Self::ShapeUnaryQuery,
        Self::Distance,
        Self::Overlap,
        Self::Clip,
        Self::Manifold,
        Self::PairDispatch,
        Self::FeatureTransition,
        Self::TreeLifecycle,
        Self::TreeQuery,
        Self::TreeRay,
        Self::TreeMetrics,
        Self::BroadPhaseMoveTouch,
        Self::BroadPhasePairs,
        Self::BroadPhaseFilter,
        Self::BroadPhaseRefilter,
        Self::TimeOfImpact,
    ];

    #[must_use]
    pub const fn policy_path(self) -> &'static str {
        match self {
            Self::ShapeConstruction => "collision.shape_construction.result",
            Self::ShapeUnaryQuery => "collision.shape_unary_query.result",
            Self::Distance => "collision.distance.result",
            Self::Overlap => "collision.overlap.result",
            Self::Clip => "collision.clip.result",
            Self::Manifold => "collision.manifold.result",
            Self::PairDispatch => "collision.pair_dispatch.result",
            Self::FeatureTransition => "collision.feature_transition.result",
            Self::TreeLifecycle => "collision.tree_lifecycle.result",
            Self::TreeQuery => "collision.tree_query.result",
            Self::TreeRay => "collision.tree_ray.result",
            Self::TreeMetrics => "collision.tree_metrics.result",
            Self::BroadPhaseMoveTouch => "collision.broad_phase_move_touch.result",
            Self::BroadPhasePairs => "collision.broad_phase_pairs.result",
            Self::BroadPhaseFilter => "collision.broad_phase_filter.result",
            Self::BroadPhaseRefilter => "collision.broad_phase_refilter.result",
            Self::TimeOfImpact => "collision.time_of_impact.result",
        }
    }

    #[must_use]
    pub const fn expected_horizon(self) -> CollisionProbeHorizon {
        match self {
            Self::Distance
            | Self::Manifold
            | Self::PairDispatch
            | Self::TreeLifecycle
            | Self::TreeQuery
            | Self::TreeRay
            | Self::TreeMetrics
            | Self::BroadPhaseMoveTouch
            | Self::BroadPhasePairs
            | Self::BroadPhaseFilter
            | Self::BroadPhaseRefilter
            | Self::TimeOfImpact => CollisionProbeHorizon::PhaseLocal,
            Self::ShapeConstruction
            | Self::ShapeUnaryQuery
            | Self::Overlap
            | Self::Clip
            | Self::FeatureTransition => CollisionProbeHorizon::Operation,
        }
    }

    #[must_use]
    pub const fn expected_collection_policy(self) -> CollectionPolicy {
        match self {
            Self::TreeQuery | Self::TreeRay => CollectionPolicy::Set,
            _ => CollectionPolicy::Ordered,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollisionRejectionCategory {
    NonFiniteValue,
    InvalidGeometry,
    InvalidChildIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollisionRejectionField {
    CircleCenter,
    CircleRadius,
    EdgeStart,
    EdgeEnd,
    EdgePrevious,
    EdgeNext,
    PolygonVertices,
    ChainVertices,
    ChildIndex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CollisionExpectedOutcome {
    Accepted,
    Rejected {
        category: CollisionRejectionCategory,
        field: CollisionRejectionField,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CollisionProbeHorizon {
    Operation,
    PhaseLocal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CollisionShapeDefinition {
    Circle {
        shape_id: Box<str>,
        center: Vec2Bits,
        radius_bits: FloatBits,
    },
    Edge {
        shape_id: Box<str>,
        start: Vec2Bits,
        end: Vec2Bits,
        maybe_previous: Option<Vec2Bits>,
        maybe_next: Option<Vec2Bits>,
    },
    Polygon {
        shape_id: Box<str>,
        vertices: Box<[Vec2Bits]>,
    },
    Chain {
        shape_id: Box<str>,
        vertices: Box<[Vec2Bits]>,
        closed: bool,
        maybe_previous: Option<Vec2Bits>,
        maybe_next: Option<Vec2Bits>,
    },
}

impl CollisionShapeDefinition {
    #[must_use]
    pub fn shape_id(&self) -> &str {
        match self {
            Self::Circle { shape_id, .. }
            | Self::Edge { shape_id, .. }
            | Self::Polygon { shape_id, .. }
            | Self::Chain { shape_id, .. } => shape_id,
        }
    }

    #[must_use]
    pub fn child_count(&self) -> usize {
        match self {
            Self::Circle { .. } | Self::Edge { .. } | Self::Polygon { .. } => 1,
            Self::Chain {
                vertices, closed, ..
            } => {
                if *closed {
                    vertices.len()
                } else {
                    vertices.len().saturating_sub(1)
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollisionShapeKind {
    Circle,
    Edge,
    Polygon,
    Chain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CollisionProxyFingerprint {
    pub(super) shape_kind: CollisionShapeKind,
    pub(super) child_index: u32,
    pub(super) radius_bits: FloatBits,
    pub(super) vertices: Box<[Vec2Bits]>,
}

impl CollisionProxyFingerprint {
    #[must_use]
    pub const fn shape_kind(&self) -> CollisionShapeKind {
        self.shape_kind
    }

    #[must_use]
    pub const fn child_index(&self) -> u32 {
        self.child_index
    }

    #[must_use]
    pub const fn radius_bits(&self) -> FloatBits {
        self.radius_bits
    }

    #[must_use]
    pub fn vertices(&self) -> &[Vec2Bits] {
        &self.vertices
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollisionSupportPair {
    pub index_a: u32,
    pub index_b: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CollisionCacheBits {
    pub(super) proxy_a: CollisionProxyFingerprint,
    pub(super) proxy_b: CollisionProxyFingerprint,
    pub(super) support_pairs: Box<[CollisionSupportPair]>,
    pub(super) metric_bits: FloatBits,
}

impl CollisionCacheBits {
    #[must_use]
    pub const fn proxy_a(&self) -> &CollisionProxyFingerprint {
        &self.proxy_a
    }

    #[must_use]
    pub const fn proxy_b(&self) -> &CollisionProxyFingerprint {
        &self.proxy_b
    }

    #[must_use]
    pub fn support_pairs(&self) -> &[CollisionSupportPair] {
        &self.support_pairs
    }

    #[must_use]
    pub const fn metric_bits(&self) -> FloatBits {
        self.metric_bits
    }

    pub(super) fn item_count(&self) -> usize {
        2 + self.proxy_a.vertices.len() + self.proxy_b.vertices.len() + self.support_pairs.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollisionFeatureBits {
    pub index_a: u8,
    pub index_b: u8,
    pub kind_a: CollisionFeatureKind,
    pub kind_b: CollisionFeatureKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollisionFeatureKind {
    Vertex,
    Face,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollisionClipPointBits {
    pub point: Vec2Bits,
    pub feature: CollisionFeatureBits,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CollisionTreeCommand {
    Create {
        payload_id: u32,
        lower: Vec2Bits,
        upper: Vec2Bits,
    },
    Move {
        payload_id: u32,
        lower: Vec2Bits,
        upper: Vec2Bits,
        displacement: Vec2Bits,
    },
    Touch {
        payload_id: u32,
    },
    Destroy {
        payload_id: u32,
    },
    Query {
        lower: Vec2Bits,
        upper: Vec2Bits,
    },
    Ray {
        start: Vec2Bits,
        end: Vec2Bits,
        max_fraction_bits: FloatBits,
    },
    Refilter {
        payload_id: u32,
        category_bits: u16,
        mask_bits: u16,
        group_index: i16,
    },
    UpdatePairs,
    Metrics,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CollisionProbeInput {
    Shape {
        shape: CollisionShapeDefinition,
        child_index: u32,
        transform: TransformBits,
        query_point: Vec2Bits,
    },
    Pair {
        shapes: Box<[CollisionShapeDefinition]>,
        child_indices: [u32; 2],
        transforms: [TransformBits; 2],
        use_radii: bool,
        maybe_cache: Option<CollisionCacheBits>,
    },
    Clip {
        points: [CollisionClipPointBits; 2],
        normal: Vec2Bits,
        offset_bits: FloatBits,
        vertex_index_a: u8,
    },
    Features {
        previous: Box<[CollisionFeatureBits]>,
        current: Box<[CollisionFeatureBits]>,
    },
    Tree {
        commands: Box<[CollisionTreeCommand]>,
    },
    TimeOfImpact {
        shapes: Box<[CollisionShapeDefinition]>,
        child_indices: [u32; 2],
        sweeps: [SweepBits; 2],
        t_max_bits: FloatBits,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CollisionProbeCase {
    pub(super) case_id: Box<str>,
    pub(super) witness_family: CollisionWitnessFamily,
    pub(super) expected_outcome: CollisionExpectedOutcome,
    pub(super) operation: CollisionProbeOperation,
    pub(super) policy_path: Box<str>,
    pub(super) horizon: CollisionProbeHorizon,
    pub(super) collection_policy: CollectionPolicy,
    pub(super) input: CollisionProbeInput,
}

impl CollisionProbeCase {
    #[must_use]
    pub fn case_id(&self) -> &str {
        &self.case_id
    }
    #[must_use]
    pub const fn witness_family(&self) -> CollisionWitnessFamily {
        self.witness_family
    }
    #[must_use]
    pub const fn expected_outcome(&self) -> CollisionExpectedOutcome {
        self.expected_outcome
    }
    #[must_use]
    pub const fn operation(&self) -> CollisionProbeOperation {
        self.operation
    }
    #[must_use]
    pub fn policy_path(&self) -> &str {
        &self.policy_path
    }
    #[must_use]
    pub const fn horizon(&self) -> CollisionProbeHorizon {
        self.horizon
    }
    #[must_use]
    pub const fn collection_policy(&self) -> CollectionPolicy {
        self.collection_policy
    }
    #[must_use]
    pub const fn input(&self) -> &CollisionProbeInput {
        &self.input
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CollisionProbeScenario {
    pub(super) scenario_id: ScenarioId,
    pub(super) source: ScenarioSource,
    pub(super) cases: Box<[CollisionProbeCase]>,
}

impl CollisionProbeScenario {
    #[must_use]
    pub const fn scenario_id(&self) -> &ScenarioId {
        &self.scenario_id
    }
    #[must_use]
    pub const fn source(&self) -> &ScenarioSource {
        &self.source
    }
    #[must_use]
    pub fn cases(&self) -> &[CollisionProbeCase] {
        &self.cases
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CollisionProbeRequestRecord {
    pub(super) protocol_version: ProtocolVersion,
    pub(super) record_kind: CollisionProbeRequestKind,
    pub(super) request_id: RequestId,
    pub(super) scenario_schema_version: ScenarioSchemaVersion,
    pub(super) requested_trace_schema_version: TraceSchemaVersion,
    pub(super) tolerance_profile_version: ToleranceProfileVersion,
    pub(super) tolerance_profile_sha256: Sha256Hex,
    pub(super) scenario: CollisionProbeScenario,
}

impl CollisionProbeRequestRecord {
    #[must_use]
    pub const fn request_id(&self) -> &RequestId {
        &self.request_id
    }
    #[must_use]
    pub const fn scenario(&self) -> &CollisionProbeScenario {
        &self.scenario
    }
    #[must_use]
    pub const fn tolerance_profile_sha256(&self) -> &Sha256Hex {
        &self.tolerance_profile_sha256
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum CollisionProbeRequestKind {
    CollisionProbeRequest,
}
pub(super) const fn validation(kind: CollisionProbeErrorKind) -> CollisionProbeDecodeError {
    CollisionProbeDecodeError::Validation(kind)
}
