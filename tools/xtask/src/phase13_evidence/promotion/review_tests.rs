use super::*;

fn fresh_packet() -> ReviewPacket {
    let mut packet = review_packet_for_test("independent-reviewer", "");
    packet.producer_sha = "f".repeat(40);
    packet.bundle_sha256 = "e".repeat(64);
    packet.acquisition.run_id = 99;
    packet.acquisition.artifact_id = 123;
    packet.acquisition.artifact_name = format!("phase13-staged-99-{}", packet.producer_sha);
    packet.replacement_sha256 = promoted_paths()
        .into_iter()
        .map(|path| (path, "d".repeat(64)))
        .collect();
    packet.review_sha256 = review_sha256(&packet).expect("fresh packet hashes");
    packet
}

#[test]
fn fresh_review_packet_is_valid() {
    // Arrange
    let packet = fresh_packet();
    // Act
    let result = validate_packet_identity(&packet);
    // Assert
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn changed_acquisition_invalidates_previous_acknowledgement() {
    // Arrange
    let mut packet = fresh_packet();
    let acknowledgement = ReviewAcknowledgement {
        schema_version: 2,
        reviewer_id: packet.reviewer_id.clone(),
        review_sha256: packet.review_sha256.clone(),
        acknowledgement: "I reviewed this exact seven-path diff.".to_owned(),
        reviewed_at: "2026-09-14T12:00:00Z".to_owned(),
    };
    packet.acquisition.artifact_id += 1;
    packet.review_sha256 = review_sha256(&packet).expect("changed packet hashes");
    // Act
    let result = validate_review_ack(&packet, Some(&acknowledgement));
    // Assert
    assert_eq!(
        result
            .expect_err("old acknowledgment cannot bind new acquisition")
            .kind(),
        PromotionErrorKind::Acknowledgement
    );
}

#[test]
fn wrong_packet_artifact_source_fails() {
    // Arrange
    let mut packet = fresh_packet();
    packet.producer_sha = "b".repeat(40);
    packet.review_sha256 = review_sha256(&packet).expect("changed packet hashes");
    // Act
    let result = validate_packet_identity(&packet);
    // Assert
    assert_eq!(
        result.expect_err("artifact must bind packet P").kind(),
        PromotionErrorKind::Provider
    );
}

#[test]
fn expired_receipt_history_is_not_invalidated_by_provider_retention() {
    // Arrange
    let mut packet = fresh_packet();
    packet.acquisition.artifact_created_at = "2000-01-01T00:00:00Z".to_owned();
    packet.acquisition.artifact_expires_at = "2000-04-01T00:00:00Z".to_owned();
    packet.review_sha256 = review_sha256(&packet).expect("historical packet hashes");
    // Act
    let result = validate_packet_identity(&packet);
    // Assert
    assert!(result.is_ok());
}

fn copied_reviewed_tree() -> PathBuf {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let copied = super::super::new_staging_root(&repository, "review-contract")
        .expect("fresh review fixture directory");
    for path in PROMOTED_PATHS {
        let destination = copied.join(path);
        fs::create_dir_all(destination.parent().expect("fixture parent"))
            .expect("fixture directories created");
        fs::copy(repository.join(path), destination).expect("copy historical reviewed bytes");
    }
    copied
}

#[test]
fn historical_tracked_receipt_remains_valid() {
    // Arrange
    let root = copied_reviewed_tree();
    // Act
    let result = tracked_reviewed_check(&root);
    // Assert
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn ledger_source_disagreeing_with_receipt_fails() {
    // Arrange
    let root = copied_reviewed_tree();
    let path = root.join(ARTIFACT_MANIFEST_PATH);
    let ledger = fs::read_to_string(&path).expect("fixture ledger reads");
    let receipt: PromotionReceipt = read_json(&root.join(RECEIPT_PATH)).expect("fixture receipt");
    fs::write(
        &path,
        ledger.replacen(
            &format!("producer_sha = \"{}\"", receipt.producer_sha),
            &format!("producer_sha = \"{}\"", "e".repeat(40)),
            1,
        ),
    )
    .expect("alter ledger source only");
    // Act
    let result = tracked_reviewed_check(&root);
    // Assert
    assert_eq!(
        result.expect_err("ledger P must match receipt P").kind(),
        PromotionErrorKind::Ledger
    );
}
