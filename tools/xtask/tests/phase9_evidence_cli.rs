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
    evidence_cases::exact_ref_rejects_denylisted_historical_run_before_evidence_access()
}

#[test]
fn exact_ref_accepts_closed_run_job_artifact_and_archive_metadata() -> TestResult {
    evidence_cases::exact_ref_accepts_closed_run_job_artifact_and_archive_metadata()
}

#[test]
fn exact_ref_rejects_wrong_duplicate_and_expired_live_metadata() -> TestResult {
    evidence_cases::exact_ref_rejects_wrong_duplicate_and_expired_live_metadata()
}

#[test]
#[cfg(unix)]
fn exact_ref_rejects_symlinked_archive_ancestor_without_touching_target() -> TestResult {
    evidence_cases::exact_ref_rejects_symlinked_archive_ancestor_without_touching_target()
}

#[test]
fn local_accepts_complete_canonical_and_sanitizer_evidence() -> TestResult {
    evidence_cases::local_accepts_complete_canonical_and_sanitizer_evidence()
}

#[test]
fn local_rejects_schema_v3_with_regeneration_guidance() -> TestResult {
    evidence_cases::local_rejects_schema_v3_with_regeneration_guidance()
}

#[test]
fn local_rejects_extra_missing_and_symlink_entries() -> TestResult {
    evidence_cases::local_rejects_extra_missing_and_symlink_entries()
}

#[test]
fn local_rejects_failed_logs_and_identity_substitution() -> TestResult {
    evidence_cases::local_rejects_failed_logs_and_identity_substitution()
}

#[test]
fn local_rejects_retained_policy_witness_and_payload_corruption() -> TestResult {
    evidence_cases::local_rejects_retained_policy_witness_and_payload_corruption()
}

#[test]
fn local_rejects_incomplete_policies_and_semantic_manifest_disagreement() -> TestResult {
    evidence_cases::local_rejects_incomplete_policies_and_semantic_manifest_disagreement()
}

#[test]
fn local_rejects_zero_energy_and_empty_stuck_witnesses() -> TestResult {
    evidence_cases::local_rejects_zero_energy_and_empty_stuck_witnesses()
}

#[test]
fn local_rejects_digest_recomputed_in_range_binding_mutations() -> TestResult {
    evidence_cases::local_rejects_digest_recomputed_in_range_binding_mutations()
}

#[test]
fn local_rejects_digest_recomputed_false_semantic_assertions() -> TestResult {
    evidence_cases::local_rejects_digest_recomputed_false_semantic_assertions()
}

#[test]
fn local_rejects_digest_recomputed_cross_run_proof_mutations() -> TestResult {
    evidence_cases::local_rejects_digest_recomputed_cross_run_proof_mutations()
}

#[test]
fn local_recomputes_comparator_instead_of_trusting_match_payload() -> TestResult {
    evidence_cases::local_recomputes_comparator_instead_of_trusting_match_payload()
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

#[test]
fn proof_topology_accepts_canonical_paths_and_reviewed_reuse() -> TestResult {
    proof_topology_cases::proof_topology_accepts_canonical_paths_and_reviewed_reuse()
}

#[test]
fn proof_topology_rejects_baseline_and_required_pair_aliases() -> TestResult {
    proof_topology_cases::proof_topology_rejects_baseline_and_required_pair_aliases()
}

#[test]
fn proof_topology_rejects_noncanonical_path_spellings() -> TestResult {
    proof_topology_cases::proof_topology_rejects_noncanonical_path_spellings()
}

#[test]
fn proof_topology_cli_rejects_recomputed_baseline_and_pair_aliases() -> TestResult {
    proof_topology_cases::proof_topology_cli_rejects_recomputed_baseline_and_pair_aliases()
}

#[test]
fn proof_topology_cli_rejects_recomputed_first_divergence_path_only_mutation() -> TestResult {
    proof_topology_cases::proof_topology_cli_rejects_recomputed_first_divergence_path_only_mutation(
    )
}

#[derive(Clone, Copy)]
enum ProofTopologyMutation {
    BaselineNativeReplay,
    ReplayPairAlias,
    DebugReleaseAlias,
    MinimizedCopiedAlias,
}

fn evidence_case<'a>(manifest: &'a EvidenceManifest, case_id: &str) -> &'a EvidenceCase {
    manifest
        .cases
        .iter()
        .find(|case| case.case_id == case_id)
        .expect("reviewed evidence case")
}

#[path = "phase9_evidence_cli/evidence_cases.rs"]
mod evidence_cases;
#[path = "phase9_evidence_cli/proof_topology_cases.rs"]
mod proof_topology_cases;

#[path = "phase9_evidence_cli/manifest_builders.rs"]
mod manifest_builders;
use manifest_builders::*;
#[path = "phase9_evidence_cli/proof_mutation.rs"]
mod proof_mutation;
use proof_mutation::*;
#[path = "phase9_evidence_cli/filesystem_support.rs"]
mod filesystem_support;
use filesystem_support::*;
