#[test]
fn rigid_comparator_termination_observes_only_status_and_hit_count() {
    // Arrange
    let (phase6, phase7) = profiles();
    let request = request_with_ray_rules(
        &phase7,
        json!([{
            "target": { "fixture_id": "nc-dynamic-fixture", "child_index": 0 },
            "directive": { "kind": "terminate" }
        }]),
    );
    let baseline = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound terminating ray request should execute");
    let mut native_value = serde_json::to_value(&baseline).expect("result should serialize");
    let ray = observation_mut(phase7_observations(&mut native_value), "ray_cast");
    ray["observation"]["completion"] = json!("terminated");
    ray["observation"]["hits"] = json!([{
        "fixture_id": "nc-dynamic-fixture",
        "child_index": 0,
        "point": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() },
        "normal": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
        "fraction_bits": 0.25_f32.to_bits()
    }]);
    let native = decode_result(&native_value);
    let mut oracle_value = native_value;
    let oracle_hit = &mut observation_mut(phase7_observations(&mut oracle_value), "ray_cast")["observation"]
        ["hits"][0];
    oracle_hit["point"]["x_bits"] = json!(100.0_f32.to_bits());
    let oracle = decode_result(&oracle_value);

    // Act
    let outcome = compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
        .expect("registered Phase 7 fields should compare");

    // Assert
    assert_eq!(outcome, RigidComparisonOutcome::Match);
}

#[test]
fn rigid_comparator_does_not_reapply_inherited_exact_bits_after_phase7_numeric_match() {
    // Arrange
    let (phase6, phase7) = profiles();
    let request = request(&phase7);
    let native = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound Phase 7 request should execute");
    let mut oracle_value = serde_json::to_value(&native).expect("result should serialize");
    let position_bits = oracle_value["timelines"][3]["checkpoints"][0]["bodies"][1]
        ["transform"]["position"]["x_bits"]
        .as_u64()
        .expect("island position should use exact float bits");
    oracle_value["timelines"][3]["checkpoints"][0]["bodies"][1]["transform"]["position"]["x_bits"] =
        json!(position_bits + 1);
    let oracle = decode_result(&oracle_value);

    // Act
    let outcome = compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
        .expect("registered Phase 7 fields should compare");

    // Assert
    assert_eq!(outcome, RigidComparisonOutcome::Match);
}

#[test]
fn rigid_minimization_preserves_divergent_action_setup_directives_budget_and_bits() {
    // Arrange
    let (phase6, phase7) = profiles();
    let request = request(&phase7);
    let native = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound Phase 7 request should execute");
    let mut oracle_value = serde_json::to_value(&native).expect("result should serialize");
    let ray = observation_mut(phase7_observations(&mut oracle_value), "ray_cast");
    for hit in ray["observation"]["hits"]
        .as_array_mut()
        .expect("ray hits should be an array")
    {
        hit["point"]["x_bits"] = json!(100.0_f32.to_bits());
    }
    let oracle = decode_result(&oracle_value);
    let RigidComparisonOutcome::PhysicsMismatch(report) =
        compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
            .expect("registered Phase 7 fields should compare")
    else {
        panic!("ray-point mutation must mismatch");
    };
    assert_eq!(report.action_id(), "phase7-action-20");
    assert!(
        report
            .maybe_entity()
            .is_some_and(|entity| entity.ends_with(":0"))
    );
    assert!(report.maybe_expected_bits().is_some());
    assert!(report.maybe_actual_bits().is_some());
    assert!(report.maybe_expected_decimal().is_some());
    assert!(report.maybe_actual_decimal().is_some());
    assert!(report.maybe_completion_context().is_some());
    let target = report.signature().clone();
    let original = serde_json::to_value(&request).expect("request should serialize");

    // Act
    let result = minimize_rigid_world_request(
        &request,
        &target,
        MinimizationBudget::new(256, Duration::from_secs(1)),
        |_candidate| RigidEvaluation::new(Some(target.clone()), Duration::from_millis(1)),
    )
    .expect("Phase 7 minimization should retain its exact failure class");

    // Assert
    let minimized = serde_json::to_value(result.request()).expect("request should serialize");
    for action_id in ["nc-create-dynamic", "phase7-action-18", "phase7-action-20"] {
        assert_eq!(
            action(&minimized, action_id),
            action(&original, action_id),
            "required setup and divergent operations must remain bit-identical"
        );
    }
    assert_eq!(target.action_id(), "phase7-action-20");
    assert_eq!(target.kind(), RigidMismatchKind::Numeric);
}

#[test]
fn second_checkpoint_evidence_and_minimization_use_its_local_action_window() {
    // Arrange
    let (phase6, phase7) = profiles();
    let request = request_with_split_phase7_checkpoints(&phase7);
    let native = NativeRigidWorldExecutor::execute(&request)
        .expect("split Phase 7 checkpoint request should execute");
    let mut oracle_value = serde_json::to_value(&native).expect("result should serialize");
    let ray = observation_mut(
        checkpoint_observations_mut(&mut oracle_value, "nc-fixtures-destroyed"),
        "ray_cast",
    );
    for hit in ray["observation"]["hits"]
        .as_array_mut()
        .expect("ray hits should be an array")
    {
        hit["point"]["x_bits"] = json!(100.0_f32.to_bits());
    }
    let oracle = decode_result(&oracle_value);
    let RigidComparisonOutcome::PhysicsMismatch(report) =
        compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
            .expect("registered Phase 7 fields should compare")
    else {
        panic!("second-checkpoint ray mutation must mismatch");
    };
    let target = report.signature().clone();
    let original = serde_json::to_value(&request).expect("request should serialize");

    // Act
    let result = minimize_rigid_world_request(
        &request,
        &target,
        MinimizationBudget::new(256, Duration::from_secs(1)),
        |_candidate| RigidEvaluation::new(Some(target.clone()), Duration::from_millis(1)),
    )
    .expect("minimization should preserve the second-checkpoint signature");

    // Assert
    assert_eq!(target.checkpoint_id(), "nc-fixtures-destroyed");
    assert_eq!(target.action_id(), "phase7-action-20");
    assert_eq!(report.stage(), "phase7-adapter");
    let minimized = serde_json::to_value(result.request()).expect("request should serialize");
    for action_id in [
        "nc-create-dynamic",
        "phase7-action-18",
        "phase7-action-19",
        "phase7-action-20",
    ] {
        assert_eq!(
            action(&minimized, action_id),
            action(&original, action_id),
            "the second-checkpoint protected prefix must remain bit-identical"
        );
    }
}

fn action<'a>(request: &'a Value, action_id: &str) -> &'a Value {
    request["scenario"]["timelines"][0]["actions"]
        .as_array()
        .expect("actions should be an array")
        .iter()
        .find(|record| record["action_id"] == action_id)
        .expect("required action should remain present")
}
