//! Closed Phase 9 witness-corpus and evidence-boundary tests.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use liquidfun_differential::{
    NativeRigidWorldExecutor, OracleExecutable, OraclePreset, PHASE9_REGISTRY_ID,
    PHASE9_REQUIRED_POLICY_PATHS, Phase9ComparisonOutcome, RigidComparisonOutcome,
    RigidMismatchReport, compare_complete_phase9_rigid_world_results,
    compare_phase8_rigid_world_results, effective_compile_command_sha256,
    execute_rigid_world_process, phase9_policy_for_path, run_phase9_differential,
};
use liquidfun_test_protocol::{
    HarnessLimits, Phase6PolicyProfile, Phase7PolicyProfile, Phase8PolicyProfile,
    Phase9ObservationKind, Phase9SemanticAssertion, Phase9WitnessBinding,
    Phase9WitnessBindingErrorKind, RigidWorldResultRecord, RigidWorldWitnessFamily, ScenarioId,
    decode_rigid_world_request_jsonl, decode_rigid_world_result_jsonl,
    validate_phase9_witness_bindings,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const RETAINED_REQUEST: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/rigid-world-request.jsonl");
const MANIFEST: &str = include_str!("fixtures/rigid_world/phase9/phase9-v1.json");
const PINNED_WITNESS: &[u8] =
    include_bytes!("../../../reference/artifacts/phase9/lifecycle-contact-witnesses.json");
const PHASE6_POLICY: &str = include_str!("../../../protocol/tolerances/phase6-v1.toml");
const PHASE7_POLICY: &str = include_str!("../../../protocol/tolerances/phase7-v1.toml");
const PHASE8_POLICY: &str = include_str!("../../../protocol/tolerances/phase8-v1.toml");
const PHASE6_POLICY_SHA256: &str =
    "7f10df148852866fd20d11b8d27adcddc0ad463ac3d3d716a8946ca5c8f1c63a";
const PHASE7_POLICY_SHA256: &str =
    "fd772b2cf523a6d40bf978bc4d0da18a4564181a93e6b2bdeb8e4d40d5613311";
const PHASE8_POLICY_SHA256: &str =
    "2843ca40bec5b1c680135664c58c12a8388a7a9e86ad77f8ef5a268f3f15a6bf";
const FAKE_PHASE9_RESULT_UNITS: [&str; 4] = [
    "collision_probe.cpp",
    "math_probe.cpp",
    "protocol_bits.cpp",
    "rigid_world.cpp",
];
const COMMON_ACTIONS: &[&str] = &[
    "create-growable",
    "create-fixed",
    "create-phase9-a",
    "create-phase9-b",
    "create-phase9-coupling",
    "create-phase9-capacity",
    "create-phase9-c",
    "create-phase9-d",
    "inspect-system",
    "inspect-particle",
    "resume",
    "statistics",
    "statistics-fixed",
    "phase9-step",
    "destroy-fixed",
    "destroy-growable",
];
const FORCE_ACTIONS: &[&str] = &[
    "position",
    "velocity",
    "force",
    "inspect-after-force",
    "impulse",
    "inspect-after-impulse",
];

const REQUIRED_BRANCHES: &[&str] = &[
    "multiple_systems",
    "newest_first",
    "paused_system",
    "stable_ids_sort",
    "stable_ids_compact",
    "optional_lanes",
    "fixed_buffer",
    "growable_buffer",
    "fixed_full",
    "teardown",
    "finite_lifetime",
    "infinite_lifetime",
    "equal_lifetime",
    "oldest_lifetime",
    "maximum_lifetime",
    "requested_destruction_callback",
    "unrequested_destruction_callback",
    "zombie_pending",
    "capacity_eviction",
    "particle_contact",
    "body_contact",
    "strict_contact_enabled",
    "strict_contact_disabled",
    "listener_flag_enabled",
    "listener_flag_disabled",
    "filter_flag_enabled",
    "filter_flag_disabled",
    "contact_order",
    "contact_multiplicity",
    "coupling_fields",
    "dynamic_body_reaction",
    "static_body_no_reaction",
    "force_range",
    "impulse_range",
    "statistics_counts",
    "collision_energy",
    "stuck_candidates",
    "system_aabb",
    "world_aabb",
    "system_culling",
    "query_continue",
    "query_terminate",
    "system_ray",
    "world_ray",
    "ray_culling",
    "ray_start_inside_exclusion",
    "ray_ignore",
    "ray_continue",
    "ray_clip",
    "ray_terminate",
    "retained_phase6_through_phase8",
    "phase10_rejection",
    "closed_policy_registry",
    "replay_identity",
    "minimization_identity",
    "first_divergence_stability",
    "d0_byte_identity",
    "debug_release_agreement",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CorpusManifest {
    profile: String,
    retained_request_sha256: String,
    pinned_upstream_revision: String,
    pinned_witness_sha256: String,
    forbidden_phase10_members: Vec<String>,
    cases: Vec<CorpusCase>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CorpusCase {
    case_id: String,
    authority: Authority,
    fixture: String,
    request_sha256: String,
    witnesses: Vec<Phase9WitnessBinding>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Authority {
    PinnedOracle,
    Independent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceManifest {
    schema_version: u32,
    case_record_schema_version: u32,
    profile: String,
    upstream_revision: String,
    semantic_manifest_sha256: String,
    cases: Vec<EvidenceCaseRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceCaseRecord {
    case_id: String,
    reached_branches: Vec<ScenarioId>,
    witnesses: Vec<Phase9WitnessBinding>,
    witness_binding_sha256: String,
    consumed_policy_paths: Vec<String>,
    retained_rigid: RetainedRigidRecord,
    request_path: String,
    request_sha256: String,
    native_result_path: String,
    native_result_sha256: String,
    oracle_result_path: String,
    oracle_result_sha256: String,
    complete_comparison_path: String,
    complete_comparison_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RetainedRigidRecord {
    comparator: String,
    phase6_policy_sha256: String,
    phase7_policy_sha256: String,
    phase8_policy_sha256: String,
    outcome: String,
    comparison_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RetainedRigidPayload<'a> {
    comparator: &'a str,
    phase6_policy_sha256: &'a str,
    phase7_policy_sha256: &'a str,
    phase8_policy_sha256: &'a str,
    outcome: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CompleteComparisonPayload {
    outcome: String,
    consumed_policy_paths: Vec<String>,
}

#[derive(Debug, Clone)]
struct EvidenceCasePayloads {
    request: Vec<u8>,
    native_result: Vec<u8>,
    oracle_result: Vec<u8>,
    complete_comparison: Vec<u8>,
}

fn manifest() -> CorpusManifest {
    serde_json::from_str(MANIFEST).expect("the checked-in Phase 9 manifest should be strict JSON")
}

fn request_value() -> Value {
    serde_json::from_slice(RETAINED_REQUEST).expect("the retained rigid request should be JSON")
}

fn decode_value(value: &Value) -> Result<liquidfun_test_protocol::RigidWorldRequestRecord, String> {
    let mut bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .map_err(|error| error.to_string())
}

fn bounded_phase9_request(case_id: &str) -> liquidfun_test_protocol::RigidWorldRequestRecord {
    let mut value = request_value();
    value["request_id"] = json!(format!("phase-09-{case_id}"));
    let timeline = &mut value["scenario"]["timelines"][0];
    configure_phase9_declarations(timeline, case_id);
    let mut phase9_actions = phase9_actions();
    order_phase9_actions(&mut phase9_actions, case_id);
    retain_relevant_actions(&mut phase9_actions, case_id);
    let final_action = phase9_actions
        .last()
        .expect("Phase 9 corpus should have a final action")["action_id"]
        .clone();
    let insertion_index = if case_id == "contacts-listeners-filters-and-coupling" {
        6
    } else {
        0
    };
    timeline["actions"]
        .as_array_mut()
        .expect("retained actions should be an array")
        .splice(insertion_index..insertion_index, phase9_actions);
    insert_phase9_checkpoints(timeline, case_id, &final_action);
    decode_value(&value).expect("the bounded Phase 9 corpus should decode")
}

fn configure_phase9_declarations(timeline: &mut Value, case_id: &str) {
    let contact_case = case_id == "contacts-listeners-filters-and-coupling";
    let fixed_offset = if contact_case { 18.515 } else { 0.0 };
    let fixed_d_x = if contact_case { fixed_offset } else { 0.4 };
    let fixed_c_y = if contact_case { -0.2 } else { 0.0 };
    let fixed_d_y = if contact_case { 0.2 } else { 0.0 };
    let (coupling_x, coupling_y) = if contact_case {
        (0.75, 0.0)
    } else {
        (20.0, 0.25)
    };
    timeline["particle_systems"] = json!([
        {
            "system_id": "phase9-growable", "buffer_mode": { "kind": "growable", "initial_capacity": 4 },
            "paused": false, "strict_contact_check": false, "stuck_threshold": 1,
            "density_bits": 1.0_f32.to_bits(), "gravity_scale_bits": 0.0_f32.to_bits(),
            "radius_bits": 0.25_f32.to_bits(), "damping_bits": 0.0_f32.to_bits(),
            "destruction_by_age": true, "lifetime_granularity_bits": (1.0_f32 / 60.0_f32).to_bits(),
            "maximum_count": 4
        },
        {
            "system_id": "phase9-fixed-paused", "buffer_mode": { "kind": "fixed", "capacity": 2 },
            "paused": true, "strict_contact_check": true, "stuck_threshold": 1,
            "density_bits": 1.0_f32.to_bits(), "gravity_scale_bits": 1.0_f32.to_bits(),
            "radius_bits": 0.25_f32.to_bits(), "damping_bits": 0.25_f32.to_bits(),
            "destruction_by_age": true, "lifetime_granularity_bits": (1.0_f32 / 60.0_f32).to_bits(),
            "maximum_count": 2
        }
    ]);
    timeline["particles"] = json!([
        particle_with_flags(
            "phase9-a",
            "phase9-growable",
            0.0,
            0.05,
            0,
            (1 << 9) | (1 << 14) | (1 << 16)
        ),
        particle_with_flags(
            "phase9-b",
            "phase9-growable",
            0.4,
            -1.0,
            1,
            (1 << 9) | (1 << 15) | (1 << 17)
        ),
        particle_with_velocity(
            "phase9-coupling",
            "phase9-growable",
            coupling_x,
            coupling_y,
            -2.0,
            0.0,
            2.0,
            4
        ),
        particle("phase9-capacity", "phase9-growable", 3.0, 2.0, 5),
        particle("phase9-evicting", "phase9-growable", 4.0, 4.0, 6),
        particle_with_velocity(
            "phase9-c",
            "phase9-fixed-paused",
            fixed_offset,
            fixed_c_y,
            0.0,
            0.0,
            2.0,
            2
        ),
        particle_with_flags_and_velocity(
            "phase9-d",
            "phase9-fixed-paused",
            fixed_d_x,
            fixed_d_y,
            0.0,
            0.0,
            2.0,
            3,
            (1 << 9) | (1 << 15)
        ),
        particle("phase9-e", "phase9-fixed-paused", 0.8, 2.0, 9)
    ]);
}

fn phase9_actions() -> Vec<Value> {
    let mut phase9_actions = vec![
        action(
            "create-growable",
            json!({ "kind": "create_system", "system_id": "phase9-growable" }),
        ),
        action(
            "create-fixed",
            json!({ "kind": "create_system", "system_id": "phase9-fixed-paused" }),
        ),
    ];
    for id in [
        "phase9-a",
        "phase9-b",
        "phase9-coupling",
        "phase9-capacity",
        "phase9-c",
        "phase9-d",
    ] {
        phase9_actions.push(action(
            &format!("create-{id}"),
            json!({ "kind": "create_particle", "particle_id": id }),
        ));
    }
    phase9_actions.extend([
        action("inspect-system", json!({ "kind": "inspect_system", "system_id": "phase9-growable" })),
        action("inspect-particle", json!({ "kind": "inspect_particle", "particle_id": "phase9-a" })),
        action("resume", json!({ "kind": "set_paused", "system_id": "phase9-fixed-paused", "paused": false })),
        json!({
            "action_id": "phase9-step",
            "phase": "phase9",
            "action": {
                "kind": "step",
                "timestep_bits": (1.0_f32 / 60.0_f32).to_bits(),
                "velocity_iterations": 8,
                "position_iterations": 3
            }
        }),
        action("energy-velocity-c", json!({ "kind": "set_velocity", "particle_id": "phase9-c", "velocity": bits(1.0, 0.0) })),
        action("energy-velocity-d", json!({ "kind": "set_velocity", "particle_id": "phase9-d", "velocity": bits(-1.0, 0.0) })),
        action("statistics", json!({ "kind": "request_statistics", "system_id": "phase9-growable" })),
        action("statistics-fixed", json!({ "kind": "request_statistics", "system_id": "phase9-fixed-paused" })),
        action("inspect-occurrence-zero", json!({ "kind": "inspect_occurrence", "occurrence_index": 0 })),
        action("inspect-occurrence-one", json!({ "kind": "inspect_occurrence", "occurrence_index": 1 })),
        action("inspect-system-after-step", json!({ "kind": "inspect_system", "system_id": "phase9-growable" })),
        action("system-query", json!({ "kind": "query_aabb", "system_id": "phase9-growable", "lower": bits(-1.0, -1.0), "upper": bits(2.0, 2.0), "control": "continue" })),
        action("world-query", json!({ "kind": "query_aabb", "system_id": null, "lower": bits(-1.0, -1.0), "upper": bits(2.0, 2.0), "control": "continue" })),
        action("query-terminate", json!({ "kind": "query_aabb", "system_id": "phase9-growable", "lower": bits(-1.0, -1.0), "upper": bits(2.0, 2.0), "control": "terminate" })),
        action("system-ray", json!({ "kind": "ray_cast", "system_id": "phase9-growable", "start": bits(-1.0, 0.0), "end": bits(2.0, 0.0), "control": "continue" })),
        action("world-ray", json!({ "kind": "ray_cast", "system_id": null, "start": bits(-1.0, 0.0), "end": bits(2.0, 0.0), "control": "continue" })),
        action("ray-ignore", json!({ "kind": "ray_cast", "system_id": "phase9-growable", "start": bits(-1.0, 0.0), "end": bits(0.1, 0.0), "control": "ignore" })),
        action("ray-clip", json!({ "kind": "ray_cast", "system_id": "phase9-growable", "start": bits(-1.0, 0.0), "end": bits(2.0, 0.0), "control": "clip" })),
        action("ray-terminate", json!({ "kind": "ray_cast", "system_id": "phase9-growable", "start": bits(-1.0, 0.0), "end": bits(2.0, 0.0), "control": "terminate" })),
        action("position", json!({ "kind": "set_position", "particle_id": "phase9-a", "position": bits(0.25, 0.0) })),
        action("velocity", json!({ "kind": "set_velocity", "particle_id": "phase9-a", "velocity": bits(0.0, 1.0) })),
        action("force", json!({ "kind": "apply_force", "particle_ids": ["phase9-a", "phase9-b"], "force": bits(1.0, 0.0) })),
        action("inspect-after-force", json!({ "kind": "inspect_particle", "particle_id": "phase9-a" })),
        action("impulse", json!({ "kind": "apply_impulse", "particle_ids": ["phase9-a", "phase9-b"], "impulse": bits(0.0, 1.0) })),
        action("inspect-after-impulse", json!({ "kind": "inspect_particle", "particle_id": "phase9-a" })),
        action("create-evicting", json!({ "kind": "create_particle", "particle_id": "phase9-evicting" })),
        action("create-phase9-e", json!({ "kind": "create_particle", "particle_id": "phase9-e" })),
        action("mark", json!({ "kind": "mark_for_destruction", "particle_id": "phase9-b" })),
        action("compact", json!({ "kind": "compact", "system_id": "phase9-growable" })),
        action("mark-unrequested", json!({ "kind": "mark_for_destruction", "particle_id": "phase9-capacity" })),
        action("compact-unrequested", json!({ "kind": "compact", "system_id": "phase9-growable" })),
        action("destroy-fixed", json!({ "kind": "destroy_system", "system_id": "phase9-fixed-paused" })),
        action("destroy-growable", json!({ "kind": "destroy_system", "system_id": "phase9-growable" })),
    ]);
    phase9_actions
}

fn order_phase9_actions(phase9_actions: &mut Vec<Value>, case_id: &str) {
    let statistics = ["statistics", "statistics-fixed"]
        .into_iter()
        .map(|action_id| {
            let index = phase9_actions
                .iter()
                .position(|record| record["action_id"] == action_id)
                .expect("the bounded corpus retains its statistics actions");
            phase9_actions.remove(index)
        })
        .collect::<Vec<_>>();
    let step_index = phase9_actions
        .iter()
        .position(|record| record["action_id"] == "phase9-step")
        .expect("the bounded corpus retains its particle step");
    let statistics_index = if case_id == "forces-impulses-and-statistics" {
        phase9_actions
            .iter()
            .position(|record| record["action_id"] == "energy-velocity-d")
            .expect("the force case retains its collision-energy velocity setup")
            + 1
    } else {
        step_index + 1
    };
    phase9_actions.splice(statistics_index..statistics_index, statistics);
    if matches!(
        case_id,
        "lifetime-zombie-and-eviction" | "contacts-listeners-filters-and-coupling"
    ) {
        let first_step = phase9_actions
            .iter()
            .position(|record| record["action_id"] == "phase9-step")
            .expect("the lifetime case retains its first particle step");
        let step_template = phase9_actions[first_step].clone();
        let final_step = if case_id == "lifetime-zombie-and-eviction" {
            4
        } else {
            3
        };
        phase9_actions.splice(
            first_step + 1..first_step + 1,
            (2..=final_step).map(|ordinal| {
                let mut step = step_template.clone();
                step["action_id"] = json!(format!("phase9-step-{ordinal}"));
                step
            }),
        );
    }
    if case_id == "contacts-listeners-filters-and-coupling" {
        let step_index = phase9_actions
            .iter()
            .position(|record| record["action_id"] == "phase9-step")
            .expect("the contact case retains its particle step")
            + 1;
        phase9_actions.splice(
            step_index..step_index,
            [
                action(
                    "inspect-particle-contact",
                    json!({ "kind": "inspect_particle_contact", "system_id": "phase9-fixed-paused", "contact_index": 0 }),
                ),
                action(
                    "inspect-body-contact",
                    json!({ "kind": "inspect_body_contact", "system_id": "phase9-growable", "contact_index": 1 }),
                ),
                action(
                    "contact-statistics-growable",
                    json!({ "kind": "request_statistics", "system_id": "phase9-growable" }),
                ),
                action(
                    "contact-statistics-fixed",
                    json!({ "kind": "request_statistics", "system_id": "phase9-fixed-paused" }),
                ),
            ],
        );
    }
    if case_id == "forces-impulses-and-statistics" {
        let mut moved = Vec::new();
        for action_id in FORCE_ACTIONS {
            let index = phase9_actions
                .iter()
                .position(|record| record["action_id"] == *action_id)
                .expect("the force case retains its pre-step action");
            moved.push(phase9_actions.remove(index));
        }
        let step_index = phase9_actions
            .iter()
            .position(|record| record["action_id"] == "phase9-step")
            .expect("the force case retains its particle step");
        phase9_actions.splice(step_index..step_index, moved);
    }
}

fn retain_relevant_actions(phase9_actions: &mut Vec<Value>, case_id: &str) {
    let relevant: &[&str] = match case_id {
        "storage-systems-and-permutations" | "lifetime-zombie-and-eviction" => &[
            "create-evicting",
            "create-phase9-e",
            "mark",
            "compact",
            "mark-unrequested",
            "compact-unrequested",
            "inspect-system-after-step",
        ],
        "contacts-listeners-filters-and-coupling" => &[
            "inspect-particle-contact",
            "inspect-body-contact",
            "inspect-occurrence-zero",
            "contact-statistics-growable",
            "contact-statistics-fixed",
        ],
        "forces-impulses-and-statistics" => &[
            "position",
            "velocity",
            "force",
            "inspect-after-force",
            "impulse",
            "inspect-after-impulse",
            "energy-velocity-c",
            "energy-velocity-d",
        ],
        "aabb-query-control-and-culling" => &["system-query", "world-query", "query-terminate"],
        "ray-control-and-culling" => &[
            "system-ray",
            "world-ray",
            "ray-ignore",
            "ray-clip",
            "ray-terminate",
        ],
        "closed-evidence-contract" => &[],
        other => panic!("unreviewed Phase 9 corpus case {other}"),
    };
    phase9_actions.retain(|record| {
        let action_id = record["action_id"].as_str().expect("action ID");
        COMMON_ACTIONS.contains(&action_id)
            || relevant.contains(&action_id)
            || action_id.starts_with("phase9-step-")
    });
}

fn insert_phase9_checkpoints(timeline: &mut Value, case_id: &str, final_action: &Value) {
    let checkpoints = timeline["checkpoints"]
        .as_array_mut()
        .expect("retained checkpoints should be an array");
    let checkpoint_index = usize::from(case_id == "contacts-listeners-filters-and-coupling");
    let first_rigid_counts = if case_id == "contacts-listeners-filters-and-coupling" {
        json!({
            "bodies": 3, "fixtures": 3, "contacts": 0,
            "manifold_points": 0, "events": 0, "destructions": 0
        })
    } else {
        json!({
            "bodies": 0, "fixtures": 0, "contacts": 0,
            "manifold_points": 0, "events": 0, "destructions": 0
        })
    };
    checkpoints.insert(
        checkpoint_index,
        json!({
            "checkpoint_id": "phase9-only-checkpoint",
            "after_action_id": "inspect-system",
            "phase": "phase9",
            "counts": first_rigid_counts,
            "transitions": []
        }),
    );
    checkpoints.insert(
        checkpoint_index + 1,
        json!({
            "checkpoint_id": "phase9-corpus",
            "after_action_id": final_action,
            "phase": "phase9",
            "counts": {
                "bodies": if case_id == "contacts-listeners-filters-and-coupling" { 3 } else { 0 },
                "fixtures": if case_id == "contacts-listeners-filters-and-coupling" { 3 } else { 0 },
                "contacts": 0,
                "manifold_points": 0,
                "events": 0,
                "destructions": 0
            },
            "transitions": []
        }),
    );
}

fn particle(id: &str, system: &str, x: f32, lifetime: f32, color: u8) -> Value {
    particle_with_flags(id, system, x, lifetime, color, 0)
}

fn particle_with_flags(
    id: &str,
    system: &str,
    x: f32,
    lifetime: f32,
    color: u8,
    flags_bits: u32,
) -> Value {
    json!({
        "particle_id": id, "system_id": system, "position": bits(x, 0.0),
        "velocity": bits(0.0, 0.0), "flags_bits": flags_bits, "color": [color, 0, 255, 255],
        "lifetime_bits": lifetime.to_bits()
    })
}

#[allow(clippy::too_many_arguments)]
fn particle_with_velocity(
    id: &str,
    system: &str,
    x: f32,
    y: f32,
    velocity_x: f32,
    velocity_y: f32,
    lifetime: f32,
    color: u8,
) -> Value {
    json!({
        "particle_id": id, "system_id": system, "position": bits(x, y),
        "velocity": bits(velocity_x, velocity_y), "flags_bits": 0,
        "color": [color, 0, 255, 255], "lifetime_bits": lifetime.to_bits()
    })
}

#[allow(clippy::too_many_arguments)]
fn particle_with_flags_and_velocity(
    id: &str,
    system: &str,
    x: f32,
    y: f32,
    velocity_x: f32,
    velocity_y: f32,
    lifetime: f32,
    color: u8,
    flags_bits: u32,
) -> Value {
    let mut particle =
        particle_with_velocity(id, system, x, y, velocity_x, velocity_y, lifetime, color);
    particle["flags_bits"] = json!(flags_bits);
    particle
}

fn bits(x: f32, y: f32) -> Value {
    json!({ "x_bits": x.to_bits(), "y_bits": y.to_bits() })
}

fn action(id: &str, action: Value) -> Value {
    let mut record = json!({
        "action_id": id, "phase": "phase9",
        "action": { "kind": "particle" }
    });
    record["action"]["action"] = action;
    record
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn canonical_sha256(value: &impl Serialize) -> String {
    sha256(&serde_json::to_vec(value).expect("canonical evidence JSON"))
}

fn retained_rigid_record() -> RetainedRigidRecord {
    let payload = RetainedRigidPayload {
        comparator: "phase8-v1",
        phase6_policy_sha256: PHASE6_POLICY_SHA256,
        phase7_policy_sha256: PHASE7_POLICY_SHA256,
        phase8_policy_sha256: PHASE8_POLICY_SHA256,
        outcome: "match",
    };
    RetainedRigidRecord {
        comparator: payload.comparator.to_owned(),
        phase6_policy_sha256: payload.phase6_policy_sha256.to_owned(),
        phase7_policy_sha256: payload.phase7_policy_sha256.to_owned(),
        phase8_policy_sha256: payload.phase8_policy_sha256.to_owned(),
        outcome: payload.outcome.to_owned(),
        comparison_sha256: canonical_sha256(&payload),
    }
}

fn evidence_payload_paths(case_id: &str) -> (String, String, String, String) {
    let base = format!("cases/{case_id}");
    (
        format!("{base}/request.jsonl"),
        format!("{base}/native-result.json"),
        format!("{base}/oracle-result.json"),
        format!("{base}/complete-comparison.json"),
    )
}

fn evidence_case_record(
    case: &CorpusCase,
    payloads: &EvidenceCasePayloads,
    consumed_policy_paths: &[&str],
) -> EvidenceCaseRecord {
    let (request_path, native_result_path, oracle_result_path, complete_comparison_path) =
        evidence_payload_paths(&case.case_id);
    EvidenceCaseRecord {
        case_id: case.case_id.clone(),
        reached_branches: case
            .witnesses
            .iter()
            .map(|witness| witness.branch_id.clone())
            .collect(),
        witness_binding_sha256: canonical_sha256(&case.witnesses),
        witnesses: case.witnesses.clone(),
        consumed_policy_paths: consumed_policy_paths
            .iter()
            .map(|path| (*path).to_owned())
            .collect(),
        retained_rigid: retained_rigid_record(),
        request_path,
        request_sha256: sha256(&payloads.request),
        native_result_path,
        native_result_sha256: sha256(&payloads.native_result),
        oracle_result_path,
        oracle_result_sha256: sha256(&payloads.oracle_result),
        complete_comparison_path,
        complete_comparison_sha256: sha256(&payloads.complete_comparison),
    }
}

fn validate_evidence_case_value(
    value: &Value,
    payloads: &EvidenceCasePayloads,
) -> Result<(), String> {
    if value.get("retained_rigid").is_none() {
        return Err("missing retained-rigid proof".to_owned());
    }
    let record: EvidenceCaseRecord =
        serde_json::from_value(value.clone()).map_err(|error| error.to_string())?;
    let retained = &record.retained_rigid;
    if retained.comparator != "phase8-v1"
        || retained.phase6_policy_sha256 != PHASE6_POLICY_SHA256
        || retained.phase7_policy_sha256 != PHASE7_POLICY_SHA256
        || retained.phase8_policy_sha256 != PHASE8_POLICY_SHA256
    {
        return Err("retained-rigid policy digest mismatch".to_owned());
    }
    if retained.outcome != "match" {
        return Err("retained-rigid outcome mismatch".to_owned());
    }
    let retained_payload = RetainedRigidPayload {
        comparator: &retained.comparator,
        phase6_policy_sha256: &retained.phase6_policy_sha256,
        phase7_policy_sha256: &retained.phase7_policy_sha256,
        phase8_policy_sha256: &retained.phase8_policy_sha256,
        outcome: &retained.outcome,
    };
    if retained.comparison_sha256 != canonical_sha256(&retained_payload) {
        return Err("retained-rigid comparison digest mismatch".to_owned());
    }
    if record.witness_binding_sha256 != canonical_sha256(&record.witnesses) {
        return Err("witness binding digest mismatch".to_owned());
    }
    if record.request_sha256 != sha256(&payloads.request) {
        return Err("request payload digest mismatch".to_owned());
    }
    if record.native_result_sha256 != sha256(&payloads.native_result) {
        return Err("native result payload digest mismatch".to_owned());
    }
    if record.oracle_result_sha256 != sha256(&payloads.oracle_result) {
        return Err("oracle result payload digest mismatch".to_owned());
    }
    if record.complete_comparison_sha256 != sha256(&payloads.complete_comparison) {
        return Err("complete comparison payload digest mismatch".to_owned());
    }
    Ok(())
}

fn evidence_case_fixture() -> (Value, EvidenceCasePayloads) {
    let witness = valid_witness_binding("multiple_systems");
    let case = CorpusCase {
        case_id: "test-case".to_owned(),
        authority: Authority::Independent,
        fixture: "test-case.jsonl".to_owned(),
        request_sha256: sha256(b"request"),
        witnesses: vec![witness],
    };
    let complete_comparison = serde_json::to_vec(&CompleteComparisonPayload {
        outcome: "match".to_owned(),
        consumed_policy_paths: PHASE9_REQUIRED_POLICY_PATHS
            .iter()
            .map(|path| (*path).to_owned())
            .collect(),
    })
    .expect("comparison fixture");
    let payloads = EvidenceCasePayloads {
        request: b"request".to_vec(),
        native_result: b"native".to_vec(),
        oracle_result: b"oracle".to_vec(),
        complete_comparison,
    };
    let record = evidence_case_record(&case, &payloads, PHASE9_REQUIRED_POLICY_PATHS);
    (
        serde_json::to_value(record).expect("evidence fixture"),
        payloads,
    )
}

fn write_evidence_payload(root: &Path, relative: &str, bytes: &[u8]) {
    let path = root.join(relative);
    let parent = path.parent().expect("evidence payload parent");
    fs::create_dir_all(parent).expect("evidence payload directory");
    fs::write(path, bytes).expect("evidence payload");
}

fn fixture_path(case: &CorpusCase) -> std::path::PathBuf {
    let relative = Path::new(&case.fixture);
    assert!(!relative.is_absolute(), "fixture paths must be relative");
    assert!(
        relative
            .components()
            .all(|component| matches!(component, Component::Normal(_))),
        "fixture paths must not escape the Phase 9 corpus"
    );
    assert_eq!(
        relative.extension().and_then(|value| value.to_str()),
        Some("jsonl")
    );
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/rigid_world/phase9")
        .join(relative)
}

fn observation_for_witness<'a>(
    request: &'a Value,
    result: &'a Value,
    witness: &Phase9WitnessBinding,
) -> &'a Value {
    let timeline = &request["scenario"]["timelines"][0];
    let actions = timeline["actions"].as_array().expect("actions");
    let checkpoints = timeline["checkpoints"].as_array().expect("checkpoints");
    let action = actions
        .get(witness.action_index)
        .expect("typed witness action index must exist");
    let checkpoint = checkpoints
        .get(witness.checkpoint_index)
        .expect("typed witness checkpoint index must exist");
    observation_for_action(
        request,
        result,
        action["action_id"].as_str().expect("action ID"),
        checkpoint["checkpoint_id"].as_str().expect("checkpoint ID"),
    )
}

fn observation_for_action<'a>(
    request: &'a Value,
    result: &'a Value,
    action_id: &str,
    checkpoint_id: &str,
) -> &'a Value {
    let timeline = &request["scenario"]["timelines"][0];
    let actions = timeline["actions"].as_array().expect("actions");
    let checkpoints = timeline["checkpoints"].as_array().expect("checkpoints");
    let checkpoint_index = checkpoints
        .iter()
        .position(|checkpoint| checkpoint["checkpoint_id"] == checkpoint_id)
        .expect("witness checkpoint must exist");
    let action_end = actions
        .iter()
        .position(|action| action["action_id"] == checkpoints[checkpoint_index]["after_action_id"])
        .expect("checkpoint action must exist");
    let action_start = if checkpoint_index == 0 {
        0
    } else {
        actions
            .iter()
            .position(|action| {
                action["action_id"] == checkpoints[checkpoint_index - 1]["after_action_id"]
            })
            .expect("previous checkpoint action must exist")
            + 1
    };
    let target = actions[action_start..=action_end]
        .iter()
        .position(|action| action["action_id"] == action_id)
        .expect("witness action must belong to its checkpoint");
    assert_eq!(
        actions[action_start + target]["action"]["kind"],
        "particle",
        "Phase 9 semantic witnesses must name particle actions"
    );
    let observation_index = actions[action_start..action_start + target]
        .iter()
        .filter(|action| action["action"]["kind"] == "particle")
        .count();
    &result["timelines"][0]["checkpoints"][checkpoint_index]["observations"][observation_index]
}

fn phase9_observation<'a>(request: &'a Value, result: &'a Value, action_id: &str) -> &'a Value {
    let checkpoint_id = if action_id == "inspect-system" {
        "phase9-only-checkpoint"
    } else {
        "phase9-corpus"
    };
    &observation_for_action(request, result, action_id, checkpoint_id)["observation"]
}

fn particle_declaration<'a>(request: &'a Value, particle_id: &str) -> &'a Value {
    request["scenario"]["timelines"][0]["particles"]
        .as_array()
        .expect("particle declarations")
        .iter()
        .find(|particle| particle["particle_id"] == particle_id)
        .expect("declared particle")
}

fn system_declaration<'a>(request: &'a Value, system_id: &str) -> &'a Value {
    request["scenario"]["timelines"][0]["particle_systems"]
        .as_array()
        .expect("system declarations")
        .iter()
        .find(|system| system["system_id"] == system_id)
        .expect("declared system")
}

fn phase9_checkpoint<'a>(result: &'a Value, checkpoint_id: &str) -> &'a Value {
    result["timelines"][0]["checkpoints"]
        .as_array()
        .expect("result checkpoints")
        .iter()
        .find(|checkpoint| checkpoint["checkpoint_id"] == checkpoint_id)
        .expect("Phase 9 result checkpoint")
}

