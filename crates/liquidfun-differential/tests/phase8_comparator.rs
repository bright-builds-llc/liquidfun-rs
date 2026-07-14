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
    let mutations: Vec<(&str, Box<dyn Fn(&mut serde_json::Value)>)> = vec![
        (
            "joint-kind",
            Box::new(|value| {
                observation_mut(value, 9, "joint")["snapshot"]["joint_kind"] =
                    serde_json::Value::String("distance".into());
            }),
        ),
        (
            "joint-coordinate",
            Box::new(|value| {
                observation_mut(value, 9, "joint")["snapshot"]["coordinate_bits"] =
                    serde_json::Value::from(0x3f80_0000_u32);
            }),
        ),
        (
            "rope-vertex",
            Box::new(|value| {
                observation_mut(value, 15, "rope")["snapshot"]["vertices"][1]["x_bits"] =
                    serde_json::Value::from(1_u32);
            }),
        ),
        (
            "reconstruction-support",
            Box::new(|value| {
                observation_mut(value, 18, "reconstruction")["record"]["support"] =
                    serde_json::Value::String("unsupported_mouse_joint".into());
            }),
        ),
        (
            "tree-quality",
            Box::new(|value| {
                observation_mut(value, 18, "diagnostics")["snapshot"]["tree_quality_bits"] =
                    serde_json::Value::from(0_u32);
            }),
        ),
    ];

    // Act / Assert
    for (label, mutate) in mutations {
        let oracle = mutated_result(&native, mutate);
        let outcome = compare_phase8_rigid_world_results(
            &request, &native, &oracle, &phase6, &phase7, &phase8,
        );
        assert!(
            !matches!(outcome, Ok(RigidComparisonOutcome::Match)),
            "Phase 8 mutation {label} must fail closed"
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

    // Act / Assert
    assert!(Phase8PolicyProfile::parse_toml(&wildcard).is_err());
    assert!(Phase8PolicyProfile::parse_toml(&missing).is_err());
}
