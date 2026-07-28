use super::*;

pub(super) fn physics_scenario_schema() -> Value {
    closed_record(
        &json!({
            "checkpoints": {
                "items": closed_record(
                    &json!({
                        "after_command_id": semantic_id_schema(),
                        "checkpoint_id": semantic_id_schema(),
                        "observables": { "items": { "enum": ["world_counts", "simulation_time"] }, "maxItems": 128, "type": "array" },
                        "phase": bounded_string_schema()
                    }),
                    &["checkpoint_id", "after_command_id", "phase", "observables"],
                ),
                "maxItems": 4096,
                "type": "array"
            },
            "commands": {
                "items": closed_record(
                    &json!({
                        "command_id": semantic_id_schema(),
                        "kind": { "const": "step" },
                        "particle_iterations": { "maximum": 255, "minimum": 1, "type": "integer" },
                        "position_iterations": { "maximum": 255, "minimum": 1, "type": "integer" },
                        "timestep_bits": float_bits_schema(),
                        "velocity_iterations": { "maximum": 255, "minimum": 1, "type": "integer" }
                    }),
                    &["kind", "command_id", "timestep_bits", "velocity_iterations", "position_iterations", "particle_iterations"],
                ),
                "maxItems": 4096,
                "minItems": 1,
                "type": "array"
            },
            "entities": { "items": false, "maxItems": 0, "type": "array" },
            "gravity_x_bits": float_bits_schema(),
            "gravity_y_bits": float_bits_schema(),
            "scenario_id": semantic_id_schema(),
            "source": scenario_source_schema()
        }),
        &[
            "scenario_id",
            "source",
            "gravity_x_bits",
            "gravity_y_bits",
            "entities",
            "commands",
            "checkpoints",
        ],
    )
}

pub(super) fn math_probe_scenario_schema() -> Value {
    closed_record(
        &json!({
            "cases": {
                "items": closed_record(
                    &json!({
                        "case_id": semantic_id_schema(),
                        "horizon": schema_ref("math_probe_horizon"),
                        "input": math_probe_input_schema(),
                        "operation": { "enum": math_probe_operations() },
                        "policy_path": { "enum": math_probe_policy_paths() }
                    }),
                    &["case_id", "operation", "policy_path", "horizon", "input"],
                ),
                "maxItems": 256,
                "minItems": 1,
                "type": "array"
            },
            "scenario_id": semantic_id_schema(),
            "source": scenario_source_schema()
        }),
        &["scenario_id", "source", "cases"],
    )
}

pub(super) fn probe_request_schema(record_kind: &str) -> Value {
    closed_record(
        &json!({
            "protocol_version": version_schema(),
            "record_kind": { "const": record_kind },
            "request_id": semantic_id_schema(),
            "requested_trace_schema_version": version_schema(),
            "scenario": { "$ref": "scenario-v1.schema.json" },
            "scenario_schema_version": version_schema(),
            "tolerance_profile_sha256": sha256_schema(),
            "tolerance_profile_version": version_schema()
        }),
        &[
            "protocol_version",
            "record_kind",
            "request_id",
            "scenario_schema_version",
            "requested_trace_schema_version",
            "tolerance_profile_version",
            "tolerance_profile_sha256",
            "scenario",
        ],
    )
}

pub(super) fn collision_probe_scenario_schema() -> Value {
    closed_record(
        &json!({
            "cases": {
                "items": closed_record(
                    &json!({
                        "case_id": semantic_id_schema(),
                        "collection_policy": { "enum": ["ordered", "set"] },
                        "expected_outcome": collision_expected_outcome_schema(),
                        "horizon": collision_probe_horizon_schema(),
                        "input": collision_probe_input_schema(),
                        "operation": { "enum": collision_probe_operations() },
                        "policy_path": { "enum": collision_probe_policy_paths() },
                        "witness_family": { "enum": collision_witness_families() }
                    }),
                    &["case_id", "witness_family", "expected_outcome", "operation", "policy_path", "horizon", "collection_policy", "input"],
                ),
                "maxItems": 256,
                "minItems": 1,
                "type": "array"
            },
            "scenario_id": semantic_id_schema(),
            "source": scenario_source_schema()
        }),
        &["scenario_id", "source", "cases"],
    )
}

