fn request() -> liquidfun_test_protocol::RigidWorldRequestRecord {
    decode_rigid_world_request_jsonl(REQUEST, &HarnessLimits::phase2_default_v1())
        .expect("checked-in rigid-world request should decode")
}

fn encode_value(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).expect("fixture mutation should encode");
    bytes.push(b'\n');
    bytes
}

fn profile() -> Phase6PolicyProfile {
    Phase6PolicyProfile::parse_toml(POLICY).expect("checked-in rigid policy should validate")
}

fn comparison_request() -> RigidWorldRequestRecord {
    let profile = profile();
    let mut value = serde_json::from_slice::<Value>(REQUEST).expect("fixture should be JSON");
    value["tolerance_profile_sha256"] = json!(profile.profile_sha256().as_str());
    decode_rigid_world_request_jsonl(&encode_value(&value), &HarnessLimits::phase2_default_v1())
        .expect("profile-bound rigid request should decode")
}

fn comparison_request_with_partial_body_checkpoint() -> RigidWorldRequestRecord {
    let profile = profile();
    let mut value = serde_json::from_slice::<Value>(REQUEST).expect("fixture should be JSON");
    value["tolerance_profile_sha256"] = json!(profile.profile_sha256().as_str());
    let checkpoints = value["scenario"]["timelines"][0]["checkpoints"]
        .as_array_mut()
        .expect("non-colliding checkpoints should be an array");
    let final_checkpoint = checkpoints
        .last_mut()
        .expect("non-colliding timeline should have a final checkpoint");
    final_checkpoint["counts"]["destructions"] = json!(2);
    let insert_at = checkpoints.len() - 1;
    checkpoints.insert(
        insert_at,
        json!({
            "checkpoint_id": "nc-first-body-destroyed",
            "after_action_id": "nc-destroy-body-static",
            "phase": "destroy-bodies",
            "counts": {
                "bodies": 2,
                "fixtures": 0,
                "contacts": 0,
                "manifold_points": 0,
                "events": 0,
                "destructions": 1
            },
            "transitions": []
        }),
    );
    decode_rigid_world_request_jsonl(&encode_value(&value), &HarnessLimits::phase2_default_v1())
        .expect("partial body-destruction checkpoint should decode")
}

fn comparison_request_with_cascade_query_checkpoint() -> RigidWorldRequestRecord {
    let profile = profile();
    let mut value = serde_json::from_slice::<Value>(REQUEST).expect("fixture should be JSON");
    value["tolerance_profile_sha256"] = json!(profile.profile_sha256().as_str());
    let timeline = value["scenario"]["timelines"]
        .as_array_mut()
        .expect("fixture timelines should be an array")
        .iter_mut()
        .find(|timeline| timeline["witness_family"] == "world_query_and_ray_cast")
        .expect("query timeline should exist");
    let actions = timeline["actions"]
        .as_array_mut()
        .expect("query actions should be an array");
    let destroy_index = actions
        .iter()
        .position(|action| action["action_id"] == "query-12")
        .expect("right-body destruction should exist");
    actions.insert(
        destroy_index + 1,
        json!({
            "action_id": "query-12-cascade-query",
            "phase": "teardown-query",
            "action": {
                "kind": "query_aabb",
                "aabb": {
                    "lower": { "x_bits": (-4.0_f32).to_bits(), "y_bits": (-2.0_f32).to_bits() },
                    "upper": { "x_bits": 4.0_f32.to_bits(), "y_bits": 2.0_f32.to_bits() }
                },
                "directive_rules": []
            }
        }),
    );
    timeline["checkpoints"]
        .as_array_mut()
        .expect("query checkpoints should be an array")
        .push(json!({
            "checkpoint_id": "query-right-cascade-query",
            "after_action_id": "query-12-cascade-query",
            "phase": "teardown-query",
            "counts": {
                "bodies": 2,
                "fixtures": 2,
                "contacts": 0,
                "manifold_points": 0,
                "events": 0,
                "destructions": 1
            },
            "transitions": []
        }));
    decode_rigid_world_request_jsonl(&encode_value(&value), &HarnessLimits::phase2_default_v1())
        .expect("query checkpoint after fixture cascade should decode")
}

fn request_with_expanding_ray_clips() -> RigidWorldRequestRecord {
    let mut value = serde_json::from_slice::<Value>(REQUEST).expect("fixture should be JSON");
    let query_timeline = value["scenario"]["timelines"]
        .as_array_mut()
        .expect("fixture timelines should be an array")
        .iter_mut()
        .find(|timeline| timeline["witness_family"] == "world_query_and_ray_cast")
        .expect("query timeline should exist");
    let ray_action = query_timeline["actions"]
        .as_array_mut()
        .expect("query actions should be an array")
        .iter_mut()
        .find(|action| action["action_id"] == "query-10")
        .expect("clip action should exist");
    ray_action["action"]["directive_rules"] = json!([
        {
            "target": { "fixture_id": "query-right-fixture", "child_index": 0 },
            "directive": { "kind": "clip", "fraction_bits": 0.5_f32.to_bits() }
        },
        {
            "target": { "fixture_id": "query-center-fixture", "child_index": 0 },
            "directive": { "kind": "clip", "fraction_bits": 0.75_f32.to_bits() }
        }
    ]);
    decode_rigid_world_request_jsonl(&encode_value(&value), &HarnessLimits::phase2_default_v1())
        .expect("bounded expanding-clip request should decode")
}

fn assert_identity_rejected_on_each_side(
    request: &RigidWorldRequestRecord,
    complete: &RigidWorldResultRecord,
    mutated: &RigidWorldResultRecord,
    profile: &Phase6PolicyProfile,
    path: &str,
) {
    for side in [RigidEngineSide::Native, RigidEngineSide::Oracle] {
        let result = match side {
            RigidEngineSide::Native => {
                compare_rigid_world_results(request, mutated, complete, profile)
            }
            RigidEngineSide::Oracle => {
                compare_rigid_world_results(request, complete, mutated, profile)
            }
        };
        let Err(RigidComparisonFailure::Declaration(report)) = result else {
            panic!("{side:?} identity disagreement must fail declaration validation");
        };
        assert_eq!(report.engine_side(), side);
        assert_eq!(report.semantic_path(), path);
    }
}

fn assert_observation_rejected_on_each_side(
    request: &RigidWorldRequestRecord,
    complete: &RigidWorldResultRecord,
    mutated: &RigidWorldResultRecord,
    profile: &Phase6PolicyProfile,
) {
    for side in [RigidEngineSide::Native, RigidEngineSide::Oracle] {
        let result = match side {
            RigidEngineSide::Native => {
                compare_rigid_world_results(request, mutated, complete, profile)
            }
            RigidEngineSide::Oracle => {
                compare_rigid_world_results(request, complete, mutated, profile)
            }
        };
        assert!(
            matches!(result, Err(RigidComparisonFailure::Harness(_))),
            "{side:?} invalid observation must fail request-bound validation"
        );
    }
}

fn decode_result_value(value: &Value) -> RigidWorldResultRecord {
    decode_rigid_world_result_jsonl(&encode_value(value), &HarnessLimits::phase2_default_v1())
        .expect("mutated result should remain internally valid")
}

fn result_value(result: &RigidWorldResultRecord) -> Value {
    serde_json::to_value(result).expect("validated result should serialize")
}
