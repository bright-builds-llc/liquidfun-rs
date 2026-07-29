#[test]
fn cli_minimize_persists_smaller_same_signature_scenario() {
    // Arrange
    let root = repository_root();
    let compare_arguments = [
        "compare",
        "--scenario",
        "empty-world",
        "--preset",
        "oracle-debug",
        "--session-profile",
        "one-shot",
    ];
    let minimize_arguments = [
        "minimize",
        "--scenario",
        "empty-world",
        "--preset",
        "oracle-debug",
        "--session-profile",
        "one-shot",
    ];

    // Act
    let comparison = run_cli(&root, "value_mismatch", &compare_arguments);
    let (minimized, fake_root) = run_cli_with_root(&root, "value_mismatch", &minimize_arguments);

    // Assert
    assert_eq!(comparison.status.code(), Some(2));
    assert!(minimized.status.success());
    let comparison_report: serde_json::Value =
        serde_json::from_slice(&comparison.stdout).expect("comparison report should be JSON");
    let minimization_report: serde_json::Value =
        serde_json::from_slice(&minimized.stdout).expect("minimization report should be JSON");
    assert_eq!(minimization_report["result_kind"], "minimization");
    assert_eq!(minimization_report["status"], "complete");
    assert_eq!(
        minimization_report["target_signature"],
        comparison_report["mismatch"]["signature"]
    );
    let minimized_commands = minimization_report["minimized_commands"]
        .as_u64()
        .expect("minimized command count should be an integer");
    let original_commands = minimization_report["original_commands"]
        .as_u64()
        .expect("original command count should be an integer");
    let minimized_checkpoints = minimization_report["minimized_checkpoints"]
        .as_u64()
        .expect("minimized checkpoint count should be an integer");
    let original_checkpoints = minimization_report["original_checkpoints"]
        .as_u64()
        .expect("original checkpoint count should be an integer");
    assert!(minimized_commands < original_commands || minimized_checkpoints < original_checkpoints);
    let artifact_root = fake_root.join("target/differential/minimized");
    let directories = fs::read_dir(artifact_root)
        .expect("minimized artifact root should exist")
        .collect::<Result<Vec<_>, _>>()
        .expect("minimized artifact entries should be readable");
    assert_eq!(directories.len(), 1);
    let minimized_scenario = fs::read(directories[0].path().join("scenario.json"))
        .expect("minimized scenario should be readable");
    let request: serde_json::Value = serde_json::from_slice(
        &fs::read(fake_root.join("protocol/fixtures/accepted/empty-world-request.jsonl"))
            .expect("source request should be readable"),
    )
    .expect("source request should be JSON");
    let original_scenario =
        serde_json::to_vec(&request["scenario"]).expect("source scenario should serialize");
    assert!(minimized_scenario.len() < original_scenario.len());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&minimized_scenario)
            .expect("minimized scenario should be JSON"),
        minimization_report["scenario"]
    );
}

fn only_failure_directory(repository_root: &Path) -> PathBuf {
    let directories = fs::read_dir(repository_root.join("target/differential/failures"))
        .expect("failure evidence root should exist")
        .collect::<Result<Vec<_>, _>>()
        .expect("failure evidence entries should be readable");
    assert_eq!(directories.len(), 1);
    directories[0].path()
}