pub(super) fn collision_witness_families() -> Value {
    serde_json::to_value(crate::CollisionWitnessFamily::REQUIRED.as_slice())
        .expect("closed witness-family enum serialization cannot fail")
}

pub(super) fn collision_expected_outcome_schema() -> Value {
    json!({
        "oneOf": [
            closed_record(&json!({ "kind": { "const": "accepted" } }), &["kind"]),
            closed_record(
                &json!({
                    "kind": { "const": "rejected" },
                    "category": { "enum": ["non_finite_value", "invalid_geometry", "invalid_child_index"] },
                    "field": { "enum": ["circle_center", "circle_radius", "edge_start", "edge_end", "edge_previous", "edge_next", "polygon_vertices", "chain_vertices", "child_index"] }
                }),
                &["kind", "category", "field"],
            )
        ]
    })
}

pub(super) fn collision_probe_operations() -> Value {
    json!([
        "shape_construction",
        "shape_unary_query",
        "distance",
        "overlap",
        "clip",
        "manifold",
        "pair_dispatch",
        "feature_transition",
        "tree_lifecycle",
        "tree_query",
        "tree_ray",
        "tree_metrics",
        "broad_phase_move_touch",
        "broad_phase_pairs",
        "broad_phase_filter",
        "broad_phase_refilter",
        "time_of_impact"
    ])
}

pub(super) fn collision_probe_policy_paths() -> Value {
    json!([
        "collision.shape_construction.result",
        "collision.shape_unary_query.result",
        "collision.distance.result",
        "collision.overlap.result",
        "collision.clip.result",
        "collision.manifold.result",
        "collision.pair_dispatch.result",
        "collision.feature_transition.result",
        "collision.tree_lifecycle.result",
        "collision.tree_query.result",
        "collision.tree_ray.result",
        "collision.tree_metrics.result",
        "collision.broad_phase_move_touch.result",
        "collision.broad_phase_pairs.result",
        "collision.broad_phase_filter.result",
        "collision.broad_phase_refilter.result",
        "collision.time_of_impact.result"
    ])
}

pub(super) fn collision_probe_horizon_schema() -> Value {
    json!({
        "oneOf": [
            closed_record(&json!({ "kind": { "const": "operation" } }), &["kind"]),
            closed_record(&json!({ "kind": { "const": "phase_local" } }), &["kind"])
        ]
    })
}

