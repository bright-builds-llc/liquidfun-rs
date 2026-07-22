use serde_json::{Value, json};

use super::{
    SCHEMA_DESCRIPTION, closed_record, float_bits_schema, render_json_schema, semantic_id_schema,
    sha256_schema,
};

pub(super) fn render_checkpoint_schema() -> String {
    render_json_schema(&json!({
        "$id": "https://liquidfun-rs.invalid/protocol/schemas/checkpoint-v1.schema.json",
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "description": format!("{SCHEMA_DESCRIPTION} Checkpoints bind to resolved action or logical-step ordinals, never renderer frames or wall-clock time."),
        "oneOf": [catalog_run_request_schema(), canonical_checkpoint_schema()],
        "title": "liquidfun-rs resolved run and checkpoint presentation version 1",
        "x-version-axes": { "catalog_schema_version": 1, "checkpoint_schema_version": 1, "protocol_version": 1 }
    }))
}

fn catalog_run_request_schema() -> Value {
    closed_record(
        &json!({
            "catalog_schema_version": { "const": 1 },
            "generator_id": semantic_id_schema(),
            "generator_version": { "const": 1 },
            "maybe_seed": { "type": ["integer", "null"], "minimum": 0 },
            "protocol_version": { "const": 1 },
            "provenance_requirements": closed_record(&json!({
                "evidence_tier": { "enum": ["d0_replay", "d1_canonical", "d2_supported", "d3_exploratory"] },
                "limits_profile_sha256": sha256_schema(),
                "required_identity_sha256": sha256_schema()
            }), &["required_identity_sha256", "limits_profile_sha256", "evidence_tier"]),
            "record_kind": { "const": "catalog_run_request" },
            "request_id": semantic_id_schema(),
            "resolved_bytes": { "items": { "maximum": 255, "minimum": 0, "type": "integer" }, "maxItems": 1_048_576, "type": "array" },
            "resolved_sha256": sha256_schema(),
            "scenario_version": { "const": 1 },
            "settings": closed_record(&json!({
                "particle_iterations": { "maximum": 1024, "minimum": 1, "type": "integer" },
                "position_iterations": { "maximum": 1024, "minimum": 1, "type": "integer" },
                "timestep_bits": float_bits_schema(),
                "velocity_iterations": { "maximum": 1024, "minimum": 1, "type": "integer" }
            }), &["timestep_bits", "velocity_iterations", "position_iterations", "particle_iterations"]),
            "slug": semantic_id_schema()
        }),
        &[
            "protocol_version",
            "record_kind",
            "request_id",
            "catalog_schema_version",
            "slug",
            "scenario_version",
            "generator_id",
            "generator_version",
            "maybe_seed",
            "settings",
            "resolved_bytes",
            "resolved_sha256",
            "provenance_requirements",
        ],
    )
}

fn canonical_checkpoint_schema() -> Value {
    closed_record(
        &json!({
            "checkpoint_id": semantic_id_schema(),
            "checkpoint_schema_version": { "const": 1 },
            "debug_primitives": { "description": "Closed engine-neutral exact-bit point, segment, polyline, circle, transform_axes, aabb, arrow, and inert label records with stable semantic owner keys and explicit source_significant or canonicalized ordering.", "items": closed_record(&json!({}), &[]), "maxItems": 8192, "type": "array" },
            "numeric_observations": { "description": "Exact float bits paired with a closed Phase 4 policy_path.", "items": closed_record(&json!({ "observation_id": semantic_id_schema(), "policy_path": semantic_id_schema(), "value_bits": float_bits_schema() }), &["observation_id", "value_bits", "policy_path"]), "maxItems": 128, "type": "array" },
            "observations": { "description": "Exact structural presence, count, flag_bits, identity, or status values.", "items": closed_record(&json!({}), &[]), "maxItems": 128, "type": "array" },
            "ordered_occurrences": { "description": "Source-significant occurrence order is preserved.", "items": closed_record(&json!({}), &[]), "maxItems": 4096, "type": "array" },
            "position": { "additionalProperties": false, "description": "Closed action or logical_step semantic boundary.", "properties": {}, "required": [], "type": "object" },
            "profile_names": { "description": "Closed profile names only; timing and duration values are forbidden.", "items": { "enum": ["contact_lifecycle", "particle_solve", "rigid_solve", "continuous_solve", "apply_commands", "finalize"] }, "maxItems": 6, "type": "array", "uniqueItems": true },
            "protocol_version": { "const": 1 },
            "record_kind": { "const": "canonical_checkpoint" },
            "request_id": semantic_id_schema(),
            "resolved_sha256": sha256_schema(),
            "simulation_time_bits": float_bits_schema(),
            "unordered_sets": { "description": "Only explicitly unordered semantic sets are canonicalized by stable ID.", "items": closed_record(&json!({ "members": { "items": semantic_id_schema(), "maxItems": 4096, "type": "array", "uniqueItems": true }, "set_id": semantic_id_schema() }), &["set_id", "members"]), "maxItems": 128, "type": "array" }
        }),
        &[
            "protocol_version",
            "record_kind",
            "checkpoint_schema_version",
            "request_id",
            "resolved_sha256",
            "checkpoint_id",
            "position",
            "simulation_time_bits",
            "observations",
            "numeric_observations",
            "ordered_occurrences",
            "unordered_sets",
            "debug_primitives",
            "profile_names",
        ],
    )
}
