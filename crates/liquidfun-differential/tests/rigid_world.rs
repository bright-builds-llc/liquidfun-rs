//! Native Phase 6 rigid-world adapter integration tests.

#[path = "support/phase7_comparator.rs"]
mod phase7_comparator;
mod support;

use std::fs;
use std::process::Command;
use std::time::Duration;

use liquidfun_differential::{
    FailureBundleRequest, MinimizationBudget, NativeRigidWorldExecutor, OracleExecutable,
    OraclePreset, RigidComparisonFailure, RigidComparisonOutcome, RigidEngineSide, RigidEvaluation,
    RigidMismatchKind, compare_rigid_world_results, execute_rigid_world_process,
    minimize_rigid_world_request, persist_failure_bundle, validate_native_rigid_world_result,
};
use liquidfun_test_protocol::{
    HarnessLimits, Phase6PolicyProfile, RecordLimit, RigidWorldErrorKind, RigidWorldObservation,
    RigidWorldRequestRecord, RigidWorldResultRecord, RigidWorldWitnessFamily,
    decode_rigid_world_request_jsonl, decode_rigid_world_result_jsonl, encode_jsonl,
    rigid_world_checkpoint_live_identities, validate_rigid_world_result_against_request,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const REQUEST: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/rigid-world-request.jsonl");
const POLICY: &str = include_str!("../../../protocol/tolerances/phase6-v1.toml");
const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";

fn request() -> liquidfun_test_protocol::RigidWorldRequestRecord {
    decode_rigid_world_request_jsonl(REQUEST, &HarnessLimits::phase2_default_v1())
        .expect("checked-in rigid-world request should decode")
}

fn encode_value(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).expect("fixture mutation should encode");
    bytes.push(b'\n');
    bytes
}

fn profile() -> Phase6PolicyProfile {
    Phase6PolicyProfile::parse_toml(POLICY).expect("checked-in rigid policy should validate")
}

fn comparison_request() -> RigidWorldRequestRecord {
    let profile = profile();
    let mut value = serde_json::from_slice::<Value>(REQUEST).expect("fixture should be JSON");
    value["tolerance_profile_sha256"] = json!(profile.profile_sha256().as_str());
    decode_rigid_world_request_jsonl(&encode_value(&value), &HarnessLimits::phase2_default_v1())
        .expect("profile-bound rigid request should decode")
}

fn comparison_request_with_partial_body_checkpoint() -> RigidWorldRequestRecord {
    let profile = profile();
    let mut value = serde_json::from_slice::<Value>(REQUEST).expect("fixture should be JSON");
    value["tolerance_profile_sha256"] = json!(profile.profile_sha256().as_str());
    let checkpoints = value["scenario"]["timelines"][0]["checkpoints"]
        .as_array_mut()
        .expect("non-colliding checkpoints should be an array");
    let final_checkpoint = checkpoints
        .last_mut()
        .expect("non-colliding timeline should have a final checkpoint");
    final_checkpoint["counts"]["destructions"] = json!(2);
    let insert_at = checkpoints.len() - 1;
    checkpoints.insert(
        insert_at,
        json!({
            "checkpoint_id": "nc-first-body-destroyed",
            "after_action_id": "nc-destroy-body-static",
            "phase": "destroy-bodies",
            "counts": {
                "bodies": 2,
                "fixtures": 0,
                "contacts": 0,
                "manifold_points": 0,
                "events": 0,
                "destructions": 1
            },
            "transitions": []
        }),
    );
    decode_rigid_world_request_jsonl(&encode_value(&value), &HarnessLimits::phase2_default_v1())
        .expect("partial body-destruction checkpoint should decode")
}

fn comparison_request_with_cascade_query_checkpoint() -> RigidWorldRequestRecord {
    let profile = profile();
    let mut value = serde_json::from_slice::<Value>(REQUEST).expect("fixture should be JSON");
    value["tolerance_profile_sha256"] = json!(profile.profile_sha256().as_str());
    let timeline = value["scenario"]["timelines"]
        .as_array_mut()
        .expect("fixture timelines should be an array")
        .iter_mut()
        .find(|timeline| timeline["witness_family"] == "world_query_and_ray_cast")
        .expect("query timeline should exist");
    let actions = timeline["actions"]
        .as_array_mut()
        .expect("query actions should be an array");
    let destroy_index = actions
        .iter()
        .position(|action| action["action_id"] == "query-12")
        .expect("right-body destruction should exist");
    actions.insert(
        destroy_index + 1,
        json!({
            "action_id": "query-12-cascade-query",
            "phase": "teardown-query",
            "action": {
                "kind": "query_aabb",
                "aabb": {
                    "lower": { "x_bits": (-4.0_f32).to_bits(), "y_bits": (-2.0_f32).to_bits() },
                    "upper": { "x_bits": 4.0_f32.to_bits(), "y_bits": 2.0_f32.to_bits() }
                },
                "directive_rules": []
            }
        }),
    );
    timeline["checkpoints"]
        .as_array_mut()
        .expect("query checkpoints should be an array")
        .push(json!({
            "checkpoint_id": "query-right-cascade-query",
            "after_action_id": "query-12-cascade-query",
            "phase": "teardown-query",
            "counts": {
                "bodies": 2,
                "fixtures": 2,
                "contacts": 0,
                "manifold_points": 0,
                "events": 0,
                "destructions": 1
            },
            "transitions": []
        }));
    decode_rigid_world_request_jsonl(&encode_value(&value), &HarnessLimits::phase2_default_v1())
        .expect("query checkpoint after fixture cascade should decode")
}

