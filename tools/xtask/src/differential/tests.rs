use liquidfun_differential::{
    NativeMathProbeExecutor, NativeRigidWorldExecutor, Phase4ComparisonEvidence,
    Phase4HarnessFailureReason, RigidComparisonOutcome, RigidEvaluation,
    compare_phase8_rigid_world_results,
};
use liquidfun_test_protocol::{
    BuildIdentity, BuildIdentityFields, DivergenceHorizon, EvidenceTier, FloatBits, HarnessLimits,
    MathProbeDiscrete, MathProbeDiscreteField, MathProbeHorizon, MathProbeOperation,
    MathProbePolicyPath, MathProbeRequestRecord, MathProbeResult, MathProbeValue,
    Phase4BuildIdentityFields, Phase4PolicyProfile, Phase6PolicyProfile, Phase7PolicyProfile,
    Phase8PolicyProfile, decode_math_probe_request_jsonl, decode_rigid_world_request_jsonl,
    decode_rigid_world_result_jsonl,
};

use std::{fs, time::Duration};

mod policy;

use policy::supported_math_identity;

use super::{
    ORACLE_REVISION, RIGID_WORLD_REQUEST, RigidMinimizationMachineReport,
    compare_math_probe_results, horizons_match, native_source_digest_from_manifest,
    reduce_rigid_world_mismatch, rigid_world_request, tier_authorizes,
};

#[test]
fn rigid_world_request_rejects_stale_policy_provenance() {
    // Arrange
    let root = std::env::temp_dir().join(format!(
        "liquidfun-rigid-policy-provenance-{}",
        std::process::id()
    ));
    let request_path = root.join(RIGID_WORLD_REQUEST);
    fs::create_dir_all(request_path.parent().expect("request path has a parent"))
        .expect("temporary request directory should be created");
    let mut request_value: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../../../protocol/fixtures/accepted/rigid-world-request.jsonl"
    ))
    .expect("checked-in request should parse");
    request_value["tolerance_profile_sha256"] = serde_json::Value::String("0".repeat(64));
    let mut request_bytes =
        serde_json::to_vec(&request_value).expect("stale request should encode");
    request_bytes.push(b'\n');
    fs::write(&request_path, request_bytes).expect("stale request should be written");
    let policy = Phase8PolicyProfile::parse_toml(include_str!(
        "../../../../protocol/tolerances/phase8-v1.toml"
    ))
    .expect("checked-in Phase 7 policy should parse");

    // Act
    let error =
        rigid_world_request(&root, &policy).expect_err("stale request provenance must fail closed");

    // Assert
    assert_eq!(error.category, "policy");
    assert!(error.message.contains("request policy hash"));
    fs::remove_dir_all(root).expect("temporary fixture should be removed");
}

