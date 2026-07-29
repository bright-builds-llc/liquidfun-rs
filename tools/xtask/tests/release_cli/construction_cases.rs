use super::*;

fn release_constructor_source(repository: &Path) -> String {
    [
        "scripts/phase12-release-evidence.sh",
        "scripts/phase12-release-evidence/common.sh",
        "scripts/phase12-release-evidence/producer_validation.sh",
        "scripts/phase12-release-evidence/identity_validation.sh",
        "scripts/phase12-release-evidence/aggregation.sh",
    ]
    .into_iter()
    .map(|path| fs::read_to_string(repository.join(path)).expect("release constructor source"))
    .collect()
}

pub(super) fn release_constructor_is_check_first_aggregate_only_and_identity_last() {
    // Arrange
    let repository = repository_root();
    let workflow = fs::read_to_string(repository.join(".github/workflows/release.yml"))
        .expect("release workflow");
    let script = release_constructor_source(&repository);

    // Act
    let cheap = workflow
        .find("Run inexpensive candidate checks")
        .expect("cheap checks");
    let download = workflow
        .find("Download exact reviewed producer artifacts")
        .expect("artifact download");
    let audit = workflow.find("release audit").expect("release audit");
    let upload = workflow
        .find("Upload validated release-candidate evidence")
        .expect("validated upload");

    // Assert
    assert!(cheap < download && download < audit && audit < upload);
    assert_eq!(workflow.matches("release audit").count(), 1);
    assert!(workflow.contains("cancel-in-progress: false"));
    assert!(script.contains("cargo publish -p liquidfun --dry-run"));
    assert!(!workflow.contains("cargo publish -p liquidfun\n"));
    assert!(!script.contains("cargo publish -p liquidfun\n"));
    assert!(script.contains("set -euo pipefail"));
    assert!(script.contains("publish_identity_last"));
    for forbidden in [
        "cargo fuzz",
        "cargo bench",
        "phase12-miri.sh run",
        "phase12-rust-sanitizers.sh run",
        "phase12-coverage.sh rust",
        "phase12-performance.sh paired",
        "phase12-regressions.sh run",
    ] {
        assert!(
            !workflow.contains(forbidden) && !script.contains(forbidden),
            "constructor reruns an expensive producer: {forbidden}"
        );
    }
}

pub(super) fn release_constructor_names_every_exact_artifact_and_never_writes_tracked_readiness() {
    // Arrange
    let repository = repository_root();
    let workflow = fs::read_to_string(repository.join(".github/workflows/release.yml"))
        .expect("release workflow");
    let script = release_constructor_source(&repository);

    // Act / Assert
    for artifact_pattern in [
        "phase12-package-",
        "phase12-platform-msrv-",
        "phase12-platform-x86_64-unknown-linux-gnu-",
        "phase12-platform-aarch64-unknown-linux-gnu-",
        "phase12-platform-aarch64-apple-darwin-",
        "phase12-platform-x86_64-pc-windows-msvc-",
        "phase12-platform-x86_64-apple-darwin",
        "phase11-canonical-",
        "phase11-sanitizer-",
        "phase12-miri-",
        "phase12-rust-sanitizer-",
        "fuzz-protocol-",
        "fuzz-shapes_collision-",
        "fuzz-world_mutation-",
        "fuzz-particles-",
        "fuzz-groups_ownership-",
        "phase12-regressions-",
        "phase12-rust-coverage-",
        "phase12-cpp-coverage-",
        "phase12-differential-coverage-",
        "phase12-performance-",
    ] {
        assert!(
            script.contains(artifact_pattern),
            "missing exact artifact pattern: {artifact_pattern}"
        );
    }
    for tracked_output in [
        "reference/release/candidate-manifest.json",
        "reference/release/readiness.json",
        "reference/release/audit.json",
    ] {
        assert!(
            !workflow.contains(tracked_output) && !script.contains(tracked_output),
            "constructor writes tracked readiness: {tracked_output}"
        );
    }
    assert!(script.contains("validate_artifact_set"));
    assert!(script.contains("validate_producer_identities"));
    assert!(script.contains("candidate-manifest.json"));
    assert!(script.contains("audit-report.json"));
    for retained_path in [
        "${{ env.OUTPUT_DIRECTORY }}/artifacts/*.json",
        "${{ env.OUTPUT_DIRECTORY }}/package/liquidfun.crate",
        "${{ env.OUTPUT_DIRECTORY }}/package/package-identity.json",
        "${{ env.OUTPUT_DIRECTORY }}/candidate-manifest.json",
        "${{ env.OUTPUT_DIRECTORY }}/audit-report.json",
        "${{ env.OUTPUT_DIRECTORY }}/audit-identity.json",
    ] {
        assert!(
            workflow.contains(retained_path),
            "release upload omits auditable payload: {retained_path}"
        );
    }
}

