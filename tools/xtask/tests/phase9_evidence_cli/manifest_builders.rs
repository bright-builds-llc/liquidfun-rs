use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn synthetic_cross_run_proofs(
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

pub(super) fn build_manifest(root: &Path) -> TestResult<EvidenceManifest> {
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
        schema_version: 4,
        case_record_schema_version: 3,
        profile: "phase9-v1".to_owned(),
        upstream_revision: UPSTREAM_REVISION.to_owned(),
        semantic_manifest_sha256: sha256(&serde_json::to_vec(&cases)?),
        cases,
    })
}

pub(super) fn exact_artifact(id: u64, name: &str, archive: &Path, bytes: &[u8]) -> Value {
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