#[test]
fn rigid_minimization_path_runs_reducer_and_records_transform_provenance() {
    // Arrange
    let request = decode_rigid_world_request_jsonl(
        include_bytes!("../../../../protocol/fixtures/accepted/rigid-world-request.jsonl"),
        &HarnessLimits::phase2_default_v1(),
    )
    .expect("checked-in rigid request should decode");
    let phase6 = Phase6PolicyProfile::parse_toml(include_str!(
        "../../../../protocol/tolerances/phase6-v1.toml"
    ))
    .expect("checked-in Phase 6 policy should parse");
    let phase7 = Phase7PolicyProfile::parse_toml(include_str!(
        "../../../../protocol/tolerances/phase7-v1.toml"
    ))
    .expect("checked-in Phase 7 policy should parse");
    let phase8 = Phase8PolicyProfile::parse_toml(include_str!(
        "../../../../protocol/tolerances/phase8-v1.toml"
    ))
    .expect("checked-in Phase 8 policy should parse");
    let native = NativeRigidWorldExecutor::execute(&request)
        .expect("checked-in rigid request should execute");
    let mut oracle_value = serde_json::to_value(&native).expect("result should serialize");
    oracle_value["timelines"][0]["checkpoints"][0]["bodies"][0]["active"] =
        serde_json::Value::Bool(false);
    let mut oracle_bytes =
        serde_json::to_vec(&oracle_value).expect("mismatched result should encode");
    oracle_bytes.push(b'\n');
    let oracle =
        decode_rigid_world_result_jsonl(&oracle_bytes, &HarnessLimits::phase2_default_v1())
            .expect("mismatched semantic result should decode");
    let RigidComparisonOutcome::PhysicsMismatch(mismatch) =
        compare_phase8_rigid_world_results(&request, &native, &oracle, &phase6, &phase7, &phase8)
            .expect("registered rigid evidence should compare")
    else {
        panic!("active-state mutation must mismatch");
    };
    let target = mismatch.signature().clone();

    // Act
    let result = reduce_rigid_world_mismatch(
        &request,
        &target,
        liquidfun_differential::MinimizationBudget::new(128, Duration::from_secs(1)),
        |_candidate| RigidEvaluation::new(Some(target.clone()), Duration::from_millis(1)),
    )
    .expect("internal rigid minimization path should run the reducer");
    let report = RigidMinimizationMachineReport::new(&target, &request, &result)
        .expect("minimization report should encode its source request");
    let report_value = serde_json::to_value(&report).expect("report should serialize");
    let root = std::env::temp_dir().join(format!(
        "liquidfun-rigid-minimization-{}",
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root).expect("stale temporary fixture should be removed");
    }
    fs::create_dir(&root).expect("temporary persistence root should be created");
    let report_bytes = serde_json::to_vec(&report).expect("report should encode");
    let receipt = liquidfun_differential::persist_rigid_minimization_artifact(
        &root,
        &liquidfun_differential::RigidMinimizationArtifactRequest {
            request_id: request.request_id(),
            request_jsonl: result.canonical_request_bytes(),
            report_json: &report_bytes,
        },
    )
    .expect("minimized rigid request should persist");

    // Assert
    assert!(result.evaluations() > 0);
    assert!(!result.accepted_transforms().is_empty());
    assert_eq!(
        report_value["accepted_transforms"]
            .as_array()
            .expect("accepted transforms should be an array")
            .len(),
        result.accepted_transforms().len()
    );
    assert_eq!(
        report_value["status"],
        serde_json::to_value(result.status()).expect("status should serialize")
    );
    assert_eq!(
        fs::read(receipt.directory().join("request.jsonl"))
            .expect("persisted request should be readable"),
        result.canonical_request_bytes()
    );
    fs::remove_dir_all(root).expect("temporary fixture should be removed");
}

#[test]
fn native_source_digest_changes_when_an_executor_input_changes() {
    // Arrange
    let root =
        std::env::temp_dir().join(format!("liquidfun-native-manifest-{}", std::process::id()));
    let source = root.join("crates/liquidfun-differential/src/math_probe.rs");
    fs::create_dir_all(source.parent().expect("fixture source has a parent"))
        .expect("temporary fixture directory should be created");
    fs::write(&source, b"executor v1").expect("first fixture should be written");
    let manifest = "crates/liquidfun-differential/src/math_probe.rs\n";
    let original =
        native_source_digest_from_manifest(&root, manifest).expect("original manifest should hash");

    // Act
    fs::write(&source, b"executor v2").expect("changed fixture should be written");
    let changed =
        native_source_digest_from_manifest(&root, manifest).expect("changed manifest should hash");

    // Assert
    assert_ne!(original, changed);
    fs::remove_dir_all(root).expect("temporary fixture should be removed");
}

