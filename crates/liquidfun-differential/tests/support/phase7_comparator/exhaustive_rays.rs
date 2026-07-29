#[test]
fn rigid_comparator_uses_exhaustive_semantics_when_declared_clip_is_not_applied() {
    // Arrange
    let (phase6, phase7) = profiles();
    let request = request_with_out_of_ray_clip(&phase7);
    let baseline = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound out-of-ray clip request should execute");
    let mut native_value = serde_json::to_value(&baseline).expect("result should serialize");
    let ray = observation_mut(phase7_observations(&mut native_value), "ray_cast");
    assert_eq!(
        ray["observation"]["final_max_fraction_bits"],
        json!(1.0_f32.to_bits())
    );
    ray["observation"]["hits"] = json!([
        {
            "fixture_id": "nc-static-fixture",
            "child_index": 0,
            "point": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() },
            "normal": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "fraction_bits": 0.25_f32.to_bits()
        },
        {
            "fixture_id": "nc-kinematic-fixture",
            "child_index": 0,
            "point": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "normal": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() },
            "fraction_bits": 0.75_f32.to_bits()
        }
    ]);
    let native = decode_result(&native_value);
    let mut oracle_value = native_value;
    observation_mut(phase7_observations(&mut oracle_value), "ray_cast")["observation"]["hits"][1]
        ["point"]["x_bits"] = json!(2.0_f32.to_bits());
    let oracle = decode_result(&oracle_value);

    // Act
    let outcome = compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
        .expect("registered Phase 7 fields should compare");

    // Assert
    let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
        panic!("a nonminimum hit mismatch must remain visible when clipping was not applied");
    };
    assert_eq!(report.semantic_path(), "rigid_world.phase7.ray.point.x");
}

#[test]
fn rigid_comparator_uses_exhaustive_semantics_for_reached_noop_clip() {
    // Arrange
    let (phase6, phase7) = profiles();
    let request = request_with_ray_rules(
        &phase7,
        json!([{
            "target": { "fixture_id": "nc-dynamic-fixture", "child_index": 0 },
            "directive": { "kind": "clip", "fraction_bits": 1.0_f32.to_bits() }
        }]),
    );
    let baseline = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound no-op clip request should execute");
    let mut native_value = serde_json::to_value(&baseline).expect("result should serialize");
    let ray = observation_mut(phase7_observations(&mut native_value), "ray_cast");
    assert_eq!(
        ray["observation"]["final_max_fraction_bits"],
        json!(1.0_f32.to_bits())
    );
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
    observation_mut(phase7_observations(&mut oracle_value), "ray_cast")["observation"]["hits"][1]
        ["point"]["x_bits"] = json!(2.0_f32.to_bits());
    let oracle = decode_result(&oracle_value);

    // Act
    let outcome = compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
        .expect("registered Phase 7 fields should compare");

    // Assert
    let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
        panic!("a reached no-op clip must not hide a nonminimum mismatch");
    };
    assert_eq!(report.semantic_path(), "rigid_world.phase7.ray.point.x");
}

#[test]
fn rigid_comparator_reports_final_ray_interval_disagreement_first() {
    // Arrange
    let (phase6, phase7) = profiles();
    let request = request(&phase7);
    let baseline = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound strict clip request should execute");
    let mut native_value = serde_json::to_value(&baseline).expect("result should serialize");
    let ray = observation_mut(phase7_observations(&mut native_value), "ray_cast");
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
    let oracle_ray = observation_mut(phase7_observations(&mut oracle_value), "ray_cast");
    oracle_ray["observation"]["hits"]
        .as_array_mut()
        .expect("ray hits should be an array")
        .pop();
    oracle_ray["observation"]["final_max_fraction_bits"] = json!(1.0_f32.to_bits());
    let oracle = decode_result(&oracle_value);

    // Act
    let outcome = compare_phase7_rigid_world_results(&request, &native, &oracle, &phase6, &phase7)
        .expect("registered Phase 7 fields should compare");

    // Assert
    let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
        panic!("different validated final intervals must mismatch");
    };
    assert_eq!(
        report.semantic_path(),
        "rigid_world.phase7.ray.final_max_fraction"
    );
}
