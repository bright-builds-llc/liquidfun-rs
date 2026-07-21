//! Native Phase 10 adapter coverage through the strict shared protocol.

use liquidfun_differential::{NativeRigidWorldError, NativeRigidWorldExecutor};
use liquidfun_test_protocol::{
    FloatBits, HarnessLimits, Phase10BehaviorLeaf, Phase10EventKind, Phase10Observation,
    RigidWorldErrorKind, RigidWorldObservation, WitnessRole, decode_rigid_world_request_jsonl,
};
use serde_json::{Value, json};

const PHASE8_REQUEST: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/rigid-world-request.jsonl");

fn bits(value: f32) -> u32 {
    FloatBits::from_f32(value).bits()
}

fn vector(x: f32, y: f32) -> Value {
    json!({ "x_bits": bits(x), "y_bits": bits(y) })
}

fn provenance() -> Value {
    json!({
        "extension_version": 1,
        "generator_id": "phase10-native-test",
        "generator_version": "v1",
        "upstream_revision": "7f20402173fd143a3988c921bc384459c6a858f2",
        "toolchain_id": "rust-native",
        "seed": 42
    })
}

fn definition(
    system_id: &str,
    group_id: &str,
    member_ids: &[&str],
    positions: &[(f32, f32)],
) -> Value {
    json!({
        "provenance": provenance(),
        "system_id": system_id,
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

fn system_declaration(system_id: &str, capacity: usize, fixed: bool) -> Value {
    let buffer_mode = if fixed {
        json!({ "kind": "fixed", "capacity": capacity })
    } else {
        json!({ "kind": "growable", "initial_capacity": capacity })
    };
    json!({
        "system_id": system_id,
        "buffer_mode": buffer_mode,
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

#[allow(
    clippy::needless_pass_by_value,
    reason = "fixture builder consumes logical ownership into a JSON object"
)]
fn action(action_id: &str, action: Value) -> Value {
    json!({ "action_id": action_id, "phase": "phase10", "action": action })
}

fn particle_action(kind: &str, system_id: &str) -> Value {
    json!({ "kind": "particle", "action": { "kind": kind, "system_id": system_id } })
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "fixture builder consumes logical ownership into a JSON object"
)]
fn group_action(operation: Value) -> Value {
    json!({ "kind": "particle_group", "operation": operation })
}

fn base_value(systems: Vec<Value>, phase10_actions: Vec<Value>, final_action_id: &str) -> Value {
    let mut value: Value =
        serde_json::from_slice(PHASE8_REQUEST).expect("Phase 8 fixture should be JSON");
    let timeline = value["scenario"]["timelines"]
        .as_array_mut()
        .expect("fixture timelines should be an array")
        .first_mut()
        .expect("fixture should contain a timeline");
    timeline["particle_systems"] = Value::Array(systems);
    timeline["particles"] = json!([]);
    timeline["actions"]
        .as_array_mut()
        .expect("fixture actions should be an array")
        .extend(phase10_actions);
    let checkpoint = timeline["checkpoints"]
        .as_array_mut()
        .expect("fixture checkpoints should be an array")
        .last_mut()
        .expect("fixture should contain a checkpoint");
    checkpoint["after_action_id"] = json!(final_action_id);
    checkpoint["phase"] = json!("phase10");
    value
}

fn decode(value: &Value) -> liquidfun_test_protocol::RigidWorldRequestRecord {
    decode_result(value).expect("native Phase 10 request should validate")
}

fn decode_result(
    value: &Value,
) -> Result<
    liquidfun_test_protocol::RigidWorldRequestRecord,
    liquidfun_test_protocol::RigidWorldDecodeError,
> {
    let mut bytes = serde_json::to_vec(value).expect("fixture mutation should encode");
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
}

fn observation(result: &liquidfun_test_protocol::RigidWorldResultRecord) -> &Phase10Observation {
    result
        .timelines()
        .iter()
        .flat_map(|timeline| timeline.checkpoints.iter())
        .flat_map(|checkpoint| checkpoint.observations.iter())
        .find_map(|observation| {
            let RigidWorldObservation::ParticleGroup { observation } = observation else {
                return None;
            };
            Some(observation)
        })
        .expect("result should contain a Phase 10 state observation")
}

fn operation_family_value() -> Value {
    let mut append = definition("system-a", "group-b", &["particle-d"], &[(20.25, 0.0)]);
    append["destination"] = json!({ "kind": "append_to", "target_group_id": "group-b" });
    let actions = vec![
        action(
            "p10-create-system",
            particle_action("create_system", "system-a"),
        ),
        action(
            "p10-create-a",
            group_action(json!({
                "kind": "create_group",
                "definition": definition(
                    "system-a",
                    "group-a",
                    &["particle-a", "particle-b"],
                    &[(0.0, 0.0), (10.0, 0.0)]
                )
            })),
        ),
        action(
            "p10-split-a",
            group_action(json!({
                "kind": "split_group",
                "group_id": "group-a",
                "created_group_ids": ["group-c"]
            })),
        ),
        action(
            "p10-create-b",
            group_action(json!({
                "kind": "create_group",
                "definition": definition("system-a", "group-b", &["particle-c"], &[(20.0, 0.0)])
            })),
        ),
        action(
            "p10-append-b",
            group_action(json!({ "kind": "create_group", "definition": append })),
        ),
        action(
            "p10-join",
            group_action(json!({
                "kind": "join_groups",
                "target_group_id": "group-c",
                "source_group_id": "group-b"
            })),
        ),
        action(
            "p10-flags",
            group_action(json!({
                "kind": "set_group_flags",
                "group_id": "group-a",
                "group_flags_bits": 3
            })),
        ),
        action(
            "p10-step",
            group_action(json!({
                "kind": "step",
                "timestep_bits": bits(1.0 / 60.0),
                "velocity_iterations": 8,
                "position_iterations": 3,
                "particle_iterations": 2
            })),
        ),
        action(
            "p10-inspect",
            group_action(json!({ "kind": "inspect_state" })),
        ),
        action(
            "p10-destroy-a",
            group_action(json!({ "kind": "destroy_group", "group_id": "group-a" })),
        ),
        action(
            "p10-destroy-c",
            group_action(json!({ "kind": "destroy_group", "group_id": "group-c" })),
        ),
        action(
            "p10-compact",
            group_action(json!({
                "kind": "step",
                "timestep_bits": bits(1.0 / 60.0),
                "velocity_iterations": 8,
                "position_iterations": 3,
                "particle_iterations": 1
            })),
        ),
        action(
            "p10-destroy-system",
            particle_action("destroy_system", "system-a"),
        ),
    ];
    base_value(
        vec![system_declaration("system-a", 32, false)],
        actions,
        "p10-destroy-system",
    )
}

#[test]
fn native_executes_group_mutation_step_and_complete_capture_families() {
    // Arrange
    let request = decode(&operation_family_value());

    // Act
    let result = NativeRigidWorldExecutor::execute(&request)
        .expect("complete Phase 10 operation family should execute");
    let Phase10Observation::State { state } = observation(&result);

    // Assert
    assert_eq!(state.groups.len(), 2);
    assert_eq!(state.particles.len(), 4);
    assert!(
        state
            .groups
            .iter()
            .all(|group| group.system_id.as_str() == "system-a")
    );
    assert!(
        state
            .groups
            .iter()
            .any(|group| group.group_id.as_str() == "group-a")
    );
    assert!(
        state
            .groups
            .iter()
            .any(|group| group.group_id.as_str() == "group-c")
    );
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
    observation(&result)
        .validate_semantics()
        .expect("complete native capture should satisfy the strict result schema");
}

#[test]
fn native_replays_the_same_seed_to_exact_semantic_identity_and_order() {
    // Arrange
    let request = decode(&operation_family_value());

    // Act
    let first = NativeRigidWorldExecutor::execute(&request).expect("first replay should execute");
    let second = NativeRigidWorldExecutor::execute(&request).expect("second replay should execute");

    // Assert
    assert_eq!(first, second);
}

#[test]
fn native_executes_filled_and_stroke_sources_in_declared_member_order() {
    // Arrange
    let mut filled = definition("system-a", "group-fill", &["particle-fill"], &[(0.0, 0.0)]);
    filled["source"] = json!({
        "kind": "filled",
        "shapes": [{ "kind": "circle", "center": vector(0.0, 0.0), "radius_bits": bits(0.1) }]
    });
    filled["maybe_stride_bits"] = json!(bits(1.0));
    let mut stroke = definition(
        "system-a",
        "group-stroke",
        &["particle-stroke-a", "particle-stroke-b"],
        &[(0.0, 0.0), (0.5, 0.0)],
    );
    stroke["source"] = json!({
        "kind": "stroke",
        "shape": { "kind": "edge", "vertex_a": vector(0.0, 0.0), "vertex_b": vector(1.0, 0.0) }
    });
    stroke["maybe_stride_bits"] = json!(bits(0.5));
    let actions = source_actions(filled, stroke);
    let request = decode(&base_value(
        vec![system_declaration("system-a", 16, false)],
        actions,
        "source-destroy-system",
    ));

    // Act
    let result = NativeRigidWorldExecutor::execute(&request)
        .expect("filled and stroke sources should execute");
    let Phase10Observation::State { state } = observation(&result);

    // Assert
    assert_eq!(state.groups[0].member_ids[0].as_str(), "particle-fill");
    assert_eq!(
        state.groups[1]
            .member_ids
            .iter()
            .map(liquidfun_test_protocol::ScenarioId::as_str)
            .collect::<Vec<_>>(),
        ["particle-stroke-a", "particle-stroke-b"]
    );
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "fixture builder consumes logical ownership into scenario JSON"
)]
fn source_actions(filled: Value, stroke: Value) -> Vec<Value> {
    vec![
        action(
            "source-create-system",
            particle_action("create_system", "system-a"),
        ),
        action(
            "source-fill",
            group_action(json!({ "kind": "create_group", "definition": filled })),
        ),
        action(
            "source-stroke",
            group_action(json!({ "kind": "create_group", "definition": stroke })),
        ),
        action(
            "source-inspect",
            group_action(json!({ "kind": "inspect_state" })),
        ),
        action(
            "source-destroy-fill",
            group_action(json!({ "kind": "destroy_group", "group_id": "group-fill" })),
        ),
        action(
            "source-destroy-stroke",
            group_action(json!({ "kind": "destroy_group", "group_id": "group-stroke" })),
        ),
        action(
            "source-step",
            group_action(json!({
                "kind": "step",
                "timestep_bits": bits(1.0 / 60.0),
                "velocity_iterations": 8,
                "position_iterations": 3,
                "particle_iterations": 1
            })),
        ),
        action(
            "source-destroy-system",
            particle_action("destroy_system", "system-a"),
        ),
    ]
}

#[test]
fn native_preserves_multi_system_ownership_in_complete_capture() {
    // Arrange
    let actions = vec![
        action(
            "multi-create-a",
            particle_action("create_system", "system-a"),
        ),
        action(
            "multi-create-b",
            particle_action("create_system", "system-b"),
        ),
        action(
            "multi-group-a",
            group_action(json!({
                "kind": "create_group",
                "definition": definition("system-a", "group-a", &["particle-a"], &[(0.0, 0.0)])
            })),
        ),
        action(
            "multi-group-b",
            group_action(json!({
                "kind": "create_group",
                "definition": definition("system-b", "group-b", &["particle-b"], &[(1.0, 0.0)])
            })),
        ),
        action(
            "multi-inspect",
            group_action(json!({ "kind": "inspect_state" })),
        ),
        action(
            "multi-destroy-a",
            group_action(json!({ "kind": "destroy_group", "group_id": "group-a" })),
        ),
        action(
            "multi-destroy-b",
            group_action(json!({ "kind": "destroy_group", "group_id": "group-b" })),
        ),
        action(
            "multi-step",
            group_action(
                json!({ "kind": "step", "timestep_bits": bits(1.0 / 60.0), "velocity_iterations": 8, "position_iterations": 3, "particle_iterations": 1 }),
            ),
        ),
        action(
            "multi-system-a",
            particle_action("destroy_system", "system-a"),
        ),
        action(
            "multi-system-b",
            particle_action("destroy_system", "system-b"),
        ),
    ];
    let request = decode(&base_value(
        vec![
            system_declaration("system-a", 8, false),
            system_declaration("system-b", 8, false),
        ],
        actions,
        "multi-system-b",
    ));

    // Act
    let result = NativeRigidWorldExecutor::execute(&request)
        .expect("independent systems should execute without owner confusion");
    let Phase10Observation::State { state } = observation(&result);

    // Assert
    assert_eq!(state.groups[0].system_id.as_str(), "system-a");
    assert_eq!(state.groups[1].system_id.as_str(), "system-b");
    assert_eq!(state.particles[0].system_id, state.groups[0].system_id);
    assert_eq!(state.particles[1].system_id, state.groups[1].system_id);
}

#[test]
fn native_capacity_failure_returns_one_typed_error_without_a_result() {
    // Arrange
    let actions = vec![
        action(
            "capacity-create",
            particle_action("create_system", "system-a"),
        ),
        action(
            "capacity-group",
            group_action(json!({
                "kind": "create_group",
                "definition": definition(
                    "system-a",
                    "group-a",
                    &["particle-a", "particle-b"],
                    &[(0.0, 0.0), (1.0, 0.0)]
                )
            })),
        ),
        action(
            "capacity-destroy",
            group_action(json!({ "kind": "destroy_group", "group_id": "group-a" })),
        ),
        action(
            "capacity-system",
            particle_action("destroy_system", "system-a"),
        ),
    ];
    let request = decode(&base_value(
        vec![system_declaration("system-a", 1, true)],
        actions,
        "capacity-system",
    ));

    // Act
    let error = NativeRigidWorldExecutor::execute(&request)
        .expect_err("fixed capacity should reject the complete request");

    // Assert
    assert!(matches!(error, NativeRigidWorldError::Action { .. }));
    assert!(error.to_string().contains("capacity"));
}

#[test]
fn strict_boundary_rejects_stale_and_wrong_owner_ids_before_native_execution() {
    // Arrange
    let mut stale = operation_family_value();
    let stale_action = stale["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .iter_mut()
        .find(|action| action["action_id"] == "p10-flags")
        .expect("flags action should exist");
    stale_action["action"]["operation"]["group_id"] = json!("stale-group");

    let mut wrong_owner = operation_family_value();
    wrong_owner["scenario"]["timelines"][0]["particle_systems"]
        .as_array_mut()
        .expect("systems should be an array")
        .push(system_declaration("system-b", 8, false));
    let actions = wrong_owner["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array");
    let append_index = actions
        .iter()
        .position(|action| action["action_id"] == "p10-append-b")
        .expect("append action should exist");
    actions.insert(
        append_index,
        action(
            "p10-create-system-b",
            particle_action("create_system", "system-b"),
        ),
    );
    actions[append_index + 1]["action"]["operation"]["definition"]["system_id"] = json!("system-b");
    let destroy_index = actions
        .iter()
        .position(|action| action["action_id"] == "p10-destroy-system")
        .expect("system teardown should exist");
    actions.insert(
        destroy_index + 1,
        action(
            "p10-destroy-system-b",
            particle_action("destroy_system", "system-b"),
        ),
    );
    wrong_owner["scenario"]["timelines"][0]["checkpoints"]
        .as_array_mut()
        .expect("checkpoints should be an array")
        .last_mut()
        .expect("checkpoint should exist")["after_action_id"] = json!("p10-destroy-system-b");

    // Act
    let stale_error = decode_result(&stale).expect_err("stale group IDs must fail validation");
    let owner_error =
        decode_result(&wrong_owner).expect_err("cross-system append must fail validation");

    // Assert
    assert_eq!(
        stale_error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidParticleGroupAction)
    );
    assert_eq!(
        owner_error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidParticleGroupAction)
    );
}