fn request_with_expanding_ray_clips() -> RigidWorldRequestRecord {
    let mut value = serde_json::from_slice::<Value>(REQUEST).expect("fixture should be JSON");
    let query_timeline = value["scenario"]["timelines"]
        .as_array_mut()
        .expect("fixture timelines should be an array")
        .iter_mut()
        .find(|timeline| timeline["witness_family"] == "world_query_and_ray_cast")
        .expect("query timeline should exist");
    let ray_action = query_timeline["actions"]
        .as_array_mut()
        .expect("query actions should be an array")
        .iter_mut()
        .find(|action| action["action_id"] == "query-10")
        .expect("clip action should exist");
    ray_action["action"]["directive_rules"] = json!([
        {
            "target": { "fixture_id": "query-right-fixture", "child_index": 0 },
            "directive": { "kind": "clip", "fraction_bits": 0.5_f32.to_bits() }
        },
        {
            "target": { "fixture_id": "query-center-fixture", "child_index": 0 },
            "directive": { "kind": "clip", "fraction_bits": 0.75_f32.to_bits() }
        }
    ]);
    decode_rigid_world_request_jsonl(&encode_value(&value), &HarnessLimits::phase2_default_v1())
        .expect("bounded expanding-clip request should decode")
}

fn assert_identity_rejected_on_each_side(
    request: &RigidWorldRequestRecord,
    complete: &RigidWorldResultRecord,
    mutated: &RigidWorldResultRecord,
    profile: &Phase6PolicyProfile,
    path: &str,
) {
    for side in [RigidEngineSide::Native, RigidEngineSide::Oracle] {
        let result = match side {
            RigidEngineSide::Native => {
                compare_rigid_world_results(request, mutated, complete, profile)
            }
            RigidEngineSide::Oracle => {
                compare_rigid_world_results(request, complete, mutated, profile)
            }
        };
        let Err(RigidComparisonFailure::Declaration(report)) = result else {
            panic!("{side:?} identity disagreement must fail declaration validation");
        };
        assert_eq!(report.engine_side(), side);
        assert_eq!(report.semantic_path(), path);
    }
}

fn assert_observation_rejected_on_each_side(
    request: &RigidWorldRequestRecord,
    complete: &RigidWorldResultRecord,
    mutated: &RigidWorldResultRecord,
    profile: &Phase6PolicyProfile,
) {
    for side in [RigidEngineSide::Native, RigidEngineSide::Oracle] {
        let result = match side {
            RigidEngineSide::Native => {
                compare_rigid_world_results(request, mutated, complete, profile)
            }
            RigidEngineSide::Oracle => {
                compare_rigid_world_results(request, complete, mutated, profile)
            }
        };
        assert!(
            matches!(result, Err(RigidComparisonFailure::Harness(_))),
            "{side:?} invalid observation must fail request-bound validation"
        );
    }
}

fn decode_result_value(value: &Value) -> RigidWorldResultRecord {
    decode_rigid_world_result_jsonl(&encode_value(value), &HarnessLimits::phase2_default_v1())
        .expect("mutated result should remain internally valid")
}

fn result_value(result: &RigidWorldResultRecord) -> Value {
    serde_json::to_value(result).expect("validated result should serialize")
}

#[test]
fn native_executes_all_locked_families_deterministically_and_resets() {
    // Arrange
    let request = request();

    // Act
    let first = NativeRigidWorldExecutor::execute(&request)
        .expect("validated rigid-world request should execute natively");
    let second = NativeRigidWorldExecutor::execute(&request)
        .expect("a fresh native execution should reset all world state");

    // Assert
    assert_eq!(first, second);
    assert_eq!(first.timelines().len(), RigidWorldWitnessFamily::ALL.len());
    assert_eq!(first.timelines()[0].checkpoints.len(), 8);
    assert_eq!(first.timelines()[1].checkpoints.len(), 10);
    validate_native_rigid_world_result(&request, &first)
        .expect("native result should agree with every declaration");
}

