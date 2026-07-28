#[test]
fn rigid_comparator_treats_queries_as_multiplicity_preserving_multisets() {
    // Arrange
    let (phase6, phase7) = profiles();
    let request = request_with_query_rules(&phase7, json!([]));
    let baseline = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound Phase 7 request should execute");
    let mut native_value = serde_json::to_value(&baseline).expect("result should serialize");
    let query = observation_mut(phase7_observations(&mut native_value), "query");
    let occurrences = query["observation"]["occurrences"]
        .as_array_mut()
        .expect("query occurrences should be an array");
    let duplicate = occurrences[0].clone();
    occurrences.push(duplicate);
    let native = decode_result(&native_value);
    let mut oracle_value = native_value.clone();
    oracle_value["timelines"][0]["checkpoints"][6]["observations"]
        .as_array_mut()
        .expect("observations should be an array")
        .iter_mut()
        .find(|observation| observation["kind"] == "query")
        .expect("query should exist")["observation"]["occurrences"]
        .as_array_mut()
        .expect("query occurrences should be an array")
        .reverse();
    let oracle = decode_result(&oracle_value);

    // Act
    let reordered =
        compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
            .expect("registered Phase 7 fields should compare");
    let occurrences = observation_mut(phase7_observations(&mut oracle_value), "query")
        ["observation"]["occurrences"]
        .as_array_mut()
        .expect("query occurrences should be an array");
    occurrences.pop();
    let missing_duplicate = compare_phase7_rigid_world_results(
        &request,
        &native,
        &decode_result(&oracle_value),
        &phase6,
        &phase7,
    )
    .expect("registered Phase 7 fields should compare");

    // Assert
    assert_eq!(reordered, RigidComparisonOutcome::Match);
    let RigidComparisonOutcome::PhysicsMismatch(report) = missing_duplicate else {
        panic!("removing one duplicate occurrence must mismatch");
    };
    assert_eq!(report.kind(), RigidMismatchKind::Order);
    assert_eq!(
        report.semantic_path(),
        "rigid_world.phase7.query.occurrences.identity"
    );
}

#[test]
fn rigid_comparator_reports_action_stage_values_policy_and_completion_context() {
    // Arrange
    let (phase6, phase7) = profiles();
    let request = request(&phase7);
    let native = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound Phase 7 request should execute");
    let mut oracle_value = serde_json::to_value(&native).expect("result should serialize");
    let step = observation_mut(phase7_observations(&mut oracle_value), "step");
    step["outcome"]["completion"] = json!("continuous_pending");
    let oracle = decode_result(&oracle_value);

    // Act
    let outcome = compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
        .expect("registered Phase 7 fields should compare");

    // Assert
    let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
        panic!("completion mutation must mismatch");
    };
    assert_eq!(report.action_id(), "phase7-action-18");
    assert_eq!(report.stage(), "phase7-adapter");
    assert_eq!(report.maybe_entity(), None);
    assert_eq!(report.semantic_path(), "rigid_world.phase7.step.completion");
    assert_eq!(report.expected(), "Complete");
    assert_eq!(report.actual(), "ContinuousPending");
    assert_eq!(report.policy().comparison(), FieldComparison::ExactDiscrete);
    assert!(report.maybe_completion_context().is_some());
}

#[test]
fn rigid_comparator_compares_equal_fraction_ray_hits_as_multisets() {
    // Arrange
    let (phase6, phase7) = profiles();
    let request = request(&phase7);
    let baseline = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound Phase 7 request should execute");
    let mut native_value = serde_json::to_value(&baseline).expect("result should serialize");
    let ray = observation_mut(phase7_observations(&mut native_value), "ray_cast");
    ray["observation"]["hits"] = json!([
        {
            "fixture_id": "nc-static-fixture",
            "child_index": 0,
            "point": { "x_bits": 0.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "normal": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "fraction_bits": 0.5_f32.to_bits()
        },
        {
            "fixture_id": "nc-dynamic-fixture",
            "child_index": 0,
            "point": { "x_bits": 0.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "normal": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "fraction_bits": 0.5_f32.to_bits()
        }
    ]);
    let native = decode_result(&native_value);
    let mut oracle_value = native_value;
    observation_mut(phase7_observations(&mut oracle_value), "ray_cast")["observation"]["hits"]
        .as_array_mut()
        .expect("ray hits should be an array")
        .reverse();
    let oracle = decode_result(&oracle_value);

    // Act
    let outcome = compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
        .expect("registered Phase 7 fields should compare");

    // Assert
    assert_eq!(outcome, RigidComparisonOutcome::Match);
}

#[test]
fn rigid_comparator_reassigns_adversarial_duplicate_hits_in_both_actual_orders() {
    // Arrange
    let (phase6, phase7) = profiles();
    let base = 0.5_f32.to_bits();
    let expected_fractions = [base, base - 4];
    let actual_orders = [[base - 2, base + 4], [base + 4, base - 2]];

    // Act and Assert
    for actual_fractions in actual_orders {
        let (request, expected, actual) =
            duplicate_ray_results(&phase7, &expected_fractions, &actual_fractions);
        let outcome =
            compare_phase7_rigid_world_results(&request, &expected, &actual, &phase6, &phase7)
                .expect("registered Phase 7 fields should compare");
        assert_eq!(outcome, RigidComparisonOutcome::Match);
    }
}

#[test]
fn rigid_comparator_reports_stable_fraction_when_no_perfect_matching_exists() {
    // Arrange
    let (phase6, phase7) = profiles();
    let base = 0.5_f32.to_bits();
    let expected_fractions = [base, base - 4];
    let actual_orders = [[base - 2, base + 5], [base + 5, base - 2]];
    let mut reports = Vec::new();

    // Act
    for actual_fractions in actual_orders {
        let (request, expected, actual) =
            duplicate_ray_results(&phase7, &expected_fractions, &actual_fractions);
        let outcome =
            compare_phase7_rigid_world_results(&request, &expected, &actual, &phase6, &phase7)
                .expect("registered Phase 7 fields should compare");
        let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
            panic!("a duplicate-hit group without a perfect matching must mismatch");
        };
        reports.push((
            report.signature().clone(),
            report.maybe_expected_bits(),
            report.maybe_actual_bits(),
        ));
    }

    // Assert
    assert_eq!(reports[0], reports[1]);
    assert_eq!(
        reports[0].0.semantic_path(),
        "rigid_world.phase7.ray.fraction"
    );
    assert_ne!(reports[0].1, reports[0].2);
    assert_eq!(reports[0].1.map(FloatBits::bits), Some(base));
    assert_eq!(reports[0].2.map(FloatBits::bits), Some(base + 5));
}
