//! Closed Phase 8 comparator coverage.

use liquidfun_differential::{
    NativeRigidWorldExecutor, RigidComparisonOutcome, compare_phase8_rigid_world_results,
};
use liquidfun_test_protocol::{
    HarnessLimits, Phase6PolicyProfile, Phase7PolicyProfile, Phase8PolicyProfile,
    decode_rigid_world_request_jsonl, decode_rigid_world_result_jsonl,
};

const REQUEST: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/rigid-world-request.jsonl");
const PHASE6: &str = include_str!("../../../protocol/tolerances/phase6-v1.toml");
const PHASE7: &str = include_str!("../../../protocol/tolerances/phase7-v1.toml");
const PHASE8: &str = include_str!("../../../protocol/tolerances/phase8-v1.toml");

type JsonMutation = Box<dyn Fn(&mut serde_json::Value)>;

fn fixtures() -> (
    liquidfun_test_protocol::RigidWorldRequestRecord,
    liquidfun_test_protocol::RigidWorldResultRecord,
    Phase6PolicyProfile,
    Phase7PolicyProfile,
    Phase8PolicyProfile,
) {
    let request = decode_rigid_world_request_jsonl(REQUEST, &HarnessLimits::phase2_default_v1())
        .expect("Phase 8 request should decode");
    let native = NativeRigidWorldExecutor::execute(&request)
        .expect("Phase 8 request should execute natively");
    (
        request,
        native,
        Phase6PolicyProfile::parse_toml(PHASE6).expect("Phase 6 policy should parse"),
        Phase7PolicyProfile::parse_toml(PHASE7).expect("Phase 7 policy should parse"),
        Phase8PolicyProfile::parse_toml(PHASE8).expect("Phase 8 policy should parse"),
    )
}

fn mutated_result(
    native: &liquidfun_test_protocol::RigidWorldResultRecord,
    mutate: impl FnOnce(&mut serde_json::Value),
) -> liquidfun_test_protocol::RigidWorldResultRecord {
    let mut value = serde_json::to_value(native).expect("result should serialize");
    mutate(&mut value);
    let mut bytes = serde_json::to_vec(&value).expect("mutated result should serialize");
    bytes.push(b'\n');
    decode_rigid_world_result_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("bounded mutation should decode")
}

fn observation_mut<'a>(
    value: &'a mut serde_json::Value,
    timeline_index: usize,
    kind: &str,
) -> &'a mut serde_json::Value {
    value["timelines"][timeline_index]["checkpoints"][0]["observations"]
        .as_array_mut()
        .expect("checkpoint observations should be an array")
        .iter_mut()
        .find(|observation| observation["kind"] == kind)
        .expect("strengthened checkpoint should contain the requested observation kind")
}

fn joint_mut<'a>(value: &'a mut serde_json::Value, joint_id: &str) -> &'a mut serde_json::Value {
    value["timelines"]
        .as_array_mut()
        .expect("timelines should be an array")
        .iter_mut()
        .filter_map(|timeline| {
            timeline
                .get_mut("checkpoints")?
                .as_array_mut()?
                .first_mut()?
                .get_mut("observations")?
                .as_array_mut()
        })
        .flatten()
        .find(|observation| {
            observation["kind"] == "joint" && observation["snapshot"]["joint_id"] == joint_id
        })
        .expect("joint observation should exist")
}

fn assert_physics_path(
    request: &liquidfun_test_protocol::RigidWorldRequestRecord,
    native: &liquidfun_test_protocol::RigidWorldResultRecord,
    oracle: &liquidfun_test_protocol::RigidWorldResultRecord,
    phase6: &Phase6PolicyProfile,
    phase7: &Phase7PolicyProfile,
    phase8: &Phase8PolicyProfile,
    expected_path: &str,
) {
    let outcome =
        compare_phase8_rigid_world_results(request, native, oracle, phase6, phase7, phase8)
            .expect("registered mutation should compare");
    let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
        panic!("mutation must produce a physics mismatch at {expected_path}");
    };
    assert_eq!(report.semantic_path(), expected_path);
}

#[test]
fn complete_phase8_result_matches_itself() {
    // Arrange
    let (request, native, phase6, phase7, phase8) = fixtures();

    // Act
    let outcome =
        compare_phase8_rigid_world_results(&request, &native, &native, &phase6, &phase7, &phase8)
            .expect("closed policies should compare");

    // Assert
    assert!(matches!(outcome, RigidComparisonOutcome::Match));
}

