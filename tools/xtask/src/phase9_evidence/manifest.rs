use super::{
    BTreeMap, BTreeSet, CompleteComparisonPayload, Component, EvidenceManifest, HarnessLimits,
    MAXIMUM_JSON_BYTES, PHASE6_POLICY_SHA256, PHASE7_POLICY_SHA256, PHASE8_POLICY_SHA256,
    PHASE9_REQUIRED_BRANCH_IDS, Path, PathBuf, Phase9ComparisonOutcome, Phase9CrossRunProof,
    Phase9CrossRunProofRecord, Phase9EvidenceError, Phase9EvidencePayloadRef, RetainedRigidPayload,
    RetainedRigidRecord, RigidWorldRequestRecord, RigidWorldResultRecord, UPSTREAM_REVISION,
    canonical_sha256, compare_complete_phase9_rigid_world_results,
    decode_rigid_world_request_jsonl, decode_rigid_world_result_jsonl, parse_json_bytes,
    read_regular_file, require_digest, resolve_existing_descendant, sha256,
    validate_phase9_cross_run_proofs, validate_phase9_evidence_bindings,
    validate_phase9_witness_bindings, validate_rigid_world_result_against_request,
};

#[allow(
    clippy::too_many_lines,
    reason = "one manifest validator keeps the cross-linked case, digest, policy, and proof checks visible"
)]
pub(super) fn validate_manifest(
    root: &Path,
    manifest: &EvidenceManifest,
) -> Result<(), Phase9EvidenceError> {
    if manifest.schema_version != 4 || manifest.case_record_schema_version != 3 {
        return Err(Phase9EvidenceError::new(
            "manifest",
            format!(
                "schema-v4 evidence with case-record schema 3 is required; found manifest schema {} and case-record schema {}; regenerate both evidence profiles",
                manifest.schema_version, manifest.case_record_schema_version
            ),
        ));
    }
    if manifest.profile != "phase9-v1"
        || manifest.upstream_revision != UPSTREAM_REVISION
        || manifest.cases.len() != 7
    {
        return Err(Phase9EvidenceError::new(
            "manifest",
            "manifest header or case cardinality is invalid",
        ));
    }
    require_digest(
        "semantic manifest",
        &manifest.semantic_manifest_sha256,
        &canonical_sha256(&manifest.cases)?,
    )?;
    let mut case_ids = BTreeSet::new();
    let mut all_bindings = Vec::new();
    let mut all_branches = BTreeSet::new();
    let required_policies = liquidfun_differential::PHASE9_REQUIRED_POLICY_PATHS
        .iter()
        .map(|path| (*path).to_owned())
        .collect::<Vec<_>>();
    let mut maximum_actions = 0;
    let mut maximum_checkpoints = 0;
    for case in &manifest.cases {
        if !case_ids.insert(case.case_id.as_str()) || case.case_id.is_empty() {
            return Err(Phase9EvidenceError::new(
                "case",
                "case IDs must be nonempty and unique",
            ));
        }
        if case.reached_branches
            != case
                .witnesses
                .iter()
                .map(|binding| binding.branch_id.as_str().to_owned())
                .collect::<Vec<_>>()
        {
            return Err(Phase9EvidenceError::new(
                "bindings",
                format!(
                    "case `{}` branch list does not match witnesses",
                    case.case_id
                ),
            ));
        }
        for branch in &case.reached_branches {
            if !all_branches.insert(branch.as_str()) {
                return Err(Phase9EvidenceError::new(
                    "bindings",
                    format!("duplicate semantic branch `{branch}`"),
                ));
            }
        }
        if case.consumed_policy_paths != required_policies {
            return Err(Phase9EvidenceError::new(
                "policies",
                format!("case `{}` has an incomplete policy array", case.case_id),
            ));
        }
        validate_retained(&case.retained_rigid)?;
        require_digest(
            "witness binding",
            &case.witness_binding_sha256,
            &canonical_sha256(&case.witnesses)?,
        )?;
        let request_bytes =
            read_payload(root, &case.request_path, &case.request_sha256, "request")?;
        let request = decode_request(&request_bytes)?;
        let timeline = request
            .scenario()
            .timelines()
            .first()
            .ok_or_else(|| Phase9EvidenceError::new("request", "missing Phase 9 timeline"))?;
        maximum_actions = maximum_actions.max(timeline.actions().len());
        maximum_checkpoints = maximum_checkpoints.max(timeline.checkpoints().len());
        for binding in &case.witnesses {
            if binding.action_index >= timeline.actions().len()
                || binding.checkpoint_index >= timeline.checkpoints().len()
            {
                return Err(Phase9EvidenceError::new(
                    "bindings",
                    format!("case `{}` has an out-of-range binding", case.case_id),
                ));
            }
        }
        let native_bytes = read_payload(
            root,
            &case.native_result_path,
            &case.native_result_sha256,
            "native result",
        )?;
        let oracle_bytes = read_payload(
            root,
            &case.oracle_result_path,
            &case.oracle_result_sha256,
            "oracle result",
        )?;
        let native = validate_result(&request, &native_bytes, "native")?;
        let oracle = validate_result(&request, &oracle_bytes, "oracle")?;
        validate_phase9_evidence_bindings(&request, &native, &case.witnesses)
            .map_err(|error| Phase9EvidenceError::new("native", error.to_string()))?;
        validate_phase9_evidence_bindings(&request, &oracle, &case.witnesses)
            .map_err(|error| Phase9EvidenceError::new("oracle", error.to_string()))?;
        let recomputed = compare_complete_phase9_rigid_world_results(&request, &native, &oracle)
            .map_err(|error| Phase9EvidenceError::new("comparison", error.to_string()))?;
        if !matches!(recomputed, Phase9ComparisonOutcome::Match { .. }) {
            return Err(Phase9EvidenceError::new(
                "comparison",
                format!(
                    "case `{}` persisted divergent native and oracle results",
                    case.case_id
                ),
            ));
        }
        let comparison_bytes = read_payload(
            root,
            &case.complete_comparison_path,
            &case.complete_comparison_sha256,
            "complete comparison",
        )?;
        let comparison: CompleteComparisonPayload =
            parse_json_bytes(&comparison_bytes, "complete comparison")?;
        if comparison.outcome != "match" || comparison.consumed_policy_paths != required_policies {
            return Err(Phase9EvidenceError::new(
                "comparison",
                format!("case `{}` did not record a complete match", case.case_id),
            ));
        }
        Phase9CrossRunProofRecord::validate_topology(&case.case_id, &case.cross_run_proofs)
            .map_err(|error| Phase9EvidenceError::new("cross-run", error.to_string()))?;
        let mut proof_payloads = BTreeMap::new();
        for reference in cross_run_payload_refs(&case.cross_run_proofs) {
            let bytes = read_payload(
                root,
                &reference.path,
                reference.sha256.as_str(),
                "cross-run proof result",
            )?;
            if let Some(existing) = proof_payloads.insert(reference.path.to_string(), bytes.clone())
                && existing != bytes
            {
                return Err(Phase9EvidenceError::new(
                    "cross-run",
                    format!("conflicting proof payload reference `{}`", reference.path),
                ));
            }
        }
        validate_phase9_cross_run_proofs(
            &case.case_id,
            &request,
            &native,
            &oracle,
            &request_bytes,
            &native_bytes,
            &oracle_bytes,
            &case.witnesses,
            &case.cross_run_proofs,
            &proof_payloads,
            &HarnessLimits::phase2_default_v1(),
        )
        .map_err(|error| Phase9EvidenceError::new("cross-run", error.to_string()))?;
        all_bindings.extend(case.witnesses.iter().cloned());
    }
    if all_branches.len() != 58
        || all_branches != PHASE9_REQUIRED_BRANCH_IDS.lines().collect::<BTreeSet<_>>()
    {
        return Err(Phase9EvidenceError::new(
            "bindings",
            "manifest must contain exactly the 58 reviewed branches",
        ));
    }
    validate_phase9_witness_bindings(&all_bindings, maximum_actions, maximum_checkpoints)
        .map_err(|error| Phase9EvidenceError::new("bindings", error.to_string()))
}