fn assert_no_particle_lifecycle(result: &Value, particle_id: &str) {
    let occurrences = phase9_checkpoint(result, "phase9-corpus")["observations"]
        .as_array()
        .expect("Phase 9 observations")
        .iter()
        .filter(|observation| {
            observation["observation"]["kind"] == "lifecycle"
                && observation["observation"]["occurrence"]["maybe_particle_id"] == particle_id
        })
        .count();
    assert_eq!(
        occurrences, 0,
        "{particle_id} must not emit a lifecycle occurrence"
    );
}

fn assert_observed_semantic(request: &Value, result: &Value, branch: &str) {
    let asserted = assert_system_witness(request, result, branch)
        || assert_lifecycle_witness(request, result, branch)
        || assert_contact_witness(request, result, branch)
        || assert_query_witness(request, result, branch)
        || assert_contract_witness(request, result, branch);
    assert!(
        asserted,
        "missing semantic assertion for Phase 9 branch {branch}"
    );
}

fn assert_system_witness(request: &Value, result: &Value, branch: &str) -> bool {
    let observe = |action_id| phase9_observation(request, result, action_id);
    let timeline = &request["scenario"]["timelines"][0];
    let growable = system_declaration(request, "phase9-growable");
    let fixed = system_declaration(request, "phase9-fixed-paused");
    let statistics = observe("statistics");
    match branch {
        "multiple_systems" | "newest_first" => {
            assert_eq!(statistics["statistics"]["system_count"], 2);
            assert_eq!(
                timeline["particle_systems"][0]["system_id"],
                "phase9-growable"
            );
            assert_eq!(
                timeline["particle_systems"][1]["system_id"],
                "phase9-fixed-paused"
            );
        }
        "paused_system" => {
            assert_eq!(fixed["paused"], true);
            assert_eq!(
                observe("statistics-fixed")["statistics"]["particle_count"],
                2
            );
        }
        "stable_ids_sort" => assert_eq!(
            observe("inspect-system")["particle_ids"],
            json!(["phase9-a", "phase9-b", "phase9-coupling", "phase9-capacity"])
        ),
        "stable_ids_compact" => assert_eq!(
            observe("compact-unrequested")["particle_ids"],
            json!(["phase9-coupling", "phase9-evicting", "phase9-c", "phase9-e"])
        ),
        "optional_lanes" => {
            let snapshot = &observe("inspect-particle")["snapshot"];
            assert_eq!(snapshot["color"], json!([0, 0, 255, 255]));
            assert_eq!(snapshot["force"], bits(0.0, 0.0));
            assert_eq!(snapshot["weight_bits"], 0);
        }
        "fixed_buffer" => assert_eq!(
            fixed["buffer_mode"],
            json!({ "kind": "fixed", "capacity": 2 })
        ),
        "growable_buffer" => assert_eq!(
            growable["buffer_mode"],
            json!({ "kind": "growable", "initial_capacity": 4 })
        ),
        "fixed_full" => {
            let fixed_statistics = &observe("statistics-fixed")["statistics"];
            assert_eq!(fixed_statistics["particle_count"], 2);
            assert_eq!(fixed_statistics["effective_capacity"], 2);
        }
        "teardown" => {
            assert_eq!(
                observe("destroy-fixed")["occurrence"]["kind"],
                "system_destroyed"
            );
            assert_eq!(
                observe("destroy-fixed")["occurrence"]["system_id"],
                "phase9-fixed-paused"
            );
            assert_eq!(
                observe("destroy-growable")["occurrence"]["system_id"],
                "phase9-growable"
            );
        }
        "retained_phase6_through_phase8" => assert_eq!(
            request["scenario"]["timelines"]
                .as_array()
                .expect("timelines")
                .len(),
            RigidWorldWitnessFamily::ALL.len()
        ),
        "phase10_rejection" => {
            for member in [
                "particle_groups",
                "particle_pairs",
                "particle_triads",
                "particle_solver",
            ] {
                assert!(
                    timeline.get(member).is_none(),
                    "{member} must remain absent"
                );
            }
        }
        _ => return false,
    }
    true
}

