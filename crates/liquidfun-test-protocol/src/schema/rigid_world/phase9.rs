use serde_json::{Value, json};

use super::{closed_record, float_bits_schema, schema_ref, semantic_id_schema, tagged_probe_input};
use crate::{PHASE9_MAXIMUM_IDENTITIES, PHASE9_MAXIMUM_PARTICLES};

pub(super) fn particle_system_declaration_schema() -> Value {
    closed_record(
        &json!({
            "system_id": semantic_id_schema(),
            "buffer_mode": { "oneOf": [
                tagged_probe_input("growable", &json!({ "initial_capacity": { "maximum": PHASE9_MAXIMUM_PARTICLES, "minimum": 1, "type": "integer" } }), &["initial_capacity"]),
                tagged_probe_input("fixed", &json!({ "capacity": { "maximum": PHASE9_MAXIMUM_PARTICLES, "minimum": 1, "type": "integer" } }), &["capacity"])
            ] },
            "paused": { "type": "boolean" },
            "strict_contact_check": { "type": "boolean" },
            "stuck_threshold": { "maximum": u32::MAX, "minimum": 0, "type": "integer" },
            "density_bits": float_bits_schema(),
            "gravity_scale_bits": float_bits_schema(),
            "radius_bits": float_bits_schema(),
            "damping_bits": float_bits_schema(),
            "destruction_by_age": { "type": "boolean" },
            "lifetime_granularity_bits": float_bits_schema(),
            "maximum_count": { "oneOf": [
                { "maximum": PHASE9_MAXIMUM_PARTICLES, "minimum": 1, "type": "integer" },
                { "type": "null" }
            ] }
        }),
        &[
            "system_id",
            "buffer_mode",
            "paused",
            "strict_contact_check",
            "stuck_threshold",
            "density_bits",
            "gravity_scale_bits",
            "radius_bits",
            "damping_bits",
            "destruction_by_age",
            "lifetime_granularity_bits",
            "maximum_count",
        ],
    )
}

pub(super) fn particle_declaration_schema() -> Value {
    closed_record(
        &json!({
            "particle_id": semantic_id_schema(),
            "system_id": semantic_id_schema(),
            "position": schema_ref("vec2_bits"),
            "velocity": schema_ref("vec2_bits"),
            "flags_bits": { "maximum": u32::MAX, "minimum": 0, "type": "integer" },
            "color": { "items": { "maximum": 255, "minimum": 0, "type": "integer" }, "maxItems": 4, "minItems": 4, "type": "array" },
            "lifetime_bits": float_bits_schema()
        }),
        &[
            "particle_id",
            "system_id",
            "position",
            "velocity",
            "flags_bits",
            "color",
            "lifetime_bits",
        ],
    )
}

pub(super) fn particle_action_schema() -> Value {
    let identity = || json!({ "particle_id": semantic_id_schema() });
    let system = || json!({ "system_id": semantic_id_schema() });
    let identities = || json!({ "items": semantic_id_schema(), "maxItems": PHASE9_MAXIMUM_IDENTITIES, "minItems": 1, "type": "array" });
    json!({ "oneOf": [
        tagged_probe_input("create_system", &system(), &["system_id"]),
        tagged_probe_input("destroy_system", &system(), &["system_id"]),
        tagged_probe_input("create_particle", &identity(), &["particle_id"]),
        tagged_probe_input("inspect_system", &system(), &["system_id"]),
        tagged_probe_input("inspect_particle", &identity(), &["particle_id"]),
        tagged_probe_input("set_paused", &json!({ "system_id": semantic_id_schema(), "paused": { "type": "boolean" } }), &["system_id", "paused"]),
        tagged_probe_input("set_position", &json!({ "particle_id": semantic_id_schema(), "position": schema_ref("vec2_bits") }), &["particle_id", "position"]),
        tagged_probe_input("set_velocity", &json!({ "particle_id": semantic_id_schema(), "velocity": schema_ref("vec2_bits") }), &["particle_id", "velocity"]),
        tagged_probe_input("mark_for_destruction", &identity(), &["particle_id"]),
        tagged_probe_input("compact", &system(), &["system_id"]),
        tagged_probe_input("apply_force", &json!({ "particle_ids": identities(), "force": schema_ref("vec2_bits") }), &["particle_ids", "force"]),
        tagged_probe_input("apply_impulse", &json!({ "particle_ids": identities(), "impulse": schema_ref("vec2_bits") }), &["particle_ids", "impulse"]),
        tagged_probe_input("request_statistics", &system(), &["system_id"]),
        tagged_probe_input("query_aabb", &json!({ "system_id": { "oneOf": [semantic_id_schema(), { "type": "null" }] }, "lower": schema_ref("vec2_bits"), "upper": schema_ref("vec2_bits"), "control": { "enum": ["continue", "terminate"] } }), &["system_id", "lower", "upper"]),
        tagged_probe_input("ray_cast", &json!({ "system_id": { "oneOf": [semantic_id_schema(), { "type": "null" }] }, "start": schema_ref("vec2_bits"), "end": schema_ref("vec2_bits"), "control": { "enum": ["ignore", "continue", "clip", "terminate"] } }), &["system_id", "start", "end"])
    ] })
}