pub(super) fn release_constructor_rejects_mutated_platform_hash_and_tier() {
    // Arrange
    let root = producer_fixture_root("platform-mutation");
    let identity_path = root.join("identity.json");
    let verification_path = root.join("verification.json");
    fs::write(
        &identity_path,
        serde_json::to_vec(&json!({
            "schema_version": 1,
            "archive_sha256": "a".repeat(64),
            "target": "x86_64-unknown-linux-gnu",
            "compiler": "rustc 1.97.0",
            "scalar_mode": "strict_f32",
            "tier": "d1_canonical",
            "candidate_sha": CANDIDATE,
            "runner": "ubuntu-24.04",
            "workflow": "Platform release candidate",
            "job": "native",
            "run_id": 7,
            "recorded_at_unix": 1,
            "native_evidence_recorded_at_unix": null,
        }))
        .expect("identity JSON"),
    )
    .expect("identity writes");
    fs::write(
        &verification_path,
        br#"{"status":"verified","package_isolation":true,"rustdoc":true,"platform_smoke":true}"#,
    )
    .expect("verification writes");

    // Act
    let output = run_release_validator(
        "validate_platform_payload",
        &[
            identity_path.to_string_lossy().into_owned(),
            verification_path.to_string_lossy().into_owned(),
            CANDIDATE.to_owned(),
            "x86_64-unknown-linux-gnu".to_owned(),
            "7".to_owned(),
            "native".to_owned(),
            "b".repeat(64),
            "d2_supported".to_owned(),
        ],
    );

    // Assert
    assert!(!output.status.success());
}

pub(super) fn release_constructor_rejects_sanitizer_finding() {
    // Arrange
    let root = producer_fixture_root("safety-mutation");
    let summary_path = root.join("sanitizer.jsonl");
    fs::write(
        &summary_path,
        serde_json::to_vec(&json!({ "outcome": "sanitizer_finding" })).expect("sanitizer JSON"),
    )
    .expect("summary writes");

    // Act
    let output = run_release_validator(
        "validate_sanitizer_records",
        &[summary_path.to_string_lossy().into_owned()],
    );

    // Assert
    assert!(!output.status.success());
}

pub(super) fn release_constructor_rejects_canonical_gap_result() {
    // Arrange
    let root = producer_fixture_root("canonical-gap-mutation");
    let identity_path = root.join("identity.json");
    fs::write(
        &identity_path,
        serde_json::to_vec(&json!({ "semantic_sha256": "a".repeat(64) })).expect("identity JSON"),
    )
    .expect("identity writes");
    let result_path = root.join("semantic-result.json");
    fs::write(
        &result_path,
        serde_json::to_vec(&json!({
            "schema_version": 1,
            "evidence_kind": "canonical_differential",
            "candidate_commit": CANDIDATE,
            "complete": true,
            "parity_tier": "d1_canonical",
            "coverage_authority": false,
            "performance_authority": false,
            "gap_count": 1,
            "semantic_sha256": "a".repeat(64),
        }))
        .expect("canonical result JSON"),
    )
    .expect("canonical result writes");

    // Act
    let output = run_release_validator(
        "validate_canonical_payload",
        &[
            result_path.to_string_lossy().into_owned(),
            identity_path.to_string_lossy().into_owned(),
            CANDIDATE.to_owned(),
        ],
    );

    // Assert
    assert!(!output.status.success());
}