fn assert_lifecycle_witness(request: &Value, result: &Value, branch: &str) -> bool {
    let observe = |action_id| phase9_observation(request, result, action_id);
    let growable = system_declaration(request, "phase9-growable");
    let a = particle_declaration(request, "phase9-a");
    let b = particle_declaration(request, "phase9-b");
    let statistics = observe("statistics");
    match branch {
        "finite_lifetime" => assert_eq!(a["lifetime_bits"], 0.05_f32.to_bits()),
        "infinite_lifetime" => assert_eq!(b["lifetime_bits"], (-1.0_f32).to_bits()),
        "equal_lifetime" => assert_eq!(
            particle_declaration(request, "phase9-c")["lifetime_bits"],
            particle_declaration(request, "phase9-d")["lifetime_bits"]
        ),
        "oldest_lifetime" | "capacity_eviction" => {
            let occurrence = &observe("create-evicting")["occurrence"];
            assert_eq!(occurrence["kind"], "particle_destroyed");
            assert_eq!(occurrence["maybe_particle_id"], "phase9-a");
        }
        "maximum_lifetime" => {
            assert_eq!(growable["maximum_count"], 4);
            assert_eq!(statistics["statistics"]["effective_capacity"], 4);
        }
        "requested_destruction_callback" => {
            let occurrence = &observe("compact")["occurrence"];
            assert_eq!(occurrence["kind"], "particle_destroyed");
            assert_eq!(occurrence["maybe_particle_id"], "phase9-b");
        }
        "unrequested_destruction_callback" => {
            assert_eq!(observe("compact-unrequested")["kind"], "mixed_state");
            assert_no_particle_lifecycle(result, "phase9-capacity");
        }
        "zombie_pending" => {
            assert_eq!(observe("mark")["kind"], "mixed_state");
            assert!(
                observe("mark")["particle_ids"]
                    .as_array()
                    .expect("pending particle IDs")
                    .iter()
                    .any(|particle_id| particle_id == "phase9-b")
            );
            assert_eq!(
                observe("compact")["occurrence"]["maybe_particle_id"],
                "phase9-b"
            );
        }
        _ => return false,
    }
    true
}

