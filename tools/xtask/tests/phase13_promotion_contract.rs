//! Contract tests for independently reviewed transactional Phase 13 promotion.

#[allow(dead_code)]
#[path = "../src/phase13_evidence.rs"]
mod phase13_evidence;
#[allow(dead_code)]
#[path = "../src/phase13_evidence/promotion.rs"]
mod phase13_promotion;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use phase13_promotion::{
    PromotionErrorKind, ReviewAcknowledgement, replace_with_injected_failure,
    review_packet_for_test, validate_base_contract, validate_exact_paths, validate_review_ack,
    validate_staged_ledgers,
};

const SHA_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SHA_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

fn temporary_directory(label: &str) -> PathBuf {
    let ordinal = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "liquidfun-phase13-promotion-{label}-{}-{ordinal}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("temporary directory should be created");
    path
}

fn acknowledgement(reviewer: &str, digest: &str) -> ReviewAcknowledgement {
    ReviewAcknowledgement {
        schema_version: 1,
        reviewer_id: reviewer.to_owned(),
        review_diff_sha256: digest.to_owned(),
        acknowledgement: "I reviewed and acknowledge this exact seven-path diff.".to_owned(),
        reviewed_at: "2026-07-26T01:00:00Z".to_owned(),
    }
}

#[test]
fn promotion_rejects_p_when_it_is_not_an_ancestor_of_r() {
    // Arrange
    let producer_is_ancestor = false;

    // Act
    let error = validate_base_contract(SHA_A, SHA_B, producer_is_ancestor, true, true)
        .expect_err("unrelated P and R must fail");

    // Assert
    assert_eq!(error.kind(), PromotionErrorKind::Git);
}

#[test]
fn promotion_rejects_producer_closure_drift() {
    // Arrange
    let witness_closure_equal = false;

    // Act
    let error = validate_base_contract(SHA_A, SHA_B, true, witness_closure_equal, true)
        .expect_err("closure drift must fail");

    // Assert
    assert_eq!(error.kind(), PromotionErrorKind::Closure);
}

#[test]
fn promotion_rejects_path_set_mismatch() {
    // Arrange
    let actual = ["one", "extra"]
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let expected = ["one", "two"]
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();

    // Act
    let error =
        validate_exact_paths(&actual, &expected).expect_err("extra and missing paths must fail");

    // Assert
    assert_eq!(error.kind(), PromotionErrorKind::Path);
}

#[test]
fn promotion_rejects_stale_ledger_hashes() {
    // Arrange
    let staging_root = temporary_directory("stale-ledger");
    let manifest_path = staging_root.join("reference/artifacts/manifest.toml");
    fs::create_dir_all(
        manifest_path
            .parent()
            .expect("manifest path should have a parent"),
    )
    .expect("manifest parent should be created");
    fs::write(&manifest_path, ledger_fixture()).expect("ledger fixture should be written");
    let hashes = BTreeMap::from([
        (
            "reference/artifacts/phase9/lifecycle-contact-witnesses.json".to_owned(),
            DIGEST_B.to_owned(),
        ),
        (
            "reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json".to_owned(),
            DIGEST_A.to_owned(),
        ),
        (
            "reference/artifacts/catalog/rigid-stack-v1.replay-evidence.json".to_owned(),
            DIGEST_A.to_owned(),
        ),
        (
            "reference/artifacts/phase13/promotion-receipt.json".to_owned(),
            DIGEST_A.to_owned(),
        ),
    ]);

    // Act
    let error =
        validate_staged_ledgers(&staging_root, &hashes).expect_err("stale witness hash must fail");

    // Assert
    assert_eq!(error.kind(), PromotionErrorKind::Ledger);
}

#[test]
fn promotion_rejects_review_diff_digest_mismatch() {
    // Arrange
    let packet = review_packet_for_test("pRizz", DIGEST_A);
    let acknowledgement = acknowledgement("pRizz", DIGEST_B);

    // Act
    let error = validate_review_ack(&packet, Some(&acknowledgement))
        .expect_err("acknowledgement for a different diff must fail");

    // Assert
    assert_eq!(error.kind(), PromotionErrorKind::Acknowledgement);
}

