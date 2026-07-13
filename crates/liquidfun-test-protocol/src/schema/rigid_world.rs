use serde::Serialize;
use serde_json::{Value, json};

use super::{
    bounded_string_schema, closed_record, float_bits_schema, scenario_source_schema, schema_ref,
    semantic_id_schema, sha256_schema, tagged_probe_input, transform_bits_schema, uint32_schema,
    vec2_bits_schema, version_schema,
};
use crate::{
    RIGID_WORLD_MAXIMUM_ACTIONS, RIGID_WORLD_MAXIMUM_CONTINUOUS_WORK,
    RIGID_WORLD_MAXIMUM_DIRECTIVES, RIGID_WORLD_MAXIMUM_ITERATIONS,
    RIGID_WORLD_POSITION_ITERATIONS, RIGID_WORLD_TIMESTEP_BITS, RIGID_WORLD_VELOCITY_ITERATIONS,
    RigidBodyKind, RigidContactEventKind, RigidFeatureKind, RigidManifoldKind,
    RigidPartialProgressClassification, RigidQueryCompletion, RigidQueryDirective,
    RigidRayCompletion, RigidStepCompletion, RigidWakePolicy, RigidWorldWitness,
    RigidWorldWitnessFamily,
};

pub(super) fn rigid_world_request_schema() -> Value {
    closed_record(
        &json!({
            "protocol_version": version_schema(),
            "record_kind": { "const": "rigid_world_request" },
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

pub(super) fn rigid_world_scenario_schema() -> Value {
    closed_record(
        &json!({
            "scenario_id": semantic_id_schema(),
            "source": scenario_source_schema(),
            "timelines": {
                "items": rigid_world_timeline_schema(),
                "maxItems": 9,
                "minItems": 1,
                "type": "array"
            }
        }),
        &["scenario_id", "source", "timelines"],
    )
}

pub(super) fn rigid_world_trace_definitions() -> Value {
    json!({
        "rigid_transform_bits": transform_bits_schema(),
        "rigid_vec2_bits": vec2_bits_schema(),
        "vec2_bits": vec2_bits_schema()
    })
}

pub(super) fn rigid_world_result_schema() -> Value {
    closed_record(
        &json!({
            "protocol_version": version_schema(),
            "record_kind": { "const": "rigid_world_result" },
            "request_id": semantic_id_schema(),
            "trace_schema_version": version_schema(),
            "scenario_id": semantic_id_schema(),
            "timelines": {
                "items": rigid_world_timeline_result_schema(),
                "maxItems": 9,
                "minItems": 1,
                "type": "array"
            }
        }),
        &[
            "protocol_version",
            "record_kind",
            "request_id",
            "trace_schema_version",
            "scenario_id",
            "timelines",
        ],
    )
}

fn rigid_world_timeline_schema() -> Value {
    closed_record(
        &json!({
            "witness_family": { "enum": witness_families() },
            "bodies": {
                "items": body_declaration_schema(),
                "maxItems": 64,
                "minItems": 1,
                "type": "array"
            },
            "fixtures": {
                "items": fixture_declaration_schema(),
                "maxItems": 128,
                "minItems": 1,
                "type": "array"
            },
            "actions": {
                "items": action_record_schema(),
                "maxItems": RIGID_WORLD_MAXIMUM_ACTIONS,
                "minItems": 1,
                "type": "array"
            },
            "checkpoints": {
                "items": expected_checkpoint_schema(),
                "maxItems": 64,
                "minItems": 1,
                "type": "array"
            }
        }),
        &[
            "witness_family",
            "bodies",
            "fixtures",
            "actions",
            "checkpoints",
        ],
    )
}

fn body_declaration_schema() -> Value {
    closed_record(
        &json!({
            "body_id": semantic_id_schema(),
            "body_kind": { "enum": body_kinds() },
            "transform": schema_ref("transform_bits"),
            "active": { "type": "boolean" }
        }),
        &["body_id", "body_kind", "transform", "active"],
    )
}

fn fixture_declaration_schema() -> Value {
    closed_record(
        &json!({
            "fixture_id": semantic_id_schema(),
            "owner_body_id": semantic_id_schema(),
            "shape": fixture_shape_schema(),
            "density_bits": float_bits_schema(),
            "friction_bits": float_bits_schema(),
            "restitution_bits": float_bits_schema(),
            "sensor": { "type": "boolean" },
            "filter": filter_schema()
        }),
        &[
            "fixture_id",
            "owner_body_id",
            "shape",
            "density_bits",
            "friction_bits",
            "restitution_bits",
            "sensor",
            "filter",
        ],
    )
}

fn fixture_shape_schema() -> Value {
    json!({
        "oneOf": [
            tagged_probe_input(
                "circle",
                &json!({
                    "center": schema_ref("vec2_bits"),
                    "radius_bits": float_bits_schema()
                }),
                &["center", "radius_bits"],
            ),
            tagged_probe_input(
                "polygon",
                &json!({
                    "vertices": {
                        "items": schema_ref("vec2_bits"),
                        "maxItems": 8,
                        "minItems": 3,
                        "type": "array"
                    }
                }),
                &["vertices"],
            )
        ]
    })
}

fn filter_schema() -> Value {
    closed_record(
        &json!({
            "category_bits": { "maximum": u16::MAX, "minimum": 0, "type": "integer" },
            "mask_bits": { "maximum": u16::MAX, "minimum": 0, "type": "integer" },
            "group_index": { "maximum": i16::MAX, "minimum": i16::MIN, "type": "integer" }
        }),
        &["category_bits", "mask_bits", "group_index"],
    )
}

fn action_record_schema() -> Value {
    closed_record(
        &json!({
            "action_id": semantic_id_schema(),
            "phase": bounded_string_schema(),
            "action": rigid_world_action_schema()
        }),
        &["action_id", "phase", "action"],
    )
}

fn rigid_world_action_schema() -> Value {
    let body_id = || json!({ "body_id": semantic_id_schema() });
    let fixture_id = || json!({ "fixture_id": semantic_id_schema() });
    json!({
        "oneOf": [
            tagged_probe_input("create_body", &body_id(), &["body_id"]),
            tagged_probe_input("create_fixture", &fixture_id(), &["fixture_id"]),
            tagged_probe_input("inspect_body", &body_id(), &["body_id"]),
            tagged_probe_input("inspect_fixture", &fixture_id(), &["fixture_id"]),
            tagged_probe_input("set_body_transform", &json!({ "body_id": semantic_id_schema(), "transform": schema_ref("transform_bits") }), &["body_id", "transform"]),
            tagged_probe_input("set_body_type", &json!({ "body_id": semantic_id_schema(), "body_kind": { "enum": body_kinds() } }), &["body_id", "body_kind"]),
            tagged_probe_input("set_body_active", &json!({ "body_id": semantic_id_schema(), "active": { "type": "boolean" } }), &["body_id", "active"]),
            tagged_probe_input("set_linear_velocity", &json!({ "body_id": semantic_id_schema(), "velocity": schema_ref("vec2_bits") }), &["body_id", "velocity"]),
            tagged_probe_input("set_angular_velocity", &json!({ "body_id": semantic_id_schema(), "angular_velocity_bits": float_bits_schema() }), &["body_id", "angular_velocity_bits"]),
            tagged_probe_input("apply_force", &json!({ "body_id": semantic_id_schema(), "force": schema_ref("vec2_bits"), "point": schema_ref("vec2_bits"), "wake_policy": wake_policy_schema() }), &["body_id", "force", "point", "wake_policy"]),
            tagged_probe_input("apply_torque", &json!({ "body_id": semantic_id_schema(), "torque_bits": float_bits_schema(), "wake_policy": wake_policy_schema() }), &["body_id", "torque_bits", "wake_policy"]),
            tagged_probe_input("apply_linear_impulse", &json!({ "body_id": semantic_id_schema(), "impulse": schema_ref("vec2_bits"), "point": schema_ref("vec2_bits"), "wake_policy": wake_policy_schema() }), &["body_id", "impulse", "point", "wake_policy"]),
            tagged_probe_input("apply_angular_impulse", &json!({ "body_id": semantic_id_schema(), "impulse_bits": float_bits_schema(), "wake_policy": wake_policy_schema() }), &["body_id", "impulse_bits", "wake_policy"]),
            tagged_probe_input("set_body_damping", &json!({ "body_id": semantic_id_schema(), "linear_damping_bits": float_bits_schema(), "angular_damping_bits": float_bits_schema() }), &["body_id", "linear_damping_bits", "angular_damping_bits"]),
            tagged_probe_input("set_gravity_scale", &json!({ "body_id": semantic_id_schema(), "gravity_scale_bits": float_bits_schema() }), &["body_id", "gravity_scale_bits"]),
            tagged_probe_input("set_fixed_rotation", &json!({ "body_id": semantic_id_schema(), "fixed_rotation": { "type": "boolean" } }), &["body_id", "fixed_rotation"]),
            tagged_probe_input("set_sleeping_allowed", &json!({ "body_id": semantic_id_schema(), "sleeping_allowed": { "type": "boolean" } }), &["body_id", "sleeping_allowed"]),
            tagged_probe_input("set_awake", &json!({ "body_id": semantic_id_schema(), "awake": { "type": "boolean" } }), &["body_id", "awake"]),
            tagged_probe_input("set_bullet", &json!({ "body_id": semantic_id_schema(), "bullet": { "type": "boolean" } }), &["body_id", "bullet"]),
            tagged_probe_input("set_fixture_sensor", &json!({ "fixture_id": semantic_id_schema(), "sensor": { "type": "boolean" } }), &["fixture_id", "sensor"]),
            tagged_probe_input("set_fixture_material", &json!({ "fixture_id": semantic_id_schema(), "friction_bits": float_bits_schema(), "restitution_bits": float_bits_schema() }), &["fixture_id", "friction_bits", "restitution_bits"]),
            tagged_probe_input("set_fixture_filter", &json!({ "fixture_id": semantic_id_schema(), "filter": filter_schema() }), &["fixture_id", "filter"]),
            tagged_probe_input("set_fixture_density", &json!({ "fixture_id": semantic_id_schema(), "density_bits": float_bits_schema() }), &["fixture_id", "density_bits"]),
            tagged_probe_input("reset_mass_data", &body_id(), &["body_id"]),
            tagged_probe_input("set_custom_mass_data", &json!({ "body_id": semantic_id_schema(), "mass_bits": float_bits_schema(), "center": schema_ref("vec2_bits"), "inertia_bits": float_bits_schema() }), &["body_id", "mass_bits", "center", "inertia_bits"]),
            tagged_probe_input("step", &json!({ "timestep_bits": { "const": RIGID_WORLD_TIMESTEP_BITS }, "velocity_iterations": { "const": RIGID_WORLD_VELOCITY_ITERATIONS }, "position_iterations": { "const": RIGID_WORLD_POSITION_ITERATIONS } }), &["timestep_bits", "velocity_iterations", "position_iterations"]),
            tagged_probe_input("set_world_gravity", &json!({ "gravity": schema_ref("vec2_bits") }), &["gravity"]),
            tagged_probe_input("set_automatic_force_clearing", &json!({ "enabled": { "type": "boolean" } }), &["enabled"]),
            tagged_probe_input("set_warm_starting", &json!({ "enabled": { "type": "boolean" } }), &["enabled"]),
            tagged_probe_input("set_continuous_physics", &json!({ "enabled": { "type": "boolean" } }), &["enabled"]),
            tagged_probe_input("set_sub_stepping", &json!({ "enabled": { "type": "boolean" } }), &["enabled"]),
            tagged_probe_input("clear_forces", &json!({}), &[]),
            tagged_probe_input("configured_step", &json!({
                "timestep_bits": float_bits_schema(),
                "velocity_iterations": { "maximum": RIGID_WORLD_MAXIMUM_ITERATIONS, "minimum": 1, "type": "integer" },
                "position_iterations": { "maximum": RIGID_WORLD_MAXIMUM_ITERATIONS, "minimum": 1, "type": "integer" },
                "continuous_work_budget": { "maximum": RIGID_WORLD_MAXIMUM_CONTINUOUS_WORK, "minimum": 1, "type": "integer" }
            }), &["timestep_bits", "velocity_iterations", "position_iterations", "continuous_work_budget"]),
            tagged_probe_input("query_aabb", &json!({
                "aabb": aabb_schema(),
                "directive_rules": { "items": query_directive_rule_schema(), "maxItems": RIGID_WORLD_MAXIMUM_DIRECTIVES, "type": "array" }
            }), &["aabb", "directive_rules"]),
            tagged_probe_input("ray_cast", &json!({
                "start": schema_ref("vec2_bits"),
                "end": schema_ref("vec2_bits"),
                "directive_rules": { "items": ray_directive_rule_schema(), "maxItems": RIGID_WORLD_MAXIMUM_DIRECTIVES, "type": "array" }
            }), &["start", "end", "directive_rules"]),
            tagged_probe_input("shift_origin", &json!({ "shift": schema_ref("vec2_bits") }), &["shift"]),
            tagged_probe_input("destroy_fixture", &fixture_id(), &["fixture_id"]),
            tagged_probe_input("destroy_body", &body_id(), &["body_id"])
        ]
    })
}

fn expected_checkpoint_schema() -> Value {
    closed_record(
        &json!({
            "checkpoint_id": semantic_id_schema(),
            "after_action_id": semantic_id_schema(),
            "phase": bounded_string_schema(),
            "counts": expected_counts_schema(),
            "transitions": {
                "items": closed_record(
                    &json!({
                        "witness": { "enum": witnesses() },
                        "maybe_contact": {
                            "oneOf": [contact_identity_schema(), { "type": "null" }]
                        }
                    }),
                    &["witness", "maybe_contact"],
                ),
                "maxItems": 64,
                "type": "array"
            }
        }),
        &[
            "checkpoint_id",
            "after_action_id",
            "phase",
            "counts",
            "transitions",
        ],
    )
}

fn expected_counts_schema() -> Value {
    closed_record(
        &json!({
            "bodies": uint32_schema(),
            "fixtures": uint32_schema(),
            "contacts": uint32_schema(),
            "manifold_points": uint32_schema(),
            "events": uint32_schema(),
            "destructions": uint32_schema()
        }),
        &[
            "bodies",
            "fixtures",
            "contacts",
            "manifold_points",
            "events",
            "destructions",
        ],
    )
}

fn contact_identity_schema() -> Value {
    closed_record(
        &json!({
            "fixture_a_id": semantic_id_schema(),
            "child_a": uint32_schema(),
            "fixture_b_id": semantic_id_schema(),
            "child_b": uint32_schema(),
            "occurrence": { "maximum": u32::MAX, "minimum": 1, "type": "integer" }
        }),
        &[
            "fixture_a_id",
            "child_a",
            "fixture_b_id",
            "child_b",
            "occurrence",
        ],
    )
}

fn rigid_world_timeline_result_schema() -> Value {
    closed_record(
        &json!({
            "witness_family": { "enum": witness_families() },
            "checkpoints": {
                "items": checkpoint_result_schema(),
                "maxItems": 64,
                "minItems": 1,
                "type": "array"
            }
        }),
        &["witness_family", "checkpoints"],
    )
}

fn checkpoint_result_schema() -> Value {
    closed_record(
        &json!({
            "checkpoint_id": semantic_id_schema(),
            "phase": bounded_string_schema(),
            "counts": expected_counts_schema(),
            "bodies": { "items": body_snapshot_schema(), "maxItems": 64, "type": "array" },
            "fixtures": { "items": fixture_snapshot_schema(), "maxItems": 128, "type": "array" },
            "contacts": { "items": contact_result_schema(), "maxItems": 128, "type": "array" },
            "events": { "items": event_schema(), "maxItems": 256, "type": "array" },
            "destructions": { "items": destruction_schema(), "maxItems": 256, "type": "array" },
            "observations": { "items": world_observation_schema(), "maxItems": 256, "type": "array" }
        }),
        &[
            "checkpoint_id",
            "phase",
            "counts",
            "bodies",
            "fixtures",
            "contacts",
            "events",
            "destructions",
        ],
    )
}

fn wake_policy_schema() -> Value {
    json!({ "enum": enum_values(&[RigidWakePolicy::Wake, RigidWakePolicy::PreserveSleep]) })
}

fn aabb_schema() -> Value {
    closed_record(
        &json!({
            "lower": schema_ref("vec2_bits"),
            "upper": schema_ref("vec2_bits")
        }),
        &["lower", "upper"],
    )
}

fn fixture_child_selector_schema() -> Value {
    closed_record(
        &json!({
            "fixture_id": semantic_id_schema(),
            "child_index": uint32_schema()
        }),
        &["fixture_id", "child_index"],
    )
}

fn query_directive_rule_schema() -> Value {
    closed_record(
        &json!({
            "target": fixture_child_selector_schema(),
            "directive": { "enum": enum_values(&[RigidQueryDirective::Continue, RigidQueryDirective::Terminate]) }
        }),
        &["target", "directive"],
    )
}

fn ray_directive_rule_schema() -> Value {
    closed_record(
        &json!({
            "target": fixture_child_selector_schema(),
            "directive": {
                "oneOf": [
                    tagged_probe_input("ignore", &json!({}), &[]),
                    tagged_probe_input("terminate", &json!({}), &[]),
                    tagged_probe_input("continue", &json!({}), &[]),
                    tagged_probe_input("clip", &json!({ "fraction_bits": float_bits_schema() }), &["fraction_bits"])
                ]
            }
        }),
        &["target", "directive"],
    )
}

fn world_observation_schema() -> Value {
    json!({
        "oneOf": [
            tagged_probe_input("body_state", &json!({ "state": body_control_snapshot_schema() }), &["state"]),
            tagged_probe_input("step", &json!({ "outcome": step_outcome_schema() }), &["outcome"]),
            tagged_probe_input("query", &json!({ "observation": query_observation_schema() }), &["observation"]),
            tagged_probe_input("ray_cast", &json!({ "observation": ray_observation_schema() }), &["observation"]),
            tagged_probe_input("origin_shift", &json!({ "shift": schema_ref("rigid_vec2_bits") }), &["shift"])
        ]
    })
}

fn body_control_snapshot_schema() -> Value {
    closed_record(
        &json!({
            "body_id": semantic_id_schema(),
            "linear_velocity": schema_ref("rigid_vec2_bits"),
            "angular_velocity_bits": float_bits_schema(),
            "awake": { "type": "boolean" },
            "bullet": { "type": "boolean" },
            "sleeping_allowed": { "type": "boolean" },
            "fixed_rotation": { "type": "boolean" },
            "linear_damping_bits": float_bits_schema(),
            "angular_damping_bits": float_bits_schema(),
            "gravity_scale_bits": float_bits_schema()
        }),
        &[
            "body_id",
            "linear_velocity",
            "angular_velocity_bits",
            "awake",
            "bullet",
            "sleeping_allowed",
            "fixed_rotation",
            "linear_damping_bits",
            "angular_damping_bits",
            "gravity_scale_bits",
        ],
    )
}

fn step_outcome_schema() -> Value {
    json!({
        "oneOf": [
            tagged_probe_input("completed", &json!({
                "completion": { "enum": enum_values(&[RigidStepCompletion::Complete, RigidStepCompletion::ContinuousPending]) }
            }), &["completion"]),
            tagged_probe_input("partial", &json!({
                "classification": { "enum": enum_values(&[RigidPartialProgressClassification::ContinuousWorkBudgetExhausted]) }
            }), &["classification"])
        ]
    })
}

fn query_observation_schema() -> Value {
    closed_record(
        &json!({
            "completion": { "enum": enum_values(&[RigidQueryCompletion::Exhausted, RigidQueryCompletion::Terminated]) },
            "occurrences": { "items": fixture_child_selector_schema(), "maxItems": 256, "type": "array" }
        }),
        &["completion", "occurrences"],
    )
}

fn ray_observation_schema() -> Value {
    closed_record(
        &json!({
            "completion": { "enum": enum_values(&[RigidRayCompletion::Exhausted, RigidRayCompletion::Terminated]) },
            "final_max_fraction_bits": float_bits_schema(),
            "hits": { "items": ray_hit_schema(), "maxItems": 256, "type": "array" }
        }),
        &["completion", "final_max_fraction_bits", "hits"],
    )
}

fn ray_hit_schema() -> Value {
    closed_record(
        &json!({
            "fixture_id": semantic_id_schema(),
            "child_index": uint32_schema(),
            "point": schema_ref("rigid_vec2_bits"),
            "normal": schema_ref("rigid_vec2_bits"),
            "fraction_bits": float_bits_schema()
        }),
        &[
            "fixture_id",
            "child_index",
            "point",
            "normal",
            "fraction_bits",
        ],
    )
}

fn body_snapshot_schema() -> Value {
    closed_record(
        &json!({
            "body_id": semantic_id_schema(),
            "body_kind": { "enum": body_kinds() },
            "transform": schema_ref("rigid_transform_bits"),
            "active": { "type": "boolean" },
            "linear_velocity": schema_ref("rigid_vec2_bits"),
            "angular_velocity_bits": float_bits_schema(),
            "mass_bits": float_bits_schema(),
            "local_center": schema_ref("rigid_vec2_bits"),
            "inertia_bits": float_bits_schema()
        }),
        &[
            "body_id",
            "body_kind",
            "transform",
            "active",
            "linear_velocity",
            "angular_velocity_bits",
            "mass_bits",
            "local_center",
            "inertia_bits",
        ],
    )
}

fn fixture_snapshot_schema() -> Value {
    closed_record(
        &json!({
            "fixture_id": semantic_id_schema(),
            "owner_body_id": semantic_id_schema(),
            "sensor": { "type": "boolean" },
            "density_bits": float_bits_schema(),
            "friction_bits": float_bits_schema(),
            "restitution_bits": float_bits_schema(),
            "filter": filter_schema()
        }),
        &[
            "fixture_id",
            "owner_body_id",
            "sensor",
            "density_bits",
            "friction_bits",
            "restitution_bits",
            "filter",
        ],
    )
}

fn contact_result_schema() -> Value {
    closed_record(
        &json!({
            "identity": contact_identity_schema(),
            "touching": { "type": "boolean" },
            "enabled": { "type": "boolean" },
            "sensor": { "type": "boolean" },
            "mixed_friction_bits": float_bits_schema(),
            "mixed_restitution_bits": float_bits_schema(),
            "maybe_manifold": { "oneOf": [manifold_schema(), { "type": "null" }] }
        }),
        &[
            "identity",
            "touching",
            "enabled",
            "sensor",
            "mixed_friction_bits",
            "mixed_restitution_bits",
            "maybe_manifold",
        ],
    )
}

fn manifold_schema() -> Value {
    closed_record(
        &json!({
            "manifold_kind": { "enum": enum_values(&[RigidManifoldKind::Circles, RigidManifoldKind::FaceA, RigidManifoldKind::FaceB]) },
            "local_normal": schema_ref("rigid_vec2_bits"),
            "local_point": schema_ref("rigid_vec2_bits"),
            "points": { "items": manifold_point_schema(), "maxItems": 2, "type": "array" }
        }),
        &["manifold_kind", "local_normal", "local_point", "points"],
    )
}

fn manifold_point_schema() -> Value {
    closed_record(
        &json!({
            "point": schema_ref("rigid_vec2_bits"),
            "feature": closed_record(
                &json!({
                    "index_a": { "maximum": u8::MAX, "minimum": 0, "type": "integer" },
                    "index_b": { "maximum": u8::MAX, "minimum": 0, "type": "integer" },
                    "kind_a": { "enum": enum_values(&[RigidFeatureKind::Vertex, RigidFeatureKind::Face]) },
                    "kind_b": { "enum": enum_values(&[RigidFeatureKind::Vertex, RigidFeatureKind::Face]) }
                }),
                &["index_a", "index_b", "kind_a", "kind_b"],
            ),
            "normal_impulse_bits": float_bits_schema(),
            "tangent_impulse_bits": float_bits_schema()
        }),
        &[
            "point",
            "feature",
            "normal_impulse_bits",
            "tangent_impulse_bits",
        ],
    )
}

fn event_schema() -> Value {
    closed_record(
        &json!({
            "kind": { "enum": enum_values(&[
                RigidContactEventKind::Created,
                RigidContactEventKind::Begin,
                RigidContactEventKind::Persist,
                RigidContactEventKind::End,
                RigidContactEventKind::PreSolve,
                RigidContactEventKind::PostSolve,
                RigidContactEventKind::Destroyed,
            ]) },
            "contact": contact_identity_schema()
        }),
        &["kind", "contact"],
    )
}

fn destruction_schema() -> Value {
    json!({
        "oneOf": [
            tagged_probe_input("contact", &json!({ "contact": contact_identity_schema() }), &["contact"]),
            tagged_probe_input("fixture", &json!({ "fixture_id": semantic_id_schema() }), &["fixture_id"]),
            tagged_probe_input("body", &json!({ "body_id": semantic_id_schema() }), &["body_id"])
        ]
    })
}

fn witness_families() -> Value {
    enum_values(&RigidWorldWitnessFamily::ALL)
}

fn witnesses() -> Value {
    let witnesses = RigidWorldWitnessFamily::ALL
        .into_iter()
        .flat_map(RigidWorldWitnessFamily::required_witnesses)
        .copied()
        .collect::<Vec<RigidWorldWitness>>();
    enum_values(&witnesses)
}

fn body_kinds() -> Value {
    enum_values(&[
        RigidBodyKind::Static,
        RigidBodyKind::Kinematic,
        RigidBodyKind::Dynamic,
    ])
}

fn enum_values<T: Serialize>(values: &[T]) -> Value {
    serde_json::to_value(values).expect("closed protocol enum serialization cannot fail")
}