pub(super) fn release_constructor_accepts_typed_canonical_and_safety_results() {
    // Arrange
    let root = producer_fixture_root("typed-producer-results");
    let identity_path = root.join("identity.json");
    fs::write(
        &identity_path,
        serde_json::to_vec(&json!({ "semantic_sha256": "a".repeat(64) })).expect("identity JSON"),
    )
    .expect("identity writes");
    let result_path = root.join("semantic-result.json");
    fs::write(
        &result_path,
        serde_json::to_vec(&json!({
            "schema_version": 1,
            "evidence_kind": "canonical_differential",
            "candidate_commit": CANDIDATE,
            "complete": true,
            "parity_tier": "d1_canonical",
            "coverage_authority": false,
            "performance_authority": false,
            "gap_count": 0,
            "semantic_sha256": "a".repeat(64),
        }))
        .expect("canonical result JSON"),
    )
    .expect("canonical result writes");
    let logs = root.join("logs");
    fs::create_dir_all(&logs).expect("log directory");
    let log = b"clean safety run\n";
    fs::write(logs.join("math.log"), log).expect("log writes");
    let summary_path = root.join("summary.json");
    fs::write(
        &summary_path,
        serde_json::to_vec(&json!({
            "schema_version": 1,
            "evidence_kind": "miri",
            "candidate_commit": CANDIDATE,
            "toolchain_identity": "nightly-2026-07-15",
            "complete": true,
            "parity_authority": false,
            "policy": {
                "unsafe_code": "forbid",
                "unsafe_waivers": 0,
                "advisory_waivers": 0,
            },
            "cases": [{
                "name": "math",
                "path": "logs/math.log",
                "sha256": sha256(log),
                "bytes": log.len(),
            }],
        }))
        .expect("safety summary JSON"),
    )
    .expect("safety summary writes");

    // Act
    let canonical = run_release_validator(
        "validate_canonical_payload",
        &[
            result_path.to_string_lossy().into_owned(),
            identity_path.to_string_lossy().into_owned(),
            CANDIDATE.to_owned(),
        ],
    );
    let safety = run_release_validator(
        "validate_safety_payload",
        &[
            summary_path.to_string_lossy().into_owned(),
            CANDIDATE.to_owned(),
            "miri".to_owned(),
        ],
    );

    // Assert
    assert!(canonical.status.success());
    assert!(safety.status.success());
}

pub(super) fn release_constructor_rejects_safety_waiver_policy() {
    // Arrange
    let root = producer_fixture_root("safety-policy-mutation");
    let logs = root.join("logs");
    fs::create_dir_all(&logs).expect("log directory");
    let log = b"clean safety run\n";
    fs::write(logs.join("math.log"), log).expect("log writes");
    let summary_path = root.join("summary.json");
    fs::write(
        &summary_path,
        serde_json::to_vec(&json!({
            "schema_version": 1,
            "evidence_kind": "miri",
            "candidate_commit": CANDIDATE,
            "toolchain_identity": "nightly-2026-07-15",
            "complete": true,
            "parity_authority": false,
            "policy": {
                "unsafe_code": "forbid",
                "unsafe_waivers": 1,
                "advisory_waivers": 0,
            },
            "cases": [{
                "name": "math",
                "path": "logs/math.log",
                "sha256": sha256(log),
                "bytes": log.len(),
            }],
        }))
        .expect("safety summary JSON"),
    )
    .expect("safety summary writes");

    // Act
    let output = run_release_validator(
        "validate_safety_payload",
        &[
            summary_path.to_string_lossy().into_owned(),
            CANDIDATE.to_owned(),
            "miri".to_owned(),
        ],
    );

    // Assert
    assert!(!output.status.success());
}

pub(super) fn release_constructor_rejects_tampered_safety_log() {
    // Arrange
    let root = producer_fixture_root("safety-log-mutation");
    let logs = root.join("logs");
    fs::create_dir_all(&logs).expect("log directory");
    let original = b"clean safety run\n";
    let log_path = logs.join("math.log");
    fs::write(&log_path, original).expect("log writes");
    let summary_path = root.join("summary.json");
    fs::write(
        &summary_path,
        serde_json::to_vec(&json!({
            "schema_version": 1,
            "evidence_kind": "miri",
            "candidate_commit": CANDIDATE,
            "toolchain_identity": "nightly-2026-07-15",
            "complete": true,
            "parity_authority": false,
            "policy": {
                "unsafe_code": "forbid",
                "unsafe_waivers": 0,
                "advisory_waivers": 0,
            },
            "cases": [{
                "name": "math",
                "path": "logs/math.log",
                "sha256": sha256(original),
                "bytes": original.len(),
            }],
        }))
        .expect("safety summary JSON"),
    )
    .expect("safety summary writes");
    fs::write(log_path, b"tampered safety run\n").expect("log mutation writes");

    // Act
    let output = run_release_validator(
        "validate_safety_payload",
        &[
            summary_path.to_string_lossy().into_owned(),
            CANDIDATE.to_owned(),
            "miri".to_owned(),
        ],
    );

    // Assert
    assert!(!output.status.success());
}