#[test]
fn native_contract_executes_the_exact_fixed_step_tuple() {
    // Arrange
    let request = request();

    // Act
    let result = NativeRigidWorldExecutor::execute(&request)
        .expect("the validated fixed tuple should execute natively");

    // Assert
    assert_eq!(result.timelines().len(), RigidWorldWitnessFamily::ALL.len());
}

#[test]
fn native_executes_closed_phase7_actions_and_emits_semantic_observations() {
    // Arrange
    let request = support::phase7_request();

    // Act
    let result = NativeRigidWorldExecutor::execute(&request)
        .expect("validated Phase 7 actions should execute through the native adapter");

    // Assert
    let observations = &result.timelines()[0].checkpoints[6].observations;
    assert!(
        observations
            .iter()
            .any(|observation| matches!(observation, RigidWorldObservation::Step { .. }))
    );
    assert!(
        observations
            .iter()
            .any(|observation| matches!(observation, RigidWorldObservation::Query { .. }))
    );
    assert!(
        observations
            .iter()
            .any(|observation| matches!(observation, RigidWorldObservation::RayCast { .. }))
    );
    assert!(
        observations
            .iter()
            .any(|observation| matches!(observation, RigidWorldObservation::OriginShift { .. }))
    );
}

#[test]
fn oracle_rejects_step_bearing_phase8_until_plan_08_21() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(executable) = OracleExecutable::resolve(&root, OraclePreset::Debug) else {
        return;
    };
    let request = support::phase7_request();

    // Act
    let error = execute_rigid_world_process(&executable, &request, REVISION)
        .expect_err("the pre-08-21 C++ adapter must reject step-bearing Phase 8 actions");

    // Assert
    assert_eq!(
        error.retained_stderr(),
        b"liquidfun-reference: unsupported Phase 8 execution action\n"
    );
    assert!(error.child_killed());
    assert!(error.child_reaped());
}

#[test]
fn expanding_ray_clips_fail_closed_in_native_oracle_and_result_validation() {
    // Arrange
    let baseline_request = request();
    let baseline = NativeRigidWorldExecutor::execute(&baseline_request)
        .expect("baseline rigid request should execute");
    let expanding_request = request_with_expanding_ray_clips();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let maybe_executable = OracleExecutable::resolve(&root, OraclePreset::Debug).ok();

    // Act
    let native = NativeRigidWorldExecutor::execute(&expanding_request);
    let validation = validate_rigid_world_result_against_request(&expanding_request, &baseline);
    let maybe_oracle = maybe_executable
        .map(|executable| execute_rigid_world_process(&executable, &expanding_request, REVISION));

    // Assert
    assert!(
        native.is_err(),
        "native adapter must reject interval expansion"
    );
    assert_eq!(
        validation
            .expect_err("result validation must reject interval expansion")
            .rigid_world_kind(),
        Some(RigidWorldErrorKind::ResultObservationMismatch)
    );
    if let Some(oracle) = maybe_oracle {
        assert!(
            oracle.is_err(),
            "oracle adapter must reject interval expansion"
        );
    }
}

#[test]
fn result_validation_rejects_inconsistent_final_ray_interval() {
    // Arrange
    let request = request();
    let baseline =
        NativeRigidWorldExecutor::execute(&request).expect("baseline rigid request should execute");
    let mut value = result_value(&baseline);
    let observations = value["timelines"][7]["checkpoints"][0]["observations"]
        .as_array_mut()
        .expect("query observations should be an array");
    let clipped_ray = observations
        .iter_mut()
        .find(|observation| {
            observation["kind"] == "ray_cast"
                && observation["observation"]["final_max_fraction_bits"] == json!(0.5_f32.to_bits())
        })
        .expect("strictly clipped ray observation should exist");
    clipped_ray["observation"]["final_max_fraction_bits"] = json!(1.0_f32.to_bits());
    let inconsistent = decode_result_value(&value);

    // Act
    let error = validate_rigid_world_result_against_request(&request, &inconsistent)
        .expect_err("recorded final interval must match callback replay");

    // Assert
    assert_eq!(
        error.rigid_world_kind(),
        Some(RigidWorldErrorKind::ResultObservationMismatch)
    );
}

