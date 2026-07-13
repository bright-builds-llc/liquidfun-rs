use std::time::Duration;

use liquidfun_differential::{
    MinimizationBudget, NativeRigidWorldExecutor, RigidComparisonOutcome, RigidEvaluation,
    RigidMismatchKind, compare_phase7_rigid_world_results, minimize_rigid_world_request,
};
use liquidfun_test_protocol::{
    FieldComparison, FloatBits, HarnessLimits, Phase6PolicyProfile, Phase7PolicyProfile,
    RigidWorldRequestRecord, RigidWorldResultRecord, decode_rigid_world_request_jsonl,
    decode_rigid_world_result_jsonl,
};
use serde_json::{Value, json};

const PHASE6_POLICY: &str = include_str!("../../../../protocol/tolerances/phase6-v1.toml");
const PHASE7_POLICY: &str = include_str!("../../../../protocol/tolerances/phase7-v1.toml");

fn profiles() -> (Phase6PolicyProfile, Phase7PolicyProfile) {
    (
        Phase6PolicyProfile::parse_toml(PHASE6_POLICY)
            .expect("checked-in Phase 6 policy should validate"),
        Phase7PolicyProfile::parse_toml(PHASE7_POLICY)
            .expect("checked-in Phase 7 policy should validate"),
    )
}

fn request(profile: &Phase7PolicyProfile) -> RigidWorldRequestRecord {
    crate::support::phase7_request_with_profile(Some(profile.profile_sha256()))
}

fn decode_result(value: &Value) -> RigidWorldResultRecord {
    let mut bytes = serde_json::to_vec(value).expect("result mutation should encode");
    bytes.push(b'\n');
    decode_rigid_world_result_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("bounded semantic result mutation should decode")
}

