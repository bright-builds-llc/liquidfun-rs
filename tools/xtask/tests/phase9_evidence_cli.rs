//! Command-level coverage for the portable Phase 9 evidence validator.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

use liquidfun_differential::{
    NativeRigidWorldExecutor, PHASE9_REQUIRED_POLICY_PATHS, Phase9ComparisonOutcome,
    Phase9CrossRunProof, Phase9CrossRunProofRecord, Phase9EvidenceMismatch,
    Phase9EvidencePayloadRef, compare_complete_phase9_rigid_world_results,
};
use liquidfun_test_protocol::{
    HarnessLimits, Phase9SemanticAssertion, Phase9WitnessBinding, Sha256Hex,
    decode_rigid_world_request_jsonl,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const REJECTED_RUN: u64 = 29_439_515_367;
const SUPERSEDED_RUN: u64 = 29_583_793_056;
const EXACT_RUN: u64 = 30_000_000_001;
const APPROVED_SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const UPSTREAM_REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const PHASE6_POLICY_SHA256: &str =
    "7f10df148852866fd20d11b8d27adcddc0ad463ac3d3d716a8946ca5c8f1c63a";
const PHASE7_POLICY_SHA256: &str =
    "fd772b2cf523a6d40bf978bc4d0da18a4564181a93e6b2bdeb8e4d40d5613311";
const PHASE8_POLICY_SHA256: &str =
    "2843ca40bec5b1c680135664c58c12a8388a7a9e86ad77f8ef5a268f3f15a6bf";

struct TestRoot {
    path: PathBuf,
}

impl TestRoot {
    fn new(label: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = workspace_root()
            .join("target")
            .join(format!("phase9-evidence-cli-{label}-{nonce}"));
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    fn relative(&self, child: &str) -> String {
        self.path
            .join(child)
            .strip_prefix(workspace_root())
            .expect("test fixture remains under the workspace")
            .to_string_lossy()
            .into_owned()
    }

    fn write_valid_local_evidence(&self) -> TestResult {
        let manifest = build_manifest(&self.path)?;
        write_evidence_directory(&self.path.join("canonical"), "canonical-local", &manifest)?;
        write_evidence_directory(&self.path.join("sanitizer"), "sanitizer-local", &manifest)?;
        Ok(())
    }

    fn write_valid_exact_ref_evidence(&self) -> TestResult<Value> {
        let manifest = build_manifest(&self.path)?;
        write_evidence_directory(&self.path.join("canonical"), "canonical-local", &manifest)?;
        write_evidence_directory(&self.path.join("sanitizer"), "sanitizer-local", &manifest)?;
        write_identity_for(
            &self.path.join("canonical"),
            "canonical-linux",
            EXACT_RUN,
            APPROVED_SHA,
        )?;
        write_identity_for(
            &self.path.join("sanitizer"),
            "sanitizer-linux",
            EXACT_RUN,
            APPROVED_SHA,
        )?;
        let canonical_archive = self.path.join("canonical.zip");
        let sanitizer_archive = self.path.join("sanitizer.zip");
        write_zip(&self.path.join("canonical"), &canonical_archive)?;
        write_zip(&self.path.join("sanitizer"), &sanitizer_archive)?;
        let canonical_bytes = fs::read(&canonical_archive)?;
        let sanitizer_bytes = fs::read(&sanitizer_archive)?;
        let canonical_name = format!("phase9-canonical-{EXACT_RUN}-{APPROVED_SHA}");
        let sanitizer_name = format!("phase9-sanitizer-{EXACT_RUN}-{APPROVED_SHA}");
        Ok(json!({
            "repository": "bright-builds-llc/liquidfun-rs",
            "branch": "main",
            "approved_sha": APPROVED_SHA,
            "head_sha": APPROVED_SHA,
            "dispatched_at": "2026-07-17T00:00:00Z",
            "run_id": EXACT_RUN,
            "run_url": "https://example.invalid/run",
            "workflow_name": "Oracle CI",
            "event": "workflow_dispatch",
            "conclusion": "success",
            "created_at": "2026-07-17T00:00:00Z",
            "updated_at": "2026-07-17T00:01:00Z",
            "jobs": {
                "canonical": {
                    "id": 101,
                    "name": "Canonical Linux oracle",
                    "url": "https://example.invalid/job/101",
                    "conclusion": "success"
                },
                "sanitizer": {
                    "id": 102,
                    "name": "Scheduled fail-fast sanitizer and reset corpus",
                    "url": "https://example.invalid/job/102",
                    "conclusion": "success"
                }
            },
            "artifacts": {
                "canonical": exact_artifact(
                    201,
                    &canonical_name,
                    &canonical_archive,
                    &canonical_bytes,
                ),
                "sanitizer": exact_artifact(
                    202,
                    &sanitizer_name,
                    &sanitizer_archive,
                    &sanitizer_bytes,
                )
            },
            "live_run": {
                "id": EXACT_RUN,
                "head_sha": APPROVED_SHA,
                "name": "Oracle CI",
                "event": "workflow_dispatch",
                "conclusion": "success"
            },
            "live_jobs": [
                { "id": 101, "name": "Canonical Linux oracle", "conclusion": "success" },
                { "id": 102, "name": "Scheduled fail-fast sanitizer and reset corpus", "conclusion": "success" }
            ],
            "live_artifacts": [
                { "id": 201, "name": canonical_name, "digest": format!("sha256:{}", sha256(&canonical_bytes)), "expired": false },
                { "id": 202, "name": sanitizer_name, "digest": format!("sha256:{}", sha256(&sanitizer_bytes)), "expired": false }
            ]
        }))
    }

    fn write_run_json(&self, run: &Value) -> TestResult {
        fs::write(self.path.join("run.json"), serde_json::to_vec_pretty(run)?)?;
        Ok(())
    }

    fn run_exact_ref(&self) -> std::io::Result<Output> {
        run_xtask(&[
            "phase9-evidence",
            "validate",
            "--mode",
            "exact-ref",
            "--canonical-dir",
            &self.relative("canonical"),
            "--sanitizer-dir",
            &self.relative("sanitizer"),
            "--run-json",
            &self.relative("run.json"),
            "--deny-run-id",
            &REJECTED_RUN.to_string(),
            "--deny-run-id",
            &SUPERSEDED_RUN.to_string(),
        ])
    }

    fn run_local(&self) -> std::io::Result<Output> {
        run_xtask(&[
            "phase9-evidence",
            "validate",
            "--mode",
            "local",
            "--canonical-dir",
            &self.relative("canonical"),
            "--sanitizer-dir",
            &self.relative("sanitizer"),
            "--deny-run-id",
            &REJECTED_RUN.to_string(),
            "--deny-run-id",
            &SUPERSEDED_RUN.to_string(),
        ])
    }

    fn mutate_json(
        &self,
        directory: &str,
        relative: &str,
        mutate: impl FnOnce(&mut Value),
    ) -> TestResult {
        let path = self.path.join(directory).join(relative);
        let mut value: Value = serde_json::from_slice(&fs::read(&path)?)?;
        mutate(&mut value);
        let mut bytes = serde_json::to_vec_pretty(&value)?;
        bytes.push(b'\n');
        fs::write(&path, bytes)?;
        refresh_identity(&self.path.join(directory))?;
        Ok(())
    }

    fn mutate_manifest_semantics(
        &self,
        directory: &str,
        mutate: impl FnOnce(&mut Value),
    ) -> TestResult {
        let path = self.path.join(directory).join("phase9-manifest.json");
        let mut manifest: Value = serde_json::from_slice(&fs::read(&path)?)?;
        mutate(&mut manifest);
        for case in manifest["cases"].as_array_mut().expect("manifest cases") {
            let witnesses: Vec<Phase9WitnessBinding> =
                serde_json::from_value(case["witnesses"].clone())?;
            case["witness_binding_sha256"] = json!(sha256(&serde_json::to_vec(&witnesses)?));
        }
        let cases: Vec<EvidenceCase> = serde_json::from_value(manifest["cases"].clone())?;
        manifest["semantic_manifest_sha256"] = json!(sha256(&serde_json::to_vec(&cases)?));
        let mut bytes = serde_json::to_vec_pretty(&manifest)?;
        bytes.push(b'\n');
        fs::write(path, bytes)?;
        refresh_identity(&self.path.join(directory))?;
        Ok(())
    }

    fn mutate_case_payload(
        &self,
        directory: &str,
        case_id: &str,
        path_field: &str,
        digest_field: &str,
        mutate: impl FnOnce(&mut Value),
    ) -> TestResult {
        let evidence_root = self.path.join(directory);
        let manifest_path = evidence_root.join("phase9-manifest.json");
        let mut manifest: Value = serde_json::from_slice(&fs::read(&manifest_path)?)?;
        let case = manifest["cases"]
            .as_array_mut()
            .expect("manifest cases")
            .iter_mut()
            .find(|case| case["case_id"] == case_id)
            .expect("reviewed evidence case");
        let relative = case[path_field].as_str().expect("payload path").to_owned();
        let payload_path = evidence_root.join(relative);
        let mut payload: Value = serde_json::from_slice(&fs::read(&payload_path)?)?;
        mutate(&mut payload);
        let bytes = serde_json::to_vec(&payload)?;
        fs::write(&payload_path, &bytes)?;
        case[digest_field] = json!(sha256(&bytes));
        let cases: Vec<EvidenceCase> = serde_json::from_value(manifest["cases"].clone())?;
        manifest["semantic_manifest_sha256"] = json!(sha256(&serde_json::to_vec(&cases)?));
        let mut manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
        manifest_bytes.push(b'\n');
        fs::write(manifest_path, manifest_bytes)?;
        refresh_identity(&evidence_root)?;
        Ok(())
    }

    fn mutate_cross_run_payload(
        &self,
        directory: &str,
        branch_id: &str,
        reference_field: &str,
        mutate: impl FnOnce(&mut Value),
    ) -> TestResult {
        let evidence_root = self.path.join(directory);
        let manifest_path = evidence_root.join("phase9-manifest.json");
        let mut manifest: Value = serde_json::from_slice(&fs::read(&manifest_path)?)?;
        let record = manifest["cases"]
            .as_array()
            .expect("manifest cases")
            .iter()
            .flat_map(|case| case["cross_run_proofs"].as_array().expect("proofs"))
            .find(|record| record["branch_id"] == branch_id)
            .expect("reviewed proof");
        let field = find_object_field(&record["proof"], reference_field);
        let reference = field.get("result").unwrap_or(field);
        let relative = reference["path"].as_str().expect("proof path").to_owned();
        let payload_path = evidence_root.join(&relative);
        let mut payload: Value = serde_json::from_slice(&fs::read(&payload_path)?)?;
        mutate(&mut payload);
        let bytes = serde_json::to_vec(&payload)?;
        fs::write(payload_path, &bytes)?;
        let digest = sha256(&bytes);
        update_payload_reference_digests(&mut manifest, &relative, &digest);
        let cases: Vec<EvidenceCase> = serde_json::from_value(manifest["cases"].clone())?;
        manifest["semantic_manifest_sha256"] = json!(sha256(&serde_json::to_vec(&cases)?));
        let mut manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
        manifest_bytes.push(b'\n');
        fs::write(manifest_path, manifest_bytes)?;
        refresh_identity(&evidence_root)?;
        Ok(())
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn exact_ref_rejects_denylisted_historical_run_before_evidence_access() -> TestResult {
    for run_id in [REJECTED_RUN, SUPERSEDED_RUN] {
        // Arrange
        let root = TestRoot::new("deny-run")?;
        fs::write(
            root.path.join("run.json"),
            format!(r#"{{"run_id":{run_id}}}"#),
        )?;

        // Act
        let output = run_xtask(&[
            "phase9-evidence",
            "validate",
            "--mode",
            "exact-ref",
            "--canonical-dir",
            &root.relative("missing-canonical"),
            "--sanitizer-dir",
            &root.relative("missing-sanitizer"),
            "--run-json",
            &root.relative("run.json"),
            "--deny-run-id",
            &run_id.to_string(),
        ])?;

        // Assert
        assert!(!output.status.success());
        assert_output_contains(&output, "denylisted");
    }
    Ok(())
}

#[test]
fn exact_ref_accepts_closed_run_job_artifact_and_archive_metadata() -> TestResult {
    // Arrange
    let root = TestRoot::new("valid-exact")?;
    let run = root.write_valid_exact_ref_evidence()?;
    root.write_run_json(&run)?;

    // Act
    let output = root.run_exact_ref()?;

    // Assert
    assert_success(&output);
    Ok(())
}

#[test]
fn exact_ref_rejects_wrong_duplicate_and_expired_live_metadata() -> TestResult {
    // Arrange
    let root = TestRoot::new("invalid-exact-metadata")?;
    let valid = root.write_valid_exact_ref_evidence()?;
    let mut wrong_job = valid.clone();
    wrong_job["jobs"]["canonical"]["name"] = json!("wrong");
    root.write_run_json(&wrong_job)?;
    let wrong_job_output = root.run_exact_ref()?;
    let mut duplicate_job = valid.clone();
    let duplicate = duplicate_job["live_jobs"][0].clone();
    duplicate_job["live_jobs"]
        .as_array_mut()
        .expect("live jobs")
        .push(duplicate);
    root.write_run_json(&duplicate_job)?;
    let duplicate_job_output = root.run_exact_ref()?;
    let mut expired = valid;
    expired["artifacts"]["sanitizer"]["expired"] = json!(true);
    expired["live_artifacts"][1]["expired"] = json!(true);
    root.write_run_json(&expired)?;

    // Act
    let expired_output = root.run_exact_ref()?;

    // Assert
    assert_failure(&wrong_job_output);
    assert_failure(&duplicate_job_output);
    assert_failure(&expired_output);
    Ok(())
}

#[test]
#[cfg(unix)]
fn exact_ref_rejects_symlinked_archive_ancestor_without_touching_target() -> TestResult {
    use std::os::unix::fs::symlink;

    // Arrange
    let root = TestRoot::new("archive-symlink")?;
    let mut run = root.write_valid_exact_ref_evidence()?;
    let external = root.path.with_extension("external");
    fs::create_dir_all(&external)?;
    let external_archive = external.join("canonical.zip");
    fs::copy(root.path.join("canonical.zip"), &external_archive)?;
    let marker = external.join("external-marker");
    fs::write(&marker, b"must survive")?;
    let archive_link = root.path.join("archive-link");
    symlink(&external, &archive_link)?;
    run["artifacts"]["canonical"]["archive_path"] = json!(
        archive_link
            .join("canonical.zip")
            .strip_prefix(workspace_root())?
            .to_string_lossy()
    );
    root.write_run_json(&run)?;

    // Act
    let output = root.run_exact_ref()?;

    // Assert
    assert_failure(&output);
    assert_output_contains(&output, "symlink component");
    assert_eq!(fs::read(&marker)?, b"must survive");

    fs::remove_file(archive_link)?;
    fs::remove_dir_all(external)?;
    Ok(())
}

#[test]
fn local_accepts_complete_canonical_and_sanitizer_evidence() -> TestResult {
    // Arrange
    let root = TestRoot::new("valid-local")?;
    root.write_valid_local_evidence()?;

    // Act
    let output = root.run_local()?;

    // Assert
    assert_success(&output);
    assert!(String::from_utf8_lossy(&output.stdout).contains("58 semantic bindings"));
    Ok(())
}

#[test]
fn local_rejects_extra_missing_and_symlink_entries() -> TestResult {
    // Arrange
    let extra = TestRoot::new("extra")?;
    extra.write_valid_local_evidence()?;
    fs::write(extra.path.join("canonical/unexpected.txt"), b"unexpected")?;
    let missing = TestRoot::new("missing")?;
    missing.write_valid_local_evidence()?;
    fs::remove_file(missing.path.join("canonical/inventory.log"))?;
    let symlink = TestRoot::new("symlink")?;
    symlink.write_valid_local_evidence()?;
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        "provenance.log",
        symlink.path.join("canonical/forbidden-link"),
    )?;

    // Act
    let extra_output = extra.run_local()?;
    let missing_output = missing.run_local()?;
    let symlink_output = symlink.run_local()?;

    // Assert
    assert_failure(&extra_output);
    assert_failure(&missing_output);
    assert_failure(&symlink_output);
    Ok(())
}

#[test]
fn local_rejects_failed_logs_and_identity_substitution() -> TestResult {
    // Arrange
    let failed = TestRoot::new("failed-log")?;
    failed.write_valid_local_evidence()?;
    fs::write(
        failed.path.join("canonical/phase9-trace.log"),
        b"test result: FAILED. 0 passed; 1 failed\n",
    )?;
    refresh_identity(&failed.path.join("canonical"))?;
    let substituted = TestRoot::new("substitution")?;
    substituted.write_valid_local_evidence()?;
    let canonical_identity = fs::read(substituted.path.join("canonical/identity.json"))?;
    fs::write(
        substituted.path.join("sanitizer/identity.json"),
        canonical_identity,
    )?;

    // Act
    let failed_output = failed.run_local()?;
    let substituted_output = substituted.run_local()?;

    // Assert
    assert_failure(&failed_output);
    assert_failure(&substituted_output);
    Ok(())
}

#[test]
fn local_rejects_retained_policy_witness_and_payload_corruption() -> TestResult {
    // Arrange
    let retained = TestRoot::new("retained")?;
    retained.write_valid_local_evidence()?;
    retained.mutate_json("canonical", "phase9-manifest.json", |manifest| {
        manifest["cases"][0]["retained_rigid"]["phase8_policy_sha256"] = json!("0".repeat(64));
    })?;
    let witness = TestRoot::new("witness")?;
    witness.write_valid_local_evidence()?;
    witness.mutate_json("canonical", "phase9-manifest.json", |manifest| {
        manifest["cases"][0]["witnesses"][0]["action_index"] = json!(usize::MAX);
    })?;
    let payload = TestRoot::new("payload")?;
    payload.write_valid_local_evidence()?;
    let native_path = payload
        .path
        .join("canonical/cases/storage-systems-and-permutations/native-result.json");
    fs::write(native_path, b"{}")?;
    refresh_identity(&payload.path.join("canonical"))?;

    // Act
    let retained_output = retained.run_local()?;
    let witness_output = witness.run_local()?;
    let payload_output = payload.run_local()?;

    // Assert
    assert_failure(&retained_output);
    assert_failure(&witness_output);
    assert_failure(&payload_output);
    Ok(())
}

#[test]
fn local_rejects_incomplete_policies_and_semantic_manifest_disagreement() -> TestResult {
    // Arrange
    let policies = TestRoot::new("policies")?;
    policies.write_valid_local_evidence()?;
    policies.mutate_json("canonical", "phase9-manifest.json", |manifest| {
        manifest["cases"][0]["consumed_policy_paths"]
            .as_array_mut()
            .expect("policy array")
            .pop();
    })?;
    let disagreement = TestRoot::new("disagreement")?;
    disagreement.write_valid_local_evidence()?;
    disagreement.mutate_json("sanitizer", "phase9-manifest.json", |manifest| {
        manifest["cases"].as_array_mut().expect("cases").swap(0, 1);
        manifest["semantic_manifest_sha256"] = json!(sha256(
            &serde_json::to_vec(&manifest["cases"]).expect("cases bytes")
        ));
    })?;

    // Act
    let policy_output = policies.run_local()?;
    let disagreement_output = disagreement.run_local()?;

    // Assert
    assert_failure(&policy_output);
    assert_failure(&disagreement_output);
    Ok(())
}

#[test]
fn local_rejects_zero_energy_and_empty_stuck_witnesses() -> TestResult {
    // Arrange
    let zero = TestRoot::new("zero-energy")?;
    zero.write_valid_local_evidence()?;
    zero.mutate_manifest_semantics("canonical", |manifest| {
        let binding = find_binding_mut(manifest, "collision_energy");
        binding["semantic_assertion"]["minimum_bits"] = json!(0);
    })?;
    let empty = TestRoot::new("empty-stuck")?;
    empty.write_valid_local_evidence()?;
    empty.mutate_manifest_semantics("canonical", |manifest| {
        let binding = find_binding_mut(manifest, "stuck_candidates");
        binding["semantic_assertion"]["particle_ids"] = json!([]);
    })?;

    // Act
    let zero_output = zero.run_local()?;
    let empty_output = empty.run_local()?;

    // Assert
    assert_failure(&zero_output);
    assert_failure(&empty_output);
    assert_output_contains(&zero_output, "bindings");
    assert_output_contains(&empty_output, "bindings");
    Ok(())
}

#[test]
fn local_rejects_digest_recomputed_in_range_binding_mutations() -> TestResult {
    // Arrange
    let wrong_action = TestRoot::new("wrong-action")?;
    wrong_action.write_valid_local_evidence()?;
    wrong_action.mutate_manifest_semantics("canonical", |manifest| {
        find_binding_mut(manifest, "stable_ids_sort")["action_index"] = json!(9);
    })?;
    let wrong_checkpoint = TestRoot::new("wrong-checkpoint")?;
    wrong_checkpoint.write_valid_local_evidence()?;
    wrong_checkpoint.mutate_manifest_semantics("canonical", |manifest| {
        find_binding_mut(manifest, "optional_lanes")["checkpoint_index"] = json!(0);
    })?;
    let wrong_observation = TestRoot::new("wrong-observation")?;
    wrong_observation.write_valid_local_evidence()?;
    wrong_observation.mutate_case_payload(
        "canonical",
        "storage-systems-and-permutations",
        "native_result_path",
        "native_result_sha256",
        |result| {
            let particle =
                result["timelines"][0]["checkpoints"][1]["observations"][0]["observation"].clone();
            result["timelines"][0]["checkpoints"][0]["observations"][8]["observation"] = particle;
        },
    )?;

    // Act
    let wrong_action_output = wrong_action.run_local()?;
    let wrong_checkpoint_output = wrong_checkpoint.run_local()?;
    let wrong_observation_output = wrong_observation.run_local()?;

    // Assert
    for output in [
        &wrong_action_output,
        &wrong_checkpoint_output,
        &wrong_observation_output,
    ] {
        assert_failure(output);
    }
    assert_output_contains(&wrong_action_output, "expected action");
    assert_output_contains(&wrong_checkpoint_output, "selected checkpoint");
    Ok(())
}

#[test]
fn local_rejects_digest_recomputed_false_semantic_assertions() -> TestResult {
    for (label, branch, mutate) in [
        (
            "false-lifetime",
            "finite_lifetime",
            ("particle_id", json!("phase9-b")),
        ),
        (
            "false-contact",
            "strict_contact_enabled",
            ("contact_count", json!(3)),
        ),
        (
            "false-listener",
            "listener_flag_enabled",
            ("event_count", json!(2)),
        ),
        (
            "false-filter",
            "filter_flag_disabled",
            ("contact_count", json!(0)),
        ),
    ] {
        // Arrange
        let root = TestRoot::new(label)?;
        root.write_valid_local_evidence()?;
        root.mutate_manifest_semantics("canonical", |manifest| {
            find_binding_mut(manifest, branch)["semantic_assertion"][mutate.0] = mutate.1;
        })?;

        // Act
        let output = root.run_local()?;

        // Assert
        assert_failure(&output);
        assert_output_contains(&output, "semantic assertion");
    }
    Ok(())
}

#[test]
fn local_rejects_digest_recomputed_cross_run_proof_mutations() -> TestResult {
    for (label, branch, reference_field) in [
        ("false-replay", "replay_identity", "replay_native"),
        ("false-minimization", "minimization_identity", "copied"),
        (
            "false-first-divergence",
            "first_divergence_stability",
            "minimized",
        ),
        ("false-d0", "d0_byte_identity", "repeated_native"),
        (
            "false-debug-release",
            "debug_release_agreement",
            "release_oracle",
        ),
    ] {
        // Arrange
        let root = TestRoot::new(label)?;
        root.write_valid_local_evidence()?;
        root.mutate_cross_run_payload("canonical", branch, reference_field, |result| {
            let body = first_result_member_mut(result, "bodies");
            body["active"] = json!(!body["active"].as_bool().expect("body active"));
        })?;

        // Act
        let output = root.run_local()?;

        // Assert
        assert_failure(&output);
        assert_output_contains(&output, "cross-run");
    }
    Ok(())
}

#[test]
fn local_recomputes_comparator_instead_of_trusting_match_payload() -> TestResult {
    // Arrange
    let root = TestRoot::new("divergent-pair")?;
    root.write_valid_local_evidence()?;
    root.mutate_case_payload(
        "canonical",
        "closed-evidence-contract",
        "oracle_result_path",
        "oracle_result_sha256",
        |result| {
            let body = result["timelines"]
                .as_array_mut()
                .expect("timelines")
                .iter_mut()
                .flat_map(|timeline| timeline["checkpoints"].as_array_mut().expect("checkpoints"))
                .find_map(|checkpoint| checkpoint["bodies"].as_array_mut()?.first_mut())
                .expect("retained body");
            body["active"] = json!(!body["active"].as_bool().expect("body active"));
        },
    )?;

    // Act
    let output = root.run_local()?;

    // Assert
    assert_failure(&output);
    assert_output_contains(&output, "persisted divergent native and oracle results");
    Ok(())
}

#[derive(Deserialize, Serialize)]
struct EvidenceManifest {
    schema_version: u32,
    case_record_schema_version: u32,
    profile: String,
    upstream_revision: String,
    semantic_manifest_sha256: String,
    cases: Vec<EvidenceCase>,
}

#[derive(Deserialize, Serialize)]
struct EvidenceCase {
    case_id: String,
    reached_branches: Vec<String>,
    witnesses: Vec<Phase9WitnessBinding>,
    witness_binding_sha256: String,
    consumed_policy_paths: Vec<String>,
    retained_rigid: RetainedRigid,
    request_path: String,
    request_sha256: String,
    native_result_path: String,
    native_result_sha256: String,
    oracle_result_path: String,
    oracle_result_sha256: String,
    complete_comparison_path: String,
    complete_comparison_sha256: String,
    cross_run_proofs: Vec<Phase9CrossRunProofRecord>,
}

#[derive(Deserialize, Serialize)]
struct RetainedRigid {
    comparator: String,
    phase6_policy_sha256: String,
    phase7_policy_sha256: String,
    phase8_policy_sha256: String,
    outcome: String,
    comparison_sha256: String,
}

#[derive(Serialize)]
struct RetainedPayload<'a> {
    comparator: &'a str,
    phase6_policy_sha256: &'a str,
    phase7_policy_sha256: &'a str,
    phase8_policy_sha256: &'a str,
    outcome: &'a str,
}

#[allow(clippy::too_many_arguments)]
fn synthetic_cross_run_proofs(
    root: &Path,
    base: &str,
    request_record: &liquidfun_test_protocol::RigidWorldRequestRecord,
    native_record: &liquidfun_test_protocol::RigidWorldResultRecord,
    request: &[u8],
    native: &[u8],
    oracle: &[u8],
    witnesses: &[Phase9WitnessBinding],
) -> TestResult<Vec<Phase9CrossRunProofRecord>> {
    let case_witnesses = witnesses
        .iter()
        .filter(|witness| witness.semantic_assertion.requires_case_evidence())
        .collect::<Vec<_>>();
    if case_witnesses.is_empty() {
        return Ok(Vec::new());
    }
    let mut minimized_value = serde_json::to_value(native_record)?;
    let body = first_result_member_mut(&mut minimized_value, "bodies");
    body["active"] = json!(!body["active"].as_bool().expect("body active"));
    let minimized_record = serde_json::from_value(minimized_value)?;
    let mut copied_value = serde_json::to_value(native_record)?;
    let body = first_result_member_mut(&mut copied_value, "bodies");
    body["active"] = json!(!body["active"].as_bool().expect("body active"));
    let fixture = first_result_member_mut(&mut copied_value, "fixtures");
    fixture["sensor"] = json!(!fixture["sensor"].as_bool().expect("fixture sensor"));
    let copied_record = serde_json::from_value(copied_value)?;
    let minimized_report = retained_mismatch(request_record, native_record, &minimized_record);
    let copied_report = retained_mismatch(request_record, native_record, &copied_record);
    let payloads = [
        ("replay-native.json", native.to_vec()),
        ("replay-oracle.json", oracle.to_vec()),
        ("minimized.json", serde_json::to_vec(&minimized_record)?),
        ("copied.json", serde_json::to_vec(&copied_record)?),
        ("debug.json", oracle.to_vec()),
        ("release.json", oracle.to_vec()),
    ];
    let mut references = std::collections::BTreeMap::new();
    for (name, bytes) in payloads {
        let path = format!("{base}/proofs/{name}");
        write_payload(root, &path, &bytes)?;
        references.insert(
            name,
            Phase9EvidencePayloadRef {
                path: path.into(),
                sha256: digest(&bytes),
            },
        );
    }
    let mismatch = |name: &str,
                    report: &liquidfun_differential::RigidMismatchReport|
     -> Phase9EvidenceMismatch {
        Phase9EvidenceMismatch {
            result: references.get(name).expect("proof reference").clone(),
            signature_sha256: report.signature().signature_sha256().clone(),
            semantic_path: report.semantic_path().into(),
        }
    };
    let records = case_witnesses
        .into_iter()
        .map(|witness| {
            let proof = match &witness.semantic_assertion {
                Phase9SemanticAssertion::ReplayResultDigestEquality => {
                    Phase9CrossRunProof::ReplayResultDigestEquality {
                        replay_native: references["replay-native.json"].clone(),
                        replay_oracle: references["replay-oracle.json"].clone(),
                    }
                }
                Phase9SemanticAssertion::MinimizedFailureSignaturePreservation => {
                    Phase9CrossRunProof::MinimizedFailureSignaturePreservation {
                        minimized: mismatch("minimized.json", &minimized_report),
                        copied: mismatch("copied.json", &copied_report),
                    }
                }
                Phase9SemanticAssertion::DeliberateFirstDivergence => {
                    Phase9CrossRunProof::DeliberateFirstDivergence {
                        minimized: mismatch("minimized.json", &minimized_report),
                        copied: mismatch("copied.json", &copied_report),
                    }
                }
                Phase9SemanticAssertion::D0RepeatedResultDigestEquality => {
                    Phase9CrossRunProof::D0RepeatedResultDigestEquality {
                        repeated_native: references["replay-native.json"].clone(),
                        repeated_oracle: references["replay-oracle.json"].clone(),
                    }
                }
                Phase9SemanticAssertion::DebugReleaseResultDigestEquality => {
                    Phase9CrossRunProof::DebugReleaseResultDigestEquality {
                        debug_oracle: references["debug.json"].clone(),
                        release_oracle: references["release.json"].clone(),
                    }
                }
                _ => unreachable!("filtered case evidence"),
            };
            Phase9CrossRunProofRecord {
                branch_id: witness.branch_id.clone(),
                request_sha256: digest(request),
                native_result_sha256: digest(native),
                oracle_result_sha256: digest(oracle),
                proof,
            }
        })
        .collect();
    Ok(records)
}

#[test]
fn proof_topology_accepts_canonical_paths_and_reviewed_reuse() -> TestResult {
    // Arrange
    let root = TestRoot::new("proof-topology-valid")?;
    let manifest = build_manifest(&root.path)?;
    let case = evidence_case(&manifest, "closed-evidence-contract");

    // Act
    let result =
        Phase9CrossRunProofRecord::validate_topology(&case.case_id, &case.cross_run_proofs);

    // Assert
    assert_eq!(result, Ok(()));
    Ok(())
}

#[test]
fn proof_topology_rejects_baseline_and_required_pair_aliases() -> TestResult {
    for (label, branch, field, path, expected) in [
        (
            "baseline",
            "replay_identity",
            "replay_native",
            "cases/closed-evidence-contract/native-result.json",
            "replay-native",
        ),
        (
            "replay-alias",
            "replay_identity",
            "replay_oracle",
            "cases/closed-evidence-contract/proofs/replay-native.json",
            "replay-oracle",
        ),
        (
            "debug-release-alias",
            "debug_release_agreement",
            "release_oracle",
            "cases/closed-evidence-contract/proofs/debug.json",
            "release",
        ),
        (
            "minimized-copied-alias",
            "minimization_identity",
            "copied",
            "cases/closed-evidence-contract/proofs/minimized.json",
            "copied",
        ),
    ] {
        // Arrange
        let root = TestRoot::new(label)?;
        let manifest = build_manifest(&root.path)?;
        let case = evidence_case(&manifest, "closed-evidence-contract");
        let mut records = case.cross_run_proofs.clone();
        set_proof_path(&mut records, branch, field, path)?;

        // Act
        let error = Phase9CrossRunProofRecord::validate_topology(&case.case_id, &records)
            .expect_err("forbidden topology must fail");

        // Assert
        assert!(
            error.to_string().contains(expected),
            "unexpected topology error: {error}"
        );
    }
    Ok(())
}

#[test]
fn proof_topology_rejects_noncanonical_path_spellings() -> TestResult {
    for (label, path) in [
        ("wrong-case", "cases/other-case/proofs/replay-native.json"),
        (
            "dot-component",
            "cases/closed-evidence-contract/./proofs/replay-native.json",
        ),
        (
            "duplicate-separator",
            "cases/closed-evidence-contract//proofs/replay-native.json",
        ),
        (
            "backslash",
            r"cases\closed-evidence-contract\proofs\replay-native.json",
        ),
        (
            "parent-traversal",
            "cases/closed-evidence-contract/proofs/../replay-native.json",
        ),
        ("absolute", "/tmp/replay-native.json"),
        ("drive-absolute", r"C:\tmp\replay-native.json"),
    ] {
        // Arrange
        let root = TestRoot::new(label)?;
        let manifest = build_manifest(&root.path)?;
        let case = evidence_case(&manifest, "closed-evidence-contract");
        let mut records = case.cross_run_proofs.clone();
        set_proof_path(&mut records, "replay_identity", "replay_native", path)?;

        // Act
        let result = Phase9CrossRunProofRecord::validate_topology(&case.case_id, &records);

        // Assert
        assert!(result.is_err(), "{label} unexpectedly passed");
    }
    Ok(())
}

fn evidence_case<'a>(manifest: &'a EvidenceManifest, case_id: &str) -> &'a EvidenceCase {
    manifest
        .cases
        .iter()
        .find(|case| case.case_id == case_id)
        .expect("reviewed evidence case")
}

fn set_proof_path(
    records: &mut [Phase9CrossRunProofRecord],
    branch_id: &str,
    field: &str,
    path: &str,
) -> TestResult {
    let record = records
        .iter_mut()
        .find(|record| record.branch_id.as_str() == branch_id)
        .expect("reviewed proof record");
    let mut value = serde_json::to_value(&*record)?;
    let mut reference =
        find_object_field_mut(&mut value["proof"], field).expect("reviewed proof reference field");
    if reference.get("result").is_some() {
        reference = reference
            .get_mut("result")
            .expect("mismatch reference result");
    }
    reference["path"] = json!(path);
    *record = serde_json::from_value(value)?;
    Ok(())
}

fn first_result_member_mut<'a>(value: &'a mut Value, member: &str) -> &'a mut Value {
    value["timelines"]
        .as_array_mut()
        .expect("timelines")
        .iter_mut()
        .flat_map(|timeline| timeline["checkpoints"].as_array_mut().expect("checkpoints"))
        .find_map(|checkpoint| checkpoint[member].as_array_mut()?.first_mut())
        .expect("retained result member")
}

