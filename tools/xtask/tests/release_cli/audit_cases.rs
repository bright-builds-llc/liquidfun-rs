use super::*;

pub(super) fn release_constructor_rejects_differential_coverage_miss() {
    // Arrange
    let root = producer_fixture_root("coverage-mutation");
    let artifact_path = root.join("differential-leaves.json");
    let artifact = serde_json::to_vec(&json!({
        "schema_version": 1,
        "parity_authority": false,
        "exercised": ["subsystem.world-stepping"],
        "missed": ["subsystem.particle-contacts"],
    }))
    .expect("coverage JSON");
    fs::write(&artifact_path, &artifact).expect("artifact writes");
    let summary_path = root.join("summary.json");
    fs::write(
        &summary_path,
        serde_json::to_vec(&json!({
            "schema_version": 1,
            "evidence_kind": "differential_coverage",
            "candidate_commit": CANDIDATE,
            "toolchain_identity": "semantic-leaf-v1",
            "artifact_path": "differential-leaves.json",
            "artifact_sha256": sha256(&artifact),
            "parity_authority": false,
        }))
        .expect("summary JSON"),
    )
    .expect("summary writes");

    // Act
    let output = run_release_validator(
        "validate_coverage_payload",
        &[
            summary_path.to_string_lossy().into_owned(),
            CANDIDATE.to_owned(),
            "differential_coverage".to_owned(),
        ],
    );

    // Assert
    assert!(!output.status.success());
}

pub(super) fn release_constructor_rejects_failed_regression_result() {
    // Arrange
    let root = producer_fixture_root("regression-mutation");
    let completion = serde_json::to_vec(&json!({
        "schema_version": 1,
        "candidate_sha": CANDIDATE,
        "complete": true,
        "results": [{
            "regression_id": "regression-one",
            "candidate_sha": CANDIDATE,
            "named_test_path": "suite::regression_one",
            "minimized_sha256": "a".repeat(64),
            "outcome": "failed",
        }],
    }))
    .expect("completion JSON");
    fs::write(root.join("completion.json"), &completion).expect("completion writes");
    fs::write(
        root.join("identity.json"),
        serde_json::to_vec(&json!({ "completion_sha256": sha256(&completion) }))
            .expect("validation identity JSON"),
    )
    .expect("validation identity writes");
    let producer_identity_path = root.join("producer-identity.json");
    fs::write(
        &producer_identity_path,
        serde_json::to_vec(&json!({
            "named_test_count": 1,
            "regression_manifest_sha256":
                file_sha256(&repository_root(), "reference/regressions/manifest.toml"),
        }))
        .expect("producer identity JSON"),
    )
    .expect("producer identity writes");

    // Act
    let output = run_release_validator(
        "validate_regression_payload",
        &[
            root.to_string_lossy().into_owned(),
            CANDIDATE.to_owned(),
            producer_identity_path.to_string_lossy().into_owned(),
        ],
    );

    // Assert
    assert!(!output.status.success());
}

pub(super) fn audit_sources_have_no_producer_process_or_network_capability() {
    // Arrange
    let repository = repository_root();
    let sources = [
        "tools/xtask/src/release.rs",
        "tools/xtask/src/release/domain.rs",
        "tools/xtask/src/release/validation.rs",
        "tools/xtask/src/release/report.rs",
    ];
    let forbidden = [
        "Command::new",
        "std::process",
        "reqwest",
        "curl ",
        "gh workflow",
        "cargo fuzz",
        "cargo bench",
    ];

    // Act
    let source = sources
        .iter()
        .map(|relative| {
            fs::read_to_string(repository.join(relative)).expect("release source can be read")
        })
        .collect::<String>();

    // Assert
    for token in forbidden {
        assert!(!source.contains(token), "forbidden producer token: {token}");
    }
}

pub(super) fn schema_and_required_registry_are_closed_and_bounded() {
    // Arrange
    let repository = repository_root();
    let schema: Value = serde_json::from_slice(
        &fs::read(repository.join("reference/release/schema.json")).expect("schema"),
    )
    .expect("schema JSON");
    let required: RequiredRegistry = toml::from_str(
        &fs::read_to_string(repository.join("reference/release/required-evidence.toml"))
            .expect("registry"),
    )
    .expect("registry TOML");

    // Act
    let kinds = schema["$defs"]["evidence"]["properties"]["kind"]["enum"]
        .as_array()
        .expect("closed kind enum");

    // Assert
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["properties"]["items"]["maxItems"], 256);
    assert_eq!(schema["$defs"]["evidence"]["additionalProperties"], false);
    assert_eq!(
        schema["$defs"]["evidence"]["properties"]["review_status"]["const"],
        "reviewed"
    );
    assert_eq!(required.evidence.len(), 19);
    assert_eq!(kinds.len(), 16);
}
