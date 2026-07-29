use super::*;

pub(super) fn render_protocol_schema() -> String {
    render_json_schema(&json!({
        "$id": "https://liquidfun-rs.invalid/protocol/schemas/protocol-v1.schema.json",
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "description": format!("{SCHEMA_DESCRIPTION} This schema presents newline-delimited transport records; framing and duplicate-member rejection remain codec responsibilities."),
        "oneOf": [
            closed_record(
                &json!({
                    "build_identity": build_identity_schema(),
                    "identity_sha256": sha256_schema(),
                    "protocol_version": version_schema(),
                    "record_kind": { "const": "handshake" },
                    "supported_scenario_versions": version_array_schema(),
                    "supported_tolerance_versions": version_array_schema(),
                    "supported_trace_versions": version_array_schema()
                }),
                &["protocol_version", "record_kind", "supported_scenario_versions", "supported_trace_versions", "supported_tolerance_versions", "build_identity", "identity_sha256"],
            ),
            closed_record(
                &json!({
                    "protocol_version": version_schema(),
                    "record_kind": { "const": "scenario_request" },
                    "request_id": semantic_id_schema(),
                    "requested_trace_schema_version": version_schema(),
                    "scenario": { "$ref": "scenario-v1.schema.json" },
                    "scenario_schema_version": version_schema(),
                    "tolerance_profile_sha256": sha256_schema(),
                    "tolerance_profile_version": version_schema()
                }),
                &["protocol_version", "record_kind", "request_id", "scenario_schema_version", "requested_trace_schema_version", "tolerance_profile_version", "tolerance_profile_sha256", "scenario"],
            ),
            probe_request_schema("math_probe_request"),
            probe_request_schema("collision_probe_request"),
            rigid_world_request_schema()
        ],
        "title": "liquidfun-rs protocol presentation version 1",
        "x-version-axes": {
            "protocol_version": 1,
            "scenario_schema_version": 1,
            "tolerance_profile_version": 1,
            "trace_schema_version": 1
        }
    }))
}

pub(super) fn render_scenario_schema() -> String {
    render_json_schema(&json!({
        "$id": "https://liquidfun-rs.invalid/protocol/schemas/scenario-v1.schema.json",
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$defs": scenario_definitions(),
        "description": SCHEMA_DESCRIPTION,
        "oneOf": [
            physics_scenario_schema(),
            math_probe_scenario_schema(),
            collision_probe_scenario_schema(),
            rigid_world_scenario_schema()
        ],
        "title": "liquidfun-rs scenario presentation version 1",
        "x-version-axes": { "scenario_schema_version": 1 }
    }))
}

pub(super) fn render_trace_schema() -> String {
    render_json_schema(&json!({
        "$id": "https://liquidfun-rs.invalid/protocol/schemas/trace-v1.schema.json",
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$defs": rigid_world_trace_definitions(),
        "description": format!("{SCHEMA_DESCRIPTION} Record-sequence state transitions and reset proof validation remain typed-validator responsibilities."),
        "oneOf": [
            closed_record(
                &json!({
                    "engine_kind": { "enum": ["native_rust", "cpp_oracle"] },
                    "identity_sha256": sha256_schema(),
                    "protocol_version": version_schema(),
                    "record_kind": { "const": "trace_begin" },
                    "request_id": semantic_id_schema(),
                    "scenario_id": semantic_id_schema(),
                    "scenario_sha256": sha256_schema(),
                    "source": scenario_source_schema(),
                    "tolerance_profile_sha256": sha256_schema(),
                    "tolerance_profile_version": version_schema(),
                    "trace_schema_version": version_schema()
                }),
                &["protocol_version", "record_kind", "request_id", "trace_schema_version", "scenario_id", "scenario_sha256", "source", "tolerance_profile_version", "tolerance_profile_sha256", "engine_kind", "identity_sha256"],
            ),
            closed_record(
                &json!({
                    "checkpoint_id": semantic_id_schema(),
                    "identity_sha256": sha256_schema(),
                    "ordinal": uint32_schema(),
                    "phase": bounded_string_schema(),
                    "protocol_version": version_schema(),
                    "record_kind": { "const": "checkpoint" },
                    "request_id": semantic_id_schema(),
                    "simulation_time_bits": float_bits_schema(),
                    "world_counts": world_counts_schema()
                }),
                &["protocol_version", "record_kind", "request_id", "checkpoint_id", "ordinal", "phase", "simulation_time_bits", "world_counts", "identity_sha256"],
            ),
            closed_record(
                &json!({
                    "checkpoint_count": uint32_schema(),
                    "identity_sha256": sha256_schema(),
                    "protocol_version": version_schema(),
                    "record_kind": { "const": "trace_end" },
                    "request_id": semantic_id_schema(),
                    "reset_epoch": uint64_schema(),
                    "reset_verified": { "const": true },
                    "trace_payload_sha256": sha256_schema()
                }),
                &["protocol_version", "record_kind", "request_id", "checkpoint_count", "trace_payload_sha256", "reset_epoch", "reset_verified", "identity_sha256"],
            ),
            math_probe_result_schema(),
            collision_probe_result_schema(),
            rigid_world_result_schema(),
            closed_record(
                &json!({
                    "protocol_version": version_schema(),
                    "record_kind": { "const": "math_probe_end" },
                    "request_id": semantic_id_schema(),
                    "reset_epoch": uint64_schema(),
                    "reset_verified": { "const": true },
                    "result_count": uint32_schema()
                }),
                &["protocol_version", "record_kind", "request_id", "result_count", "reset_epoch", "reset_verified"],
            )
        ],
        "title": "liquidfun-rs trace presentation version 1",
        "x-version-axes": {
            "protocol_version": 1,
            "tolerance_profile_version": 1,
            "trace_schema_version": 1
        }
    }))
}
