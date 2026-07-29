fn particle(id: &str, system: &str, x: f32, lifetime: f32, color: u8) -> Value {
    particle_with_flags(id, system, x, lifetime, color, 0)
}

fn particle_with_flags(
    id: &str,
    system: &str,
    x: f32,
    lifetime: f32,
    color: u8,
    flags_bits: u32,
) -> Value {
    json!({
        "particle_id": id, "system_id": system, "position": bits(x, 0.0),
        "velocity": bits(0.0, 0.0), "flags_bits": flags_bits, "color": [color, 0, 255, 255],
        "lifetime_bits": lifetime.to_bits()
    })
}

#[allow(clippy::too_many_arguments)]
fn particle_with_velocity(
    id: &str,
    system: &str,
    x: f32,
    y: f32,
    velocity_x: f32,
    velocity_y: f32,
    lifetime: f32,
    color: u8,
) -> Value {
    json!({
        "particle_id": id, "system_id": system, "position": bits(x, y),
        "velocity": bits(velocity_x, velocity_y), "flags_bits": 0,
        "color": [color, 0, 255, 255], "lifetime_bits": lifetime.to_bits()
    })
}

#[allow(clippy::too_many_arguments)]
fn particle_with_flags_and_velocity(
    id: &str,
    system: &str,
    x: f32,
    y: f32,
    velocity_x: f32,
    velocity_y: f32,
    lifetime: f32,
    color: u8,
    flags_bits: u32,
) -> Value {
    let mut particle =
        particle_with_velocity(id, system, x, y, velocity_x, velocity_y, lifetime, color);
    particle["flags_bits"] = json!(flags_bits);
    particle
}

fn bits(x: f32, y: f32) -> Value {
    json!({ "x_bits": x.to_bits(), "y_bits": y.to_bits() })
}

