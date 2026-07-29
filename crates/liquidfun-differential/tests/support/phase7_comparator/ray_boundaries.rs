#[test]
fn rigid_comparator_preserves_duplicate_ray_hit_multiplicity() {
    // Arrange
    let (phase6, phase7) = profiles();
    let fraction = 0.5_f32.to_bits();
    let (request, expected, actual) =
        duplicate_ray_results(&phase7, &[fraction, fraction], &[fraction]);

    // Act
    let outcome =
        compare_phase7_rigid_world_results(&request, &expected, &actual, &phase6, &phase7)
            .expect("registered Phase 7 fields should compare");

    // Assert
    let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
        panic!("removing one duplicate ray hit must mismatch");
    };
    assert_eq!(report.kind(), RigidMismatchKind::Order);
    assert_eq!(
        report.semantic_path(),
        "rigid_world.phase7.ray.hit.identity"
    );
}

#[test]
fn rigid_comparator_ignores_reversed_pre_clip_history_above_the_final_interval() {
    // Arrange
    let (phase6, phase7) = profiles();
    let (request, native, oracle) = arbitrary_clip_order_results(&phase7);

    // Act
    let outcome = compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
        .expect("valid arbitrary clip histories should compare");

    // Assert
    assert_eq!(outcome, RigidComparisonOutcome::Match);
}

#[test]
fn rigid_comparator_reports_mismatch_inside_the_final_interval() {
    // Arrange
    let (phase6, phase7) = profiles();
    let (request, native, oracle) = arbitrary_clip_order_results(&phase7);
    let mut oracle_value = serde_json::to_value(&oracle).expect("result should serialize");
    let ray = observation_mut(phase7_observations(&mut oracle_value), "ray_cast");
    let hit = ray["observation"]["hits"]
        .as_array_mut()
        .expect("ray hits should be an array")
        .iter_mut()
        .find(|hit| hit["fixture_id"] == "nc-static-fixture")
        .expect("inside-interval hit should exist");
    hit["point"]["x_bits"] = json!((-0.5_f32).to_bits());
    let oracle = decode_result(&oracle_value);

    // Act
    let outcome = compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
        .expect("valid arbitrary clip histories should compare");

    // Assert
    let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
        panic!("inside-interval numeric divergence must mismatch");
    };
    assert_eq!(report.semantic_path(), "rigid_world.phase7.ray.point.x");
}

#[test]
fn rigid_comparator_retains_boundary_tolerant_ray_hits_in_both_engine_directions() {
    // Arrange
    let (phase6, phase7) = profiles();
    let boundary = 0.1_f32.to_bits();

    // Act and Assert
    for ulps in 0..=4 {
        for (expected_fraction, actual_fraction) in
            [(boundary, boundary + ulps), (boundary + ulps, boundary)]
        {
            let (request, expected, actual) =
                boundary_clip_results(&phase7, expected_fraction, actual_fraction);
            let outcome =
                compare_phase7_rigid_world_results(&request, &expected, &actual, &phase6, &phase7)
                    .expect("registered Phase 7 fields should compare");
            assert_eq!(outcome, RigidComparisonOutcome::Match);
        }
    }
}

#[test]
fn rigid_comparator_discards_ray_hits_proven_beyond_the_boundary_policy() {
    // Arrange
    let (phase6, phase7) = profiles();
    let boundary = 0.1_f32.to_bits();
    let (request, expected, actual) = boundary_clip_results(&phase7, boundary + 5, boundary + 9);
    let actual = with_single_ray_point_x(&actual, 2.0_f32.to_bits());

    // Act
    let outcome =
        compare_phase7_rigid_world_results(&request, &expected, &actual, &phase6, &phase7)
            .expect("registered Phase 7 fields should compare");

    // Assert
    assert_eq!(outcome, RigidComparisonOutcome::Match);

    for (expected_fraction, actual_fraction) in [(boundary, boundary + 5), (boundary + 5, boundary)]
    {
        let (request, expected, actual) =
            boundary_clip_results(&phase7, expected_fraction, actual_fraction);
        let outcome =
            compare_phase7_rigid_world_results(&request, &expected, &actual, &phase6, &phase7)
                .expect("registered Phase 7 fields should compare");
        let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
            panic!("a five-ULP boundary straddle must mismatch");
        };
        assert_eq!(report.kind(), RigidMismatchKind::Order);
        assert_eq!(
            report.semantic_path(),
            "rigid_world.phase7.ray.hit.identity"
        );
    }
}

#[test]
fn rigid_comparator_compares_payloads_retained_by_the_boundary_policy() {
    // Arrange
    let (phase6, phase7) = profiles();
    let boundary = 0.1_f32.to_bits();
    let (request, expected, actual) = boundary_clip_results(&phase7, boundary, boundary + 4);
    let actual = with_single_ray_point_x(&actual, 2.0_f32.to_bits());

    // Act
    let outcome =
        compare_phase7_rigid_world_results(&request, &expected, &actual, &phase6, &phase7)
            .expect("registered Phase 7 fields should compare");

    // Assert
    let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
        panic!("retained boundary-band payload divergence must mismatch");
    };
    assert_eq!(report.semantic_path(), "rigid_world.phase7.ray.point.x");
}

#[test]
fn rigid_comparator_treats_exhaustive_ray_hits_as_record_multisets() {
    // Arrange
    let (phase6, phase7) = profiles();
    let request = request_with_ray_rules(&phase7, json!([]));
    let baseline = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound exhaustive ray request should execute");
    let mut native_value = serde_json::to_value(&baseline).expect("result should serialize");
    let ray = observation_mut(phase7_observations(&mut native_value), "ray_cast");
    ray["observation"]["completion"] = json!("exhausted");
    ray["observation"]["hits"] = json!([
        {
            "fixture_id": "nc-static-fixture",
            "child_index": 0,
            "point": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() },
            "normal": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "fraction_bits": 0.25_f32.to_bits()
        },
        {
            "fixture_id": "nc-dynamic-fixture",
            "child_index": 0,
            "point": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "normal": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() },
            "fraction_bits": 0.75_f32.to_bits()
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
