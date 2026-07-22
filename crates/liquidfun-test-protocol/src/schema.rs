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

fn render_scenario_schema() -> String {
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

fn render_trace_schema() -> String {
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

fn physics_scenario_schema() -> Value {
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

fn math_probe_scenario_schema() -> Value {
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

fn probe_request_schema(record_kind: &str) -> Value {
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

fn collision_probe_scenario_schema() -> Value {
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

fn collision_witness_families() -> Value {
    serde_json::to_value(crate::CollisionWitnessFamily::REQUIRED.as_slice())
        .expect("closed witness-family enum serialization cannot fail")
}

fn collision_expected_outcome_schema() -> Value {
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

fn collision_probe_operations() -> Value {
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

fn collision_probe_policy_paths() -> Value {
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

fn collision_probe_horizon_schema() -> Value {
    json!({
        "oneOf": [
            closed_record(&json!({ "kind": { "const": "operation" } }), &["kind"]),
            closed_record(&json!({ "kind": { "const": "phase_local" } }), &["kind"])
        ]
    })
}

fn collision_probe_input_schema() -> Value {
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

fn collision_shape_schema() -> Value {
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

fn collision_cache_schema() -> Value {
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

fn collision_proxy_fingerprint_schema() -> Value {
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

fn collision_feature_schema() -> Value {
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

fn collision_tree_command_schema() -> Value {
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

fn nullable_schema(value: &Value) -> Value {
    json!({ "oneOf": [value, { "type": "null" }] })
}

fn math_probe_result_schema() -> Value {
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

fn math_probe_operations() -> Value {
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

fn math_probe_policy_paths() -> Value {
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

fn math_probe_horizon_schema() -> Value {
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

fn math_probe_input_schema() -> Value {
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

fn scenario_definitions() -> Value {
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

fn collision_probe_result_schema() -> Value {
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