pub(super) fn collision_probe_input_schema() -> Value {
    let shape = collision_shape_schema();
    let transform = schema_ref("transform_bits");
    let sweep = schema_ref("sweep_bits");
    let vec2 = schema_ref("vec2_bits");
    let feature = collision_feature_schema();
    let clip_point = closed_record(
        &json!({ "point": vec2, "feature": feature }),
        &["point", "feature"],
    );
    json!({
        "oneOf": [
            tagged_probe_input(
                "shape",
                &json!({
                    "shape": shape,
                    "child_index": uint32_schema(),
                    "transform": transform,
                    "query_point": schema_ref("vec2_bits")
                }),
                &["shape", "child_index", "transform", "query_point"],
            ),
            tagged_probe_input(
                "pair",
                &json!({
                    "shapes": { "items": collision_shape_schema(), "maxItems": 2, "minItems": 2, "type": "array" },
                    "child_indices": { "items": uint32_schema(), "maxItems": 2, "minItems": 2, "type": "array" },
                    "transforms": { "items": schema_ref("transform_bits"), "maxItems": 2, "minItems": 2, "type": "array" },
                    "use_radii": { "type": "boolean" },
                    "maybe_cache": nullable_schema(&collision_cache_schema())
                }),
                &["shapes", "child_indices", "transforms", "use_radii", "maybe_cache"],
            ),
            tagged_probe_input(
                "clip",
                &json!({
                    "points": { "items": clip_point, "maxItems": 2, "minItems": 2, "type": "array" },
                    "normal": schema_ref("vec2_bits"),
                    "offset_bits": float_bits_schema(),
                    "vertex_index_a": { "maximum": u8::MAX, "minimum": 0, "type": "integer" }
                }),
                &["points", "normal", "offset_bits", "vertex_index_a"],
            ),
            tagged_probe_input(
                "features",
                &json!({
                    "previous": { "items": collision_feature_schema(), "maxItems": 2, "type": "array" },
                    "current": { "items": collision_feature_schema(), "maxItems": 2, "type": "array" }
                }),
                &["previous", "current"],
            ),
            tagged_probe_input(
                "tree",
                &json!({
                    "commands": { "items": collision_tree_command_schema(), "maxItems": 128, "minItems": 1, "type": "array" }
                }),
                &["commands"],
            ),
            tagged_probe_input(
                "time_of_impact",
                &json!({
                    "shapes": { "items": collision_shape_schema(), "maxItems": 2, "minItems": 2, "type": "array" },
                    "child_indices": { "items": uint32_schema(), "maxItems": 2, "minItems": 2, "type": "array" },
                    "sweeps": { "items": sweep, "maxItems": 2, "minItems": 2, "type": "array" },
                    "t_max_bits": float_bits_schema()
                }),
                &["shapes", "child_indices", "sweeps", "t_max_bits"],
            )
        ]
    })
}

pub(super) fn collision_shape_schema() -> Value {
    let vec2 = schema_ref("vec2_bits");
    json!({
        "oneOf": [
            tagged_probe_input(
                "circle",
                &json!({ "shape_id": semantic_id_schema(), "center": vec2, "radius_bits": float_bits_schema() }),
                &["shape_id", "center", "radius_bits"],
            ),
            tagged_probe_input(
                "edge",
                &json!({
                    "shape_id": semantic_id_schema(),
                    "start": schema_ref("vec2_bits"),
                    "end": schema_ref("vec2_bits"),
                    "maybe_previous": nullable_schema(&schema_ref("vec2_bits")),
                    "maybe_next": nullable_schema(&schema_ref("vec2_bits"))
                }),
                &["shape_id", "start", "end", "maybe_previous", "maybe_next"],
            ),
            tagged_probe_input(
                "polygon",
                &json!({
                    "shape_id": semantic_id_schema(),
                    "vertices": { "items": schema_ref("vec2_bits"), "maxItems": 32, "type": "array" }
                }),
                &["shape_id", "vertices"],
            ),
            tagged_probe_input(
                "chain",
                &json!({
                    "shape_id": semantic_id_schema(),
                    "vertices": { "items": schema_ref("vec2_bits"), "maxItems": 32, "type": "array" },
                    "closed": { "type": "boolean" },
                    "maybe_previous": nullable_schema(&schema_ref("vec2_bits")),
                    "maybe_next": nullable_schema(&schema_ref("vec2_bits"))
                }),
                &["shape_id", "vertices", "closed", "maybe_previous", "maybe_next"],
            )
        ]
    })
}

pub(super) fn collision_cache_schema() -> Value {
    closed_record(
        &json!({
            "proxy_a": collision_proxy_fingerprint_schema(),
            "proxy_b": collision_proxy_fingerprint_schema(),
            "support_pairs": {
                "items": closed_record(
                    &json!({ "index_a": uint32_schema(), "index_b": uint32_schema() }),
                    &["index_a", "index_b"],
                ),
                "maxItems": 4,
                "type": "array"
            },
            "metric_bits": float_bits_schema()
        }),
        &["proxy_a", "proxy_b", "support_pairs", "metric_bits"],
    )
}