fn action(id: &str, action: Value) -> Value {
    let mut record = json!({
        "action_id": id, "phase": "phase9",
        "action": { "kind": "particle" }
    });
    record["action"]["action"] = action;
    record
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn canonical_sha256(value: &impl Serialize) -> String {
    sha256(&serde_json::to_vec(value).expect("canonical evidence JSON"))
}

fn retained_rigid_record() -> RetainedRigidRecord {
    let payload = RetainedRigidPayload {
        comparator: "phase8-v1",
        phase6_policy_sha256: PHASE6_POLICY_SHA256,
        phase7_policy_sha256: PHASE7_POLICY_SHA256,
        phase8_policy_sha256: PHASE8_POLICY_SHA256,
        outcome: "match",
    };
    RetainedRigidRecord {
        comparator: payload.comparator.to_owned(),
        phase6_policy_sha256: payload.phase6_policy_sha256.to_owned(),
        phase7_policy_sha256: payload.phase7_policy_sha256.to_owned(),
        phase8_policy_sha256: payload.phase8_policy_sha256.to_owned(),
        outcome: payload.outcome.to_owned(),
        comparison_sha256: canonical_sha256(&payload),
    }
}

fn evidence_payload_paths(case_id: &str) -> (String, String, String, String) {
    let base = format!("cases/{case_id}");
    (
        format!("{base}/request.jsonl"),
        format!("{base}/native-result.json"),
        format!("{base}/oracle-result.json"),
        format!("{base}/complete-comparison.json"),
    )
}

fn evidence_case_record(
    case: &CorpusCase,
    payloads: &EvidenceCasePayloads,
    consumed_policy_paths: &[&str],
    cross_run_proofs: Vec<Phase9CrossRunProofRecord>,
) -> EvidenceCaseRecord {
    let (request_path, native_result_path, oracle_result_path, complete_comparison_path) =
        evidence_payload_paths(&case.case_id);
    EvidenceCaseRecord {
        case_id: case.case_id.clone(),
        reached_branches: case
            .witnesses
            .iter()
            .map(|witness| witness.branch_id.clone())
            .collect(),
        witness_binding_sha256: canonical_sha256(&case.witnesses),
        witnesses: case.witnesses.clone(),
        consumed_policy_paths: consumed_policy_paths
            .iter()
            .map(|path| (*path).to_owned())
            .collect(),
        retained_rigid: retained_rigid_record(),
        request_path,
        request_sha256: sha256(&payloads.request),
        native_result_path,
        native_result_sha256: sha256(&payloads.native_result),
        oracle_result_path,
        oracle_result_sha256: sha256(&payloads.oracle_result),
        complete_comparison_path,
        complete_comparison_sha256: sha256(&payloads.complete_comparison),
        cross_run_proofs,
    }
}

fn validate_evidence_case_value(
    value: &Value,
    payloads: &EvidenceCasePayloads,
) -> Result<(), String> {
    if value.get("retained_rigid").is_none() {
        return Err("missing retained-rigid proof".to_owned());
    }
    let record: EvidenceCaseRecord =
        serde_json::from_value(value.clone()).map_err(|error| error.to_string())?;
    let retained = &record.retained_rigid;
    if retained.comparator != "phase8-v1"
        || retained.phase6_policy_sha256 != PHASE6_POLICY_SHA256
        || retained.phase7_policy_sha256 != PHASE7_POLICY_SHA256
        || retained.phase8_policy_sha256 != PHASE8_POLICY_SHA256
    {
        return Err("retained-rigid policy digest mismatch".to_owned());
    }
    if retained.outcome != "match" {
        return Err("retained-rigid outcome mismatch".to_owned());
    }
    let retained_payload = RetainedRigidPayload {
        comparator: &retained.comparator,
        phase6_policy_sha256: &retained.phase6_policy_sha256,
        phase7_policy_sha256: &retained.phase7_policy_sha256,
        phase8_policy_sha256: &retained.phase8_policy_sha256,
        outcome: &retained.outcome,
    };
    if retained.comparison_sha256 != canonical_sha256(&retained_payload) {
        return Err("retained-rigid comparison digest mismatch".to_owned());
    }
    if record.witness_binding_sha256 != canonical_sha256(&record.witnesses) {
        return Err("witness binding digest mismatch".to_owned());
    }
    if record.request_sha256 != sha256(&payloads.request) {
        return Err("request payload digest mismatch".to_owned());
    }
    if record.native_result_sha256 != sha256(&payloads.native_result) {
        return Err("native result payload digest mismatch".to_owned());
    }
    if record.oracle_result_sha256 != sha256(&payloads.oracle_result) {
        return Err("oracle result payload digest mismatch".to_owned());
    }
    if record.complete_comparison_sha256 != sha256(&payloads.complete_comparison) {
        return Err("complete comparison payload digest mismatch".to_owned());
    }
    Ok(())
}

fn evidence_case_fixture() -> (Value, EvidenceCasePayloads) {
    let witness = valid_witness_binding("multiple_systems");
    let case = CorpusCase {
        case_id: "test-case".to_owned(),
        authority: Authority::Independent,
        fixture: "test-case.jsonl".to_owned(),
        request_sha256: sha256(b"request"),
        witnesses: vec![witness],
    };
    let complete_comparison = serde_json::to_vec(&CompleteComparisonPayload {
        outcome: "match".to_owned(),
        consumed_policy_paths: PHASE9_REQUIRED_POLICY_PATHS
            .iter()
            .map(|path| (*path).to_owned())
            .collect(),
    })
    .expect("comparison fixture");
    let payloads = EvidenceCasePayloads {
        request: b"request".to_vec(),
        native_result: b"native".to_vec(),
        oracle_result: b"oracle".to_vec(),
        complete_comparison,
    };
    let record = evidence_case_record(&case, &payloads, PHASE9_REQUIRED_POLICY_PATHS, Vec::new());
    (
        serde_json::to_value(record).expect("evidence fixture"),
        payloads,
    )
}

fn write_evidence_payload(root: &Path, relative: &str, bytes: &[u8]) {
    let path = root.join(relative);
    let parent = path.parent().expect("evidence payload parent");
    fs::create_dir_all(parent).expect("evidence payload directory");
    fs::write(path, bytes).expect("evidence payload");
}

fn fixture_path(case: &CorpusCase) -> std::path::PathBuf {
    let relative = Path::new(&case.fixture);
    assert!(!relative.is_absolute(), "fixture paths must be relative");
    assert!(
        relative
            .components()
            .all(|component| matches!(component, Component::Normal(_))),
        "fixture paths must not escape the Phase 9 corpus"
    );
    assert_eq!(
        relative.extension().and_then(|value| value.to_str()),
        Some("jsonl")
    );
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/rigid_world/phase9")
        .join(relative)
}

fn observation_for_witness<'a>(
    request: &'a Value,
    result: &'a Value,
    witness: &Phase9WitnessBinding,
) -> &'a Value {
    let timeline = &request["scenario"]["timelines"][0];
    let actions = timeline["actions"].as_array().expect("actions");
    let checkpoints = timeline["checkpoints"].as_array().expect("checkpoints");
    let action = actions
        .get(witness.action_index)
        .expect("typed witness action index must exist");
    let checkpoint = checkpoints
        .get(witness.checkpoint_index)
        .expect("typed witness checkpoint index must exist");
    observation_for_action(
        request,
        result,
        action["action_id"].as_str().expect("action ID"),
        checkpoint["checkpoint_id"].as_str().expect("checkpoint ID"),
    )
}