fn assert_contact_witness(request: &Value, result: &Value, branch: &str) -> bool {
    let observe = |action_id| phase9_observation(request, result, action_id);
    let growable = system_declaration(request, "phase9-growable");
    let fixed = system_declaration(request, "phase9-fixed-paused");
    let a = particle_declaration(request, "phase9-a");
    let b = particle_declaration(request, "phase9-b");
    match branch {
        "particle_contact" => assert_eq!(
            observe("inspect-particle-contact")["contact"]["system_id"],
            "phase9-fixed-paused"
        ),
        "body_contact" => assert_eq!(
            observe("inspect-body-contact")["contact"]["fixture_id"],
            "nc-kinematic-fixture"
        ),
        "strict_contact_enabled" => assert_eq!(fixed["strict_contact_check"], true),
        "strict_contact_disabled" => assert_eq!(growable["strict_contact_check"], false),
        "listener_flag_enabled" => {
            assert_ne!(b["flags_bits"].as_u64().expect("flags") & (1 << 15), 0);
        }
        "listener_flag_disabled" => assert_eq!(
            particle_declaration(request, "phase9-capacity")["flags_bits"],
            0
        ),
        "filter_flag_enabled" => {
            assert_ne!(a["flags_bits"].as_u64().expect("flags") & (1 << 16), 0);
        }
        "filter_flag_disabled" => assert_eq!(
            particle_declaration(request, "phase9-coupling")["flags_bits"],
            0
        ),
        "contact_order" => {
            let particle_contact = observe("inspect-particle-contact");
            assert_eq!(particle_contact["contact"]["particle_a_id"], "phase9-c");
            assert_eq!(particle_contact["contact"]["particle_b_id"], "phase9-d");
        }
        "contact_multiplicity" => {
            let particle_contact = observe("inspect-particle-contact");
            assert_ne!(
                particle_contact["contact"]["particle_a_id"],
                particle_contact["contact"]["particle_b_id"]
            );
            assert!(
                particle_contact["contact"]["weight_bits"]
                    .as_u64()
                    .expect("weight")
                    > 0
            );
        }
        "coupling_fields" => {
            let body_contact = observe("inspect-body-contact");
            assert_eq!(body_contact["contact"]["particle_id"], "phase9-coupling");
            assert!(body_contact["contact"]["mass_bits"].as_u64().expect("mass") > 0);
            assert!(
                body_contact["contact"]["weight_bits"]
                    .as_u64()
                    .expect("weight")
                    > 0
            );
        }
        "dynamic_body_reaction" => {
            let body = phase9_checkpoint(result, "phase9-corpus")["bodies"]
                .as_array()
                .expect("bodies")
                .iter()
                .find(|body| body["body_id"] == "nc-dynamic")
                .expect("dynamic body");
            assert!(
                body["linear_velocity"]["x_bits"] != 0 || body["linear_velocity"]["y_bits"] != 0
            );
        }
        "static_body_no_reaction" => {
            let body = phase9_checkpoint(result, "phase9-corpus")["bodies"]
                .as_array()
                .expect("bodies")
                .iter()
                .find(|body| body["body_id"] == "nc-static")
                .expect("static body");
            assert_eq!(body["linear_velocity"], bits(0.0, 0.0));
        }
        _ => return false,
    }
    true
}

fn assert_query_witness(request: &Value, result: &Value, branch: &str) -> bool {
    let observe = |action_id| phase9_observation(request, result, action_id);
    let statistics = observe("statistics");
    match branch {
        "force_range" => assert_ne!(
            observe("inspect-after-force")["snapshot"]["force"]["x_bits"],
            0
        ),
        "impulse_range" => assert_ne!(
            observe("inspect-after-impulse")["snapshot"]["velocity"]["y_bits"],
            0
        ),
        "statistics_counts" => {
            assert_eq!(statistics["statistics"]["particle_count"], 4);
            assert_eq!(statistics["statistics"]["system_count"], 2);
        }
        "collision_energy" => assert_eq!(statistics["statistics"]["collision_energy_bits"], 0),
        "stuck_candidates" => assert_eq!(statistics["statistics"]["stuck_particle_ids"], json!([])),
        "system_aabb" | "system_culling" | "query_continue" => {
            let system_query = observe("system-query");
            assert_eq!(
                system_query["particle_ids"],
                json!(["phase9-a", "phase9-b"])
            );
            assert_eq!(system_query["terminated"], false);
        }
        "world_aabb" => assert_eq!(
            observe("world-query")["particle_ids"],
            json!(["phase9-c", "phase9-d", "phase9-a", "phase9-b"])
        ),
        "query_terminate" => {
            assert_eq!(
                observe("query-terminate")["particle_ids"],
                json!(["phase9-a"])
            );
            assert_eq!(observe("query-terminate")["terminated"], true);
        }
        "system_ray" | "ray_culling" | "ray_start_inside_exclusion" | "ray_continue" => {
            let system_ray = observe("system-ray");
            assert_eq!(system_ray["particle_ids"], json!(["phase9-a", "phase9-b"]));
            assert_eq!(system_ray["terminated"], false);
            assert!(
                system_ray["fractions_bits"]
                    .as_array()
                    .expect("fractions")
                    .iter()
                    .all(|bits| bits.as_u64().expect("fraction bits") != 0)
            );
        }
        "world_ray" => assert_eq!(
            observe("world-ray")["particle_ids"],
            json!(["phase9-c", "phase9-d", "phase9-a", "phase9-b"])
        ),
        "ray_ignore" => assert_eq!(observe("ray-ignore")["terminated"], false),
        "ray_clip" => assert_eq!(observe("ray-clip")["particle_ids"], json!(["phase9-a"])),
        "ray_terminate" => assert_eq!(observe("ray-terminate")["terminated"], true),
        _ => return false,
    }
    true
}

fn assert_contract_witness(request: &Value, result: &Value, branch: &str) -> bool {
    match branch {
        "closed_policy_registry" => assert_eq!(PHASE9_REQUIRED_POLICY_PATHS.len(), 22),
        "replay_identity"
        | "minimization_identity"
        | "first_divergence_stability"
        | "d0_byte_identity"
        | "debug_release_agreement" => {
            assert_eq!(result["request_id"], request["request_id"]);
            assert_eq!(result["scenario_id"], request["scenario"]["scenario_id"]);
        }
        _ => return false,
    }
    true
}