fn retained_mismatch(
    request: &liquidfun_test_protocol::RigidWorldRequestRecord,
    native: &liquidfun_test_protocol::RigidWorldResultRecord,
    mutated: &liquidfun_test_protocol::RigidWorldResultRecord,
) -> Box<liquidfun_differential::RigidMismatchReport> {
    let outcome = compare_complete_phase9_rigid_world_results(request, native, mutated)
        .expect("synthetic mismatch comparison");
    let Phase9ComparisonOutcome::RetainedRigidMismatch(report) = outcome else {
        panic!("synthetic result must produce retained mismatch");
    };
    report
}

fn digest(bytes: &[u8]) -> Sha256Hex {
    Sha256Hex::new(sha256(bytes)).expect("computed digest")
}

fn build_manifest(root: &Path) -> TestResult<EvidenceManifest> {
    let source: Value = serde_json::from_slice(&fs::read(
        workspace_root()
            .join("crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/phase9-v1.json"),
    )?)?;
    let source_cases = source["cases"].as_array().expect("source cases");
    let policies = PHASE9_REQUIRED_POLICY_PATHS
        .iter()
        .map(|path| (*path).to_owned())
        .collect::<Vec<_>>();
    let retained_payload = RetainedPayload {
        comparator: "phase8-v1",
        phase6_policy_sha256: PHASE6_POLICY_SHA256,
        phase7_policy_sha256: PHASE7_POLICY_SHA256,
        phase8_policy_sha256: PHASE8_POLICY_SHA256,
        outcome: "match",
    };
    let mut cases = Vec::new();
    for source_case in source_cases {
        let case_id = source_case["case_id"].as_str().expect("case ID").to_owned();
        let fixture = source_case["fixture"].as_str().expect("fixture path");
        let request = fs::read(
            workspace_root()
                .join("crates/liquidfun-differential/tests/fixtures/rigid_world/phase9")
                .join(fixture),
        )?;
        let decoded =
            decode_rigid_world_request_jsonl(&request, &HarnessLimits::phase2_default_v1())?;
        let result = NativeRigidWorldExecutor::execute(&decoded)?;
        let native = serde_json::to_vec(&result)?;
        let oracle = native.clone();
        let comparison = serde_json::to_vec(&json!({
            "outcome": "match",
            "consumed_policy_paths": policies.clone(),
        }))?;
        let witnesses: Vec<Phase9WitnessBinding> =
            serde_json::from_value(source_case["witnesses"].clone())?;
        let reached_branches = witnesses
            .iter()
            .map(|witness| witness.branch_id.as_str().to_owned())
            .collect::<Vec<_>>();
        let base = format!("cases/{case_id}");
        write_payload(root, &format!("{base}/request.jsonl"), &request)?;
        write_payload(root, &format!("{base}/native-result.json"), &native)?;
        write_payload(root, &format!("{base}/oracle-result.json"), &oracle)?;
        write_payload(
            root,
            &format!("{base}/complete-comparison.json"),
            &comparison,
        )?;
        let cross_run_proofs = synthetic_cross_run_proofs(
            root, &base, &decoded, &result, &request, &native, &oracle, &witnesses,
        )?;
        cases.push(EvidenceCase {
            case_id,
            reached_branches,
            witness_binding_sha256: sha256(&serde_json::to_vec(&witnesses)?),
            witnesses,
            consumed_policy_paths: policies.clone(),
            retained_rigid: RetainedRigid {
                comparator: "phase8-v1".to_owned(),
                phase6_policy_sha256: PHASE6_POLICY_SHA256.to_owned(),
                phase7_policy_sha256: PHASE7_POLICY_SHA256.to_owned(),
                phase8_policy_sha256: PHASE8_POLICY_SHA256.to_owned(),
                outcome: "match".to_owned(),
                comparison_sha256: sha256(&serde_json::to_vec(&retained_payload)?),
            },
            request_path: format!("{base}/request.jsonl"),
            request_sha256: sha256(&request),
            native_result_path: format!("{base}/native-result.json"),
            native_result_sha256: sha256(&native),
            oracle_result_path: format!("{base}/oracle-result.json"),
            oracle_result_sha256: sha256(&oracle),
            complete_comparison_path: format!("{base}/complete-comparison.json"),
            complete_comparison_sha256: sha256(&comparison),
            cross_run_proofs,
        });
    }
    Ok(EvidenceManifest {
        schema_version: 3,
        case_record_schema_version: 2,
        profile: "phase9-v1".to_owned(),
        upstream_revision: UPSTREAM_REVISION.to_owned(),
        semantic_manifest_sha256: sha256(&serde_json::to_vec(&cases)?),
        cases,
    })
}

