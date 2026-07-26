//! Contract tests for the final exact-head Phase 13 acceptance gate.

#[allow(dead_code)]
#[path = "../src/phase13_acceptance.rs"]
mod phase13_acceptance;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

use phase13_acceptance::{
    AcceptanceErrorKind, AcceptanceState, AcceptanceStep, HeadSnapshot, IdentityContract,
    StepCompletion, validate_head_snapshot, validate_identity_contract,
    validate_repository_identity_at,
};

const P: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const R: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const Q: &str = "cccccccccccccccccccccccccccccccccccccccc";
const A: &str = "dddddddddddddddddddddddddddddddddddddddd";
const BUNDLE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const WITNESS: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const REPLAY: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const PATH_SET: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";

fn identity_contract() -> IdentityContract {
    let required_trailers = BTreeMap::from([
        ("Phase13-Bundle-SHA256".to_owned(), BUNDLE.to_owned()),
        ("Phase13-Producer-SHA".to_owned(), P.to_owned()),
        ("Phase13-Promotion-Base-SHA".to_owned(), R.to_owned()),
    ]);
    IdentityContract {
        producer_sha: P.to_owned(),
        bundle_sha256: BUNDLE.to_owned(),
        promotion_base_sha: R.to_owned(),
        promotion_sha: Q.to_owned(),
        acceptance_sha: A.to_owned(),
        producer_is_ancestor_of_base: true,
        witness_closure_at_r: WITNESS.to_owned(),
        replay_closure_at_r: REPLAY.to_owned(),
        witness_closure_at_a: WITNESS.to_owned(),
        replay_closure_at_a: REPLAY.to_owned(),
        expected_witness_closure: WITNESS.to_owned(),
        expected_replay_closure: REPLAY.to_owned(),
        promotion_first_parent: R.to_owned(),
        required_trailers: required_trailers.clone(),
        actual_trailers: required_trailers,
        expected_promoted_path_set_sha256: PATH_SET.to_owned(),
        actual_promoted_path_set_sha256: PATH_SET.to_owned(),
        promotion_is_ancestor_of_acceptance: true,
    }
}

fn completed(step: AcceptanceStep) -> StepCompletion {
    StepCompletion {
        step,
        command: format!("test::{step:?}"),
        succeeded: true,
    }
}

fn complete_state() -> AcceptanceState {
    let mut state = AcceptanceState::new();
    for step in AcceptanceStep::ORDERED {
        state
            .record(completed(step))
            .expect("ordered successful step should record");
    }
    state
}

#[test]
fn identity_rejects_p_when_it_is_not_an_ancestor_of_r() {
    // Arrange
    let mut contract = identity_contract();
    contract.producer_is_ancestor_of_base = false;

    // Act
    let error = validate_identity_contract(&contract).expect_err("unrelated P and R must fail");

    // Assert
    assert_eq!(error.kind(), AcceptanceErrorKind::Identity);
}

#[test]
fn identity_rejects_witness_closure_drift_at_r() {
    // Arrange
    let mut contract = identity_contract();
    contract.witness_closure_at_r = REPLAY.to_owned();

    // Act
    let error = validate_identity_contract(&contract).expect_err("R closure drift must fail");

    // Assert
    assert_eq!(error.kind(), AcceptanceErrorKind::Closure);
}

#[test]
fn identity_rejects_replay_closure_drift_at_a() {
    // Arrange
    let mut contract = identity_contract();
    contract.replay_closure_at_a = WITNESS.to_owned();

    // Act
    let error = validate_identity_contract(&contract).expect_err("A closure drift must fail");

    // Assert
    assert_eq!(error.kind(), AcceptanceErrorKind::Closure);
}

#[test]
fn identity_rejects_wrong_q_first_parent() {
    // Arrange
    let mut contract = identity_contract();
    contract.promotion_first_parent = P.to_owned();

    // Act
    let error = validate_identity_contract(&contract).expect_err("wrong Q parent must fail");

    // Assert
    assert_eq!(error.kind(), AcceptanceErrorKind::Identity);
}

#[test]
fn identity_rejects_wrong_q_trailers() {
    // Arrange
    let mut contract = identity_contract();
    contract
        .actual_trailers
        .insert("Phase13-Producer-SHA".to_owned(), R.to_owned());

    // Act
    let error = validate_identity_contract(&contract).expect_err("wrong Q trailers must fail");

    // Assert
    assert_eq!(error.kind(), AcceptanceErrorKind::Identity);
}

