use super::*;

pub(super) fn mutate_proof_topology(manifest: &mut Value, mutation: ProofTopologyMutation) {
    let case = evidence_case_value_mut(manifest);
    match mutation {
        ProofTopologyMutation::BaselineNativeReplay => {
            let path = case["native_result_path"].clone();
            let sha256 = case["native_result_sha256"].clone();
            let reference = proof_reference_value_mut(case, "replay_identity", "replay_native");
            reference["path"] = path;
            reference["sha256"] = sha256;
        }
        ProofTopologyMutation::ReplayPairAlias => replace_proof_reference(
            case,
            "replay_identity",
            "replay_oracle",
            "replay_identity",
            "replay_native",
        ),
        ProofTopologyMutation::DebugReleaseAlias => replace_proof_reference(
            case,
            "debug_release_agreement",
            "release_oracle",
            "debug_release_agreement",
            "debug_oracle",
        ),
        ProofTopologyMutation::MinimizedCopiedAlias => replace_proof_reference(
            case,
            "minimization_identity",
            "copied",
            "minimization_identity",
            "minimized",
        ),
    }
}

pub(super) fn replace_proof_reference(
    case: &mut Value,
    target_branch: &str,
    target_field: &str,
    source_branch: &str,
    source_field: &str,
) {
    let source = proof_reference_value(case, source_branch, source_field).clone();
    *proof_reference_value_mut(case, target_branch, target_field) = source;
}

pub(super) fn evidence_case_value_mut(manifest: &mut Value) -> &mut Value {
    manifest["cases"]
        .as_array_mut()
        .expect("manifest cases")
        .iter_mut()
        .find(|case| case["case_id"] == "closed-evidence-contract")
        .expect("closed evidence case")
}

pub(super) fn proof_record_value_mut<'a>(case: &'a mut Value, branch_id: &str) -> &'a mut Value {
    case["cross_run_proofs"]
        .as_array_mut()
        .expect("cross-run proofs")
        .iter_mut()
        .find(|record| record["branch_id"] == branch_id)
        .expect("reviewed proof record")
}

pub(super) fn proof_reference_value<'a>(
    case: &'a Value,
    branch_id: &str,
    field: &str,
) -> &'a Value {
    let record = case["cross_run_proofs"]
        .as_array()
        .expect("cross-run proofs")
        .iter()
        .find(|record| record["branch_id"] == branch_id)
        .expect("reviewed proof record");
    let reference = find_object_field(&record["proof"], field);
    reference.get("result").unwrap_or(reference)
}

pub(super) fn proof_reference_value_mut<'a>(
    case: &'a mut Value,
    branch_id: &str,
    field: &str,
) -> &'a mut Value {
    let record = proof_record_value_mut(case, branch_id);
    let mut reference =
        find_object_field_mut(&mut record["proof"], field).expect("reviewed proof field");
    if reference.get("result").is_some() {
        reference = reference
            .get_mut("result")
            .expect("mismatch result reference");
    }
    reference
}

pub(super) fn set_proof_path(
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

pub(super) fn first_result_member_mut<'a>(value: &'a mut Value, member: &str) -> &'a mut Value {
    value["timelines"]
        .as_array_mut()
        .expect("timelines")
        .iter_mut()
        .flat_map(|timeline| timeline["checkpoints"].as_array_mut().expect("checkpoints"))
        .find_map(|checkpoint| checkpoint[member].as_array_mut()?.first_mut())
        .expect("retained result member")
}

pub(super) fn retained_mismatch(
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

pub(super) fn digest(bytes: &[u8]) -> Sha256Hex {
    Sha256Hex::new(sha256(bytes)).expect("computed digest")
}
