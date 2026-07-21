use serde_json::{Value, json};

use super::super::{
    closed_record, float_bits_schema, schema_ref, semantic_id_schema, tagged_probe_input,
    uint32_schema,
};
use crate::{
    PHASE10_MAXIMUM_CONTACTS, PHASE10_MAXIMUM_EVENTS, PHASE10_MAXIMUM_GROUPS,
    PHASE10_MAXIMUM_PAIRS, PHASE10_MAXIMUM_PARTICLES, PHASE10_MAXIMUM_SHAPE_VERTICES,
    PHASE10_MAXIMUM_SHAPES, PHASE10_MAXIMUM_STEPS, PHASE10_MAXIMUM_TRIADS,
    PHASE10_MAXIMUM_WITNESSES, PHASE10_PUBLIC_GROUP_FLAG_MASK, PHASE10_PUBLIC_PARTICLE_FLAG_MASK,
    Phase10BehaviorLeaf, Phase10EventKind, Phase10RejectionReason, WitnessRole,
};

pub(super) fn phase10_operation_schema() -> Value {
    json!({ "oneOf": [
        tagged_probe_input("create_group", &json!({ "definition": group_definition_schema() }), &["definition"]),
        tagged_probe_input("join_groups", &json!({ "target_group_id": semantic_id_schema(), "source_group_id": semantic_id_schema() }), &["target_group_id", "source_group_id"]),
        tagged_probe_input("split_group", &json!({ "group_id": semantic_id_schema(), "created_group_ids": id_array(PHASE10_MAXIMUM_GROUPS, 1) }), &["group_id", "created_group_ids"]),
        tagged_probe_input("set_group_flags", &json!({ "group_id": semantic_id_schema(), "group_flags_bits": group_flags_schema() }), &["group_id", "group_flags_bits"]),
        tagged_probe_input("destroy_group", &json!({ "group_id": semantic_id_schema() }), &["group_id"]),
        tagged_probe_input("step", &json!({
            "timestep_bits": float_bits_schema(),
            "velocity_iterations": bounded_step_schema(),
            "position_iterations": bounded_step_schema(),
            "particle_iterations": bounded_step_schema()
        }), &["timestep_bits", "velocity_iterations", "position_iterations", "particle_iterations"]),
        tagged_probe_input("inspect_state", &json!({}), &[])
    ] })
}

pub(super) fn phase10_observation_schema() -> Value {
    json!({ "oneOf": [
        tagged_probe_input("state", &json!({ "state": state_schema() }), &["state"])
    ] })
}

fn group_definition_schema() -> Value {
    closed_record(
        &json!({
            "system_id": semantic_id_schema(),
            "provenance": provenance_schema(),
            "group_id": semantic_id_schema(),
            "member_ids": id_array(PHASE10_MAXIMUM_PARTICLES, 1),
            "source": group_source_schema(),
            "destination": destination_schema(),
            "particle_flags_bits": particle_flags_schema(),
            "group_flags_bits": group_flags_schema(),
            "transform": schema_ref("rigid_transform_bits"),
            "linear_velocity": schema_ref("rigid_vec2_bits"),
            "angular_velocity_bits": float_bits_schema(),
            "color": { "type": "array", "items": { "type": "integer", "minimum": 0, "maximum": 255 }, "minItems": 4, "maxItems": 4 },
            "strength_bits": float_bits_schema(),
            "maybe_stride_bits": nullable_float_schema(),
            "lifetime_bits": float_bits_schema()
        }),
        &[
            "provenance",
            "system_id",
            "group_id",
            "member_ids",
            "source",
            "destination",
            "particle_flags_bits",
            "group_flags_bits",
            "transform",
            "linear_velocity",
            "angular_velocity_bits",
            "color",
            "strength_bits",
            "maybe_stride_bits",
            "lifetime_bits",
        ],
    )
}

fn group_source_schema() -> Value {
    json!({ "oneOf": [
        tagged_probe_input("filled", &json!({ "shapes": { "type": "array", "items": shape_schema(), "minItems": 1, "maxItems": PHASE10_MAXIMUM_SHAPES } }), &["shapes"]),
        tagged_probe_input("stroke", &json!({ "shape": shape_schema() }), &["shape"]),
        tagged_probe_input("explicit", &json!({ "positions": { "type": "array", "items": schema_ref("rigid_vec2_bits"), "minItems": 1, "maxItems": PHASE10_MAXIMUM_PARTICLES } }), &["positions"])
    ] })
}

