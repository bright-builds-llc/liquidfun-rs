#[test]
fn native_executes_all_locked_families_deterministically_and_resets() {
    // Arrange
    let request = request();

    // Act
    let first = NativeRigidWorldExecutor::execute(&request)
        .expect("validated rigid-world request should execute natively");
    let second = NativeRigidWorldExecutor::execute(&request)
        .expect("a fresh native execution should reset all world state");

    // Assert
    assert_eq!(first, second);
    assert_eq!(first.timelines().len(), RigidWorldWitnessFamily::ALL.len());
    assert_eq!(first.timelines()[0].checkpoints.len(), 8);
    assert_eq!(first.timelines()[1].checkpoints.len(), 10);
    validate_native_rigid_world_result(&request, &first)
        .expect("native result should agree with every declaration");
}

#[test]
fn native_contract_executes_the_exact_fixed_step_tuple() {
    // Arrange
    let request = request();

    // Act
    let result = NativeRigidWorldExecutor::execute(&request)
        .expect("the validated fixed tuple should execute natively");

    // Assert
    assert_eq!(result.timelines().len(), RigidWorldWitnessFamily::ALL.len());
}

#[test]
fn native_executes_closed_phase7_actions_and_emits_semantic_observations() {
    // Arrange
    let request = support::phase7_request();

    // Act
    let result = NativeRigidWorldExecutor::execute(&request)
        .expect("validated Phase 7 actions should execute through the native adapter");

    // Assert
    let observations = &result.timelines()[0].checkpoints[6].observations;
    assert!(
        observations
            .iter()
            .any(|observation| matches!(observation, RigidWorldObservation::Step { .. }))
    );
    assert!(
        observations
            .iter()
            .any(|observation| matches!(observation, RigidWorldObservation::Query { .. }))
    );
    assert!(
        observations
            .iter()
            .any(|observation| matches!(observation, RigidWorldObservation::RayCast { .. }))
    );
    assert!(
        observations
            .iter()
            .any(|observation| matches!(observation, RigidWorldObservation::OriginShift { .. }))
    );
}

#[test]
fn oracle_executes_step_bearing_phase8_after_plan_08_21() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(executable) = OracleExecutable::resolve(&root, OraclePreset::Debug) else {
        return;
    };
    let request = support::phase7_request();

    // Act
    let captured = execute_rigid_world_process(&executable, &request, REVISION)
        .expect("the Plan 08-21 C++ adapter should execute step-bearing Phase 8 actions");

    // Assert
    assert_eq!(captured.result().timelines().len(), 19);
    assert!(captured.reset_verified());
    assert_eq!(captured.reset_epoch(), 1);
}

#[test]
fn expanding_ray_clips_fail_closed_in_native_oracle_and_result_validation() {
    // Arrange
    let baseline_request = request();
    let baseline = NativeRigidWorldExecutor::execute(&baseline_request)
        .expect("baseline rigid request should execute");
    let expanding_request = request_with_expanding_ray_clips();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let maybe_executable = OracleExecutable::resolve(&root, OraclePreset::Debug).ok();

    // Act
    let native = NativeRigidWorldExecutor::execute(&expanding_request);
    let validation = validate_rigid_world_result_against_request(&expanding_request, &baseline);
    let maybe_oracle = maybe_executable
        .map(|executable| execute_rigid_world_process(&executable, &expanding_request, REVISION));

    // Assert
    assert!(
        native.is_err(),
        "native adapter must reject interval expansion"
    );
    assert_eq!(
        validation
            .expect_err("result validation must reject interval expansion")
            .rigid_world_kind(),
        Some(RigidWorldErrorKind::ResultObservationMismatch)
    );
    if let Some(oracle) = maybe_oracle {
        assert!(
            oracle.is_err(),
            "oracle adapter must reject interval expansion"
        );
    }
}

#[test]
fn result_validation_rejects_inconsistent_final_ray_interval() {
    // Arrange
    let request = request();
    let baseline =
        NativeRigidWorldExecutor::execute(&request).expect("baseline rigid request should execute");
    let mut value = result_value(&baseline);
    let observations = value["timelines"][7]["checkpoints"][0]["observations"]
        .as_array_mut()
        .expect("query observations should be an array");
    let clipped_ray = observations
        .iter_mut()
        .find(|observation| {
            observation["kind"] == "ray_cast"
                && observation["observation"]["final_max_fraction_bits"] == json!(0.5_f32.to_bits())
        })
        .expect("strictly clipped ray observation should exist");
    clipped_ray["observation"]["final_max_fraction_bits"] = json!(1.0_f32.to_bits());
    let inconsistent = decode_result_value(&value);

    // Act
    let error = validate_rigid_world_result_against_request(&request, &inconsistent)
        .expect_err("recorded final interval must match callback replay");

    // Assert
    assert_eq!(
        error.rigid_world_kind(),
        Some(RigidWorldErrorKind::ResultObservationMismatch)
    );
}