fn request_with_ray_rules(
    profile: &Phase7PolicyProfile,
    directive_rules: Value,
) -> RigidWorldRequestRecord {
    let mut value = serde_json::to_value(request(profile)).expect("request should serialize");
    let ray_action = value["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("timeline actions should be an array")
        .iter_mut()
        .find(|record| record["action_id"] == "phase7-action-20")
        .expect("Phase 7 ray action should exist");
    ray_action["action"]["directive_rules"] = directive_rules;
    let mut bytes = serde_json::to_vec(&value).expect("request mutation should encode");
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("bounded ray request mutation should decode")
}

fn request_with_query_rules(
    profile: &Phase7PolicyProfile,
    directive_rules: Value,
) -> RigidWorldRequestRecord {
    let mut value = serde_json::to_value(request(profile)).expect("request should serialize");
    let query_action = value["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("timeline actions should be an array")
        .iter_mut()
        .find(|record| record["action_id"] == "phase7-action-19")
        .expect("Phase 7 query action should exist");
    query_action["action"]["directive_rules"] = directive_rules;
    let mut bytes = serde_json::to_vec(&value).expect("request mutation should encode");
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("bounded query request mutation should decode")
}

fn request_with_out_of_ray_clip(profile: &Phase7PolicyProfile) -> RigidWorldRequestRecord {
    let request = request_with_ray_rules(
        profile,
        json!([{
            "target": { "fixture_id": "nc-dynamic-fixture", "child_index": 0 },
            "directive": { "kind": "clip", "fraction_bits": 0.5_f32.to_bits() }
        }]),
    );
    let mut value = serde_json::to_value(request).expect("request should serialize");
    let ray_action = value["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("timeline actions should be an array")
        .iter_mut()
        .find(|record| record["action_id"] == "phase7-action-20")
        .expect("Phase 7 ray action should exist");
    ray_action["action"]["end"]["x_bits"] = json!(5.0_f32.to_bits());
    let mut bytes = serde_json::to_vec(&value).expect("request mutation should encode");
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("bounded ray request mutation should decode")
}

fn arbitrary_clip_order_results(
    profile: &Phase7PolicyProfile,
) -> (
    RigidWorldRequestRecord,
    RigidWorldResultRecord,
    RigidWorldResultRecord,
) {
    let request = request_with_ray_rules(
        profile,
        json!([{
            "target": { "fixture_id": "nc-dynamic-fixture", "child_index": 0 },
            "directive": { "kind": "clip", "fraction_bits": 0.1_f32.to_bits() }
        }]),
    );
    let baseline = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound arbitrary clip request should execute");
    let mut native_value = serde_json::to_value(&baseline).expect("result should serialize");
    let ray = observation_mut(phase7_observations(&mut native_value), "ray_cast");
    ray["observation"]["completion"] = json!("exhausted");
    ray["observation"]["final_max_fraction_bits"] = json!(0.1_f32.to_bits());
    ray["observation"]["hits"] = json!([
        {
            "fixture_id": "nc-dynamic-fixture",
            "child_index": 0,
            "point": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "normal": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() },
            "fraction_bits": 0.75_f32.to_bits()
        },
        {
            "fixture_id": "nc-static-fixture",
            "child_index": 0,
            "point": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() },
            "normal": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "fraction_bits": 0.05_f32.to_bits()
        }
    ]);
    let native = decode_result(&native_value);
    let mut oracle_value = native_value;
    observation_mut(phase7_observations(&mut oracle_value), "ray_cast")["observation"]["hits"]
        .as_array_mut()
        .expect("ray hits should be an array")
        .reverse();
    let oracle = decode_result(&oracle_value);
    (request, native, oracle)
}

fn duplicate_ray_results(
    profile: &Phase7PolicyProfile,
    expected_fractions: &[u32],
    actual_fractions: &[u32],
) -> (
    RigidWorldRequestRecord,
    RigidWorldResultRecord,
    RigidWorldResultRecord,
) {
    let request = request_with_ray_rules(profile, json!([]));
    let baseline = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound exhaustive ray request should execute");
    let mut expected_value = serde_json::to_value(&baseline).expect("result should serialize");
    let ray = observation_mut(phase7_observations(&mut expected_value), "ray_cast");
    ray["observation"]["completion"] = json!("exhausted");
    ray["observation"]["final_max_fraction_bits"] = json!(1.0_f32.to_bits());
    ray["observation"]["hits"] = Value::Array(
        expected_fractions
            .iter()
            .copied()
            .map(duplicate_ray_hit)
            .collect(),
    );
    let expected = decode_result(&expected_value);
    let mut actual_value = expected_value;
    observation_mut(phase7_observations(&mut actual_value), "ray_cast")["observation"]["hits"] =
        Value::Array(
            actual_fractions
                .iter()
                .copied()
                .map(duplicate_ray_hit)
                .collect(),
        );
    let actual = decode_result(&actual_value);
    (request, expected, actual)
}

fn duplicate_ray_hit(fraction_bits: u32) -> Value {
    json!({
        "fixture_id": "nc-static-fixture",
        "child_index": 0,
        "point": { "x_bits": 0.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
        "normal": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
        "fraction_bits": fraction_bits
    })
}

fn request_with_split_phase7_checkpoints(profile: &Phase7PolicyProfile) -> RigidWorldRequestRecord {
    let mut value = serde_json::to_value(request(profile)).expect("request should serialize");
    let checkpoints = value["scenario"]["timelines"][0]["checkpoints"]
        .as_array_mut()
        .expect("non-colliding checkpoints should be an array");
    checkpoints.insert(
        6,
        json!({
            "checkpoint_id": "phase7-step-checkpoint",
            "after_action_id": "phase7-action-18",
            "phase": "phase7-adapter",
            "counts": {
                "bodies": 3,
                "fixtures": 3,
                "contacts": 0,
                "manifold_points": 0,
                "events": 0,
                "destructions": 0
            },
            "transitions": []
        }),
    );
    let mut bytes = serde_json::to_vec(&value).expect("request mutation should encode");
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("split Phase 7 checkpoint request should decode")
}

fn phase7_observations(value: &mut Value) -> &mut Vec<Value> {
    value["timelines"][0]["checkpoints"][6]["observations"]
        .as_array_mut()
        .expect("Phase 7 checkpoint should contain observations")
}

fn observation_mut<'a>(observations: &'a mut [Value], kind: &str) -> &'a mut Value {
    observations
        .iter_mut()
        .find(|observation| observation["kind"] == kind)
        .expect("requested Phase 7 observation should exist")
}

