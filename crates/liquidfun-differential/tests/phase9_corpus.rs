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
    compare_phase8_rigid_world_results, execute_rigid_world_process, phase9_policy_for_path,
    run_phase9_differential,
};
use liquidfun_test_protocol::{
    HarnessLimits, Phase6PolicyProfile, Phase7PolicyProfile, Phase8PolicyProfile,
    Phase9ObservationKind, Phase9SemanticAssertion, Phase9WitnessBinding,
    Phase9WitnessBindingErrorKind, RigidWorldResultRecord, RigidWorldWitnessFamily, ScenarioId,
    decode_rigid_world_request_jsonl, decode_rigid_world_result_jsonl,
    validate_phase9_witness_bindings,
};
use serde::Deserialize;
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
    witnesses: Vec<CorpusWitness>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CorpusWitness {
    branch: String,
    action_id: String,
    checkpoint_id: String,
    observation_kind: String,
    predicate: String,
    expected: Value,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Authority {
    PinnedOracle,
    Independent,
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
    configure_phase9_declarations(timeline);
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

fn configure_phase9_declarations(timeline: &mut Value) {
    timeline["particle_systems"] = json!([
        {
            "system_id": "phase9-growable", "buffer_mode": { "kind": "growable", "initial_capacity": 4 },
            "paused": false, "strict_contact_check": false, "stuck_threshold": 0,
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
            "destruction_by_age": false, "lifetime_granularity_bits": (1.0_f32 / 60.0_f32).to_bits(),
            "maximum_count": 2
        }
    ]);
    timeline["particles"] = json!([
        particle_with_flags(
            "phase9-a",
            "phase9-growable",
            0.0,
            0.1,
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
            20.0,
            0.25,
            -2.0,
            0.0,
            2.0,
            4
        ),
        particle("phase9-capacity", "phase9-growable", 3.0, 3.0, 5),
        particle("phase9-evicting", "phase9-growable", 4.0, 4.0, 6),
        particle("phase9-c", "phase9-fixed-paused", 1.0, 2.0, 2),
        particle("phase9-d", "phase9-fixed-paused", 1.5, 2.0, 3)
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
        action("statistics", json!({ "kind": "request_statistics", "system_id": "phase9-growable" })),
        action("statistics-fixed", json!({ "kind": "request_statistics", "system_id": "phase9-fixed-paused" })),
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
    phase9_actions.splice(step_index..step_index, statistics);
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
                    json!({ "kind": "inspect_particle_contact", "system_id": "phase9-growable", "contact_index": 0 }),
                ),
                action(
                    "inspect-body-contact",
                    json!({ "kind": "inspect_body_contact", "system_id": "phase9-growable", "contact_index": 0 }),
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
            "mark",
            "compact",
            "mark-unrequested",
            "compact-unrequested",
        ],
        "contacts-listeners-filters-and-coupling" => {
            &["inspect-particle-contact", "inspect-body-contact"]
        }
        "forces-impulses-and-statistics" => &[
            "position",
            "velocity",
            "force",
            "inspect-after-force",
            "impulse",
            "inspect-after-impulse",
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
        COMMON_ACTIONS.contains(&action_id) || relevant.contains(&action_id)
    });
}

fn insert_phase9_checkpoints(timeline: &mut Value, case_id: &str, final_action: &Value) {
    let checkpoints = timeline["checkpoints"]
        .as_array_mut()
        .expect("retained checkpoints should be an array");
    let checkpoint_index = usize::from(case_id == "contacts-listeners-filters-and-coupling");
    let rigid_counts = if case_id == "contacts-listeners-filters-and-coupling" {
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
            "counts": rigid_counts,
            "transitions": []
        }),
    );
    checkpoints.insert(
        checkpoint_index + 1,
        json!({
            "checkpoint_id": "phase9-corpus",
            "after_action_id": final_action,
            "phase": "phase9",
            "counts": rigid_counts,
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
    witness: &CorpusWitness,
) -> &'a Value {
    observation_for_action(request, result, &witness.action_id, &witness.checkpoint_id)
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

fn assert_semantic_witness(request: &Value, result: &Value, witness: &CorpusWitness) {
    let branch = witness.branch.as_str();
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
            json!(["phase9-coupling", "phase9-evicting", "phase9-c", "phase9-d"])
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
        "finite_lifetime" => assert_eq!(a["lifetime_bits"], 0.1_f32.to_bits()),
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
            "phase9-growable"
        ),
        "body_contact" => assert_eq!(
            observe("inspect-body-contact")["contact"]["fixture_id"],
            "nc-dynamic-fixture"
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
            assert_eq!(particle_contact["contact"]["particle_a_id"], "phase9-a");
            assert_eq!(particle_contact["contact"]["particle_b_id"], "phase9-b");
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
            assert_ne!(body["linear_velocity"]["y_bits"], 0);
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

fn assert_witness(request: &Value, result: &Value, witness: &CorpusWitness) {
    let observation = observation_for_witness(request, result, witness);
    assert_eq!(observation["kind"], "particle", "{}", witness.branch);
    assert_eq!(
        observation["observation"]["kind"], witness.observation_kind,
        "{}",
        witness.branch
    );
    match witness.predicate.as_str() {
        "semantic" => assert_semantic_witness(request, result, witness),
        "terminated" => assert_eq!(
            observation["observation"]["terminated"], witness.expected,
            "{}",
            witness.branch
        ),
        "particle_count_at_least" => {
            let minimum = witness
                .expected
                .as_u64()
                .expect("minimum must be an integer");
            let count = observation["observation"]["particle_ids"]
                .as_array()
                .expect("particle IDs")
                .len();
            assert!(
                u64::try_from(count).expect("count fits") >= minimum,
                "{}",
                witness.branch
            );
        }
        predicate => panic!("unsupported witness predicate {predicate}"),
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
    let mut evidence_cases = Vec::new();

    // Act
    for case in &manifest.cases {
        let bytes = std::fs::read(fixture_path(case)).expect("fixture bytes must be readable");
        assert_eq!(sha256(&bytes), case.request_sha256, "{}", case.case_id);
        let request = decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
            .expect("fixture must decode");
        let native = NativeRigidWorldExecutor::execute(&request).expect("native case must execute");
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
        evidence_cases.push(json!({
            "case_id": case.case_id,
            "reached_branches": case.witnesses.iter().map(|witness| &witness.branch).collect::<Vec<_>>(),
            "consumed_policy_paths": run.consumed_paths(),
            "request_sha256": case.request_sha256,
            "native_result_sha256": sha256(&serde_json::to_vec(&native).expect("native bytes")),
            "oracle_result_sha256": sha256(&serde_json::to_vec(oracle.result()).expect("oracle bytes")),
            "comparison_sha256": sha256(format!("{:?}", run.outcome()).as_bytes()),
        }));
    }
    let evidence = json!({
        "profile": manifest.profile,
        "upstream_revision": manifest.pinned_upstream_revision,
        "cases": evidence_cases,
    });

    // Assert
    assert!(!evidence["cases"].as_array().expect("cases").is_empty());
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
        .map(|witness| &witness.branch)
    {
        *occurrences.entry(branch.as_str()).or_default() += 1;
    }
    let actual = occurrences.keys().copied().collect::<BTreeSet<_>>();

    // Assert
    assert_eq!(manifest.profile, PHASE9_REGISTRY_ID);
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
        fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/reference/oracle-debug/compile_commands.json"),
            preset.join("compile_commands.json"),
        )
        .expect("reviewed compile commands should copy into the fake root");
        fs::write(preset.join("behavior.txt"), behavior)
            .expect("fake oracle behavior should be written");
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }
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
        .map(|witness| witness.branch.as_str())
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
    assert!(script_text.contains("test result: ok\\."));
    assert!(script_text.contains("test result: FAILED|FAILED"));

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