fn assert_witness(request: &Value, result: &Value, witness: &Phase9WitnessBinding) {
    let observation = observation_for_witness(request, result, witness);
    let branch = witness.branch_id.as_str();
    assert_eq!(observation["kind"], "particle", "{branch}");
    assert_eq!(
        observation["observation"]["kind"],
        serde_json::to_value(witness.observation_kind).expect("observation kind should serialize"),
        "{branch}"
    );
    match &witness.semantic_assertion {
        Phase9SemanticAssertion::ObservedSemantic { branch_id } => {
            assert_observed_semantic(request, result, branch_id.as_str());
        }
        Phase9SemanticAssertion::FiniteLifetimeExpired { particle_id } => {
            assert!(
                !observation["observation"]["particle_ids"]
                    .as_array()
                    .expect("system particle IDs")
                    .iter()
                    .any(|id| id == particle_id.as_str()),
                "{} must have expired",
                particle_id.as_str()
            );
        }
        Phase9SemanticAssertion::InfiniteLifetimeSurvives { particle_id } => {
            assert!(
                observation["observation"]["particle_ids"]
                    .as_array()
                    .expect("system particle IDs")
                    .iter()
                    .any(|id| id == particle_id.as_str()),
                "{} must survive",
                particle_id.as_str()
            );
        }
        Phase9SemanticAssertion::EqualExpirationOrder { particle_ids } => {
            assert_eq!(
                particle_declaration(request, particle_ids[0].as_str())["lifetime_bits"],
                particle_declaration(request, particle_ids[1].as_str())["lifetime_bits"]
            );
            assert_eq!(
                observation["observation"]["occurrence"]["maybe_particle_id"],
                particle_ids[1].as_str(),
                "equal expirations must evict newest-first"
            );
        }
        Phase9SemanticAssertion::StrictContactCardinality {
            enabled,
            contact_count,
        } => {
            let system_id = observation["observation"]["statistics"]["maybe_system_id"]
                .as_str()
                .expect("statistics system ID");
            assert_eq!(
                system_declaration(request, system_id)["strict_contact_check"],
                *enabled
            );
            assert_eq!(
                observation["observation"]["statistics"]["body_contact_count"],
                *contact_count
            );
        }
        Phase9SemanticAssertion::ListenerEventEffect {
            enabled,
            event_count,
        } => {
            let occurrences = phase9_checkpoint(result, "phase9-corpus")["observations"]
                .as_array()
                .expect("Phase 9 observations")
                .iter()
                .filter(|candidate| {
                    candidate["observation"]["kind"] == "lifecycle"
                        && candidate["observation"]["occurrence"]["kind"] == "contact_created"
                        && (*enabled
                            || candidate["observation"]["occurrence"]["maybe_particle_id"]
                                == "phase9-capacity"
                            || candidate["observation"]["occurrence"]["maybe_other_particle_id"]
                                == "phase9-capacity")
                })
                .count();
            assert_eq!(
                u32::try_from(occurrences).expect("event count fits"),
                *event_count
            );
        }
        Phase9SemanticAssertion::FilterContactEffect {
            enabled,
            contact_count,
        } => {
            let expected_system = if *enabled {
                "phase9-growable"
            } else {
                "phase9-fixed-paused"
            };
            assert_eq!(
                observation["observation"]["statistics"]["maybe_system_id"],
                expected_system
            );
            assert_eq!(
                observation["observation"]["statistics"]["particle_contact_count"],
                *contact_count
            );
        }
        Phase9SemanticAssertion::CollisionEnergyPositiveFinite { minimum_bits } => {
            let bits = u32::try_from(
                observation["observation"]["statistics"]["collision_energy_bits"]
                    .as_u64()
                    .expect("collision-energy bits"),
            )
            .expect("collision-energy bits fit");
            let energy = f32::from_bits(bits);
            assert!(energy.is_finite());
            assert!(energy >= minimum_bits.to_f32());
        }
        Phase9SemanticAssertion::StuckCandidatesNonempty { particle_ids } => {
            let stuck = observation["observation"]["statistics"]["stuck_particle_ids"]
                .as_array()
                .expect("stuck particle IDs");
            assert!(!stuck.is_empty());
            for particle_id in particle_ids {
                assert!(stuck.iter().any(|id| id == particle_id.as_str()));
            }
        }
        Phase9SemanticAssertion::ReplayResultDigestEquality
        | Phase9SemanticAssertion::MinimizedFailureSignaturePreservation
        | Phase9SemanticAssertion::DeliberateFirstDivergence
        | Phase9SemanticAssertion::D0RepeatedResultDigestEquality
        | Phase9SemanticAssertion::DebugReleaseResultDigestEquality => {
            assert_eq!(
                observation["observation"]["snapshot"]["particle_id"],
                "phase9-a"
            );
        }
    }
}

fn assert_result_evidence_bindings(
    root: &Path,
    executable: &OracleExecutable,
    revision: &str,
    request: &liquidfun_test_protocol::RigidWorldRequestRecord,
    native: &RigidWorldResultRecord,
    oracle: &RigidWorldResultRecord,
    bindings: &[Phase9WitnessBinding],
) {
    let has = |predicate: fn(&Phase9SemanticAssertion) -> bool| {
        bindings
            .iter()
            .any(|binding| predicate(&binding.semantic_assertion))
    };
    if !bindings.iter().any(|binding| {
        matches!(
            binding.semantic_assertion,
            Phase9SemanticAssertion::ReplayResultDigestEquality
                | Phase9SemanticAssertion::MinimizedFailureSignaturePreservation
                | Phase9SemanticAssertion::DeliberateFirstDivergence
                | Phase9SemanticAssertion::D0RepeatedResultDigestEquality
                | Phase9SemanticAssertion::DebugReleaseResultDigestEquality
        )
    }) {
        return;
    }
    let replay_native =
        NativeRigidWorldExecutor::execute(request).expect("native result replay must execute");
    let replay_oracle = execute_rigid_world_process(executable, request, revision)
        .expect("oracle result replay must execute");
    if has(|assertion| {
        matches!(
            assertion,
            Phase9SemanticAssertion::ReplayResultDigestEquality
        )
    }) {
        assert_eq!(
            sha256(&serde_json::to_vec(native).expect("native result bytes")),
            sha256(&serde_json::to_vec(&replay_native).expect("native replay bytes"))
        );
        assert_eq!(
            sha256(&serde_json::to_vec(oracle).expect("oracle result bytes")),
            sha256(&serde_json::to_vec(replay_oracle.result()).expect("oracle replay bytes"))
        );
    }
    if has(|assertion| {
        matches!(
            assertion,
            Phase9SemanticAssertion::D0RepeatedResultDigestEquality
        )
    }) {
        assert_eq!(
            serde_json::to_vec(native).expect("native D0 bytes"),
            serde_json::to_vec(&replay_native).expect("native repeated D0 bytes")
        );
        assert_eq!(
            serde_json::to_vec(oracle).expect("oracle D0 bytes"),
            serde_json::to_vec(replay_oracle.result()).expect("oracle repeated D0 bytes")
        );
    }

    let needs_deliberate_mismatch = bindings.iter().any(|binding| {
        matches!(
            binding.semantic_assertion,
            Phase9SemanticAssertion::MinimizedFailureSignaturePreservation
                | Phase9SemanticAssertion::DeliberateFirstDivergence
        )
    });
    if needs_deliberate_mismatch {
        let minimized = mutated_phase9_result(native, |value| {
            let body = first_checkpoint_member_mut(value, "bodies");
            body["active"] = json!(!body["active"].as_bool().expect("body active"));
        });
        let copied = mutated_phase9_result(native, |value| {
            let body = first_checkpoint_member_mut(value, "bodies");
            body["active"] = json!(!body["active"].as_bool().expect("body active"));
            let fixture = first_checkpoint_member_mut(value, "fixtures");
            fixture["sensor"] = json!(!fixture["sensor"].as_bool().expect("fixture sensor"));
        });
        let minimized_report = expected_retained_mismatch(request, native, &minimized);
        let copied_report = expected_retained_mismatch(request, native, &copied);
        if has(|assertion| {
            matches!(
                assertion,
                Phase9SemanticAssertion::MinimizedFailureSignaturePreservation
            )
        }) {
            assert_eq!(copied_report.signature(), minimized_report.signature());
        }
        if has(|assertion| {
            matches!(
                assertion,
                Phase9SemanticAssertion::DeliberateFirstDivergence
            )
        }) {
            assert_eq!(copied_report.semantic_path(), "rigid_world.body.active");
        }
    }

    if has(|assertion| {
        matches!(
            assertion,
            Phase9SemanticAssertion::DebugReleaseResultDigestEquality
        )
    }) && std::env::var("LIQUIDFUN_PHASE9_ORACLE_MODE").as_deref() == Ok("canonical")
    {
        let release = OracleExecutable::resolve(root, OraclePreset::Release)
            .expect("canonical evidence requires the release oracle");
        let optimized = execute_rigid_world_process(&release, request, revision)
            .expect("release result must execute");
        assert_eq!(
            sha256(&serde_json::to_vec(oracle).expect("debug result bytes")),
            sha256(&serde_json::to_vec(optimized.result()).expect("release result bytes"))
        );
    }
}

#[test]
fn executable_cases() {
    // Arrange
    let manifest = manifest();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let preset = match std::env::var("LIQUIDFUN_PHASE9_ORACLE_MODE").as_deref() {
        Ok("sanitizer") => OraclePreset::AsanUbsan,
        Ok("canonical") | Err(_) => OraclePreset::Debug,
        Ok(mode) => panic!("unsupported Phase 9 oracle mode {mode}"),
    };
    let Ok(executable) = OracleExecutable::resolve(&root, preset) else {
        eprintln!("SKIP: build the selected Phase 9 oracle to execute the corpus");
        return;
    };
    let maybe_evidence_root = std::env::var("LIQUIDFUN_PHASE9_EVIDENCE_MANIFEST")
        .ok()
        .map(|output| {
            let relative_output = PathBuf::from(output);
            assert!(!relative_output.is_absolute() && relative_output.starts_with("target"));
            assert!(
                relative_output
                    .components()
                    .all(|component| matches!(component, Component::Normal(_)))
            );
            root.join(
                relative_output
                    .parent()
                    .expect("evidence manifest must have a parent"),
            )
        });
    let mut evidence_cases = Vec::new();
    let (phase6, phase7, phase8) = retained_profiles();

    // Act
    for case in &manifest.cases {
        let bytes = std::fs::read(fixture_path(case)).expect("fixture bytes must be readable");
        assert_eq!(sha256(&bytes), case.request_sha256, "{}", case.case_id);
        let request = decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
            .expect("fixture must decode");
        let native = NativeRigidWorldExecutor::execute(&request)
            .unwrap_or_else(|error| panic!("native case {} must execute: {error}", case.case_id));
        let oracle =
            execute_rigid_world_process(&executable, &request, &manifest.pinned_upstream_revision)
                .expect("oracle case must execute");
        let run =
            run_phase9_differential(&executable, &request, &manifest.pinned_upstream_revision)
                .expect("case comparison must execute");
        assert_eq!(run.request_sha256().as_str(), case.request_sha256);
        assert_eq!(run.native_request_sha256(), run.request_sha256());
        assert_eq!(run.oracle_request_sha256(), run.request_sha256());
        assert_eq!(run.consumed_paths(), PHASE9_REQUIRED_POLICY_PATHS);
        let retained = compare_phase8_rigid_world_results(
            &request,
            &native,
            oracle.result(),
            &phase6,
            &phase7,
            &phase8,
        )
        .expect("retained rigid comparison must execute");
        assert_eq!(retained, RigidComparisonOutcome::Match);
        assert!(
            matches!(run.outcome(), Phase9ComparisonOutcome::Match { .. }),
            "{} produced an unexpected Phase 9 mismatch: {:?}",
            case.case_id,
            run.outcome()
        );
        let request_value = serde_json::to_value(&request).expect("request JSON");
        let native_value = serde_json::to_value(&native).expect("native JSON");
        let oracle_value = serde_json::to_value(oracle.result()).expect("oracle JSON");
        for witness in &case.witnesses {
            assert_witness(&request_value, &native_value, witness);
            assert_witness(&request_value, &oracle_value, witness);
        }
        assert_result_evidence_bindings(
            &root,
            &executable,
            &manifest.pinned_upstream_revision,
            &request,
            &native,
            oracle.result(),
            &case.witnesses,
        );
        let complete_comparison = CompleteComparisonPayload {
            outcome: "match".to_owned(),
            consumed_policy_paths: run
                .consumed_paths()
                .iter()
                .map(|path| (*path).to_owned())
                .collect(),
        };
        let payloads = EvidenceCasePayloads {
            request: bytes,
            native_result: serde_json::to_vec(&native).expect("native bytes"),
            oracle_result: serde_json::to_vec(oracle.result()).expect("oracle bytes"),
            complete_comparison: serde_json::to_vec(&complete_comparison)
                .expect("complete comparison bytes"),
        };
        let record = evidence_case_record(case, &payloads, run.consumed_paths());
        validate_evidence_case_value(
            &serde_json::to_value(&record).expect("evidence case value"),
            &payloads,
        )
        .expect("generated evidence case must validate");
        if let Some(evidence_root) = &maybe_evidence_root {
            write_evidence_payload(evidence_root, &record.request_path, &payloads.request);
            write_evidence_payload(
                evidence_root,
                &record.native_result_path,
                &payloads.native_result,
            );
            write_evidence_payload(
                evidence_root,
                &record.oracle_result_path,
                &payloads.oracle_result,
            );
            write_evidence_payload(
                evidence_root,
                &record.complete_comparison_path,
                &payloads.complete_comparison,
            );
        }
        evidence_cases.push(record);
    }
    let evidence = EvidenceManifest {
        schema_version: 2,
        case_record_schema_version: 1,
        profile: manifest.profile,
        upstream_revision: manifest.pinned_upstream_revision,
        semantic_manifest_sha256: canonical_sha256(&evidence_cases),
        cases: evidence_cases,
    };

    // Assert
    assert!(!evidence.cases.is_empty());
    if let Ok(output) = std::env::var("LIQUIDFUN_PHASE9_EVIDENCE_MANIFEST") {
        let relative_output = Path::new(&output);
        assert!(!relative_output.is_absolute() && relative_output.starts_with("target"));
        assert!(
            relative_output
                .components()
                .all(|component| { matches!(component, Component::Normal(_)) })
        );
        let output = root.join(relative_output);
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent).expect("evidence directory");
        }
        let mut bytes = serde_json::to_vec_pretty(&evidence).expect("evidence JSON");
        bytes.push(b'\n');
        std::fs::write(output, bytes).expect("evidence manifest");
    }
}

