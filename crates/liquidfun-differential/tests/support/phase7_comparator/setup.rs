fn profiles() -> (Phase6PolicyProfile, Phase7PolicyProfile) {
    (
        Phase6PolicyProfile::parse_toml(PHASE6_POLICY)
            .expect("checked-in Phase 6 policy should validate"),
        Phase7PolicyProfile::parse_toml(PHASE7_POLICY)
            .expect("checked-in Phase 7 policy should validate"),
    )
}

fn request(profile: &Phase7PolicyProfile) -> RigidWorldRequestRecord {
    crate::support::phase7_request_with_profile(Some(profile.profile_sha256()))
}

fn decode_result(value: &Value) -> RigidWorldResultRecord {
    let mut bytes = serde_json::to_vec(value).expect("result mutation should encode");
    bytes.push(b'\n');
    decode_rigid_world_result_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("bounded semantic result mutation should decode")
}

fn request_with_ray_rules(
    profile: &Phase7PolicyProfile,
    directive_rules: Value,
) -> RigidWorldRequestRecord {
    let mut value = serde_json::to_value(request(profile)).expect("request should serialize");
    let ray_action = value["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("timeline actions should be an array")
        .iter_mut()
        .find(|record| record["action_id"] == "phase7-action-20")
        .expect("Phase 7 ray action should exist");
    ray_action["action"]["directive_rules"] = directive_rules;
    let mut bytes = serde_json::to_vec(&value).expect("request mutation should encode");
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("bounded ray request mutation should decode")
}

fn request_with_query_rules(
    profile: &Phase7PolicyProfile,
    directive_rules: Value,
) -> RigidWorldRequestRecord {
    let mut value = serde_json::to_value(request(profile)).expect("request should serialize");
    let query_action = value["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("timeline actions should be an array")
        .iter_mut()
        .find(|record| record["action_id"] == "phase7-action-19")
        .expect("Phase 7 query action should exist");
    query_action["action"]["directive_rules"] = directive_rules;
    let mut bytes = serde_json::to_vec(&value).expect("request mutation should encode");
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("bounded query request mutation should decode")
}

fn request_with_out_of_ray_clip(profile: &Phase7PolicyProfile) -> RigidWorldRequestRecord {
    let request = request_with_ray_rules(
        profile,
        json!([{
            "target": { "fixture_id": "nc-dynamic-fixture", "child_index": 0 },
            "directive": { "kind": "clip", "fraction_bits": 0.5_f32.to_bits() }
        }]),
    );
    let mut value = serde_json::to_value(request).expect("request should serialize");
    let ray_action = value["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("timeline actions should be an array")
        .iter_mut()
        .find(|record| record["action_id"] == "phase7-action-20")
        .expect("Phase 7 ray action should exist");
    ray_action["action"]["end"]["x_bits"] = json!(5.0_f32.to_bits());
    let mut bytes = serde_json::to_vec(&value).expect("request mutation should encode");
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("bounded ray request mutation should decode")
}

fn arbitrary_clip_order_results(
    profile: &Phase7PolicyProfile,
) -> (
    RigidWorldRequestRecord,
    RigidWorldResultRecord,
    RigidWorldResultRecord,
) {
    let request = request_with_ray_rules(
        profile,
        json!([{
            "target": { "fixture_id": "nc-dynamic-fixture", "child_index": 0 },
            "directive": { "kind": "clip", "fraction_bits": 0.1_f32.to_bits() }
        }]),
    );
    let baseline = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound arbitrary clip request should execute");
    let mut native_value = serde_json::to_value(&baseline).expect("result should serialize");
    let ray = observation_mut(phase7_observations(&mut native_value), "ray_cast");
    ray["observation"]["completion"] = json!("exhausted");
    ray["observation"]["final_max_fraction_bits"] = json!(0.1_f32.to_bits());
    ray["observation"]["hits"] = json!([
        {
            "fixture_id": "nc-dynamic-fixture",
            "child_index": 0,
            "point": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "normal": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() },
            "fraction_bits": 0.75_f32.to_bits()
        },
        {
            "fixture_id": "nc-static-fixture",
            "child_index": 0,
            "point": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() },
            "normal": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "fraction_bits": 0.05_f32.to_bits()
        }
    ]);
    let native = decode_result(&native_value);
    let mut oracle_value = native_value;
    observation_mut(phase7_observations(&mut oracle_value), "ray_cast")["observation"]["hits"]
        .as_array_mut()
        .expect("ray hits should be an array")
        .reverse();
    let oracle = decode_result(&oracle_value);
    (request, native, oracle)
}

fn duplicate_ray_results(
    profile: &Phase7PolicyProfile,
    expected_fractions: &[u32],
    actual_fractions: &[u32],
) -> (
    RigidWorldRequestRecord,
    RigidWorldResultRecord,
    RigidWorldResultRecord,
) {
    let request = request_with_ray_rules(profile, json!([]));
    let baseline = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound exhaustive ray request should execute");
    let mut expected_value = serde_json::to_value(&baseline).expect("result should serialize");
    let ray = observation_mut(phase7_observations(&mut expected_value), "ray_cast");
    ray["observation"]["completion"] = json!("exhausted");
    ray["observation"]["final_max_fraction_bits"] = json!(1.0_f32.to_bits());
    ray["observation"]["hits"] = Value::Array(
        expected_fractions
            .iter()
            .copied()
            .map(duplicate_ray_hit)
            .collect(),
    );
    let expected = decode_result(&expected_value);
    let mut actual_value = expected_value;
    observation_mut(phase7_observations(&mut actual_value), "ray_cast")["observation"]["hits"] =
        Value::Array(
            actual_fractions
                .iter()
                .copied()
                .map(duplicate_ray_hit)
                .collect(),
        );
    let actual = decode_result(&actual_value);
    (request, expected, actual)
}

fn duplicate_ray_hit(fraction_bits: u32) -> Value {
    json!({
        "fixture_id": "nc-static-fixture",
        "child_index": 0,
        "point": { "x_bits": 0.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
        "normal": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
        "fraction_bits": fraction_bits
    })
}

fn boundary_clip_results(
    profile: &Phase7PolicyProfile,
    expected_fraction_bits: u32,
    actual_fraction_bits: u32,
) -> (
    RigidWorldRequestRecord,
    RigidWorldResultRecord,
    RigidWorldResultRecord,
) {
    let final_fraction_bits = 0.1_f32.to_bits();
    let request = request_with_ray_rules(
        profile,
        json!([{
            "target": { "fixture_id": "nc-dynamic-fixture", "child_index": 0 },
            "directive": { "kind": "clip", "fraction_bits": final_fraction_bits }
        }]),
    );
    let baseline = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound boundary clip request should execute");
    let mut expected_value = serde_json::to_value(&baseline).expect("result should serialize");
    let ray = observation_mut(phase7_observations(&mut expected_value), "ray_cast");
    ray["observation"]["completion"] = json!("exhausted");
    ray["observation"]["final_max_fraction_bits"] = json!(final_fraction_bits);
    ray["observation"]["hits"] = json!([boundary_clip_hit(expected_fraction_bits)]);
    let expected = decode_result(&expected_value);

    let mut actual_value = expected_value;
    observation_mut(phase7_observations(&mut actual_value), "ray_cast")["observation"]["hits"] =
        json!([boundary_clip_hit(actual_fraction_bits)]);
    let actual = decode_result(&actual_value);
    (request, expected, actual)
}

fn boundary_clip_hit(fraction_bits: u32) -> Value {
    json!({
        "fixture_id": "nc-dynamic-fixture",
        "child_index": 0,
        "point": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
        "normal": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() },
        "fraction_bits": fraction_bits
    })
}

fn with_single_ray_point_x(
    result: &RigidWorldResultRecord,
    point_x_bits: u32,
) -> RigidWorldResultRecord {
    let mut value = serde_json::to_value(result).expect("result should serialize");
    let hit =
        &mut observation_mut(phase7_observations(&mut value), "ray_cast")["observation"]["hits"][0];
    hit["point"]["x_bits"] = json!(point_x_bits);
    decode_result(&value)
}

fn request_with_split_phase7_checkpoints(profile: &Phase7PolicyProfile) -> RigidWorldRequestRecord {
    let mut value = serde_json::to_value(request(profile)).expect("request should serialize");
    let checkpoints = value["scenario"]["timelines"][0]["checkpoints"]
        .as_array_mut()
        .expect("non-colliding checkpoints should be an array");
    checkpoints.insert(
        6,
        json!({
            "checkpoint_id": "phase7-step-checkpoint",
            "after_action_id": "phase7-action-18",
            "phase": "phase7-adapter",
            "counts": {
                "bodies": 3,
                "fixtures": 3,
                "contacts": 0,
                "manifold_points": 0,
                "events": 0,
                "destructions": 0
            },
            "transitions": []
        }),
    );
    let mut bytes = serde_json::to_vec(&value).expect("request mutation should encode");
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("split Phase 7 checkpoint request should decode")
}

fn phase7_observations(value: &mut Value) -> &mut Vec<Value> {
    value["timelines"][0]["checkpoints"][6]["observations"]
        .as_array_mut()
        .expect("Phase 7 checkpoint should contain observations")
}

fn observation_mut<'a>(observations: &'a mut [Value], kind: &str) -> &'a mut Value {
    observations
        .iter_mut()
        .find(|observation| observation["kind"] == kind)
        .expect("requested Phase 7 observation should exist")
}

fn checkpoint_observations_mut<'a>(
    value: &'a mut Value,
    checkpoint_id: &str,
) -> &'a mut Vec<Value> {
    value["timelines"][0]["checkpoints"]
        .as_array_mut()
        .expect("non-colliding checkpoints should be an array")
        .iter_mut()
        .find(|checkpoint| checkpoint["checkpoint_id"] == checkpoint_id)
        .expect("requested checkpoint should exist")["observations"]
        .as_array_mut()
        .expect("checkpoint observations should be an array")
}
