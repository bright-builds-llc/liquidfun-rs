use std::collections::BTreeMap;

use liquidfun::{
    collision::{
        Aabb, BroadPhase, ChainShape, ChildIndex, CircleShape, DistanceResult, DynamicTree,
        EdgeShape, FeatureKind, FilterData, PairOrientation, PolygonShape, QueryControl,
        RayCastControl, RayCastInput, Shape,
        differential::{
            DistanceCacheSeed, DistanceCacheSeedPair, DistanceCacheSeedRejection,
            DistanceCacheSeedReset, DistanceProxyFingerprint, DistanceProxyKind,
            DistanceProxyVertexBits, distance_diagnostic,
        },
    },
    math::{Sweep, Transform, Vec2},
};
use liquidfun_test_protocol::{
    CollisionCacheBits, CollisionFeatureBits, CollisionFeatureKind, CollisionProbeCase,
    CollisionProbeDiscreteValue, CollisionProbeNumericValue, CollisionProbeOperation,
    CollisionProxyFingerprint, CollisionRejectionCategory, CollisionRejectionField,
    CollisionShapeDefinition, CollisionShapeKind, CollisionTreeCommand, CollisionWitnessFamily,
    FloatBits, SweepBits, TransformBits, Vec2Bits,
};

use super::CollisionProbeExecutionError;

pub(super) fn execute_tree(
    case: &CollisionProbeCase,
    commands: &[CollisionTreeCommand],
    numeric: &mut Vec<CollisionProbeNumericValue>,
    discrete: &mut Vec<CollisionProbeDiscreteValue>,
    payload_ids: &mut Vec<u32>,
) -> Result<(), CollisionProbeExecutionError> {
    let mut tree = native(case, DynamicTree::<u32>::new())?;
    let mut proxies = BTreeMap::new();
    for command in commands {
        match *command {
            CollisionTreeCommand::Create {
                payload_id,
                lower,
                upper,
            } => {
                let proxy = native(
                    case,
                    tree.create_proxy(aabb(case, lower, upper)?, payload_id),
                )?;
                proxies.insert(payload_id, proxy);
                label(discrete, "created", payload_id);
            }
            CollisionTreeCommand::Move {
                payload_id,
                lower,
                upper,
                displacement,
            } => {
                if let Some(proxy) = proxies.get(&payload_id).copied() {
                    label(
                        discrete,
                        "moved",
                        native(
                            case,
                            tree.move_proxy(proxy, aabb(case, lower, upper)?, vec2(displacement)),
                        )?,
                    );
                } else {
                    label(discrete, "missing_payload", payload_id);
                }
            }
            CollisionTreeCommand::Destroy { payload_id } => {
                if let Some(proxy) = proxies.remove(&payload_id) {
                    label(
                        discrete,
                        "destroyed",
                        native(case, tree.destroy_proxy(proxy))?,
                    );
                } else {
                    label(discrete, "missing_payload", payload_id);
                }
            }
            CollisionTreeCommand::Query { lower, upper } => {
                tree.query(aabb(case, lower, upper)?, |_proxy, payload| {
                    payload_ids.push(*payload);
                    QueryControl::Continue
                });
            }
            CollisionTreeCommand::Ray {
                start,
                end,
                max_fraction_bits,
            } => {
                let input = RayCastInput::new(vec2(start), vec2(end), max_fraction_bits.to_f32())
                    .map_err(|error| failure(case, error))?;
                native(
                    case,
                    tree.ray_cast(input, |_proxy, payload, _input| {
                        payload_ids.push(*payload);
                        RayCastControl::Ignore
                    }),
                )?;
            }
            CollisionTreeCommand::Metrics => push_tree_metrics(
                numeric,
                discrete,
                tree.proxy_count(),
                tree.height(),
                tree.max_balance(),
                tree.area_ratio(),
            ),
            CollisionTreeCommand::Touch { payload_id }
            | CollisionTreeCommand::Refilter { payload_id, .. } => {
                label(discrete, "unsupported_tree_command_payload", payload_id);
            }
            CollisionTreeCommand::UpdatePairs => label(discrete, "update_pairs", "not_applicable"),
        }
    }
    if case.operation() == CollisionProbeOperation::TreeLifecycle {
        label(discrete, "tree_valid", tree.validate());
    }
    if case.collection_policy() == liquidfun_test_protocol::CollectionPolicy::Set {
        payload_ids.sort_unstable();
        payload_ids.dedup();
    }
    Ok(())
}

