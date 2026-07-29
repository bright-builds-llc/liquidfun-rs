fn scenario_id(value: &str) -> ScenarioId {
    ScenarioId::new(value).expect("test identity is valid")
}

fn payloads(case_id: &str) -> Phase10EvidencePayloads {
    let path = |role: &str| format!("cases/{case_id}/proofs/{role}.json").into_boxed_str();
    Phase10EvidencePayloads {
        native: path("native"),
        oracle: path("oracle"),
        comparison: path("comparison"),
        replay_native: path("replay-native"),
        replay_oracle: path("replay-oracle"),
        debug_oracle: path("debug-oracle"),
        release_oracle: path("release-oracle"),
        minimized: path("minimized"),
        copied: path("copied"),
        inherited: path("inherited"),
    }
}

fn witness(role: WitnessRole, index: usize) -> Phase10EvidenceWitnessRef {
    Phase10EvidenceWitnessRef {
        role,
        action_index: index,
        checkpoint_index: 0,
        observation_index: index,
    }
}

type CompleteContract = (
    Vec<Phase10EvidenceBinding>,
    HashMap<ScenarioId, (usize, usize, usize)>,
);

fn complete_contract() -> CompleteContract {
    let case_id = scenario_id("group-construction-and-mutation");
    let bindings = required_phase10_evidence_leaves()
        .into_iter()
        .enumerate()
        .map(|(index, leaf)| Phase10EvidenceBinding {
            leaf,
            case_id: case_id.clone(),
            implementation: "crates/liquidfun/src/particle/solver.rs:phase10_semantics".into(),
            tests: Phase10EvidenceTestRefs {
                focused: format!("crates/liquidfun/tests/particle_solver_flags.rs:focused_{index}")
                    .into_boxed_str(),
                integration: format!(
                    "crates/liquidfun-differential/tests/phase10_native.rs:integration_{index}"
                )
                .into_boxed_str(),
                property: format!(
                    "crates/liquidfun/tests/particle_group_properties.rs:property_{index}"
                )
                .into_boxed_str(),
            },
            control: witness(WitnessRole::Control, index * 3),
            activation: witness(WitnessRole::Activation, index * 3 + 1),
            maybe_interaction: Some(witness(WitnessRole::Interaction, index * 3 + 2)),
            observation_path: "phase10.witness.kind".into(),
            policy_path: "phase10.witness.kind".into(),
            payloads: payloads(case_id.as_str()),
        })
        .collect::<Vec<_>>();
    let bounds = HashMap::from([(case_id, (bindings.len() * 3, 1, bindings.len() * 3))]);
    (bindings, bounds)
}

#[test]
fn evidence_contract_accepts_exact_closed_leaf_inventory() {
    // Arrange
    let (bindings, bounds) = complete_contract();

    // Act
    let result = validate_phase10_evidence_contract(&bindings, &bounds);

    // Assert
    assert!(result.is_ok());
    assert_eq!(bindings.len(), 80);
}

#[test]
fn evidence_contract_rejects_missing_duplicate_and_unknown_leaves() {
    // Arrange
    let (bindings, bounds) = complete_contract();
    let mut missing = bindings.clone();
    missing.pop();
    let mut duplicate = bindings.clone();
    duplicate[1].leaf = duplicate[0].leaf.clone();
    let mut unknown = bindings.clone();
    unknown[0].leaf = Phase10EvidenceLeaf::Inherited {
        branch_id: scenario_id("unreviewed-branch"),
    };

    // Act / Assert
    assert!(validate_phase10_evidence_contract(&missing, &bounds).is_err());
    assert!(validate_phase10_evidence_contract(&duplicate, &bounds).is_err());
    assert!(validate_phase10_evidence_contract(&unknown, &bounds).is_err());
}

#[test]
fn evidence_contract_rejects_aliases_ranges_private_passes_and_open_paths() {
    // Arrange
    let (bindings, bounds) = complete_contract();
    let mut aliased = bindings.clone();
    aliased[0].payloads.oracle = aliased[0].payloads.native.clone();
    let mut out_of_range = bindings.clone();
    out_of_range[0].activation.action_index = usize::MAX;
    let mut private_pass = bindings.clone();
    private_pass[0].implementation = "crates/liquidfun/src/particle/PassId.rs".into();
    let mut wildcard = bindings.clone();
    wildcard[0].observation_path = "phase10.*".into();

    // Act / Assert
    assert!(validate_phase10_evidence_contract(&aliased, &bounds).is_err());
    assert!(validate_phase10_evidence_contract(&out_of_range, &bounds).is_err());
    assert!(validate_phase10_evidence_contract(&private_pass, &bounds).is_err());
    assert!(validate_phase10_evidence_contract(&wildcard, &bounds).is_err());
}

#[test]
fn evidence_contract_rejects_metadata_repair_after_semantic_substitution() {
    // Arrange
    let (mut bindings, bounds) = complete_contract();
    bindings[0].activation.role = WitnessRole::Control;
    bindings[0].payloads = payloads(bindings[0].case_id.as_str());

    // Act
    let result = validate_phase10_evidence_contract(&bindings, &bounds);

    // Assert
    assert!(result.is_err());
}