fn checkpoint_observations_mut<'a>(
    value: &'a mut Value,
    checkpoint_id: &str,
) -> &'a mut Vec<Value> {
    value["timelines"][0]["checkpoints"]
        .as_array_mut()
        .expect("non-colliding checkpoints should be an array")
        .iter_mut()
        .find(|checkpoint| checkpoint["checkpoint_id"] == checkpoint_id)
        .expect("requested checkpoint should exist")["observations"]
        .as_array_mut()
        .expect("checkpoint observations should be an array")
}

#[test]
fn rigid_comparator_treats_queries_as_multiplicity_preserving_multisets() {
    // Arrange
    let (phase6, phase7) = profiles();
    let request = request_with_query_rules(&phase7, json!([]));
    let baseline = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound Phase 7 request should execute");
    let mut native_value = serde_json::to_value(&baseline).expect("result should serialize");
    let query = observation_mut(phase7_observations(&mut native_value), "query");
    let occurrences = query["observation"]["occurrences"]
        .as_array_mut()
        .expect("query occurrences should be an array");
    let duplicate = occurrences[0].clone();
    occurrences.push(duplicate);
    let native = decode_result(&native_value);
    let mut oracle_value = native_value.clone();
    oracle_value["timelines"][0]["checkpoints"][6]["observations"]
        .as_array_mut()
        .expect("observations should be an array")
        .iter_mut()
        .find(|observation| observation["kind"] == "query")
        .expect("query should exist")["observation"]["occurrences"]
        .as_array_mut()
        .expect("query occurrences should be an array")
        .reverse();
    let oracle = decode_result(&oracle_value);

    // Act
    let reordered =
        compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
            .expect("registered Phase 7 fields should compare");
    let occurrences = observation_mut(phase7_observations(&mut oracle_value), "query")
        ["observation"]["occurrences"]
        .as_array_mut()
        .expect("query occurrences should be an array");
    occurrences.pop();
    let missing_duplicate = compare_phase7_rigid_world_results(
        &request,
        &native,
        &decode_result(&oracle_value),
        &phase6,
        &phase7,
    )
    .expect("registered Phase 7 fields should compare");

    // Assert
    assert_eq!(reordered, RigidComparisonOutcome::Match);
    let RigidComparisonOutcome::PhysicsMismatch(report) = missing_duplicate else {
        panic!("removing one duplicate occurrence must mismatch");
    };
    assert_eq!(report.kind(), RigidMismatchKind::Order);
    assert_eq!(
        report.semantic_path(),
        "rigid_world.phase7.query.occurrences.identity"
    );
}

#[test]
fn rigid_comparator_reports_action_stage_values_policy_and_completion_context() {
    // Arrange
    let (phase6, phase7) = profiles();
    let request = request(&phase7);
    let native = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound Phase 7 request should execute");
    let mut oracle_value = serde_json::to_value(&native).expect("result should serialize");
    let step = observation_mut(phase7_observations(&mut oracle_value), "step");
    step["outcome"]["completion"] = json!("continuous_pending");
    let oracle = decode_result(&oracle_value);

    // Act
    let outcome = compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
        .expect("registered Phase 7 fields should compare");

    // Assert
    let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
        panic!("completion mutation must mismatch");
    };
    assert_eq!(report.action_id(), "phase7-action-18");
    assert_eq!(report.stage(), "phase7-adapter");
    assert_eq!(report.maybe_entity(), None);
    assert_eq!(report.semantic_path(), "rigid_world.phase7.step.completion");
    assert_eq!(report.expected(), "Complete");
    assert_eq!(report.actual(), "ContinuousPending");
    assert_eq!(report.policy().comparison(), FieldComparison::ExactDiscrete);
    assert!(report.maybe_completion_context().is_some());
}

