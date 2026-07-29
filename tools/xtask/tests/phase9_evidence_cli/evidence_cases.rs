use super::*;

pub(super) fn exact_ref_rejects_denylisted_historical_run_before_evidence_access() -> TestResult {
    for run_id in [REJECTED_RUN, SUPERSEDED_RUN] {
        // Arrange
        let root = TestRoot::new("deny-run")?;
        fs::write(
            root.path.join("run.json"),
            format!(r#"{{"run_id":{run_id}}}"#),
        )?;

        // Act
        let output = run_xtask(&[
            "phase9-evidence",
            "validate",
            "--mode",
            "exact-ref",
            "--canonical-dir",
            &root.relative("missing-canonical"),
            "--sanitizer-dir",
            &root.relative("missing-sanitizer"),
            "--run-json",
            &root.relative("run.json"),
            "--deny-run-id",
            &run_id.to_string(),
        ])?;

        // Assert
        assert!(!output.status.success());
        assert_output_contains(&output, "denylisted");
    }
    Ok(())
}

pub(super) fn exact_ref_accepts_closed_run_job_artifact_and_archive_metadata() -> TestResult {
    // Arrange
    let root = TestRoot::new("valid-exact")?;
    let run = root.write_valid_exact_ref_evidence()?;
    root.write_run_json(&run)?;

    // Act
    let output = root.run_exact_ref()?;

    // Assert
    assert_success(&output);
    Ok(())
}

pub(super) fn exact_ref_rejects_wrong_duplicate_and_expired_live_metadata() -> TestResult {
    // Arrange
    let root = TestRoot::new("invalid-exact-metadata")?;
    let valid = root.write_valid_exact_ref_evidence()?;
    let mut wrong_job = valid.clone();
    wrong_job["jobs"]["canonical"]["name"] = json!("wrong");
    root.write_run_json(&wrong_job)?;
    let wrong_job_output = root.run_exact_ref()?;
    let mut duplicate_job = valid.clone();
    let duplicate = duplicate_job["live_jobs"][0].clone();
    duplicate_job["live_jobs"]
        .as_array_mut()
        .expect("live jobs")
        .push(duplicate);
    root.write_run_json(&duplicate_job)?;
    let duplicate_job_output = root.run_exact_ref()?;
    let mut expired = valid;
    expired["artifacts"]["sanitizer"]["expired"] = json!(true);
    expired["live_artifacts"][1]["expired"] = json!(true);
    root.write_run_json(&expired)?;

    // Act
    let expired_output = root.run_exact_ref()?;

    // Assert
    assert_failure(&wrong_job_output);
    assert_failure(&duplicate_job_output);
    assert_failure(&expired_output);
    Ok(())
}

#[cfg(unix)]
pub(super) fn exact_ref_rejects_symlinked_archive_ancestor_without_touching_target() -> TestResult {
    use std::os::unix::fs::symlink;

    // Arrange
    let root = TestRoot::new("archive-symlink")?;
    let mut run = root.write_valid_exact_ref_evidence()?;
    let external = root.path.with_extension("external");
    fs::create_dir_all(&external)?;
    let external_archive = external.join("canonical.zip");
    fs::copy(root.path.join("canonical.zip"), &external_archive)?;
    let marker = external.join("external-marker");
    fs::write(&marker, b"must survive")?;
    let archive_link = root.path.join("archive-link");
    symlink(&external, &archive_link)?;
    run["artifacts"]["canonical"]["archive_path"] = json!(
        archive_link
            .join("canonical.zip")
            .strip_prefix(workspace_root())?
            .to_string_lossy()
    );
    root.write_run_json(&run)?;

    // Act
    let output = root.run_exact_ref()?;

    // Assert
    assert_failure(&output);
    assert_output_contains(&output, "symlink component");
    assert_eq!(fs::read(&marker)?, b"must survive");

    fs::remove_file(archive_link)?;
    fs::remove_dir_all(external)?;
    Ok(())
}

pub(super) fn local_accepts_complete_canonical_and_sanitizer_evidence() -> TestResult {
    // Arrange
    let root = TestRoot::new("valid-local")?;
    root.write_valid_local_evidence()?;

    // Act
    let output = root.run_local()?;

    // Assert
    assert_success(&output);
    assert!(String::from_utf8_lossy(&output.stdout).contains("58 semantic bindings"));
    Ok(())
}

pub(super) fn local_rejects_schema_v3_with_regeneration_guidance() -> TestResult {
    // Arrange
    let root = TestRoot::new("schema-v3")?;
    root.write_valid_local_evidence()?;
    root.mutate_json("canonical", "phase9-manifest.json", |manifest| {
        manifest["schema_version"] = json!(3);
        manifest["case_record_schema_version"] = json!(2);
    })?;

    // Act
    let output = root.run_local()?;

    // Assert
    assert_failure(&output);
    assert_output_contains(&output, "schema-v4");
    assert_output_contains(&output, "regenerate");
    Ok(())
}

pub(super) fn local_rejects_extra_missing_and_symlink_entries() -> TestResult {
    // Arrange
    let extra = TestRoot::new("extra")?;
    extra.write_valid_local_evidence()?;
    fs::write(extra.path.join("canonical/unexpected.txt"), b"unexpected")?;
    let missing = TestRoot::new("missing")?;
    missing.write_valid_local_evidence()?;
    fs::remove_file(missing.path.join("canonical/inventory.log"))?;
    let symlink = TestRoot::new("symlink")?;
    symlink.write_valid_local_evidence()?;
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        "provenance.log",
        symlink.path.join("canonical/forbidden-link"),
    )?;

    // Act
    let extra_output = extra.run_local()?;
    let missing_output = missing.run_local()?;
    let symlink_output = symlink.run_local()?;

    // Assert
    assert_failure(&extra_output);
    assert_failure(&missing_output);
    assert_failure(&symlink_output);
    Ok(())
}

