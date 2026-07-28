fn retained_profiles() -> (
    Phase6PolicyProfile,
    Phase7PolicyProfile,
    Phase8PolicyProfile,
) {
    (
        Phase6PolicyProfile::parse_toml(PHASE6_POLICY)
            .expect("checked-in Phase 6 policy should parse"),
        Phase7PolicyProfile::parse_toml(PHASE7_POLICY)
            .expect("checked-in Phase 7 policy should parse"),
        Phase8PolicyProfile::parse_toml(PHASE8_POLICY)
            .expect("checked-in Phase 8 policy should parse"),
    )
}

fn mutated_phase9_result(
    native: &RigidWorldResultRecord,
    mutate: impl FnOnce(&mut Value),
) -> RigidWorldResultRecord {
    let mut value = serde_json::to_value(native).expect("result should serialize");
    mutate(&mut value);
    let mut bytes = serde_json::to_vec(&value).expect("mutation should serialize");
    bytes.push(b'\n');
    decode_rigid_world_result_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("request-valid retained mutation should decode")
}

fn first_checkpoint_member_mut<'a>(value: &'a mut Value, member: &str) -> &'a mut Value {
    value["timelines"]
        .as_array_mut()
        .expect("timelines should be an array")
        .iter_mut()
        .flat_map(|timeline| {
            timeline["checkpoints"]
                .as_array_mut()
                .expect("checkpoints should be an array")
        })
        .filter(|checkpoint| {
            checkpoint
                .get("observations")
                .and_then(Value::as_array)
                .is_none_or(|observations| {
                    observations.iter().all(|observation| {
                        matches!(
                            observation["kind"].as_str(),
                            Some("body_state" | "step" | "query" | "ray_cast" | "origin_shift")
                        )
                    })
                })
        })
        .find_map(|checkpoint| {
            checkpoint
                .get_mut(member)
                .expect("checkpoint member should exist")
                .as_array_mut()
                .and_then(|values| values.first_mut())
        })
        .unwrap_or_else(|| panic!("a checkpoint should contain `{member}`"))
}

fn first_observation_mut(value: &mut Value, predicate: impl Fn(&Value) -> bool) -> &mut Value {
    value["timelines"]
        .as_array_mut()
        .expect("timelines should be an array")
        .iter_mut()
        .flat_map(|timeline| {
            timeline["checkpoints"]
                .as_array_mut()
                .expect("checkpoints should be an array")
        })
        .filter_map(|checkpoint| {
            checkpoint
                .get_mut("observations")
                .and_then(Value::as_array_mut)
        })
        .flatten()
        .find(|observation| predicate(observation))
        .expect("the requested observation should exist")
}

fn expected_retained_mismatch(
    request: &liquidfun_test_protocol::RigidWorldRequestRecord,
    native: &RigidWorldResultRecord,
    oracle: &RigidWorldResultRecord,
) -> Box<RigidMismatchReport> {
    let (phase6, phase7, phase8) = retained_profiles();
    let outcome =
        compare_phase8_rigid_world_results(request, native, oracle, &phase6, &phase7, &phase8)
            .expect("request-valid retained mutation should compare");
    let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
        panic!("retained mutation must produce a Phase 8 physics mismatch");
    };
    report
}

fn assert_complete_retained_signature(
    request: &liquidfun_test_protocol::RigidWorldRequestRecord,
    native: &RigidWorldResultRecord,
    oracle: &RigidWorldResultRecord,
    expected_path: &str,
) {
    let expected = expected_retained_mismatch(request, native, oracle);
    let outcome = compare_complete_phase9_rigid_world_results(request, native, oracle)
        .expect("request-valid retained mutation should compare");
    let Phase9ComparisonOutcome::RetainedRigidMismatch(actual) = outcome else {
        panic!("retained mutation must win at {expected_path}");
    };
    assert_eq!(expected.semantic_path(), expected_path);
    assert_eq!(actual.signature(), expected.signature());
}
