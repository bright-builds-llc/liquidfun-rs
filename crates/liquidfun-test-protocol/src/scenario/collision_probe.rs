#![allow(
    missing_docs,
    reason = "closed private-harness wire variants are self-describing"
)]

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::{ScenarioSource, SweepBits, TransformBits, Vec2Bits};
use crate::{
    CodecError, FloatBits, HarnessLimits, ProtocolVersion, RecordLimit, RequestId, ScenarioId,
    ScenarioSchemaVersion, Sha256Hex, ToleranceProfileVersion, TraceSchemaVersion,
    codec::{BoundedString, BoundedVec, decode_jsonl},
    tolerance::CollectionPolicy,
};

const MAXIMUM_ID_BYTES: usize = 128;
const MAXIMUM_STRING_BYTES: usize = 4 * 1024;
const MAXIMUM_CASES: usize = 256;
const MAXIMUM_SHAPE_VERTICES: usize = 32;
const MAXIMUM_COMMANDS: usize = 128;
const MAXIMUM_RESULT_FIELDS: usize = 128;
const MAXIMUM_AGGREGATE_ITEMS: usize = 2_048;

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
                    vertices.len() - 1
                }
            }
        }
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
    support_pairs: Box<[CollisionSupportPair]>,
    metric_bits: FloatBits,
}

impl CollisionCacheBits {
    #[must_use]
    pub fn support_pairs(&self) -> &[CollisionSupportPair] {
        &self.support_pairs
    }

