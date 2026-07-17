//! Pinned C++ Phase 9 particle-oracle integration tests.

use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use liquidfun_differential::{
    NativeRigidWorldExecutor, OracleExecutable, OraclePreset, PHASE9_REQUIRED_POLICY_PATHS,
    Phase9ComparisonOutcome, Phase9DifferentialError, compare_phase9_rigid_world_results,
    execute_rigid_world_process, run_phase9_differential,
};
use liquidfun_test_protocol::{HarnessLimits, decode_rigid_world_request_jsonl};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const REQUEST: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/rigid-world-request.jsonl");
const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const WITNESS: &[u8] =
    include_bytes!("../../../reference/artifacts/phase9/lifecycle-contact-witnesses.json");
const WITNESS_PROVENANCE: &str =
    include_str!("../../../reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json");

fn phase9_request() -> liquidfun_test_protocol::RigidWorldRequestRecord {
    let mut value: Value =
        serde_json::from_slice(REQUEST).expect("checked-in Phase 8 request should be JSON");
    let timeline = value["scenario"]["timelines"]
        .as_array_mut()
        .expect("fixture timelines should be an array")
        .first_mut()
        .expect("fixture should contain a timeline");
    timeline["particle_systems"] = json!([{
        "system_id": "oracle-system",
        "buffer_mode": { "kind": "growable", "initial_capacity": 4 },
        "paused": false,
        "strict_contact_check": true,
        "stuck_threshold": 2,
        "density_bits": 1.0_f32.to_bits(),
        "gravity_scale_bits": 1.0_f32.to_bits(),
        "radius_bits": 0.1_f32.to_bits(),
        "damping_bits": 0.0_f32.to_bits(),
        "destruction_by_age": true,
        "lifetime_granularity_bits": (1.0_f32 / 60.0_f32).to_bits(),
        "maximum_count": 8
    }]);
    timeline["particles"] = json!([{
        "particle_id": "oracle-particle",
        "system_id": "oracle-system",
        "position": { "x_bits": 0.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
        "velocity": { "x_bits": 0.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
        "flags_bits": 0,
        "color": [255, 255, 255, 255],
        "lifetime_bits": 1.0_f32.to_bits()
    }]);
    let actions = timeline["actions"]
        .as_array_mut()
        .expect("fixture actions should be an array");
    actions.push(json!({
        "action_id": "oracle-create-system",
        "phase": "phase9",
        "action": {
            "kind": "particle",
            "action": { "kind": "create_system", "system_id": "oracle-system" }
        }
    }));
    actions.push(json!({
        "action_id": "oracle-create-particle",
        "phase": "phase9",
        "action": {
            "kind": "particle",
            "action": { "kind": "create_particle", "particle_id": "oracle-particle" }
        }
    }));
    actions.push(json!({
        "action_id": "oracle-destroy-system",
        "phase": "phase9",
        "action": {
            "kind": "particle",
            "action": { "kind": "destroy_system", "system_id": "oracle-system" }
        }
    }));
    preserve_retained_checkpoint_before_phase9(
        timeline,
        "nc-bodies-destroyed",
        "oracle-destroy-system",
    );

    let mut bytes = serde_json::to_vec(&value).expect("Phase 9 request should encode");
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("bounded Phase 9 request should decode")
}

fn coupling_request() -> liquidfun_test_protocol::RigidWorldRequestRecord {
    let mut value: Value =
        serde_json::from_slice(REQUEST).expect("checked-in Phase 8 request should be JSON");
    let timeline = value["scenario"]["timelines"]
        .as_array_mut()
        .expect("fixture timelines should be an array")
        .first_mut()
        .expect("fixture should contain a timeline");
    timeline["particle_systems"] = json!([{
        "system_id": "coupling-system",
        "buffer_mode": { "kind": "fixed", "capacity": 4 },
        "paused": false,
        "strict_contact_check": true,
        "stuck_threshold": 1,
        "density_bits": 1.0_f32.to_bits(),
        "gravity_scale_bits": 0.0_f32.to_bits(),
        "radius_bits": 0.5_f32.to_bits(),
        "damping_bits": 1.0_f32.to_bits(),
        "destruction_by_age": true,
        "lifetime_granularity_bits": (1.0_f32 / 60.0_f32).to_bits(),
        "maximum_count": 4
    }]);
    timeline["particles"] = json!([{
        "particle_id": "coupling-particle",
        "system_id": "coupling-system",
        "position": { "x_bits": 20.0_f32.to_bits(), "y_bits": 0.25_f32.to_bits() },
        "velocity": { "x_bits": (-2.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() },
        "flags_bits": 0,
        "color": [64, 128, 255, 255],
        "lifetime_bits": 2.0_f32.to_bits()
    }]);
    let actions = timeline["actions"]
        .as_array_mut()
        .expect("fixture actions should be an array");
    let step_index = actions
        .iter()
        .position(|record| record["action_id"] == "nc-step-static-kinematic")
        .expect("first configured step should exist");
    actions.splice(
        step_index..step_index,
        [
            json!({
                "action_id": "coupling-create-system", "phase": "phase9",
                "action": { "kind": "particle", "action": {
                    "kind": "create_system", "system_id": "coupling-system"
                }}
            }),
            json!({
                "action_id": "coupling-create-particle", "phase": "phase9",
                "action": { "kind": "particle", "action": {
                    "kind": "create_particle", "particle_id": "coupling-particle"
                }}
            }),
        ],
    );
    let step_index = actions
        .iter()
        .position(|record| record["action_id"] == "nc-step-static-kinematic")
        .expect("first configured step should remain present");
    let after_step_index = step_index + 1;
    actions.splice(
        after_step_index..after_step_index,
        [
            json!({
                "action_id": "coupling-statistics", "phase": "phase9",
                "action": { "kind": "particle", "action": {
                    "kind": "request_statistics", "system_id": "coupling-system"
                }}
            }),
            json!({
                "action_id": "coupling-destroy-system", "phase": "phase9",
                "action": { "kind": "particle", "action": {
                    "kind": "destroy_system", "system_id": "coupling-system"
                }}
            }),
        ],
    );
    preserve_retained_checkpoint_before_phase9(
        timeline,
        "nc-static-kinematic-rejected",
        "coupling-destroy-system",
    );

    let mut bytes = serde_json::to_vec(&value).expect("coupling request should encode");
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("bounded coupling request should decode")
}

fn preserve_retained_checkpoint_before_phase9(
    timeline: &mut Value,
    checkpoint_id: &str,
    phase9_after_action_id: &str,
) {
    let checkpoints = timeline["checkpoints"]
        .as_array_mut()
        .expect("fixture checkpoints should be an array");
    let checkpoint_index = checkpoints
        .iter()
        .position(|checkpoint| checkpoint["checkpoint_id"] == checkpoint_id)
        .expect("retargeted checkpoint should exist");
    let mut retained = checkpoints[checkpoint_index].clone();
    retained["checkpoint_id"] = json!(format!("{checkpoint_id}-retained"));
    checkpoints.insert(checkpoint_index, retained);
    let phase9 = &mut checkpoints[checkpoint_index + 1];
    phase9["after_action_id"] = json!(phase9_after_action_id);
    phase9["phase"] = json!("phase9");
    phase9["counts"]["destructions"] = json!(0);
    phase9["transitions"] = json!([]);
}

fn full_phase9_request() -> liquidfun_test_protocol::RigidWorldRequestRecord {
    let mut value = serde_json::to_value(phase9_request()).expect("request should serialize");
    let timeline = &mut value["scenario"]["timelines"][0];
    timeline["particle_systems"]
        .as_array_mut()
        .expect("systems should be an array")
        .push(json!({
            "system_id": "oracle-system-newest",
            "buffer_mode": { "kind": "fixed", "capacity": 4 },
            "paused": false, "strict_contact_check": true, "stuck_threshold": 1,
            "density_bits": 1.0_f32.to_bits(), "gravity_scale_bits": 1.0_f32.to_bits(),
            "radius_bits": 0.1_f32.to_bits(), "damping_bits": 0.25_f32.to_bits(),
            "destruction_by_age": true,
            "lifetime_granularity_bits": (1.0_f32 / 60.0_f32).to_bits(),
            "maximum_count": 4
        }));
    timeline["particles"]
        .as_array_mut()
        .expect("particles should be an array")
        .push(json!({
            "particle_id": "oracle-particle-newest", "system_id": "oracle-system-newest",
            "position": { "x_bits": 0.25_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "velocity": { "x_bits": 0.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "flags_bits": 0, "color": [255, 128, 64, 255], "lifetime_bits": 2.0_f32.to_bits()
        }));
    let actions = timeline["actions"]
        .as_array_mut()
        .expect("actions should be an array");
    let destroy_index = actions
        .iter()
        .position(|record| record["action_id"] == "oracle-destroy-system")
        .expect("oldest system destruction should exist");
    actions.splice(
        destroy_index..destroy_index,
        [
            json!({ "action_id": "oracle-create-newest-system", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "create_system", "system_id": "oracle-system-newest" }}}),
            json!({ "action_id": "oracle-create-newest-particle", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "create_particle", "particle_id": "oracle-particle-newest" }}}),
            json!({ "action_id": "oracle-inspect-system", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "inspect_system", "system_id": "oracle-system-newest" }}}),
            json!({ "action_id": "oracle-inspect-particle", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "inspect_particle", "particle_id": "oracle-particle-newest" }}}),
            json!({ "action_id": "oracle-pause", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "set_paused", "system_id": "oracle-system-newest", "paused": true }}}),
            json!({ "action_id": "oracle-resume", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "set_paused", "system_id": "oracle-system-newest", "paused": false }}}),
            json!({ "action_id": "oracle-position", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "set_position", "particle_id": "oracle-particle-newest", "position": { "x_bits": 0.5_f32.to_bits(), "y_bits": 0.0_f32.to_bits() }}}}),
            json!({ "action_id": "oracle-velocity", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "set_velocity", "particle_id": "oracle-particle-newest", "velocity": { "x_bits": 0.0_f32.to_bits(), "y_bits": 1.0_f32.to_bits() }}}}),
            json!({ "action_id": "oracle-force", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "apply_force", "particle_ids": ["oracle-particle-newest"], "force": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() }}}}),
            json!({ "action_id": "oracle-impulse", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "apply_impulse", "particle_ids": ["oracle-particle-newest"], "impulse": { "x_bits": 0.0_f32.to_bits(), "y_bits": 1.0_f32.to_bits() }}}}),
            json!({ "action_id": "oracle-statistics", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "request_statistics", "system_id": "oracle-system-newest" }}}),
            json!({ "action_id": "oracle-query", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "query_aabb", "system_id": null, "lower": { "x_bits": (-1.0_f32).to_bits(), "y_bits": (-1.0_f32).to_bits() }, "upper": { "x_bits": 1.0_f32.to_bits(), "y_bits": 1.0_f32.to_bits() }}}}),
            json!({ "action_id": "oracle-ray", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "ray_cast", "system_id": null, "start": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() }, "end": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() }}}}),
            json!({ "action_id": "oracle-mark", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "mark_for_destruction", "particle_id": "oracle-particle-newest" }}}),
            json!({ "action_id": "oracle-compact", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "compact", "system_id": "oracle-system-newest" }}}),
            json!({ "action_id": "oracle-destroy-newest-system", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "destroy_system", "system_id": "oracle-system-newest" }}}),
        ],
    );
    let mut bytes = serde_json::to_vec(&value).expect("full request should encode");
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("full bounded Phase 9 request should decode")
}

