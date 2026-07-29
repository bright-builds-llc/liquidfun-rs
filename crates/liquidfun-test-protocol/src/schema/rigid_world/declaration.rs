use super::*;

pub(super) fn rigid_world_timeline_schema() -> Value {
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
            "joints": {
                "items": joint_declaration_schema(),
                "maxItems": RIGID_WORLD_MAXIMUM_JOINTS,
                "type": "array"
            },
            "ropes": {
                "items": rope_declaration_schema(),
                "maxItems": RIGID_WORLD_MAXIMUM_ROPES,
                "type": "array"
            },
            "particle_systems": {
                "items": particle_system_declaration_schema(),
                "maxItems": crate::PHASE9_MAXIMUM_PARTICLE_SYSTEMS,
                "type": "array"
            },
            "particles": {
                "items": particle_declaration_schema(),
                "maxItems": crate::PHASE9_MAXIMUM_PARTICLES,
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

pub(super) fn body_declaration_schema() -> Value {
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

pub(super) fn fixture_declaration_schema() -> Value {
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

pub(super) fn fixture_shape_schema() -> Value {
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

pub(super) fn joint_declaration_schema() -> Value {
    closed_record(
        &json!({
            "joint_id": semantic_id_schema(),
            "body_a_id": semantic_id_schema(),
            "body_b_id": semantic_id_schema(),
            "collide_connected": { "type": "boolean" },
            "definition": joint_definition_schema()
        }),
        &[
            "joint_id",
            "body_a_id",
            "body_b_id",
            "collide_connected",
            "definition",
        ],
    )
}

#[allow(
    clippy::too_many_lines,
    reason = "the closed eleven-kind schema is audited exhaustively"
)]
pub(super) fn joint_definition_schema() -> Value {
    let anchors = || {
        json!({
            "local_anchor_a": schema_ref("vec2_bits"),
            "local_anchor_b": schema_ref("vec2_bits")
        })
    };
    let mut revolute = anchors();
    extend_properties(
        &mut revolute,
        &json!({
            "reference_angle_bits": float_bits_schema(),
            "lower_angle_bits": float_bits_schema(),
            "upper_angle_bits": float_bits_schema(),
            "motor_speed_bits": float_bits_schema(),
            "max_motor_torque_bits": float_bits_schema(),
            "limit_enabled": { "type": "boolean" },
            "motor_enabled": { "type": "boolean" }
        }),
    );
    let mut prismatic = anchors();
    extend_properties(
        &mut prismatic,
        &json!({
            "local_axis_a": schema_ref("vec2_bits"),
            "reference_angle_bits": float_bits_schema(),
            "lower_translation_bits": float_bits_schema(),
            "upper_translation_bits": float_bits_schema(),
            "motor_speed_bits": float_bits_schema(),
            "max_motor_force_bits": float_bits_schema(),
            "limit_enabled": { "type": "boolean" },
            "motor_enabled": { "type": "boolean" }
        }),
    );
    let mut distance = anchors();
    extend_properties(
        &mut distance,
        &json!({
            "length_bits": float_bits_schema(),
            "frequency_bits": float_bits_schema(),
            "damping_ratio_bits": float_bits_schema()
        }),
    );
    let mut wheel = anchors();
    extend_properties(
        &mut wheel,
        &json!({
            "local_axis_a": schema_ref("vec2_bits"),
            "motor_speed_bits": float_bits_schema(),
            "max_motor_torque_bits": float_bits_schema(),
            "frequency_bits": float_bits_schema(),
            "damping_ratio_bits": float_bits_schema(),
            "motor_enabled": { "type": "boolean" }
        }),
    );
    let mut weld = anchors();
    extend_properties(
        &mut weld,
        &json!({
            "reference_angle_bits": float_bits_schema(),
            "frequency_bits": float_bits_schema(),
            "damping_ratio_bits": float_bits_schema()
        }),
    );
    let mut friction = anchors();
    extend_properties(
        &mut friction,
        &json!({
            "max_force_bits": float_bits_schema(),
            "max_torque_bits": float_bits_schema()
        }),
    );
    let mut rope = anchors();
    extend_properties(
        &mut rope,
        &json!({ "max_length_bits": float_bits_schema() }),
    );
    json!({ "oneOf": [
        tagged_probe_input("revolute", &revolute, &["local_anchor_a", "local_anchor_b", "reference_angle_bits", "lower_angle_bits", "upper_angle_bits", "motor_speed_bits", "max_motor_torque_bits", "limit_enabled", "motor_enabled"]),
        tagged_probe_input("prismatic", &prismatic, &["local_anchor_a", "local_anchor_b", "local_axis_a", "reference_angle_bits", "lower_translation_bits", "upper_translation_bits", "motor_speed_bits", "max_motor_force_bits", "limit_enabled", "motor_enabled"]),
        tagged_probe_input("distance", &distance, &["local_anchor_a", "local_anchor_b", "length_bits", "frequency_bits", "damping_ratio_bits"]),
        tagged_probe_input("pulley", &json!({ "ground_anchor_a": schema_ref("vec2_bits"), "ground_anchor_b": schema_ref("vec2_bits"), "local_anchor_a": schema_ref("vec2_bits"), "local_anchor_b": schema_ref("vec2_bits"), "length_a_bits": float_bits_schema(), "length_b_bits": float_bits_schema(), "ratio_bits": float_bits_schema() }), &["ground_anchor_a", "ground_anchor_b", "local_anchor_a", "local_anchor_b", "length_a_bits", "length_b_bits", "ratio_bits"]),
        tagged_probe_input("mouse", &json!({ "target": schema_ref("vec2_bits"), "max_force_bits": float_bits_schema(), "frequency_bits": float_bits_schema(), "damping_ratio_bits": float_bits_schema() }), &["target", "max_force_bits", "frequency_bits", "damping_ratio_bits"]),
        tagged_probe_input("gear", &json!({ "joint_a_id": semantic_id_schema(), "joint_b_id": semantic_id_schema(), "ratio_bits": float_bits_schema() }), &["joint_a_id", "joint_b_id", "ratio_bits"]),
        tagged_probe_input("wheel", &wheel, &["local_anchor_a", "local_anchor_b", "local_axis_a", "motor_speed_bits", "max_motor_torque_bits", "frequency_bits", "damping_ratio_bits", "motor_enabled"]),
        tagged_probe_input("weld", &weld, &["local_anchor_a", "local_anchor_b", "reference_angle_bits", "frequency_bits", "damping_ratio_bits"]),
        tagged_probe_input("friction", &friction, &["local_anchor_a", "local_anchor_b", "max_force_bits", "max_torque_bits"]),
        tagged_probe_input("rope", &rope, &["local_anchor_a", "local_anchor_b", "max_length_bits"]),
        tagged_probe_input("motor", &json!({ "linear_offset": schema_ref("vec2_bits"), "angular_offset_bits": float_bits_schema(), "max_force_bits": float_bits_schema(), "max_torque_bits": float_bits_schema(), "correction_factor_bits": float_bits_schema() }), &["linear_offset", "angular_offset_bits", "max_force_bits", "max_torque_bits", "correction_factor_bits"])
    ] })
}

pub(super) fn extend_properties(target: &mut Value, additions: &Value) {
    let Some(target) = target.as_object_mut() else {
        unreachable!("schema properties are always objects");
    };
    let Some(additions) = additions.as_object() else {
        unreachable!("schema property additions are always objects");
    };
    target.extend(additions.clone());
}

pub(super) fn rope_declaration_schema() -> Value {
    closed_record(
        &json!({
            "rope_id": semantic_id_schema(),
            "vertices": { "items": schema_ref("vec2_bits"), "maxItems": RIGID_WORLD_MAXIMUM_ROPE_VERTICES, "minItems": 3, "type": "array" },
            "masses_bits": { "items": float_bits_schema(), "maxItems": RIGID_WORLD_MAXIMUM_ROPE_VERTICES, "minItems": 3, "type": "array" },
            "gravity": schema_ref("vec2_bits"),
            "damping_bits": float_bits_schema(),
            "stretch_stiffness_bits": float_bits_schema(),
            "bend_stiffness_bits": float_bits_schema()
        }),
        &[
            "rope_id",
            "vertices",
            "masses_bits",
            "gravity",
            "damping_bits",
            "stretch_stiffness_bits",
            "bend_stiffness_bits",
        ],
    )
}

pub(super) fn joint_mutation_schema() -> Value {
    json!({ "oneOf": [
        tagged_probe_input("limit_enabled", &json!({ "enabled": { "type": "boolean" } }), &["enabled"]),
        tagged_probe_input("limits", &json!({ "lower_bits": float_bits_schema(), "upper_bits": float_bits_schema() }), &["lower_bits", "upper_bits"]),
        tagged_probe_input("motor_enabled", &json!({ "enabled": { "type": "boolean" } }), &["enabled"]),
        tagged_probe_input("motor_speed", &json!({ "speed_bits": float_bits_schema() }), &["speed_bits"]),
        tagged_probe_input("max_motor_force", &json!({ "force_bits": float_bits_schema() }), &["force_bits"]),
        tagged_probe_input("max_motor_torque", &json!({ "torque_bits": float_bits_schema() }), &["torque_bits"]),
        tagged_probe_input("length", &json!({ "length_bits": float_bits_schema() }), &["length_bits"]),
        tagged_probe_input("frequency", &json!({ "frequency_bits": float_bits_schema() }), &["frequency_bits"]),
        tagged_probe_input("damping_ratio", &json!({ "damping_ratio_bits": float_bits_schema() }), &["damping_ratio_bits"]),
        tagged_probe_input("mouse_target", &json!({ "target": schema_ref("vec2_bits") }), &["target"]),
        tagged_probe_input("max_force", &json!({ "force_bits": float_bits_schema() }), &["force_bits"]),
        tagged_probe_input("max_torque", &json!({ "torque_bits": float_bits_schema() }), &["torque_bits"]),
        tagged_probe_input("gear_ratio", &json!({ "ratio_bits": float_bits_schema() }), &["ratio_bits"]),
        tagged_probe_input("rope_max_length", &json!({ "max_length_bits": float_bits_schema() }), &["max_length_bits"]),
        tagged_probe_input("linear_offset", &json!({ "offset": schema_ref("vec2_bits") }), &["offset"]),
        tagged_probe_input("angular_offset", &json!({ "offset_bits": float_bits_schema() }), &["offset_bits"]),
        tagged_probe_input("correction_factor", &json!({ "factor_bits": float_bits_schema() }), &["factor_bits"])
    ] })
}

pub(super) fn contact_directive_target_schema() -> Value {
    closed_record(
        &json!({ "fixture_a_id": semantic_id_schema(), "fixture_b_id": semantic_id_schema() }),
        &["fixture_a_id", "fixture_b_id"],
    )
}

pub(super) fn pre_solve_directive_schema() -> Value {
    closed_record(
        &json!({
            "enabled": { "type": "boolean" },
            "maybe_friction_bits": { "oneOf": [float_bits_schema(), { "type": "null" }] },
            "maybe_restitution_bits": { "oneOf": [float_bits_schema(), { "type": "null" }] },
            "maybe_tangent_speed_bits": { "oneOf": [float_bits_schema(), { "type": "null" }] }
        }),
        &[
            "enabled",
            "maybe_friction_bits",
            "maybe_restitution_bits",
            "maybe_tangent_speed_bits",
        ],
    )
}

pub(super) fn filter_schema() -> Value {
    closed_record(
        &json!({
            "category_bits": { "maximum": u16::MAX, "minimum": 0, "type": "integer" },
            "mask_bits": { "maximum": u16::MAX, "minimum": 0, "type": "integer" },
            "group_index": { "maximum": i16::MAX, "minimum": i16::MIN, "type": "integer" }
        }),
        &["category_bits", "mask_bits", "group_index"],
    )
}