pub(super) fn cross_run_payload_refs(
    records: &[Phase9CrossRunProofRecord],
) -> Vec<&Phase9EvidencePayloadRef> {
    let mut references = Vec::new();
    for record in records {
        match &record.proof {
            Phase9CrossRunProof::ReplayResultDigestEquality {
                replay_native,
                replay_oracle,
            } => references.extend([replay_native, replay_oracle]),
            Phase9CrossRunProof::MinimizedFailureSignaturePreservation { minimized, copied }
            | Phase9CrossRunProof::DeliberateFirstDivergence { minimized, copied } => {
                references.extend([&minimized.result, &copied.result]);
            }
            Phase9CrossRunProof::D0RepeatedResultDigestEquality {
                repeated_native,
                repeated_oracle,
            } => references.extend([repeated_native, repeated_oracle]),
            Phase9CrossRunProof::DebugReleaseResultDigestEquality {
                debug_oracle,
                release_oracle,
            } => references.extend([debug_oracle, release_oracle]),
        }
    }
    references
}

pub(super) fn validate_retained(record: &RetainedRigidRecord) -> Result<(), Phase9EvidenceError> {
    if record.comparator != "phase8-v1"
        || record.phase6_policy_sha256 != PHASE6_POLICY_SHA256
        || record.phase7_policy_sha256 != PHASE7_POLICY_SHA256
        || record.phase8_policy_sha256 != PHASE8_POLICY_SHA256
        || record.outcome != "match"
    {
        return Err(Phase9EvidenceError::new(
            "retained-rigid",
            "retained-rigid comparator, policy, or outcome mismatch",
        ));
    }
    let payload = RetainedRigidPayload {
        comparator: &record.comparator,
        phase6_policy_sha256: &record.phase6_policy_sha256,
        phase7_policy_sha256: &record.phase7_policy_sha256,
        phase8_policy_sha256: &record.phase8_policy_sha256,
        outcome: &record.outcome,
    };
    require_digest(
        "retained-rigid comparison",
        &record.comparison_sha256,
        &canonical_sha256(&payload)?,
    )
}