fn static_coupling_request() -> liquidfun_test_protocol::RigidWorldRequestRecord {
    let mut value = serde_json::to_value(coupling_request()).expect("request should serialize");
    let bodies = value["scenario"]["timelines"][0]["bodies"]
        .as_array_mut()
        .expect("bodies should be an array");
    let dynamic = bodies
        .iter_mut()
        .find(|body| body["body_id"] == "nc-dynamic")
        .expect("coupling body should exist");
    dynamic["body_kind"] = json!("static");
    let mut bytes = serde_json::to_vec(&value).expect("static request should encode");
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("static coupling request should decode")
}

fn phase9_action_index(value: &Value, action_id: &str) -> usize {
    value["scenario"]["timelines"][0]["actions"]
        .as_array()
        .expect("actions should be an array")
        .iter()
        .position(|record| record["action_id"] == action_id)
        .expect("requested Phase 9 action should exist")
}

fn raw_oracle_failure(executable: &std::path::Path, value: &Value) -> String {
    let mut bytes = serde_json::to_vec(value).expect("invalid request should encode");
    bytes.push(b'\n');
    let mut child = Command::new(executable)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("oracle should spawn");
    let mut stdout = BufReader::new(child.stdout.take().expect("stdout should be captured"));
    let mut handshake = String::new();
    stdout
        .read_line(&mut handshake)
        .expect("handshake should be readable");
    let mut stdin = child.stdin.take().expect("stdin should be captured");
    stdin
        .write_all(&bytes)
        .and_then(|()| stdin.flush())
        .expect("invalid request should reach the decoder");
    drop(stdin);
    let mut unexpected_stdout = String::new();
    stdout
        .read_to_string(&mut unexpected_stdout)
        .expect("remaining stdout should be readable");
    let output = child.wait_with_output().expect("oracle should be reaped");
    assert!(serde_json::from_str::<Value>(&handshake).is_ok());
    assert!(
        unexpected_stdout.is_empty(),
        "stdout must remain JSONL-only"
    );
    assert!(!output.status.success(), "invalid request must fail hard");
    String::from_utf8(output.stderr).expect("oracle diagnostics should be UTF-8")
}

