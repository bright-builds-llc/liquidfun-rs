use std::collections::BTreeSet;

use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    CollectionPolicy, DiscretePolicy, FloatBits, FloatPolicy, Sha256Hex, ToleranceProfile,
    ToleranceProfileVersion,
};

mod checkpoint;
mod rigid_world;

use checkpoint::render_checkpoint_schema;

use rigid_world::{
    rigid_world_request_schema, rigid_world_result_schema, rigid_world_scenario_schema,
    rigid_world_trace_definitions,
};

const PHASE2_DESCRIPTION: &str = "Phase 2 sets no broad rigid-body, joint, or particle tolerance values; synthetic numeric policies exist only for comparator coverage.";
const SCHEMA_DESCRIPTION: &str = "Deterministic presentation only. Typed Rust and C++ validation remains authoritative for cross-field references, uniqueness, ordering, hashes, and aggregate limits.";

mod presentations;
mod probes;
mod tolerance;

use presentations::*;
use probes::*;
use tolerance::*;

pub(super) fn schema_ref(name: &str) -> Value {
    json!({ "$ref": format!("#/$defs/{name}") })
}

pub(super) fn tagged_probe_input(kind: &str, fields: &Value, required: &[&str]) -> Value {
    let mut properties = fields
        .as_object()
        .expect("probe input fields are always JSON objects")
        .clone();
    properties.insert("kind".to_owned(), json!({ "const": kind }));
    let mut required_fields = vec!["kind"];
    required_fields.extend_from_slice(required);
    closed_record(&Value::Object(properties), &required_fields)
}

pub(super) fn vec2_bits_schema() -> Value {
    closed_record(
        &json!({ "x_bits": float_bits_schema(), "y_bits": float_bits_schema() }),
        &["x_bits", "y_bits"],
    )
}

fn vec3_bits_schema() -> Value {
    closed_record(
        &json!({ "x_bits": float_bits_schema(), "y_bits": float_bits_schema(), "z_bits": float_bits_schema() }),
        &["x_bits", "y_bits", "z_bits"],
    )
}

fn mat22_bits_schema() -> Value {
    closed_record(
        &json!({ "first": schema_ref("vec2_bits"), "second": schema_ref("vec2_bits") }),
        &["first", "second"],
    )
}

fn mat33_bits_schema() -> Value {
    closed_record(
        &json!({ "first": schema_ref("vec3_bits"), "second": schema_ref("vec3_bits"), "third": schema_ref("vec3_bits") }),
        &["first", "second", "third"],
    )
}

pub(super) fn transform_bits_schema() -> Value {
    closed_record(
        &json!({ "position": schema_ref("vec2_bits"), "angle_bits": float_bits_schema() }),
        &["position", "angle_bits"],
    )
}

fn sweep_bits_schema() -> Value {
    closed_record(
        &json!({
            "local_center": schema_ref("vec2_bits"),
            "initial_center": schema_ref("vec2_bits"),
            "center": schema_ref("vec2_bits"),
            "initial_angle_bits": float_bits_schema(),
            "angle_bits": float_bits_schema(),
            "initial_fraction_bits": float_bits_schema()
        }),
        &[
            "local_center",
            "initial_center",
            "center",
            "initial_angle_bits",
            "angle_bits",
            "initial_fraction_bits",
        ],
    )
}

pub(super) fn render_json_schema(document: &Value) -> String {
    let mut rendered = serde_json::to_string_pretty(&document)
        .expect("schema documents contain only JSON-native values");
    rendered.push('\n');
    rendered
}

pub(super) fn closed_record(properties: &Value, required: &[&str]) -> Value {
    json!({
        "additionalProperties": false,
        "properties": properties,
        "required": required,
        "type": "object"
    })
}

pub(super) fn version_schema() -> Value {
    json!({ "const": 1, "type": "integer" })
}

fn version_array_schema() -> Value {
    json!({ "items": version_schema(), "maxItems": 16, "minItems": 1, "type": "array" })
}

pub(super) fn uint32_schema() -> Value {
    json!({ "maximum": u32::MAX, "minimum": 0, "type": "integer" })
}

pub(super) fn uint64_schema() -> Value {
    json!({ "maximum": u64::MAX, "minimum": 0, "type": "integer" })
}

pub(super) fn float_bits_schema() -> Value {
    uint32_schema()
}

pub(super) fn semantic_id_schema() -> Value {
    json!({ "maxLength": 128, "pattern": "^[a-z0-9][a-z0-9._-]{0,127}$", "type": "string" })
}

pub(super) fn bounded_string_schema() -> Value {
    json!({ "maxLength": 4096, "minLength": 1, "type": "string" })
}

pub(super) fn sha256_schema() -> Value {
    json!({ "pattern": "^[0-9a-f]{64}$", "type": "string" })
}

pub(super) fn scenario_source_schema() -> Value {
    json!({
        "oneOf": [
            closed_record(&json!({ "kind": { "const": "named" }, "name": bounded_string_schema() }), &["kind", "name"]),
            closed_record(
                &json!({
                    "generator_id": bounded_string_schema(),
                    "generator_version": { "maximum": u32::MAX, "minimum": 1, "type": "integer" },
                    "kind": { "const": "seeded" },
                    "seed": uint64_schema()
                }),
                &["kind", "generator_id", "generator_version", "seed"],
            )
        ]
    })
}

fn build_identity_schema() -> Value {
    let string = bounded_string_schema();
    closed_record(
        &json!({
            "adapter_content_sha256": sha256_schema(),
            "adapter_revision": string,
            "build_type": bounded_string_schema(),
            "cmake_preset": bounded_string_schema(),
            "compiler_id": bounded_string_schema(),
            "compiler_version": bounded_string_schema(),
            "effective_compile_flags": bounded_string_schema(),
            "effective_link_flags": bounded_string_schema(),
            "oracle_revision": { "pattern": "^[0-9a-f]{40}$", "type": "string" },
            "sanitizer_mode": bounded_string_schema(),
            "target": bounded_string_schema()
        }),
        &[
            "oracle_revision",
            "adapter_revision",
            "adapter_content_sha256",
            "cmake_preset",
            "compiler_id",
            "compiler_version",
            "target",
            "build_type",
            "effective_compile_flags",
            "effective_link_flags",
            "sanitizer_mode",
        ],
    )
}

fn world_counts_schema() -> Value {
    closed_record(
        &json!({
            "bodies": uint32_schema(),
            "contacts": uint32_schema(),
            "fixtures": uint32_schema(),
            "joints": uint32_schema(),
            "particle_groups": uint32_schema(),
            "particle_systems": uint32_schema(),
            "particles": uint32_schema()
        }),
        &[
            "bodies",
            "fixtures",
            "joints",
            "contacts",
            "particle_systems",
            "particle_groups",
            "particles",
        ],
    )
}

#[cfg(test)]
mod tests;