#[test]
#[ignore = "explicit fixture regeneration tool"]
fn regenerate_case_fixture() {
    for case in manifest().cases {
        let request = bounded_phase9_request(&case.case_id);
        let mut bytes = serde_json::to_vec(&request).expect("generated fixture should encode");
        bytes.push(b'\n');
        std::fs::write(fixture_path(&case), bytes).expect("generated fixture should be written");
    }
}

#[test]
fn manifest_declares_every_phase9_branch_exactly_once() {
    // Arrange
    let manifest = manifest();
    let required = REQUIRED_BRANCHES.iter().copied().collect::<BTreeSet<_>>();

    // Act
    let mut occurrences = BTreeMap::<&str, usize>::new();
    for branch in manifest
        .cases
        .iter()
        .flat_map(|case| &case.witnesses)
        .map(|witness| &witness.branch_id)
    {
        *occurrences.entry(branch.as_str()).or_default() += 1;
    }
    let actual = occurrences.keys().copied().collect::<BTreeSet<_>>();
    let bindings = manifest
        .cases
        .iter()
        .flat_map(|case| case.witnesses.iter().cloned())
        .collect::<Vec<_>>();
    let (maximum_actions, maximum_checkpoints) = manifest
        .cases
        .iter()
        .map(|case| {
            let request: Value =
                serde_json::from_slice(&fs::read(fixture_path(case)).expect("fixture bytes"))
                    .expect("fixture JSON");
            (
                request["scenario"]["timelines"][0]["actions"]
                    .as_array()
                    .expect("actions")
                    .len(),
                request["scenario"]["timelines"][0]["checkpoints"]
                    .as_array()
                    .expect("checkpoints")
                    .len(),
            )
        })
        .fold((0, 0), |maximum, count| {
            (maximum.0.max(count.0), maximum.1.max(count.1))
        });

    // Assert
    assert_eq!(manifest.profile, PHASE9_REGISTRY_ID);
    validate_phase9_witness_bindings(&bindings, maximum_actions, maximum_checkpoints)
        .expect("the manifest must be a closed typed witness registry");
    assert_eq!(bindings.len(), REQUIRED_BRANCHES.len());
    assert_eq!(actual, required);
    assert!(occurrences.values().all(|count| *count == 1));
    assert!(manifest.cases.iter().all(|case| !case.case_id.is_empty()));
    assert!(
        manifest
            .cases
            .iter()
            .any(|case| case.authority == Authority::PinnedOracle)
    );
    assert!(
        manifest
            .cases
            .iter()
            .any(|case| case.authority == Authority::Independent)
    );
}

fn witness_id(value: &str) -> ScenarioId {
    ScenarioId::new(value).expect("test witness identity should validate")
}

fn valid_witness_binding(branch_id: &str) -> Phase9WitnessBinding {
    let semantic_assertion = match branch_id {
        "finite_lifetime" => Phase9SemanticAssertion::FiniteLifetimeExpired {
            particle_id: witness_id("phase9-a"),
        },
        "infinite_lifetime" => Phase9SemanticAssertion::InfiniteLifetimeSurvives {
            particle_id: witness_id("phase9-b"),
        },
        "equal_lifetime" => Phase9SemanticAssertion::EqualExpirationOrder {
            particle_ids: vec![witness_id("phase9-c"), witness_id("phase9-d")].into_boxed_slice(),
        },
        "strict_contact_enabled" | "strict_contact_disabled" => {
            Phase9SemanticAssertion::StrictContactCardinality {
                enabled: branch_id == "strict_contact_enabled",
                contact_count: u32::from(branch_id == "strict_contact_enabled"),
            }
        }
        "listener_flag_enabled" | "listener_flag_disabled" => {
            Phase9SemanticAssertion::ListenerEventEffect {
                enabled: branch_id == "listener_flag_enabled",
                event_count: u32::from(branch_id == "listener_flag_enabled"),
            }
        }
        "filter_flag_enabled" | "filter_flag_disabled" => {
            Phase9SemanticAssertion::FilterContactEffect {
                enabled: branch_id == "filter_flag_enabled",
                contact_count: u32::from(branch_id == "filter_flag_enabled"),
            }
        }
        "collision_energy" => Phase9SemanticAssertion::CollisionEnergyPositiveFinite {
            minimum_bits: 1.0_f32.to_bits().into(),
        },
        "stuck_candidates" => Phase9SemanticAssertion::StuckCandidatesNonempty {
            particle_ids: vec![witness_id("phase9-coupling")].into_boxed_slice(),
        },
        "replay_identity" => Phase9SemanticAssertion::ReplayResultDigestEquality,
        "minimization_identity" => Phase9SemanticAssertion::MinimizedFailureSignaturePreservation,
        "first_divergence_stability" => Phase9SemanticAssertion::DeliberateFirstDivergence,
        "d0_byte_identity" => Phase9SemanticAssertion::D0RepeatedResultDigestEquality,
        "debug_release_agreement" => Phase9SemanticAssertion::DebugReleaseResultDigestEquality,
        _ => Phase9SemanticAssertion::ObservedSemantic {
            branch_id: witness_id(branch_id),
        },
    };
    Phase9WitnessBinding {
        branch_id: witness_id(branch_id),
        action_index: 0,
        checkpoint_index: 0,
        observation_kind: semantic_assertion.expected_observation_kind(),
        semantic_assertion,
    }
}

fn valid_witness_bindings() -> Vec<Phase9WitnessBinding> {
    REQUIRED_BRANCHES
        .iter()
        .copied()
        .map(valid_witness_binding)
        .collect()
}

#[test]
fn witness_binding_rejects_generic_identity_for_result_evidence() {
    // Arrange
    let mut bindings = valid_witness_bindings();
    let binding = bindings
        .iter_mut()
        .find(|binding| binding.branch_id.as_str() == "replay_identity")
        .expect("replay binding");
    binding.semantic_assertion = Phase9SemanticAssertion::ObservedSemantic {
        branch_id: witness_id("replay_identity"),
    };

    // Act
    let result = validate_phase9_witness_bindings(&bindings, 1, 1);

    // Assert
    assert_eq!(
        result.expect_err("generic identity must fail").kind(),
        Phase9WitnessBindingErrorKind::BranchAssertionMismatch
    );
}

#[test]
fn witness_binding_rejects_zero_collision_energy() {
    // Arrange
    let mut bindings = valid_witness_bindings();
    let binding = bindings
        .iter_mut()
        .find(|binding| binding.branch_id.as_str() == "collision_energy")
        .expect("collision-energy binding");
    binding.semantic_assertion = Phase9SemanticAssertion::CollisionEnergyPositiveFinite {
        minimum_bits: 0.0_f32.to_bits().into(),
    };

    // Act
    let result = validate_phase9_witness_bindings(&bindings, 1, 1);

    // Assert
    assert_eq!(
        result.expect_err("zero energy must fail").kind(),
        Phase9WitnessBindingErrorKind::InvalidSemanticAssertion
    );
}

#[test]
fn witness_binding_rejects_empty_stuck_candidates() {
    // Arrange
    let mut bindings = valid_witness_bindings();
    let binding = bindings
        .iter_mut()
        .find(|binding| binding.branch_id.as_str() == "stuck_candidates")
        .expect("stuck-candidate binding");
    binding.semantic_assertion = Phase9SemanticAssertion::StuckCandidatesNonempty {
        particle_ids: Box::new([]),
    };

    // Act
    let result = validate_phase9_witness_bindings(&bindings, 1, 1);

    // Assert
    assert_eq!(
        result.expect_err("empty stuck candidates must fail").kind(),
        Phase9WitnessBindingErrorKind::InvalidSemanticAssertion
    );
}

#[test]
fn witness_binding_rejects_wrong_observation_kind() {
    // Arrange
    let mut bindings = valid_witness_bindings();
    bindings[0].observation_kind = Phase9ObservationKind::Lifecycle;

    // Act
    let result = validate_phase9_witness_bindings(&bindings, 1, 1);

    // Assert
    assert_eq!(
        result.expect_err("wrong observation kind must fail").kind(),
        Phase9WitnessBindingErrorKind::ObservationKindMismatch
    );
}

#[test]
fn witness_binding_rejects_invalid_action_index() {
    // Arrange
    let mut invalid_action = valid_witness_bindings();
    invalid_action[0].action_index = 1;

    // Act
    let result = validate_phase9_witness_bindings(&invalid_action, 1, 1);

    // Assert
    assert_eq!(
        result.expect_err("invalid action index must fail").kind(),
        Phase9WitnessBindingErrorKind::ActionIndexOutOfRange
    );
}

#[test]
fn witness_binding_rejects_invalid_checkpoint_index() {
    // Arrange
    let mut bindings = valid_witness_bindings();
    bindings[0].checkpoint_index = 1;

    // Act
    let result = validate_phase9_witness_bindings(&bindings, 1, 1);

    // Assert
    assert_eq!(
        result
            .expect_err("invalid checkpoint index must fail")
            .kind(),
        Phase9WitnessBindingErrorKind::CheckpointIndexOutOfRange
    );
}

#[test]
fn witness_binding_rejects_duplicate_branch_ids() {
    // Arrange
    let mut bindings = valid_witness_bindings();
    bindings[1] = bindings[0].clone();

    // Act
    let result = validate_phase9_witness_bindings(&bindings, 1, 1);

    // Assert
    assert_eq!(
        result.expect_err("duplicate branch must fail").kind(),
        Phase9WitnessBindingErrorKind::DuplicateBranch
    );
}

#[test]
fn witness_binding_rejects_unknown_assertion_kind() {
    // Arrange
    let value = json!({
        "branch_id": "collision_energy",
        "action_index": 0,
        "checkpoint_index": 0,
        "observation_kind": "statistics",
        "semantic_assertion": {
            "kind": "request_identity_equality"
        }
    });

    // Act
    let result = serde_json::from_value::<Phase9WitnessBinding>(value);

    // Assert
    assert!(result.is_err());
}

#[test]
fn witness_binding_rejects_more_than_reviewed_limit() {
    // Arrange
    let mut bindings = valid_witness_bindings();
    bindings.push(bindings[0].clone());

    // Act
    let result = validate_phase9_witness_bindings(&bindings, 1, 1);

    // Assert
    assert_eq!(
        result.expect_err("oversized binding set must fail").kind(),
        Phase9WitnessBindingErrorKind::TooManyBindings
    );
}