#[test]
fn native_centered_inertia_zero_origin_branch_executes_without_mutation_failure() {
    // Arrange
    let mut value = serde_json::from_slice::<Value>(REQUEST).expect("fixture should be JSON");
    let actions = value["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("fixture actions should be an array");
    let custom_mass = actions
        .iter_mut()
        .find(|action| action["action_id"] == "nc-custom-mass")
        .expect("custom mass action should exist");
    custom_mass["action"]["mass_bits"] = json!(1.0_f32.to_bits());
    custom_mass["action"]["center"]["x_bits"] = json!(1.0_f32.to_bits());
    custom_mass["action"]["center"]["y_bits"] = json!(0.0_f32.to_bits());
    custom_mass["action"]["inertia_bits"] = json!(0.0_f32.to_bits());
    let request = decode_rigid_world_request_jsonl(
        &encode_value(&value),
        &HarnessLimits::phase2_default_v1(),
    )
    .expect("zero origin inertia should decode through the no-inertia branch");

    // Act
    let result = NativeRigidWorldExecutor::execute(&request);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn native_boundary_rejects_invalid_owner_and_unknown_identity() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut invalid_owner =
        serde_json::from_slice::<Value>(REQUEST).expect("fixture should be JSON");
    invalid_owner["scenario"]["timelines"][0]["fixtures"][0]["owner_body_id"] =
        json!("missing-body");
    let mut unknown_identity =
        serde_json::from_slice::<Value>(REQUEST).expect("fixture should be JSON");
    let actions = unknown_identity["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("fixture actions should be an array");
    let inspect_body = actions
        .iter_mut()
        .find(|action| action["action_id"] == "nc-inspect-body")
        .expect("inspect-body action should exist");
    inspect_body["action"]["body_id"] = json!("missing-body");

    // Act
    let owner_error = decode_rigid_world_request_jsonl(&encode_value(&invalid_owner), &limits)
        .expect_err("an invalid owner must fail before native effects");
    let identity_error =
        decode_rigid_world_request_jsonl(&encode_value(&unknown_identity), &limits)
            .expect_err("an unknown semantic identity must fail before native effects");

    // Assert
    assert_eq!(
        owner_error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidOwner)
    );
    assert_eq!(
        identity_error.rigid_world_kind(),
        Some(RigidWorldErrorKind::UnknownBody)
    );
}

#[test]
fn native_validation_rejects_declaration_disagreement() {
    // Arrange
    let request = request();
    let result =
        NativeRigidWorldExecutor::execute(&request).expect("baseline native result should execute");
    let limits = HarnessLimits::phase2_default_v1();
    let encoded = encode_jsonl(&result, &limits, RecordLimit::Output)
        .expect("baseline native result should encode");
    let mut value = serde_json::from_slice::<Value>(&encoded).expect("result should be JSON");
    value["timelines"][0]["checkpoints"][0]["counts"]["bodies"] = json!(2);
    value["timelines"][0]["checkpoints"][0]["bodies"]
        .as_array_mut()
        .expect("body snapshots should be an array")
        .pop();
    let changed = decode_rigid_world_result_jsonl(&encode_value(&value), &limits)
        .expect("internally consistent changed result should decode");

    // Act
    let error = validate_native_rigid_world_result(&request, &changed)
        .expect_err("changed declared counts must reject the native result");

    // Assert
    assert!(error.to_string().contains("declaration"));
}

#[test]
fn native_cli_dispatches_through_existing_binary() {
    // Arrange
    let request_path = std::env::temp_dir().join(format!(
        "liquidfun-rigid-world-{}-{}.jsonl",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    fs::write(&request_path, REQUEST).expect("temporary request should write");

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_liquidfun-differential"))
        .args(["native-rigid-world", "--request"])
        .arg(&request_path)
        .output()
        .expect("native rigid-world command should launch");
    let _ = fs::remove_file(request_path);

    // Assert
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: liquidfun_test_protocol::RigidWorldResultRecord =
        serde_json::from_slice(&output.stdout).expect("CLI stdout should be one result record");
    assert_eq!(result.timelines().len(), RigidWorldWitnessFamily::ALL.len());
}

#[test]
fn native_rigid_source_changes_build_identity() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest = fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("native-math-sources.txt"),
    )
    .expect("native source manifest should be readable");
    let sources = manifest
        .lines()
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    let required = [
        "crates/liquidfun-differential/src/rigid_world.rs",
        "crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs",
        "crates/liquidfun/src/rigid_differential.rs",
        "crates/liquidfun/src/world/contact_manager.rs",
        "crates/liquidfun/src/world/contact_solver.rs",
        "crates/liquidfun/src/world/step.rs",
    ];
    for path in required {
        assert!(sources.contains(&path), "missing identity source {path}");
    }

    // Act
    let digest = source_digest(&root, &sources, None);
    let adapter =
        liquidfun_differential::EmptyWorldAdapter::new("0123456789abcdef0123456789abcdef01234567")
            .expect("native identity should validate");

    // Assert
    assert_eq!(
        digest,
        adapter.build_identity().adapter_content_sha256().as_str()
    );
    for changed in required {
        assert_ne!(digest, source_digest(&root, &sources, Some(changed)));
    }
}

#[test]
fn comparison_validates_each_engine_declaration_before_cross_engine_fields() {
    // Arrange
    let request = comparison_request();
    let profile = profile();
    let native = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound request should execute natively");
    let mut oracle_value = result_value(&native);
    oracle_value["timelines"][0]["checkpoints"][0]["counts"]["bodies"] = json!(2);
    oracle_value["timelines"][0]["checkpoints"][0]["bodies"]
        .as_array_mut()
        .expect("body snapshots should be an array")
        .pop();
    let oracle = decode_result_value(&oracle_value);

    // Act
    let result = compare_rigid_world_results(&request, &native, &oracle, &profile);

    // Assert
    let Err(RigidComparisonFailure::Declaration(report)) = result else {
        panic!("declaration disagreement must precede cross-engine comparison");
    };
    assert_eq!(report.action_id(), "nc-create-dynamic-fixture");
    assert_eq!(report.checkpoint_id(), "nc-created");
    assert_eq!(report.semantic_path(), "rigid_world.checkpoint.counts");
}

#[test]
fn comparison_rejects_omitted_phase7_observations_on_each_engine_side() {
    // Arrange
    let request = comparison_request();
    let profile = profile();
    let complete = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound request should execute natively");
    let mut omitted_value = result_value(&complete);
    omitted_value["timelines"][2]["checkpoints"][0]["observations"] = json!([]);
    let omitted = decode_result_value(&omitted_value);

    // Act
    let native_error = compare_rigid_world_results(&request, &omitted, &complete, &profile);
    let oracle_error = compare_rigid_world_results(&request, &complete, &omitted, &profile);

    // Assert
    assert!(matches!(
        native_error,
        Err(RigidComparisonFailure::Harness(_))
    ));
    assert!(matches!(
        oracle_error,
        Err(RigidComparisonFailure::Harness(_))
    ));
}

#[test]
fn comparison_rejects_invalid_all_continue_query_termination_on_each_engine_side() {
    // Arrange
    let request = comparison_request();
    let profile = profile();
    let complete = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound request should execute natively");
    let mut mutated_value = result_value(&complete);
    let observations = mutated_value["timelines"][7]["checkpoints"][0]["observations"]
        .as_array_mut()
        .expect("query checkpoint should contain observations");
    let query = observations
        .iter_mut()
        .find(|observation| {
            observation["kind"] == "query"
                && observation["observation"]["completion"] == "exhausted"
        })
        .expect("all-continue query observation should exist");
    query["observation"]["completion"] = json!("terminated");
    let mutated = decode_result_value(&mutated_value);

    // Act and Assert
    assert_observation_rejected_on_each_side(&request, &complete, &mutated, &profile);
}

#[test]
fn comparison_rejects_query_occurrence_removed_by_body_cascade_on_each_engine_side() {
    // Arrange
    let request = comparison_request_with_cascade_query_checkpoint();
    let profile = profile();
    let complete = NativeRigidWorldExecutor::execute(&request)
        .expect("request with a post-cascade query should execute natively");
    let mut mutated_value = result_value(&complete);
    let occurrences = mutated_value["timelines"][7]["checkpoints"][1]["observations"][0]
        ["observation"]["occurrences"]
        .as_array_mut()
        .expect("post-cascade query should contain occurrences");
    occurrences.push(json!({
        "fixture_id": "query-right-fixture",
        "child_index": 0
    }));
    let mutated = decode_result_value(&mutated_value);

    // Act and Assert
    assert_observation_rejected_on_each_side(&request, &complete, &mutated, &profile);
}

#[test]
fn comparison_rejects_unknown_ray_hit_identity_on_each_engine_side() {
    // Arrange
    let request = comparison_request();
    let profile = profile();
    let complete = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound request should execute natively");
    let mut mutated_value = result_value(&complete);
    let observations = mutated_value["timelines"][7]["checkpoints"][0]["observations"]
        .as_array_mut()
        .expect("query checkpoint should contain observations");
    let ray = observations
        .iter_mut()
        .find(|observation| {
            observation["kind"] == "ray_cast"
                && observation["observation"]["hits"]
                    .as_array()
                    .is_some_and(|hits| !hits.is_empty())
        })
        .expect("ray observation with a hit should exist");
    ray["observation"]["hits"][0]["fixture_id"] = json!("unknown-ray-fixture");
    let mutated = decode_result_value(&mutated_value);

    // Act and Assert
    assert_observation_rejected_on_each_side(&request, &complete, &mutated, &profile);
}

#[test]
fn comparison_rejects_every_non_finite_ray_hit_coordinate_on_each_engine_side() {
    // Arrange
    let request = comparison_request();
    let profile = profile();
    let complete = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound request should execute natively");
    let complete_value = result_value(&complete);

    // Act and Assert
    for vector in ["point", "normal"] {
        for coordinate in ["x_bits", "y_bits"] {
            for invalid_bits in [
                f32::NAN.to_bits(),
                f32::INFINITY.to_bits(),
                f32::NEG_INFINITY.to_bits(),
            ] {
                let mut mutated_value = complete_value.clone();
                let observations = mutated_value["timelines"][7]["checkpoints"][0]["observations"]
                    .as_array_mut()
                    .expect("query checkpoint should contain observations");
                let ray = observations
                    .iter_mut()
                    .find(|observation| {
                        observation["kind"] == "ray_cast"
                            && observation["observation"]["hits"]
                                .as_array()
                                .is_some_and(|hits| !hits.is_empty())
                    })
                    .expect("ray observation with a hit should exist");
                ray["observation"]["hits"][0][vector][coordinate] = json!(invalid_bits);
                let mutated = decode_result_value(&mutated_value);
                assert_observation_rejected_on_each_side(&request, &complete, &mutated, &profile);
            }
        }
    }
}

#[test]
fn comparison_rejects_invalid_child_hit_before_valid_ray_termination_on_each_engine_side() {
    // Arrange
    let request = comparison_request();
    let profile = profile();
    let complete = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound request should execute natively");
    let mut mutated_value = result_value(&complete);
    let observations = mutated_value["timelines"][7]["checkpoints"][0]["observations"]
        .as_array_mut()
        .expect("query checkpoint should contain observations");
    let ray = observations
        .iter_mut()
        .find(|observation| {
            observation["kind"] == "ray_cast"
                && observation["observation"]["completion"] == "terminated"
        })
        .expect("terminated ray observation should exist");
    let hits = ray["observation"]["hits"]
        .as_array_mut()
        .expect("terminated ray should contain hits");
    let mut fabricated = hits
        .first()
        .expect("terminated ray should contain its terminating hit")
        .clone();
    fabricated["child_index"] = json!(1);
    hits.insert(0, fabricated);
    let mutated = decode_result_value(&mutated_value);

    // Act and Assert
    assert_observation_rejected_on_each_side(&request, &complete, &mutated, &profile);
}

#[test]
fn comparison_rejects_same_count_stale_body_identities_on_each_engine_side() {
    // Arrange
    let request = comparison_request_with_partial_body_checkpoint();
    let profile = profile();
    let complete = NativeRigidWorldExecutor::execute(&request)
        .expect("request with partial body destruction should execute");
    let mut mutated_value = result_value(&complete);
    let bodies = mutated_value["timelines"][0]["checkpoints"][7]["bodies"]
        .as_array_mut()
        .expect("partial destruction checkpoint should contain body snapshots");
    assert_eq!(bodies[0]["body_id"], json!("nc-kinematic"));
    assert_eq!(bodies[1]["body_id"], json!("nc-dynamic"));
    bodies[0]["body_id"] = json!("nc-static");
    bodies[1]["body_id"] = json!("nc-kinematic");
    let mutated = decode_result_value(&mutated_value);

    // Act and Assert
    assert_identity_rejected_on_each_side(
        &request,
        &complete,
        &mutated,
        &profile,
        "rigid_world.checkpoint.bodies.declaration_order",
    );
}

#[test]
fn comparison_rejects_same_count_stale_fixture_identities_on_each_engine_side() {
    // Arrange
    let request = comparison_request();
    let profile = profile();
    let complete = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound request should execute natively");
    let mut mutated_value = result_value(&complete);
    let fixtures = mutated_value["timelines"][1]["checkpoints"][8]["fixtures"]
        .as_array_mut()
        .expect("fixture-destruction checkpoint should contain one fixture snapshot");
    assert_eq!(fixtures[0]["fixture_id"], json!("contact-static-fixture"));
    fixtures[0]["fixture_id"] = json!("contact-dynamic-fixture");
    let mutated = decode_result_value(&mutated_value);

    // Act and Assert
    assert_identity_rejected_on_each_side(
        &request,
        &complete,
        &mutated,
        &profile,
        "rigid_world.checkpoint.fixtures.declaration_order",
    );
}

#[test]
fn checkpoint_live_identities_apply_body_destruction_fixture_cascades() {
    // Arrange
    let mut value = serde_json::from_slice::<Value>(REQUEST).expect("fixture should be JSON");
    value["scenario"]["timelines"][8]["checkpoints"]
        .as_array_mut()
        .expect("origin checkpoints should be an array")
        .push(json!({
            "checkpoint_id": "origin-right-destroyed",
            "after_action_id": "origin-09",
            "phase": "teardown",
            "counts": {
                "bodies": 1,
                "fixtures": 1,
                "contacts": 0,
                "manifold_points": 0,
                "events": 0,
                "destructions": 2
            },
            "transitions": []
        }));
    let request = decode_rigid_world_request_jsonl(
        &encode_value(&value),
        &HarnessLimits::phase2_default_v1(),
    )
    .expect("cascade checkpoint should decode");

    // Act
    let identities = rigid_world_checkpoint_live_identities(&request.scenario().timelines()[8], 1)
        .expect("validated cascade checkpoint should have live identities");

    // Assert
    assert_eq!(
        identities
            .body_ids()
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>(),
        ["origin-left"]
    );
    assert_eq!(
        identities
            .fixture_ids()
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>(),
        ["origin-left-fixture"]
    );
}

#[test]
fn comparison_reports_stable_first_numeric_divergence() {
    // Arrange
    let request = comparison_request();
    let profile = profile();
    let native = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound request should execute natively");
    let mut oracle_value = result_value(&native);
    let bits = oracle_value["timelines"][0]["checkpoints"][0]["bodies"][0]["transform"]["position"]
        ["x_bits"]
        .as_u64()
        .expect("position bits should be unsigned");
    oracle_value["timelines"][0]["checkpoints"][0]["bodies"][0]["transform"]["position"]["x_bits"] =
        json!(bits ^ 1);
    let oracle = decode_result_value(&oracle_value);

    // Act
    let first = compare_rigid_world_results(&request, &native, &oracle, &profile)
        .expect("aligned declarations should reach physics comparison");
    let second = compare_rigid_world_results(&request, &native, &oracle, &profile)
        .expect("replay should reach the same physics comparison");

    // Assert
    let RigidComparisonOutcome::PhysicsMismatch(first_report) = first else {
        panic!("one-bit mutation should mismatch");
    };
    let RigidComparisonOutcome::PhysicsMismatch(second_report) = second else {
        panic!("one-bit replay mutation should mismatch");
    };
    assert_eq!(first_report.kind(), RigidMismatchKind::Numeric);
    assert_eq!(
        first_report.semantic_path(),
        "rigid_world.body.transform.position.x"
    );
    assert_eq!(first_report.signature(), second_report.signature());
}

#[test]
fn comparison_never_canonicalizes_manager_report_or_destruction_order() {
    // Arrange
    let request = comparison_request();
    let profile = profile();
    let native = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound request should execute natively");
    let mut oracle_value = result_value(&native);
    let checkpoints = oracle_value["timelines"][1]["checkpoints"]
        .as_array_mut()
        .expect("checkpoints should be an array");
    let checkpoint = checkpoints
        .iter_mut()
        .find(|checkpoint| {
            checkpoint["events"]
                .as_array()
                .is_some_and(|events| events.len() >= 2)
        })
        .expect("contact timeline should contain ordered report events");
    checkpoint["events"]
        .as_array_mut()
        .expect("events should be an array")
        .swap(0, 1);
    let oracle = decode_result_value(&oracle_value);

    // Act
    let outcome = compare_rigid_world_results(&request, &native, &oracle, &profile)
        .expect("reordered reports remain declaration-valid");

    // Assert
    let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
        panic!("report-order mutation must mismatch");
    };
    assert_eq!(report.kind(), RigidMismatchKind::Order);
    assert_eq!(
        report.semantic_path(),
        "rigid_world.checkpoint.events.report_order"
    );
}