#[test]
fn every_phase8_observation_family_reports_a_first_divergence() {
    // Arrange
    let (request, native, phase6, phase7, phase8) = fixtures();
    let mutations: Vec<(&str, &str, JsonMutation)> = vec![
        (
            "joint-kind",
            "rigid_world.phase8.joint.kind",
            Box::new(|value| {
                observation_mut(value, 9, "joint")["snapshot"]["joint_kind"] =
                    serde_json::Value::String("distance".into());
            }),
        ),
        (
            "joint-coordinate",
            "rigid_world.phase8.joint.coordinate",
            Box::new(|value| {
                observation_mut(value, 9, "joint")["snapshot"]["coordinate_bits"] =
                    serde_json::Value::from(0x3f80_0000_u32);
            }),
        ),
        (
            "rope-vertex",
            "rigid_world.phase8.rope.vertex.x",
            Box::new(|value| {
                observation_mut(value, 15, "rope")["snapshot"]["vertices"][1]["x_bits"] =
                    serde_json::Value::from(1_u32);
            }),
        ),
        (
            "reconstruction-support",
            "rigid_world.phase8.reconstruction.support",
            Box::new(|value| {
                observation_mut(value, 18, "reconstruction")["record"]["support"] =
                    serde_json::Value::String("unsupported_mouse_joint".into());
            }),
        ),
        (
            "tree-quality",
            "rigid_world.phase8.diagnostics.tree_quality",
            Box::new(|value| {
                observation_mut(value, 18, "diagnostics")["snapshot"]["tree_quality_bits"] =
                    serde_json::Value::from(0_u32);
            }),
        ),
    ];

    // Act / Assert
    for (label, expected_path, mutate) in mutations {
        let oracle = mutated_result(&native, mutate);
        assert_physics_path(
            &request,
            &native,
            &oracle,
            &phase6,
            &phase7,
            &phase8,
            expected_path,
        );
        assert!(!label.is_empty());
    }
}

#[test]
fn joint_topology_branch_speed_reactions_and_every_gear_lane_have_stable_paths() {
    // Arrange
    let (request, native, phase6, phase7, phase8) = fixtures();
    let mutations: Vec<(&str, JsonMutation)> = vec![
        (
            "rigid_world.phase8.joint.body_ids",
            Box::new(|value| {
                joint_mut(value, "diagnostic-revolute")["snapshot"]["body_b_id"] =
                    serde_json::Value::String("diagnostic-moving-b".into());
            }),
        ),
        (
            "rigid_world.phase8.joint.collide_connected",
            Box::new(|value| {
                joint_mut(value, "diagnostic-revolute")["snapshot"]["collide_connected"] =
                    serde_json::Value::Bool(true);
            }),
        ),
        (
            "rigid_world.phase8.joint.dependencies.order",
            Box::new(|value| {
                joint_mut(value, "diagnostic-gear")["snapshot"]["dependencies"]
                    .as_array_mut()
                    .expect("dependencies")
                    .reverse();
            }),
        ),
        (
            "rigid_world.phase8.joint.branch_state",
            Box::new(|value| {
                joint_mut(value, "diagnostic-revolute")["snapshot"]["branch_state"] =
                    serde_json::Value::String("active".into());
            }),
        ),
        (
            "rigid_world.phase8.joint.speed",
            Box::new(|value| {
                joint_mut(value, "joint-def-revolute")["snapshot"]["speed_bits"] =
                    serde_json::Value::from(0x3f80_0000_u32);
            }),
        ),
        (
            "rigid_world.phase8.joint.reaction_force.x",
            Box::new(|value| {
                joint_mut(value, "joint-def-revolute")["snapshot"]["reaction_force"]["x_bits"] =
                    serde_json::Value::from(0x447a_0000_u32);
            }),
        ),
        (
            "rigid_world.phase8.joint.reaction_force.y",
            Box::new(|value| {
                joint_mut(value, "joint-def-revolute")["snapshot"]["reaction_force"]["y_bits"] =
                    serde_json::Value::from(0x3f80_0000_u32);
            }),
        ),
        (
            "rigid_world.phase8.joint.reaction_torque",
            Box::new(|value| {
                joint_mut(value, "joint-def-prismatic")["snapshot"]["reaction_torque_bits"] =
                    serde_json::Value::from(0x447a_0000_u32);
            }),
        ),
    ];
    let gear_ids = [
        "gear-0-joint",
        "gear-1-joint",
        "gear-2-joint",
        "gear-3-joint",
    ];

    // Act / Assert
    for (expected_path, mutate) in mutations {
        let oracle = mutated_result(&native, mutate);
        assert_physics_path(
            &request,
            &native,
            &oracle,
            &phase6,
            &phase7,
            &phase8,
            expected_path,
        );
    }
    for gear_id in gear_ids {
        let oracle = mutated_result(&native, |value| {
            joint_mut(value, gear_id)["snapshot"]["coordinate_bits"] =
                serde_json::Value::from(0x447a_0000_u32);
        });
        assert_physics_path(
            &request,
            &native,
            &oracle,
            &phase6,
            &phase7,
            &phase8,
            "rigid_world.phase8.joint.coordinate",
        );
    }
}

