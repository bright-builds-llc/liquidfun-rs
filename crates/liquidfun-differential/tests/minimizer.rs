//! Focused deterministic validity-preserving scenario minimizer tests.

use std::time::Duration;

use liquidfun_differential::{
    BudgetExhausted, Evaluation, FailureSignature, MinimizationBudget, MinimizationStatus,
    MismatchKind, PhaseName, ScenarioTransform, SemanticPath, minimize,
};
use liquidfun_test_protocol::{
    CheckpointId, HarnessLimits, ScenarioSource, ValidatedScenarioV1, decode_scenario_json,
    decode_scenario_request_jsonl,
};

const REQUEST_BYTES: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/empty-world-request.jsonl");

fn fixture_scenario() -> ValidatedScenarioV1 {
    decode_scenario_request_jsonl(REQUEST_BYTES, &HarnessLimits::phase2_default_v1())
        .expect("checked-in request should validate")
        .scenario()
        .clone()
}

fn seeded_scenario() -> ValidatedScenarioV1 {
    let bytes = String::from_utf8(REQUEST_BYTES.to_vec())
        .expect("fixture request should be UTF-8")
        .replace(
            "\"source\":{\"kind\":\"named\",\"name\":\"empty-world\"}",
            "\"source\":{\"kind\":\"seeded\",\"generator_id\":\"phase2-generator\",\"generator_version\":1,\"seed\":42}",
        );
    decode_scenario_request_jsonl(bytes.as_bytes(), &HarnessLimits::phase2_default_v1())
        .expect("seeded request should validate")
        .scenario()
        .clone()
}

fn signature(checkpoint_id: &str) -> FailureSignature {
    FailureSignature::new(
        CheckpointId::new(checkpoint_id).expect("checkpoint ID should validate"),
        PhaseName::new(checkpoint_id).expect("phase should validate"),
        SemanticPath::SimulationTime,
        MismatchKind::Numeric,
    )
}

fn evaluates_signature(candidate: &ValidatedScenarioV1, target: &FailureSignature) -> Evaluation {
    let maybe_signature = candidate
        .checkpoints()
        .iter()
        .any(|checkpoint| checkpoint.checkpoint_id() == target.checkpoint_id())
        .then(|| target.clone());
    Evaluation::new(maybe_signature, Duration::ZERO)
}

#[test]
fn minimizer_reduces_valid_scenario_and_preserves_canonical_value() {
    // Arrange
    let scenario = seeded_scenario();
    let target = signature("after-step-1");
    let budget = MinimizationBudget::new(32, Duration::from_secs(1));

    // Act
    let result = minimize(&scenario, &target, budget, |candidate| {
        evaluates_signature(candidate, &target)
    })
    .expect("valid scenarios should minimize");
    let reparsed = decode_scenario_json(
        result.canonical_scenario_bytes(),
        &HarnessLimits::phase2_default_v1(),
    )
    .expect("canonical minimized bytes should reparse");

    // Assert
    assert_eq!(result.status(), MinimizationStatus::Complete);
    assert_eq!(result.scenario().commands().len(), 1);
    assert_eq!(result.scenario().checkpoints().len(), 1);
    assert_eq!(reparsed, *result.scenario());
    assert!(matches!(
        result.original_source(),
        ScenarioSource::Seeded {
            generator_id,
            generator_version: 1,
            seed: 42,
        } if generator_id.as_ref() == "phase2-generator"
    ));
}

#[test]
fn invalid_reference_candidates_are_rejected_before_evaluation() {
    // Arrange
    let scenario = fixture_scenario();
    let target = signature("after-step-1");
    let budget = MinimizationBudget::new(32, Duration::from_secs(1));
    let mut evaluations = 0_usize;

    // Act
    let result = minimize(&scenario, &target, budget, |candidate| {
        evaluations += 1;
        let references_are_valid = candidate.checkpoints().iter().all(|checkpoint| {
            candidate.commands().iter().any(|command| {
                command.command_id().as_str() == checkpoint.after_command_id().as_str()
            })
        });
        assert!(references_are_valid);
        evaluates_signature(candidate, &target)
    })
    .expect("invalid candidates should be skipped");

    // Assert
    assert!(result.rejected_invalid_candidates() > 0);
    assert_eq!(evaluations, result.evaluations());
}

#[test]
fn changed_failure_signature_is_never_accepted() {
    // Arrange
    let scenario = fixture_scenario();
    let target = signature("after-step-1");
    let different = signature("after-step-2");
    let budget = MinimizationBudget::new(32, Duration::from_secs(1));

    // Act
    let result = minimize(&scenario, &target, budget, |_| {
        Evaluation::new(Some(different.clone()), Duration::ZERO)
    })
    .expect("changed signatures should be classified");

    // Assert
    assert_eq!(result.scenario(), &scenario);
    assert!(result.rejected_changed_signatures() > 0);
    assert!(result.accepted_transforms().is_empty());
}

#[test]
fn transform_order_is_stable_across_runs() {
    // Arrange
    let scenario = fixture_scenario();
    let target = signature("after-step-1");
    let budget = MinimizationBudget::new(32, Duration::from_secs(1));

    // Act
    let first = minimize(&scenario, &target, budget, |candidate| {
        evaluates_signature(candidate, &target)
    })
    .expect("first minimization should succeed");
    let second = minimize(&scenario, &target, budget, |candidate| {
        evaluates_signature(candidate, &target)
    })
    .expect("second minimization should succeed");

    // Assert
    assert_eq!(first.attempted_transforms(), second.attempted_transforms());
    assert_eq!(
        first.accepted_transforms(),
        &[
            ScenarioTransform::RemoveCheckpoints { start: 1, end: 2 },
            ScenarioTransform::RemoveCommands { start: 1, end: 2 },
        ]
    );
}

#[test]
fn attempt_budget_returns_best_valid_candidate_as_incomplete() {
    // Arrange
    let scenario = fixture_scenario();
    let target = signature("after-step-1");
    let budget = MinimizationBudget::new(1, Duration::from_secs(1));

    // Act
    let result = minimize(&scenario, &target, budget, |_| {
        Evaluation::new(None, Duration::ZERO)
    })
    .expect("attempt exhaustion should return a result");

    // Assert
    assert_eq!(
        result.status(),
        MinimizationStatus::Incomplete(BudgetExhausted::Attempts)
    );
    assert_eq!(result.attempts(), 1);
    assert_eq!(result.scenario(), &scenario);
}

#[test]
fn deadline_budget_returns_best_valid_candidate_as_incomplete() {
    // Arrange
    let scenario = fixture_scenario();
    let target = signature("after-step-1");
    let budget = MinimizationBudget::new(32, Duration::from_millis(5));

    // Act
    let result = minimize(&scenario, &target, budget, |candidate| {
        let mut evaluation = evaluates_signature(candidate, &target);
        evaluation.set_elapsed(Duration::from_millis(5));
        evaluation
    })
    .expect("deadline exhaustion should return a result");

    // Assert
    assert_eq!(
        result.status(),
        MinimizationStatus::Incomplete(BudgetExhausted::Deadline)
    );
    assert_eq!(result.attempts(), 1);
    assert_eq!(result.scenario().checkpoints().len(), 2);
}
