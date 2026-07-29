use super::*;

pub(super) fn complete_manifest_has_stable_human_and_json_ready_reports() {
    // Arrange
    let fixture = Fixture::complete("ready");
    let manifest_path = fixture.write_manifest("manifest.json", &fixture.manifest);

    // Act
    let human = run_audit(&manifest_path, "human");
    let machine = run_audit(&manifest_path, "json");

    // Assert
    assert_success(&human);
    assert_success(&machine);
    let human = String::from_utf8(human.stdout).expect("human output is UTF-8");
    assert!(human.starts_with("release audit: READY\n"));
    assert!(human.contains(&format!("candidate: {CANDIDATE}\nevidence: 19\n")));
    let machine: Value = serde_json::from_slice(&machine.stdout).expect("machine output is JSON");
    assert_eq!(machine["decision"], "ready");
    assert_eq!(machine["candidate_commit"], CANDIDATE);
    assert_eq!(machine["evidence_count"], 19);
    assert_eq!(
        machine["evidence"]
            .as_array()
            .expect("evidence array")
            .len(),
        19
    );
}

pub(super) fn every_required_kind_target_identity_is_independently_mandatory() {
    // Arrange
    let fixture = Fixture::complete("missing-each");
    let items = fixture.manifest["items"].as_array().expect("items").clone();

    // Act / Assert
    for (index, item) in items.iter().enumerate() {
        let mut manifest = fixture.manifest.clone();
        manifest["items"]
            .as_array_mut()
            .expect("items")
            .remove(index);
        let manifest_path = fixture.write_manifest(&format!("missing-{index}.json"), &manifest);
        let output = run_audit(&manifest_path, "json");
        assert_failure_contains(
            &output,
            "release/evidence-set",
            &format!("{}/{}", item["kind"], item["target"]),
        );
    }
}

pub(super) fn manifest_rejects_mixed_candidates_bad_hashes_duplicates_and_unreviewed_items() {
    // Arrange
    let fixture = Fixture::complete("identity-negatives");
    let mut mixed = fixture.manifest.clone();
    mixed["items"][0]["candidate_commit"] = json!(OTHER_CANDIDATE);
    let mut bad_hash = fixture.manifest.clone();
    bad_hash["items"][0]["artifact_sha256"] = json!("0".repeat(64));
    let mut duplicate = fixture.manifest.clone();
    let duplicate_item = duplicate["items"][0].clone();
    duplicate["items"]
        .as_array_mut()
        .expect("items")
        .push(duplicate_item);
    let mut unreviewed = fixture.manifest.clone();
    unreviewed["items"][0]["review_status"] = json!("pending");
    let cases = [
        ("mixed.json", mixed, "release/mixed-candidate"),
        ("bad-hash.json", bad_hash, "release/artifact-hash"),
        ("duplicate.json", duplicate, "release/duplicate-evidence"),
        ("unreviewed.json", unreviewed, "release/evidence-identity"),
    ];

    // Act / Assert
    for (name, manifest, category) in cases {
        let manifest_path = fixture.write_manifest(name, &manifest);
        assert_failure_contains(&run_audit(&manifest_path, "json"), category, name);
    }
}

pub(super) fn manifest_rejects_wrong_producer_workflow_job_and_run_identity() {
    // Arrange
    let fixture = Fixture::complete("producer-negatives");
    let mut wrong_workflow = fixture.manifest.clone();
    wrong_workflow["items"][0]["producer"]["workflow"] = json!("substituted.yml");
    let mut wrong_job = fixture.manifest.clone();
    wrong_job["items"][0]["producer"]["job"] = json!("substituted");
    let mut malformed_run = fixture.manifest.clone();
    malformed_run["items"][0]["producer"]["run_id"] = json!("run-1");
    let cases = [
        ("wrong-workflow.json", wrong_workflow),
        ("wrong-job.json", wrong_job),
        ("malformed-run.json", malformed_run),
    ];

    // Act / Assert
    for (name, manifest) in cases {
        let manifest_path = fixture.write_manifest(name, &manifest);
        assert_failure_contains(
            &run_audit(&manifest_path, "json"),
            "release/evidence-identity",
            name,
        );
    }
}

pub(super) fn manifest_rejects_unknown_kinds_and_the_item_bound() {
    // Arrange
    let fixture = Fixture::complete("closed-schema");
    let mut unknown = fixture.manifest.clone();
    unknown["items"][0]["kind"] = json!("shell_command");
    let mut oversized = fixture.manifest.clone();
    let first = oversized["items"][0].clone();
    let items = oversized["items"].as_array_mut().expect("items");
    while items.len() <= 256 {
        items.push(first.clone());
    }
    let unknown_path = fixture.write_manifest("unknown.json", &unknown);
    let oversized_path = fixture.write_manifest("oversized.json", &oversized);

    // Act / Assert
    assert_failure_contains(
        &run_audit(&unknown_path, "json"),
        "release/manifest-schema",
        "unknown kind",
    );
    assert_failure_contains(
        &run_audit(&oversized_path, "json"),
        "release/manifest",
        "item bound",
    );
}

pub(super) fn conditional_platform_rejects_stale_or_policy_inconsistent_support() {
    // Arrange
    let mut fixture = Fixture::complete("conditional-stale");
    fixture.mutate_claim("conditional_platform", "x86_64-apple-darwin", |claims| {
        claims["disposition"] = json!("supported");
        claims["recorded_at_unix"] = json!(1);
        claims["expires_at_unix"] = json!(2);
    });
    let manifest_path = fixture.write_manifest("manifest.json", &fixture.manifest);

    // Act
    let output = run_audit(&manifest_path, "json");

    // Assert
    assert_failure_contains(
        &output,
        "release/conditional-platform",
        "stale conditional target",
    );
}

