#[test]
#[allow(
    clippy::too_many_lines,
    reason = "one end-to-end test proves every case and cross-run evidence role"
)]
fn executable_cases() {
    // Arrange
    let manifest = manifest();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let preset = match std::env::var("LIQUIDFUN_PHASE9_ORACLE_MODE").as_deref() {
        Ok("sanitizer") => OraclePreset::AsanUbsan,
        Ok("canonical") | Err(_) => OraclePreset::Debug,
        Ok(mode) => panic!("unsupported Phase 9 oracle mode {mode}"),
    };
    let Ok(executable) = OracleExecutable::resolve(&root, preset) else {
        eprintln!("SKIP: build the selected Phase 9 oracle to execute the corpus");
        return;
    };
    let maybe_evidence_root = std::env::var("LIQUIDFUN_PHASE9_EVIDENCE_MANIFEST")
        .ok()
        .map(|output| {
            let relative_output = PathBuf::from(output);
            assert!(!relative_output.is_absolute() && relative_output.starts_with("target"));
            assert!(
                relative_output
                    .components()
                    .all(|component| matches!(component, Component::Normal(_)))
            );
            root.join(
                relative_output
                    .parent()
                    .expect("evidence manifest must have a parent"),
            )
        });
    let mut evidence_cases = Vec::new();
    let (phase6, phase7, phase8) = retained_profiles();

    // Act
    for case in &manifest.cases {
        let bytes = std::fs::read(fixture_path(case)).expect("fixture bytes must be readable");
        assert_eq!(sha256(&bytes), case.request_sha256, "{}", case.case_id);
        let request = decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
            .expect("fixture must decode");
        let native = NativeRigidWorldExecutor::execute(&request)
            .unwrap_or_else(|error| panic!("native case {} must execute: {error}", case.case_id));
        let oracle =
            execute_rigid_world_process(&executable, &request, &manifest.pinned_upstream_revision)
                .expect("oracle case must execute");
        let run =
            run_phase9_differential(&executable, &request, &manifest.pinned_upstream_revision)
                .expect("case comparison must execute");
        assert_eq!(run.request_sha256().as_str(), case.request_sha256);
        assert_eq!(run.native_request_sha256(), run.request_sha256());
        assert_eq!(run.oracle_request_sha256(), run.request_sha256());
        assert_eq!(run.consumed_paths(), PHASE9_REQUIRED_POLICY_PATHS);
        let retained = compare_phase8_rigid_world_results(
            &request,
            &native,
            oracle.result(),
            &phase6,
            &phase7,
            &phase8,
        )
        .expect("retained rigid comparison must execute");
        assert_eq!(retained, RigidComparisonOutcome::Match);
        assert!(
            matches!(run.outcome(), Phase9ComparisonOutcome::Match { .. }),
            "{} produced an unexpected Phase 9 mismatch: {:?}",
            case.case_id,
            run.outcome()
        );
        let request_value = serde_json::to_value(&request).expect("request JSON");
        let native_value = serde_json::to_value(&native).expect("native JSON");
        let oracle_value = serde_json::to_value(oracle.result()).expect("oracle JSON");
        validate_phase9_evidence_bindings(&request, &native, &case.witnesses)
            .expect("native witnesses must resolve and satisfy their exact observations");
        validate_phase9_evidence_bindings(&request, oracle.result(), &case.witnesses)
            .expect("oracle witnesses must resolve and satisfy their exact observations");
        for witness in &case.witnesses {
            assert_witness(&request_value, &native_value, witness);
            assert_witness(&request_value, &oracle_value, witness);
        }
        let complete_comparison = CompleteComparisonPayload {
            outcome: "match".to_owned(),
            consumed_policy_paths: run
                .consumed_paths()
                .iter()
                .map(|path| (*path).to_owned())
                .collect(),
        };
        let payloads = EvidenceCasePayloads {
            request: bytes,
            native_result: serde_json::to_vec(&native).expect("native bytes"),
            oracle_result: serde_json::to_vec(oracle.result()).expect("oracle bytes"),
            complete_comparison: serde_json::to_vec(&complete_comparison)
                .expect("complete comparison bytes"),
        };
        let (cross_run_proofs, proof_payloads) = build_cross_run_proofs(
            &root,
            &case.case_id,
            &executable,
            &manifest.pinned_upstream_revision,
            &request,
            &native,
            oracle.result(),
            &payloads.request,
            &payloads.native_result,
            &payloads.oracle_result,
            &case.witnesses,
        );
        validate_phase9_cross_run_proofs(
            &case.case_id,
            &request,
            &native,
            oracle.result(),
            &payloads.request,
            &payloads.native_result,
            &payloads.oracle_result,
            &case.witnesses,
            &cross_run_proofs,
            &proof_payloads,
            &HarnessLimits::phase2_default_v1(),
        )
        .expect("generated cross-run evidence must recompute");
        let record = evidence_case_record(case, &payloads, run.consumed_paths(), cross_run_proofs);
        validate_evidence_case_value(
            &serde_json::to_value(&record).expect("evidence case value"),
            &payloads,
        )
        .expect("generated evidence case must validate");
        if let Some(evidence_root) = &maybe_evidence_root {
            write_evidence_payload(evidence_root, &record.request_path, &payloads.request);
            write_evidence_payload(
                evidence_root,
                &record.native_result_path,
                &payloads.native_result,
            );
            write_evidence_payload(
                evidence_root,
                &record.oracle_result_path,
                &payloads.oracle_result,
            );
            write_evidence_payload(
                evidence_root,
                &record.complete_comparison_path,
                &payloads.complete_comparison,
            );
            for (path, bytes) in &proof_payloads {
                write_evidence_payload(evidence_root, path, bytes);
            }
        }
        evidence_cases.push(record);
    }
    let evidence = EvidenceManifest {
        schema_version: 4,
        case_record_schema_version: 3,
        profile: manifest.profile,
        upstream_revision: manifest.pinned_upstream_revision,
        semantic_manifest_sha256: canonical_sha256(&evidence_cases),
        cases: evidence_cases,
    };

    // Assert
    assert!(!evidence.cases.is_empty());
    coverage_observation::observe(&[
        "public-api.liquidfun-box2d-box2d-dynamics-b2body-h",
        "public-api.liquidfun-box2d-box2d-dynamics-b2contactmanager-h",
        "public-api.liquidfun-box2d-box2d-dynamics-b2fixture-h",
        "public-api.liquidfun-box2d-box2d-dynamics-b2island-h",
        "public-api.liquidfun-box2d-box2d-dynamics-b2timestep-h",
        "public-api.liquidfun-box2d-box2d-dynamics-b2world-h",
        "public-api.liquidfun-box2d-box2d-dynamics-b2worldcallbacks-h",
        "public-api.liquidfun-box2d-box2d-dynamics-contacts-b2circlecontact-h",
        "public-api.liquidfun-box2d-box2d-dynamics-contacts-b2contact-h",
        "public-api.liquidfun-box2d-box2d-dynamics-contacts-b2contactsolver-h",
        "public-api.liquidfun-box2d-box2d-particle-b2particle-h",
        "public-api.liquidfun-box2d-box2d-particle-b2particlesystem-h",
        "source-area.liquidfun-box2d-box2d-dynamics",
        "source-area.liquidfun-box2d-box2d-dynamics-contacts",
        "subsystem.contacts-and-filtering",
        "subsystem.particle-contacts-and-coupling",
        "subsystem.particle-storage-and-lifecycle",
        "subsystem.rigid-bodies-and-fixtures",
        "subsystem.rigid-islands-and-solver",
        "subsystem.world-operations-and-observation",
    ])
    .expect("successful Phase 9 comparison should emit its covered leaves");
    if let Ok(output) = std::env::var("LIQUIDFUN_PHASE9_EVIDENCE_MANIFEST") {
        let relative_output = Path::new(&output);
        assert!(!relative_output.is_absolute() && relative_output.starts_with("target"));
        assert!(
            relative_output
                .components()
                .all(|component| { matches!(component, Component::Normal(_)) })
        );
        let output = root.join(relative_output);
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent).expect("evidence directory");
        }
        let mut bytes = serde_json::to_vec_pretty(&evidence).expect("evidence JSON");
        bytes.push(b'\n');
        std::fs::write(output, bytes).expect("evidence manifest");
    }
}

