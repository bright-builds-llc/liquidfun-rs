//! Exhaustive acceptance tests for the pure release-manifest audit.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const CANDIDATE: &str = "1111111111111111111111111111111111111111";
const OTHER_CANDIDATE: &str = "2222222222222222222222222222222222222222";
static TEST_ORDINAL: AtomicU64 = AtomicU64::new(1);

#[derive(Deserialize)]
struct RequiredRegistry {
    evidence: Vec<RequiredEvidence>,
}

#[derive(Deserialize)]
struct RequiredEvidence {
    kind: String,
    target: String,
    workflow: String,
    job: String,
    toolchain: String,
}

struct Fixture {
    root: PathBuf,
    manifest: Value,
}

impl Fixture {
    fn complete(name: &str) -> Self {
        let repository = repository_root();
        let ordinal = TEST_ORDINAL.fetch_add(1, Ordering::Relaxed);
        let root = repository
            .join("target/xtask-release-tests")
            .join(format!("{name}-{}-{ordinal}", std::process::id()));
        fs::create_dir_all(&root).expect("fixture directory");
        let archive_path = root.join("liquidfun.crate");
        fs::write(&archive_path, b"reviewed package archive\n").expect("package archive");
        let package_sha256 = sha256(&fs::read(&archive_path).expect("package archive can be read"));
        let required: RequiredRegistry = toml::from_str(
            &fs::read_to_string(repository.join("reference/release/required-evidence.toml"))
                .expect("required evidence registry"),
        )
        .expect("required evidence TOML");
        let mut items = Vec::new();
        for (index, evidence) in required.evidence.iter().enumerate() {
            let claims = claims_for(&repository, evidence, &archive_path, &package_sha256);
            let payload_sha256 = sha256(&serde_json::to_vec(&claims).expect("claims serialize"));
            let artifact = json!({
                "schema_version": 1,
                "kind": evidence.kind,
                "target": evidence.target,
                "candidate_commit": CANDIDATE,
                "status": "passed",
                "payload_sha256": payload_sha256,
                "claims": claims,
            });
            let artifact_path = root.join(format!("artifact-{index}.json"));
            let artifact_bytes = serde_json::to_vec_pretty(&artifact).expect("artifact serializes");
            fs::write(&artifact_path, &artifact_bytes).expect("artifact writes");
            items.push(json!({
                "kind": evidence.kind,
                "target": evidence.target,
                "candidate_commit": CANDIDATE,
                "producer": {
                    "workflow": evidence.workflow,
                    "job": evidence.job,
                    "run_id": (index + 1).to_string(),
                },
                "artifact_path": repository_relative(&repository, &artifact_path),
                "artifact_sha256": sha256(&artifact_bytes),
                "payload_sha256": payload_sha256,
                "toolchain": evidence.toolchain,
                "review_status": "reviewed",
                "status": "passed",
            }));
        }
        Self {
            root,
            manifest: json!({
                "schema_version": 1,
                "candidate_commit": CANDIDATE,
                "items": items,
            }),
        }
    }

    fn write_manifest(&self, name: &str, manifest: &Value) -> PathBuf {
        let manifest_path = self.root.join(name);
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(manifest).expect("manifest serializes"),
        )
        .expect("manifest writes");
        manifest_path
    }

    fn mutate_claim(&mut self, kind: &str, target: &str, mutate: impl FnOnce(&mut Value)) {
        let repository = repository_root();
        let item = self
            .manifest
            .get_mut("items")
            .and_then(Value::as_array_mut)
            .expect("items")
            .iter_mut()
            .find(|item| item["kind"] == kind && item["target"] == target)
            .expect("evidence item");
        let artifact_path = repository.join(
            item["artifact_path"]
                .as_str()
                .expect("artifact path is text"),
        );
        let mut artifact: Value =
            serde_json::from_slice(&fs::read(&artifact_path).expect("artifact can be read"))
                .expect("artifact JSON");
        mutate(artifact.get_mut("claims").expect("claims"));
        let payload_sha256 =
            sha256(&serde_json::to_vec(&artifact["claims"]).expect("claims serialize"));
        artifact["payload_sha256"] = json!(payload_sha256);
        let artifact_bytes = serde_json::to_vec_pretty(&artifact).expect("artifact serializes");
        fs::write(&artifact_path, &artifact_bytes).expect("artifact rewrites");
        item["payload_sha256"] = artifact["payload_sha256"].clone();
        item["artifact_sha256"] = json!(sha256(&artifact_bytes));
    }
}