#[test]
fn rigid_comparator_compares_equal_fraction_ray_hits_as_multisets() {
    // Arrange
    let (phase6, phase7) = profiles();
    let request = request(&phase7);
    let baseline = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound Phase 7 request should execute");
    let mut native_value = serde_json::to_value(&baseline).expect("result should serialize");
    let ray = observation_mut(phase7_observations(&mut native_value), "ray_cast");
    ray["observation"]["hits"] = json!([
        {
            "fixture_id": "nc-static-fixture",
            "child_index": 0,
            "point": { "x_bits": 0.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "normal": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "fraction_bits": 0.5_f32.to_bits()
        },
        {
            "fixture_id": "nc-dynamic-fixture",
            "child_index": 0,
            "point": { "x_bits": 0.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "normal": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "fraction_bits": 0.5_f32.to_bits()
        }
    ]);
    let native = decode_result(&native_value);
    let mut oracle_value = native_value;
    observation_mut(phase7_observations(&mut oracle_value), "ray_cast")["observation"]["hits"]
        .as_array_mut()
        .expect("ray hits should be an array")
        .reverse();
    let oracle = decode_result(&oracle_value);

    // Act
    let outcome = compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
        .expect("registered Phase 7 fields should compare");

    // Assert
    assert_eq!(outcome, RigidComparisonOutcome::Match);
}

#[test]
fn rigid_comparator_reassigns_adversarial_duplicate_hits_in_both_actual_orders() {
    // Arrange
    let (phase6, phase7) = profiles();
    let base = 0.5_f32.to_bits();
    let expected_fractions = [base, base - 4];
    let actual_orders = [[base - 2, base + 4], [base + 4, base - 2]];

    // Act and Assert
    for actual_fractions in actual_orders {
        let (request, expected, actual) =
            duplicate_ray_results(&phase7, &expected_fractions, &actual_fractions);
        let outcome =
            compare_phase7_rigid_world_results(&request, &expected, &actual, &phase6, &phase7)
                .expect("registered Phase 7 fields should compare");
        assert_eq!(outcome, RigidComparisonOutcome::Match);
    }
}

#[test]
fn rigid_comparator_reports_stable_fraction_when_no_perfect_matching_exists() {
    // Arrange
    let (phase6, phase7) = profiles();
    let base = 0.5_f32.to_bits();
    let expected_fractions = [base, base - 4];
    let actual_orders = [[base - 2, base + 5], [base + 5, base - 2]];
    let mut reports = Vec::new();

    // Act
    for actual_fractions in actual_orders {
        let (request, expected, actual) =
            duplicate_ray_results(&phase7, &expected_fractions, &actual_fractions);
        let outcome =
            compare_phase7_rigid_world_results(&request, &expected, &actual, &phase6, &phase7)
                .expect("registered Phase 7 fields should compare");
        let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
            panic!("a duplicate-hit group without a perfect matching must mismatch");
        };
        reports.push((
            report.signature().clone(),
            report.maybe_expected_bits(),
            report.maybe_actual_bits(),
        ));
    }

    // Assert
    assert_eq!(reports[0], reports[1]);
    assert_eq!(
        reports[0].0.semantic_path(),
        "rigid_world.phase7.ray.fraction"
    );
    assert_ne!(reports[0].1, reports[0].2);
    assert_eq!(reports[0].1.map(FloatBits::bits), Some(base));
    assert_eq!(reports[0].2.map(FloatBits::bits), Some(base + 5));
}

#[test]
fn rigid_comparator_preserves_duplicate_ray_hit_multiplicity() {
    // Arrange
    let (phase6, phase7) = profiles();
    let fraction = 0.5_f32.to_bits();
    let (request, expected, actual) =
        duplicate_ray_results(&phase7, &[fraction, fraction], &[fraction]);

    // Act
    let outcome =
        compare_phase7_rigid_world_results(&request, &expected, &actual, &phase6, &phase7)
            .expect("registered Phase 7 fields should compare");

    // Assert
    let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
        panic!("removing one duplicate ray hit must mismatch");
    };
    assert_eq!(report.kind(), RigidMismatchKind::Order);
    assert_eq!(
        report.semantic_path(),
        "rigid_world.phase7.ray.hit.identity"
    );
}