fn observation_for_action<'a>(
    request: &'a Value,
    result: &'a Value,
    action_id: &str,
    checkpoint_id: &str,
) -> &'a Value {
    let timeline = &request["scenario"]["timelines"][0];
    let actions = timeline["actions"].as_array().expect("actions");
    let checkpoints = timeline["checkpoints"].as_array().expect("checkpoints");
    let checkpoint_index = checkpoints
        .iter()
        .position(|checkpoint| checkpoint["checkpoint_id"] == checkpoint_id)
        .expect("witness checkpoint must exist");
    let action_end = actions
        .iter()
        .position(|action| action["action_id"] == checkpoints[checkpoint_index]["after_action_id"])
        .expect("checkpoint action must exist");
    let action_start = if checkpoint_index == 0 {
        0
    } else {
        actions
            .iter()
            .position(|action| {
                action["action_id"] == checkpoints[checkpoint_index - 1]["after_action_id"]
            })
            .expect("previous checkpoint action must exist")
            + 1
    };
    let target = actions[action_start..=action_end]
        .iter()
        .position(|action| action["action_id"] == action_id)
        .expect("witness action must belong to its checkpoint");
    assert_eq!(
        actions[action_start + target]["action"]["kind"],
        "particle",
        "Phase 9 semantic witnesses must name particle actions"
    );
    let observation_index = actions[action_start..action_start + target]
        .iter()
        .filter(|action| action["action"]["kind"] == "particle")
        .count();
    &result["timelines"][0]["checkpoints"][checkpoint_index]["observations"][observation_index]
}

fn phase9_observation<'a>(request: &'a Value, result: &'a Value, action_id: &str) -> &'a Value {
    let checkpoint_id = if action_id == "inspect-system" {
        "phase9-only-checkpoint"
    } else {
        "phase9-corpus"
    };
    &observation_for_action(request, result, action_id, checkpoint_id)["observation"]
}

fn particle_declaration<'a>(request: &'a Value, particle_id: &str) -> &'a Value {
    request["scenario"]["timelines"][0]["particles"]
        .as_array()
        .expect("particle declarations")
        .iter()
        .find(|particle| particle["particle_id"] == particle_id)
        .expect("declared particle")
}

fn system_declaration<'a>(request: &'a Value, system_id: &str) -> &'a Value {
    request["scenario"]["timelines"][0]["particle_systems"]
        .as_array()
        .expect("system declarations")
        .iter()
        .find(|system| system["system_id"] == system_id)
        .expect("declared system")
}

fn phase9_checkpoint<'a>(result: &'a Value, checkpoint_id: &str) -> &'a Value {
    result["timelines"][0]["checkpoints"]
        .as_array()
        .expect("result checkpoints")
        .iter()
        .find(|checkpoint| checkpoint["checkpoint_id"] == checkpoint_id)
        .expect("Phase 9 result checkpoint")
}

fn assert_no_particle_lifecycle(result: &Value, particle_id: &str) {
    let occurrences = phase9_checkpoint(result, "phase9-corpus")["observations"]
        .as_array()
        .expect("Phase 9 observations")
        .iter()
        .filter(|observation| {
            observation["observation"]["kind"] == "lifecycle"
                && observation["observation"]["occurrence"]["maybe_particle_id"] == particle_id
        })
        .count();
    assert_eq!(
        occurrences, 0,
        "{particle_id} must not emit a lifecycle occurrence"
    );
}