pub(super) fn execute_broad_phase(
    case: &CollisionProbeCase,
    commands: &[CollisionTreeCommand],
    numeric: &mut Vec<CollisionProbeNumericValue>,
    discrete: &mut Vec<CollisionProbeDiscreteValue>,
    payload_ids: &mut Vec<u32>,
) -> Result<(), CollisionProbeExecutionError> {
    let mut broad = native(case, BroadPhase::<u32>::new())?;
    let mut proxies = BTreeMap::new();
    for command in commands {
        match *command {
            CollisionTreeCommand::Create {
                payload_id,
                lower,
                upper,
            } => {
                let proxy = native(
                    case,
                    broad.create_proxy(
                        aabb(case, lower, upper)?,
                        payload_id,
                        FilterData::default(),
                    ),
                )?;
                proxies.insert(payload_id, proxy);
            }
            CollisionTreeCommand::Move {
                payload_id,
                lower,
                upper,
                displacement,
            } => {
                if let Some(proxy) = proxies.get(&payload_id).copied() {
                    label(
                        discrete,
                        "moved",
                        native(
                            case,
                            broad.move_proxy(proxy, aabb(case, lower, upper)?, vec2(displacement)),
                        )?,
                    );
                } else {
                    label(discrete, "missing_payload", payload_id);
                }
            }
            CollisionTreeCommand::Touch { payload_id } => {
                if let Some(proxy) = proxies.get(&payload_id).copied() {
                    native(case, broad.touch_proxy(proxy))?;
                    label(discrete, "touched", payload_id);
                } else {
                    label(discrete, "missing_payload", payload_id);
                }
            }
            CollisionTreeCommand::Destroy { payload_id } => {
                if let Some(proxy) = proxies.remove(&payload_id) {
                    label(
                        discrete,
                        "destroyed",
                        native(case, broad.destroy_proxy(proxy))?,
                    );
                } else {
                    label(discrete, "missing_payload", payload_id);
                }
            }
            CollisionTreeCommand::Refilter {
                payload_id,
                category_bits,
                mask_bits,
                group_index,
            } => {
                if let Some(proxy) = proxies.get(&payload_id).copied() {
                    native(
                        case,
                        broad.set_filter_data(
                            proxy,
                            FilterData::new(category_bits, mask_bits, group_index),
                        ),
                    )?;
                    label(discrete, "refiltered", payload_id);
                } else {
                    label(discrete, "missing_payload", payload_id);
                }
            }
            CollisionTreeCommand::UpdatePairs => native(
                case,
                broad.update_pairs(|_a, first, _b, second| {
                    payload_ids.push(*first);
                    payload_ids.push(*second);
                }),
            )?,
            CollisionTreeCommand::Metrics => push_tree_metrics(
                numeric,
                discrete,
                broad.proxy_count(),
                broad.tree_height(),
                broad.tree_max_balance(),
                broad.tree_area_ratio(),
            ),
            CollisionTreeCommand::Query { .. } | CollisionTreeCommand::Ray { .. } => {
                label(discrete, "unsupported_broad_phase_command", "query_or_ray");
            }
        }
    }
    Ok(())
}

pub(super) fn push_distance_result(
    numeric: &mut Vec<CollisionProbeNumericValue>,
    discrete: &mut Vec<CollisionProbeDiscreteValue>,
    result: &DistanceResult,
) {
    number(numeric, "point_a_x", result.point_a().x);
    number(numeric, "point_a_y", result.point_a().y);
    number(numeric, "point_b_x", result.point_b().x);
    number(numeric, "point_b_y", result.point_b().y);
    number(numeric, "distance", result.distance());
    let diagnostic = distance_diagnostic(result.cache(), result.iterations());
    number(numeric, "cache_metric", diagnostic.metric());
    label(discrete, "iterations", diagnostic.iterations());
    label(
        discrete,
        "termination",
        format!("{:?}", diagnostic.termination()).to_ascii_lowercase(),
    );
    for (index, pair) in diagnostic.support_pairs().iter().enumerate() {
        label(discrete, format!("support_{index}_a"), pair.index_a());
        label(discrete, format!("support_{index}_b"), pair.index_b());
    }
}