#[test]
fn rigid_comparator_ignores_reversed_pre_clip_history_above_the_final_interval() {
    // Arrange
    let (phase6, phase7) = profiles();
    let (request, native, oracle) = arbitrary_clip_order_results(&phase7);

    // Act
    let outcome = compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
        .expect("valid arbitrary clip histories should compare");

    // Assert
    assert_eq!(outcome, RigidComparisonOutcome::Match);
}

#[test]
fn rigid_comparator_reports_mismatch_inside_the_final_interval() {
    // Arrange
    let (phase6, phase7) = profiles();
    let (request, native, oracle) = arbitrary_clip_order_results(&phase7);
    let mut oracle_value = serde_json::to_value(&oracle).expect("result should serialize");
    let ray = observation_mut(phase7_observations(&mut oracle_value), "ray_cast");
    let hit = ray["observation"]["hits"]
        .as_array_mut()
        .expect("ray hits should be an array")
        .iter_mut()
        .find(|hit| hit["fixture_id"] == "nc-static-fixture")
        .expect("inside-interval hit should exist");
    hit["point"]["x_bits"] = json!((-0.5_f32).to_bits());
    let oracle = decode_result(&oracle_value);

    // Act
    let outcome = compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
        .expect("valid arbitrary clip histories should compare");

    // Assert
    let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
        panic!("inside-interval numeric divergence must mismatch");
    };
    assert_eq!(report.semantic_path(), "rigid_world.phase7.ray.point.x");
}

#[test]
fn rigid_comparator_treats_exhaustive_ray_hits_as_record_multisets() {
    // Arrange
    let (phase6, phase7) = profiles();
    let request = request_with_ray_rules(&phase7, json!([]));
    let baseline = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound exhaustive ray request should execute");
    let mut native_value = serde_json::to_value(&baseline).expect("result should serialize");
    let ray = observation_mut(phase7_observations(&mut native_value), "ray_cast");
    ray["observation"]["completion"] = json!("exhausted");
    ray["observation"]["hits"] = json!([
        {
            "fixture_id": "nc-static-fixture",
            "child_index": 0,
            "point": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() },
            "normal": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "fraction_bits": 0.25_f32.to_bits()
        },
        {
            "fixture_id": "nc-dynamic-fixture",
            "child_index": 0,
            "point": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "normal": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() },
            "fraction_bits": 0.75_f32.to_bits()
        }
    ]);
    let native = decode_result(&native_value);
    let mut oracle_value = native_value;
    observation_mut(phase7_observations(&mut oracle_value), "ray_cast")["observation"]["hits"]
        .as_array_mut()
        .expect("ray hits should be an array")
        .reverse();
    let oracle = decode_result(&oracle_value);

    // Act
    let outcome = compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
        .expect("registered Phase 7 fields should compare");

    // Assert
    assert_eq!(outcome, RigidComparisonOutcome::Match);
}

