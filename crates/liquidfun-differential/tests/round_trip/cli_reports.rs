#[test]
fn cli_compare_and_replay_emit_deterministic_match_reports() {
    // Arrange
    let root = repository_root();
    let compare_arguments = [
        "compare",
        "--scenario",
        "empty-world",
        "--preset",
        "oracle-debug",
        "--session-profile",
        "reuse",
    ];
    let replay_arguments = [
        "replay",
        "--scenario",
        "empty-world",
        "--preset",
        "oracle-debug",
        "--session-profile",
        "one-shot",
    ];

    // Act
    let compare = run_cli(&root, "valid", &compare_arguments);
    let replay = run_cli(&root, "valid", &replay_arguments);

    // Assert
    assert!(compare.status.success());
    assert!(replay.status.success());
    let compare_json: serde_json::Value =
        serde_json::from_slice(&compare.stdout).expect("compare report should be JSON");
    let replay_json: serde_json::Value =
        serde_json::from_slice(&replay.stdout).expect("replay report should be JSON");
    assert_eq!(compare_json["result_kind"], "match");
    assert_eq!(compare_json["requests"].as_array().map(Vec::len), Some(2));
    assert_eq!(replay_json["result_kind"], "match");
}

#[test]
fn cli_sanitizer_profile_reuses_one_process_and_proves_reset() {
    // Arrange
    let root = repository_root();
    let arguments = [
        "compare",
        "--scenario",
        "empty-world",
        "--preset",
        "oracle-asan-ubsan",
        "--session-profile",
        "sanitizer",
    ];

    // Act
    let output = run_cli(&root, "valid", &arguments);

    // Assert
    assert!(output.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("sanitizer report should be JSON");
    let requests = report["requests"]
        .as_array()
        .expect("sanitizer report should contain requests");
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0]["cpp_reset_epoch"], 1);
    assert_eq!(requests[1]["cpp_reset_epoch"], 2);
    assert_eq!(requests[0]["rust_reset_epoch"], 1);
    assert_eq!(requests[1]["rust_reset_epoch"], 2);
}

#[test]
fn exact_request_replay_preserves_serialized_source_metadata() {
    // Arrange
    let root = fake_repository("valid");
    let bytes = fs::read(root.join("protocol/fixtures/accepted/empty-world-request.jsonl"))
        .expect("exact request should be readable");

    // Act
    let outcome = replay_exact(
        &root,
        &bytes,
        OraclePreset::Debug,
        SessionProfile::OneShot,
        REVISION,
    )
    .expect("exact validated request should replay");

    // Assert
    let DifferentialRunOutcome::Match(run) = outcome else {
        panic!("exact replay should match");
    };
    assert_eq!(run.requests()[0].request_id(), "empty-world-request");
}

#[test]
fn maximum_length_request_id_runs_in_reuse_and_sanitizer_profiles() {
    // Arrange
    let root = fake_repository("valid");
    let request_id = "r".repeat(128);
    let request =
        fs::read_to_string(root.join("protocol/fixtures/accepted/empty-world-request.jsonl"))
            .expect("exact request should be readable")
            .replace("empty-world-request", &request_id);
    let profiles = [
        (OraclePreset::Debug, SessionProfile::Reuse),
        (OraclePreset::AsanUbsan, SessionProfile::Sanitizer),
    ];

    // Act and Assert
    for (preset, profile) in profiles {
        let outcome = replay_exact(&root, request.as_bytes(), preset, profile, REVISION)
            .expect("maximum-length request identity should remain valid");
        let DifferentialRunOutcome::Match(run) = outcome else {
            panic!("bounded request identities should match");
        };
        assert_eq!(run.requests().len(), 2);
        assert_eq!(run.requests()[0].request_id(), request_id);
        assert!(run.requests()[1].request_id().len() <= 128);
    }
}

#[test]
fn cli_distinguishes_harness_failure_from_physics_mismatch_exit_codes() {
    // Arrange
    let root = repository_root();
    let arguments = [
        "compare",
        "--scenario",
        "empty-world",
        "--preset",
        "oracle-debug",
        "--session-profile",
        "one-shot",
    ];

    // Act
    let harness = run_cli(&root, "malformed", &arguments);
    let mismatch = run_cli(&root, "value_mismatch", &arguments);

    // Assert
    assert_eq!(harness.status.code(), Some(3));
    assert_eq!(mismatch.status.code(), Some(2));
    let harness_json: serde_json::Value =
        serde_json::from_slice(&harness.stdout).expect("harness report should be JSON");
    let mismatch_json: serde_json::Value =
        serde_json::from_slice(&mismatch.stdout).expect("mismatch report should be JSON");
    assert_eq!(harness_json["result_kind"], "harness_failure");
    assert_eq!(harness_json["failure_kind"], "malformed_record");
    assert_eq!(mismatch_json["result_kind"], "physics_mismatch");
}
