#[test]
fn comparison_reports_stable_first_numeric_divergence() {
    // Arrange
    let request = comparison_request();
    let profile = profile();
    let native = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound request should execute natively");
    let mut oracle_value = result_value(&native);
    let bits = oracle_value["timelines"][0]["checkpoints"][0]["bodies"][0]["transform"]["position"]
        ["x_bits"]
        .as_u64()
        .expect("position bits should be unsigned");
    oracle_value["timelines"][0]["checkpoints"][0]["bodies"][0]["transform"]["position"]["x_bits"] =
        json!(bits ^ 1);
    let oracle = decode_result_value(&oracle_value);

    // Act
    let first = compare_rigid_world_results(&request, &native, &oracle, &profile)
        .expect("aligned declarations should reach physics comparison");
    let second = compare_rigid_world_results(&request, &native, &oracle, &profile)
        .expect("replay should reach the same physics comparison");

    // Assert
    let RigidComparisonOutcome::PhysicsMismatch(first_report) = first else {
        panic!("one-bit mutation should mismatch");
    };
    let RigidComparisonOutcome::PhysicsMismatch(second_report) = second else {
        panic!("one-bit replay mutation should mismatch");
    };
    assert_eq!(first_report.kind(), RigidMismatchKind::Numeric);
    assert_eq!(
        first_report.semantic_path(),
        "rigid_world.body.transform.position.x"
    );
    assert_eq!(first_report.signature(), second_report.signature());
}

#[test]
fn comparison_never_canonicalizes_manager_report_or_destruction_order() {
    // Arrange
    let request = comparison_request();
    let profile = profile();
    let native = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound request should execute natively");
    let mut oracle_value = result_value(&native);
    let checkpoints = oracle_value["timelines"][1]["checkpoints"]
        .as_array_mut()
        .expect("checkpoints should be an array");
    let checkpoint = checkpoints
        .iter_mut()
        .find(|checkpoint| {
            checkpoint["events"]
                .as_array()
                .is_some_and(|events| events.len() >= 2)
        })
        .expect("contact timeline should contain ordered report events");
    checkpoint["events"]
        .as_array_mut()
        .expect("events should be an array")
        .swap(0, 1);
    let oracle = decode_result_value(&oracle_value);

    // Act
    let outcome = compare_rigid_world_results(&request, &native, &oracle, &profile)
        .expect("reordered reports remain declaration-valid");

    // Assert
    let RigidComparisonOutcome::PhysicsMismatch(report) = outcome else {
        panic!("report-order mutation must mismatch");
    };
    assert_eq!(report.kind(), RigidMismatchKind::Order);
    assert_eq!(
        report.semantic_path(),
        "rigid_world.checkpoint.events.report_order"
    );
}

#[test]
fn supervisor_accepts_the_step_bearing_oracle_after_plan_08_21() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(executable) = OracleExecutable::resolve(&root, OraclePreset::Debug) else {
        return;
    };
    let request = request();

    // Act
    let captured = execute_rigid_world_process(&executable, &request, REVISION)
        .expect("the supervisor should accept the implemented step-bearing request");

    // Assert
    assert_eq!(captured.result().timelines().len(), 19);
    assert!(captured.reset_verified());
}