pub(super) fn cache_seed(
    case: &CollisionProbeCase,
    cache: &CollisionCacheBits,
) -> Result<DistanceCacheSeed, CollisionProbeExecutionError> {
    let proxy_a = cache_fingerprint(case, cache.proxy_a())?;
    let proxy_b = cache_fingerprint(case, cache.proxy_b())?;
    let pairs = cache
        .support_pairs()
        .iter()
        .map(|pair| DistanceCacheSeedPair::new(pair.index_a as usize, pair.index_b as usize))
        .collect();
    DistanceCacheSeed::new(proxy_a, proxy_b, pairs, cache.metric_bits().to_f32())
        .map_err(|error| failure(case, error))
}

fn cache_fingerprint(
    case: &CollisionProbeCase,
    fingerprint: &CollisionProxyFingerprint,
) -> Result<DistanceProxyFingerprint, CollisionProbeExecutionError> {
    let kind = match fingerprint.shape_kind() {
        CollisionShapeKind::Circle => DistanceProxyKind::Circle,
        CollisionShapeKind::Edge => DistanceProxyKind::Edge,
        CollisionShapeKind::Polygon => DistanceProxyKind::Polygon,
        CollisionShapeKind::Chain => DistanceProxyKind::Chain,
    };
    let vertices = fingerprint
        .vertices()
        .iter()
        .map(|vertex| DistanceProxyVertexBits::new(vertex.x_bits.bits(), vertex.y_bits.bits()))
        .collect();
    DistanceProxyFingerprint::new(
        kind,
        fingerprint.child_index() as usize,
        fingerprint.radius_bits().bits(),
        vertices,
    )
    .map_err(|error| failure(case, error))
}

pub(super) const fn cache_rejection(reason: DistanceCacheSeedRejection) -> &'static str {
    match reason {
        DistanceCacheSeedRejection::ProxyAFingerprintMismatch => "proxy_a_fingerprint_mismatch",
        DistanceCacheSeedRejection::ProxyBFingerprintMismatch => "proxy_b_fingerprint_mismatch",
        DistanceCacheSeedRejection::SupportCountOutOfRange => "support_count_out_of_range",
        DistanceCacheSeedRejection::SupportIndexAOutOfRange => "support_index_a_out_of_range",
        DistanceCacheSeedRejection::SupportIndexBOutOfRange => "support_index_b_out_of_range",
        DistanceCacheSeedRejection::DuplicateSupportPair => "duplicate_support_pair",
        DistanceCacheSeedRejection::NonFiniteMetric => "non_finite_metric",
    }
}

pub(super) const fn cache_reset(reason: DistanceCacheSeedReset) -> &'static str {
    match reason {
        DistanceCacheSeedReset::MetricRatio => "metric_ratio",
        DistanceCacheSeedReset::MetricTooSmall => "metric_too_small",
    }
}