#[test]
fn native_centered_inertia_zero_origin_branch_executes_without_mutation_failure() {
    // Arrange
    let mut value = serde_json::from_slice::<Value>(REQUEST).expect("fixture should be JSON");
    let actions = value["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("fixture actions should be an array");
    let custom_mass = actions
        .iter_mut()
        .find(|action| action["action_id"] == "nc-custom-mass")
        .expect("custom mass action should exist");
    custom_mass["action"]["mass_bits"] = json!(1.0_f32.to_bits());
    custom_mass["action"]["center"]["x_bits"] = json!(1.0_f32.to_bits());
    custom_mass["action"]["center"]["y_bits"] = json!(0.0_f32.to_bits());
    custom_mass["action"]["inertia_bits"] = json!(0.0_f32.to_bits());
    let request = decode_rigid_world_request_jsonl(
        &encode_value(&value),
        &HarnessLimits::phase2_default_v1(),
    )
    .expect("zero origin inertia should decode through the no-inertia branch");

    // Act
    let result = NativeRigidWorldExecutor::execute(&request);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn native_boundary_rejects_invalid_owner_and_unknown_identity() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut invalid_owner =
        serde_json::from_slice::<Value>(REQUEST).expect("fixture should be JSON");
    invalid_owner["scenario"]["timelines"][0]["fixtures"][0]["owner_body_id"] =
        json!("missing-body");
    let mut unknown_identity =
        serde_json::from_slice::<Value>(REQUEST).expect("fixture should be JSON");
    let actions = unknown_identity["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("fixture actions should be an array");
    let inspect_body = actions
        .iter_mut()
        .find(|action| action["action_id"] == "nc-inspect-body")
        .expect("inspect-body action should exist");
    inspect_body["action"]["body_id"] = json!("missing-body");

    // Act
    let owner_error = decode_rigid_world_request_jsonl(&encode_value(&invalid_owner), &limits)
        .expect_err("an invalid owner must fail before native effects");
    let identity_error =
        decode_rigid_world_request_jsonl(&encode_value(&unknown_identity), &limits)
            .expect_err("an unknown semantic identity must fail before native effects");

    // Assert
    assert_eq!(
        owner_error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidOwner)
    );
    assert_eq!(
        identity_error.rigid_world_kind(),
        Some(RigidWorldErrorKind::UnknownBody)
    );
}

#[test]
fn native_validation_rejects_declaration_disagreement() {
    // Arrange
    let request = request();
    let result =
        NativeRigidWorldExecutor::execute(&request).expect("baseline native result should execute");
    let limits = HarnessLimits::phase2_default_v1();
    let encoded = encode_jsonl(&result, &limits, RecordLimit::Output)
        .expect("baseline native result should encode");
    let mut value = serde_json::from_slice::<Value>(&encoded).expect("result should be JSON");
    value["timelines"][0]["checkpoints"][0]["counts"]["bodies"] = json!(2);
    value["timelines"][0]["checkpoints"][0]["bodies"]
        .as_array_mut()
        .expect("body snapshots should be an array")
        .pop();
    let changed = decode_rigid_world_result_jsonl(&encode_value(&value), &limits)
        .expect("internally consistent changed result should decode");

    // Act
    let error = validate_native_rigid_world_result(&request, &changed)
        .expect_err("changed declared counts must reject the native result");

    // Assert
    assert!(error.to_string().contains("declaration"));
}

#[test]
fn native_cli_dispatches_through_existing_binary() {
    // Arrange
    let request_path = std::env::temp_dir().join(format!(
        "liquidfun-rigid-world-{}-{}.jsonl",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    fs::write(&request_path, REQUEST).expect("temporary request should write");

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_liquidfun-differential"))
        .args(["native-rigid-world", "--request"])
        .arg(&request_path)
        .output()
        .expect("native rigid-world command should launch");
    let _ = fs::remove_file(request_path);

    // Assert
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: liquidfun_test_protocol::RigidWorldResultRecord =
        serde_json::from_slice(&output.stdout).expect("CLI stdout should be one result record");
    assert_eq!(result.timelines().len(), RigidWorldWitnessFamily::ALL.len());
}

#[test]
fn native_rigid_source_changes_build_identity() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest = fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("native-math-sources.txt"),
    )
    .expect("native source manifest should be readable");
    let sources = manifest
        .lines()
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    let required = [
        "crates/liquidfun-differential/src/rigid_world.rs",
        "crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs",
        "crates/liquidfun/src/rigid_differential.rs",
        "crates/liquidfun/src/world/contact_manager.rs",
        "crates/liquidfun/src/world/contact_solver.rs",
        "crates/liquidfun/src/world/step.rs",
    ];
    for path in required {
        assert!(sources.contains(&path), "missing identity source {path}");
    }

    // Act
    let digest = source_digest(&root, &sources, None);
    let adapter =
        liquidfun_differential::EmptyWorldAdapter::new("0123456789abcdef0123456789abcdef01234567")
            .expect("native identity should validate");

    // Assert
    assert_eq!(
        digest,
        adapter.build_identity().adapter_content_sha256().as_str()
    );
    for changed in required {
        assert_ne!(digest, source_digest(&root, &sources, Some(changed)));
    }
}