#[test]
fn complete_manifest_has_stable_human_and_json_ready_reports() {
    // Arrange
    let fixture = Fixture::complete("ready");
    let manifest_path = fixture.write_manifest("manifest.json", &fixture.manifest);

    // Act
    let human = run_audit(&manifest_path, "human");
    let machine = run_audit(&manifest_path, "json");

    // Assert
    assert_success(&human);
    assert_success(&machine);
    let human = String::from_utf8(human.stdout).expect("human output is UTF-8");
    assert!(human.starts_with("release audit: READY\n"));
    assert!(human.contains(&format!("candidate: {CANDIDATE}\nevidence: 19\n")));
    let machine: Value = serde_json::from_slice(&machine.stdout).expect("machine output is JSON");
    assert_eq!(machine["decision"], "ready");
    assert_eq!(machine["candidate_commit"], CANDIDATE);
    assert_eq!(machine["evidence_count"], 19);
    assert_eq!(
        machine["evidence"]
            .as_array()
            .expect("evidence array")
            .len(),
        19
    );
}

#[test]
fn every_required_kind_target_identity_is_independently_mandatory() {
    // Arrange
    let fixture = Fixture::complete("missing-each");
    let items = fixture.manifest["items"].as_array().expect("items").clone();

    // Act / Assert
    for (index, item) in items.iter().enumerate() {
        let mut manifest = fixture.manifest.clone();
        manifest["items"]
            .as_array_mut()
            .expect("items")
            .remove(index);
        let manifest_path = fixture.write_manifest(&format!("missing-{index}.json"), &manifest);
        let output = run_audit(&manifest_path, "json");
        assert_failure_contains(
            &output,
            "release/evidence-set",
            &format!("{}/{}", item["kind"], item["target"]),
        );
    }
}

#[test]
fn manifest_rejects_mixed_candidates_bad_hashes_duplicates_and_unreviewed_items() {
    // Arrange
    let fixture = Fixture::complete("identity-negatives");
    let mut mixed = fixture.manifest.clone();
    mixed["items"][0]["candidate_commit"] = json!(OTHER_CANDIDATE);
    let mut bad_hash = fixture.manifest.clone();
    bad_hash["items"][0]["artifact_sha256"] = json!("0".repeat(64));
    let mut duplicate = fixture.manifest.clone();
    let duplicate_item = duplicate["items"][0].clone();
    duplicate["items"]
        .as_array_mut()
        .expect("items")
        .push(duplicate_item);
    let mut unreviewed = fixture.manifest.clone();
    unreviewed["items"][0]["review_status"] = json!("pending");
    let cases = [
        ("mixed.json", mixed, "release/mixed-candidate"),
        ("bad-hash.json", bad_hash, "release/artifact-hash"),
        ("duplicate.json", duplicate, "release/duplicate-evidence"),
        ("unreviewed.json", unreviewed, "release/evidence-identity"),
    ];

    // Act / Assert
    for (name, manifest, category) in cases {
        let manifest_path = fixture.write_manifest(name, &manifest);
        assert_failure_contains(&run_audit(&manifest_path, "json"), category, name);
    }
}

#[test]
fn manifest_rejects_wrong_producer_workflow_job_and_run_identity() {
    // Arrange
    let fixture = Fixture::complete("producer-negatives");
    let mut wrong_workflow = fixture.manifest.clone();
    wrong_workflow["items"][0]["producer"]["workflow"] = json!("substituted.yml");
    let mut wrong_job = fixture.manifest.clone();
    wrong_job["items"][0]["producer"]["job"] = json!("substituted");
    let mut malformed_run = fixture.manifest.clone();
    malformed_run["items"][0]["producer"]["run_id"] = json!("run-1");
    let cases = [
        ("wrong-workflow.json", wrong_workflow),
        ("wrong-job.json", wrong_job),
        ("malformed-run.json", malformed_run),
    ];

    // Act / Assert
    for (name, manifest) in cases {
        let manifest_path = fixture.write_manifest(name, &manifest);
        assert_failure_contains(
            &run_audit(&manifest_path, "json"),
            "release/evidence-identity",
            name,
        );
    }
}

