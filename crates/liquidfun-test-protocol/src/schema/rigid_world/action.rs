use super::*;

pub(super) fn action_record_schema() -> Value {
    closed_record(
        &json!({
            "action_id": semantic_id_schema(),
            "phase": bounded_string_schema(),
            "action": rigid_world_action_schema()
        }),
        &["action_id", "phase", "action"],
    )
}

pub(super) fn rigid_world_action_schema() -> Value {
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
            tagged_probe_input("create_joint", &json!({ "joint_id": semantic_id_schema() }), &["joint_id"]),
            tagged_probe_input("inspect_joint", &json!({ "joint_id": semantic_id_schema() }), &["joint_id"]),
            tagged_probe_input("mutate_joint", &json!({ "joint_id": semantic_id_schema(), "mutation": joint_mutation_schema() }), &["joint_id", "mutation"]),
            tagged_probe_input("destroy_joint", &json!({ "joint_id": semantic_id_schema() }), &["joint_id"]),
            tagged_probe_input("create_rope", &json!({ "rope_id": semantic_id_schema() }), &["rope_id"]),
            tagged_probe_input("set_rope_angle", &json!({ "rope_id": semantic_id_schema(), "angle_bits": float_bits_schema() }), &["rope_id", "angle_bits"]),
            tagged_probe_input("step_rope", &json!({ "rope_id": semantic_id_schema(), "timestep_bits": float_bits_schema(), "iterations": { "maximum": RIGID_WORLD_MAXIMUM_ITERATIONS, "minimum": 1, "type": "integer" } }), &["rope_id", "timestep_bits", "iterations"]),
            tagged_probe_input("inspect_rope", &json!({ "rope_id": semantic_id_schema() }), &["rope_id"]),
            tagged_probe_input("destroy_rope", &json!({ "rope_id": semantic_id_schema() }), &["rope_id"]),
            tagged_probe_input("set_contact_filter_directive", &json!({ "target": contact_directive_target_schema(), "should_collide": { "type": "boolean" } }), &["target", "should_collide"]),
            tagged_probe_input("set_pre_solve_directive", &json!({ "target": contact_directive_target_schema(), "directive": pre_solve_directive_schema() }), &["target", "directive"]),
            tagged_probe_input("request_reconstruction", &json!({}), &[]),
            tagged_probe_input("request_diagnostics", &json!({}), &[]),
            tagged_probe_input("particle", &json!({ "action": particle_action_schema() }), &["action"]),
            tagged_probe_input("particle_group", &json!({ "operation": phase10_operation_schema() }), &["operation"]),
            tagged_probe_input("destroy_fixture", &fixture_id(), &["fixture_id"]),
            tagged_probe_input("destroy_body", &body_id(), &["body_id"])
        ]
    })
}

pub(super) fn expected_checkpoint_schema() -> Value {
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

pub(super) fn expected_counts_schema() -> Value {
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

pub(super) fn contact_identity_schema() -> Value {
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
