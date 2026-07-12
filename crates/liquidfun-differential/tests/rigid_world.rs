//! Native Phase 6 rigid-world adapter integration tests.

use std::fs;
use std::process::Command;

use liquidfun_differential::{NativeRigidWorldExecutor, validate_native_rigid_world_result};
use liquidfun_test_protocol::{
    HarnessLimits, RecordLimit, RigidWorldErrorKind, decode_rigid_world_request_jsonl,
    decode_rigid_world_result_jsonl, encode_jsonl,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const REQUEST: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/rigid-world-request.jsonl");

fn request() -> liquidfun_test_protocol::RigidWorldRequestRecord {
    decode_rigid_world_request_jsonl(REQUEST, &HarnessLimits::phase2_default_v1())
        .expect("checked-in rigid-world request should decode")
}

fn encode_value(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).expect("fixture mutation should encode");
    bytes.push(b'\n');
    bytes
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