#[test]
fn manifest_rejects_unknown_kinds_and_the_item_bound() {
    // Arrange
    let fixture = Fixture::complete("closed-schema");
    let mut unknown = fixture.manifest.clone();
    unknown["items"][0]["kind"] = json!("shell_command");
    let mut oversized = fixture.manifest.clone();
    let first = oversized["items"][0].clone();
    let items = oversized["items"].as_array_mut().expect("items");
    while items.len() <= 256 {
        items.push(first.clone());
    }
    let unknown_path = fixture.write_manifest("unknown.json", &unknown);
    let oversized_path = fixture.write_manifest("oversized.json", &oversized);

    // Act / Assert
    assert_failure_contains(
        &run_audit(&unknown_path, "json"),
        "release/manifest-schema",
        "unknown kind",
    );
    assert_failure_contains(
        &run_audit(&oversized_path, "json"),
        "release/manifest",
        "item bound",
    );
}

#[test]
fn conditional_platform_rejects_stale_or_policy_inconsistent_support() {
    // Arrange
    let mut fixture = Fixture::complete("conditional-stale");
    fixture.mutate_claim("conditional_platform", "x86_64-apple-darwin", |claims| {
        claims["disposition"] = json!("supported");
        claims["recorded_at_unix"] = json!(1);
        claims["expires_at_unix"] = json!(2);
    });
    let manifest_path = fixture.write_manifest("manifest.json", &fixture.manifest);

    // Act
    let output = run_audit(&manifest_path, "json");

    // Assert
    assert_failure_contains(
        &output,
        "release/conditional-platform",
        "stale conditional target",
    );
}

#[test]
fn performance_and_coverage_can_never_be_promoted_into_parity_authority() {
    // Arrange
    let mut performance = Fixture::complete("performance-authority");
    performance.mutate_claim("performance", "x86_64-unknown-linux-gnu", |claims| {
        claims["profile_authority"] = json!(true);
    });
    let performance_manifest = performance.write_manifest("manifest.json", &performance.manifest);
    let mut coverage = Fixture::complete("coverage-authority");
    coverage.mutate_claim("rust_coverage", "x86_64-unknown-linux-gnu", |claims| {
        claims["parity_authority"] = json!(true);
    });
    let coverage_manifest = coverage.write_manifest("manifest.json", &coverage.manifest);

    // Act / Assert
    assert_failure_contains(
        &run_audit(&performance_manifest, "json"),
        "release/performance",
        "profile authority",
    );
    assert_failure_contains(
        &run_audit(&coverage_manifest, "json"),
        "release/coverage",
        "coverage parity authority",
    );
}

#[test]
fn unsafe_and_advisory_weakening_fail_closed() {
    // Arrange
    let mut unsafe_fixture = Fixture::complete("unsafe-weakening");
    unsafe_fixture.mutate_claim("rust_safety", "x86_64-unknown-linux-gnu", |claims| {
        claims["unsafe_waivers"] = json!(1);
    });
    let unsafe_manifest = unsafe_fixture.write_manifest("manifest.json", &unsafe_fixture.manifest);
    let mut advisory_fixture = Fixture::complete("advisory-weakening");
    advisory_fixture.mutate_claim("notices", "all", |claims| {
        claims["advisory_waivers"] = json!(1);
    });
    let advisory_manifest =
        advisory_fixture.write_manifest("manifest.json", &advisory_fixture.manifest);

    // Act / Assert
    assert_failure_contains(
        &run_audit(&unsafe_manifest, "json"),
        "release/safety",
        "unsafe weakening",
    );
    assert_failure_contains(
        &run_audit(&advisory_manifest, "json"),
        "release/notices",
        "advisory weakening",
    );
}

#[test]
fn incomplete_corpus_and_compatibility_gaps_reject_readiness() {
    // Arrange
    let mut corpus = Fixture::complete("corpus-gap");
    corpus.mutate_claim("corpus_closure", "all", |claims| {
        claims["unresolved_count"] = json!(1);
    });
    let corpus_manifest = corpus.write_manifest("manifest.json", &corpus.manifest);
    let mut compatibility = Fixture::complete("compatibility-gap");
    compatibility.mutate_claim("compatibility_closure", "all", |claims| {
        claims["gap_count"] = json!(1);
    });
    let compatibility_manifest =
        compatibility.write_manifest("manifest.json", &compatibility.manifest);

    // Act / Assert
    assert_failure_contains(
        &run_audit(&corpus_manifest, "json"),
        "release/corpus-closure",
        "incomplete corpus",
    );
    assert_failure_contains(
        &run_audit(&compatibility_manifest, "json"),
        "release/compatibility-closure",
        "compatibility gap",
    );
}

