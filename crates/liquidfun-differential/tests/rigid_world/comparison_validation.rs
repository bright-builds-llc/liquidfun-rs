#[test]
fn comparison_validates_each_engine_declaration_before_cross_engine_fields() {
    // Arrange
    let request = comparison_request();
    let profile = profile();
    let native = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound request should execute natively");
    let mut oracle_value = result_value(&native);
    oracle_value["timelines"][0]["checkpoints"][0]["counts"]["bodies"] = json!(2);
    oracle_value["timelines"][0]["checkpoints"][0]["bodies"]
        .as_array_mut()
        .expect("body snapshots should be an array")
        .pop();
    let oracle = decode_result_value(&oracle_value);

    // Act
    let result = compare_rigid_world_results(&request, &native, &oracle, &profile);

    // Assert
    let Err(RigidComparisonFailure::Declaration(report)) = result else {
        panic!("declaration disagreement must precede cross-engine comparison");
    };
    assert_eq!(report.action_id(), "nc-create-dynamic-fixture");
    assert_eq!(report.checkpoint_id(), "nc-created");
    assert_eq!(report.semantic_path(), "rigid_world.checkpoint.counts");
}

#[test]
fn comparison_rejects_omitted_phase7_observations_on_each_engine_side() {
    // Arrange
    let request = comparison_request();
    let profile = profile();
    let complete = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound request should execute natively");
    let mut omitted_value = result_value(&complete);
    omitted_value["timelines"][2]["checkpoints"][0]["observations"] = json!([]);
    let omitted = decode_result_value(&omitted_value);

    // Act
    let native_error = compare_rigid_world_results(&request, &omitted, &complete, &profile);
    let oracle_error = compare_rigid_world_results(&request, &complete, &omitted, &profile);

    // Assert
    assert!(matches!(
        native_error,
        Err(RigidComparisonFailure::Harness(_))
    ));
    assert!(matches!(
        oracle_error,
        Err(RigidComparisonFailure::Harness(_))
    ));
}

#[test]
fn comparison_rejects_invalid_all_continue_query_termination_on_each_engine_side() {
    // Arrange
    let request = comparison_request();
    let profile = profile();
    let complete = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound request should execute natively");
    let mut mutated_value = result_value(&complete);
    let observations = mutated_value["timelines"][7]["checkpoints"][0]["observations"]
        .as_array_mut()
        .expect("query checkpoint should contain observations");
    let query = observations
        .iter_mut()
        .find(|observation| {
            observation["kind"] == "query"
                && observation["observation"]["completion"] == "exhausted"
        })
        .expect("all-continue query observation should exist");
    query["observation"]["completion"] = json!("terminated");
    let mutated = decode_result_value(&mutated_value);

    // Act and Assert
    assert_observation_rejected_on_each_side(&request, &complete, &mutated, &profile);
}

#[test]
fn comparison_rejects_query_occurrence_removed_by_body_cascade_on_each_engine_side() {
    // Arrange
    let request = comparison_request_with_cascade_query_checkpoint();
    let profile = profile();
    let complete = NativeRigidWorldExecutor::execute(&request)
        .expect("request with a post-cascade query should execute natively");
    let mut mutated_value = result_value(&complete);
    let occurrences = mutated_value["timelines"][7]["checkpoints"][1]["observations"][0]
        ["observation"]["occurrences"]
        .as_array_mut()
        .expect("post-cascade query should contain occurrences");
    occurrences.push(json!({
        "fixture_id": "query-right-fixture",
        "child_index": 0
    }));
    let mutated = decode_result_value(&mutated_value);

    // Act and Assert
    assert_observation_rejected_on_each_side(&request, &complete, &mutated, &profile);
}

#[test]
fn comparison_rejects_unknown_ray_hit_identity_on_each_engine_side() {
    // Arrange
    let request = comparison_request();
    let profile = profile();
    let complete = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound request should execute natively");
    let mut mutated_value = result_value(&complete);
    let observations = mutated_value["timelines"][7]["checkpoints"][0]["observations"]
        .as_array_mut()
        .expect("query checkpoint should contain observations");
    let ray = observations
        .iter_mut()
        .find(|observation| {
            observation["kind"] == "ray_cast"
                && observation["observation"]["hits"]
                    .as_array()
                    .is_some_and(|hits| !hits.is_empty())
        })
        .expect("ray observation with a hit should exist");
    ray["observation"]["hits"][0]["fixture_id"] = json!("unknown-ray-fixture");
    let mutated = decode_result_value(&mutated_value);

    // Act and Assert
    assert_observation_rejected_on_each_side(&request, &complete, &mutated, &profile);
}