#[test]
fn reduction_preserves_validity_family_and_exact_first_divergence_signature() {
    // Arrange
    let profile = profile();
    let mut value = serde_json::from_slice::<Value>(REQUEST).expect("fixture should be JSON");
    value["tolerance_profile_sha256"] = json!(profile.profile_sha256().as_str());
    let actions = value["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array");
    let duplicate = actions[19].clone();
    let mut duplicate = duplicate;
    duplicate["action_id"] = json!("nc-custom-mass-redundant");
    actions.insert(20, duplicate);
    let request = decode_rigid_world_request_jsonl(
        &encode_value(&value),
        &HarnessLimits::phase2_default_v1(),
    )
    .expect("request with a redundant valid step should decode");
    let native = NativeRigidWorldExecutor::execute(&request)
        .expect("request with redundant step should execute");
    let mut oracle_value = result_value(&native);
    let bits = oracle_value["timelines"][0]["checkpoints"][0]["bodies"][0]["mass_bits"]
        .as_u64()
        .expect("mass bits should be unsigned");
    oracle_value["timelines"][0]["checkpoints"][0]["bodies"][0]["mass_bits"] = json!(bits ^ 1);
    let oracle = decode_result_value(&oracle_value);
    let RigidComparisonOutcome::PhysicsMismatch(report) =
        compare_rigid_world_results(&request, &native, &oracle, &profile)
            .expect("declarations should align")
    else {
        panic!("one-bit mutation should mismatch");
    };
    let target = report.signature().clone();
    let original_actions = request.scenario().timelines()[0].actions().len();

    // Act
    let result = minimize_rigid_world_request(
        &request,
        &target,
        MinimizationBudget::new(128, Duration::from_secs(1)),
        |_candidate| RigidEvaluation::new(Some(target.clone()), Duration::from_millis(1)),
    )
    .expect("typed rigid reduction should serialize its best candidate");

    // Assert
    assert_eq!(
        result.request().scenario().timelines().len(),
        RigidWorldWitnessFamily::ALL.len()
    );
    assert!(
        result.request().scenario().timelines()[0].actions().len() < original_actions,
        "the redundant action should be removable"
    );
    assert!(!result.accepted_transforms().is_empty());
    decode_rigid_world_request_jsonl(
        result.canonical_request_bytes(),
        &HarnessLimits::phase2_default_v1(),
    )
    .expect("reduced bytes must remain a valid rigid request");
}

#[test]
fn comparison_failure_bundle_retains_exact_rigid_signature() {
    // Arrange
    let request = comparison_request();
    let profile = profile();
    let native = NativeRigidWorldExecutor::execute(&request)
        .expect("profile-bound request should execute natively");
    let mut oracle_value = result_value(&native);
    oracle_value["timelines"][0]["checkpoints"][0]["bodies"][0]["active"] = json!(false);
    let oracle = decode_result_value(&oracle_value);
    let RigidComparisonOutcome::PhysicsMismatch(report) =
        compare_rigid_world_results(&request, &native, &oracle, &profile)
            .expect("declarations should align")
    else {
        panic!("active-state mutation should mismatch");
    };
    let root = std::env::temp_dir().join(format!(
        "liquidfun-rigid-bundle-{}-{}",
        std::process::id(),
        report.signature().signature_sha256().as_str()
    ));
    fs::create_dir(&root).expect("temporary bundle root should be created");
    let request_jsonl = encode_jsonl(
        &request,
        &HarnessLimits::phase2_default_v1(),
        RecordLimit::Input,
    )
    .expect("request should encode");
    let report_json = report.render_machine().expect("report should encode");
    let signature_json = serde_json::to_vec(report.signature()).expect("signature should encode");

    // Act
    let receipt = persist_failure_bundle(
        &root,
        &FailureBundleRequest {
            result_kind: "physics_mismatch",
            request_id: request.request_id(),
            request_jsonl: &request_jsonl,
            report_json: &report_json,
            identity_json: b"{}",
            stderr: b"",
            maybe_failure_signature_json: Some(&signature_json),
        },
    )
    .expect("bounded rigid failure bundle should persist");

    // Assert
    assert_eq!(
        fs::read(receipt.directory().join("failure-signature.json"))
            .expect("signature evidence should be readable"),
        signature_json
    );
    fs::remove_dir_all(&root).expect("temporary bundle root should clean up");
}

fn source_digest(root: &std::path::Path, sources: &[&str], maybe_changed: Option<&str>) -> String {
    let mut hasher = Sha256::new();
    for relative in sources {
        let mut bytes = fs::read(root.join(relative)).expect("identity source should exist");
        if maybe_changed == Some(*relative) {
            bytes.push(b'!');
        }
        let file_digest = Sha256::digest(bytes);
        hasher.update((relative.len() as u64).to_be_bytes());
        hasher.update(relative.as_bytes());
        hasher.update(file_digest);
    }
    format!("{digest:x}", digest = hasher.finalize())
}