fn write_evidence_directory(root: &Path, job: &str, manifest: &EvidenceManifest) -> TestResult {
    fs::create_dir_all(root)?;
    let source_payloads = root.parent().expect("fixture root").join("cases");
    copy_directory(&source_payloads, &root.join("cases"))?;
    let mut manifest_bytes = serde_json::to_vec_pretty(manifest)?;
    manifest_bytes.push(b'\n');
    fs::write(root.join("phase9-manifest.json"), manifest_bytes)?;
    fs::write(
        root.join("phase9-trace.log"),
        b"test result: ok. 25 passed; 0 failed; 1 ignored\n",
    )?;
    fs::write(root.join("provenance.log"), b"provenance verified\n")?;
    fs::write(root.join("inventory.log"), b"inventory verified\n")?;
    fs::write(root.join("read-only.log"), b"")?;
    write_identity(root, job)
}

fn write_identity(root: &Path, job: &str) -> TestResult {
    write_identity_for(root, job, 0, "local")
}

fn write_identity_for(root: &Path, job: &str, run_id: u64, head_sha: &str) -> TestResult {
    let files = regular_files(root)?
        .into_iter()
        .filter(|path| path != "identity.json")
        .map(|path| {
            Ok(json!({
                "path": path,
                "sha256": sha256(&fs::read(root.join(&path))?),
            }))
        })
        .collect::<TestResult<Vec<_>>>()?;
    let identity = json!({
        "run_id": run_id,
        "job": job,
        "head_sha": head_sha,
        "upstream_revision": UPSTREAM_REVISION,
        "rust": "1.97.0",
        "cmake": "4.3.3",
        "ninja": "1.13.2",
        "clang": "22.1.8",
        "target": "x86_64-unknown-linux-gnu",
        "policy": "phase9-v1",
        "trace": {
            "path": "phase9-trace.log",
            "sha256": sha256(&fs::read(root.join("phase9-trace.log"))?),
        },
        "manifest": {
            "path": "phase9-manifest.json",
            "sha256": sha256(&fs::read(root.join("phase9-manifest.json"))?),
        },
        "files": files,
    });
    let mut bytes = serde_json::to_vec_pretty(&identity)?;
    bytes.push(b'\n');
    fs::write(root.join("identity.json"), bytes)?;
    Ok(())
}