#[test]
fn supervisor_fails_closed_before_the_step_bearing_oracle_is_available() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(executable) = OracleExecutable::resolve(&root, OraclePreset::Debug) else {
        return;
    };
    let request = request();

    // Act
    let error = execute_rigid_world_process(&executable, &request, REVISION)
        .expect_err("the supervisor must reject the unsupported step-bearing request");

    // Assert
    assert_eq!(
        error.retained_stderr(),
        b"liquidfun-reference: unsupported Phase 8 execution action\n"
    );
    assert!(error.child_killed());
    assert!(error.child_reaped());
}

#[test]
fn reduction_preserves_validity_family_and_exact_first_divergence_signature() {
    // Arrange
    let profile = profile();
    let mut value = serde_json::from_slice::<Value>(REQUEST).expect("fixture should be JSON");
    value["tolerance_profile_sha256"] = json!(profile.profile_sha256().as_str());
    let actions = value["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array");
    let duplicate = actions[19].clone();
    let mut duplicate = duplicate;
    duplicate["action_id"] = json!("nc-custom-mass-redundant");
    actions.insert(20, duplicate);
    let request = decode_rigid_world_request_jsonl(
        &encode_value(&value),
        &HarnessLimits::phase2_default_v1(),
    )
    .expect("request with a redundant valid step should decode");
    let native = NativeRigidWorldExecutor::execute(&request)
        .expect("request with redundant step should execute");
    let mut oracle_value = result_value(&native);
    let bits = oracle_value["timelines"][0]["checkpoints"][0]["bodies"][0]["mass_bits"]
        .as_u64()
        .expect("mass bits should be unsigned");
    oracle_value["timelines"][0]["checkpoints"][0]["bodies"][0]["mass_bits"] = json!(bits ^ 1);
    let oracle = decode_result_value(&oracle_value);
    let RigidComparisonOutcome::PhysicsMismatch(report) =
        compare_rigid_world_results(&request, &native, &oracle, &profile)
            .expect("declarations should align")
    else {
        panic!("one-bit mutation should mismatch");
    };
    let target = report.signature().clone();
    let original_actions = request.scenario().timelines()[0].actions().len();

    // Act
    let result = minimize_rigid_world_request(
        &request,
        &target,
        MinimizationBudget::new(128, Duration::from_secs(1)),
        |_candidate| RigidEvaluation::new(Some(target.clone()), Duration::from_millis(1)),
    )
    .expect("typed rigid reduction should serialize its best candidate");

    // Assert
    assert_eq!(
        result.request().scenario().timelines().len(),
        RigidWorldWitnessFamily::ALL.len()
    );
    assert!(
        result.request().scenario().timelines()[0].actions().len() < original_actions,
        "the redundant action should be removable"
    );
    assert!(!result.accepted_transforms().is_empty());
    decode_rigid_world_request_jsonl(
        result.canonical_request_bytes(),
        &HarnessLimits::phase2_default_v1(),
    )
    .expect("reduced bytes must remain a valid rigid request");
}

