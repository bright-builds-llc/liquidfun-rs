//! Contract tests for the final exact-head Phase 13 acceptance gate.

#[allow(dead_code)]
#[path = "../src/phase13_acceptance.rs"]
mod phase13_acceptance;

use std::collections::BTreeMap;
use std::fs;
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
const CHECKOUT_ACTION: &str = "actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0";
const UPLOAD_ACTION: &str = "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a";

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

fn repository_file(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(path)).expect("repository contract file should be readable")
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

#[test]
fn workflow_uses_only_immutable_action_revisions() {
    // Arrange
    let workflow = repository_file(".github/workflows/phase13-acceptance.yml");

    // Act
    let action_uses = workflow
        .lines()
        .filter_map(|line| line.trim().strip_prefix("uses: "))
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(
        action_uses,
        vec![CHECKOUT_ACTION, UPLOAD_ACTION, UPLOAD_ACTION]
    );
}

#[test]
fn workflow_checks_out_the_exact_full_history_with_the_oracle() {
    // Arrange
    let workflow = repository_file(".github/workflows/phase13-acceptance.yml");

    // Act
    let has_exact_checkout = workflow.contains("ref: ${{ github.sha }}")
        && workflow.contains("fetch-depth: 0")
        && workflow.contains("submodules: recursive");

    // Assert
    assert!(has_exact_checkout);
    assert!(workflow.contains(r#"test "$(git rev-parse HEAD)" = "${GITHUB_SHA}""#));
}

#[test]
fn workflow_pins_the_canonical_phase13_toolchain() {
    // Arrange
    let workflow = repository_file(".github/workflows/phase13-acceptance.yml");

    // Act
    let has_toolchain = workflow.contains("runs-on: ubuntu-24.04")
        && workflow.contains("rustup toolchain install 1.97.0")
        && workflow.contains("cmake version 4.3.3")
        && workflow.contains("test \"$(ninja --version)\" = \"1.13.2\"")
        && workflow.contains(r"clang version 22\.1\.8");

    // Assert
    assert!(has_toolchain);
}

#[test]
fn workflow_invokes_only_the_aggregate_phase13_acceptance_command() {
    // Arrange
    let workflow = repository_file(".github/workflows/phase13-acceptance.yml");

    // Act
    let aggregate_invocations = workflow.matches("cargo xtask phase13 acceptance").count();

    // Assert
    assert_eq!(aggregate_invocations, 1);
    assert!(!workflow.contains("cargo xtask phase14"));
    assert!(!workflow.contains("cargo xtask phase15"));
    assert!(!workflow.contains("workflow_run:"));
}

#[test]
fn workflow_uploads_differential_failures_only_after_failure() {
    // Arrange
    let workflow = repository_file(".github/workflows/phase13-acceptance.yml");

    // Act
    let failure_step = workflow
        .split("- name: Upload bounded differential failure evidence")
        .nth(1)
        .expect("failure upload step should exist")
        .split("- name:")
        .next()
        .expect("failure upload step should be bounded");

    // Assert
    assert!(failure_step.contains("if: failure()"));
    assert!(failure_step.contains(UPLOAD_ACTION));
    assert!(failure_step.contains("path: target/differential/failures"));
}

#[test]
fn workflow_uploads_terminal_identity_only_after_success() {
    // Arrange
    let workflow = repository_file(".github/workflows/phase13-acceptance.yml");

    // Act
    let identity_step = workflow
        .split("- name: Upload terminal Phase 13 identity")
        .nth(1)
        .expect("identity upload step should exist")
        .split("- name:")
        .next()
        .expect("identity upload step should be bounded");

    // Assert
    assert!(identity_step.contains("if: success()"));
    assert!(identity_step.contains(UPLOAD_ACTION));
    assert!(identity_step.contains("path: target/phase13-acceptance/identity.json"));
    assert!(identity_step.contains("if-no-files-found: error"));
}

#[test]
fn workflow_just_recipe_is_a_thin_acceptance_delegation() {
    // Arrange
    let justfile = repository_file("justfile");

    // Act
    let recipe = justfile
        .split("phase13-acceptance:\n")
        .nth(1)
        .expect("Phase 13 acceptance recipe should exist")
        .split("\n\n")
        .next()
        .expect("Phase 13 recipe should be bounded");

    // Assert
    assert_eq!(recipe, "    cargo xtask phase13 acceptance");
}