fn exact_artifact(id: u64, name: &str, archive: &Path, bytes: &[u8]) -> Value {
    json!({
        "id": id,
        "name": name,
        "api_url": "https://example.invalid/artifact",
        "archive_download_url": "https://example.invalid/artifact.zip",
        "digest": format!("sha256:{}", sha256(bytes)),
        "size_in_bytes": bytes.len(),
        "expired": false,
        "created_at": "2026-07-17T00:00:00Z",
        "expires_at": "2026-10-15T00:00:00Z",
        "archive_path": archive
            .strip_prefix(workspace_root())
            .expect("archive remains under workspace")
            .to_string_lossy(),
    })
}

fn write_zip(source: &Path, archive: &Path) -> TestResult {
    let files = regular_files(source)?;
    let status = Command::new("zip")
        .arg("-q")
        .arg(archive)
        .args(files)
        .current_dir(source)
        .status()?;
    if !status.success() {
        return Err("zip failed while constructing exact-ref fixture".into());
    }
    Ok(())
}

fn refresh_identity(root: &Path) -> TestResult {
    let identity: Value = serde_json::from_slice(&fs::read(root.join("identity.json"))?)?;
    let job = identity["job"].as_str().expect("identity job").to_owned();
    write_identity(root, &job)
}

fn regular_files(root: &Path) -> TestResult<BTreeSet<String>> {
    let mut pending = vec![(root.to_path_buf(), PathBuf::new())];
    let mut files = BTreeSet::new();
    while let Some((directory, relative)) = pending.pop() {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let child_relative = relative.join(entry.file_name());
            if entry.file_type()?.is_dir() {
                pending.push((entry.path(), child_relative));
            } else if entry.file_type()?.is_file() {
                files.insert(child_relative.to_string_lossy().into_owned());
            }
        }
    }
    Ok(files)
}