pub(super) fn performance_and_coverage_can_never_be_promoted_into_parity_authority() {
    // Arrange
    let mut performance = Fixture::complete("performance-authority");
    performance.mutate_claim("performance", "x86_64-unknown-linux-gnu", |claims| {
        claims["profile_authority"] = json!(true);
    });
    let performance_manifest = performance.write_manifest("manifest.json", &performance.manifest);
    let mut coverage = Fixture::complete("coverage-authority");
    coverage.mutate_claim("rust_coverage", "x86_64-unknown-linux-gnu", |claims| {
        claims["parity_authority"] = json!(true);
    });
    let coverage_manifest = coverage.write_manifest("manifest.json", &coverage.manifest);

    // Act / Assert
    assert_failure_contains(
        &run_audit(&performance_manifest, "json"),
        "release/performance",
        "profile authority",
    );
    assert_failure_contains(
        &run_audit(&coverage_manifest, "json"),
        "release/coverage",
        "coverage parity authority",
    );
}

pub(super) fn unsafe_and_advisory_weakening_fail_closed() {
    // Arrange
    let mut unsafe_fixture = Fixture::complete("unsafe-weakening");
    unsafe_fixture.mutate_claim("rust_safety", "x86_64-unknown-linux-gnu", |claims| {
        claims["unsafe_waivers"] = json!(1);
    });
    let unsafe_manifest = unsafe_fixture.write_manifest("manifest.json", &unsafe_fixture.manifest);
    let mut advisory_fixture = Fixture::complete("advisory-weakening");
    advisory_fixture.mutate_claim("notices", "all", |claims| {
        claims["advisory_waivers"] = json!(1);
    });
    let advisory_manifest =
        advisory_fixture.write_manifest("manifest.json", &advisory_fixture.manifest);

    // Act / Assert
    assert_failure_contains(
        &run_audit(&unsafe_manifest, "json"),
        "release/safety",
        "unsafe weakening",
    );
    assert_failure_contains(
        &run_audit(&advisory_manifest, "json"),
        "release/notices",
        "advisory weakening",
    );
}

pub(super) fn incomplete_corpus_and_compatibility_gaps_reject_readiness() {
    // Arrange
    let mut corpus = Fixture::complete("corpus-gap");
    corpus.mutate_claim("corpus_closure", "all", |claims| {
        claims["unresolved_count"] = json!(1);
    });
    let corpus_manifest = corpus.write_manifest("manifest.json", &corpus.manifest);
    let mut compatibility = Fixture::complete("compatibility-gap");
    compatibility.mutate_claim("compatibility_closure", "all", |claims| {
        claims["gap_count"] = json!(1);
    });
    let compatibility_manifest =
        compatibility.write_manifest("manifest.json", &compatibility.manifest);

    // Act / Assert
    assert_failure_contains(
        &run_audit(&corpus_manifest, "json"),
        "release/corpus-closure",
        "incomplete corpus",
    );
    assert_failure_contains(
        &run_audit(&compatibility_manifest, "json"),
        "release/compatibility-closure",
        "compatibility gap",
    );
}

pub(super) fn package_hash_must_join_every_msrv_and_platform_record() {
    // Arrange
    let mut fixture = Fixture::complete("package-drift");
    fixture.mutate_claim("platform", "aarch64-apple-darwin", |claims| {
        claims["package_sha256"] = json!("3".repeat(64));
    });
    let manifest_path = fixture.write_manifest("manifest.json", &fixture.manifest);

    // Act
    let output = run_audit(&manifest_path, "json");

    // Assert
    assert_failure_contains(&output, "release/package-drift", "package drift");
}

pub(super) fn artifact_payload_hash_is_independently_recomputed() {
    // Arrange
    let repository = repository_root();
    let mut fixture = Fixture::complete("payload-hash");
    let item = fixture.manifest["items"]
        .as_array_mut()
        .expect("items")
        .first_mut()
        .expect("first item");
    let artifact_path = repository.join(
        item["artifact_path"]
            .as_str()
            .expect("artifact path is text"),
    );
    let mut artifact: Value =
        serde_json::from_slice(&fs::read(&artifact_path).expect("artifact can be read"))
            .expect("artifact JSON");
    artifact["payload_sha256"] = json!("0".repeat(64));
    let artifact_bytes = serde_json::to_vec_pretty(&artifact).expect("artifact serializes");
    fs::write(&artifact_path, &artifact_bytes).expect("artifact rewrites");
    item["artifact_sha256"] = json!(sha256(&artifact_bytes));
    let manifest_path = fixture.write_manifest("manifest.json", &fixture.manifest);

    // Act
    let output = run_audit(&manifest_path, "json");

    // Assert
    assert_failure_contains(&output, "release/payload-hash", "payload hash");
}

pub(super) fn missing_manifest_referenced_payload_fails_closed() {
    // Arrange
    let repository = repository_root();
    let fixture = Fixture::complete("missing-payload");
    let artifact_path = repository.join(
        fixture.manifest["items"][0]["artifact_path"]
            .as_str()
            .expect("artifact path is text"),
    );
    fs::remove_file(artifact_path).expect("artifact removes");
    let manifest_path = fixture.write_manifest("manifest.json", &fixture.manifest);

    // Act
    let output = run_audit(&manifest_path, "json");

    // Assert
    assert_failure_contains(&output, "release/artifact-path", "missing payload");
}
