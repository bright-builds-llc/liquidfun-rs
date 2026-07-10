use std::collections::BTreeSet;

use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    CollectionPolicy, DiscretePolicy, FloatBits, FloatPolicy, Sha256Hex, ToleranceProfile,
    ToleranceProfileVersion,
};

const PHASE2_DESCRIPTION: &str = "Phase 2 sets no broad rigid-body, joint, or particle tolerance values; synthetic numeric policies exist only for comparator coverage.";
const SCHEMA_DESCRIPTION: &str = "Deterministic presentation only. Typed Rust and C++ validation remains authoritative for cross-field references, uniqueness, ordering, hashes, and aggregate limits.";

#[derive(Debug, thiserror::Error)]
enum PresentationError {
    #[error("invalid tolerance profile TOML: {0}")]
    InvalidToml(#[from] toml::de::Error),
    #[error("tolerance profile presentation does not match typed authority: {0}")]
    AuthorityMismatch(&'static str),
    #[error("duplicate tolerance policy field `{0}`")]
    DuplicatePolicy(String),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ToleranceProfilePresentation {
    profile_id: String,
    version: ToleranceProfileVersion,
    profile_sha256: Sha256Hex,
    description: String,
    float_policies: Vec<FloatPolicyPresentation>,
    discrete_policies: Vec<DiscretePolicyPresentation>,
    collection_policies: Vec<CollectionPolicyPresentation>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FloatPolicyPresentation {
    field: String,
    scope: String,
    policy: PresentedFloatPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum PresentedFloatPolicy {
    ExactBits,
    Absolute {
        max_bits: FloatBits,
    },
    AbsoluteRelative {
        absolute_bits: FloatBits,
        relative_bits: FloatBits,
    },
    Ulps {
        max: u32,
    },
}

impl From<FloatPolicy> for PresentedFloatPolicy {
    fn from(policy: FloatPolicy) -> Self {
        match policy {
            FloatPolicy::ExactBits => Self::ExactBits,
            FloatPolicy::Absolute { max_bits } => Self::Absolute { max_bits },
            FloatPolicy::AbsoluteRelative {
                absolute_bits,
                relative_bits,
            } => Self::AbsoluteRelative {
                absolute_bits,
                relative_bits,
            },
            FloatPolicy::Ulps { max } => Self::Ulps { max },
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DiscretePolicyPresentation {
    field: String,
    kind: DiscretePolicy,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CollectionPolicyPresentation {
    field: String,
    kind: CollectionPolicy,
}

fn render_tolerance_profile_presentation() -> String {
    let profile = ToleranceProfile::phase2_v1();
    let [absolute, absolute_relative, ulps] = ToleranceProfile::synthetic_float_policies();
    let FloatPolicy::Absolute { max_bits } = absolute else {
        unreachable!("typed synthetic absolute policy must retain its variant");
    };
    let FloatPolicy::AbsoluteRelative {
        absolute_bits,
        relative_bits,
    } = absolute_relative
    else {
        unreachable!("typed synthetic absolute-relative policy must retain its variant");
    };
    let FloatPolicy::Ulps { max } = ulps else {
        unreachable!("typed synthetic ULP policy must retain its variant");
    };

    format!(
        concat!(
            "profile_id = \"{}\"\n",
            "version = {}\n",
            "profile_sha256 = \"{}\"\n",
            "description = \"{}\"\n",
            "\n",
            "[[float_policies]]\n",
            "field = \"simulation_time\"\n",
            "scope = \"phase2_trace\"\n",
            "policy = {{ kind = \"exact_bits\" }}\n",
            "\n",
            "[[float_policies]]\n",
            "field = \"synthetic_absolute\"\n",
            "scope = \"comparator_coverage\"\n",
            "policy = {{ kind = \"absolute\", max_bits = {} }}\n",
            "\n",
            "[[float_policies]]\n",
            "field = \"synthetic_absolute_relative\"\n",
            "scope = \"comparator_coverage\"\n",
            "policy = {{ kind = \"absolute_relative\", absolute_bits = {}, relative_bits = {} }}\n",
            "\n",
            "[[float_policies]]\n",
            "field = \"synthetic_ulps\"\n",
            "scope = \"comparator_coverage\"\n",
            "policy = {{ kind = \"ulps\", max = {} }}\n",
            "\n",
            "[[discrete_policies]]\n",
            "field = \"world_counts\"\n",
            "kind = \"exact\"\n",
            "\n",
            "[[collection_policies]]\n",
            "field = \"checkpoints\"\n",
            "kind = \"ordered\"\n"
        ),
        profile.profile_id(),
        profile.version().get(),
        profile.profile_sha256().as_str(),
        PHASE2_DESCRIPTION,
        max_bits.bits(),
        absolute_bits.bits(),
        relative_bits.bits(),
        max,
    )
}

fn check_tolerance_profile_presentation(input: &str) -> Result<(), PresentationError> {
    let presentation: ToleranceProfilePresentation = toml::from_str(input)?;
    let profile = ToleranceProfile::phase2_v1();

    validate_profile_header(&presentation, &profile)?;
    validate_float_policies(&presentation.float_policies, &profile)?;
    validate_discrete_policies(&presentation.discrete_policies, &profile)?;
    validate_collection_policies(&presentation.collection_policies, &profile)?;

    if render_tolerance_profile_presentation() != input {
        return Err(PresentationError::AuthorityMismatch(
            "tracked bytes are not the deterministic rendering",
        ));
    }
    Ok(())
}

fn validate_profile_header(
    presentation: &ToleranceProfilePresentation,
    profile: &ToleranceProfile,
) -> Result<(), PresentationError> {
    if presentation.profile_id != profile.profile_id() {
        return Err(PresentationError::AuthorityMismatch("profile ID"));
    }
    if presentation.version != profile.version() {
        return Err(PresentationError::AuthorityMismatch("profile version"));
    }
    if presentation.profile_sha256 != *profile.profile_sha256() {
        return Err(PresentationError::AuthorityMismatch("profile hash"));
    }
    if presentation.description != PHASE2_DESCRIPTION {
        return Err(PresentationError::AuthorityMismatch("profile description"));
    }
    Ok(())
}

fn validate_float_policies(
    policies: &[FloatPolicyPresentation],
    profile: &ToleranceProfile,
) -> Result<(), PresentationError> {
    reject_duplicate_fields(policies.iter().map(|policy| policy.field.as_str()))?;
    let synthetic = ToleranceProfile::synthetic_float_policies();
    let expected = [
        (
            "simulation_time",
            "phase2_trace",
            PresentedFloatPolicy::from(profile.simulation_time()),
        ),
        (
            "synthetic_absolute",
            "comparator_coverage",
            PresentedFloatPolicy::from(synthetic[0]),
        ),
        (
            "synthetic_absolute_relative",
            "comparator_coverage",
            PresentedFloatPolicy::from(synthetic[1]),
        ),
        (
            "synthetic_ulps",
            "comparator_coverage",
            PresentedFloatPolicy::from(synthetic[2]),
        ),
    ];
    let matches_authority = policies.len() == expected.len()
        && policies.iter().zip(expected).all(|(actual, expected)| {
            actual.field == expected.0 && actual.scope == expected.1 && actual.policy == expected.2
        });
    if !matches_authority {
        return Err(PresentationError::AuthorityMismatch("float policies"));
    }
    Ok(())
}

fn validate_discrete_policies(
    policies: &[DiscretePolicyPresentation],
    profile: &ToleranceProfile,
) -> Result<(), PresentationError> {
    reject_duplicate_fields(policies.iter().map(|policy| policy.field.as_str()))?;
    if policies.len() != 1
        || policies[0].field != "world_counts"
        || policies[0].kind != profile.world_counts()
    {
        return Err(PresentationError::AuthorityMismatch("discrete policies"));
    }
    Ok(())
}

fn validate_collection_policies(
    policies: &[CollectionPolicyPresentation],
    profile: &ToleranceProfile,
) -> Result<(), PresentationError> {
    reject_duplicate_fields(policies.iter().map(|policy| policy.field.as_str()))?;
    if policies.len() != 1
        || policies[0].field != "checkpoints"
        || policies[0].kind != profile.checkpoints()
    {
        return Err(PresentationError::AuthorityMismatch("collection policies"));
    }
    Ok(())
}

fn reject_duplicate_fields<'a>(
    fields: impl Iterator<Item = &'a str>,
) -> Result<(), PresentationError> {
    let mut unique = BTreeSet::new();
    for field in fields {
        if !unique.insert(field) {
            return Err(PresentationError::DuplicatePolicy(field.to_owned()));
        }
    }
    Ok(())
}

fn render_protocol_schema() -> String {
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
            )
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

fn render_scenario_schema() -> String {
    render_json_schema(&json!({
        "$id": "https://liquidfun-rs.invalid/protocol/schemas/scenario-v1.schema.json",
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "additionalProperties": false,
        "description": SCHEMA_DESCRIPTION,
        "properties": {
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
        },
        "required": ["scenario_id", "source", "gravity_x_bits", "gravity_y_bits", "entities", "commands", "checkpoints"],
        "title": "liquidfun-rs scenario presentation version 1",
        "type": "object",
        "x-version-axes": { "scenario_schema_version": 1 }
    }))
}

fn render_trace_schema() -> String {
    render_json_schema(&json!({
        "$id": "https://liquidfun-rs.invalid/protocol/schemas/trace-v1.schema.json",
        "$schema": "https://json-schema.org/draft/2020-12/schema",
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

fn render_json_schema(document: &Value) -> String {
    let mut rendered = serde_json::to_string_pretty(&document)
        .expect("schema documents contain only JSON-native values");
    rendered.push('\n');
    rendered
}

fn closed_record(properties: &Value, required: &[&str]) -> Value {
    json!({
        "additionalProperties": false,
        "properties": properties,
        "required": required,
        "type": "object"
    })
}

fn version_schema() -> Value {
    json!({ "const": 1, "type": "integer" })
}

fn version_array_schema() -> Value {
    json!({ "items": version_schema(), "maxItems": 16, "minItems": 1, "type": "array" })
}

fn uint32_schema() -> Value {
    json!({ "maximum": u32::MAX, "minimum": 0, "type": "integer" })
}

fn uint64_schema() -> Value {
    json!({ "maximum": u64::MAX, "minimum": 0, "type": "integer" })
}

fn float_bits_schema() -> Value {
    uint32_schema()
}

fn semantic_id_schema() -> Value {
    json!({ "maxLength": 128, "pattern": "^[a-z0-9][a-z0-9._-]{0,127}$", "type": "string" })
}

fn bounded_string_schema() -> Value {
    json!({ "maxLength": 4096, "minLength": 1, "type": "string" })
}

fn sha256_schema() -> Value {
    json!({ "pattern": "^[0-9a-f]{64}$", "type": "string" })
}

fn scenario_source_schema() -> Value {
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