fn shape_schema() -> Value {
    json!({ "oneOf": [
        tagged_probe_input("circle", &json!({ "center": schema_ref("rigid_vec2_bits"), "radius_bits": float_bits_schema() }), &["center", "radius_bits"]),
        tagged_probe_input("polygon", &json!({ "vertices": { "type": "array", "items": schema_ref("rigid_vec2_bits"), "minItems": 3, "maxItems": 8 } }), &["vertices"]),
        tagged_probe_input("edge", &json!({ "vertex_a": schema_ref("rigid_vec2_bits"), "vertex_b": schema_ref("rigid_vec2_bits") }), &["vertex_a", "vertex_b"]),
        tagged_probe_input("chain", &json!({ "vertices": { "type": "array", "items": schema_ref("rigid_vec2_bits"), "minItems": 2, "maxItems": PHASE10_MAXIMUM_SHAPE_VERTICES }, "looped": { "type": "boolean" } }), &["vertices", "looped"])
    ] })
}

fn destination_schema() -> Value {
    json!({ "oneOf": [
        tagged_probe_input("new", &json!({}), &[]),
        tagged_probe_input("append_to", &json!({ "target_group_id": semantic_id_schema() }), &["target_group_id"])
    ] })
}

fn state_schema() -> Value {
    closed_record(
        &json!({
            "provenance": provenance_schema(),
            "outcome": outcome_schema(),
            "groups": array_schema(&group_snapshot_schema(), PHASE10_MAXIMUM_GROUPS),
            "particles": array_schema(&particle_snapshot_schema(), PHASE10_MAXIMUM_PARTICLES),
            "pairs": array_schema(&pair_schema(), PHASE10_MAXIMUM_PAIRS),
            "triads": array_schema(&triad_schema(), PHASE10_MAXIMUM_TRIADS),
            "particle_contacts": array_schema(&particle_contact_schema(), PHASE10_MAXIMUM_CONTACTS),
            "body_contacts": array_schema(&body_contact_schema(), PHASE10_MAXIMUM_CONTACTS),
            "events": array_schema(&event_schema(), PHASE10_MAXIMUM_EVENTS),
            "witnesses": array_schema(&witness_schema(), PHASE10_MAXIMUM_WITNESSES)
        }),
        &[
            "provenance",
            "outcome",
            "groups",
            "particles",
            "pairs",
            "triads",
            "particle_contacts",
            "body_contacts",
            "events",
            "witnesses",
        ],
    )
}

fn provenance_schema() -> Value {
    closed_record(
        &json!({
            "extension_version": { "const": crate::PHASE10_RIGID_WORLD_EXTENSION_VERSION },
            "generator_id": semantic_id_schema(),
            "generator_version": semantic_id_schema(),
            "upstream_revision": semantic_id_schema(),
            "toolchain_id": semantic_id_schema(),
            "seed": { "type": "integer", "minimum": 0, "maximum": u64::MAX }
        }),
        &[
            "extension_version",
            "generator_id",
            "generator_version",
            "upstream_revision",
            "toolchain_id",
            "seed",
        ],
    )
}

fn outcome_schema() -> Value {
    json!({ "oneOf": [
        tagged_probe_input("completed", &json!({}), &[]),
        tagged_probe_input("rejected", &json!({ "reason": { "enum": super::enum_values(&[
            Phase10RejectionReason::CapacityExceeded,
            Phase10RejectionReason::InvalidHandle,
            Phase10RejectionReason::InvalidRecipe,
            Phase10RejectionReason::Locked,
            Phase10RejectionReason::Poisoned,
            Phase10RejectionReason::NumericFailure,
        ]) } }), &["reason"])
    ] })
}

fn group_snapshot_schema() -> Value {
    closed_record(
        &json!({
            "ordinal": uint32_schema(), "group_id": semantic_id_schema(), "system_id": semantic_id_schema(),
            "member_ids": id_array(PHASE10_MAXIMUM_PARTICLES, 0), "group_flags_bits": group_flags_schema(),
            "transform": schema_ref("rigid_transform_bits"), "center": schema_ref("rigid_vec2_bits"),
            "linear_velocity": schema_ref("rigid_vec2_bits"), "angular_velocity_bits": float_bits_schema(),
            "mass_bits": float_bits_schema(), "inertia_bits": float_bits_schema(),
            "maybe_depths_bits": { "oneOf": [array_schema(&float_bits_schema(), PHASE10_MAXIMUM_PARTICLES), { "type": "null" }] }
        }),
        &[
            "ordinal",
            "group_id",
            "system_id",
            "member_ids",
            "group_flags_bits",
            "transform",
            "center",
            "linear_velocity",
            "angular_velocity_bits",
            "mass_bits",
            "inertia_bits",
            "maybe_depths_bits",
        ],
    )
}