pub(super) fn classify_shape_rejection(
    definition: &CollisionShapeDefinition,
    child_index: u32,
) -> Option<(CollisionRejectionCategory, CollisionRejectionField)> {
    use CollisionRejectionCategory::{InvalidChildIndex, InvalidGeometry, NonFiniteValue};
    use CollisionRejectionField::{
        ChainVertices, ChildIndex, CircleCenter, CircleRadius, EdgeEnd, EdgeNext, EdgePrevious,
        EdgeStart, PolygonVertices,
    };

    let rejection = match definition {
        CollisionShapeDefinition::Circle {
            center,
            radius_bits,
            ..
        } => {
            if !vec2_is_finite(*center) {
                Some((NonFiniteValue, CircleCenter))
            } else if !radius_bits.to_f32().is_finite() {
                Some((NonFiniteValue, CircleRadius))
            } else if radius_bits.to_f32() < 0.0 {
                Some((InvalidGeometry, CircleRadius))
            } else {
                None
            }
        }
        CollisionShapeDefinition::Edge {
            start,
            end,
            maybe_previous,
            maybe_next,
            ..
        } => {
            if !vec2_is_finite(*start) {
                Some((NonFiniteValue, EdgeStart))
            } else if !vec2_is_finite(*end) {
                Some((NonFiniteValue, EdgeEnd))
            } else if start == end {
                Some((InvalidGeometry, EdgeEnd))
            } else if maybe_previous.is_some_and(|point| !vec2_is_finite(point)) {
                Some((NonFiniteValue, EdgePrevious))
            } else if maybe_previous == &Some(*start) {
                Some((InvalidGeometry, EdgePrevious))
            } else if maybe_next.is_some_and(|point| !vec2_is_finite(point)) {
                Some((NonFiniteValue, EdgeNext))
            } else if maybe_next == &Some(*end) {
                Some((InvalidGeometry, EdgeNext))
            } else {
                None
            }
        }
        CollisionShapeDefinition::Polygon { vertices, .. } => {
            if vertices.iter().copied().any(|point| !vec2_is_finite(point)) {
                Some((NonFiniteValue, PolygonVertices))
            } else if PolygonShape::new(&vertices.iter().copied().map(vec2).collect::<Vec<_>>())
                .is_err()
            {
                Some((InvalidGeometry, PolygonVertices))
            } else {
                None
            }
        }
        CollisionShapeDefinition::Chain {
            vertices,
            closed,
            maybe_previous,
            maybe_next,
            ..
        } => {
            if vertices.iter().copied().any(|point| !vec2_is_finite(point))
                || maybe_previous.is_some_and(|point| !vec2_is_finite(point))
                || maybe_next.is_some_and(|point| !vec2_is_finite(point))
            {
                Some((NonFiniteValue, ChainVertices))
            } else {
                let vertices = vertices.iter().copied().map(vec2).collect::<Vec<_>>();
                let result = if *closed {
                    ChainShape::closed(&vertices)
                } else {
                    ChainShape::open(&vertices, maybe_previous.map(vec2), maybe_next.map(vec2))
                };
                result.err().map(|_| (InvalidGeometry, ChainVertices))
            }
        }
    };
    if rejection.is_some() {
        return rejection;
    }
    (child_index as usize >= definition.child_count()).then_some((InvalidChildIndex, ChildIndex))
}

fn vec2_is_finite(value: Vec2Bits) -> bool {
    value.x_bits.to_f32().is_finite() && value.y_bits.to_f32().is_finite()
}

pub(super) fn wire_name(value: CollisionWitnessFamily) -> String {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(name)) => name,
        _ => "invalid_witness_family".to_owned(),
    }
}

pub(super) fn build_shape(
    case: &CollisionProbeCase,
    definition: &CollisionShapeDefinition,
) -> Result<Shape, CollisionProbeExecutionError> {
    let result = match definition {
        CollisionShapeDefinition::Circle {
            center,
            radius_bits,
            ..
        } => CircleShape::new(vec2(*center), radius_bits.to_f32()).map(Shape::from),
        CollisionShapeDefinition::Edge {
            start,
            end,
            maybe_previous,
            maybe_next,
            ..
        } => EdgeShape::with_adjacency(
            vec2(*start),
            vec2(*end),
            maybe_previous.map(vec2),
            maybe_next.map(vec2),
        )
        .map(Shape::from),
        CollisionShapeDefinition::Polygon { vertices, .. } => {
            PolygonShape::new(&vertices.iter().copied().map(vec2).collect::<Vec<_>>())
                .map(Shape::from)
        }
        CollisionShapeDefinition::Chain {
            vertices,
            closed,
            maybe_previous,
            maybe_next,
            ..
        } => {
            let vertices = vertices.iter().copied().map(vec2).collect::<Vec<_>>();
            if *closed {
                ChainShape::closed(&vertices)
            } else {
                ChainShape::open(&vertices, maybe_previous.map(vec2), maybe_next.map(vec2))
            }
            .map(Shape::from)
        }
    };
    native(case, result)
}

