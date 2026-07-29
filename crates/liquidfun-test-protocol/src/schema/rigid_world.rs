use serde::Serialize;
use serde_json::{Value, json};

mod action;
mod declaration;
mod phase10;
mod phase9;
mod result;

use super::{
    bounded_string_schema, closed_record, float_bits_schema, scenario_source_schema, schema_ref,
    semantic_id_schema, sha256_schema, tagged_probe_input, transform_bits_schema, uint32_schema,
    vec2_bits_schema, version_schema,
};
use crate::{
    RIGID_WORLD_MAXIMUM_ACTIONS, RIGID_WORLD_MAXIMUM_CONTINUOUS_WORK,
    RIGID_WORLD_MAXIMUM_DIRECTIVES, RIGID_WORLD_MAXIMUM_ITERATIONS, RIGID_WORLD_MAXIMUM_JOINTS,
    RIGID_WORLD_MAXIMUM_ROPE_VERTICES, RIGID_WORLD_MAXIMUM_ROPES, RIGID_WORLD_POSITION_ITERATIONS,
    RIGID_WORLD_TIMESTEP_BITS, RIGID_WORLD_VELOCITY_ITERATIONS, RigidBodyKind,
    RigidContactEventKind, RigidFeatureKind, RigidJointBranchState, RigidJointKind,
    RigidLifecycleObservationKind, RigidManifoldKind, RigidPartialProgressClassification,
    RigidQueryCompletion, RigidQueryDirective, RigidRayCompletion, RigidReconstructionKind,
    RigidReconstructionSupport, RigidStepCompletion, RigidWakePolicy, RigidWorldWitness,
    RigidWorldWitnessFamily,
};
use action::*;
use declaration::*;
use phase9::{
    particle_action_schema, particle_declaration_schema, particle_observation_schema,
    particle_system_declaration_schema,
};
use phase10::{phase10_observation_schema, phase10_operation_schema};
use result::*;

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
                "maxItems": 19,
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
                "maxItems": 19,
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
