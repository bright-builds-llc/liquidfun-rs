//! Pinned C++ Phase 9 particle-oracle integration tests.

use std::{
    io::{BufRead, BufReader, Read, Write},
    process::{Command, Stdio},
};

use liquidfun_differential::{OracleExecutable, OraclePreset, execute_rigid_world_process};
use liquidfun_test_protocol::{HarnessLimits, decode_rigid_world_request_jsonl};
use serde_json::{Value, json};

const REQUEST: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/rigid-world-request.jsonl");
const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";

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