#[test]
fn rigid_comparator_uses_exhaustive_semantics_when_declared_clip_is_not_applied() {
    // Arrange
    let (phase6, phase7) = profiles();
    let request = request_with_out_of_ray_clip(&phase7);
    let baseline = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound out-of-ray clip request should execute");
    let mut native_value = serde_json::to_value(&baseline).expect("result should serialize");
    let ray = observation_mut(phase7_observations(&mut native_value), "ray_cast");
    assert_eq!(
        ray["observation"]["final_max_fraction_bits"],
        json!(1.0_f32.to_bits())
    );
    ray["observation"]["hits"] = json!([
        {
            "fixture_id": "nc-static-fixture",
            "child_index": 0,
            "point": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() },
            "normal": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "fraction_bits": 0.25_f32.to_bits()
        },
        {
            "fixture_id": "nc-kinematic-fixture",
            "child_index": 0,
            "point": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "normal": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() },
            "fraction_bits": 0.75_f32.to_bits()
        }
    ]);
    let native = decode_result(&native_value);
    let mut oracle_value = native_value;
    observation_mut(phase7_observations(&mut oracle_value), "ray_cast")["observation"]["hits"][1]
        ["point"]["x_bits"] = json!(2.0_f32.to_bits());
    let oracle = decode_result(&oracle_value);

    // Act
    let outcome = compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
        .expect("registered Phase 7 fields should compare");

    // Assert
    let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
        panic!("a nonminimum hit mismatch must remain visible when clipping was not applied");
    };
    assert_eq!(report.semantic_path(), "rigid_world.phase7.ray.point.x");
}

#[test]
fn rigid_comparator_uses_exhaustive_semantics_for_reached_noop_clip() {
    // Arrange
    let (phase6, phase7) = profiles();
    let request = request_with_ray_rules(
        &phase7,
        json!([{
            "target": { "fixture_id": "nc-dynamic-fixture", "child_index": 0 },
            "directive": { "kind": "clip", "fraction_bits": 1.0_f32.to_bits() }
        }]),
    );
    let baseline = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound no-op clip request should execute");
    let mut native_value = serde_json::to_value(&baseline).expect("result should serialize");
    let ray = observation_mut(phase7_observations(&mut native_value), "ray_cast");
    assert_eq!(
        ray["observation"]["final_max_fraction_bits"],
        json!(1.0_f32.to_bits())
    );
    ray["observation"]["hits"] = json!([
        {
            "fixture_id": "nc-static-fixture",
            "child_index": 0,
            "point": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() },
            "normal": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "fraction_bits": 0.25_f32.to_bits()
        },
        {
            "fixture_id": "nc-dynamic-fixture",
            "child_index": 0,
            "point": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "normal": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() },
            "fraction_bits": 0.75_f32.to_bits()
        }
    ]);
    let native = decode_result(&native_value);
    let mut oracle_value = native_value;
    observation_mut(phase7_observations(&mut oracle_value), "ray_cast")["observation"]["hits"][1]
        ["point"]["x_bits"] = json!(2.0_f32.to_bits());
    let oracle = decode_result(&oracle_value);

    // Act
    let outcome = compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
        .expect("registered Phase 7 fields should compare");

    // Assert
    let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
        panic!("a reached no-op clip must not hide a nonminimum mismatch");
    };
    assert_eq!(report.semantic_path(), "rigid_world.phase7.ray.point.x");
}