pub(super) fn local_rejects_failed_logs_and_identity_substitution() -> TestResult {
    // Arrange
    let failed = TestRoot::new("failed-log")?;
    failed.write_valid_local_evidence()?;
    fs::write(
        failed.path.join("canonical/phase9-trace.log"),
        b"test result: FAILED. 0 passed; 1 failed\n",
    )?;
    refresh_identity(&failed.path.join("canonical"))?;
    let substituted = TestRoot::new("substitution")?;
    substituted.write_valid_local_evidence()?;
    let canonical_identity = fs::read(substituted.path.join("canonical/identity.json"))?;
    fs::write(
        substituted.path.join("sanitizer/identity.json"),
        canonical_identity,
    )?;

    // Act
    let failed_output = failed.run_local()?;
    let substituted_output = substituted.run_local()?;

    // Assert
    assert_failure(&failed_output);
    assert_failure(&substituted_output);
    Ok(())
}

pub(super) fn local_rejects_retained_policy_witness_and_payload_corruption() -> TestResult {
    // Arrange
    let retained = TestRoot::new("retained")?;
    retained.write_valid_local_evidence()?;
    retained.mutate_json("canonical", "phase9-manifest.json", |manifest| {
        manifest["cases"][0]["retained_rigid"]["phase8_policy_sha256"] = json!("0".repeat(64));
    })?;
    let witness = TestRoot::new("witness")?;
    witness.write_valid_local_evidence()?;
    witness.mutate_json("canonical", "phase9-manifest.json", |manifest| {
        manifest["cases"][0]["witnesses"][0]["action_index"] = json!(usize::MAX);
    })?;
    let payload = TestRoot::new("payload")?;
    payload.write_valid_local_evidence()?;
    let native_path = payload
        .path
        .join("canonical/cases/storage-systems-and-permutations/native-result.json");
    fs::write(native_path, b"{}")?;
    refresh_identity(&payload.path.join("canonical"))?;

    // Act
    let retained_output = retained.run_local()?;
    let witness_output = witness.run_local()?;
    let payload_output = payload.run_local()?;

    // Assert
    assert_failure(&retained_output);
    assert_failure(&witness_output);
    assert_failure(&payload_output);
    Ok(())
}

pub(super) fn local_rejects_incomplete_policies_and_semantic_manifest_disagreement() -> TestResult {
    // Arrange
    let policies = TestRoot::new("policies")?;
    policies.write_valid_local_evidence()?;
    policies.mutate_json("canonical", "phase9-manifest.json", |manifest| {
        manifest["cases"][0]["consumed_policy_paths"]
            .as_array_mut()
            .expect("policy array")
            .pop();
    })?;
    let disagreement = TestRoot::new("disagreement")?;
    disagreement.write_valid_local_evidence()?;
    disagreement.mutate_json("sanitizer", "phase9-manifest.json", |manifest| {
        manifest["cases"].as_array_mut().expect("cases").swap(0, 1);
        manifest["semantic_manifest_sha256"] = json!(sha256(
            &serde_json::to_vec(&manifest["cases"]).expect("cases bytes")
        ));
    })?;

    // Act
    let policy_output = policies.run_local()?;
    let disagreement_output = disagreement.run_local()?;

    // Assert
    assert_failure(&policy_output);
    assert_failure(&disagreement_output);
    Ok(())
}

pub(super) fn local_rejects_zero_energy_and_empty_stuck_witnesses() -> TestResult {
    // Arrange
    let zero = TestRoot::new("zero-energy")?;
    zero.write_valid_local_evidence()?;
    zero.mutate_manifest_semantics("canonical", |manifest| {
        let binding = find_binding_mut(manifest, "collision_energy");
        binding["semantic_assertion"]["minimum_bits"] = json!(0);
    })?;
    let empty = TestRoot::new("empty-stuck")?;
    empty.write_valid_local_evidence()?;
    empty.mutate_manifest_semantics("canonical", |manifest| {
        let binding = find_binding_mut(manifest, "stuck_candidates");
        binding["semantic_assertion"]["particle_ids"] = json!([]);
    })?;

    // Act
    let zero_output = zero.run_local()?;
    let empty_output = empty.run_local()?;

    // Assert
    assert_failure(&zero_output);
    assert_failure(&empty_output);
    assert_output_contains(&zero_output, "bindings");
    assert_output_contains(&empty_output, "bindings");
    Ok(())
}