pub(super) fn child(
    case: &CollisionProbeCase,
    shape: &Shape,
    index: u32,
) -> Result<ChildIndex, CollisionProbeExecutionError> {
    native(case, shape.child_index(index as usize))
}
pub(super) fn vec2(value: Vec2Bits) -> Vec2 {
    Vec2::new(value.x_bits.to_f32(), value.y_bits.to_f32())
}
pub(super) fn transform_from(value: TransformBits) -> Transform {
    Transform::from_position_angle(vec2(value.position), value.angle_bits.to_f32())
}
pub(super) fn sweep(
    case: &CollisionProbeCase,
    value: SweepBits,
) -> Result<Sweep, CollisionProbeExecutionError> {
    native(
        case,
        Sweep::new(
            vec2(value.local_center),
            vec2(value.initial_center),
            vec2(value.center),
            value.initial_angle_bits.to_f32(),
            value.angle_bits.to_f32(),
            value.initial_fraction_bits.to_f32(),
        ),
    )
}
pub(super) fn aabb(
    case: &CollisionProbeCase,
    lower: Vec2Bits,
    upper: Vec2Bits,
) -> Result<Aabb, CollisionProbeExecutionError> {
    native(case, Aabb::new(vec2(lower), vec2(upper)))
}
pub(super) fn feature(value: CollisionFeatureBits) -> liquidfun::collision::ContactFeatureId {
    liquidfun::collision::ContactFeatureId::new(
        value.index_a,
        value.index_b,
        feature_kind(value.kind_a),
        feature_kind(value.kind_b),
    )
}
const fn feature_kind(value: CollisionFeatureKind) -> FeatureKind {
    match value {
        CollisionFeatureKind::Vertex => FeatureKind::Vertex,
        CollisionFeatureKind::Face => FeatureKind::Face,
    }
}
pub(super) fn number(
    values: &mut Vec<CollisionProbeNumericValue>,
    field: impl Into<Box<str>>,
    value: f32,
) {
    values.push(CollisionProbeNumericValue::new(
        field,
        FloatBits::from_f32(value),
    ));
}
#[allow(
    clippy::needless_pass_by_value,
    reason = "call sites pass compact scalars and owned diagnostic strings uniformly"
)]
pub(super) fn label(
    values: &mut Vec<CollisionProbeDiscreteValue>,
    field: impl Into<Box<str>>,
    value: impl ToString,
) {
    values.push(CollisionProbeDiscreteValue::new(field, value.to_string()));
}
pub(super) fn push_aabb(values: &mut Vec<CollisionProbeNumericValue>, value: Aabb) {
    number(values, "lower_x", value.lower_bound().x);
    number(values, "lower_y", value.lower_bound().y);
    number(values, "upper_x", value.upper_bound().x);
    number(values, "upper_y", value.upper_bound().y);
}
fn push_tree_metrics(
    numeric: &mut Vec<CollisionProbeNumericValue>,
    discrete: &mut Vec<CollisionProbeDiscreteValue>,
    count: usize,
    height: i32,
    balance: i32,
    ratio: f32,
) {
    label(discrete, "proxy_count", count);
    label(discrete, "height", height);
    label(discrete, "max_balance", balance);
    number(numeric, "area_ratio", ratio);
}
pub(super) fn push_feature(
    values: &mut Vec<CollisionProbeDiscreteValue>,
    index: usize,
    feature: liquidfun::collision::ContactFeatureId,
) {
    label(
        values,
        format!("feature_{index}_index_a"),
        feature.index_a(),
    );
    label(
        values,
        format!("feature_{index}_index_b"),
        feature.index_b(),
    );
    label(
        values,
        format!("feature_{index}_kind_a"),
        format!("{:?}", feature.kind_a()).to_ascii_lowercase(),
    );
    label(
        values,
        format!("feature_{index}_kind_b"),
        format!("{:?}", feature.kind_b()).to_ascii_lowercase(),
    );
}
pub(super) const fn shape_kind(shape: &Shape) -> &'static str {
    match shape {
        Shape::Circle(_) => "circle",
        Shape::Edge(_) => "edge",
        Shape::Polygon(_) => "polygon",
        Shape::Chain(_) => "chain",
    }
}
pub(super) const fn orientation(value: PairOrientation) -> &'static str {
    match value {
        PairOrientation::Primary => "primary",
        PairOrientation::Reversed => "reversed",
    }
}
pub(super) fn native<T, E: std::fmt::Display>(
    case: &CollisionProbeCase,
    result: Result<T, E>,
) -> Result<T, CollisionProbeExecutionError> {
    result.map_err(|error| failure(case, error))
}
pub(super) fn failure(
    case: &CollisionProbeCase,
    error: impl std::fmt::Display,
) -> CollisionProbeExecutionError {
    CollisionProbeExecutionError::Native {
        case_id: case.case_id().into(),
        message: error.to_string().into_boxed_str(),
    }
}