#[test]
#[ignore = "explicit fixture regeneration tool"]
fn regenerate_case_fixture() {
    for case in manifest().cases {
        let request = bounded_phase9_request(&case.case_id);
        let mut bytes = serde_json::to_vec(&request).expect("generated fixture should encode");
        bytes.push(b'\n');
        std::fs::write(fixture_path(&case), bytes).expect("generated fixture should be written");
    }
}

#[test]
fn manifest_declares_every_phase9_branch_exactly_once() {
    // Arrange
    let manifest = manifest();
    let required = REQUIRED_BRANCHES.iter().copied().collect::<BTreeSet<_>>();

    // Act
    let mut occurrences = BTreeMap::<&str, usize>::new();
    for branch in manifest
        .cases
        .iter()
        .flat_map(|case| &case.witnesses)
        .map(|witness| &witness.branch_id)
    {
        *occurrences.entry(branch.as_str()).or_default() += 1;
    }
    let actual = occurrences.keys().copied().collect::<BTreeSet<_>>();
    let bindings = manifest
        .cases
        .iter()
        .flat_map(|case| case.witnesses.iter().cloned())
        .collect::<Vec<_>>();
    let (maximum_actions, maximum_checkpoints) = manifest
        .cases
        .iter()
        .map(|case| {
            let request: Value =
                serde_json::from_slice(&fs::read(fixture_path(case)).expect("fixture bytes"))
                    .expect("fixture JSON");
            (
                request["scenario"]["timelines"][0]["actions"]
                    .as_array()
                    .expect("actions")
                    .len(),
                request["scenario"]["timelines"][0]["checkpoints"]
                    .as_array()
                    .expect("checkpoints")
                    .len(),
            )
        })
        .fold((0, 0), |maximum, count| {
            (maximum.0.max(count.0), maximum.1.max(count.1))
        });

    // Assert
    assert_eq!(manifest.profile, PHASE9_REGISTRY_ID);
    validate_phase9_witness_bindings(&bindings, maximum_actions, maximum_checkpoints)
        .expect("the manifest must be a closed typed witness registry");
    assert_eq!(bindings.len(), REQUIRED_BRANCHES.len());
    assert_eq!(actual, required);
    assert!(occurrences.values().all(|count| *count == 1));
    assert!(manifest.cases.iter().all(|case| !case.case_id.is_empty()));
    assert!(
        manifest
            .cases
            .iter()
            .any(|case| case.authority == Authority::PinnedOracle)
    );
    assert!(
        manifest
            .cases
            .iter()
            .any(|case| case.authority == Authority::Independent)
    );
}
