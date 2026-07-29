#[test]
fn manifest_closes_five_cases_all_leaves_and_named_policy_calibration() {
    // Arrange
    let manifest = corpus_manifest();
    let expected = required_phase10_evidence_leaves()
        .iter()
        .map(leaf_id)
        .collect::<BTreeSet<_>>();

    // Act
    let actual = manifest
        .cases
        .iter()
        .flat_map(|case| case.leaves.iter().cloned())
        .collect::<BTreeSet<_>>();
    let calibrations = phase10_policy_calibrations().collect::<Vec<_>>();

    // Assert
    assert_eq!(manifest.schema_version, 1);
    assert_eq!(manifest.profile, "phase10-v1");
    assert_eq!(manifest.upstream_revision, UPSTREAM_REVISION);
    assert_eq!(manifest.cases.len(), 5);
    assert_eq!(manifest.policy_count, PHASE10_REQUIRED_POLICY_PATHS.len());
    assert_eq!(calibrations.len(), PHASE10_REQUIRED_POLICY_PATHS.len());
    assert!(calibrations.iter().all(|calibration| {
        !calibration.justification.is_empty()
            && calibration
                .boundary_test
                .starts_with("crates/liquidfun-differential/tests/")
    }));
    assert_eq!(manifest.policy_sha256, sha256(&policy_bytes()));
    assert_eq!(manifest.leaf_sha256, sha256(&leaf_bytes()));
    let retained_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/rigid_world/phase10")
        .join(&manifest.retained_phase9_manifest);
    let retained = std::fs::read(retained_path).expect("retained Phase 9 manifest is readable");
    assert_eq!(manifest.retained_phase9_manifest_sha256, sha256(&retained));
    assert_eq!(
        manifest.manifest_payload_sha256,
        sha256(&manifest_payload_bytes(&manifest))
    );
    assert_eq!(actual, expected);
    assert_eq!(actual.len(), 80);
    assert_eq!(
        manifest
            .cases
            .iter()
            .map(|case| case.leaves.len())
            .sum::<usize>(),
        actual.len(),
        "a leaf may appear in exactly one case"
    );
}

#[test]
fn corpus_request_digests_are_sealed() {
    // Arrange
    let manifest = corpus_manifest();

    // Act
    let actual = manifest
        .cases
        .iter()
        .map(|case| {
            let recipe = recipe(case);
            let (bytes, _) = case_request(&recipe);
            let fixture = std::fs::read(fixture_path(case)).expect("fixture bytes are readable");
            (case.case_id.clone(), sha256(&fixture), sha256(&bytes))
        })
        .collect::<Vec<_>>();
    let expected = manifest
        .cases
        .iter()
        .map(|case| {
            (
                case.case_id.clone(),
                case.fixture_sha256.clone(),
                case.request_sha256.clone(),
            )
        })
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(expected, actual);
}