#[test]
fn actual_xtask_math_mismatch_carries_typed_machine_evidence() {
    // Arrange
    let request = decode_math_probe_request_jsonl(
        include_bytes!("../../../../protocol/fixtures/accepted/math-probe-request.jsonl"),
        &HarnessLimits::phase2_default_v1(),
    )
    .expect("checked-in request should decode");
    let policy = Phase4PolicyProfile::parse_toml(include_str!(
        "../../../../protocol/tolerances/phase4-v1.toml"
    ))
    .expect("checked-in policy should parse");
    let mut actual = NativeMathProbeExecutor::execute(&request)
        .expect("checked-in request should execute")
        .into_vec();
    let case_index = actual
        .iter()
        .position(|result| {
            result
                .values()
                .first()
                .is_some_and(|value| value.bits().to_f32().is_finite())
        })
        .expect("fixture should contain a finite scalar result");
    let result = &actual[case_index];
    let mut values = result.values().to_vec();
    values[0] = MathProbeValue::new(values[0].field(), FloatBits::new(0x7f80_0000));
    actual[case_index] = MathProbeResult::new(
        result.case_id(),
        result.operation(),
        result.policy_path(),
        result.horizon(),
        values,
        result.discrete().to_vec(),
    );
    let oracle_identity = supported_math_identity("11");
    let native_identity = supported_math_identity("22");

    // Act
    let error = compare_math_probe_results(
        &request,
        &actual,
        &policy,
        &oracle_identity,
        &native_identity,
    )
    .expect_err("deliberate divergence should fail");

    // Assert
    assert_eq!(error.category, "physics-mismatch");
    let evidence = error
        .maybe_phase4_evidence
        .expect("actual xtask comparison should retain typed mismatch evidence");
    assert!(matches!(
        evidence.as_ref(),
        Phase4ComparisonEvidence::NumericMismatch(_)
    ));
    let machine = evidence
        .render_machine()
        .expect("typed report should serialize");
    let machine = String::from_utf8(machine).expect("JSON report should be UTF-8");
    assert!(machine.contains("\"policy_id\":\"phase4-v1\""));
    assert!(machine.contains("\"evidence_tier\":\"d2_supported\""));
    assert!(machine.contains("\"oracle_build_sha256\""));
    assert!(machine.contains("\"native_build_sha256\""));
    assert!(machine.contains("\"collection_policy\":\"ordered\""));
}

#[test]
fn actual_xtask_result_count_failure_is_typed_harness_evidence() {
    // Arrange
    let (request, policy, mut actual, oracle_identity, native_identity) = math_fixture();
    actual.pop().expect("fixture should contain results");

    // Act
    let error = compare_math_probe_results(
        &request,
        &actual,
        &policy,
        &oracle_identity,
        &native_identity,
    )
    .expect_err("result count violation should fail");

    // Assert
    assert_harness_reason(error, Phase4HarnessFailureReason::ResultCount);
}

#[test]
fn actual_xtask_structural_echo_failure_is_typed_harness_evidence() {
    // Arrange
    let (request, policy, mut actual, oracle_identity, native_identity) = math_fixture();
    let result = &actual[0];
    actual[0] = MathProbeResult::new(
        result.case_id(),
        MathProbeOperation::Abs,
        result.policy_path(),
        result.horizon(),
        result.values().to_vec(),
        result.discrete().to_vec(),
    );

    // Act
    let error = compare_math_probe_results(
        &request,
        &actual,
        &policy,
        &oracle_identity,
        &native_identity,
    )
    .expect_err("operation echo violation should fail");

    // Assert
    assert_harness_reason(error, Phase4HarnessFailureReason::OperationEcho);
}