#[test]
fn rigid_comparator_reports_final_ray_interval_disagreement_first() {
    // Arrange
    let (phase6, phase7) = profiles();
    let request = request(&phase7);
    let baseline = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound strict clip request should execute");
    let mut native_value = serde_json::to_value(&baseline).expect("result should serialize");
    let ray = observation_mut(phase7_observations(&mut native_value), "ray_cast");
    ray["observation"]["hits"] = json!([
        {
            "fixture_id": "nc-static-fixture",
            "child_index": 0,
            "point": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() },
            "normal": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "fraction_bits": 0.25_f32.to_bits()
        },
        {
            "fixture_id": "nc-dynamic-fixture",
            "child_index": 0,
            "point": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "normal": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() },
            "fraction_bits": 0.75_f32.to_bits()
        }
    ]);
    let native = decode_result(&native_value);
    let mut oracle_value = native_value;
    let oracle_ray = observation_mut(phase7_observations(&mut oracle_value), "ray_cast");
    oracle_ray["observation"]["hits"]
        .as_array_mut()
        .expect("ray hits should be an array")
        .pop();
    oracle_ray["observation"]["final_max_fraction_bits"] = json!(1.0_f32.to_bits());
    let oracle = decode_result(&oracle_value);

    // Act
    let outcome = compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
        .expect("registered Phase 7 fields should compare");

    // Assert
    let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
        panic!("different validated final intervals must mismatch");
    };
    assert_eq!(
        report.semantic_path(),
        "rigid_world.phase7.ray.final_max_fraction"
    );
}

#[test]
fn rigid_comparator_termination_observes_only_status_and_hit_count() {
    // Arrange
    let (phase6, phase7) = profiles();
    let request = request_with_ray_rules(
        &phase7,
        json!([{
            "target": { "fixture_id": "nc-dynamic-fixture", "child_index": 0 },
            "directive": { "kind": "terminate" }
        }]),
    );
    let baseline = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound terminating ray request should execute");
    let mut native_value = serde_json::to_value(&baseline).expect("result should serialize");
    let ray = observation_mut(phase7_observations(&mut native_value), "ray_cast");
    ray["observation"]["completion"] = json!("terminated");
    ray["observation"]["hits"] = json!([{
        "fixture_id": "nc-dynamic-fixture",
        "child_index": 0,
        "point": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() },
        "normal": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
        "fraction_bits": 0.25_f32.to_bits()
    }]);
    let native = decode_result(&native_value);
    let mut oracle_value = native_value;
    let oracle_hit = &mut observation_mut(phase7_observations(&mut oracle_value), "ray_cast")["observation"]
        ["hits"][0];
    oracle_hit["point"]["x_bits"] = json!(100.0_f32.to_bits());
    let oracle = decode_result(&oracle_value);

    // Act
    let outcome = compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
        .expect("registered Phase 7 fields should compare");

    // Assert
    assert_eq!(outcome, RigidComparisonOutcome::Match);
}

#[test]
fn rigid_comparator_does_not_reapply_inherited_exact_bits_after_phase7_numeric_match() {
    // Arrange
    let (phase6, phase7) = profiles();
    let request = request(&phase7);
    let native = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound Phase 7 request should execute");
    let mut oracle_value = serde_json::to_value(&native).expect("result should serialize");
    let position_bits = oracle_value["timelines"][3]["checkpoints"][0]["bodies"][1]
        ["transform"]["position"]["x_bits"]
        .as_u64()
        .expect("island position should use exact float bits");
    oracle_value["timelines"][3]["checkpoints"][0]["bodies"][1]["transform"]["position"]["x_bits"] =
        json!(position_bits + 1);
    let oracle = decode_result(&oracle_value);

    // Act
    let outcome = compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
        .expect("registered Phase 7 fields should compare");

    // Assert
    assert_eq!(outcome, RigidComparisonOutcome::Match);
}