pub(super) fn local_rejects_digest_recomputed_in_range_binding_mutations() -> TestResult {
    // Arrange
    let wrong_action = TestRoot::new("wrong-action")?;
    wrong_action.write_valid_local_evidence()?;
    wrong_action.mutate_manifest_semantics("canonical", |manifest| {
        find_binding_mut(manifest, "stable_ids_sort")["action_index"] = json!(9);
    })?;
    let wrong_checkpoint = TestRoot::new("wrong-checkpoint")?;
    wrong_checkpoint.write_valid_local_evidence()?;
    wrong_checkpoint.mutate_manifest_semantics("canonical", |manifest| {
        find_binding_mut(manifest, "optional_lanes")["checkpoint_index"] = json!(0);
    })?;
    let wrong_observation = TestRoot::new("wrong-observation")?;
    wrong_observation.write_valid_local_evidence()?;
    wrong_observation.mutate_case_payload(
        "canonical",
        "storage-systems-and-permutations",
        "native_result_path",
        "native_result_sha256",
        |result| {
            let particle =
                result["timelines"][0]["checkpoints"][1]["observations"][0]["observation"].clone();
            result["timelines"][0]["checkpoints"][0]["observations"][8]["observation"] = particle;
        },
    )?;

    // Act
    let wrong_action_output = wrong_action.run_local()?;
    let wrong_checkpoint_output = wrong_checkpoint.run_local()?;
    let wrong_observation_output = wrong_observation.run_local()?;

    // Assert
    for output in [
        &wrong_action_output,
        &wrong_checkpoint_output,
        &wrong_observation_output,
    ] {
        assert_failure(output);
    }
    assert_output_contains(&wrong_action_output, "expected action");
    assert_output_contains(&wrong_checkpoint_output, "selected checkpoint");
    Ok(())
}

pub(super) fn local_rejects_digest_recomputed_false_semantic_assertions() -> TestResult {
    for (label, branch, mutate) in [
        (
            "false-lifetime",
            "finite_lifetime",
            ("particle_id", json!("phase9-b")),
        ),
        (
            "false-contact",
            "strict_contact_enabled",
            ("contact_count", json!(3)),
        ),
        (
            "false-listener",
            "listener_flag_enabled",
            ("event_count", json!(2)),
        ),
        (
            "false-filter",
            "filter_flag_disabled",
            ("contact_count", json!(0)),
        ),
    ] {
        // Arrange
        let root = TestRoot::new(label)?;
        root.write_valid_local_evidence()?;
        root.mutate_manifest_semantics("canonical", |manifest| {
            find_binding_mut(manifest, branch)["semantic_assertion"][mutate.0] = mutate.1;
        })?;

        // Act
        let output = root.run_local()?;

        // Assert
        assert_failure(&output);
        assert_output_contains(&output, "semantic assertion");
    }
    Ok(())
}

pub(super) fn local_rejects_digest_recomputed_cross_run_proof_mutations() -> TestResult {
    for (label, branch, reference_field) in [
        ("false-replay", "replay_identity", "replay_native"),
        ("false-minimization", "minimization_identity", "copied"),
        (
            "false-first-divergence",
            "first_divergence_stability",
            "minimized",
        ),
        ("false-d0", "d0_byte_identity", "repeated_native"),
        (
            "false-debug-release",
            "debug_release_agreement",
            "release_oracle",
        ),
    ] {
        // Arrange
        let root = TestRoot::new(label)?;
        root.write_valid_local_evidence()?;
        root.mutate_cross_run_payload("canonical", branch, reference_field, |result| {
            let body = first_result_member_mut(result, "bodies");
            body["active"] = json!(!body["active"].as_bool().expect("body active"));
        })?;

        // Act
        let output = root.run_local()?;

        // Assert
        assert_failure(&output);
        assert_output_contains(&output, "cross-run");
    }
    Ok(())
}

pub(super) fn local_recomputes_comparator_instead_of_trusting_match_payload() -> TestResult {
    // Arrange
    let root = TestRoot::new("divergent-pair")?;
    root.write_valid_local_evidence()?;
    root.mutate_case_payload(
        "canonical",
        "closed-evidence-contract",
        "oracle_result_path",
        "oracle_result_sha256",
        |result| {
            let body = result["timelines"]
                .as_array_mut()
                .expect("timelines")
                .iter_mut()
                .flat_map(|timeline| timeline["checkpoints"].as_array_mut().expect("checkpoints"))
                .find_map(|checkpoint| checkpoint["bodies"].as_array_mut()?.first_mut())
                .expect("retained body");
            body["active"] = json!(!body["active"].as_bool().expect("body active"));
        },
    )?;

    // Act
    let output = root.run_local()?;

    // Assert
    assert_failure(&output);
    assert_output_contains(&output, "persisted divergent native and oracle results");
    Ok(())
}