pub(super) fn collision_proxy_fingerprint_schema() -> Value {
    closed_record(
        &json!({
            "shape_kind": { "enum": ["circle", "edge", "polygon", "chain"] },
            "child_index": uint32_schema(),
            "radius_bits": float_bits_schema(),
            "vertices": { "items": schema_ref("vec2_bits"), "maxItems": 32, "minItems": 1, "type": "array" }
        }),
        &["shape_kind", "child_index", "radius_bits", "vertices"],
    )
}

pub(super) fn collision_feature_schema() -> Value {
    closed_record(
        &json!({
            "index_a": { "maximum": u8::MAX, "minimum": 0, "type": "integer" },
            "index_b": { "maximum": u8::MAX, "minimum": 0, "type": "integer" },
            "kind_a": { "enum": ["vertex", "face"] },
            "kind_b": { "enum": ["vertex", "face"] }
        }),
        &["index_a", "index_b", "kind_a", "kind_b"],
    )
}

pub(super) fn collision_tree_command_schema() -> Value {
    let vec2 = schema_ref("vec2_bits");
    json!({
        "oneOf": [
            tagged_probe_input("create", &json!({ "payload_id": uint32_schema(), "lower": vec2, "upper": schema_ref("vec2_bits") }), &["payload_id", "lower", "upper"]),
            tagged_probe_input("move", &json!({ "payload_id": uint32_schema(), "lower": schema_ref("vec2_bits"), "upper": schema_ref("vec2_bits"), "displacement": schema_ref("vec2_bits") }), &["payload_id", "lower", "upper", "displacement"]),
            tagged_probe_input("touch", &json!({ "payload_id": uint32_schema() }), &["payload_id"]),
            tagged_probe_input("destroy", &json!({ "payload_id": uint32_schema() }), &["payload_id"]),
            tagged_probe_input("query", &json!({ "lower": schema_ref("vec2_bits"), "upper": schema_ref("vec2_bits") }), &["lower", "upper"]),
            tagged_probe_input("ray", &json!({ "start": schema_ref("vec2_bits"), "end": schema_ref("vec2_bits"), "max_fraction_bits": float_bits_schema() }), &["start", "end", "max_fraction_bits"]),
            tagged_probe_input("refilter", &json!({ "payload_id": uint32_schema(), "category_bits": { "maximum": u16::MAX, "minimum": 0, "type": "integer" }, "mask_bits": { "maximum": u16::MAX, "minimum": 0, "type": "integer" }, "group_index": { "maximum": i16::MAX, "minimum": i16::MIN, "type": "integer" } }), &["payload_id", "category_bits", "mask_bits", "group_index"]),
            tagged_probe_input("update_pairs", &json!({}), &[]),
            tagged_probe_input("metrics", &json!({}), &[])
        ]
    })
}

pub(super) fn nullable_schema(value: &Value) -> Value {
    json!({ "oneOf": [value, { "type": "null" }] })
}

pub(super) fn math_probe_result_schema() -> Value {
    closed_record(
        &json!({
            "case_id": semantic_id_schema(),
            "discrete": {
                "items": closed_record(
                    &json!({
                        "field": { "enum": ["predicate", "non_zero_determinant", "normalized", "advanced"] },
                        "value": { "type": "boolean" }
                    }),
                    &["field", "value"],
                ),
                "maxItems": 16,
                "type": "array"
            },
            "horizon": math_probe_horizon_schema(),
            "operation": { "enum": math_probe_operations() },
            "policy_path": { "enum": math_probe_policy_paths() },
            "values": {
                "items": closed_record(
                    &json!({
                        "bits": float_bits_schema(),
                        "class": { "enum": ["zero", "subnormal", "normal", "infinite", "nan"] },
                        "field": { "enum": ["value", "x", "y", "z", "length", "sine", "cosine", "position_x", "position_y", "angle", "initial_center_x", "initial_center_y", "initial_angle", "initial_fraction", "left_associated", "right_associated", "even_midpoint", "odd_midpoint"] },
                        "negative": { "type": "boolean" }
                    }),
                    &["field", "bits", "class", "negative"],
                ),
                "maxItems": 64,
                "type": "array"
            }
        }),
        &[
            "case_id",
            "operation",
            "policy_path",
            "horizon",
            "values",
            "discrete",
        ],
    )
}