#[test]
fn decode_accepts_bounded_phase9_request_in_existing_cpp_process() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(executable) = OracleExecutable::resolve(&root, OraclePreset::Debug) else {
        eprintln!("SKIP: build oracle-debug to exercise the pinned C++ decoder");
        return;
    };
    let request = phase9_request();

    // Act
    let result = execute_rigid_world_process(&executable, &request, REVISION);

    // Assert
    assert!(result.is_ok(), "bounded Phase 9 request failed: {result:?}");
}

#[test]
fn decode_accepts_negative_finite_lifetime_as_infinite_in_cpp() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(executable) = OracleExecutable::resolve(&root, OraclePreset::Debug) else {
        eprintln!("SKIP: build oracle-debug to exercise the pinned C++ decoder");
        return;
    };
    let mut value = serde_json::to_value(phase9_request()).expect("request should serialize");
    value["scenario"]["timelines"][0]["particles"][0]["lifetime_bits"] =
        json!((-1.0_f32).to_bits());
    let mut bytes = serde_json::to_vec(&value).expect("negative lifetime request should encode");
    bytes.push(b'\n');
    let request = decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("finite negative lifetime should cross the Rust boundary");

    // Act
    let result = execute_rigid_world_process(&executable, &request, REVISION);

    // Assert
    assert!(
        result.is_ok(),
        "negative lifetime must remain infinite: {result:?}"
    );
    assert_eq!(
        request.scenario().timelines()[0].particles()[0]
            .lifetime_bits
            .bits(),
        (-1.0_f32).to_bits()
    );
}