#[test]
fn promotion_rejects_missing_acknowledgement() {
    // Arrange
    let packet = review_packet_for_test("pRizz", DIGEST_A);

    // Act
    let error = validate_review_ack(&packet, None).expect_err("missing acknowledgement must fail");

    // Assert
    assert_eq!(error.kind(), PromotionErrorKind::Acknowledgement);
}

#[test]
fn promotion_rejects_acknowledgement_from_the_wrong_reviewer() {
    // Arrange
    let packet = review_packet_for_test("pRizz", DIGEST_A);
    let acknowledgement = acknowledgement("someone-else", DIGEST_A);

    // Act
    let error = validate_review_ack(&packet, Some(&acknowledgement))
        .expect_err("different reviewer identity must fail");

    // Assert
    assert_eq!(error.kind(), PromotionErrorKind::Acknowledgement);
}

#[test]
fn promotion_reports_partial_replacement_failure() {
    // Arrange
    let (repository_root, staging_root, paths) = transaction_fixture("partial-failure");

    // Act
    let error = replace_with_injected_failure(&repository_root, &staging_root, &paths, Some(1))
        .expect_err("injected partial failure must be reported");

    // Assert
    assert_eq!(error.kind(), PromotionErrorKind::Transaction);
}

#[test]
fn promotion_rolls_back_every_path_after_partial_failure() {
    // Arrange
    let (repository_root, staging_root, paths) = transaction_fixture("rollback");

    // Act
    let _error = replace_with_injected_failure(&repository_root, &staging_root, &paths, Some(1))
        .expect_err("injected partial failure must be reported");

    // Assert
    assert_eq!(
        fs::read(repository_root.join("one.txt")).expect("first original should remain"),
        b"old-one"
    );
    assert_eq!(
        fs::read(repository_root.join("nested/two.txt")).expect("second original should remain"),
        b"old-two"
    );
}

#[test]
fn promotion_replaces_every_path_when_transaction_succeeds() {
    // Arrange
    let (repository_root, staging_root, paths) = transaction_fixture("success");

    // Act
    replace_with_injected_failure(&repository_root, &staging_root, &paths, None)
        .expect("complete replacement should succeed");

    // Assert
    assert_eq!(
        fs::read(repository_root.join("one.txt")).expect("first replacement should exist"),
        b"new-one"
    );
    assert_eq!(
        fs::read(repository_root.join("nested/two.txt")).expect("second replacement should exist"),
        b"new-two"
    );
}

fn transaction_fixture(label: &str) -> (PathBuf, PathBuf, [&'static str; 2]) {
    let root = temporary_directory(label);
    let repository_root = root.join("repository");
    let staging_root = root.join("staging");
    write_fixture_file(&repository_root.join("one.txt"), b"old-one");
    write_fixture_file(&repository_root.join("nested/two.txt"), b"old-two");
    write_fixture_file(&staging_root.join("one.txt"), b"new-one");
    write_fixture_file(&staging_root.join("nested/two.txt"), b"new-two");
    (repository_root, staging_root, ["one.txt", "nested/two.txt"])
}

fn write_fixture_file(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().expect("fixture path should have a parent"))
        .expect("fixture parent should be created");
    fs::write(path, bytes).expect("fixture should be written");
}

fn ledger_fixture() -> String {
    let paths = [
        "reference/artifacts/phase9/lifecycle-contact-witnesses.json",
        "reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json",
        "reference/artifacts/catalog/rigid-stack-v1.replay-evidence.json",
        "reference/artifacts/phase13/promotion-receipt.json",
    ];
    paths
        .into_iter()
        .map(|path| {
            format!(
                "[[artifact_schemas.phase13_evidence.records]]\npath = \"{path}\"\nsha256 = \"{DIGEST_A}\"\n"
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
