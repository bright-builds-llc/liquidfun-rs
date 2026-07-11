//! Native execution for the closed Phase 5 collision-probe family.

use std::collections::BTreeMap;

use liquidfun::{
    collision::{
        Aabb, BroadPhase, ChainShape, ChildIndex, CircleShape, CollisionOutcome, DynamicTree,
        EdgeShape, FeatureKind, FilterData, PairOrientation, PolygonShape, QueryControl,
        RayCastControl, RayCastInput, Shape, TimeOfImpactInput, collide_shapes,
        differential::{
            ClipDiagnosticInput, DiagnosticFeature, clip_segment_diagnostic, distance_diagnostic,
            time_of_impact_diagnostic,
        },
        distance, test_overlap, time_of_impact,
    },
    math::{Sweep, Transform, Vec2},
};
use liquidfun_test_protocol::{
    CollisionFeatureBits, CollisionFeatureKind, CollisionProbeCase, CollisionProbeDecodeError,
    CollisionProbeDiscreteValue, CollisionProbeInput, CollisionProbeNumericValue,
    CollisionProbeOperation, CollisionProbeRequestRecord, CollisionProbeResult,
    CollisionShapeDefinition, CollisionTreeCommand, FloatBits, SweepBits, TransformBits, Vec2Bits,
};