pub(super) fn particle_observation_schema() -> Value {
    json!({ "oneOf": [
        tagged_probe_input("system", &json!({ "system_id": semantic_id_schema(), "paused": { "type": "boolean" }, "particle_ids": { "items": semantic_id_schema(), "maxItems": PHASE9_MAXIMUM_IDENTITIES, "type": "array" } }), &["system_id", "paused", "particle_ids"]),
        tagged_probe_input("particle", &json!({ "snapshot": particle_snapshot_schema() }), &["snapshot"]),
        tagged_probe_input("lifecycle", &json!({ "occurrence": occurrence_schema() }), &["occurrence"]),
        tagged_probe_input("particle_contact", &json!({ "contact": particle_contact_schema() }), &["contact"]),
        tagged_probe_input("body_contact", &json!({ "contact": body_contact_schema() }), &["contact"]),
        tagged_probe_input("statistics", &json!({ "statistics": statistics_schema() }), &["statistics"]),
        tagged_probe_input("query", &json!({ "terminated": { "type": "boolean" }, "particle_ids": { "items": semantic_id_schema(), "maxItems": PHASE9_MAXIMUM_IDENTITIES, "type": "array" } }), &["terminated", "particle_ids"]),
        tagged_probe_input("ray_cast", &json!({ "terminated": { "type": "boolean" }, "particle_ids": { "items": semantic_id_schema(), "maxItems": PHASE9_MAXIMUM_IDENTITIES, "type": "array" }, "fractions_bits": { "items": float_bits_schema(), "maxItems": PHASE9_MAXIMUM_IDENTITIES, "type": "array" } }), &["terminated", "particle_ids", "fractions_bits"]),
        tagged_probe_input("mixed_state", &json!({ "body_ids": { "items": semantic_id_schema(), "maxItems": PHASE9_MAXIMUM_IDENTITIES, "type": "array" }, "particle_ids": { "items": semantic_id_schema(), "maxItems": PHASE9_MAXIMUM_IDENTITIES, "type": "array" } }), &["body_ids", "particle_ids"])
    ] })
}

fn particle_snapshot_schema() -> Value {
    closed_record(
        &json!({
            "particle_id": semantic_id_schema(), "system_id": semantic_id_schema(),
            "position": schema_ref("rigid_vec2_bits"), "velocity": schema_ref("rigid_vec2_bits"),
            "flags_bits": { "maximum": u32::MAX, "minimum": 0, "type": "integer" },
            "color": { "items": { "maximum": 255, "minimum": 0, "type": "integer" }, "maxItems": 4, "minItems": 4, "type": "array" },
            "weight_bits": float_bits_schema(), "force": schema_ref("rigid_vec2_bits"),
            "pending_destruction": { "type": "boolean" }
        }),
        &[
            "particle_id",
            "system_id",
            "position",
            "velocity",
            "flags_bits",
            "color",
            "weight_bits",
            "force",
            "pending_destruction",
        ],
    )
}

fn occurrence_schema() -> Value {
    closed_record(
        &json!({
            "ordinal": { "maximum": u32::MAX, "minimum": 0, "type": "integer" },
            "kind": { "enum": ["filter_decision", "contact_created", "contact_destroyed", "particle_destroyed", "system_destroyed", "query_visited", "ray_visited"] },
            "system_id": semantic_id_schema(),
            "maybe_particle_id": nullable_id_schema(), "maybe_other_particle_id": nullable_id_schema(),
            "maybe_fixture_id": nullable_id_schema()
        }),
        &[
            "ordinal",
            "kind",
            "system_id",
            "maybe_particle_id",
            "maybe_other_particle_id",
            "maybe_fixture_id",
        ],
    )
}

fn particle_contact_schema() -> Value {
    closed_record(
        &json!({
            "system_id": semantic_id_schema(), "particle_a_id": semantic_id_schema(),
            "particle_b_id": semantic_id_schema(), "flags_bits": { "maximum": u32::MAX, "minimum": 0, "type": "integer" },
            "weight_bits": float_bits_schema(), "normal": schema_ref("rigid_vec2_bits")
        }),
        &[
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
            "system_id": semantic_id_schema(), "particle_id": semantic_id_schema(),
            "body_id": semantic_id_schema(), "fixture_id": semantic_id_schema(),
            "weight_bits": float_bits_schema(), "normal": schema_ref("rigid_vec2_bits"),
            "mass_bits": float_bits_schema()
        }),
        &[
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

fn statistics_schema() -> Value {
    closed_record(
        &json!({
            "maybe_system_id": nullable_id_schema(), "system_count": uint_schema(),
            "particle_count": uint_schema(), "pending_particle_count": uint_schema(),
            "particle_contact_count": uint_schema(), "body_contact_count": uint_schema(),
            "stuck_particle_ids": { "items": semantic_id_schema(), "maxItems": PHASE9_MAXIMUM_IDENTITIES, "type": "array" },
            "collision_energy_bits": float_bits_schema(), "declared_capacity": uint_schema(),
            "effective_capacity": uint_schema()
        }),
        &[
            "maybe_system_id",
            "system_count",
            "particle_count",
            "pending_particle_count",
            "particle_contact_count",
            "body_contact_count",
            "stuck_particle_ids",
            "collision_energy_bits",
            "declared_capacity",
            "effective_capacity",
        ],
    )
}

fn nullable_id_schema() -> Value {
    json!({ "oneOf": [semantic_id_schema(), { "type": "null" }] })
}

fn uint_schema() -> Value {
    json!({ "maximum": u32::MAX, "minimum": 0, "type": "integer" })
}
