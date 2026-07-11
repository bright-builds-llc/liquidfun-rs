//! Native execution for the closed Phase 5 collision-probe family.

mod support;

use support::{
    build_shape, cache_rejection, cache_reset, cache_seed, child, classify_shape_rejection,
    execute_broad_phase, execute_tree, failure, feature, label, native, number, orientation,
    push_aabb, push_distance_result, push_feature, shape_kind, sweep, transform_from, vec2,
    wire_name,
};

use liquidfun::collision::{
    CollisionOutcome, TimeOfImpactInput, collide_shapes,
    differential::{
        ClipDiagnosticInput, DiagnosticFeature, DistanceCacheReplayOutcome,
        clip_segment_diagnostic, replay_distance_cache, time_of_impact_diagnostic,
    },
    distance, test_overlap, time_of_impact,
};
use liquidfun_test_protocol::{
    CollisionExpectedOutcome, CollisionProbeCase, CollisionProbeDecodeError, CollisionProbeInput,
    CollisionProbeOperation, CollisionProbeRequestRecord, CollisionProbeResult,
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
    if matches!(
        case.expected_outcome(),
        CollisionExpectedOutcome::Rejected { .. }
    ) {
        let CollisionProbeInput::Shape {
            shape, child_index, ..
        } = case.input()
        else {
            return Err(failure(
                case,
                "expected-rejected case escaped the shape-construction boundary",
            ));
        };
        if let Some((category, field)) = classify_shape_rejection(shape, *child_index) {
            return Ok(CollisionProbeResult::rejected(
                case.case_id(),
                case.operation(),
                category,
                field,
            ));
        }
    }
    let mut numeric = Vec::new();
    let mut discrete = Vec::new();
    let mut payload_ids = Vec::new();
    label(
        &mut discrete,
        "witness_family",
        wire_name(case.witness_family()),
    );
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
                maybe_cache,
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
                    if let Some(cache) = maybe_cache {
                        let replay = native(
                            case,
                            replay_distance_cache(
                                &first,
                                child_a,
                                transform_a,
                                &second,
                                child_b,
                                transform_b,
                                *use_radii,
                                cache_seed(case, cache)?,
                            ),
                        )?;
                        match replay {
                            DistanceCacheReplayOutcome::Used { result } => {
                                label(&mut discrete, "cache_outcome", "used");
                                push_distance_result(&mut numeric, &mut discrete, &result);
                            }
                            DistanceCacheReplayOutcome::Reset { result, reason } => {
                                label(&mut discrete, "cache_outcome", "reset");
                                label(&mut discrete, "cache_reason", cache_reset(reason));
                                push_distance_result(&mut numeric, &mut discrete, &result);
                            }
                            DistanceCacheReplayOutcome::Rejected { reason } => {
                                label(&mut discrete, "cache_outcome", "rejected");
                                label(&mut discrete, "cache_reason", cache_rejection(reason));
                            }
                        }
                    } else {
                        label(&mut discrete, "cache_outcome", "cold");
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
                        push_distance_result(&mut numeric, &mut discrete, &result);
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