#[test]
fn mixed_identity_matches_native_declaration_order() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(executable) = OracleExecutable::resolve(&root, OraclePreset::Debug) else {
        eprintln!("SKIP: build oracle-debug to exercise mixed identity");
        return;
    };
    let request = coupling_request();

    // Act
    let native = NativeRigidWorldExecutor::execute(&request)
        .expect("native mixed identity request should execute");
    let oracle = execute_rigid_world_process(&executable, &request, REVISION)
        .expect("oracle mixed identity request should execute");
    let native_value = serde_json::to_value(native).expect("native result should serialize");
    let oracle_value =
        serde_json::to_value(oracle.result()).expect("oracle result should serialize");
    let find_live_mixed = |value: &Value| {
        value["timelines"][0]["checkpoints"]
            .as_array()
            .expect("checkpoints should be an array")
            .iter()
            .filter_map(|checkpoint| checkpoint["observations"].as_array())
            .flatten()
            .find(|observation| {
                observation["observation"]["kind"] == "mixed_state"
                    && observation["observation"]["particle_ids"]
                        .as_array()
                        .is_some_and(|ids| !ids.is_empty())
            })
            .expect("a live mixed-state observation should exist")
            .clone()
    };

    // Assert
    let native_mixed = find_live_mixed(&native_value);
    let oracle_mixed = find_live_mixed(&oracle_value);
    assert_eq!(
        oracle_mixed["observation"]["body_ids"],
        native_mixed["observation"]["body_ids"]
    );
    assert_eq!(
        oracle_mixed["observation"]["particle_ids"],
        native_mixed["observation"]["particle_ids"]
    );
}