    #[must_use]
    pub const fn metric_bits(&self) -> FloatBits {
        self.metric_bits
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
    case_id: Box<str>,
    operation: CollisionProbeOperation,
    policy_path: Box<str>,
    horizon: CollisionProbeHorizon,
    collection_policy: CollectionPolicy,
    input: CollisionProbeInput,
}

impl CollisionProbeCase {
    #[must_use]
    pub fn case_id(&self) -> &str {
        &self.case_id
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
    scenario_id: ScenarioId,
    source: ScenarioSource,
    cases: Box<[CollisionProbeCase]>,
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
    protocol_version: ProtocolVersion,
    record_kind: CollisionProbeRequestKind,
    request_id: RequestId,
    scenario_schema_version: ScenarioSchemaVersion,
    requested_trace_schema_version: TraceSchemaVersion,
    tolerance_profile_version: ToleranceProfileVersion,
    tolerance_profile_sha256: Sha256Hex,
    scenario: CollisionProbeScenario,
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
enum CollisionProbeRequestKind {
    CollisionProbeRequest,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRequest {
    protocol_version: ProtocolVersion,
    record_kind: CollisionProbeRequestKind,
    request_id: BoundedString<MAXIMUM_ID_BYTES>,
    scenario_schema_version: ScenarioSchemaVersion,
    requested_trace_schema_version: TraceSchemaVersion,
    tolerance_profile_version: ToleranceProfileVersion,
    tolerance_profile_sha256: Sha256Hex,
    scenario: RawScenario,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawScenario {
    scenario_id: BoundedString<MAXIMUM_ID_BYTES>,
    source: RawSource,
    cases: BoundedVec<RawCase, MAXIMUM_CASES>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RawSource {
    Named {
        name: BoundedString<MAXIMUM_STRING_BYTES>,
    },
    Seeded {
        generator_id: BoundedString<MAXIMUM_STRING_BYTES>,
        generator_version: u32,
        seed: u64,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCase {
    case_id: BoundedString<MAXIMUM_ID_BYTES>,
    operation: CollisionProbeOperation,
    policy_path: BoundedString<MAXIMUM_STRING_BYTES>,
    horizon: CollisionProbeHorizon,
    collection_policy: CollectionPolicy,
    input: RawInput,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RawShape {
    Circle {
        shape_id: BoundedString<MAXIMUM_ID_BYTES>,
        center: Vec2Bits,
        radius_bits: FloatBits,
    },
    Edge {
        shape_id: BoundedString<MAXIMUM_ID_BYTES>,
        start: Vec2Bits,
        end: Vec2Bits,
        maybe_previous: Option<Vec2Bits>,
        maybe_next: Option<Vec2Bits>,
    },
    Polygon {
        shape_id: BoundedString<MAXIMUM_ID_BYTES>,
        vertices: BoundedVec<Vec2Bits, MAXIMUM_SHAPE_VERTICES>,
    },
    Chain {
        shape_id: BoundedString<MAXIMUM_ID_BYTES>,
        vertices: BoundedVec<Vec2Bits, MAXIMUM_SHAPE_VERTICES>,
        closed: bool,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCache {
    support_pairs: BoundedVec<CollisionSupportPair, 3>,
    metric_bits: FloatBits,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RawTreeCommand {
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

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RawInput {
    Shape {
        shape: RawShape,
        child_index: u32,
        transform: TransformBits,
        query_point: Vec2Bits,
    },
    Pair {
        shapes: BoundedVec<RawShape, 2>,
        child_indices: [u32; 2],
        transforms: [TransformBits; 2],
        use_radii: bool,
        maybe_cache: Option<RawCache>,
    },
    Clip {
        points: [CollisionClipPointBits; 2],
        normal: Vec2Bits,
        offset_bits: FloatBits,
        vertex_index_a: u8,
    },
    Features {
        previous: BoundedVec<CollisionFeatureBits, 2>,
        current: BoundedVec<CollisionFeatureBits, 2>,
    },
    Tree {
        commands: BoundedVec<RawTreeCommand, MAXIMUM_COMMANDS>,
    },
    TimeOfImpact {
        shapes: BoundedVec<RawShape, 2>,
        child_indices: [u32; 2],
        sweeps: [SweepBits; 2],
        t_max_bits: FloatBits,
    },
}

/// Decodes one newline-complete bounded collision-probe request.
///
/// # Errors
///
/// Returns [`CollisionProbeDecodeError`] for framing, closed-field, geometry,
/// identity, topology, operation, policy, horizon, or resource-limit failures.
pub fn decode_collision_probe_request_jsonl(
    bytes: &[u8],
    limits: &HarnessLimits,
) -> Result<CollisionProbeRequestRecord, CollisionProbeDecodeError> {
    let raw = decode_jsonl::<RawRequest>(bytes, limits, RecordLimit::Input)?;
    validate_request(raw)
}

fn validate_request(
    raw: RawRequest,
) -> Result<CollisionProbeRequestRecord, CollisionProbeDecodeError> {
    let request_id = RequestId::new(raw.request_id.into_string())
        .map_err(|_| validation(CollisionProbeErrorKind::InvalidIdentifier))?;
    let scenario_id = ScenarioId::new(raw.scenario.scenario_id.into_string())
        .map_err(|_| validation(CollisionProbeErrorKind::InvalidIdentifier))?;
    let source = validate_source(raw.scenario.source)?;
    let raw_cases = raw.scenario.cases.into_vec();
    if raw_cases.is_empty() {
        return Err(validation(CollisionProbeErrorKind::NoCases));
    }
    let mut ids = HashSet::with_capacity(raw_cases.len());
    let mut cases = Vec::with_capacity(raw_cases.len());
    let mut aggregate = 0_usize;
    for raw_case in raw_cases {
        let case_id = raw_case.case_id.into_string();
        let policy_path = raw_case.policy_path.into_string();
        ScenarioId::new(case_id.clone())
            .map_err(|_| validation(CollisionProbeErrorKind::InvalidIdentifier))?;
        if !ids.insert(case_id.clone()) {
            return Err(validation(CollisionProbeErrorKind::DuplicateCaseId));
        }
        if policy_path != raw_case.operation.policy_path() {
            return Err(validation(CollisionProbeErrorKind::PolicyPathMismatch));
        }
        if raw_case.horizon != raw_case.operation.expected_horizon() {
            return Err(validation(CollisionProbeErrorKind::HorizonMismatch));
        }
        if raw_case.collection_policy != raw_case.operation.expected_collection_policy() {
            return Err(validation(
                CollisionProbeErrorKind::CollectionPolicyMismatch,
            ));
        }
        let (input, item_count) = validate_input(raw_case.operation, raw_case.input)?;
        aggregate = aggregate
            .checked_add(item_count)
            .ok_or_else(|| validation(CollisionProbeErrorKind::AggregateLimitExceeded))?;
        if aggregate > MAXIMUM_AGGREGATE_ITEMS {
            return Err(validation(CollisionProbeErrorKind::AggregateLimitExceeded));
        }
        cases.push(CollisionProbeCase {
            case_id: case_id.into_boxed_str(),
            operation: raw_case.operation,
            policy_path: policy_path.into_boxed_str(),
            horizon: raw_case.horizon,
            collection_policy: raw_case.collection_policy,
            input,
        });
    }
    Ok(CollisionProbeRequestRecord {
        protocol_version: raw.protocol_version,
        record_kind: raw.record_kind,
        request_id,
        scenario_schema_version: raw.scenario_schema_version,
        requested_trace_schema_version: raw.requested_trace_schema_version,
        tolerance_profile_version: raw.tolerance_profile_version,
        tolerance_profile_sha256: raw.tolerance_profile_sha256,
        scenario: CollisionProbeScenario {
            scenario_id,
            source,
            cases: cases.into_boxed_slice(),
        },
    })
}

fn validate_source(raw: RawSource) -> Result<ScenarioSource, CollisionProbeDecodeError> {
    match raw {
        RawSource::Named { name } => {
            let name = name.into_string();
            if name.trim().is_empty() {
                return Err(validation(CollisionProbeErrorKind::InvalidSource));
            }
            Ok(ScenarioSource::Named {
                name: name.into_boxed_str(),
            })
        }
        RawSource::Seeded {
            generator_id,
            generator_version,
            seed,
        } => {
            let generator_id = generator_id.into_string();
            if generator_id.trim().is_empty() || generator_version == 0 {
                return Err(validation(CollisionProbeErrorKind::InvalidSource));
            }
            Ok(ScenarioSource::Seeded {
                generator_id: generator_id.into_boxed_str(),
                generator_version,
                seed,
            })
        }
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "one exhaustive match makes the closed operation/input registry auditable"
)]
fn validate_input(
    operation: CollisionProbeOperation,
    raw: RawInput,
) -> Result<(CollisionProbeInput, usize), CollisionProbeDecodeError> {
    match (operation, raw) {
        (
            CollisionProbeOperation::ShapeConstruction | CollisionProbeOperation::ShapeUnaryQuery,
            RawInput::Shape {
                shape,
                child_index,
                transform,
                query_point,
            },
        ) => {
            validate_transform(transform)?;
            validate_vec2(query_point)?;
            let shape = validate_shape(shape)?;
            validate_child(&shape, child_index)?;
            Ok((
                CollisionProbeInput::Shape {
                    shape,
                    child_index,
                    transform,
                    query_point,
                },
                1,
            ))
        }
        (
            CollisionProbeOperation::Distance
            | CollisionProbeOperation::Overlap
            | CollisionProbeOperation::Manifold
            | CollisionProbeOperation::PairDispatch,
            RawInput::Pair {
                shapes,
                child_indices,
                transforms,
                use_radii,
                maybe_cache,
            },
        ) => {
            let shapes = shapes.into_vec();
            if shapes.len() != 2 {
                return Err(validation(CollisionProbeErrorKind::OperationInputMismatch));
            }
            let shapes = shapes
                .into_iter()
                .map(validate_shape)
                .collect::<Result<Vec<_>, _>>()?;
            for ((shape, child), transform) in shapes.iter().zip(child_indices).zip(transforms) {
                validate_child(shape, child)?;
                validate_transform(transform)?;
            }
            let maybe_cache = maybe_cache.map(validate_cache).transpose()?;
            Ok((
                CollisionProbeInput::Pair {
                    shapes: shapes.into_boxed_slice(),
                    child_indices,
                    transforms,
                    use_radii,
                    maybe_cache,
                },
                2,
            ))
        }
        (
            CollisionProbeOperation::Clip,
            RawInput::Clip {
                points,
                normal,
                offset_bits,
                vertex_index_a,
            },
        ) => {
            for point in points {
                validate_vec2(point.point)?;
            }
            validate_vec2(normal)?;
            validate_finite(offset_bits)?;
            Ok((
                CollisionProbeInput::Clip {
                    points,
                    normal,
                    offset_bits,
                    vertex_index_a,
                },
                2,
            ))
        }
        (CollisionProbeOperation::FeatureTransition, RawInput::Features { previous, current }) => {
            let previous = previous.into_vec().into_boxed_slice();
            let current = current.into_vec().into_boxed_slice();
            let count = previous.len() + current.len();
            Ok((CollisionProbeInput::Features { previous, current }, count))
        }
        (
            CollisionProbeOperation::TreeLifecycle
            | CollisionProbeOperation::TreeQuery
            | CollisionProbeOperation::TreeRay
            | CollisionProbeOperation::TreeMetrics
            | CollisionProbeOperation::BroadPhaseMoveTouch
            | CollisionProbeOperation::BroadPhasePairs
            | CollisionProbeOperation::BroadPhaseFilter
            | CollisionProbeOperation::BroadPhaseRefilter,
            RawInput::Tree { commands },
        ) => {
            let commands = commands
                .into_vec()
                .into_iter()
                .map(validate_tree_command)
                .collect::<Result<Vec<_>, _>>()?;
            if commands.is_empty() {
                return Err(validation(CollisionProbeErrorKind::OperationInputMismatch));
            }
            let count = commands.len();
            Ok((
                CollisionProbeInput::Tree {
                    commands: commands.into_boxed_slice(),
                },
                count,
            ))
        }
        (
            CollisionProbeOperation::TimeOfImpact,
            RawInput::TimeOfImpact {
                shapes,
                child_indices,
                sweeps,
                t_max_bits,
            },
        ) => {
            let shapes = shapes.into_vec();
            if shapes.len() != 2 {
                return Err(validation(CollisionProbeErrorKind::OperationInputMismatch));
            }
            let shapes = shapes
                .into_iter()
                .map(validate_shape)
                .collect::<Result<Vec<_>, _>>()?;
            for (shape, child) in shapes.iter().zip(child_indices) {
                validate_child(shape, child)?;
            }
            for sweep in sweeps {
                validate_sweep(sweep)?;
            }
            let t_max = t_max_bits.to_f32();
            if !(0.0..=1.0).contains(&t_max) {
                return Err(validation(CollisionProbeErrorKind::InvalidGeometry));
            }
            Ok((
                CollisionProbeInput::TimeOfImpact {
                    shapes: shapes.into_boxed_slice(),
                    child_indices,
                    sweeps,
                    t_max_bits,
                },
                2,
            ))
        }
        _ => Err(validation(CollisionProbeErrorKind::OperationInputMismatch)),
    }
}

fn validate_shape(raw: RawShape) -> Result<CollisionShapeDefinition, CollisionProbeDecodeError> {
    let shape = match raw {
        RawShape::Circle {
            shape_id,
            center,
            radius_bits,
        } => {
            validate_vec2(center)?;
            validate_finite(radius_bits)?;
            if radius_bits.to_f32() < 0.0 {
                return Err(validation(CollisionProbeErrorKind::InvalidGeometry));
            }
            CollisionShapeDefinition::Circle {
                shape_id: validate_id(shape_id)?,
                center,
                radius_bits,
            }
        }
        RawShape::Edge {
            shape_id,
            start,
            end,
            maybe_previous,
            maybe_next,
        } => {
            validate_vec2(start)?;
            validate_vec2(end)?;
            if start == end {
                return Err(validation(CollisionProbeErrorKind::InvalidGeometry));
            }
            if let Some(point) = maybe_previous {
                validate_vec2(point)?;
            }
            if let Some(point) = maybe_next {
                validate_vec2(point)?;
            }
            CollisionShapeDefinition::Edge {
                shape_id: validate_id(shape_id)?,
                start,
                end,
                maybe_previous,
                maybe_next,
            }
        }
        RawShape::Polygon { shape_id, vertices } => {
            let vertices = vertices.into_vec();
            if !(3..=8).contains(&vertices.len()) {
                return Err(validation(CollisionProbeErrorKind::InvalidGeometry));
            }
            for vertex in &vertices {
                validate_vec2(*vertex)?;
            }
            CollisionShapeDefinition::Polygon {
                shape_id: validate_id(shape_id)?,
                vertices: vertices.into_boxed_slice(),
            }
        }
        RawShape::Chain {
            shape_id,
            vertices,
            closed,
        } => {
            let vertices = vertices.into_vec();
            let minimum = if closed { 3 } else { 2 };
            if vertices.len() < minimum {
                return Err(validation(CollisionProbeErrorKind::InvalidGeometry));
            }
            for vertex in &vertices {
                validate_vec2(*vertex)?;
            }
            CollisionShapeDefinition::Chain {
                shape_id: validate_id(shape_id)?,
                vertices: vertices.into_boxed_slice(),
                closed,
            }
        }
    };
    Ok(shape)
}

fn validate_cache(raw: RawCache) -> Result<CollisionCacheBits, CollisionProbeDecodeError> {
    validate_finite(raw.metric_bits)?;
    Ok(CollisionCacheBits {
        support_pairs: raw.support_pairs.into_vec().into_boxed_slice(),
        metric_bits: raw.metric_bits,
    })
}

fn validate_tree_command(
    raw: RawTreeCommand,
) -> Result<CollisionTreeCommand, CollisionProbeDecodeError> {
    let command = match raw {
        RawTreeCommand::Create {
            payload_id,
            lower,
            upper,
        } => {
            validate_aabb(lower, upper)?;
            CollisionTreeCommand::Create {
                payload_id,
                lower,
                upper,
            }
        }
        RawTreeCommand::Move {
            payload_id,
            lower,
            upper,
            displacement,
        } => {
            validate_aabb(lower, upper)?;
            validate_vec2(displacement)?;
            CollisionTreeCommand::Move {
                payload_id,
                lower,
                upper,
                displacement,
            }
        }
        RawTreeCommand::Touch { payload_id } => CollisionTreeCommand::Touch { payload_id },
        RawTreeCommand::Destroy { payload_id } => CollisionTreeCommand::Destroy { payload_id },
        RawTreeCommand::Query { lower, upper } => {
            validate_aabb(lower, upper)?;
            CollisionTreeCommand::Query { lower, upper }
        }
        RawTreeCommand::Ray {
            start,
            end,
            max_fraction_bits,
        } => {
            validate_vec2(start)?;
            validate_vec2(end)?;
            let fraction = max_fraction_bits.to_f32();
            if !(0.0..=1.0).contains(&fraction) {
                return Err(validation(CollisionProbeErrorKind::InvalidGeometry));
            }
            CollisionTreeCommand::Ray {
                start,
                end,
                max_fraction_bits,
            }
        }
        RawTreeCommand::Refilter {
            payload_id,
            category_bits,
            mask_bits,
            group_index,
        } => CollisionTreeCommand::Refilter {
            payload_id,
            category_bits,
            mask_bits,
            group_index,
        },
        RawTreeCommand::UpdatePairs => CollisionTreeCommand::UpdatePairs,
        RawTreeCommand::Metrics => CollisionTreeCommand::Metrics,
    };
    Ok(command)
}

fn validate_aabb(lower: Vec2Bits, upper: Vec2Bits) -> Result<(), CollisionProbeDecodeError> {
    validate_vec2(lower)?;
    validate_vec2(upper)?;
    if lower.x_bits.to_f32() > upper.x_bits.to_f32()
        || lower.y_bits.to_f32() > upper.y_bits.to_f32()
    {
        return Err(validation(CollisionProbeErrorKind::InvalidGeometry));
    }
    Ok(())
}

fn validate_sweep(sweep: SweepBits) -> Result<(), CollisionProbeDecodeError> {
    validate_vec2(sweep.local_center)?;
    validate_vec2(sweep.initial_center)?;
    validate_vec2(sweep.center)?;
    validate_finite(sweep.initial_angle_bits)?;
    validate_finite(sweep.angle_bits)?;
    let fraction = sweep.initial_fraction_bits.to_f32();
    if !(0.0..=1.0).contains(&fraction) {
        return Err(validation(CollisionProbeErrorKind::InvalidGeometry));
    }
    Ok(())
}

fn validate_transform(transform: TransformBits) -> Result<(), CollisionProbeDecodeError> {
    validate_vec2(transform.position)?;
    validate_finite(transform.angle_bits)
}

fn validate_vec2(value: Vec2Bits) -> Result<(), CollisionProbeDecodeError> {
    validate_finite(value.x_bits)?;
    validate_finite(value.y_bits)
}

fn validate_finite(value: FloatBits) -> Result<(), CollisionProbeDecodeError> {
    if !value.to_f32().is_finite() {
        return Err(validation(CollisionProbeErrorKind::InvalidGeometry));
    }
    Ok(())
}

fn validate_child(
    shape: &CollisionShapeDefinition,
    child: u32,
) -> Result<(), CollisionProbeDecodeError> {
    if usize::try_from(child)
        .ok()
        .is_none_or(|child| child >= shape.child_count())
    {
        return Err(validation(CollisionProbeErrorKind::InvalidChildIndex));
    }
    Ok(())
}

fn validate_id(
    value: BoundedString<MAXIMUM_ID_BYTES>,
) -> Result<Box<str>, CollisionProbeDecodeError> {
    let value = value.into_string();
    ScenarioId::new(value.clone())
        .map_err(|_| validation(CollisionProbeErrorKind::InvalidIdentifier))?;
    Ok(value.into_boxed_str())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CollisionProbeNumericValue {
    field: Box<str>,
    bits: FloatBits,
}

impl CollisionProbeNumericValue {
    #[must_use]
    pub fn new(field: impl Into<Box<str>>, bits: FloatBits) -> Self {
        Self {
            field: field.into(),
            bits,
        }
    }
    #[must_use]
    pub fn field(&self) -> &str {
        &self.field
    }
    #[must_use]
    pub const fn bits(&self) -> FloatBits {
        self.bits
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CollisionProbeDiscreteValue {
    field: Box<str>,
    value: Box<str>,
}

impl CollisionProbeDiscreteValue {
    #[must_use]
    pub fn new(field: impl Into<Box<str>>, value: impl Into<Box<str>>) -> Self {
        Self {
            field: field.into(),
            value: value.into(),
        }
    }
    #[must_use]
    pub fn field(&self) -> &str {
        &self.field
    }
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CollisionProbeResult {
    case_id: Box<str>,
    operation: CollisionProbeOperation,
    policy_path: Box<str>,
    horizon: CollisionProbeHorizon,
    collection_policy: CollectionPolicy,
    numeric: Box<[CollisionProbeNumericValue]>,
    discrete: Box<[CollisionProbeDiscreteValue]>,
    payload_ids: Box<[u32]>,
}

impl CollisionProbeResult {
    /// Creates one bounded result aligned to the operation's closed metadata.
    ///
    /// # Errors
    ///
    /// Returns [`CollisionProbeDecodeError`] when aggregate fields exceed the
    /// reviewed limit or a set-like payload collection contains duplicates.
    pub fn new(
        case_id: impl Into<Box<str>>,
        operation: CollisionProbeOperation,
        numeric: Vec<CollisionProbeNumericValue>,
        discrete: Vec<CollisionProbeDiscreteValue>,
        payload_ids: Vec<u32>,
    ) -> Result<Self, CollisionProbeDecodeError> {
        if numeric.len() + discrete.len() + payload_ids.len() > MAXIMUM_RESULT_FIELDS {
            return Err(validation(CollisionProbeErrorKind::AggregateLimitExceeded));
        }
        if operation.expected_collection_policy() == CollectionPolicy::Set {
            let unique: HashSet<_> = payload_ids.iter().copied().collect();
            if unique.len() != payload_ids.len() {
                return Err(validation(CollisionProbeErrorKind::DuplicateSetPayload));
            }
        }
        Ok(Self {
            case_id: case_id.into(),
            operation,
            policy_path: operation.policy_path().into(),
            horizon: operation.expected_horizon(),
            collection_policy: operation.expected_collection_policy(),
            numeric: numeric.into_boxed_slice(),
            discrete: discrete.into_boxed_slice(),
            payload_ids: payload_ids.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn case_id(&self) -> &str {
        &self.case_id
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
    pub fn numeric(&self) -> &[CollisionProbeNumericValue] {
        &self.numeric
    }
    #[must_use]
    pub fn discrete(&self) -> &[CollisionProbeDiscreteValue] {
        &self.discrete
    }
    #[must_use]
    pub fn payload_ids(&self) -> &[u32] {
        &self.payload_ids
    }
}

const fn validation(kind: CollisionProbeErrorKind) -> CollisionProbeDecodeError {
    CollisionProbeDecodeError::Validation(kind)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode_jsonl;

    const REQUEST: &[u8] =
        include_bytes!("../../../../protocol/fixtures/accepted/collision-probe-request.jsonl");

    #[test]
    fn collision_probe_operation_registry_is_closed_and_complete() {
        // Arrange
        let limits = HarnessLimits::phase2_default_v1();

        // Act
        let request = decode_collision_probe_request_jsonl(REQUEST, &limits)
            .expect("checked-in collision request should decode");
        let operations: HashSet<_> = request
            .scenario()
            .cases()
            .iter()
            .map(CollisionProbeCase::operation)
            .collect();

        // Assert
        assert_eq!(operations, HashSet::from(CollisionProbeOperation::ALL));
        assert_eq!(
            encode_jsonl(&request, &limits, RecordLimit::Input)
                .expect("validated request should encode"),
            REQUEST
        );
    }

    #[test]
    fn collision_probe_rejects_collection_and_horizon_mismatch() {
        // Arrange
        let json = br#"{"protocol_version":1,"record_kind":"collision_probe_request","request_id":"r","scenario_schema_version":1,"requested_trace_schema_version":1,"tolerance_profile_version":1,"tolerance_profile_sha256":"0000000000000000000000000000000000000000000000000000000000000000","scenario":{"scenario_id":"s","source":{"kind":"named","name":"n"},"cases":[{"case_id":"query","operation":"tree_query","policy_path":"collision.tree_query.result","horizon":{"kind":"operation"},"collection_policy":"ordered","input":{"kind":"tree","commands":[{"kind":"query","lower":{"x_bits":0,"y_bits":0},"upper":{"x_bits":1065353216,"y_bits":1065353216}}]}}]}}
"#;

        // Act
        let error = decode_collision_probe_request_jsonl(json, &HarnessLimits::phase2_default_v1())
            .expect_err("mismatched closed metadata should fail");

        // Assert
        assert!(matches!(
            error,
            CollisionProbeDecodeError::Validation(
                CollisionProbeErrorKind::HorizonMismatch
                    | CollisionProbeErrorKind::CollectionPolicyMismatch
            )
        ));
    }

    #[test]
    fn collision_probe_rejects_unknown_duplicate_missing_policy_and_invalid_child() {
        // Arrange
        let limits = HarnessLimits::phase2_default_v1();
        let text = std::str::from_utf8(REQUEST).expect("fixture should be UTF-8");
        let unknown = text.replacen(
            "\"request_id\":\"phase-05-collision-probe-request\"",
            "\"request_id\":\"phase-05-collision-probe-request\",\"unknown\":true",
            1,
        );
        let duplicate = text.replacen(
            "\"request_id\":\"phase-05-collision-probe-request\"",
            "\"request_id\":\"phase-05-collision-probe-request\",\"request_id\":\"duplicate\"",
            1,
        );
        let missing_policy = text.replacen(
            "\"policy_path\":\"collision.shape_construction.result\",",
            "",
            1,
        );
        let invalid_child = text.replacen("\"child_index\":0", "\"child_index\":1", 1);

        // Act
        let errors = [unknown, duplicate, missing_policy, invalid_child].map(|record| {
            decode_collision_probe_request_jsonl(record.as_bytes(), &limits)
                .expect_err("invalid collision request should fail")
        });

        // Assert
        assert!(matches!(errors[0], CollisionProbeDecodeError::Codec(_)));
        assert!(matches!(errors[1], CollisionProbeDecodeError::Codec(_)));
        assert!(matches!(errors[2], CollisionProbeDecodeError::Codec(_)));
        assert!(matches!(
            errors[3],
            CollisionProbeDecodeError::Validation(CollisionProbeErrorKind::InvalidChildIndex)
        ));
    }
}