#[test]
fn package_hash_must_join_every_msrv_and_platform_record() {
    // Arrange
    let mut fixture = Fixture::complete("package-drift");
    fixture.mutate_claim("platform", "aarch64-apple-darwin", |claims| {
        claims["package_sha256"] = json!("3".repeat(64));
    });
    let manifest_path = fixture.write_manifest("manifest.json", &fixture.manifest);

    // Act
    let output = run_audit(&manifest_path, "json");

    // Assert
    assert_failure_contains(&output, "release/package-drift", "package drift");
}

#[test]
fn artifact_payload_hash_is_independently_recomputed() {
    // Arrange
    let repository = repository_root();
    let mut fixture = Fixture::complete("payload-hash");
    let item = fixture.manifest["items"]
        .as_array_mut()
        .expect("items")
        .first_mut()
        .expect("first item");
    let artifact_path = repository.join(
        item["artifact_path"]
            .as_str()
            .expect("artifact path is text"),
    );
    let mut artifact: Value =
        serde_json::from_slice(&fs::read(&artifact_path).expect("artifact can be read"))
            .expect("artifact JSON");
    artifact["payload_sha256"] = json!("0".repeat(64));
    let artifact_bytes = serde_json::to_vec_pretty(&artifact).expect("artifact serializes");
    fs::write(&artifact_path, &artifact_bytes).expect("artifact rewrites");
    item["artifact_sha256"] = json!(sha256(&artifact_bytes));
    let manifest_path = fixture.write_manifest("manifest.json", &fixture.manifest);

    // Act
    let output = run_audit(&manifest_path, "json");

    // Assert
    assert_failure_contains(&output, "release/payload-hash", "payload hash");
}

#[test]
fn missing_manifest_referenced_payload_fails_closed() {
    // Arrange
    let repository = repository_root();
    let fixture = Fixture::complete("missing-payload");
    let artifact_path = repository.join(
        fixture.manifest["items"][0]["artifact_path"]
            .as_str()
            .expect("artifact path is text"),
    );
    fs::remove_file(artifact_path).expect("artifact removes");
    let manifest_path = fixture.write_manifest("manifest.json", &fixture.manifest);

    // Act
    let output = run_audit(&manifest_path, "json");

    // Assert
    assert_failure_contains(&output, "release/artifact-path", "missing payload");
}

