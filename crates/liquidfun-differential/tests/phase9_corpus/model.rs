#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CorpusManifest {
    profile: String,
    retained_request_sha256: String,
    pinned_upstream_revision: String,
    pinned_witness_sha256: String,
    forbidden_phase10_members: Vec<String>,
    cases: Vec<CorpusCase>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CorpusCase {
    case_id: String,
    authority: Authority,
    fixture: String,
    request_sha256: String,
    witnesses: Vec<Phase9WitnessBinding>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum Authority {
    PinnedOracle,
    Independent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceManifest {
    schema_version: u32,
    case_record_schema_version: u32,
    profile: String,
    upstream_revision: String,
    semantic_manifest_sha256: String,
    cases: Vec<EvidenceCaseRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceCaseRecord {
    case_id: String,
    reached_branches: Vec<ScenarioId>,
    witnesses: Vec<Phase9WitnessBinding>,
    witness_binding_sha256: String,
    consumed_policy_paths: Vec<String>,
    retained_rigid: RetainedRigidRecord,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RetainedRigidRecord {
    comparator: String,
    phase6_policy_sha256: String,
    phase7_policy_sha256: String,
    phase8_policy_sha256: String,
    outcome: String,
    comparison_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RetainedRigidPayload<'a> {
    comparator: &'a str,
    phase6_policy_sha256: &'a str,
    phase7_policy_sha256: &'a str,
    phase8_policy_sha256: &'a str,
    outcome: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CompleteComparisonPayload {
    outcome: String,
    consumed_policy_paths: Vec<String>,
}

#[derive(Debug, Clone)]
struct EvidenceCasePayloads {
    request: Vec<u8>,
    native_result: Vec<u8>,
    oracle_result: Vec<u8>,
    complete_comparison: Vec<u8>,
}

fn manifest() -> CorpusManifest {
    serde_json::from_str(MANIFEST).expect("the checked-in Phase 9 manifest should be strict JSON")
}

fn request_value() -> Value {
    serde_json::from_slice(RETAINED_REQUEST).expect("the retained rigid request should be JSON")
}

fn decode_value(value: &Value) -> Result<liquidfun_test_protocol::RigidWorldRequestRecord, String> {
    let mut bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .map_err(|error| error.to_string())
}

fn bounded_phase9_request(case_id: &str) -> liquidfun_test_protocol::RigidWorldRequestRecord {
    let mut value = request_value();
    value["request_id"] = json!(format!("phase-09-{case_id}"));
    let timeline = &mut value["scenario"]["timelines"][0];
    configure_phase9_declarations(timeline, case_id);
    let mut phase9_actions = phase9_actions();
    order_phase9_actions(&mut phase9_actions, case_id);
    retain_relevant_actions(&mut phase9_actions, case_id);
    let final_action = phase9_actions
        .last()
        .expect("Phase 9 corpus should have a final action")["action_id"]
        .clone();
    let insertion_index = if case_id == "contacts-listeners-filters-and-coupling" {
        6
    } else {
        0
    };
    timeline["actions"]
        .as_array_mut()
        .expect("retained actions should be an array")
        .splice(insertion_index..insertion_index, phase9_actions);
    insert_phase9_checkpoints(timeline, case_id, &final_action);
    decode_value(&value).expect("the bounded Phase 9 corpus should decode")
}