/// Typed failure while mapping a validated collision probe onto native kernels.
#[derive(Debug, thiserror::Error)]
pub enum CollisionProbeExecutionError {
    /// A validated wire value could not be reconstructed by a checked kernel API.
    #[error("collision probe case {case_id} failed checked native execution: {message}")]
    Native {
        /// Stable case identity.
        case_id: Box<str>,
        /// Bounded typed-error presentation.
        message: Box<str>,
    },
    /// Result construction rejected an aggregate invariant.
    #[error(transparent)]
    Result(#[from] CollisionProbeDecodeError),
}

/// Stateless executor for one validated request.
pub struct NativeCollisionProbeExecutor;

impl NativeCollisionProbeExecutor {
    /// Executes every case in request order with fresh per-case state.
    ///
    /// # Errors
    ///
    /// Returns [`CollisionProbeExecutionError`] when checked Rust geometry cannot be reconstructed
    /// or a bounded result invariant is violated.
    pub fn execute(
        request: &CollisionProbeRequestRecord,
    ) -> Result<Box<[CollisionProbeResult]>, CollisionProbeExecutionError> {
        request
            .scenario()
            .cases()
            .iter()
            .map(execute_case)
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "the closed operation registry remains auditable in one dispatch"
)]
fn execute_case(
    case: &CollisionProbeCase,
) -> Result<CollisionProbeResult, CollisionProbeExecutionError> {
    let mut numeric = Vec::new();
    let mut discrete = Vec::new();
    let mut payload_ids = Vec::new();
    match (case.operation(), case.input()) {
        (CollisionProbeOperation::ShapeConstruction, CollisionProbeInput::Shape { shape, .. }) => {
            let shape = build_shape(case, shape)?;
            number(&mut numeric, "radius", shape.radius());
            label(&mut discrete, "shape_kind", shape_kind(&shape));
            label(&mut discrete, "child_count", shape.child_count());
        }
        (
            CollisionProbeOperation::ShapeUnaryQuery,
            CollisionProbeInput::Shape {
                shape,
                child_index,
                transform,
                query_point,
            },
        ) => {
            let shape = build_shape(case, shape)?;
            let child = child(case, &shape, *child_index)?;
            let transform = transform_from(*transform);
            let point = vec2(*query_point);
            let contains = native(case, shape.test_point(transform, point))?;
            let point_distance = native(case, shape.distance_to_point(transform, point, child))?;
            let aabb = native(case, shape.compute_aabb(transform, child))?;
            label(&mut discrete, "contains", contains);
            number(&mut numeric, "distance", point_distance.distance());
            number(&mut numeric, "normal_x", point_distance.normal().x);
            number(&mut numeric, "normal_y", point_distance.normal().y);
            push_aabb(&mut numeric, aabb);
        }
        (
            operation @ (CollisionProbeOperation::Distance
            | CollisionProbeOperation::Overlap
            | CollisionProbeOperation::Manifold
            | CollisionProbeOperation::PairDispatch),
            CollisionProbeInput::Pair {
                shapes,
                child_indices,
                transforms,
                use_radii,
                ..
            },
        ) => {
            let first = build_shape(case, &shapes[0])?;
            let second = build_shape(case, &shapes[1])?;
            let child_a = child(case, &first, child_indices[0])?;
            let child_b = child(case, &second, child_indices[1])?;
            let transform_a = transform_from(transforms[0]);
            let transform_b = transform_from(transforms[1]);
            match operation {
                CollisionProbeOperation::Distance => {
                    let result = native(
                        case,
                        distance(
                            &first,
                            child_a,
                            transform_a,
                            &second,
                            child_b,
                            transform_b,
                            *use_radii,
                            None,
                        ),
                    )?;
                    number(&mut numeric, "point_a_x", result.point_a().x);
                    number(&mut numeric, "point_a_y", result.point_a().y);
                    number(&mut numeric, "point_b_x", result.point_b().x);
                    number(&mut numeric, "point_b_y", result.point_b().y);
                    number(&mut numeric, "distance", result.distance());
                    let diagnostic = distance_diagnostic(result.cache(), result.iterations());
                    number(&mut numeric, "cache_metric", diagnostic.metric());
                    label(&mut discrete, "iterations", diagnostic.iterations());
                    label(
                        &mut discrete,
                        "termination",
                        format!("{:?}", diagnostic.termination()).to_ascii_lowercase(),
                    );
                    for (index, pair) in diagnostic.support_pairs().iter().enumerate() {
                        label(&mut discrete, format!("support_{index}_a"), pair.index_a());
                        label(&mut discrete, format!("support_{index}_b"), pair.index_b());
                    }
                }
                CollisionProbeOperation::Overlap => {
                    label(
                        &mut discrete,
                        "overlap",
                        native(
                            case,
                            test_overlap(
                                &first,
                                child_a,
                                transform_a,
                                &second,
                                child_b,
                                transform_b,
                            ),
                        )?,
                    );
                }
                CollisionProbeOperation::Manifold | CollisionProbeOperation::PairDispatch => {
                    match native(
                        case,
                        collide_shapes(&first, child_a, transform_a, &second, child_b, transform_b),
                    )? {
                        CollisionOutcome::Touching(pair) => {
                            label(&mut discrete, "outcome", "touching");
                            label(
                                &mut discrete,
                                "orientation",
                                orientation(pair.orientation()),
                            );
                            label(
                                &mut discrete,
                                "manifold_kind",
                                format!("{:?}", pair.manifold().kind()).to_ascii_lowercase(),
                            );
                            label(&mut discrete, "point_count", pair.manifold().points().len());
                        }
                        CollisionOutcome::Separated => label(&mut discrete, "outcome", "separated"),
                        CollisionOutcome::Unsupported => {
                            label(&mut discrete, "outcome", "unsupported");
                        }
                    }
                }
                _ => unreachable!("outer match restricts pair operations"),
            }
        }
        (
            CollisionProbeOperation::Clip,
            CollisionProbeInput::Clip {
                points,
                normal,
                offset_bits,
                vertex_index_a,
            },
        ) => {
            let input = ClipDiagnosticInput::new(
                [
                    (
                        vec2(points[0].point),
                        DiagnosticFeature::new(feature(points[0].feature)),
                    ),
                    (
                        vec2(points[1].point),
                        DiagnosticFeature::new(feature(points[1].feature)),
                    ),
                ],
                vec2(*normal),
                offset_bits.to_f32(),
                *vertex_index_a,
            )
            .map_err(|error| failure(case, error))?;
            let result = clip_segment_diagnostic(input);
            label(&mut discrete, "point_count", result.points().len());
            for (index, point) in result.points().iter().enumerate() {
                number(&mut numeric, format!("point_{index}_x"), point.point().x);
                number(&mut numeric, format!("point_{index}_y"), point.point().y);
                push_feature(&mut discrete, index, point.feature().feature());
            }
        }
        (
            CollisionProbeOperation::FeatureTransition,
            CollisionProbeInput::Features { previous, current },
        ) => {
            for (index, old) in previous.iter().enumerate() {
                label(
                    &mut discrete,
                    format!("previous_{index}"),
                    if current.contains(old) {
                        "persisted"
                    } else {
                        "removed"
                    },
                );
            }
            for (index, new) in current.iter().enumerate() {
                label(
                    &mut discrete,
                    format!("current_{index}"),
                    if previous.contains(new) {
                        "persisted"
                    } else {
                        "added"
                    },
                );
            }
        }
        (
            CollisionProbeOperation::TreeLifecycle
            | CollisionProbeOperation::TreeQuery
            | CollisionProbeOperation::TreeRay
            | CollisionProbeOperation::TreeMetrics,
            CollisionProbeInput::Tree { commands },
        ) => {
            execute_tree(
                case,
                commands,
                &mut numeric,
                &mut discrete,
                &mut payload_ids,
            )?;
        }
        (
            CollisionProbeOperation::BroadPhaseMoveTouch
            | CollisionProbeOperation::BroadPhasePairs
            | CollisionProbeOperation::BroadPhaseFilter
            | CollisionProbeOperation::BroadPhaseRefilter,
            CollisionProbeInput::Tree { commands },
        ) => {
            execute_broad_phase(
                case,
                commands,
                &mut numeric,
                &mut discrete,
                &mut payload_ids,
            )?;
        }
        (
            CollisionProbeOperation::TimeOfImpact,
            CollisionProbeInput::TimeOfImpact {
                shapes,
                child_indices,
                sweeps,
                t_max_bits,
            },
        ) => {
            let first = build_shape(case, &shapes[0])?;
            let second = build_shape(case, &shapes[1])?;
            let input = TimeOfImpactInput::new(
                &first,
                child(case, &first, child_indices[0])?,
                sweep(case, sweeps[0])?,
                &second,
                child(case, &second, child_indices[1])?,
                sweep(case, sweeps[1])?,
                t_max_bits.to_f32(),
            )
            .map_err(|error| failure(case, error))?;
            let output = native(case, time_of_impact(&input))?;
            let diagnostic = time_of_impact_diagnostic(output.state(), output.time());
            number(&mut numeric, "time", diagnostic.time());
            label(
                &mut discrete,
                "state",
                format!("{:?}", diagnostic.state()).to_ascii_lowercase(),
            );
            label(
                &mut discrete,
                "termination",
                format!("{:?}", diagnostic.termination()).to_ascii_lowercase(),
            );
        }
        _ => {
            return Err(CollisionProbeExecutionError::Native {
                case_id: case.case_id().into(),
                message: "validated operation/input mapping became unreachable".into(),
            });
        }
    }
    CollisionProbeResult::new(
        case.case_id(),
        case.operation(),
        numeric,
        discrete,
        payload_ids,
    )
    .map_err(Into::into)
}

fn execute_tree(
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

fn execute_broad_phase(
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

fn build_shape(
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
            vertices, closed, ..
        } => {
            let vertices = vertices.iter().copied().map(vec2).collect::<Vec<_>>();
            if *closed {
                ChainShape::closed(&vertices)
            } else {
                ChainShape::open(&vertices, None, None)
            }
            .map(Shape::from)
        }
    };
    native(case, result)
}

fn child(
    case: &CollisionProbeCase,
    shape: &Shape,
    index: u32,
) -> Result<ChildIndex, CollisionProbeExecutionError> {
    native(case, shape.child_index(index as usize))
}
fn vec2(value: Vec2Bits) -> Vec2 {
    Vec2::new(value.x_bits.to_f32(), value.y_bits.to_f32())
}
fn transform_from(value: TransformBits) -> Transform {
    Transform::from_position_angle(vec2(value.position), value.angle_bits.to_f32())
}
fn sweep(
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
fn aabb(
    case: &CollisionProbeCase,
    lower: Vec2Bits,
    upper: Vec2Bits,
) -> Result<Aabb, CollisionProbeExecutionError> {
    native(case, Aabb::new(vec2(lower), vec2(upper)))
}
fn feature(value: CollisionFeatureBits) -> liquidfun::collision::ContactFeatureId {
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
fn number(values: &mut Vec<CollisionProbeNumericValue>, field: impl Into<Box<str>>, value: f32) {
    values.push(CollisionProbeNumericValue::new(
        field,
        FloatBits::from_f32(value),
    ));
}
#[allow(
    clippy::needless_pass_by_value,
    reason = "call sites pass compact scalars and owned diagnostic strings uniformly"
)]
fn label(
    values: &mut Vec<CollisionProbeDiscreteValue>,
    field: impl Into<Box<str>>,
    value: impl ToString,
) {
    values.push(CollisionProbeDiscreteValue::new(field, value.to_string()));
}
fn push_aabb(values: &mut Vec<CollisionProbeNumericValue>, value: Aabb) {
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
fn push_feature(
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
const fn shape_kind(shape: &Shape) -> &'static str {
    match shape {
        Shape::Circle(_) => "circle",
        Shape::Edge(_) => "edge",
        Shape::Polygon(_) => "polygon",
        Shape::Chain(_) => "chain",
    }
}
const fn orientation(value: PairOrientation) -> &'static str {
    match value {
        PairOrientation::Primary => "primary",
        PairOrientation::Reversed => "reversed",
    }
}
fn native<T, E: std::fmt::Display>(
    case: &CollisionProbeCase,
    result: Result<T, E>,
) -> Result<T, CollisionProbeExecutionError> {
    result.map_err(|error| failure(case, error))
}
fn failure(
    case: &CollisionProbeCase,
    error: impl std::fmt::Display,
) -> CollisionProbeExecutionError {
    CollisionProbeExecutionError::Native {
        case_id: case.case_id().into(),
        message: error.to_string().into_boxed_str(),
    }
}
