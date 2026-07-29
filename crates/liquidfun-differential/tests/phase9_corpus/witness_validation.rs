fn witness_id(value: &str) -> ScenarioId {
    ScenarioId::new(value).expect("test witness identity should validate")
}

fn valid_witness_binding(branch_id: &str) -> Phase9WitnessBinding {
    let semantic_assertion = match branch_id {
        "finite_lifetime" => Phase9SemanticAssertion::FiniteLifetimeExpired {
            particle_id: witness_id("phase9-a"),
        },
        "infinite_lifetime" => Phase9SemanticAssertion::InfiniteLifetimeSurvives {
            particle_id: witness_id("phase9-b"),
        },
        "equal_lifetime" => Phase9SemanticAssertion::EqualExpirationOrder {
            particle_ids: vec![witness_id("phase9-c"), witness_id("phase9-d")].into_boxed_slice(),
        },
        "strict_contact_enabled" | "strict_contact_disabled" => {
            Phase9SemanticAssertion::StrictContactCardinality {
                enabled: branch_id == "strict_contact_enabled",
                contact_count: u32::from(branch_id == "strict_contact_enabled"),
            }
        }
        "listener_flag_enabled" | "listener_flag_disabled" => {
            Phase9SemanticAssertion::ListenerEventEffect {
                enabled: branch_id == "listener_flag_enabled",
                event_count: u32::from(branch_id == "listener_flag_enabled"),
            }
        }
        "filter_flag_enabled" | "filter_flag_disabled" => {
            Phase9SemanticAssertion::FilterContactEffect {
                enabled: branch_id == "filter_flag_enabled",
                contact_count: u32::from(branch_id == "filter_flag_enabled"),
            }
        }
        "collision_energy" => Phase9SemanticAssertion::CollisionEnergyPositiveFinite {
            minimum_bits: 1.0_f32.to_bits().into(),
        },
        "stuck_candidates" => Phase9SemanticAssertion::StuckCandidatesNonempty {
            particle_ids: vec![witness_id("phase9-coupling")].into_boxed_slice(),
        },
        "replay_identity" => Phase9SemanticAssertion::ReplayResultDigestEquality,
        "minimization_identity" => Phase9SemanticAssertion::MinimizedFailureSignaturePreservation,
        "first_divergence_stability" => Phase9SemanticAssertion::DeliberateFirstDivergence,
        "d0_byte_identity" => Phase9SemanticAssertion::D0RepeatedResultDigestEquality,
        "debug_release_agreement" => Phase9SemanticAssertion::DebugReleaseResultDigestEquality,
        _ => Phase9SemanticAssertion::ObservedSemantic {
            branch_id: witness_id(branch_id),
        },
    };
    Phase9WitnessBinding {
        branch_id: witness_id(branch_id),
        action_index: 0,
        checkpoint_index: 0,
        observation_kind: semantic_assertion.expected_observation_kind(),
        semantic_assertion,
    }
}

fn valid_witness_bindings() -> Vec<Phase9WitnessBinding> {
    REQUIRED_BRANCHES
        .iter()
        .copied()
        .map(valid_witness_binding)
        .collect()
}

#[test]
fn witness_binding_rejects_generic_identity_for_result_evidence() {
    // Arrange
    let mut bindings = valid_witness_bindings();
    let binding = bindings
        .iter_mut()
        .find(|binding| binding.branch_id.as_str() == "replay_identity")
        .expect("replay binding");
    binding.semantic_assertion = Phase9SemanticAssertion::ObservedSemantic {
        branch_id: witness_id("replay_identity"),
    };

    // Act
    let result = validate_phase9_witness_bindings(&bindings, 1, 1);

    // Assert
    assert_eq!(
        result.expect_err("generic identity must fail").kind(),
        Phase9WitnessBindingErrorKind::BranchAssertionMismatch
    );
}

#[test]
fn witness_binding_rejects_zero_collision_energy() {
    // Arrange
    let mut bindings = valid_witness_bindings();
    let binding = bindings
        .iter_mut()
        .find(|binding| binding.branch_id.as_str() == "collision_energy")
        .expect("collision-energy binding");
    binding.semantic_assertion = Phase9SemanticAssertion::CollisionEnergyPositiveFinite {
        minimum_bits: 0.0_f32.to_bits().into(),
    };

    // Act
    let result = validate_phase9_witness_bindings(&bindings, 1, 1);

    // Assert
    assert_eq!(
        result.expect_err("zero energy must fail").kind(),
        Phase9WitnessBindingErrorKind::InvalidSemanticAssertion
    );
}