pub(super) fn math_probe_operations() -> Value {
    json!([
        "is_valid",
        "abs",
        "min",
        "max",
        "clamp",
        "inv_sqrt",
        "vec_length",
        "vec_normalize",
        "dot",
        "cross",
        "mat22_solve",
        "mat33_solve",
        "mat22_inverse",
        "mat33_sym_inverse",
        "rotation",
        "transform",
        "sweep_transform",
        "sweep_advance",
        "sweep_normalize",
        "cancellation",
        "halfway_rounding",
        "overflow",
        "underflow",
        "fma_witness"
    ])
}

pub(super) fn math_probe_policy_paths() -> Value {
    json!([
        "math.branch.is_valid",
        "math.operation.abs",
        "math.operation.min",
        "math.pass_through.max",
        "math.operation.clamp",
        "math.operation.inv_sqrt",
        "math.vector.length",
        "math.vector.normalize",
        "math.vector.dot",
        "math.vector.cross",
        "math.matrix22.solve",
        "math.matrix33.solve",
        "math.matrix22.inverse",
        "math.matrix33.symmetric_inverse",
        "math.rotation",
        "math.transform.operation",
        "math.transform.steps_32",
        "math.sweep.transform",
        "math.sweep.advance_steps_4",
        "math.sweep.normalize",
        "math.arithmetic.cancellation",
        "math.arithmetic.halfway_rounding",
        "math.arithmetic.overflow",
        "math.arithmetic.underflow",
        "math.arithmetic.fma_witness"
    ])
}

pub(super) fn math_probe_horizon_schema() -> Value {
    json!({
        "oneOf": [
            closed_record(&json!({ "kind": { "const": "operation" } }), &["kind"]),
            closed_record(
                &json!({ "kind": { "const": "scenario_steps" }, "steps": { "maximum": 32, "minimum": 1, "type": "integer" } }),
                &["kind", "steps"],
            )
        ]
    })
}

