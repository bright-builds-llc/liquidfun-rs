use super::*;

pub(super) fn proof_topology_accepts_canonical_paths_and_reviewed_reuse() -> TestResult {
    // Arrange
    let root = TestRoot::new("proof-topology-valid")?;
    let manifest = build_manifest(&root.path)?;
    let case = evidence_case(&manifest, "closed-evidence-contract");

    // Act
    let result =
        Phase9CrossRunProofRecord::validate_topology(&case.case_id, &case.cross_run_proofs);

    // Assert
    assert_eq!(result, Ok(()));
    Ok(())
}

pub(super) fn proof_topology_rejects_baseline_and_required_pair_aliases() -> TestResult {
    for (label, branch, field, path, expected) in [
        (
            "baseline",
            "replay_identity",
            "replay_native",
            "cases/closed-evidence-contract/native-result.json",
            "replay-native",
        ),
        (
            "replay-alias",
            "replay_identity",
            "replay_oracle",
            "cases/closed-evidence-contract/proofs/replay-native.json",
            "replay-oracle",
        ),
        (
            "debug-release-alias",
            "debug_release_agreement",
            "release_oracle",
            "cases/closed-evidence-contract/proofs/debug.json",
            "release",
        ),
        (
            "minimized-copied-alias",
            "minimization_identity",
            "copied",
            "cases/closed-evidence-contract/proofs/minimized.json",
            "copied",
        ),
    ] {
        // Arrange
        let root = TestRoot::new(label)?;
        let manifest = build_manifest(&root.path)?;
        let case = evidence_case(&manifest, "closed-evidence-contract");
        let mut records = case.cross_run_proofs.clone();
        set_proof_path(&mut records, branch, field, path)?;

        // Act
        let error = Phase9CrossRunProofRecord::validate_topology(&case.case_id, &records)
            .expect_err("forbidden topology must fail");

        // Assert
        assert!(
            error.to_string().contains(expected),
            "unexpected topology error: {error}"
        );
    }
    Ok(())
}

pub(super) fn proof_topology_rejects_noncanonical_path_spellings() -> TestResult {
    for (label, path) in [
        ("wrong-case", "cases/other-case/proofs/replay-native.json"),
        (
            "dot-component",
            "cases/closed-evidence-contract/./proofs/replay-native.json",
        ),
        (
            "duplicate-separator",
            "cases/closed-evidence-contract//proofs/replay-native.json",
        ),
        (
            "backslash",
            r"cases\closed-evidence-contract\proofs\replay-native.json",
        ),
        (
            "parent-traversal",
            "cases/closed-evidence-contract/proofs/../replay-native.json",
        ),
        ("absolute", "/tmp/replay-native.json"),
        ("drive-absolute", r"C:\tmp\replay-native.json"),
    ] {
        // Arrange
        let root = TestRoot::new(label)?;
        let manifest = build_manifest(&root.path)?;
        let case = evidence_case(&manifest, "closed-evidence-contract");
        let mut records = case.cross_run_proofs.clone();
        set_proof_path(&mut records, "replay_identity", "replay_native", path)?;

        // Act
        let result = Phase9CrossRunProofRecord::validate_topology(&case.case_id, &records);

        // Assert
        assert!(result.is_err(), "{label} unexpectedly passed");
    }
    Ok(())
}

pub(super) fn proof_topology_cli_rejects_recomputed_baseline_and_pair_aliases() -> TestResult {
    for (label, mutate, expected) in [
        (
            "topology-baseline",
            ProofTopologyMutation::BaselineNativeReplay,
            "replay-native",
        ),
        (
            "topology-replay-alias",
            ProofTopologyMutation::ReplayPairAlias,
            "replay-oracle",
        ),
        (
            "topology-build-alias",
            ProofTopologyMutation::DebugReleaseAlias,
            "release",
        ),
        (
            "topology-reduction-alias",
            ProofTopologyMutation::MinimizedCopiedAlias,
            "copied",
        ),
    ] {
        // Arrange
        let root = TestRoot::new(label)?;
        root.write_valid_local_evidence()?;
        root.mutate_manifest_semantics("canonical", |manifest| {
            mutate_proof_topology(manifest, mutate);
        })?;

        // Act
        let output = root.run_local()?;

        // Assert
        assert_failure(&output);
        assert_output_contains(&output, expected);
    }
    Ok(())
}

pub(super) fn proof_topology_cli_rejects_recomputed_first_divergence_path_only_mutation()
-> TestResult {
    // Arrange
    let root = TestRoot::new("topology-first-divergence-path")?;
    root.write_valid_local_evidence()?;
    root.mutate_manifest_semantics("canonical", |manifest| {
        let case = evidence_case_value_mut(manifest);
        let record = proof_record_value_mut(case, "first_divergence_stability");
        let mismatch =
            find_object_field_mut(&mut record["proof"], "minimized").expect("minimized mismatch");
        mismatch["semantic_path"] = json!("rigid_world.fixture.sensor");
    })?;

    // Act
    let output = root.run_local()?;

    // Assert
    assert_failure(&output);
    assert_output_contains(&output, "persisted mismatch identity");
    Ok(())
}