#[test]
fn identity_rejects_wrong_q_promoted_path_set() {
    // Arrange
    let mut contract = identity_contract();
    contract.actual_promoted_path_set_sha256 = REPLAY.to_owned();

    // Act
    let error = validate_identity_contract(&contract).expect_err("wrong Q tree must fail");

    // Assert
    assert_eq!(error.kind(), AcceptanceErrorKind::Identity);
}

#[test]
fn identity_rejects_a_outside_q_ancestry() {
    // Arrange
    let mut contract = identity_contract();
    contract.promotion_is_ancestor_of_acceptance = false;

    // Act
    let error = validate_identity_contract(&contract).expect_err("A outside Q must fail");

    // Assert
    assert_eq!(error.kind(), AcceptanceErrorKind::Identity);
}

#[test]
fn head_snapshot_rejects_a_drift() {
    // Arrange
    let snapshot = HeadSnapshot {
        expected_sha: A.to_owned(),
        observed_sha: Q.to_owned(),
        clean: true,
    };

    // Act
    let error = validate_head_snapshot(&snapshot).expect_err("changed HEAD must fail");

    // Assert
    assert_eq!(error.kind(), AcceptanceErrorKind::Head);
}

#[test]
fn head_snapshot_rejects_dirty_source_state() {
    // Arrange
    let snapshot = HeadSnapshot {
        expected_sha: A.to_owned(),
        observed_sha: A.to_owned(),
        clean: false,
    };

    // Act
    let error = validate_head_snapshot(&snapshot).expect_err("dirty state must fail");

    // Assert
    assert_eq!(error.kind(), AcceptanceErrorKind::Head);
}

#[test]
fn state_rejects_reordered_step() {
    // Arrange
    let mut state = AcceptanceState::new();

    // Act
    let error = state
        .record(completed(AcceptanceStep::Provenance))
        .expect_err("reordered step must fail");

    // Assert
    assert_eq!(error.kind(), AcceptanceErrorKind::Ordering);
}

#[test]
fn state_rejects_duplicate_step() {
    // Arrange
    let mut state = AcceptanceState::new();
    state
        .record(completed(AcceptanceStep::Identity))
        .expect("first step should record");

    // Act
    let error = state
        .record(completed(AcceptanceStep::Identity))
        .expect_err("duplicate step must fail");

    // Assert
    assert_eq!(error.kind(), AcceptanceErrorKind::Ordering);
}

#[test]
fn state_short_circuits_failed_step() {
    // Arrange
    let mut state = AcceptanceState::new();
    let mut failed = completed(AcceptanceStep::Identity);
    failed.succeeded = false;

    // Act
    let failure = state
        .record(failed)
        .expect_err("failed step must fail closed");
    let later = state
        .record(completed(AcceptanceStep::Provenance))
        .expect_err("failed state must reject later work");

    // Assert
    assert_eq!(failure.kind(), AcceptanceErrorKind::Step);
    assert_eq!(later.kind(), AcceptanceErrorKind::Step);
}

#[test]
fn publication_rejects_skipped_work() {
    // Arrange
    let mut state = AcceptanceState::new();
    state
        .record(completed(AcceptanceStep::Identity))
        .expect("identity should record");

    // Act
    let error = state
        .publish(identity_contract())
        .expect_err("incomplete state must not publish");

    // Assert
    assert_eq!(error.kind(), AcceptanceErrorKind::Publication);
}

#[test]
fn publication_records_every_ordered_step_only_after_success() {
    // Arrange
    let state = complete_state();

    // Act
    let identity = state
        .publish(identity_contract())
        .expect("complete valid state should publish");

    // Assert
    assert_eq!(identity.acceptance_sha, A);
    assert_eq!(identity.ordered_steps.len(), AcceptanceStep::ORDERED.len());
}

#[test]
fn repository_history_satisfies_the_non_circular_identity_contract() {
    // Arrange
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("Git should be available");
    assert!(output.status.success());
    let acceptance_sha = String::from_utf8(output.stdout)
        .expect("HEAD should be UTF-8")
        .trim()
        .to_owned();

    // Act
    let result = validate_repository_identity_at(&root, &acceptance_sha);

    // Assert
    result.expect("tracked P/B/R/Q and current A should satisfy the identity contract");
}
