//! Pinned process-oracle coverage for the strict Phase 10 extension.

use std::{
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

use liquidfun_differential::{OracleExecutable, OraclePreset, execute_rigid_world_process};
use liquidfun_test_protocol::{
    FloatBits, HarnessLimits, Phase10BehaviorLeaf, Phase10EventKind, Phase10Observation,
    RigidWorldObservation, WitnessRole, decode_rigid_world_request_jsonl,
    decode_rigid_world_result_jsonl, validate_rigid_world_result_against_request,
};
use serde_json::{Value, json};

const BASE_REQUEST: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/rigid-world-request.jsonl");
const UPSTREAM_REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";

fn bits(value: f32) -> u32 {
    FloatBits::from_f32(value).bits()
}

fn vector(x: f32, y: f32) -> Value {
    json!({ "x_bits": bits(x), "y_bits": bits(y) })
}

fn system() -> Value {
    json!({
        "system_id": "system-a",
        "buffer_mode": { "kind": "growable", "initial_capacity": 32 },
        "paused": false,
        "strict_contact_check": true,
        "stuck_threshold": 2,
        "density_bits": bits(1.0),
        "gravity_scale_bits": bits(1.0),
        "radius_bits": bits(0.25),
        "damping_bits": bits(0.0),
        "destruction_by_age": true,
        "lifetime_granularity_bits": bits(1.0 / 60.0),
        "maximum_count": null
    })
}

fn definition(group_id: &str, member_ids: &[&str], positions: &[(f32, f32)]) -> Value {
    json!({
        "provenance": {
            "extension_version": 1,
            "generator_id": "phase10-oracle-test",
            "generator_version": "v1",
            "upstream_revision": UPSTREAM_REVISION,
            "toolchain_id": "pinned-cpp-oracle",
            "seed": 42
        },
        "system_id": "system-a",
        "group_id": group_id,
        "member_ids": member_ids,
        "source": {
            "kind": "explicit",
            "positions": positions.iter().map(|(x, y)| vector(*x, *y)).collect::<Vec<_>>()
        },
        "destination": { "kind": "new" },
        "particle_flags_bits": 1 << 3,
        "group_flags_bits": 0,
        "transform": { "position": vector(0.0, 0.0), "angle_bits": bits(0.0) },
        "linear_velocity": vector(0.0, 0.0),
        "angular_velocity_bits": bits(0.0),
        "color": [1, 2, 3, 4],
        "strength_bits": bits(1.0),
        "maybe_stride_bits": null,
        "lifetime_bits": bits(0.0)
    })
}

fn record(action_id: &str, action: Value) -> Value {
    json!({ "action_id": action_id, "phase": "phase10", "action": action })
}

fn particle_action(kind: &str) -> Value {
    json!({ "kind": "particle", "action": { "kind": kind, "system_id": "system-a" } })
}

fn group_action(operation: Value) -> Value {
    json!({ "kind": "particle_group", "operation": operation })
}

fn request_value() -> Value {
    let mut append = definition("group-b", &["particle-d"], &[(20.25, 0.0)]);
    append["destination"] = json!({ "kind": "append_to", "target_group_id": "group-b" });
    let actions = vec![
        record("p10-create-system", particle_action("create_system")),
        record(
            "p10-create-a",
            group_action(json!({
                "kind": "create_group",
                "definition": definition(
                    "group-a",
                    &["particle-a", "particle-b"],
                    &[(0.0, 0.0), (10.0, 0.0)]
                )
            })),
        ),
        record(
            "p10-split-a",
            group_action(json!({
                "kind": "split_group", "group_id": "group-a",
                "created_group_ids": ["group-c"]
            })),
        ),
        record(
            "p10-create-b",
            group_action(json!({
                "kind": "create_group",
                "definition": definition("group-b", &["particle-c"], &[(20.0, 0.0)])
            })),
        ),
        record(
            "p10-append-b",
            group_action(json!({ "kind": "create_group", "definition": append })),
        ),
        record(
            "p10-join",
            group_action(json!({
                "kind": "join_groups", "target_group_id": "group-c",
                "source_group_id": "group-b"
            })),
        ),
        record(
            "p10-flags",
            group_action(json!({
                "kind": "set_group_flags", "group_id": "group-a", "group_flags_bits": 3
            })),
        ),
        record(
            "p10-step",
            group_action(json!({
                "kind": "step", "timestep_bits": bits(1.0 / 60.0),
                "velocity_iterations": 8, "position_iterations": 3, "particle_iterations": 2
            })),
        ),
        record(
            "p10-inspect",
            group_action(json!({ "kind": "inspect_state" })),
        ),
        record(
            "p10-destroy-a",
            group_action(json!({ "kind": "destroy_group", "group_id": "group-a" })),
        ),
        record(
            "p10-destroy-c",
            group_action(json!({ "kind": "destroy_group", "group_id": "group-c" })),
        ),
        record(
            "p10-compact",
            group_action(json!({
                "kind": "step", "timestep_bits": bits(1.0 / 60.0),
                "velocity_iterations": 8, "position_iterations": 3, "particle_iterations": 1
            })),
        ),
        record("p10-destroy-system", particle_action("destroy_system")),
    ];
    request_with_actions(actions, "p10-destroy-system")
}

fn request_with_actions(actions: Vec<Value>, final_action_id: &str) -> Value {
    let mut value: Value = serde_json::from_slice(BASE_REQUEST).expect("base request should parse");
    let timeline = value["scenario"]["timelines"]
        .as_array_mut()
        .expect("timelines should be an array")
        .first_mut()
        .expect("base request should contain a timeline");
    timeline["particle_systems"] = json!([system()]);
    timeline["particles"] = json!([]);
    timeline["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .extend(actions);
    let checkpoint = timeline["checkpoints"]
        .as_array_mut()
        .expect("checkpoints should be an array")
        .last_mut()
        .expect("timeline should contain a checkpoint");
    checkpoint["after_action_id"] = json!(final_action_id);
    checkpoint["phase"] = json!("phase10");
    value
}

fn source_request_value() -> Value {
    let mut filled = definition("group-fill", &["particle-fill"], &[(0.0, 0.0)]);
    filled["source"] = json!({
        "kind": "filled",
        "shapes": [{
            "kind": "circle", "center": vector(0.0, 0.0), "radius_bits": bits(0.1)
        }]
    });
    filled["maybe_stride_bits"] = json!(bits(1.0));
    let mut stroke = definition(
        "group-stroke",
        &["particle-stroke-a", "particle-stroke-b"],
        &[(0.0, 0.0), (0.5, 0.0)],
    );
    stroke["source"] = json!({
        "kind": "stroke",
        "shape": {
            "kind": "edge", "vertex_a": vector(0.0, 0.0), "vertex_b": vector(1.0, 0.0)
        }
    });
    stroke["maybe_stride_bits"] = json!(bits(0.5));
    request_with_actions(
        vec![
            record("source-create-system", particle_action("create_system")),
            record(
                "source-filled",
                group_action(json!({ "kind": "create_group", "definition": filled })),
            ),
            record(
                "source-stroke",
                group_action(json!({ "kind": "create_group", "definition": stroke })),
            ),
            record(
                "source-inspect",
                group_action(json!({ "kind": "inspect_state" })),
            ),
            record(
                "source-destroy-filled",
                group_action(json!({ "kind": "destroy_group", "group_id": "group-fill" })),
            ),
            record(
                "source-destroy-stroke",
                group_action(json!({ "kind": "destroy_group", "group_id": "group-stroke" })),
            ),
            record(
                "source-step",
                group_action(json!({
                    "kind": "step", "timestep_bits": bits(1.0 / 60.0),
                    "velocity_iterations": 8, "position_iterations": 3,
                    "particle_iterations": 1
                })),
            ),
            record("source-destroy-system", particle_action("destroy_system")),
        ],
        "source-destroy-system",
    )
}

fn decode(value: &Value) -> liquidfun_test_protocol::RigidWorldRequestRecord {
    let mut bytes = serde_json::to_vec(value).expect("request should encode");
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("Phase 10 request should validate")
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn oracle() -> Option<OracleExecutable> {
    OracleExecutable::resolve(&root(), OraclePreset::Debug).ok()
}

fn phase10_state(
    result: &liquidfun_test_protocol::RigidWorldResultRecord,
) -> &liquidfun_test_protocol::Phase10StateObservation {
    result
        .timelines()
        .iter()
        .flat_map(|timeline| &timeline.checkpoints)
        .flat_map(|checkpoint| &checkpoint.observations)
        .find_map(|observation| {
            let RigidWorldObservation::ParticleGroup {
                observation: Phase10Observation::State { state },
            } = observation
            else {
                return None;
            };
            Some(state)
        })
        .expect("oracle result should contain a Phase 10 state")
}

#[test]
fn oracle_executes_every_group_operation_and_captures_semantic_identity() {
    // Arrange
    let Some(executable) = oracle() else {
        eprintln!("SKIP: build with cargo xtask upstream configure/build --preset oracle-debug");
        return;
    };
    let request = decode(&request_value());

    // Act
    let capture = execute_rigid_world_process(&executable, &request, UPSTREAM_REVISION)
        .expect("pinned Phase 10 oracle should execute");
    let state = phase10_state(capture.result());

    // Assert
    assert_eq!(state.groups.len(), 2);
    assert_eq!(state.particles.len(), 4);
    assert_eq!(state.groups[0].group_id.as_str(), "group-a");
    assert_eq!(state.groups[1].group_id.as_str(), "group-c");
    assert!(
        state
            .events
            .iter()
            .any(|event| event.kind == Phase10EventKind::GroupsJoined)
    );
    assert!(
        state
            .events
            .iter()
            .any(|event| event.kind == Phase10EventKind::GroupSplit)
    );
    for leaf in [
        Phase10BehaviorLeaf::GroupCreate,
        Phase10BehaviorLeaf::GroupAppend,
        Phase10BehaviorLeaf::GroupJoin,
        Phase10BehaviorLeaf::GroupSplit,
        Phase10BehaviorLeaf::GroupFlags,
    ] {
        assert!(state.witnesses.iter().any(|witness| {
            witness.behavior_leaf == leaf && witness.role == WitnessRole::Activation
        }));
    }
}

#[test]
fn oracle_replay_is_exact_and_contains_no_private_pass_trace() {
    // Arrange
    let Some(executable) = oracle() else {
        eprintln!("SKIP: build with cargo xtask upstream configure/build --preset oracle-debug");
        return;
    };
    let request = decode(&request_value());

    // Act
    let first = execute_rigid_world_process(&executable, &request, UPSTREAM_REVISION)
        .expect("first oracle replay should execute");
    let second = execute_rigid_world_process(&executable, &request, UPSTREAM_REVISION)
        .expect("second oracle replay should execute");

    // Assert
    assert_eq!(first.result(), second.result());
    let encoded = serde_json::to_string(first.result()).expect("result should encode");
    assert!(!encoded.contains("pass_id"));
    assert!(!encoded.contains("pass_trace"));
}

#[test]
fn oracle_executes_filled_and_stroke_sources_in_declared_member_order() {
    // Arrange
    let Some(executable) = oracle() else {
        eprintln!("SKIP: build with cargo xtask upstream configure/build --preset oracle-debug");
        return;
    };
    let request = decode(&source_request_value());

    // Act
    let capture = execute_rigid_world_process(&executable, &request, UPSTREAM_REVISION)
        .expect("filled and stroke source request should execute");
    let state = phase10_state(capture.result());

    // Assert
    assert_eq!(state.groups[0].member_ids[0].as_str(), "particle-fill");
    assert_eq!(
        state.groups[1]
            .member_ids
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>(),
        ["particle-stroke-a", "particle-stroke-b"]
    );
}

#[test]
fn request_loop_rejects_malformed_input_then_recovers_for_a_valid_batch() {
    // Arrange
    let Some(_executable) = oracle() else {
        eprintln!("SKIP: build with cargo xtask upstream configure/build --preset oracle-debug");
        return;
    };
    let program = root().join("target/reference/oracle-debug/liquidfun-reference");
    let mut child = Command::new(program)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("oracle process should spawn");
    let stdout = child.stdout.take().expect("stdout should be piped");
    let mut reader = BufReader::new(stdout);
    let mut handshake = String::new();
    reader
        .read_line(&mut handshake)
        .expect("handshake should read");
    let request = decode(&request_value());
    let mut stdin = child.stdin.take().expect("stdin should be piped");

    // Act
    let mut malformed = request_value();
    malformed["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .iter_mut()
        .find(|record| record["action_id"] == "p10-inspect")
        .expect("inspect operation should exist")["action"]["operation"]["unreviewed"] =
        json!(true);
    serde_json::to_writer(&mut stdin, &malformed).expect("malformed request should write");
    stdin
        .write_all(b"\n")
        .expect("malformed request terminator should write");
    let mut oversized = request_value();
    oversized["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .iter_mut()
        .find(|record| record["action_id"] == "p10-step")
        .expect("step operation should exist")["action"]["operation"]["particle_iterations"] =
        json!(1025);
    serde_json::to_writer(&mut stdin, &oversized).expect("oversized request should write");
    stdin
        .write_all(b"\n")
        .expect("oversized request terminator should write");
    serde_json::to_writer(&mut stdin, &request).expect("valid request should write");
    stdin
        .write_all(b"\n")
        .expect("request terminator should write");
    drop(stdin);
    let mut result = String::new();
    reader.read_line(&mut result).expect("result should read");
    let mut end = String::new();
    reader.read_line(&mut end).expect("end should read");
    let output = child
        .wait_with_output()
        .expect("oracle should exit cleanly");
    let decoded_result =
        decode_rigid_world_result_jsonl(result.as_bytes(), &HarnessLimits::phase2_default_v1())
            .expect("recovered result should satisfy the closed result schema");
    validate_rigid_world_result_against_request(&request, &decoded_result)
        .expect("recovered result should match its request declarations");

    // Assert
    assert!(handshake.contains("handshake"));
    assert!(
        result.contains("rigid_world_result"),
        "expected result record, received {result:?} with stderr {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(end.contains("rigid_world_end"));
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("request rejected"));
    assert!(stderr.contains("unknown member"));
    assert!(stderr.contains("reviewed bounds"));
}

#[test]
fn cargo_workspace_has_no_production_cpp_oracle_dependency() {
    // Arrange
    let manifest =
        std::fs::read_to_string(root().join("Cargo.toml")).expect("workspace manifest should read");
    let liquidfun_manifest = std::fs::read_to_string(root().join("crates/liquidfun/Cargo.toml"))
        .expect("published crate manifest should read");

    // Act
    let combined = format!("{manifest}\n{liquidfun_manifest}");

    // Assert
    assert!(!combined.contains("tools/reference"));
    assert!(!combined.contains("build.rs"));
    assert!(!root().join("crates/liquidfun/build.rs").exists());
}