fn particle_snapshot_schema() -> Value {
    closed_record(
        &json!({
            "particle_id": semantic_id_schema(), "system_id": semantic_id_schema(), "group_id": semantic_id_schema(),
            "position": schema_ref("rigid_vec2_bits"), "velocity": schema_ref("rigid_vec2_bits"),
            "flags_bits": particle_flags_schema(),
            "color": { "type": "array", "items": { "type": "integer", "minimum": 0, "maximum": 255 }, "minItems": 4, "maxItems": 4 },
            "weight_bits": float_bits_schema()
        }),
        &[
            "particle_id",
            "system_id",
            "group_id",
            "position",
            "velocity",
            "flags_bits",
            "color",
            "weight_bits",
        ],
    )
}

fn pair_schema() -> Value {
    closed_record(
        &json!({
            "ordinal": uint32_schema(), "particle_a_id": semantic_id_schema(), "particle_b_id": semantic_id_schema(),
            "flags_bits": particle_flags_schema(), "strength_bits": float_bits_schema(), "distance_bits": float_bits_schema()
        }),
        &[
            "ordinal",
            "particle_a_id",
            "particle_b_id",
            "flags_bits",
            "strength_bits",
            "distance_bits",
        ],
    )
}

fn triad_schema() -> Value {
    closed_record(
        &json!({
            "ordinal": uint32_schema(), "particle_a_id": semantic_id_schema(), "particle_b_id": semantic_id_schema(), "particle_c_id": semantic_id_schema(),
            "flags_bits": particle_flags_schema(), "strength_bits": float_bits_schema(),
            "pa": schema_ref("rigid_vec2_bits"), "pb": schema_ref("rigid_vec2_bits"), "pc": schema_ref("rigid_vec2_bits"),
            "ka_bits": float_bits_schema(), "kb_bits": float_bits_schema(), "kc_bits": float_bits_schema(), "s_bits": float_bits_schema()
        }),
        &[
            "ordinal",
            "particle_a_id",
            "particle_b_id",
            "particle_c_id",
            "flags_bits",
            "strength_bits",
            "pa",
            "pb",
            "pc",
            "ka_bits",
            "kb_bits",
            "kc_bits",
            "s_bits",
        ],
    )
}

fn particle_contact_schema() -> Value {
    closed_record(
        &json!({
            "ordinal": uint32_schema(), "system_id": semantic_id_schema(), "particle_a_id": semantic_id_schema(), "particle_b_id": semantic_id_schema(),
            "flags_bits": particle_flags_schema(), "weight_bits": float_bits_schema(), "normal": schema_ref("rigid_vec2_bits")
        }),
        &[
            "ordinal",
            "system_id",
            "particle_a_id",
            "particle_b_id",
            "flags_bits",
            "weight_bits",
            "normal",
        ],
    )
}

fn body_contact_schema() -> Value {
    closed_record(
        &json!({
            "ordinal": uint32_schema(), "system_id": semantic_id_schema(), "particle_id": semantic_id_schema(),
            "body_id": semantic_id_schema(), "fixture_id": semantic_id_schema(), "weight_bits": float_bits_schema(),
            "normal": schema_ref("rigid_vec2_bits"), "mass_bits": float_bits_schema()
        }),
        &[
            "ordinal",
            "system_id",
            "particle_id",
            "body_id",
            "fixture_id",
            "weight_bits",
            "normal",
            "mass_bits",
        ],
    )
}