#[test]
fn pinned_witness_is_consumed_before_generalized_oracle_execution() {
    // Arrange
    let witness: Value = serde_json::from_slice(WITNESS).expect("witness should be JSON");
    let provenance: Value =
        serde_json::from_str(WITNESS_PROVENANCE).expect("provenance should be JSON");

    // Act
    let digest = format!("{:x}", Sha256::digest(WITNESS));
    let oldest = witness["witnesses"][0]["oldest_selection_order"]
        .as_array()
        .expect("oldest order should be an array");
    let strict = witness["witnesses"][1]["strict_order"]
        .as_array()
        .expect("strict order should be an array");

    // Assert
    assert_eq!(witness["oracle_revision"], REVISION);
    assert_eq!(provenance["oracle_revision"], REVISION);
    assert_eq!(provenance["witness_sha256"], digest);
    assert_eq!(
        oldest,
        json!([
            "particle-7",
            "particle-6",
            "particle-5",
            "particle-4",
            "particle-3",
            "particle-2",
            "particle-1",
            "particle-0"
        ])
        .as_array()
        .expect("expected oldest order should be an array")
    );
    assert_eq!(
        strict,
        json!(["fixture-5", "fixture-4", "fixture-3", "fixture-2"])
            .as_array()
            .expect("expected strict order should be an array")
    );
}