#[test]
fn witness_binding_rejects_empty_stuck_candidates() {
    // Arrange
    let mut bindings = valid_witness_bindings();
    let binding = bindings
        .iter_mut()
        .find(|binding| binding.branch_id.as_str() == "stuck_candidates")
        .expect("stuck-candidate binding");
    binding.semantic_assertion = Phase9SemanticAssertion::StuckCandidatesNonempty {
        particle_ids: Box::new([]),
    };

    // Act
    let result = validate_phase9_witness_bindings(&bindings, 1, 1);

    // Assert
    assert_eq!(
        result.expect_err("empty stuck candidates must fail").kind(),
        Phase9WitnessBindingErrorKind::InvalidSemanticAssertion
    );
}

#[test]
fn witness_binding_rejects_wrong_observation_kind() {
    // Arrange
    let mut bindings = valid_witness_bindings();
    bindings[0].observation_kind = Phase9ObservationKind::Lifecycle;

    // Act
    let result = validate_phase9_witness_bindings(&bindings, 1, 1);

    // Assert
    assert_eq!(
        result.expect_err("wrong observation kind must fail").kind(),
        Phase9WitnessBindingErrorKind::ObservationKindMismatch
    );
}

#[test]
fn witness_binding_rejects_invalid_action_index() {
    // Arrange
    let mut invalid_action = valid_witness_bindings();
    invalid_action[0].action_index = 1;

    // Act
    let result = validate_phase9_witness_bindings(&invalid_action, 1, 1);

    // Assert
    assert_eq!(
        result.expect_err("invalid action index must fail").kind(),
        Phase9WitnessBindingErrorKind::ActionIndexOutOfRange
    );
}

#[test]
fn witness_binding_rejects_invalid_checkpoint_index() {
    // Arrange
    let mut bindings = valid_witness_bindings();
    bindings[0].checkpoint_index = 1;

    // Act
    let result = validate_phase9_witness_bindings(&bindings, 1, 1);

    // Assert
    assert_eq!(
        result
            .expect_err("invalid checkpoint index must fail")
            .kind(),
        Phase9WitnessBindingErrorKind::CheckpointIndexOutOfRange
    );
}

#[test]
fn witness_binding_rejects_duplicate_branch_ids() {
    // Arrange
    let mut bindings = valid_witness_bindings();
    bindings[1] = bindings[0].clone();

    // Act
    let result = validate_phase9_witness_bindings(&bindings, 1, 1);

    // Assert
    assert_eq!(
        result.expect_err("duplicate branch must fail").kind(),
        Phase9WitnessBindingErrorKind::DuplicateBranch
    );
}

#[test]
fn witness_binding_rejects_unknown_assertion_kind() {
    // Arrange
    let value = json!({
        "branch_id": "collision_energy",
        "action_index": 0,
        "checkpoint_index": 0,
        "observation_kind": "statistics",
        "semantic_assertion": {
            "kind": "request_identity_equality"
        }
    });

    // Act
    let result = serde_json::from_value::<Phase9WitnessBinding>(value);

    // Assert
    assert!(result.is_err());
}

#[test]
fn witness_binding_rejects_more_than_reviewed_limit() {
    // Arrange
    let mut bindings = valid_witness_bindings();
    bindings.push(bindings[0].clone());

    // Act
    let result = validate_phase9_witness_bindings(&bindings, 1, 1);

    // Assert
    assert_eq!(
        result.expect_err("oversized binding set must fail").kind(),
        Phase9WitnessBindingErrorKind::TooManyBindings
    );
}

#[test]
fn witness_binding_requires_exact_reviewed_branch_registry() {
    // Arrange
    let complete = valid_witness_bindings();
    let missing = &complete[1..];
    let mut extra = complete.clone();
    extra[0] = Phase9WitnessBinding {
        branch_id: witness_id("unreviewed-branch"),
        action_index: 0,
        checkpoint_index: 0,
        observation_kind: Phase9ObservationKind::Particle,
        semantic_assertion: Phase9SemanticAssertion::ObservedSemantic {
            branch_id: witness_id("unreviewed-branch"),
        },
    };

    // Act
    let complete_result = validate_phase9_witness_bindings(&complete, 1, 1);
    let missing_result = validate_phase9_witness_bindings(missing, 1, 1);
    let extra_result = validate_phase9_witness_bindings(&extra, 1, 1);

    // Assert
    assert!(complete_result.is_ok());
    assert_eq!(
        missing_result.expect_err("missing branch must fail").kind(),
        Phase9WitnessBindingErrorKind::MissingBranch
    );
    assert_eq!(
        extra_result.expect_err("extra branch must fail").kind(),
        Phase9WitnessBindingErrorKind::ExtraBranch
    );
    assert_eq!(complete.len(), 58);
}