#[test]
fn witness_binding_requires_exact_reviewed_branch_registry() {
    // Arrange
    let complete = valid_witness_bindings();
    let missing = &complete[1..];
    let mut extra = complete.clone();
    extra[0] = Phase9WitnessBinding {
        branch_id: witness_id("unreviewed-branch"),
        action_index: 0,
        checkpoint_index: 0,
        observation_kind: Phase9ObservationKind::Particle,
        semantic_assertion: Phase9SemanticAssertion::ObservedSemantic {
            branch_id: witness_id("unreviewed-branch"),
        },
    };

    // Act
    let complete_result = validate_phase9_witness_bindings(&complete, 1, 1);
    let missing_result = validate_phase9_witness_bindings(missing, 1, 1);
    let extra_result = validate_phase9_witness_bindings(&extra, 1, 1);

    // Assert
    assert!(complete_result.is_ok());
    assert_eq!(
        missing_result.expect_err("missing branch must fail").kind(),
        Phase9WitnessBindingErrorKind::MissingBranch
    );
    assert_eq!(
        extra_result.expect_err("extra branch must fail").kind(),
        Phase9WitnessBindingErrorKind::ExtraBranch
    );
    assert_eq!(complete.len(), 58);
}

#[test]
fn corpus_is_bound_to_retained_rigid_and_pinned_oracle_bytes() {
    // Arrange
    let manifest = manifest();

    // Act
    let request_digest = format!("{:x}", Sha256::digest(RETAINED_REQUEST));
    let witness_digest = format!("{:x}", Sha256::digest(PINNED_WITNESS));
    let retained =
        decode_rigid_world_request_jsonl(RETAINED_REQUEST, &HarnessLimits::phase2_default_v1())
            .expect("the retained request should decode");

    // Assert
    assert_eq!(manifest.retained_request_sha256, request_digest);
    assert_eq!(manifest.pinned_witness_sha256, witness_digest);
    assert_eq!(
        manifest.pinned_upstream_revision,
        "7f20402173fd143a3988c921bc384459c6a858f2"
    );
    assert_eq!(
        retained.scenario().timelines().len(),
        RigidWorldWitnessFamily::ALL.len()
    );
}

#[test]
fn corpus_executes_with_stable_ids_and_d0_bytes() {
    // Arrange
    let request = bounded_phase9_request("closed-evidence-contract");

    // Act
    let first = NativeRigidWorldExecutor::execute(&request).expect("Phase 9 corpus should execute");
    let second =
        NativeRigidWorldExecutor::execute(&request).expect("Phase 9 replay should execute");
    let first_bytes = serde_json::to_vec(&first).expect("first result should encode");
    let second_bytes = serde_json::to_vec(&second).expect("second result should encode");

    // Assert
    assert_eq!(first, second);
    assert_eq!(first_bytes, second_bytes);
    assert_eq!(
        first.timelines()[0]
            .checkpoints
            .first()
            .expect("Phase 9 checkpoint should exist")
            .phase
            .as_ref(),
        "phase9"
    );
}

fn retained_profiles() -> (
    Phase6PolicyProfile,
    Phase7PolicyProfile,
    Phase8PolicyProfile,
) {
    (
        Phase6PolicyProfile::parse_toml(PHASE6_POLICY)
            .expect("checked-in Phase 6 policy should parse"),
        Phase7PolicyProfile::parse_toml(PHASE7_POLICY)
            .expect("checked-in Phase 7 policy should parse"),
        Phase8PolicyProfile::parse_toml(PHASE8_POLICY)
            .expect("checked-in Phase 8 policy should parse"),
    )
}

fn mutated_phase9_result(
    native: &RigidWorldResultRecord,
    mutate: impl FnOnce(&mut Value),
) -> RigidWorldResultRecord {
    let mut value = serde_json::to_value(native).expect("result should serialize");
    mutate(&mut value);
    let mut bytes = serde_json::to_vec(&value).expect("mutation should serialize");
    bytes.push(b'\n');
    decode_rigid_world_result_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("request-valid retained mutation should decode")
}

fn first_checkpoint_member_mut<'a>(value: &'a mut Value, member: &str) -> &'a mut Value {
    value["timelines"]
        .as_array_mut()
        .expect("timelines should be an array")
        .iter_mut()
        .flat_map(|timeline| {
            timeline["checkpoints"]
                .as_array_mut()
                .expect("checkpoints should be an array")
        })
        .filter(|checkpoint| {
            checkpoint
                .get("observations")
                .and_then(Value::as_array)
                .is_none_or(|observations| {
                    observations.iter().all(|observation| {
                        matches!(
                            observation["kind"].as_str(),
                            Some("body_state" | "step" | "query" | "ray_cast" | "origin_shift")
                        )
                    })
                })
        })
        .find_map(|checkpoint| {
            checkpoint
                .get_mut(member)
                .expect("checkpoint member should exist")
                .as_array_mut()
                .and_then(|values| values.first_mut())
        })
        .unwrap_or_else(|| panic!("a checkpoint should contain `{member}`"))
}

fn first_observation_mut(value: &mut Value, predicate: impl Fn(&Value) -> bool) -> &mut Value {
    value["timelines"]
        .as_array_mut()
        .expect("timelines should be an array")
        .iter_mut()
        .flat_map(|timeline| {
            timeline["checkpoints"]
                .as_array_mut()
                .expect("checkpoints should be an array")
        })
        .filter_map(|checkpoint| {
            checkpoint
                .get_mut("observations")
                .and_then(Value::as_array_mut)
        })
        .flatten()
        .find(|observation| predicate(observation))
        .expect("the requested observation should exist")
}

fn expected_retained_mismatch(
    request: &liquidfun_test_protocol::RigidWorldRequestRecord,
    native: &RigidWorldResultRecord,
    oracle: &RigidWorldResultRecord,
) -> Box<RigidMismatchReport> {
    let (phase6, phase7, phase8) = retained_profiles();
    let outcome =
        compare_phase8_rigid_world_results(request, native, oracle, &phase6, &phase7, &phase8)
            .expect("request-valid retained mutation should compare");
    let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
        panic!("retained mutation must produce a Phase 8 physics mismatch");
    };
    report
}

fn assert_complete_retained_signature(
    request: &liquidfun_test_protocol::RigidWorldRequestRecord,
    native: &RigidWorldResultRecord,
    oracle: &RigidWorldResultRecord,
    expected_path: &str,
) {
    let expected = expected_retained_mismatch(request, native, oracle);
    let outcome = compare_complete_phase9_rigid_world_results(request, native, oracle)
        .expect("request-valid retained mutation should compare");
    let Phase9ComparisonOutcome::RetainedRigidMismatch(actual) = outcome else {
        panic!("retained mutation must win at {expected_path}");
    };
    assert_eq!(expected.semantic_path(), expected_path);
    assert_eq!(actual.signature(), expected.signature());
}

#[test]
fn phase9_comparator_rejects_retained_body_mutation() {
    // Arrange
    let request = bounded_phase9_request("closed-evidence-contract");
    let native =
        NativeRigidWorldExecutor::execute(&request).expect("Phase 9 corpus should execute");
    let oracle = mutated_phase9_result(&native, |value| {
        let body = first_checkpoint_member_mut(value, "bodies");
        let active = body["active"]
            .as_bool()
            .expect("body active state should be boolean");
        body["active"] = json!(!active);
    });

    // Act / Assert
    assert_complete_retained_signature(&request, &native, &oracle, "rigid_world.body.active");
}

#[test]
fn phase9_comparator_rejects_retained_fixture_mutation() {
    // Arrange
    let request = bounded_phase9_request("closed-evidence-contract");
    let native =
        NativeRigidWorldExecutor::execute(&request).expect("Phase 9 corpus should execute");
    let oracle = mutated_phase9_result(&native, |value| {
        let fixture = first_checkpoint_member_mut(value, "fixtures");
        let sensor = fixture["sensor"]
            .as_bool()
            .expect("fixture sensor state should be boolean");
        fixture["sensor"] = json!(!sensor);
    });

    // Act / Assert
    assert_complete_retained_signature(&request, &native, &oracle, "rigid_world.fixture.sensor");
}

#[test]
fn phase9_comparator_rejects_retained_numeric_mutation() {
    // Arrange
    let request = bounded_phase9_request("closed-evidence-contract");
    let native =
        NativeRigidWorldExecutor::execute(&request).expect("Phase 9 corpus should execute");
    let oracle = mutated_phase9_result(&native, |value| {
        let diagnostics =
            first_observation_mut(value, |observation| observation["kind"] == "diagnostics");
        diagnostics["snapshot"]["tree_quality_bits"] = json!(100.0_f32.to_bits());
    });

    // Act / Assert
    assert_complete_retained_signature(
        &request,
        &native,
        &oracle,
        "rigid_world.phase8.diagnostics.tree_quality",
    );
}

#[test]
fn phase9_comparator_rejects_retained_before_particle_mutation() {
    // Arrange
    let request = bounded_phase9_request("closed-evidence-contract");
    let native =
        NativeRigidWorldExecutor::execute(&request).expect("Phase 9 corpus should execute");
    let oracle = mutated_phase9_result(&native, |value| {
        let body = first_checkpoint_member_mut(value, "bodies");
        let active = body["active"]
            .as_bool()
            .expect("body active state should be boolean");
        body["active"] = json!(!active);
        let statistics = first_observation_mut(value, |observation| {
            observation["kind"] == "particle" && observation["observation"]["kind"] == "statistics"
        });
        statistics["observation"]["statistics"]["particle_contact_count"] = json!(u32::MAX);
    });

    // Act / Assert
    assert_complete_retained_signature(&request, &native, &oracle, "rigid_world.body.active");
}

#[test]
fn phase9_comparator_rejects_retained_process_result_through_runner() {
    // Arrange
    let fake = FakePhase9OracleRoot::new("rigid_d1_mismatch");
    let executable = OracleExecutable::resolve(fake.path(), OraclePreset::Debug)
        .expect("fake oracle should occupy the reviewed preset path");
    let request =
        decode_rigid_world_request_jsonl(RETAINED_REQUEST, &HarnessLimits::phase2_default_v1())
            .expect("retained rigid request should decode");
    let revision = manifest().pinned_upstream_revision;

    // Act
    let run = run_phase9_differential(&executable, &request, &revision)
        .expect("request-valid retained process mutation should compare");

    // Assert
    let Phase9ComparisonOutcome::RetainedRigidMismatch(report) = run.outcome() else {
        panic!("runner must report its retained rigid mismatch");
    };
    assert_eq!(report.semantic_path(), "rigid_world.body.active");
}

#[test]
fn fake_phase9_oracle_root_writes_closed_compile_database() {
    // Arrange
    let fake = FakePhase9OracleRoot::new("rigid_d1_mismatch");
    let compile_database = fake
        .path()
        .join("target/reference/oracle-debug/compile_commands.json");
    let entries: Vec<Value> = serde_json::from_slice(
        &fs::read(compile_database).expect("fake compile database should be readable"),
    )
    .expect("fake compile database should decode");

    // Act
    let units = entries
        .iter()
        .map(|entry| {
            Path::new(
                entry["file"]
                    .as_str()
                    .expect("fake compile database file should be a string"),
            )
            .file_name()
            .and_then(|value| value.to_str())
            .expect("fake compile database file should have a UTF-8 name")
            .to_owned()
        })
        .collect::<BTreeSet<_>>();
    let digest = effective_compile_command_sha256(fake.path(), "oracle-debug")
        .expect("fake compile database should have a reviewed command shape");

    // Assert
    assert_eq!(entries.len(), FAKE_PHASE9_RESULT_UNITS.len());
    assert_eq!(
        units,
        FAKE_PHASE9_RESULT_UNITS
            .into_iter()
            .map(str::to_owned)
            .collect::<BTreeSet<_>>()
    );
    assert_eq!(digest.len(), 64);
    assert!(digest.bytes().all(|byte| byte.is_ascii_hexdigit()));
}