pub(super) fn decode_request(bytes: &[u8]) -> Result<RigidWorldRequestRecord, Phase9EvidenceError> {
    decode_rigid_world_request_jsonl(bytes, &HarnessLimits::phase2_default_v1())
        .map_err(|error| Phase9EvidenceError::new("request", error.to_string()))
}

pub(super) fn validate_result(
    request: &RigidWorldRequestRecord,
    bytes: &[u8],
    side: &'static str,
) -> Result<RigidWorldResultRecord, Phase9EvidenceError> {
    let mut jsonl = bytes.to_vec();
    if !jsonl.ends_with(b"\n") {
        jsonl.push(b'\n');
    }
    let result = decode_rigid_world_result_jsonl(&jsonl, &HarnessLimits::phase2_default_v1())
        .map_err(|error| Phase9EvidenceError::new(side, error.to_string()))?;
    validate_rigid_world_result_against_request(request, &result)
        .map_err(|error| Phase9EvidenceError::new(side, error.to_string()))?;
    Ok(result)
}

pub(super) fn read_payload(
    root: &Path,
    relative: &str,
    expected_digest: &str,
    label: &'static str,
) -> Result<Vec<u8>, Phase9EvidenceError> {
    let path = checked_payload_path(root, relative)?;
    let bytes = read_regular_file(&path, label, MAXIMUM_JSON_BYTES)?;
    require_digest(label, expected_digest, &sha256(&bytes))?;
    Ok(bytes)
}

pub(super) fn checked_payload_path(
    root: &Path,
    value: &str,
) -> Result<PathBuf, Phase9EvidenceError> {
    let relative = Path::new(value);
    if relative.is_absolute()
        || !relative.starts_with("cases")
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(Phase9EvidenceError::new(
            "path",
            format!("unsafe evidence payload path `{value}`"),
        ));
    }
    resolve_existing_descendant(root, relative, "path")
}
