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
    manifest_cases::complete_manifest_has_stable_human_and_json_ready_reports();
}

#[test]
fn every_required_kind_target_identity_is_independently_mandatory() {
    manifest_cases::every_required_kind_target_identity_is_independently_mandatory();
}

#[test]
fn manifest_rejects_mixed_candidates_bad_hashes_duplicates_and_unreviewed_items() {
    manifest_cases::manifest_rejects_mixed_candidates_bad_hashes_duplicates_and_unreviewed_items();
}

#[test]
fn manifest_rejects_wrong_producer_workflow_job_and_run_identity() {
    manifest_cases::manifest_rejects_wrong_producer_workflow_job_and_run_identity();
}

#[test]
fn manifest_rejects_unknown_kinds_and_the_item_bound() {
    manifest_cases::manifest_rejects_unknown_kinds_and_the_item_bound();
}

#[test]
fn conditional_platform_rejects_stale_or_policy_inconsistent_support() {
    manifest_cases::conditional_platform_rejects_stale_or_policy_inconsistent_support();
}

#[test]
fn performance_and_coverage_can_never_be_promoted_into_parity_authority() {
    manifest_cases::performance_and_coverage_can_never_be_promoted_into_parity_authority();
}

#[test]
fn unsafe_and_advisory_weakening_fail_closed() {
    manifest_cases::unsafe_and_advisory_weakening_fail_closed();
}

#[test]
fn incomplete_corpus_and_compatibility_gaps_reject_readiness() {
    manifest_cases::incomplete_corpus_and_compatibility_gaps_reject_readiness();
}

#[test]
fn package_hash_must_join_every_msrv_and_platform_record() {
    manifest_cases::package_hash_must_join_every_msrv_and_platform_record();
}

#[test]
fn artifact_payload_hash_is_independently_recomputed() {
    manifest_cases::artifact_payload_hash_is_independently_recomputed();
}

#[test]
fn missing_manifest_referenced_payload_fails_closed() {
    manifest_cases::missing_manifest_referenced_payload_fails_closed();
}

#[test]
fn release_constructor_is_check_first_aggregate_only_and_identity_last() {
    construction_cases::release_constructor_is_check_first_aggregate_only_and_identity_last();
}

#[test]
fn release_constructor_names_every_exact_artifact_and_never_writes_tracked_readiness() {
    construction_cases::release_constructor_names_every_exact_artifact_and_never_writes_tracked_readiness();
}

#[test]
fn release_constructor_rejects_mutated_platform_hash_and_tier() {
    construction_cases::release_constructor_rejects_mutated_platform_hash_and_tier();
}

#[test]
fn release_constructor_rejects_sanitizer_finding() {
    construction_cases::release_constructor_rejects_sanitizer_finding();
}

#[test]
fn release_constructor_rejects_canonical_gap_result() {
    construction_cases::release_constructor_rejects_canonical_gap_result();
}

#[test]
fn release_constructor_accepts_typed_canonical_and_safety_results() {
    construction_cases::release_constructor_accepts_typed_canonical_and_safety_results();
}

#[test]
fn release_constructor_rejects_safety_waiver_policy() {
    construction_cases::release_constructor_rejects_safety_waiver_policy();
}

#[test]
fn release_constructor_rejects_tampered_safety_log() {
    construction_cases::release_constructor_rejects_tampered_safety_log();
}

#[test]
fn release_constructor_rejects_differential_coverage_miss() {
    audit_cases::release_constructor_rejects_differential_coverage_miss();
}

#[test]
fn release_constructor_rejects_failed_regression_result() {
    audit_cases::release_constructor_rejects_failed_regression_result();
}

#[test]
fn audit_sources_have_no_producer_process_or_network_capability() {
    audit_cases::audit_sources_have_no_producer_process_or_network_capability();
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
    audit_cases::schema_and_required_registry_are_closed_and_bounded();
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

#[path = "release_cli/audit_cases.rs"]
mod audit_cases;
#[path = "release_cli/construction_cases.rs"]
mod construction_cases;
#[path = "release_cli/manifest_cases.rs"]
mod manifest_cases;
