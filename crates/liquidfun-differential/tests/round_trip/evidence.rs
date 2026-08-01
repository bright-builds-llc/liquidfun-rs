#[test]
fn cli_harness_failure_persists_bounded_hash_indexed_evidence() {
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
    let (output, fake_root) = run_cli_with_root(&root, "malformed", &arguments);
    let failure_root = fake_root.join("target/differential/failures");
    let directories = fs::read_dir(&failure_root)
        .expect("failure evidence root should exist")
        .collect::<Result<Vec<_>, _>>()
        .expect("failure evidence entries should be readable");

    // Assert
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(directories.len(), 1);
    let directory = directories[0].path();
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(directory.join("manifest.json")).expect("manifest should be readable"),
    )
    .expect("manifest should be valid JSON");
    for name in [
        "request.jsonl",
        "report.json",
        "identity.json",
        "stderr.txt",
    ] {
        let bytes = fs::read(directory.join(name)).expect("evidence file should be readable");
        assert!(bytes.len() <= HarnessLimits::phase2_default_v1().input_record_bytes());
        assert_eq!(
            manifest["files"][name]["sha256"],
            format!("{:x}", Sha256::digest(&bytes))
        );
        assert_eq!(manifest["files"][name]["bytes"], bytes.len());
    }
    assert_eq!(manifest["result_kind"], "harness_failure");
    let identity: serde_json::Value = serde_json::from_slice(
        &fs::read(directory.join("identity.json")).expect("identity should be readable"),
    )
    .expect("identity should be valid JSON");
    assert_eq!(identity["oracle_revision"], REVISION);
    assert_eq!(identity["preset"], "oracle-debug");
    assert_eq!(identity["session_profile"], "one-shot");
    assert_eq!(
        fs::read(directory.join("request.jsonl")).expect("request should be readable"),
        fs::read(fake_root.join("protocol/fixtures/accepted/empty-world-request.jsonl"))
            .expect("source request should be readable")
    );
}

#[test]
fn cli_reuse_and_sanitizer_bundles_bind_the_second_request_and_session_identity() {
    // Arrange
    let root = repository_root();
    let original_request_id = "empty-world-request";
    let expected_request_id = format!("reuse-{:x}", Sha256::digest(original_request_id.as_bytes()));
    let cases = [
        (
            "oracle-debug",
            "reuse",
            "second_malformed",
            3,
            "harness_failure",
        ),
        (
            "oracle-debug",
            "reuse",
            "second_value_mismatch",
            2,
            "physics_mismatch",
        ),
        (
            "oracle-asan-ubsan",
            "sanitizer",
            "second_malformed",
            3,
            "harness_failure",
        ),
        (
            "oracle-asan-ubsan",
            "sanitizer",
            "second_value_mismatch",
            2,
            "physics_mismatch",
        ),
    ];

    // Act
    let results = std::thread::scope(|scope| {
        cases
            .into_iter()
            .map(|(preset, profile, behavior, exit_code, result_kind)| {
                let root = &root;
                scope.spawn(move || {
                    let arguments = [
                        "compare",
                        "--scenario",
                        "empty-world",
                        "--preset",
                        preset,
                        "--session-profile",
                        profile,
                    ];
                    let (output, fake_root) = run_cli_with_root(&root, behavior, &arguments);
                    (
                        preset,
                        profile,
                        behavior,
                        exit_code,
                        result_kind,
                        output,
                        fake_root,
                    )
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| handle.join().expect("concurrent CLI case should join"))
            .collect::<Vec<_>>()
    });

    // Assert
    for (preset, profile, behavior, exit_code, result_kind, output, fake_root) in results {
        let directory = only_failure_directory(&fake_root);
        let manifest: serde_json::Value = serde_json::from_slice(
            &fs::read(directory.join("manifest.json")).expect("manifest should be readable"),
        )
        .expect("manifest should be JSON");
        let request_bytes =
            fs::read(directory.join("request.jsonl")).expect("request should be readable");
        let request =
            decode_scenario_request_jsonl(&request_bytes, &HarnessLimits::phase2_default_v1())
                .expect("persisted request should validate");
        let canonical = encode_jsonl(
            &request,
            &HarnessLimits::phase2_default_v1(),
            RecordLimit::Input,
        )
        .expect("persisted request should re-encode");
        let report: serde_json::Value = serde_json::from_slice(
            &fs::read(directory.join("report.json")).expect("report should be readable"),
        )
        .expect("report should be JSON");
        let identity: serde_json::Value = serde_json::from_slice(
            &fs::read(directory.join("identity.json")).expect("identity should be readable"),
        )
        .expect("identity should be JSON");
        let failure_kind = report["failure_kind"].as_str().unwrap_or("not_applicable");
        let diagnostic = format!(
            "{preset}/{profile}/{behavior}: exit={:?}, result_kind={}, failure_kind={failure_kind}, stderr={}",
            output.status.code(),
            report["result_kind"],
            String::from_utf8_lossy(&output.stderr)
        );
        let session_identity = identity["session_identity_sha256"]
            .as_str()
            .unwrap_or_else(|| panic!("validated session identity should be present: {diagnostic}"));

        assert_eq!(output.status.code(), Some(exit_code), "{diagnostic}");
        assert_eq!(request_bytes, canonical, "{diagnostic}");
        assert_eq!(request.request_id().as_str(), expected_request_id, "{diagnostic}");
        assert_eq!(manifest["request_id"], expected_request_id, "{diagnostic}");
        assert_eq!(manifest["result_kind"], result_kind, "{diagnostic}");
        assert_eq!(report["request_id"], expected_request_id, "{diagnostic}");
        assert_eq!(report["result_kind"], result_kind, "{diagnostic}");
        assert_eq!(
            report["session_identity_sha256"], session_identity,
            "{diagnostic}"
        );
        assert_eq!(session_identity.len(), 64, "{diagnostic}");
        assert!(
            session_identity.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "{diagnostic}"
        );
        if result_kind == "physics_mismatch" {
            assert_eq!(
                report["mismatch"]["request_id"], expected_request_id,
                "{diagnostic}"
            );
        }
    }
}