#[test]
fn decode_rejects_phase10_group_topology_as_hard_cpp_harness_failure() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let executable = root.join("target/reference/oracle-debug/liquidfun-reference");
    if !executable.is_file() {
        eprintln!("SKIP: build oracle-debug to exercise the pinned C++ decoder");
        return;
    }
    let mut value = serde_json::to_value(phase9_request()).expect("request should serialize");
    value["scenario"]["timelines"][0]["particle_groups"] = json!([{ "group_id": "phase10-group" }]);
    let mut bytes = serde_json::to_vec(&value).expect("invalid request should encode");
    bytes.push(b'\n');
    let mut child = Command::new(executable)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("oracle should spawn");
    let mut stdout = BufReader::new(child.stdout.take().expect("stdout should be captured"));
    let mut handshake = String::new();
    stdout
        .read_line(&mut handshake)
        .expect("handshake should be readable");

    // Act
    let mut stdin = child.stdin.take().expect("stdin should be captured");
    stdin
        .write_all(&bytes)
        .and_then(|()| stdin.flush())
        .expect("invalid request should reach the decoder");
    drop(stdin);
    let mut unexpected_stdout = String::new();
    stdout
        .read_to_string(&mut unexpected_stdout)
        .expect("remaining stdout should be readable");
    let output = child.wait_with_output().expect("oracle should be reaped");

    // Assert
    assert!(serde_json::from_str::<Value>(&handshake).is_ok());
    assert!(
        unexpected_stdout.is_empty(),
        "stdout must remain JSONL-only"
    );
    assert!(!output.status.success(), "Phase 10 request must fail hard");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("unknown member particle_groups"),
        "stderr should classify the undeclared Phase 10 family: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn decode_rejects_invalid_phase9_lifecycle_matrix_before_execution() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let executable = root.join("target/reference/oracle-debug/liquidfun-reference");
    if !executable.is_file() {
        eprintln!("SKIP: build oracle-debug to exercise lifecycle rejection");
        return;
    }
    let base = serde_json::to_value(full_phase9_request()).expect("request should serialize");
    let mut duplicate_system = base.clone();
    let index = phase9_action_index(&duplicate_system, "oracle-create-newest-system");
    let duplicate = duplicate_system["scenario"]["timelines"][0]["actions"][index].clone();
    duplicate_system["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .insert(index + 1, duplicate);

    let mut use_before_create = base.clone();
    let system_index = phase9_action_index(&use_before_create, "oracle-create-newest-system");
    let particle_index = phase9_action_index(&use_before_create, "oracle-create-newest-particle");
    use_before_create["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .swap(system_index, particle_index);

    let mut duplicate_particle = base.clone();
    let index = phase9_action_index(&duplicate_particle, "oracle-create-newest-particle");
    let duplicate = duplicate_particle["scenario"]["timelines"][0]["actions"][index].clone();
    duplicate_particle["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .insert(index + 1, duplicate);

    let mut unknown_particle = base.clone();
    let index = phase9_action_index(&unknown_particle, "oracle-inspect-particle");
    unknown_particle["scenario"]["timelines"][0]["actions"][index]["action"]["action"]["particle_id"] =
        json!("unknown-particle");

    let mut pending_use = base.clone();
    let index = phase9_action_index(&pending_use, "oracle-mark");
    pending_use["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .insert(
            index + 1,
            json!({ "action_id": "oracle-inspect-pending", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "inspect_particle", "particle_id": "oracle-particle-newest" }
            }}),
        );

    let mut repeated_mark = base.clone();
    let index = phase9_action_index(&repeated_mark, "oracle-mark");
    repeated_mark["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .insert(
            index + 1,
            json!({ "action_id": "oracle-mark-again", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "mark_for_destruction", "particle_id": "oracle-particle-newest" }
            }}),
        );

    let mut cross_system_range = base.clone();
    let index = phase9_action_index(&cross_system_range, "oracle-force");
    cross_system_range["scenario"]["timelines"][0]["actions"][index]["action"]["action"]["particle_ids"] =
        json!(["oracle-particle-newest", "oracle-particle"]);

    let mut destroyed_owner = base.clone();
    let index = phase9_action_index(&destroyed_owner, "oracle-destroy-newest-system");
    destroyed_owner["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .insert(
            index + 1,
            json!({ "action_id": "oracle-query-destroyed", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "query_aabb", "system_id": "oracle-system-newest",
                    "lower": { "x_bits": 0, "y_bits": 0 }, "upper": { "x_bits": 1_065_353_216, "y_bits": 1_065_353_216 } }
            }}),
        );

    let mut recreate_after_compaction = base.clone();
    let index = phase9_action_index(&recreate_after_compaction, "oracle-compact");
    recreate_after_compaction["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .insert(
            index + 1,
            json!({ "action_id": "oracle-recreate", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "create_particle", "particle_id": "oracle-particle-newest" }
            }}),
        );
    let mutations = [
        duplicate_system,
        use_before_create,
        duplicate_particle,
        unknown_particle,
        pending_use,
        repeated_mark,
        cross_system_range,
        destroyed_owner,
        recreate_after_compaction,
    ];

    // Act
    let diagnostics = mutations.map(|value| raw_oracle_failure(&executable, &value));

    // Assert
    assert!(
        diagnostics
            .iter()
            .all(|message| message.contains("Phase 9"))
    );
}

#[test]
fn coupling_trace_records_body_contact_and_rigid_reaction() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(executable) = OracleExecutable::resolve(&root, OraclePreset::Debug) else {
        eprintln!("SKIP: build oracle-debug to exercise pinned coupling");
        return;
    };
    let request = coupling_request();

    // Act
    let captured = execute_rigid_world_process(&executable, &request, REVISION)
        .expect("bounded coupling request should execute");
    let value = serde_json::to_value(captured.result()).expect("result should serialize");
    let checkpoint = value["timelines"][0]["checkpoints"]
        .as_array()
        .expect("checkpoints should be an array")
        .iter()
        .find(|checkpoint| checkpoint["checkpoint_id"] == "nc-static-kinematic-rejected")
        .expect("coupling checkpoint should exist");
    let statistics = checkpoint["observations"]
        .as_array()
        .expect("observations should be an array")
        .iter()
        .find(|observation| observation["observation"]["kind"] == "statistics")
        .expect("source statistics should be recorded");
    let dynamic = checkpoint["bodies"]
        .as_array()
        .expect("bodies should be an array")
        .iter()
        .find(|body| body["body_id"] == "nc-dynamic")
        .expect("dynamic coupling body should remain live");

    // Assert
    assert!(statistics["observation"]["statistics"]["body_contact_count"] != 0);
    assert!(
        dynamic["linear_velocity"]["x_bits"] != 0
            || dynamic["linear_velocity"]["y_bits"] != 0
            || dynamic["angular_velocity_bits"] != 0,
        "off-center particle contact should produce a rigid reaction"
    );
}

