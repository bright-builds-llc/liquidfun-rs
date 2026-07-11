use std::collections::HashSet;

use serde::Deserialize;

mod shape_validation;
mod tree_validation;

use shape_validation::{validate_cache, validate_rejected_shape, validate_shape};
use tree_validation::validate_tree_command;

use super::{
    types::{
        CollisionCacheBits, CollisionClipPointBits, CollisionExpectedOutcome, CollisionFeatureBits,
        CollisionProbeCase, CollisionProbeDecodeError, CollisionProbeErrorKind,
        CollisionProbeHorizon, CollisionProbeInput, CollisionProbeOperation,
        CollisionProbeRequestKind, CollisionProbeRequestRecord, CollisionProbeScenario,
        CollisionProxyFingerprint, CollisionRejectionCategory, CollisionRejectionField,
        CollisionShapeDefinition, CollisionShapeKind, CollisionSupportPair, CollisionTreeCommand,
        validation,
    },
    witness_registry::CollisionWitnessFamily,
};
use crate::{
    FloatBits, HarnessLimits, ProtocolVersion, RecordLimit, RequestId, ScenarioId,
    ScenarioSchemaVersion, ScenarioSource, Sha256Hex, SweepBits, ToleranceProfileVersion,
    TraceSchemaVersion, TransformBits, Vec2Bits,
    codec::{BoundedString, BoundedVec, decode_jsonl},
    tolerance::CollectionPolicy,
};

const MAXIMUM_ID_BYTES: usize = 128;
const MAXIMUM_STRING_BYTES: usize = 4 * 1024;
const MAXIMUM_CASES: usize = 256;
const MAXIMUM_SHAPE_VERTICES: usize = 32;
const MAXIMUM_COMMANDS: usize = 128;
const MAXIMUM_AGGREGATE_ITEMS: usize = 2_048;

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
    witness_family: CollisionWitnessFamily,
    expected_outcome: CollisionExpectedOutcome,
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
        maybe_previous: Option<Vec2Bits>,
        maybe_next: Option<Vec2Bits>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProxyFingerprint {
    shape_kind: CollisionShapeKind,
    child_index: u32,
    radius_bits: FloatBits,
    vertices: BoundedVec<Vec2Bits, MAXIMUM_SHAPE_VERTICES>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCache {
    proxy_a: RawProxyFingerprint,
    proxy_b: RawProxyFingerprint,
    support_pairs: BoundedVec<CollisionSupportPair, 4>,
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
    let mut witness_families = HashSet::with_capacity(raw_cases.len());
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
        if raw_case.witness_family.expected_operation() != raw_case.operation
            || raw_case.witness_family.expects_rejection()
                != matches!(
                    raw_case.expected_outcome,
                    CollisionExpectedOutcome::Rejected { .. }
                )
        {
            return Err(validation(CollisionProbeErrorKind::WitnessFamilyMismatch));
        }
        witness_families.insert(raw_case.witness_family);
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
        let (input, item_count) = validate_input(
            raw_case.operation,
            raw_case.expected_outcome,
            raw_case.input,
        )?;
        aggregate = aggregate
            .checked_add(item_count)
            .ok_or_else(|| validation(CollisionProbeErrorKind::AggregateLimitExceeded))?;
        if aggregate > MAXIMUM_AGGREGATE_ITEMS {
            return Err(validation(CollisionProbeErrorKind::AggregateLimitExceeded));
        }
        cases.push(CollisionProbeCase {
            case_id: case_id.into_boxed_str(),
            witness_family: raw_case.witness_family,
            expected_outcome: raw_case.expected_outcome,
            operation: raw_case.operation,
            policy_path: policy_path.into_boxed_str(),
            horizon: raw_case.horizon,
            collection_policy: raw_case.collection_policy,
            input,
        });
    }
    if CollisionWitnessFamily::REQUIRED
        .iter()
        .any(|family| !witness_families.contains(family))
    {
        return Err(validation(CollisionProbeErrorKind::MissingWitnessFamily));
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
    expected_outcome: CollisionExpectedOutcome,
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
            let shape = match expected_outcome {
                CollisionExpectedOutcome::Accepted => {
                    let shape = validate_shape(shape)?;
                    validate_child(&shape, child_index)?;
                    shape
                }
                CollisionExpectedOutcome::Rejected { category, field }
                    if operation == CollisionProbeOperation::ShapeConstruction =>
                {
                    validate_rejected_shape(shape, child_index, category, field)?
                }
                CollisionExpectedOutcome::Rejected { .. } => {
                    return Err(validation(CollisionProbeErrorKind::OperationInputMismatch));
                }
            };
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
            if !matches!(expected_outcome, CollisionExpectedOutcome::Accepted) {
                return Err(validation(CollisionProbeErrorKind::OperationInputMismatch));
            }
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
            let item_count = 2 + maybe_cache
                .as_ref()
                .map_or(0, CollisionCacheBits::item_count);
            Ok((
                CollisionProbeInput::Pair {
                    shapes: shapes.into_boxed_slice(),
                    child_indices,
                    transforms,
                    use_radii,
                    maybe_cache,
                },
                item_count,
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
            if !matches!(expected_outcome, CollisionExpectedOutcome::Accepted) {
                return Err(validation(CollisionProbeErrorKind::OperationInputMismatch));
            }
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
            if !matches!(expected_outcome, CollisionExpectedOutcome::Accepted) {
                return Err(validation(CollisionProbeErrorKind::OperationInputMismatch));
            }
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
            if !matches!(expected_outcome, CollisionExpectedOutcome::Accepted) {
                return Err(validation(CollisionProbeErrorKind::OperationInputMismatch));
            }
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
            if !matches!(expected_outcome, CollisionExpectedOutcome::Accepted) {
                return Err(validation(CollisionProbeErrorKind::OperationInputMismatch));
            }
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