#[test]
fn comparison_failure_bundle_retains_exact_rigid_signature() {
    // Arrange
    let request = comparison_request();
    let profile = profile();
    let native = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound request should execute natively");
    let mut oracle_value = result_value(&native);
    oracle_value["timelines"][0]["checkpoints"][0]["bodies"][0]["active"] = json!(false);
    let oracle = decode_result_value(&oracle_value);
    let RigidComparisonOutcome::PhysicsMismatch(report) =
        compare_rigid_world_results(&request, &native, &oracle, &profile)
            .expect("declarations should align")
    else {
        panic!("active-state mutation should mismatch");
    };
    let root = std::env::temp_dir().join(format!(
        "liquidfun-rigid-bundle-{}-{}",
        std::process::id(),
        report.signature().signature_sha256().as_str()
    ));
    fs::create_dir(&root).expect("temporary bundle root should be created");
    let request_jsonl = encode_jsonl(
        &request,
        &HarnessLimits::phase2_default_v1(),
        RecordLimit::Input,
    )
    .expect("request should encode");
    let report_json = report.render_machine().expect("report should encode");
    let signature_json = serde_json::to_vec(report.signature()).expect("signature should encode");

    // Act
    let receipt = persist_failure_bundle(
        &root,
        &FailureBundleRequest {
            result_kind: "physics_mismatch",
            request_id: request.request_id(),
            request_jsonl: &request_jsonl,
            report_json: &report_json,
            identity_json: b"{}",
            stderr: b"",
            maybe_failure_signature_json: Some(&signature_json),
        },
    )
    .expect("bounded rigid failure bundle should persist");

    // Assert
    assert_eq!(
        fs::read(receipt.directory().join("failure-signature.json"))
            .expect("signature evidence should be readable"),
        signature_json
    );
    fs::remove_dir_all(&root).expect("temporary bundle root should clean up");
}

fn source_digest(root: &std::path::Path, sources: &[&str], maybe_changed: Option<&str>) -> String {
    let mut hasher = Sha256::new();
    for relative in sources {
        let mut bytes = fs::read(root.join(relative)).expect("identity source should exist");
        if maybe_changed == Some(*relative) {
            bytes.push(b'!');
        }
        let file_digest = Sha256::digest(bytes);
        hasher.update((relative.len() as u64).to_be_bytes());
        hasher.update(relative.as_bytes());
        hasher.update(file_digest);
    }
    format!("{digest:x}", digest = hasher.finalize())
}