pub(super) fn math_probe_input_schema() -> Value {
    let float = float_bits_schema();
    let vec2 = schema_ref("vec2_bits");
    let vec3 = schema_ref("vec3_bits");
    let mat22 = schema_ref("mat22_bits");
    let mat33 = schema_ref("mat33_bits");
    let transform = schema_ref("transform_bits");
    let sweep = schema_ref("sweep_bits");
    json!({
        "oneOf": [
            tagged_probe_input("scalar", &json!({ "value_bits": float }), &["value_bits"]),
            tagged_probe_input("binary", &json!({ "a_bits": float, "b_bits": float }), &["a_bits", "b_bits"]),
            tagged_probe_input("clamp", &json!({ "value_bits": float, "low_bits": float, "high_bits": float }), &["value_bits", "low_bits", "high_bits"]),
            tagged_probe_input("vector2", &json!({ "vector": vec2 }), &["vector"]),
            tagged_probe_input("vector_pair", &json!({ "a": vec2, "b": vec2 }), &["a", "b"]),
            tagged_probe_input("mat22_solve", &json!({ "matrix": mat22, "right": vec2 }), &["matrix", "right"]),
            tagged_probe_input("mat33_solve", &json!({ "matrix": mat33, "right": vec3 }), &["matrix", "right"]),
            tagged_probe_input("mat22", &json!({ "matrix": mat22 }), &["matrix"]),
            tagged_probe_input("mat33", &json!({ "matrix": mat33 }), &["matrix"]),
            tagged_probe_input("rotation", &json!({ "angle_bits": float }), &["angle_bits"]),
            tagged_probe_input("transform", &json!({ "left": transform, "right": transform, "point": vec2 }), &["left", "right", "point"]),
            tagged_probe_input("sweep_transform", &json!({ "sweep": sweep, "fraction_bits": float }), &["sweep", "fraction_bits"]),
            tagged_probe_input("sweep_advance", &json!({ "sweep": sweep, "fractions_bits": { "items": float, "maxItems": 32, "minItems": 1, "type": "array" } }), &["sweep", "fractions_bits"]),
            tagged_probe_input("sweep", &json!({ "sweep": sweep }), &["sweep"]),
            tagged_probe_input("cancellation", &json!({ "large_bits": float, "opposite_bits": float, "tail_bits": float }), &["large_bits", "opposite_bits", "tail_bits"]),
            tagged_probe_input("halfway_rounding", &json!({ "even_bits": float, "odd_bits": float, "half_ulp_bits": float }), &["even_bits", "odd_bits", "half_ulp_bits"]),
            tagged_probe_input("scale", &json!({ "value_bits": float, "factor_bits": float }), &["value_bits", "factor_bits"]),
            tagged_probe_input("fma_witness", &json!({ "a_bits": float, "b_bits": float, "c_bits": float }), &["a_bits", "b_bits", "c_bits"])
        ]
    })
}

pub(super) fn scenario_definitions() -> Value {
    json!({
        "collision_probe_result": collision_probe_result_schema(),
        "mat22_bits": mat22_bits_schema(),
        "mat33_bits": mat33_bits_schema(),
        "math_probe_horizon": math_probe_horizon_schema(),
        "rigid_transform_bits": transform_bits_schema(),
        "rigid_vec2_bits": vec2_bits_schema(),
        "sweep_bits": sweep_bits_schema(),
        "transform_bits": transform_bits_schema(),
        "vec2_bits": vec2_bits_schema(),
        "vec3_bits": vec3_bits_schema()
    })
}

pub(super) fn collision_probe_result_schema() -> Value {
    let accepted = closed_record(
        &json!({
            "kind": { "const": "accepted" },
            "numeric": {
                "items": closed_record(
                    &json!({ "field": bounded_string_schema(), "bits": float_bits_schema() }),
                    &["field", "bits"],
                ),
                "maxItems": 128,
                "type": "array"
            },
            "discrete": {
                "items": closed_record(
                    &json!({ "field": bounded_string_schema(), "value": bounded_string_schema() }),
                    &["field", "value"],
                ),
                "maxItems": 128,
                "type": "array"
            },
            "payload_ids": { "items": uint32_schema(), "maxItems": 128, "type": "array" }
        }),
        &["kind", "numeric", "discrete", "payload_ids"],
    );
    let rejected = closed_record(
        &json!({
            "kind": { "const": "rejected" },
            "category": { "enum": ["non_finite_value", "invalid_geometry", "invalid_child_index"] },
            "field": { "enum": ["circle_center", "circle_radius", "edge_start", "edge_end", "edge_previous", "edge_next", "polygon_vertices", "chain_vertices", "child_index"] }
        }),
        &["kind", "category", "field"],
    );
    closed_record(
        &json!({
            "case_id": semantic_id_schema(),
            "operation": { "enum": collision_probe_operations() },
            "policy_path": { "enum": collision_probe_policy_paths() },
            "horizon": collision_probe_horizon_schema(),
            "collection_policy": { "enum": ["ordered", "set"] },
            "outcome": { "oneOf": [accepted, rejected] }
        }),
        &[
            "case_id",
            "operation",
            "policy_path",
            "horizon",
            "collection_policy",
            "outcome",
        ],
    )
}
