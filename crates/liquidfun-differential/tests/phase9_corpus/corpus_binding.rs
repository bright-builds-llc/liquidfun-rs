#[test]
fn corpus_is_bound_to_retained_rigid_and_pinned_oracle_bytes() {
    // Arrange
    let manifest = manifest();

    // Act
    let request_digest = format!("{:x}", Sha256::digest(RETAINED_REQUEST));
    let witness_digest = format!("{:x}", Sha256::digest(PINNED_WITNESS));
    let retained =
        decode_rigid_world_request_jsonl(RETAINED_REQUEST, &HarnessLimits::phase2_default_v1())
            .expect("the retained request should decode");

    // Assert
    assert_eq!(manifest.retained_request_sha256, request_digest);
    assert_eq!(manifest.pinned_witness_sha256, witness_digest);
    assert_eq!(
        manifest.pinned_upstream_revision,
        "7f20402173fd143a3988c921bc384459c6a858f2"
    );
    assert_eq!(
        retained.scenario().timelines().len(),
        RigidWorldWitnessFamily::ALL.len()
    );
}

#[test]
fn corpus_executes_with_stable_ids_and_d0_bytes() {
    // Arrange
    let request = bounded_phase9_request("closed-evidence-contract");

    // Act
    let first = NativeRigidWorldExecutor::execute(&request).expect("Phase 9 corpus should execute");
    let second =
        NativeRigidWorldExecutor::execute(&request).expect("Phase 9 replay should execute");
    let first_bytes = serde_json::to_vec(&first).expect("first result should encode");
    let second_bytes = serde_json::to_vec(&second).expect("second result should encode");

    // Assert
    assert_eq!(first, second);
    assert_eq!(first_bytes, second_bytes);
    assert_eq!(
        first.timelines()[0]
            .checkpoints
            .first()
            .expect("Phase 9 checkpoint should exist")
            .phase
            .as_ref(),
        "phase9"
    );
}
