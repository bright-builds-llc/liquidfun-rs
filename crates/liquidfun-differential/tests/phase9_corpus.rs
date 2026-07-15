//! Closed Phase 9 witness-corpus and evidence-boundary tests.

use std::collections::{BTreeMap, BTreeSet};

use liquidfun_differential::{
    NativeRigidWorldExecutor, OracleExecutable, OraclePreset, PHASE9_REGISTRY_ID,
    PHASE9_REQUIRED_POLICY_PATHS, execute_rigid_world_process, phase9_policy_for_path,
};
use liquidfun_test_protocol::{
    HarnessLimits, RigidWorldWitnessFamily, decode_rigid_world_request_jsonl,
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const RETAINED_REQUEST: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/rigid-world-request.jsonl");
const MANIFEST: &str = include_str!("fixtures/rigid_world/phase9/phase9-v1.json");
const PINNED_WITNESS: &[u8] =
    include_bytes!("../../../reference/artifacts/phase9/lifecycle-contact-witnesses.json");

const REQUIRED_BRANCHES: &[&str] = &[
    "multiple_systems",
    "newest_first",
    "paused_system",
    "stable_ids_sort",
    "stable_ids_rotate",
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
    branches: Vec<String>,
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

fn bounded_phase9_request() -> liquidfun_test_protocol::RigidWorldRequestRecord {
    let mut value = request_value();
    let timeline = &mut value["scenario"]["timelines"][0];
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
        particle("phase9-a", "phase9-growable", 0.0, 1.0, 0),
        particle("phase9-b", "phase9-growable", 0.5, 0.0, 1),
        particle("phase9-c", "phase9-fixed-paused", 1.0, 2.0, 2),
        particle("phase9-d", "phase9-fixed-paused", 1.5, 2.0, 3)
    ]);
    let actions = timeline["actions"]
        .as_array_mut()
        .expect("retained actions should be an array");
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
    for id in ["phase9-a", "phase9-b", "phase9-c", "phase9-d"] {
        phase9_actions.push(action(
            &format!("create-{id}"),
            json!({ "kind": "create_particle", "particle_id": id }),
        ));
    }
    phase9_actions.extend([
        action("inspect-system", json!({ "kind": "inspect_system", "system_id": "phase9-growable" })),
        action("inspect-particle", json!({ "kind": "inspect_particle", "particle_id": "phase9-a" })),
        action("resume", json!({ "kind": "set_paused", "system_id": "phase9-fixed-paused", "paused": false })),
        action("position", json!({ "kind": "set_position", "particle_id": "phase9-a", "position": bits(0.25, 0.0) })),
        action("velocity", json!({ "kind": "set_velocity", "particle_id": "phase9-a", "velocity": bits(0.0, 1.0) })),
        action("force", json!({ "kind": "apply_force", "particle_ids": ["phase9-a", "phase9-b"], "force": bits(1.0, 0.0) })),
        action("impulse", json!({ "kind": "apply_impulse", "particle_ids": ["phase9-a", "phase9-b"], "impulse": bits(0.0, 1.0) })),
        action("statistics", json!({ "kind": "request_statistics", "system_id": "phase9-growable" })),
        action("system-query", json!({ "kind": "query_aabb", "system_id": "phase9-growable", "lower": bits(-1.0, -1.0), "upper": bits(2.0, 2.0) })),
        action("world-query", json!({ "kind": "query_aabb", "system_id": null, "lower": bits(-1.0, -1.0), "upper": bits(2.0, 2.0) })),
        action("system-ray", json!({ "kind": "ray_cast", "system_id": "phase9-growable", "start": bits(-1.0, 0.0), "end": bits(2.0, 0.0) })),
        action("world-ray", json!({ "kind": "ray_cast", "system_id": null, "start": bits(-1.0, 0.0), "end": bits(2.0, 0.0) })),
        action("mark", json!({ "kind": "mark_for_destruction", "particle_id": "phase9-a" })),
        action("compact", json!({ "kind": "compact", "system_id": "phase9-growable" })),
        action("destroy-fixed", json!({ "kind": "destroy_system", "system_id": "phase9-fixed-paused" })),
        action("destroy-growable", json!({ "kind": "destroy_system", "system_id": "phase9-growable" })),
    ]);
    let final_action = phase9_actions
        .last()
        .expect("Phase 9 corpus should have a final action")["action_id"]
        .clone();
    actions.splice(0..0, phase9_actions);
    let checkpoints = timeline["checkpoints"]
        .as_array_mut()
        .expect("retained checkpoints should be an array");
    checkpoints.insert(
        0,
        json!({
            "checkpoint_id": "phase9-corpus",
            "after_action_id": final_action,
            "phase": "phase9",
            "counts": {
                "bodies": 0, "fixtures": 0, "contacts": 0,
                "manifold_points": 0, "events": 0, "destructions": 0
            },
            "transitions": []
        }),
    );
    decode_value(&value).expect("the bounded Phase 9 corpus should decode")
}

fn particle(id: &str, system: &str, x: f32, lifetime: f32, color: u8) -> Value {
    json!({
        "particle_id": id, "system_id": system, "position": bits(x, 0.0),
        "velocity": bits(0.0, 0.0), "flags_bits": 0, "color": [color, 0, 255, 255],
        "lifetime_bits": lifetime.to_bits()
    })
}

fn bits(x: f32, y: f32) -> Value {
    json!({ "x_bits": x.to_bits(), "y_bits": y.to_bits() })
}

fn action(id: &str, action: Value) -> Value {
    json!({
        "action_id": id, "phase": "phase9",
        "action": { "kind": "particle", "action": action }
    })
}

#[test]
fn manifest_declares_every_phase9_branch_exactly_once() {
    // Arrange
    let manifest = manifest();
    let required = REQUIRED_BRANCHES.iter().copied().collect::<BTreeSet<_>>();

    // Act
    let mut occurrences = BTreeMap::<&str, usize>::new();
    for branch in manifest.cases.iter().flat_map(|case| &case.branches) {
        *occurrences.entry(branch).or_default() += 1;
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
    let request = bounded_phase9_request();

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

#[test]
fn corpus_rejects_missing_declarations_and_phase10_members() {
    // Arrange
    let manifest = manifest();
    let mut missing = serde_json::from_str::<Value>(MANIFEST).expect("manifest should be JSON");
    missing["cases"][0]["branches"]
        .as_array_mut()
        .expect("branches should be an array")
        .remove(0);

    // Act / Assert
    let decoded: CorpusManifest = serde_json::from_value(missing).expect("shape remains valid");
    let decoded_branches = decoded
        .cases
        .iter()
        .flat_map(|case| &case.branches)
        .map(String::as_str)
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
    let request = bounded_phase9_request();
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

    // Assert
    assert_eq!(first.response_bytes(), replay.response_bytes());
    assert_eq!(first.result(), replay.result());
    if mode == "canonical" {
        let release = OracleExecutable::resolve(&root, OraclePreset::Release)
            .expect("the required release Phase 9 oracle must exist");
        let optimized = execute_rigid_world_process(&release, &request, &revision)
            .expect("the release Phase 9 oracle run should pass");
        assert_eq!(first.result(), optimized.result());
    }
}