fn copy_directory(source: &Path, destination: &Path) -> TestResult {
    for (relative, _) in regular_files(source)?
        .into_iter()
        .map(|relative| (relative.clone(), source.join(relative)))
    {
        let target = destination.join(&relative);
        fs::create_dir_all(target.parent().expect("payload parent"))?;
        fs::copy(source.join(relative), target)?;
    }
    Ok(())
}

fn write_payload(root: &Path, relative: &str, bytes: &[u8]) -> TestResult {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("payload parent"))?;
    fs::write(path, bytes)?;
    Ok(())
}

fn find_binding_mut<'a>(manifest: &'a mut Value, branch_id: &str) -> &'a mut Value {
    manifest["cases"]
        .as_array_mut()
        .expect("manifest cases")
        .iter_mut()
        .flat_map(|case| {
            case["witnesses"]
                .as_array_mut()
                .expect("case witnesses")
                .iter_mut()
        })
        .find(|binding| binding["branch_id"] == branch_id)
        .expect("reviewed branch binding")
}

fn find_object_field<'a>(value: &'a Value, field: &str) -> &'a Value {
    find_object_field_maybe(value, field).expect("proof reference field")
}

fn find_object_field_maybe<'a>(value: &'a Value, field: &str) -> Option<&'a Value> {
    if let Some(found) = value.get(field) {
        return Some(found);
    }
    value
        .as_object()
        .into_iter()
        .flat_map(|object| object.values())
        .find_map(|child| find_object_field_maybe(child, field))
}

fn find_object_field_mut<'a>(value: &'a mut Value, field: &str) -> Option<&'a mut Value> {
    if value.get(field).is_some() {
        return value.get_mut(field);
    }
    value
        .as_object_mut()
        .into_iter()
        .flat_map(|object| object.values_mut())
        .find_map(|child| find_object_field_mut(child, field))
}

fn update_payload_reference_digests(value: &mut Value, path: &str, digest: &str) {
    match value {
        Value::Array(values) => {
            for value in values {
                update_payload_reference_digests(value, path, digest);
            }
        }
        Value::Object(object) => {
            if object.get("path").and_then(Value::as_str) == Some(path) {
                object.insert("sha256".to_owned(), json!(digest));
            }
            for value in object.values_mut() {
                update_payload_reference_digests(value, path, digest);
            }
        }
        _ => {}
    }
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn run_xtask(args: &[&str]) -> std::io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .current_dir(workspace_root())
        .output()
}

fn assert_output_contains(output: &Output, needle: &str) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(needle),
        "stderr did not contain `{needle}`:\n{stderr}"
    );
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_failure(output: &Output) {
    assert!(
        !output.status.success(),
        "command unexpectedly succeeded:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("xtask belongs to the workspace")
}
