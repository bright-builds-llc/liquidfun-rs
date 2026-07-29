#[test]
fn result_accepts_unmodified_native_phase9_action_contracts() {
    // Arrange
    let request = phase9_result_request();

    // Act
    let result = NativeRigidWorldExecutor::execute(&request);

    // Assert
    let result = result.expect("native Phase 9 result should satisfy its exact action contract");
    validate_rigid_world_result_against_request(&request, &result)
        .expect("unmodified native result should remain request-bound valid");
}

#[test]
fn result_rejects_wrong_phase9_nested_variant_and_statistics_owner() {
    // Arrange
    let request = phase9_result_request();
    let result = NativeRigidWorldExecutor::execute(&request)
        .expect("baseline Phase 9 result should execute");
    let mut wrong_variant = result_value(&result);
    let statistics =
        particle_observation_mut(phase9_observations_mut(&mut wrong_variant), "statistics");
    statistics["observation"] = json!({ "kind": "query", "terminated": false, "particle_ids": [] });
    let mut wrong_owner = result_value(&result);
    particle_observation_mut(phase9_observations_mut(&mut wrong_owner), "statistics")["observation"]
        ["statistics"]["maybe_system_id"] = json!("phase9-system-a");

    // Act / Assert
    assert_result_observation_mismatch(&request, &wrong_variant);
    assert_result_observation_mismatch(&request, &wrong_owner);
}

#[test]
fn result_rejects_unknown_wrong_owner_and_duplicate_query_particles() {
    // Arrange
    let request = phase9_result_request();
    let result = NativeRigidWorldExecutor::execute(&request)
        .expect("baseline Phase 9 result should execute");
    let mut unknown = result_value(&result);
    particle_observation_mut(phase9_observations_mut(&mut unknown), "query")["observation"]["particle_ids"] =
        json!(["unknown-particle"]);
    let mut wrong_owner = result_value(&result);
    particle_observation_mut(phase9_observations_mut(&mut wrong_owner), "query")["observation"]["particle_ids"] =
        json!(["phase9-particle-a"]);
    let mut duplicate = result_value(&result);
    particle_observation_mut(phase9_observations_mut(&mut duplicate), "query")["observation"]["particle_ids"] =
        json!(["phase9-particle-b", "phase9-particle-b"]);

    // Act / Assert
    assert_result_observation_mismatch(&request, &unknown);
    assert_result_observation_mismatch(&request, &wrong_owner);
    assert_result_observation_mismatch(&request, &duplicate);
}

#[test]
fn result_rejects_reordered_future_and_stale_mixed_particle_identities() {
    // Arrange
    let request = phase9_result_request();
    let result = NativeRigidWorldExecutor::execute(&request)
        .expect("baseline Phase 9 result should execute");
    let mut reordered = result_value(&result);
    mixed_observation_with_particles_mut(
        phase9_observations_mut(&mut reordered),
        &["phase9-particle-a", "phase9-particle-b"],
    )["observation"]["particle_ids"] = json!(["phase9-particle-b", "phase9-particle-a"]);
    let mut future = result_value(&result);
    mixed_observation_with_particles_mut(phase9_observations_mut(&mut future), &[])["observation"]
        ["particle_ids"] = json!(["phase9-particle-b"]);
    let mut stale = result_value(&result);
    mixed_observation_with_particles_mut(
        phase9_observations_mut(&mut stale),
        &["phase9-particle-b"],
    )["observation"]["particle_ids"] = json!(["phase9-particle-a", "phase9-particle-b"]);

    // Act / Assert
    assert_result_observation_mismatch(&request, &reordered);
    assert_result_observation_mismatch(&request, &future);
    assert_result_observation_mismatch(&request, &stale);
}

#[test]
fn result_rejects_ray_parallel_length_and_extra_particle_observation() {
    // Arrange
    let request = phase9_result_request();
    let result = NativeRigidWorldExecutor::execute(&request)
        .expect("baseline Phase 9 result should execute");
    let mut wrong_length = result_value(&result);
    particle_observation_mut(phase9_observations_mut(&mut wrong_length), "ray_cast")["observation"]
        ["fractions_bits"] = json!([]);
    let mut extra = result_value(&result);
    let observations = phase9_observations_mut(&mut extra);
    let duplicate = observations
        .iter()
        .find(|observation| observation["kind"] == "particle")
        .cloned()
        .expect("baseline should contain a particle observation");
    observations.push(duplicate);

    // Act / Assert
    assert!(
        decode_result_value(&wrong_length).is_err(),
        "parallel ray arrays must fail bounded result decoding"
    );
    assert_result_observation_mismatch(&request, &extra);
}
