#[test]
fn phase9_comparator_rejects_retained_body_mutation() {
    // Arrange
    let request = bounded_phase9_request("closed-evidence-contract");
    let native =
        NativeRigidWorldExecutor::execute(&request).expect("Phase 9 corpus should execute");
    let oracle = mutated_phase9_result(&native, |value| {
        let body = first_checkpoint_member_mut(value, "bodies");
        let active = body["active"]
            .as_bool()
            .expect("body active state should be boolean");
        body["active"] = json!(!active);
    });

    // Act / Assert
    assert_complete_retained_signature(&request, &native, &oracle, "rigid_world.body.active");
}

#[test]
fn phase9_comparator_rejects_retained_fixture_mutation() {
    // Arrange
    let request = bounded_phase9_request("closed-evidence-contract");
    let native =
        NativeRigidWorldExecutor::execute(&request).expect("Phase 9 corpus should execute");
    let oracle = mutated_phase9_result(&native, |value| {
        let fixture = first_checkpoint_member_mut(value, "fixtures");
        let sensor = fixture["sensor"]
            .as_bool()
            .expect("fixture sensor state should be boolean");
        fixture["sensor"] = json!(!sensor);
    });

    // Act / Assert
    assert_complete_retained_signature(&request, &native, &oracle, "rigid_world.fixture.sensor");
}

#[test]
fn phase9_comparator_rejects_retained_numeric_mutation() {
    // Arrange
    let request = bounded_phase9_request("closed-evidence-contract");
    let native =
        NativeRigidWorldExecutor::execute(&request).expect("Phase 9 corpus should execute");
    let oracle = mutated_phase9_result(&native, |value| {
        let diagnostics =
            first_observation_mut(value, |observation| observation["kind"] == "diagnostics");
        diagnostics["snapshot"]["tree_quality_bits"] = json!(100.0_f32.to_bits());
    });

    // Act / Assert
    assert_complete_retained_signature(
        &request,
        &native,
        &oracle,
        "rigid_world.phase8.diagnostics.tree_quality",
    );
}

#[test]
fn phase9_comparator_rejects_retained_before_particle_mutation() {
    // Arrange
    let request = bounded_phase9_request("closed-evidence-contract");
    let native =
        NativeRigidWorldExecutor::execute(&request).expect("Phase 9 corpus should execute");
    let oracle = mutated_phase9_result(&native, |value| {
        let body = first_checkpoint_member_mut(value, "bodies");
        let active = body["active"]
            .as_bool()
            .expect("body active state should be boolean");
        body["active"] = json!(!active);
        let statistics = first_observation_mut(value, |observation| {
            observation["kind"] == "particle" && observation["observation"]["kind"] == "statistics"
        });
        statistics["observation"]["statistics"]["particle_contact_count"] = json!(u32::MAX);
    });

    // Act / Assert
    assert_complete_retained_signature(&request, &native, &oracle, "rigid_world.body.active");
}

#[test]
fn phase9_comparator_rejects_retained_process_result_through_runner() {
    // Arrange
    let fake = FakePhase9OracleRoot::new("rigid_d1_mismatch");
    let executable = OracleExecutable::resolve(fake.path(), OraclePreset::Debug)
        .expect("fake oracle should occupy the reviewed preset path");
    let request =
        decode_rigid_world_request_jsonl(RETAINED_REQUEST, &HarnessLimits::phase2_default_v1())
            .expect("retained rigid request should decode");
    let revision = manifest().pinned_upstream_revision;

    // Act
    let run = run_phase9_differential(&executable, &request, &revision)
        .expect("request-valid retained process mutation should compare");

    // Assert
    let Phase9ComparisonOutcome::RetainedRigidMismatch(report) = run.outcome() else {
        panic!("runner must report its retained rigid mismatch");
    };
    assert_eq!(report.semantic_path(), "rigid_world.body.active");
}

#[test]
fn fake_phase9_oracle_root_writes_closed_compile_database() {
    // Arrange
    let fake = FakePhase9OracleRoot::new("rigid_d1_mismatch");
    let compile_database = fake
        .path()
        .join("target/reference/oracle-debug/compile_commands.json");
    let entries: Vec<Value> = serde_json::from_slice(
        &fs::read(compile_database).expect("fake compile database should be readable"),
    )
    .expect("fake compile database should decode");

    // Act
    let units = entries
        .iter()
        .map(|entry| {
            Path::new(
                entry["file"]
                    .as_str()
                    .expect("fake compile database file should be a string"),
            )
            .file_name()
            .and_then(|value| value.to_str())
            .expect("fake compile database file should have a UTF-8 name")
            .to_owned()
        })
        .collect::<BTreeSet<_>>();
    let digest = effective_compile_command_sha256(fake.path(), "oracle-debug")
        .expect("fake compile database should have a reviewed command shape");

    // Assert
    assert_eq!(entries.len(), FAKE_PHASE9_RESULT_UNITS.len());
    assert_eq!(
        units,
        FAKE_PHASE9_RESULT_UNITS
            .into_iter()
            .map(str::to_owned)
            .collect::<BTreeSet<_>>()
    );
    assert_eq!(digest.len(), 64);
    assert!(digest.bytes().all(|byte| byte.is_ascii_hexdigit()));
}

#[test]
fn retained_rigid_record_rejects_missing_or_mutated_proof() {
    // Arrange
    let (valid, payloads) = evidence_case_fixture();
    let mut missing = valid.clone();
    missing
        .as_object_mut()
        .expect("evidence case object")
        .remove("retained_rigid");
    let mut mutated = valid.clone();
    mutated["retained_rigid"]["phase8_policy_sha256"] = json!("0".repeat(64));

    // Act
    let valid_result = validate_evidence_case_value(&valid, &payloads);
    let missing_result = validate_evidence_case_value(&missing, &payloads);
    let mutated_result = validate_evidence_case_value(&mutated, &payloads);

    // Assert
    assert!(valid_result.is_ok());
    assert_eq!(
        missing_result,
        Err("missing retained-rigid proof".to_owned())
    );
    assert_eq!(
        mutated_result,
        Err("retained-rigid policy digest mismatch".to_owned())
    );
}

#[test]
fn witness_binding_record_rejects_semantic_or_payload_digest_mutation() {
    // Arrange
    let (valid, payloads) = evidence_case_fixture();
    let mut semantic = valid.clone();
    semantic["witnesses"][0]["action_index"] = json!(usize::MAX);
    let mut corrupted_payloads = payloads.clone();
    corrupted_payloads.native_result.push(b'!');

    // Act
    let valid_result = validate_evidence_case_value(&valid, &payloads);
    let semantic_result = validate_evidence_case_value(&semantic, &payloads);
    let payload_result = validate_evidence_case_value(&valid, &corrupted_payloads);

    // Assert
    assert!(valid_result.is_ok());
    assert_eq!(
        semantic_result,
        Err("witness binding digest mismatch".to_owned())
    );
    assert_eq!(
        payload_result,
        Err("native result payload digest mismatch".to_owned())
    );
}