#[test]
fn comparison_rejects_every_non_finite_ray_hit_coordinate_on_each_engine_side() {
    // Arrange
    let request = comparison_request();
    let profile = profile();
    let complete = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound request should execute natively");
    let complete_value = result_value(&complete);

    // Act and Assert
    for vector in ["point", "normal"] {
        for coordinate in ["x_bits", "y_bits"] {
            for invalid_bits in [
                f32::NAN.to_bits(),
                f32::INFINITY.to_bits(),
                f32::NEG_INFINITY.to_bits(),
            ] {
                let mut mutated_value = complete_value.clone();
                let observations = mutated_value["timelines"][7]["checkpoints"][0]["observations"]
                    .as_array_mut()
                    .expect("query checkpoint should contain observations");
                let ray = observations
                    .iter_mut()
                    .find(|observation| {
                        observation["kind"] == "ray_cast"
                            && observation["observation"]["hits"]
                                .as_array()
                                .is_some_and(|hits| !hits.is_empty())
                    })
                    .expect("ray observation with a hit should exist");
                ray["observation"]["hits"][0][vector][coordinate] = json!(invalid_bits);
                let mutated = decode_result_value(&mutated_value);
                assert_observation_rejected_on_each_side(&request, &complete, &mutated, &profile);
            }
        }
    }
}

#[test]
fn comparison_rejects_invalid_child_hit_before_valid_ray_termination_on_each_engine_side() {
    // Arrange
    let request = comparison_request();
    let profile = profile();
    let complete = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound request should execute natively");
    let mut mutated_value = result_value(&complete);
    let observations = mutated_value["timelines"][7]["checkpoints"][0]["observations"]
        .as_array_mut()
        .expect("query checkpoint should contain observations");
    let ray = observations
        .iter_mut()
        .find(|observation| {
            observation["kind"] == "ray_cast"
                && observation["observation"]["completion"] == "terminated"
        })
        .expect("terminated ray observation should exist");
    let hits = ray["observation"]["hits"]
        .as_array_mut()
        .expect("terminated ray should contain hits");
    let mut fabricated = hits
        .first()
        .expect("terminated ray should contain its terminating hit")
        .clone();
    fabricated["child_index"] = json!(1);
    hits.insert(0, fabricated);
    let mutated = decode_result_value(&mutated_value);

    // Act and Assert
    assert_observation_rejected_on_each_side(&request, &complete, &mutated, &profile);
}

#[test]
fn comparison_rejects_same_count_stale_body_identities_on_each_engine_side() {
    // Arrange
    let request = comparison_request_with_partial_body_checkpoint();
    let profile = profile();
    let complete = NativeRigidWorldExecutor::execute(&request)
        .expect("request with partial body destruction should execute");
    let mut mutated_value = result_value(&complete);
    let bodies = mutated_value["timelines"][0]["checkpoints"][7]["bodies"]
        .as_array_mut()
        .expect("partial destruction checkpoint should contain body snapshots");
    assert_eq!(bodies[0]["body_id"], json!("nc-kinematic"));
    assert_eq!(bodies[1]["body_id"], json!("nc-dynamic"));
    bodies[0]["body_id"] = json!("nc-static");
    bodies[1]["body_id"] = json!("nc-kinematic");
    let mutated = decode_result_value(&mutated_value);

    // Act and Assert
    assert_identity_rejected_on_each_side(
        &request,
        &complete,
        &mutated,
        &profile,
        "rigid_world.checkpoint.bodies.declaration_order",
    );
}

#[test]
fn comparison_rejects_same_count_stale_fixture_identities_on_each_engine_side() {
    // Arrange
    let request = comparison_request();
    let profile = profile();
    let complete = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound request should execute natively");
    let mut mutated_value = result_value(&complete);
    let fixtures = mutated_value["timelines"][1]["checkpoints"][8]["fixtures"]
        .as_array_mut()
        .expect("fixture-destruction checkpoint should contain one fixture snapshot");
    assert_eq!(fixtures[0]["fixture_id"], json!("contact-static-fixture"));
    fixtures[0]["fixture_id"] = json!("contact-dynamic-fixture");
    let mutated = decode_result_value(&mutated_value);

    // Act and Assert
    assert_identity_rejected_on_each_side(
        &request,
        &complete,
        &mutated,
        &profile,
        "rigid_world.checkpoint.fixtures.declaration_order",
    );
}

#[test]
fn checkpoint_live_identities_apply_body_destruction_fixture_cascades() {
    // Arrange
    let mut value = serde_json::from_slice::<Value>(REQUEST).expect("fixture should be JSON");
    value["scenario"]["timelines"][8]["checkpoints"]
        .as_array_mut()
        .expect("origin checkpoints should be an array")
        .push(json!({
            "checkpoint_id": "origin-right-destroyed",
            "after_action_id": "origin-09",
            "phase": "teardown",
            "counts": {
                "bodies": 1,
                "fixtures": 1,
                "contacts": 0,
                "manifold_points": 0,
                "events": 0,
                "destructions": 2
            },
            "transitions": []
        }));
    let request = decode_rigid_world_request_jsonl(
        &encode_value(&value),
        &HarnessLimits::phase2_default_v1(),
    )
    .expect("cascade checkpoint should decode");

    // Act
    let identities = rigid_world_checkpoint_live_identities(&request.scenario().timelines()[8], 1)
        .expect("validated cascade checkpoint should have live identities");

    // Assert
    assert_eq!(
        identities
            .body_ids()
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>(),
        ["origin-left"]
    );
    assert_eq!(
        identities
            .fixture_ids()
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>(),
        ["origin-left-fixture"]
    );
}
