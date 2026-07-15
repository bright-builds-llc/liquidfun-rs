//! Pinned C++ Phase 9 particle-oracle integration tests.

use std::{
    io::{BufRead, BufReader, Read, Write},
    process::{Command, Stdio},
};

use liquidfun_differential::{OracleExecutable, OraclePreset, execute_rigid_world_process};
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
    let checkpoint = timeline["checkpoints"]
        .as_array_mut()
        .expect("fixture checkpoints should be an array")
        .last_mut()
        .expect("fixture should contain a checkpoint");
    checkpoint["after_action_id"] = json!("oracle-destroy-system");
    checkpoint["phase"] = json!("phase9");

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
    actions.splice(
        step_index + 1..step_index + 1,
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
    let checkpoint = timeline["checkpoints"]
        .as_array_mut()
        .expect("fixture checkpoints should be an array")
        .iter_mut()
        .find(|checkpoint| checkpoint["checkpoint_id"] == "nc-static-kinematic-rejected")
        .expect("first step checkpoint should exist");
    checkpoint["after_action_id"] = json!("coupling-destroy-system");
    checkpoint["phase"] = json!("phase9");

    let mut bytes = serde_json::to_vec(&value).expect("coupling request should encode");
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("bounded coupling request should decode")
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