#[test]
fn rigid_minimization_preserves_divergent_action_setup_directives_budget_and_bits() {
    // Arrange
    let (phase6, phase7) = profiles();
    let request = request(&phase7);
    let native = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound Phase 7 request should execute");
    let mut oracle_value = serde_json::to_value(&native).expect("result should serialize");
    let ray = observation_mut(phase7_observations(&mut oracle_value), "ray_cast");
    for hit in ray["observation"]["hits"]
        .as_array_mut()
        .expect("ray hits should be an array")
    {
        hit["point"]["x_bits"] = json!(100.0_f32.to_bits());
    }
    let oracle = decode_result(&oracle_value);
    let RigidComparisonOutcome::PhysicsMismatch(report) =
        compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
            .expect("registered Phase 7 fields should compare")
    else {
        panic!("ray-point mutation must mismatch");
    };
    assert_eq!(report.action_id(), "phase7-action-20");
    assert!(
        report
            .maybe_entity()
            .is_some_and(|entity| entity.ends_with(":0"))
    );
    assert!(report.maybe_expected_bits().is_some());
    assert!(report.maybe_actual_bits().is_some());
    assert!(report.maybe_expected_decimal().is_some());
    assert!(report.maybe_actual_decimal().is_some());
    assert!(report.maybe_completion_context().is_some());
    let target = report.signature().clone();
    let original = serde_json::to_value(&request).expect("request should serialize");

    // Act
    let result = minimize_rigid_world_request(
        &request,
        &target,
        MinimizationBudget::new(256, Duration::from_secs(1)),
        |_candidate| RigidEvaluation::new(Some(target.clone()), Duration::from_millis(1)),
    )
    .expect("Phase 7 minimization should retain its exact failure class");

    // Assert
    let minimized = serde_json::to_value(result.request()).expect("request should serialize");
    for action_id in ["nc-create-dynamic", "phase7-action-18", "phase7-action-20"] {
        assert_eq!(
            action(&minimized, action_id),
            action(&original, action_id),
            "required setup and divergent operations must remain bit-identical"
        );
    }
    assert_eq!(target.action_id(), "phase7-action-20");
    assert_eq!(target.kind(), RigidMismatchKind::Numeric);
}

#[test]
fn second_checkpoint_evidence_and_minimization_use_its_local_action_window() {
    // Arrange
    let (phase6, phase7) = profiles();
    let request = request_with_split_phase7_checkpoints(&phase7);
    let native = NativeRigidWorldExecutor::execute(&request)
        .expect("split Phase 7 checkpoint request should execute");
    let mut oracle_value = serde_json::to_value(&native).expect("result should serialize");
    let ray = observation_mut(
        checkpoint_observations_mut(&mut oracle_value, "nc-fixtures-destroyed"),
        "ray_cast",
    );
    for hit in ray["observation"]["hits"]
        .as_array_mut()
        .expect("ray hits should be an array")
    {
        hit["point"]["x_bits"] = json!(100.0_f32.to_bits());
    }
    let oracle = decode_result(&oracle_value);
    let RigidComparisonOutcome::PhysicsMismatch(report) =
        compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
            .expect("registered Phase 7 fields should compare")
    else {
        panic!("second-checkpoint ray mutation must mismatch");
    };
    let target = report.signature().clone();
    let original = serde_json::to_value(&request).expect("request should serialize");

    // Act
    let result = minimize_rigid_world_request(
        &request,
        &target,
        MinimizationBudget::new(256, Duration::from_secs(1)),
        |_candidate| RigidEvaluation::new(Some(target.clone()), Duration::from_millis(1)),
    )
    .expect("minimization should preserve the second-checkpoint signature");

    // Assert
    assert_eq!(target.checkpoint_id(), "nc-fixtures-destroyed");
    assert_eq!(target.action_id(), "phase7-action-20");
    assert_eq!(report.stage(), "phase7-adapter");
    let minimized = serde_json::to_value(result.request()).expect("request should serialize");
    for action_id in [
        "nc-create-dynamic",
        "phase7-action-18",
        "phase7-action-19",
        "phase7-action-20",
    ] {
        assert_eq!(
            action(&minimized, action_id),
            action(&original, action_id),
            "the second-checkpoint protected prefix must remain bit-identical"
        );
    }
}

fn action<'a>(request: &'a Value, action_id: &str) -> &'a Value {
    request["scenario"]["timelines"][0]["actions"]
        .as_array()
        .expect("actions should be an array")
        .iter()
        .find(|record| record["action_id"] == action_id)
        .expect("required action should remain present")
}