fn event_schema() -> Value {
    closed_record(
        &json!({
            "ordinal": uint32_schema(), "kind": { "enum": super::enum_values(&[
                Phase10EventKind::GroupCreated, Phase10EventKind::GroupsJoined, Phase10EventKind::GroupSplit,
                Phase10EventKind::GroupDestroyed, Phase10EventKind::ParticleDestroyed,
                Phase10EventKind::ParticleContactBegin, Phase10EventKind::ParticleContactEnd,
                Phase10EventKind::BodyContactBegin, Phase10EventKind::BodyContactEnd,
            ]) },
            "system_id": semantic_id_schema(), "maybe_group_id": nullable_id_schema(),
            "maybe_particle_id": nullable_id_schema(), "maybe_other_particle_id": nullable_id_schema(),
            "maybe_body_id": nullable_id_schema()
        }),
        &[
            "ordinal",
            "kind",
            "system_id",
            "maybe_group_id",
            "maybe_particle_id",
            "maybe_other_particle_id",
            "maybe_body_id",
        ],
    )
}

fn witness_schema() -> Value {
    closed_record(
        &json!({
            "ordinal": uint32_schema(),
            "behavior_leaf": { "enum": super::enum_values(&[
                Phase10BehaviorLeaf::GroupCreate, Phase10BehaviorLeaf::GroupAppend, Phase10BehaviorLeaf::GroupJoin,
                Phase10BehaviorLeaf::GroupSplit, Phase10BehaviorLeaf::GroupFlags, Phase10BehaviorLeaf::GroupDestroy,
                Phase10BehaviorLeaf::Water, Phase10BehaviorLeaf::Zombie, Phase10BehaviorLeaf::Wall,
                Phase10BehaviorLeaf::Spring, Phase10BehaviorLeaf::Elastic, Phase10BehaviorLeaf::Viscous,
                Phase10BehaviorLeaf::Powder, Phase10BehaviorLeaf::Tensile, Phase10BehaviorLeaf::ColorMixing,
                Phase10BehaviorLeaf::Barrier, Phase10BehaviorLeaf::StaticPressure, Phase10BehaviorLeaf::Reactive,
                Phase10BehaviorLeaf::Repulsive, Phase10BehaviorLeaf::SolidGroup, Phase10BehaviorLeaf::RigidGroup,
                Phase10BehaviorLeaf::BodyInteraction,
            ]) },
            "role": { "enum": super::enum_values(&[WitnessRole::Control, WitnessRole::Activation, WitnessRole::Interaction]) },
            "observation": witness_observation_schema()
        }),
        &["ordinal", "behavior_leaf", "role", "observation"],
    )
}

fn witness_observation_schema() -> Value {
    json!({ "oneOf": [
        tagged_probe_input("control_unchanged", &json!({}), &[]),
        tagged_probe_input("flag_activated", &json!({ "flags_bits": particle_flags_schema() }), &["flags_bits"]),
        tagged_probe_input("particle_velocity", &json!({ "particle_id": semantic_id_schema(), "before": schema_ref("rigid_vec2_bits"), "after": schema_ref("rigid_vec2_bits") }), &["particle_id", "before", "after"]),
        tagged_probe_input("scalar", &json!({ "value_bits": float_bits_schema() }), &["value_bits"]),
        tagged_probe_input("count", &json!({ "value": uint32_schema() }), &["value"]),
        tagged_probe_input("occurrence", &json!({ "event_ordinal": uint32_schema() }), &["event_ordinal"]),
        tagged_probe_input("topology", &json!({ "pair_count": uint32_schema(), "triad_count": uint32_schema() }), &["pair_count", "triad_count"])
    ] })
}

fn bounded_step_schema() -> Value {
    json!({ "type": "integer", "minimum": 1, "maximum": PHASE10_MAXIMUM_STEPS })
}

fn particle_flags_schema() -> Value {
    json!({ "type": "integer", "minimum": 0, "maximum": PHASE10_PUBLIC_PARTICLE_FLAG_MASK, "multipleOf": 2 })
}

fn group_flags_schema() -> Value {
    json!({ "type": "integer", "minimum": 0, "maximum": PHASE10_PUBLIC_GROUP_FLAG_MASK })
}

fn id_array(maximum: usize, minimum: usize) -> Value {
    json!({ "type": "array", "items": semantic_id_schema(), "minItems": minimum, "maxItems": maximum })
}

fn array_schema(item: &Value, maximum: usize) -> Value {
    json!({ "type": "array", "items": item, "maxItems": maximum })
}

fn nullable_id_schema() -> Value {
    json!({ "oneOf": [semantic_id_schema(), { "type": "null" }] })
}

fn nullable_float_schema() -> Value {
    json!({ "oneOf": [float_bits_schema(), { "type": "null" }] })
}