#[test]
fn release_constructor_is_check_first_aggregate_only_and_identity_last() {
    // Arrange
    let repository = repository_root();
    let workflow = fs::read_to_string(repository.join(".github/workflows/release.yml"))
        .expect("release workflow");
    let script = fs::read_to_string(repository.join("scripts/phase12-release-evidence.sh"))
        .expect("release constructor");

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

#[test]
fn release_constructor_names_every_exact_artifact_and_never_writes_tracked_readiness() {
    // Arrange
    let repository = repository_root();
    let workflow = fs::read_to_string(repository.join(".github/workflows/release.yml"))
        .expect("release workflow");
    let script = fs::read_to_string(repository.join("scripts/phase12-release-evidence.sh"))
        .expect("release constructor");

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

#[test]
fn release_constructor_rejects_mutated_platform_hash_and_tier() {
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

#[test]
fn release_constructor_rejects_sanitizer_finding() {
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

#[test]
fn release_constructor_rejects_differential_coverage_miss() {
    // Arrange
    let root = producer_fixture_root("coverage-mutation");
    let artifact_path = root.join("differential-leaves.json");
    let artifact = serde_json::to_vec(&json!({
        "schema_version": 1,
        "parity_authority": false,
        "exercised": ["subsystem.world-stepping"],
        "missed": ["subsystem.particle-contacts"],
    }))
    .expect("coverage JSON");
    fs::write(&artifact_path, &artifact).expect("artifact writes");
    let summary_path = root.join("summary.json");
    fs::write(
        &summary_path,
        serde_json::to_vec(&json!({
            "schema_version": 1,
            "evidence_kind": "differential_coverage",
            "candidate_commit": CANDIDATE,
            "toolchain_identity": "semantic-leaf-v1",
            "artifact_path": "differential-leaves.json",
            "artifact_sha256": sha256(&artifact),
            "parity_authority": false,
        }))
        .expect("summary JSON"),
    )
    .expect("summary writes");

    // Act
    let output = run_release_validator(
        "validate_coverage_payload",
        &[
            summary_path.to_string_lossy().into_owned(),
            CANDIDATE.to_owned(),
            "differential_coverage".to_owned(),
        ],
    );

    // Assert
    assert!(!output.status.success());
}

#[test]
fn release_constructor_rejects_failed_regression_result() {
    // Arrange
    let root = producer_fixture_root("regression-mutation");
    let completion = serde_json::to_vec(&json!({
        "schema_version": 1,
        "candidate_sha": CANDIDATE,
        "complete": true,
        "results": [{
            "regression_id": "regression-one",
            "candidate_sha": CANDIDATE,
            "named_test_path": "suite::regression_one",
            "minimized_sha256": "a".repeat(64),
            "outcome": "failed",
        }],
    }))
    .expect("completion JSON");
    fs::write(root.join("completion.json"), &completion).expect("completion writes");
    fs::write(
        root.join("identity.json"),
        serde_json::to_vec(&json!({ "completion_sha256": sha256(&completion) }))
            .expect("validation identity JSON"),
    )
    .expect("validation identity writes");
    let producer_identity_path = root.join("producer-identity.json");
    fs::write(
        &producer_identity_path,
        serde_json::to_vec(&json!({
            "named_test_count": 1,
            "regression_manifest_sha256":
                file_sha256(&repository_root(), "reference/regressions/manifest.toml"),
        }))
        .expect("producer identity JSON"),
    )
    .expect("producer identity writes");

    // Act
    let output = run_release_validator(
        "validate_regression_payload",
        &[
            root.to_string_lossy().into_owned(),
            CANDIDATE.to_owned(),
            producer_identity_path.to_string_lossy().into_owned(),
        ],
    );

    // Assert
    assert!(!output.status.success());
}

#[test]
fn audit_sources_have_no_producer_process_or_network_capability() {
    // Arrange
    let repository = repository_root();
    let sources = [
        "tools/xtask/src/release.rs",
        "tools/xtask/src/release/domain.rs",
        "tools/xtask/src/release/validation.rs",
        "tools/xtask/src/release/report.rs",
    ];
    let forbidden = [
        "Command::new",
        "std::process",
        "reqwest",
        "curl ",
        "gh workflow",
        "cargo fuzz",
        "cargo bench",
    ];

    // Act
    let source = sources
        .iter()
        .map(|relative| {
            fs::read_to_string(repository.join(relative)).expect("release source can be read")
        })
        .collect::<String>();

    // Assert
    for token in forbidden {
        assert!(!source.contains(token), "forbidden producer token: {token}");
    }
}

fn producer_fixture_root(name: &str) -> PathBuf {
    let ordinal = TEST_ORDINAL.fetch_add(1, Ordering::Relaxed);
    let root = repository_root()
        .join("target/xtask-release-tests")
        .join(format!("{name}-{}-{ordinal}", std::process::id()));
    fs::create_dir_all(&root).expect("producer fixture directory");
    root
}

fn run_release_validator(function: &str, arguments: &[String]) -> Output {
    let repository = repository_root();
    let mut command = Command::new("bash");
    command
        .current_dir(&repository)
        .env("PHASE12_RELEASE_EVIDENCE_LIBRARY_ONLY", "1")
        .args([
            "-c",
            "source scripts/phase12-release-evidence.sh; validator=$1; shift; \"$validator\" \"$@\"",
            "phase12-release-validator",
            function,
        ])
        .args(arguments);
    command.output().expect("release validator process")
}

#[test]
fn schema_and_required_registry_are_closed_and_bounded() {
    // Arrange
    let repository = repository_root();
    let schema: Value = serde_json::from_slice(
        &fs::read(repository.join("reference/release/schema.json")).expect("schema"),
    )
    .expect("schema JSON");
    let required: RequiredRegistry = toml::from_str(
        &fs::read_to_string(repository.join("reference/release/required-evidence.toml"))
            .expect("registry"),
    )
    .expect("registry TOML");

    // Act
    let kinds = schema["$defs"]["evidence"]["properties"]["kind"]["enum"]
        .as_array()
        .expect("closed kind enum");

    // Assert
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["properties"]["items"]["maxItems"], 256);
    assert_eq!(schema["$defs"]["evidence"]["additionalProperties"], false);
    assert_eq!(
        schema["$defs"]["evidence"]["properties"]["review_status"]["const"],
        "reviewed"
    );
    assert_eq!(required.evidence.len(), 19);
    assert_eq!(kinds.len(), 16);
}

fn claims_for(
    repository: &Path,
    evidence: &RequiredEvidence,
    archive_path: &Path,
    package_sha256: &str,
) -> Value {
    match evidence.kind.as_str() {
        "package" => json!({
            "package_name": "liquidfun",
            "package_sha256": package_sha256,
            "archive_path": repository_relative(repository, archive_path),
            "archive_sha256": package_sha256,
            "rust_version": "1.92",
            "scalar_mode": "strict_f32",
            "package_drift": false,
        }),
        "msrv" => json!({
            "package_sha256": package_sha256,
            "package_drift": false,
            "rust_version": "1.92",
        }),
        "platform" => json!({
            "package_sha256": package_sha256,
            "package_drift": false,
            "evidence_tier": "d2_supported",
        }),
        "conditional_platform" => json!({
            "package_sha256": package_sha256,
            "package_drift": false,
            "disposition": "unsupported",
            "recorded_at_unix": null,
            "expires_at_unix": null,
        }),
        "canonical_differential" => json!({
            "parity_tier": "d1_canonical",
            "coverage_authority": false,
            "performance_authority": false,
            "gap_count": 0,
        }),
        "rust_safety" => json!({
            "unsafe_waivers": 0,
            "advisory_waivers": 0,
            "unsafe_code": "forbid",
        }),
        "cpp_sanitizer" => json!({ "findings": 0 }),
        "fuzz" => json!({ "findings": 0, "target_count": 5 }),
        "regressions" => json!({
            "manifest_sha256": file_sha256(repository, "reference/regressions/manifest.toml"),
            "missing_results": 0,
            "unreviewed_results": 0,
        }),
        "rust_coverage" | "cpp_coverage" => json!({
            "contract_sha256": file_sha256(repository, "reference/coverage/contract.json"),
            "parity_authority": false,
            "missing_subsystems": 0,
        }),
        "performance" => {
            let manifest: toml::Value = toml::from_str(
                &fs::read_to_string(repository.join("reference/performance/manifest.toml"))
                    .expect("performance manifest"),
            )
            .expect("performance TOML");
            json!({
                "policy_sha256": manifest["policy_sha256"].as_str().expect("policy SHA"),
                "timing_authority": "unprofiled_wall_clock",
                "claim_scope": "workload_only",
                "claim_status": "no_generalized_performance_claim",
                "profile_authority": false,
                "reviewed_report_count": manifest["reviewed_reports"].as_array().expect("reports").len(),
            })
        }
        "docs" => json!({ "docs_complete": true, "rustdoc_warnings": 0 }),
        "notices" => json!({
            "notices_complete": true,
            "license": "MIT",
            "advisory_waivers": 0,
        }),
        "corpus_closure" => {
            let corpus_bytes =
                fs::read(repository.join("reference/upstream-corpus.json")).expect("corpus");
            let corpus: Value = serde_json::from_slice(&corpus_bytes).expect("corpus JSON");
            json!({
                "authority_sha256": sha256(&corpus_bytes),
                "item_count": corpus["items"].as_array().expect("corpus items").len(),
                "unresolved_count": 0,
                "nonterminal_count": 0,
            })
        }
        "compatibility_closure" => json!({
            "authority_sha256": file_sha256(repository, "reference/compatibility.json"),
            "gap_count": 0,
            "unexplained_count": 0,
            "mixed_commit_count": 0,
            "coverage_promoted_to_parity": false,
            "platform_promoted_to_parity": false,
        }),
        kind => panic!("unknown required evidence kind `{kind}`"),
    }
}

fn run_audit(manifest_path: &Path, output: &str) -> Output {
    let repository = repository_root();
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .current_dir(&repository)
        .args([
            "release",
            "audit",
            "--manifest",
            &repository_relative(&repository, manifest_path),
            "--candidate",
            CANDIDATE,
            "--output",
            output,
        ])
        .output()
        .expect("release audit process")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_failure_contains(output: &Output, category: &str, case: &str) {
    assert!(
        !output.status.success(),
        "{case} unexpectedly passed: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(category), "{case}: {stderr}");
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("xtask is nested under tools/")
        .to_path_buf()
}

fn repository_relative(repository: &Path, target_path: &Path) -> String {
    target_path
        .strip_prefix(repository)
        .expect("fixture remains in repository")
        .to_string_lossy()
        .replace('\\', "/")
}

fn file_sha256(repository: &Path, relative: &str) -> String {
    sha256(&fs::read(repository.join(relative)).expect("authority can be read"))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
