#[test]
fn normalized_optional_depth_still_rejects_required_lane_disappearance() {
    // Arrange
    let manifest = corpus_manifest();
    let pressure_case = manifest
        .cases
        .iter()
        .find(|case| case.case_id == "pressure-constraints-and-rigid")
        .expect("pressure case is sealed");
    let (_, request) = case_request(&recipe(pressure_case));
    let result = NativeRigidWorldExecutor::execute(&request).expect("native case executes");
    let normalized = phase10_observation(&result).clone();
    let mut required = normalized.clone();
    let Phase10Observation::State { state } = &mut required;
    let group = state.groups.first_mut().expect("case has one group");
    group.maybe_depths_bits =
        Some(vec![FloatBits::from_f32(0.0); group.member_ids.len()].into_boxed_slice());

    // Act
    let outcome =
        compare_phase10_observations(Phase10ComparisonMode::D1Semantic, &required, &normalized)
            .expect("both observations remain semantically valid");

    // Assert
    let Phase10ComparisonOutcome::PhysicsMismatch(mismatch) = outcome else {
        panic!("required optional lane disappearance must fail closed");
    };
    assert_eq!(mismatch.semantic_path(), "phase10.group.depth");
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "the end-to-end evidence fixture keeps the two-engine comparison sequence visible"
)]
fn corpus_executes_d0_replay_and_two_engine_debug_release_comparison() {
    // Arrange
    let manifest = corpus_manifest();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let maybe_mode = std::env::var("LIQUIDFUN_PHASE10_ORACLE_MODE").ok();
    let primary_preset = match maybe_mode.as_deref() {
        None | Some("canonical") => OraclePreset::Debug,
        Some("sanitizer") => OraclePreset::AsanUbsan,
        Some(mode) => panic!("unsupported Phase 10 oracle mode {mode}"),
    };
    let resolve = |preset| match OracleExecutable::resolve(&root, preset) {
        Ok(executable) => Some(executable),
        Err(error) if maybe_mode.is_none() => {
            eprintln!("SKIP: build the Phase 10 {preset:?} oracle: {error}");
            None
        }
        Err(error) => panic!("required Phase 10 {preset:?} oracle is unavailable: {error}"),
    };
    let Some(primary) = resolve(primary_preset) else {
        return;
    };
    let Some(debug) = resolve(OraclePreset::Debug) else {
        return;
    };
    let Some(release) = resolve(OraclePreset::Release) else {
        return;
    };

    let expected_phase10_leaves = required_phase10_evidence_leaves()
        .into_iter()
        .filter_map(|leaf| {
            let Phase10EvidenceLeaf::Phase10 { behavior } = leaf else {
                return None;
            };
            Some(behavior)
        })
        .collect::<HashSet<_>>();
    let mut witnessed_phase10_leaves = HashSet::new();
    let mut evidence_cases = Vec::new();

    // Act / Assert
    for case in &manifest.cases {
        let (request_bytes, request) = case_request(&recipe(case));
        let canonical = encode_jsonl(
            &request,
            &HarnessLimits::phase2_default_v1(),
            RecordLimit::Input,
        )
        .expect("validated request re-encodes");
        assert_eq!(
            request_bytes, canonical,
            "{} request authority",
            case.case_id
        );
        let native = NativeRigidWorldExecutor::execute(&request).expect("native case executes");
        let native_replay =
            NativeRigidWorldExecutor::execute(&request).expect("native replay executes");
        let oracle = execute_rigid_world_process(&primary, &request, UPSTREAM_REVISION)
            .expect("selected oracle executes");
        let oracle_replay = execute_rigid_world_process(&primary, &request, UPSTREAM_REVISION)
            .expect("selected oracle replay executes");
        let debug_oracle = execute_rigid_world_process(&debug, &request, UPSTREAM_REVISION)
            .expect("debug oracle executes");
        let optimized = execute_rigid_world_process(&release, &request, UPSTREAM_REVISION)
            .expect("release oracle executes");
        assert_eq!(
            native, native_replay,
            "native D0 differs for {}",
            case.case_id
        );
        assert_eq!(
            oracle.response_bytes(),
            oracle_replay.response_bytes(),
            "oracle D0 differs for {}",
            case.case_id
        );
        assert_eq!(
            oracle.result(),
            debug_oracle.result(),
            "selected and debug modes differ for {}",
            case.case_id
        );
        assert_eq!(
            oracle.result(),
            optimized.result(),
            "build modes differ for {}",
            case.case_id
        );
        let native_observations = phase10_observations(&native);
        let oracle_observations = phase10_observations(oracle.result());
        assert_eq!(native_observations.len(), oracle_observations.len());
        for (native_observation, oracle_observation) in
            native_observations.iter().zip(&oracle_observations)
        {
            let outcome = compare_phase10_observations(
                Phase10ComparisonMode::D1Semantic,
                native_observation,
                oracle_observation,
            )
            .expect("strict observations compare");
            assert!(
                matches!(outcome, Phase10ComparisonOutcome::Match { .. }),
                "{} differs: {outcome:?}",
                case.case_id
            );
            let Phase10Observation::State { state } = native_observation;
            witnessed_phase10_leaves.extend(
                state
                    .witnesses
                    .iter()
                    .filter(|witness| witness.role != WitnessRole::Control)
                    .map(|witness| witness.behavior_leaf),
            );
        }
        evidence_cases.push(evidence_output::capture_case(
            case,
            &request,
            &native,
            &native_replay,
            oracle.result(),
            oracle_replay.result(),
            debug_oracle.result(),
            optimized.result(),
        ));
    }
    assert_eq!(witnessed_phase10_leaves, expected_phase10_leaves);
    coverage_observation::observe(&[
        "public-api.liquidfun-box2d-box2d-particle-b2particleassembly-h",
        "public-api.liquidfun-box2d-box2d-particle-b2particlegroup-h",
        "source-area.liquidfun-box2d-box2d-particle",
        "subsystem.particle-groups-pairs-and-triads",
        "subsystem.particle-solver-behaviors",
    ])
    .expect("successful Phase 10 comparison should emit its covered leaves");
    evidence_output::write_if_requested(&root, &manifest, &evidence_cases);
}
