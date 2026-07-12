//! Native Phase 6 rigid-world adapter integration tests.

use std::fs;
use std::process::Command;
use std::time::Duration;

use liquidfun_differential::{
    ArtifactKind, EmptyWorldAdapter, FailureBundleRequest, MinimizationBudget,
    NativeRigidWorldExecutor, OracleExecutable, OraclePreset, RigidComparisonFailure,
    RigidComparisonOutcome, RigidEvaluation, RigidMismatchKind, RigidPromotionError,
    compare_rigid_world_results, execute_rigid_world_process, minimize_rigid_world_request,
    persist_failure_bundle, validate_native_rigid_world_result, validate_rigid_promotion_authority,
};
use liquidfun_test_protocol::{
    HarnessLimits, Phase6PolicyProfile, RecordLimit, RigidWorldErrorKind, RigidWorldRequestRecord,
    RigidWorldResultRecord, decode_rigid_world_request_jsonl, decode_rigid_world_result_jsonl,
    encode_jsonl,
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

fn decode_result_value(value: &Value) -> RigidWorldResultRecord {
    decode_rigid_world_result_jsonl(&encode_value(value), &HarnessLimits::phase2_default_v1())
        .expect("mutated result should remain internally valid")
}

fn result_value(result: &RigidWorldResultRecord) -> Value {
    serde_json::to_value(result).expect("validated result should serialize")
}

#[test]
fn native_executes_both_families_deterministically_and_resets() {
    // Arrange
    let request = request();

    // Act
    let first = NativeRigidWorldExecutor::execute(&request)
        .expect("validated rigid-world request should execute natively");
    let second = NativeRigidWorldExecutor::execute(&request)
        .expect("a fresh native execution should reset all world state");

    // Assert
    assert_eq!(first, second);
    assert_eq!(first.timelines().len(), 2);
    assert_eq!(first.timelines()[0].checkpoints.len(), 5);
    assert_eq!(first.timelines()[1].checkpoints.len(), 10);
    validate_native_rigid_world_result(&request, &first)
        .expect("native result should agree with every declaration");
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
    unknown_identity["scenario"]["timelines"][0]["actions"][6]["action"]["body_id"] =
        json!("missing-body");

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
    assert_eq!(result.timelines().len(), 2);
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
fn supervisor_captures_rigid_result_identity_terminal_and_reset() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(executable) = OracleExecutable::resolve(&root, OraclePreset::Debug) else {
        return;
    };
    let request = request();

    // Act
    let captured = execute_rigid_world_process(&executable, &request, REVISION)
        .expect("reviewed rigid oracle should complete under bounded supervision");

    // Assert
    assert_eq!(captured.result().request_id(), request.request_id());
    assert_eq!(captured.reset_epoch(), 1);
    assert!(captured.reset_verified());
    assert!(!captured.response_bytes().is_empty());
    assert_eq!(captured.identity().oracle_revision(), REVISION);
}

#[test]
fn supervisor_rejects_local_d2_rigid_output_for_promotion() {
    // Arrange
    let identity = EmptyWorldAdapter::new(REVISION)
        .expect("native identity should validate")
        .build_identity()
        .clone();

    // Act
    let result = validate_rigid_promotion_authority(&identity, ArtifactKind::ReviewedTrace);

    // Assert
    assert!(matches!(
        result,
        Err(RigidPromotionError::NonCanonicalAuthority { .. })
    ));
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
    assert_eq!(result.request().scenario().timelines().len(), 2);
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