#[test]
fn actual_xtask_every_structural_failure_reason_is_typed_harness_evidence() {
    // Arrange
    let (request, policy, baseline, oracle_identity, native_identity) = math_fixture();
    let first = &baseline[0];
    let mut variants = Vec::new();

    let mut case_id = baseline.clone();
    case_id[0] = MathProbeResult::new(
        "changed-case-id",
        first.operation(),
        first.policy_path(),
        first.horizon(),
        first.values().to_vec(),
        first.discrete().to_vec(),
    );
    variants.push((case_id, Phase4HarnessFailureReason::CaseIdEcho));

    let mut policy_path = baseline.clone();
    policy_path[0] = MathProbeResult::new(
        first.case_id(),
        first.operation(),
        MathProbePolicyPath::MathOperationAbs,
        first.horizon(),
        first.values().to_vec(),
        first.discrete().to_vec(),
    );
    variants.push((policy_path, Phase4HarnessFailureReason::PolicyPathEcho));

    let mut horizon = baseline.clone();
    horizon[0] = MathProbeResult::new(
        first.case_id(),
        first.operation(),
        first.policy_path(),
        MathProbeHorizon::ScenarioSteps { steps: 4 },
        first.values().to_vec(),
        first.discrete().to_vec(),
    );
    variants.push((horizon, Phase4HarnessFailureReason::HorizonEcho));

    let value_index = baseline
        .iter()
        .position(|result| !result.values().is_empty())
        .expect("fixture should contain float values");
    let value_result = &baseline[value_index];
    let mut value_count = baseline.clone();
    let mut shortened_values = value_result.values().to_vec();
    shortened_values.pop().expect("selected result has a value");
    value_count[value_index] = MathProbeResult::new(
        value_result.case_id(),
        value_result.operation(),
        value_result.policy_path(),
        value_result.horizon(),
        shortened_values,
        value_result.discrete().to_vec(),
    );
    variants.push((value_count, Phase4HarnessFailureReason::ValueCount));

    let discrete_index = baseline
        .iter()
        .position(|result| !result.discrete().is_empty())
        .expect("fixture should contain discrete values");
    let discrete_result = &baseline[discrete_index];
    let mut discrete_count = baseline.clone();
    discrete_count[discrete_index] = MathProbeResult::new(
        discrete_result.case_id(),
        discrete_result.operation(),
        discrete_result.policy_path(),
        discrete_result.horizon(),
        discrete_result.values().to_vec(),
        Vec::new(),
    );
    variants.push((discrete_count, Phase4HarnessFailureReason::DiscreteCount));

    let mut discrete_field = baseline.clone();
    let mut changed_discrete = discrete_result.discrete().to_vec();
    changed_discrete[0] = MathProbeDiscrete::new(
        MathProbeDiscreteField::NonZeroDeterminant,
        changed_discrete[0].value(),
    );
    discrete_field[discrete_index] = MathProbeResult::new(
        discrete_result.case_id(),
        discrete_result.operation(),
        discrete_result.policy_path(),
        discrete_result.horizon(),
        discrete_result.values().to_vec(),
        changed_discrete,
    );
    variants.push((
        discrete_field,
        Phase4HarnessFailureReason::DiscreteFieldEcho,
    ));

    for (actual, expected_reason) in variants {
        // Act
        let error = compare_math_probe_results(
            &request,
            &actual,
            &policy,
            &oracle_identity,
            &native_identity,
        )
        .expect_err("structural violation should fail");

        // Assert
        assert_harness_reason(error, expected_reason);
    }
}

#[test]
fn actual_xtask_unregistered_policy_is_typed_harness_evidence() {
    // Arrange
    let (request, _policy, actual, oracle_identity, native_identity) = math_fixture();
    let path = actual[0].policy_path().as_str();
    let policy_text = policy_without_path(
        include_str!("../../../../protocol/tolerances/phase4-v1.toml"),
        path,
    );
    let policy = Phase4PolicyProfile::parse_toml(&policy_text)
        .expect("profile without one path remains structurally valid");

    // Act
    let error = compare_math_probe_results(
        &request,
        &actual,
        &policy,
        &oracle_identity,
        &native_identity,
    )
    .expect_err("missing policy should fail");

    // Assert
    assert_harness_reason(error, Phase4HarnessFailureReason::UnregisteredPolicy);
}

#[test]
fn actual_xtask_policy_horizon_violation_is_typed_harness_evidence() {
    // Arrange
    let (request, _policy, actual, oracle_identity, native_identity) = math_fixture();
    let path = actual[0].policy_path().as_str();
    let policy_text = replace_in_policy_block(
        include_str!("../../../../protocol/tolerances/phase4-v1.toml"),
        path,
        "horizon = { kind = \"operation\" }",
        "horizon = { kind = \"scenario_steps\", steps = 4 }",
    );
    let policy = Phase4PolicyProfile::parse_toml(&policy_text)
        .expect("alternate nonzero horizon remains structurally valid");

    // Act
    let error = compare_math_probe_results(
        &request,
        &actual,
        &policy,
        &oracle_identity,
        &native_identity,
    )
    .expect_err("policy horizon mismatch should fail");

    // Assert
    assert_harness_reason(error, Phase4HarnessFailureReason::PolicyHorizon);
}

#[test]
fn actual_xtask_policy_tier_violation_is_typed_harness_evidence() {
    // Arrange
    let (request, _policy, actual, oracle_identity, native_identity) = math_fixture();
    let path = actual[0].policy_path().as_str();
    let policy_text = replace_in_policy_block(
        include_str!("../../../../protocol/tolerances/phase4-v1.toml"),
        path,
        "evidence_tier = \"d1_canonical\"",
        "evidence_tier = \"d3_exploratory\"",
    );
    let policy = Phase4PolicyProfile::parse_toml(&policy_text)
        .expect("exploratory policy tier remains structurally valid");

    // Act
    let error = compare_math_probe_results(
        &request,
        &actual,
        &policy,
        &oracle_identity,
        &native_identity,
    )
    .expect_err("unauthorized policy tier should fail");

    // Assert
    assert_harness_reason(error, Phase4HarnessFailureReason::PolicyTier);
}