#[test]
fn coupling_static_body_contact_remains_stationary() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(executable) = OracleExecutable::resolve(&root, OraclePreset::Debug) else {
        eprintln!("SKIP: build oracle-debug to exercise static coupling");
        return;
    };
    let request = static_coupling_request();

    // Act
    let captured = execute_rigid_world_process(&executable, &request, REVISION)
        .expect("static coupling request should execute");
    let value = serde_json::to_value(captured.result()).expect("result should serialize");
    let checkpoint = value["timelines"][0]["checkpoints"]
        .as_array()
        .expect("checkpoints should be an array")
        .iter()
        .find(|checkpoint| checkpoint["checkpoint_id"] == "nc-static-kinematic-rejected")
        .expect("coupling checkpoint should exist");
    let dynamic = checkpoint["bodies"]
        .as_array()
        .expect("bodies should be an array")
        .iter()
        .find(|body| body["body_id"] == "nc-dynamic")
        .expect("static coupling body should remain live");

    // Assert
    assert_eq!(dynamic["body_kind"], "static");
    assert_eq!(dynamic["linear_velocity"]["x_bits"], 0);
    assert_eq!(dynamic["linear_velocity"]["y_bits"], 0);
    assert_eq!(dynamic["angular_velocity_bits"], 0);
}

#[test]
fn every_phase9_action_family_round_trips_with_semantic_observations() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(executable) = OracleExecutable::resolve(&root, OraclePreset::Debug) else {
        eprintln!("SKIP: build oracle-debug to exercise the full particle surface");
        return;
    };
    let request = full_phase9_request();
    let action_count = request.scenario().timelines()[0]
        .actions()
        .iter()
        .filter(|record| record.phase() == "phase9")
        .count();

    // Act
    let captured = execute_rigid_world_process(&executable, &request, REVISION)
        .expect("every Phase 9 action family should execute");
    let value = serde_json::to_value(captured.result()).expect("result should serialize");
    let observations = value["timelines"][0]["checkpoints"]
        .as_array()
        .expect("checkpoints should be an array")
        .iter()
        .flat_map(|checkpoint| checkpoint["observations"].as_array().into_iter().flatten())
        .filter(|observation| observation["kind"] == "particle")
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(observations.len(), action_count);
    for expected in ["mixed_state", "statistics", "query", "ray_cast"] {
        assert!(
            observations
                .iter()
                .any(|observation| observation["observation"]["kind"] == expected),
            "missing semantic observation family {expected}"
        );
    }
}

#[test]
fn long_lived_phase9_process_resets_between_requests() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let executable = root.join("target/reference/oracle-debug/liquidfun-reference");
    if !executable.is_file() {
        eprintln!("SKIP: build oracle-debug to exercise process reuse");
        return;
    }
    let mut request = serde_json::to_vec(&full_phase9_request()).expect("request should encode");
    request.push(b'\n');
    let mut child = Command::new(executable)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("oracle should spawn");
    let mut stdout = BufReader::new(child.stdout.take().expect("stdout should be captured"));
    let mut handshake = String::new();
    stdout
        .read_line(&mut handshake)
        .expect("handshake should be readable");

    // Act
    let mut stdin = child.stdin.take().expect("stdin should be captured");
    stdin
        .write_all(&request)
        .expect("first request should write");
    stdin.flush().expect("first request should flush");
    let mut records = Vec::new();
    for _ in 0..2 {
        let mut record = String::new();
        stdout
            .read_line(&mut record)
            .expect("record should be readable");
        records.push(serde_json::from_str::<Value>(&record).expect("stdout must be JSONL"));
    }
    stdin
        .write_all(&request)
        .expect("second request should write");
    stdin.flush().expect("second request should flush");
    for _ in 0..2 {
        let mut record = String::new();
        stdout
            .read_line(&mut record)
            .expect("record should be readable");
        records.push(serde_json::from_str::<Value>(&record).expect("stdout must be JSONL"));
    }
    drop(stdin);
    let output = child.wait_with_output().expect("oracle should be reaped");

    // Assert
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(records[0]["record_kind"], "rigid_world_result");
    assert_eq!(records[1]["reset_epoch"], 1);
    assert_eq!(records[2]["record_kind"], "rigid_world_result");
    assert_eq!(records[3]["reset_epoch"], 2);
}