#[test]
fn retained_rigid_record_rejects_missing_or_mutated_proof() {
    // Arrange
    let (valid, payloads) = evidence_case_fixture();
    let mut missing = valid.clone();
    missing
        .as_object_mut()
        .expect("evidence case object")
        .remove("retained_rigid");
    let mut mutated = valid.clone();
    mutated["retained_rigid"]["phase8_policy_sha256"] = json!("0".repeat(64));

    // Act
    let valid_result = validate_evidence_case_value(&valid, &payloads);
    let missing_result = validate_evidence_case_value(&missing, &payloads);
    let mutated_result = validate_evidence_case_value(&mutated, &payloads);

    // Assert
    assert!(valid_result.is_ok());
    assert_eq!(
        missing_result,
        Err("missing retained-rigid proof".to_owned())
    );
    assert_eq!(
        mutated_result,
        Err("retained-rigid policy digest mismatch".to_owned())
    );
}

#[test]
fn witness_binding_record_rejects_semantic_or_payload_digest_mutation() {
    // Arrange
    let (valid, payloads) = evidence_case_fixture();
    let mut semantic = valid.clone();
    semantic["witnesses"][0]["action_index"] = json!(usize::MAX);
    let mut corrupted_payloads = payloads.clone();
    corrupted_payloads.native_result.push(b'!');

    // Act
    let valid_result = validate_evidence_case_value(&valid, &payloads);
    let semantic_result = validate_evidence_case_value(&semantic, &payloads);
    let payload_result = validate_evidence_case_value(&valid, &corrupted_payloads);

    // Assert
    assert!(valid_result.is_ok());
    assert_eq!(
        semantic_result,
        Err("witness binding digest mismatch".to_owned())
    );
    assert_eq!(
        payload_result,
        Err("native result payload digest mismatch".to_owned())
    );
}

struct FakePhase9OracleRoot {
    root: PathBuf,
}

impl FakePhase9OracleRoot {
    fn new(behavior: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should follow the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "liquidfun-phase9-retained-oracle-{}-{nonce}",
            std::process::id()
        ));
        let preset = root.join("target/reference/oracle-debug");
        fs::create_dir_all(&preset).expect("fake preset directory should be created");
        let destination = preset.join(if cfg!(windows) {
            "liquidfun-reference.exe"
        } else {
            "liquidfun-reference"
        });
        fs::copy(env!("CARGO_BIN_EXE_liquidfun-fake-oracle"), &destination)
            .expect("fake oracle should copy into the reviewed path");
        copy_adapter_inputs(&root);
        write_fake_compile_database(&root);
        fs::write(preset.join("behavior.txt"), behavior)
            .expect("fake oracle behavior should be written");
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }
}

fn write_fake_compile_database(root: &Path) {
    let build = root.join("target/reference/oracle-debug");
    let entries = FAKE_PHASE9_RESULT_UNITS
        .map(|unit| {
            let source = root.join("tools/reference/src").join(unit);
            json!({
                "directory": build,
                "file": source,
                "command": format!(
                    "clang++ -I{}/tools/reference/src -O0 -g -o {}/{unit}.o -c {}",
                    root.display(),
                    build.display(),
                    source.display()
                ),
            })
        })
        .to_vec();
    fs::write(
        build.join("compile_commands.json"),
        serde_json::to_vec_pretty(&entries)
            .expect("fake compile database should encode deterministically"),
    )
    .expect("fake compile database should be written");
}

fn copy_adapter_inputs(destination_root: &Path) {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest_path = Path::new("tools/reference/adapter-inputs.txt");
    let manifest = fs::read_to_string(source_root.join(manifest_path))
        .expect("reviewed adapter input manifest should be readable");
    let destination_manifest = destination_root.join(manifest_path);
    fs::create_dir_all(
        destination_manifest
            .parent()
            .expect("adapter manifest should have a parent"),
    )
    .expect("adapter manifest directory should be created");
    fs::write(&destination_manifest, &manifest).expect("adapter input manifest should be copied");
    for relative in manifest
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let destination = destination_root.join(relative);
        fs::create_dir_all(
            destination
                .parent()
                .expect("adapter input should have a parent"),
        )
        .expect("adapter input directory should be created");
        fs::copy(source_root.join(relative), destination)
            .expect("reviewed adapter input should be copied");
    }
}

impl Drop for FakePhase9OracleRoot {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).expect("fake oracle root should be removable");
    }
}

#[test]
fn corpus_rejects_missing_declarations_and_phase10_members() {
    // Arrange
    let manifest = manifest();
    let mut missing = serde_json::from_str::<Value>(MANIFEST).expect("manifest should be JSON");
    missing["cases"][0]["witnesses"]
        .as_array_mut()
        .expect("branches should be an array")
        .remove(0);

    // Act / Assert
    let decoded: CorpusManifest = serde_json::from_value(missing).expect("shape remains valid");
    let decoded_branches = decoded
        .cases
        .iter()
        .flat_map(|case| &case.witnesses)
        .map(|witness| witness.branch_id.as_str())
        .collect::<BTreeSet<_>>();
    assert_ne!(
        decoded_branches,
        REQUIRED_BRANCHES.iter().copied().collect()
    );
    for member in &manifest.forbidden_phase10_members {
        let mut value = request_value();
        value["scenario"]["timelines"][0][member] = json!([]);
        assert!(decode_value(&value).is_err(), "{member} must fail closed");
    }
    assert!(
        PHASE9_REQUIRED_POLICY_PATHS
            .iter()
            .all(|path| phase9_policy_for_path(path).is_some())
    );
    assert_eq!(phase9_policy_for_path("particle.*"), None);
    assert_eq!(phase9_policy_for_path("particle.group.topology"), None);
    assert_eq!(phase9_policy_for_path("particle.solver.baseline"), None);
}

#[test]
fn required_oracle_mode_proves_replay_and_profile_agreement() {
    // Arrange
    let Ok(mode) = std::env::var("LIQUIDFUN_PHASE9_ORACLE_MODE") else {
        return;
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let request = bounded_phase9_request("closed-evidence-contract");
    let revision = manifest().pinned_upstream_revision;
    let primary_preset = match mode.as_str() {
        "canonical" => OraclePreset::Debug,
        "sanitizer" => OraclePreset::AsanUbsan,
        _ => panic!("LIQUIDFUN_PHASE9_ORACLE_MODE must be canonical or sanitizer"),
    };
    let primary = OracleExecutable::resolve(&root, primary_preset)
        .expect("the required primary Phase 9 oracle must exist");

    // Act
    let first = execute_rigid_world_process(&primary, &request, &revision)
        .expect("the primary Phase 9 oracle run should pass");
    let replay = execute_rigid_world_process(&primary, &request, &revision)
        .expect("the Phase 9 oracle replay should pass");
    let differential = run_phase9_differential(&primary, &request, &revision)
        .expect("the bounded Phase 9 corpus should compare");

    // Assert
    assert_eq!(first.response_bytes(), replay.response_bytes());
    assert_eq!(first.result(), replay.result());
    assert!(
        matches!(
            differential.outcome(),
            liquidfun_differential::Phase9ComparisonOutcome::Match { .. }
        ),
        "unexpected outcome: {:?}",
        differential.outcome()
    );
    if mode == "canonical" {
        let release = OracleExecutable::resolve(&root, OraclePreset::Release)
            .expect("the required release Phase 9 oracle must exist");
        let optimized = execute_rigid_world_process(&release, &request, &revision)
            .expect("the release Phase 9 oracle run should pass");
        assert_eq!(first.result(), optimized.result());
    }
}

#[test]
#[cfg(unix)]
fn workflow_contract_blocks_failed_evidence_identity() {
    use std::os::unix::fs::PermissionsExt;

    // Arrange
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let workflow = std::fs::read_to_string(root.join(".github/workflows/oracle.yml"))
        .expect("Oracle workflow");
    assert_eq!(
        workflow.matches("bash scripts/phase9-evidence.sh").count(),
        2
    );
    let script = root.join("scripts/phase9-evidence.sh");
    let script_text = std::fs::read_to_string(&script).expect("evidence script");
    assert!(script_text.contains("set -euo pipefail"));
    let validation = script_text
        .find("cargo xtask phase9-evidence validate-content")
        .expect("shared content validator");
    let identity = script_text
        .find("> \"$output_dir/identity.json\"")
        .expect("identity emission");
    assert!(
        validation < identity,
        "content validation must precede identity"
    );

    for (name, cargo_body) in [
        ("command-failure", "exit 7\n"),
        (
            "failed-log",
            "printf '%s\\n' 'test result: ok. 4 passed' 'test result: FAILED. 4 passed; 1 failed'\nexit 0\n",
        ),
    ] {
        let contract_root = root.join("target").join(format!(
            "phase9-workflow-contract-{}-{name}",
            std::process::id()
        ));
        let fake_bin = contract_root.join("bin");
        let output = contract_root.join("canonical");
        std::fs::create_dir_all(&fake_bin).expect("fake command directory");
        let fake_cargo = fake_bin.join("cargo");
        std::fs::write(
            &fake_cargo,
            format!("#!/usr/bin/env bash\nset -euo pipefail\n{cargo_body}"),
        )
        .expect("fake cargo");
        let mut permissions = std::fs::metadata(&fake_cargo)
            .expect("fake cargo metadata")
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&fake_cargo, permissions).expect("fake cargo executable");
        let path = format!(
            "{}:{}",
            fake_bin.display(),
            std::env::var("PATH").expect("PATH")
        );

        // Act
        let contract = Command::new("bash")
            .arg(&script)
            .arg("canonical")
            .arg(
                output
                    .strip_prefix(&root)
                    .expect("repository-relative output"),
            )
            .current_dir(&root)
            .env("PATH", path)
            .output()
            .expect("evidence script should execute");

        // Assert
        assert!(!contract.status.success(), "{name} must fail closed");
        assert!(!output.join("identity.json").exists());
        std::fs::remove_dir_all(&contract_root).expect("contract cleanup");
    }
}

#[test]
#[cfg(unix)]
fn workflow_contract_rejects_symlinked_output_before_cleanup() {
    use std::os::unix::fs::symlink;

    // Arrange
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let script = root.join("scripts/phase9-evidence.sh");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should follow the Unix epoch")
        .as_nanos();
    let contract_root = root
        .join("target")
        .join(format!("phase9-symlink-contract-{nonce}"));
    let external_root = std::env::temp_dir().join(format!("liquidfun-phase9-external-{nonce}"));
    fs::create_dir_all(&contract_root).expect("contract root");

    for shape in ["final", "ancestor"] {
        let external_output = external_root.join(shape).join("canonical");
        let marker = external_output.join("cases/external-marker");
        fs::create_dir_all(marker.parent().expect("marker parent")).expect("external cases");
        fs::write(&marker, b"must survive").expect("external marker");

        let relative_output = if shape == "final" {
            let link = contract_root.join("canonical");
            symlink(&external_output, &link).expect("final output symlink");
            link.strip_prefix(&root)
                .expect("contract output remains repository-relative")
                .to_path_buf()
        } else {
            let link = contract_root.join("linked-ancestor");
            symlink(
                external_output.parent().expect("external output parent"),
                &link,
            )
            .expect("output ancestor symlink");
            link.join("canonical")
                .strip_prefix(&root)
                .expect("contract output remains repository-relative")
                .to_path_buf()
        };

        // Act
        let output = Command::new("bash")
            .arg(&script)
            .arg("canonical")
            .arg(&relative_output)
            .current_dir(&root)
            .output()
            .expect("evidence script should execute");

        // Assert
        assert!(!output.status.success(), "{shape} symlink must fail closed");
        assert_eq!(
            fs::read(&marker).expect("external marker must remain readable"),
            b"must survive"
        );

        let link = if shape == "final" {
            contract_root.join("canonical")
        } else {
            contract_root.join("linked-ancestor")
        };
        fs::remove_file(link).expect("contract symlink cleanup");
    }

    fs::remove_dir_all(&contract_root).expect("contract root cleanup");
    fs::remove_dir_all(&external_root).expect("external root cleanup");
}