#[test]
fn actual_xtask_discrete_difference_is_typed_mismatch_evidence() {
    // Arrange
    let (request, policy, mut actual, oracle_identity, native_identity) = math_fixture();
    let case_index = actual
        .iter()
        .position(|result| !result.discrete().is_empty())
        .expect("fixture should contain a discrete result");
    let result = &actual[case_index];
    let mut discrete = result.discrete().to_vec();
    discrete[0] = MathProbeDiscrete::new(discrete[0].field(), !discrete[0].value());
    actual[case_index] = MathProbeResult::new(
        result.case_id(),
        result.operation(),
        result.policy_path(),
        result.horizon(),
        result.values().to_vec(),
        discrete,
    );

    // Act
    let error = compare_math_probe_results(
        &request,
        &actual,
        &policy,
        &oracle_identity,
        &native_identity,
    )
    .expect_err("discrete semantic difference should fail");

    // Assert
    assert_eq!(error.category, "physics-mismatch");
    let evidence = error
        .maybe_phase4_evidence
        .expect("discrete mismatch should carry typed evidence");
    assert!(matches!(
        evidence.as_ref(),
        Phase4ComparisonEvidence::DiscreteMismatch(_)
    ));
    let machine = String::from_utf8(
        evidence
            .render_machine()
            .expect("discrete evidence should serialize"),
    )
    .expect("JSON evidence should be UTF-8");
    assert!(machine.contains("\"expected_value\""));
    assert!(machine.contains("\"actual_value\""));
    assert!(machine.contains("\"policy_id\":\"phase4-v1\""));
}

fn math_fixture() -> (
    MathProbeRequestRecord,
    Phase4PolicyProfile,
    Vec<MathProbeResult>,
    BuildIdentity,
    BuildIdentity,
) {
    let request = decode_math_probe_request_jsonl(
        include_bytes!("../../../../protocol/fixtures/accepted/math-probe-request.jsonl"),
        &HarnessLimits::phase2_default_v1(),
    )
    .expect("checked-in request should decode");
    let policy = Phase4PolicyProfile::parse_toml(include_str!(
        "../../../../protocol/tolerances/phase4-v1.toml"
    ))
    .expect("checked-in policy should parse");
    let actual = NativeMathProbeExecutor::execute(&request)
        .expect("checked-in request should execute")
        .into_vec();
    (
        request,
        policy,
        actual,
        supported_math_identity("11"),
        supported_math_identity("22"),
    )
}

fn assert_harness_reason(
    error: super::DifferentialError,
    expected_reason: Phase4HarnessFailureReason,
) {
    assert_eq!(error.category, "harness-failure");
    let evidence = error
        .maybe_phase4_evidence
        .expect("harness failure should carry typed evidence");
    let Phase4ComparisonEvidence::HarnessFailure(report) = evidence.as_ref() else {
        panic!("expected typed harness evidence");
    };
    assert_eq!(report.reason(), expected_reason);
    assert!(report.render_human().len() < 1024);
    let machine = serde_json::to_vec(report).expect("harness evidence should serialize");
    assert!(machine.len() < 4096);
    assert_eq!(report.signature_sha256().as_str().len(), 64);
}

fn policy_without_path(input: &str, path: &str) -> String {
    let mut output = String::new();
    for (index, section) in input.split("[[fields]]").enumerate() {
        if index == 0 {
            output.push_str(section);
            continue;
        }
        if section.contains(&format!("semantic_path = \"{path}\"")) {
            continue;
        }
        output.push_str("[[fields]]");
        output.push_str(section);
    }
    output
}

fn replace_in_policy_block(input: &str, path: &str, original: &str, replacement: &str) -> String {
    let mut output = String::new();
    for (index, section) in input.split("[[fields]]").enumerate() {
        if index == 0 {
            output.push_str(section);
            continue;
        }
        output.push_str("[[fields]]");
        if section.contains(&format!("semantic_path = \"{path}\"")) {
            output.push_str(&section.replacen(original, replacement, 1));
        } else {
            output.push_str(section);
        }
    }
    output
}
