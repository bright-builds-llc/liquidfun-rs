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
    PromotionErrorKind, ReviewAcknowledgement, classify_reviewed_paths, promotion_receipt_for_test,
    replace_with_failing_validation, replace_with_injected_failure, review_packet_for_test,
    review_sha256_for_test, reviewed_content_digests_for_test, validate_base_contract,
    validate_content_digest_claims_for_test, validate_exact_paths, validate_review_ack,
    validate_staged_ledgers,
};

const SHA_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SHA_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

const REVIEWED_PATHS: [&str; 7] = [
    "crates/liquidfun-differential/src/fixtures/replay/catalog.rs",
    "reference/artifacts/catalog/rigid-stack-v1.replay-evidence.json",
    "reference/artifacts/phase13/promotion-receipt.json",
    "reference/artifacts/manifest.toml",
    "reference/artifacts/phase9/lifecycle-contact-witnesses.json",
    "reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json",
    "reference/source-map.toml",
];
const RECEIPT_PATH: &str = "reference/artifacts/phase13/promotion-receipt.json";

fn reviewed_replacements() -> (BTreeMap<String, Vec<u8>>, Vec<String>) {
    let changed_paths = REVIEWED_PATHS.map(str::to_owned).to_vec();
    let mut replacements = REVIEWED_PATHS
        .into_iter()
        .map(|path| {
            (
                path.to_owned(),
                format!("reviewed bytes for {path}").into_bytes(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    replacements.insert(RECEIPT_PATH.to_owned(), promotion_receipt_for_test("", ""));
    let (promoted_digest, changed_digest) =
        reviewed_content_digests_for_test(&replacements, &changed_paths)
            .expect("provisional reviewed content should hash");
    replacements.insert(
        RECEIPT_PATH.to_owned(),
        promotion_receipt_for_test(&promoted_digest, &changed_digest),
    );
    (replacements, changed_paths)
}

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
        schema_version: 2,
        reviewer_id: reviewer.to_owned(),
        review_sha256: digest.to_owned(),
        acknowledgement: "I reviewed and acknowledge this exact seven-path diff.".to_owned(),
        reviewed_at: "2026-07-26T01:00:00Z".to_owned(),
    }
}

#[test]
fn review_subject_changes_when_any_replacement_hash_changes() {
    // Arrange
    let mut packet = review_packet_for_test("pRizz", DIGEST_A);
    let original = review_sha256_for_test(&packet).expect("review subject should hash");
    packet
        .replacement_sha256
        .insert("reference/source-map.toml".to_owned(), DIGEST_B.to_owned());

    // Act
    let changed = review_sha256_for_test(&packet).expect("changed review subject should hash");

    // Assert
    assert_ne!(changed, original);
}

#[test]
fn normalized_receipt_content_digests_are_stable() {
    // Arrange
    let (replacements, changed_paths) = reviewed_replacements();

    // Act
    let first = reviewed_content_digests_for_test(&replacements, &changed_paths)
        .expect("reviewed content should hash");
    let second = reviewed_content_digests_for_test(&replacements, &changed_paths)
        .expect("reviewed content should rehash");

    // Assert
    assert_eq!(first, second);
    assert_eq!(
        validate_content_digest_claims_for_test(&replacements)
            .expect("stored content claims should validate"),
        first
    );
}

#[test]
fn normalized_receipt_binds_every_non_digest_field() {
    // Arrange
    let (mut replacements, changed_paths) = reviewed_replacements();
    let original = reviewed_content_digests_for_test(&replacements, &changed_paths)
        .expect("reviewed content should hash");
    let mut receipt: serde_json::Value = serde_json::from_slice(
        replacements
            .get(RECEIPT_PATH)
            .expect("receipt replacement should exist"),
    )
    .expect("receipt should parse");
    receipt["independent_reviewer_id"] = serde_json::Value::String("tampered".to_owned());
    replacements.insert(
        RECEIPT_PATH.to_owned(),
        serde_json::to_vec(&receipt).expect("tampered receipt should encode"),
    );

    // Act
    let tampered = reviewed_content_digests_for_test(&replacements, &changed_paths)
        .expect("tampered content should hash");

    // Assert
    assert_ne!(tampered, original);
}

#[test]
fn reviewed_content_digest_binds_exact_non_receipt_bytes() {
    // Arrange
    let (mut replacements, changed_paths) = reviewed_replacements();
    let original = reviewed_content_digests_for_test(&replacements, &changed_paths)
        .expect("reviewed content should hash");
    replacements.insert(
        "reference/source-map.toml".to_owned(),
        b"tampered source map".to_vec(),
    );

    // Act
    let tampered = reviewed_content_digests_for_test(&replacements, &changed_paths)
        .expect("tampered content should hash");

    // Assert
    assert_ne!(tampered, original);
}

#[test]
fn tampered_stored_receipt_digest_claim_is_rejected() {
    // Arrange
    let (mut replacements, _changed_paths) = reviewed_replacements();
    let mut receipt: serde_json::Value = serde_json::from_slice(
        replacements
            .get(RECEIPT_PATH)
            .expect("receipt replacement should exist"),
    )
    .expect("receipt should parse");
    receipt["promoted_content_sha256"] = serde_json::Value::String(DIGEST_B.to_owned());
    replacements.insert(
        RECEIPT_PATH.to_owned(),
        serde_json::to_vec(&receipt).expect("tampered receipt should encode"),
    );

    // Act
    let error = validate_content_digest_claims_for_test(&replacements)
        .expect_err("tampered stored claim must fail");

    // Assert
    assert_eq!(error.kind(), PromotionErrorKind::Schema);
}

#[test]
fn incremental_classification_preserves_identical_reviewed_members() {
    // Arrange
    let baseline = BTreeMap::from([
        ("one".to_owned(), DIGEST_A.to_owned()),
        ("two".to_owned(), DIGEST_A.to_owned()),
    ]);
    let replacements = BTreeMap::from([
        ("one".to_owned(), DIGEST_B.to_owned()),
        ("two".to_owned(), DIGEST_A.to_owned()),
    ]);

    // Act
    let (changed, unchanged) =
        classify_reviewed_paths(&baseline, &replacements).expect("classification should validate");

    // Assert
    assert_eq!(changed, vec!["one"]);
    assert_eq!(unchanged, vec!["two"]);
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
fn promotion_rolls_back_every_path_after_post_write_validation_failure() {
    // Arrange
    let (repository_root, staging_root, paths) =
        transaction_fixture("post-write-validation-rollback");

    // Act
    let _error = replace_with_failing_validation(&repository_root, &staging_root, &paths)
        .expect_err("injected post-write validation failure must be reported");

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
            let digest_mode = if path == RECEIPT_PATH {
                "phase13_receipt_semantic_v2"
            } else {
                "exact_bytes_sha256"
            };
            format!(
                "[[artifact_schemas.phase13_evidence.records]]\npath = \"{path}\"\nsha256 = \"{DIGEST_A}\"\ndigest_mode = \"{digest_mode}\"\n"
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