#[test]
fn lifecycle_multiplicity_kind_identity_and_destruction_identity_have_stable_paths() {
    // Arrange
    let (request, native, phase6, phase7, phase8) = fixtures();

    // Act / Assert
    let extra = mutated_result(&native, |value| {
        let observations = value["timelines"][16]["checkpoints"][0]["observations"]
            .as_array_mut()
            .expect("observations");
        let duplicate = observations[8].clone();
        observations.insert(9, duplicate);
        for (ordinal, observation) in observations
            .iter_mut()
            .filter(|observation| observation["kind"] == "lifecycle")
            .enumerate()
        {
            let ordinal = u32::try_from(ordinal).expect("fixture ordinal should fit in u32");
            observation["event"]["ordinal"] = serde_json::Value::from(ordinal);
        }
    });
    assert_physics_path(
        &request,
        &native,
        &extra,
        &phase6,
        &phase7,
        &phase8,
        "rigid_world.phase8.lifecycle.multiplicity",
    );

    for (expected_path, mutate) in [
        ("rigid_world.phase8.lifecycle.kind", 5_usize),
        ("rigid_world.phase8.lifecycle.identity", 2_usize),
        ("rigid_world.phase8.lifecycle.identity", 12_usize),
    ] {
        let oracle = mutated_result(&native, |value| {
            let event = &mut value["timelines"][if mutate == 12 { 17 } else { 16 }]["checkpoints"]
                [0]["observations"][mutate]["event"];
            if expected_path.ends_with("kind") {
                event["kind"] = serde_json::Value::String("pre_solve".into());
            } else if mutate == 12 {
                event["maybe_entity_id"] = serde_json::Value::String("destruction-base-a".into());
            } else {
                event["maybe_contact"]["occurrence"] = serde_json::Value::from(2_u32);
            }
        });
        assert_physics_path(
            &request,
            &native,
            &oracle,
            &phase6,
            &phase7,
            &phase8,
            expected_path,
        );
    }
}

#[test]
fn signed_zero_reaction_is_not_collapsed() {
    // Arrange
    let (request, native, phase6, phase7, phase8) = fixtures();
    let oracle = mutated_result(&native, |value| {
        observation_mut(value, 9, "joint")["snapshot"]["reaction_force"]["x_bits"] =
            serde_json::Value::from(0x8000_0000_u32);
    });

    // Act
    let outcome =
        compare_phase8_rigid_world_results(&request, &native, &oracle, &phase6, &phase7, &phase8)
            .expect("registered reaction policy should compare");

    // Assert
    let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
        panic!("signed zero must remain a physics-visible mismatch");
    };
    assert_eq!(
        report.semantic_path(),
        "rigid_world.phase8.joint.reaction_force.x"
    );
}

#[test]
fn wildcard_and_missing_phase8_policies_are_rejected() {
    // Arrange
    let wildcard = PHASE8.replacen(
        "rigid_world.phase8.joint.id",
        "rigid_world.phase8.joint.*",
        1,
    );
    let missing = PHASE8.replacen("[[fields]]", "[[removed_fields]]", 1);
    let widened = PHASE8.replacen("relative_bits = 981668463", "relative_bits = 1017370378", 1);

    // Act / Assert
    assert!(Phase8PolicyProfile::parse_toml(&wildcard).is_err());
    assert!(Phase8PolicyProfile::parse_toml(&missing).is_err());
    assert!(Phase8PolicyProfile::parse_toml(&widened).is_err());
}