#[test]
fn differential_runner_hashes_one_request_for_native_and_cpp_roles() {
    // Arrange
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(executable) = OracleExecutable::resolve(&root, OraclePreset::Debug) else {
        eprintln!("SKIP: build oracle-debug to exercise the Phase 9 differential runner");
        return;
    };
    let request = coupling_request();

    // Act
    let run = run_phase9_differential(&executable, &request, REVISION)
        .expect("the native and pinned C++ Phase 9 results should compare");

    // Assert
    assert_eq!(run.native_request_sha256(), run.request_sha256());
    assert_eq!(run.oracle_request_sha256(), run.request_sha256());
    assert_eq!(run.consumed_paths(), PHASE9_REQUIRED_POLICY_PATHS);
    assert!(
        matches!(run.outcome(), Phase9ComparisonOutcome::Match { .. }),
        "unexpected Phase 9 differential outcome: {:?}",
        run.outcome()
    );
}

#[test]
fn differential_comparison_rejects_a_deterministic_semantic_mutation() {
    // Arrange
    let request = full_phase9_request();
    let native =
        NativeRigidWorldExecutor::execute(&request).expect("native Phase 9 request should execute");
    let mut mutated_value = serde_json::to_value(&native).expect("result should serialize");
    let statistics = mutated_value["timelines"][0]["checkpoints"]
        .as_array_mut()
        .expect("checkpoints should be an array")
        .iter_mut()
        .filter_map(|checkpoint| {
            checkpoint
                .get_mut("observations")
                .and_then(Value::as_array_mut)
        })
        .flatten()
        .find(|observation| {
            observation["kind"] == "particle" && observation["observation"]["kind"] == "statistics"
        })
        .expect("statistics observation should exist");
    statistics["observation"]["statistics"]["particle_contact_count"] = json!(1);
    let mut bytes = serde_json::to_vec(&mutated_value).expect("mutation should encode");
    bytes.push(b'\n');
    let mutated = liquidfun_test_protocol::decode_rigid_world_result_jsonl(
        &bytes,
        &HarnessLimits::phase2_default_v1(),
    )
    .expect("mutation should remain bounded");

    // Act
    let first = compare_phase9_rigid_world_results(&request, &native, &mutated)
        .expect("semantic disagreement should not be a harness failure");
    let second = compare_phase9_rigid_world_results(&request, &native, &mutated)
        .expect("replay should preserve mismatch classification");

    // Assert
    let (
        Phase9ComparisonOutcome::PhysicsMismatch(first),
        Phase9ComparisonOutcome::PhysicsMismatch(second),
    ) = (first, second)
    else {
        panic!("the deterministic mutation must be a physics mismatch");
    };
    assert_eq!(first.semantic_path(), "particle.statistics.counts");
    assert_eq!(first.signature_sha256(), second.signature_sha256());
}

#[test]
fn differential_runner_keeps_malformed_child_output_as_harness_failure() {
    // Arrange
    let fake = FakeOracleRoot::new("malformed");
    let executable = OracleExecutable::resolve(fake.path(), OraclePreset::Debug)
        .expect("fake oracle should occupy the reviewed preset path");
    let request = full_phase9_request();

    // Act
    let result = run_phase9_differential(&executable, &request, REVISION);

    // Assert
    let error = result.expect_err("malformed child output must fail");
    assert!(matches!(error, Phase9DifferentialError::Oracle(_)));
}

struct FakeOracleRoot {
    root: PathBuf,
}

impl FakeOracleRoot {
    fn new(behavior: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should follow the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "liquidfun-phase9-oracle-{}-{nonce}",
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
        fs::write(preset.join("behavior.txt"), behavior)
            .expect("fake oracle behavior should be written");
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }
}

impl Drop for FakeOracleRoot {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).expect("fake oracle root should be removable");
    }
}
