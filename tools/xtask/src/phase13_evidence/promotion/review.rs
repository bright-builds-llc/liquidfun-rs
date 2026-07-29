use super::{
    ARTIFACT_MANIFEST_PATH, BTreeMap, BTreeSet, BUNDLE_SHA256, EXACT_BYTES_DIGEST_MODE,
    PRODUCER_SHA, PROMOTED_PATHS, Path, PathBuf, PromotionError, PromotionErrorKind,
    PromotionReceipt, RECEIPT_PATH, RECEIPT_SEMANTIC_DIGEST_MODE, REPLAY_EVIDENCE_PATH,
    ReviewAcknowledgement, ReviewPacket, WITNESS_PATH, WITNESS_PROVENANCE_PATH, absolute_path,
    canonical_diff, changed_path_set_sha256, file_sha256, filesystem_error, fs, git_file,
    git_maybe_file, git_text, promoted_path_set_sha256, promoted_paths, read_json,
    receipt_semantic_sha256, require_clean_worktree, require_options, required, review_sha256,
    reviewed_content_digests_from_root, reviewed_replacements_from_root, transaction, valid_digest,
    valid_revision, valid_utc_timestamp, validate_acquisition, validate_content_digest_claims,
    validate_staged_ledgers, validate_staged_tree,
};
#[cfg(test)]
use super::{
    Acquisition, ClosureReview, PROVIDER_ARTIFACT_ID, PROVIDER_ARTIFACT_NAME, PROVIDER_DIGEST,
    PROVIDER_REPOSITORY, PROVIDER_RUN_ID,
};

pub(super) fn promote(
    repository_root: &Path,
    options: &BTreeMap<String, String>,
) -> Result<(), PromotionError> {
    require_options(options, &["--review-packet", "--review-ack"])?;
    let (packet, _ack, staging_root) = validate_packet_and_ack(repository_root, options)?;
    require_clean_worktree(repository_root)?;
    let head = git_text(repository_root, &["rev-parse", "HEAD"])?;
    if head != packet.promotion_base_sha {
        return Err(PromotionError::new(
            PromotionErrorKind::Git,
            "promotion HEAD changed after independent review",
        ));
    }
    for path in PROMOTED_PATHS {
        let maybe_expected = git_maybe_file(repository_root, &packet.promotion_base_sha, path)?;
        let maybe_current = fs::read(repository_root.join(path)).ok();
        if maybe_expected != maybe_current {
            return Err(PromotionError::new(
                PromotionErrorKind::Git,
                format!("tracked baseline `{path}` changed after review"),
            ));
        }
    }
    transaction::replace_all_and_validate(
        repository_root,
        &staging_root,
        &PROMOTED_PATHS,
        None,
        || validate_promoted_worktree(repository_root, &packet, &staging_root),
    )?;
    println!(
        "phase13 evidence promoted transactionally at R={} review_sha256={}",
        packet.promotion_base_sha, packet.review_sha256
    );
    Ok(())
}

pub(super) fn promotion_ready(
    repository_root: &Path,
    options: &BTreeMap<String, String>,
) -> Result<(), PromotionError> {
    require_options(options, &["--review-packet", "--review-ack"])?;
    let (packet, _ack, staging_root) = validate_packet_and_ack(repository_root, options)?;
    validate_promoted_worktree(repository_root, &packet, &staging_root)?;
    tracked_reviewed_check(repository_root)?;
    println!(
        "phase13 promotion ready: R={} review_sha256={}",
        packet.promotion_base_sha, packet.review_sha256
    );
    Ok(())
}

pub(super) fn review_ack_check(
    repository_root: &Path,
    options: &BTreeMap<String, String>,
) -> Result<(), PromotionError> {
    require_options(options, &["--review-packet", "--ack"])?;
    let packet: ReviewPacket = read_json(&absolute_path(
        repository_root,
        required(options, "--review-packet")?,
    ))?;
    let acknowledgement: ReviewAcknowledgement =
        read_json(&absolute_path(repository_root, required(options, "--ack")?))?;
    validate_packet_identity(&packet)?;
    validate_review_ack(&packet, Some(&acknowledgement))?;
    println!(
        "phase13 independent review acknowledged: reviewer={} review_sha256={}",
        acknowledgement.reviewer_id, acknowledgement.review_sha256
    );
    Ok(())
}

pub(super) fn validate_packet_and_ack(
    repository_root: &Path,
    options: &BTreeMap<String, String>,
) -> Result<(ReviewPacket, ReviewAcknowledgement, PathBuf), PromotionError> {
    let packet_path = absolute_path(repository_root, required(options, "--review-packet")?);
    let ack_path = absolute_path(repository_root, required(options, "--review-ack")?);
    let packet: ReviewPacket = read_json(&packet_path)?;
    let acknowledgement: ReviewAcknowledgement = read_json(&ack_path)?;
    validate_review_ack(&packet, Some(&acknowledgement))?;
    validate_packet_identity(&packet)?;
    let staging_root = absolute_path(repository_root, &packet.staging_root);
    let replacement_sha256 = validate_staged_tree(&staging_root)?;
    if replacement_sha256 != packet.replacement_sha256 {
        return Err(PromotionError::new(
            PromotionErrorKind::Diff,
            "staged replacement hashes changed after review",
        ));
    }
    if reviewed_content_digests_from_root(&staging_root, &packet.changed_paths)?
        != (
            packet.promoted_content_sha256.clone(),
            packet.changed_content_sha256.clone(),
        )
    {
        return Err(PromotionError::new(
            PromotionErrorKind::Diff,
            "staged promoted or changed content digest changed after review",
        ));
    }
    validate_staged_ledgers(&staging_root, &replacement_sha256)?;
    let diff = canonical_diff(
        repository_root,
        &packet.promotion_base_sha,
        &staging_root,
        &packet
            .changed_paths
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
    )?;
    if diff != packet.diff || review_sha256(&packet)? != packet.review_sha256 {
        return Err(PromotionError::new(
            PromotionErrorKind::Diff,
            "prepared seven-file diff changed after review",
        ));
    }
    Ok((packet, acknowledgement, staging_root))
}

pub(super) fn validate_packet_identity(packet: &ReviewPacket) -> Result<(), PromotionError> {
    if packet.schema_version != 2
        || packet.producer_sha != PRODUCER_SHA
        || packet.bundle_sha256 != BUNDLE_SHA256
        || !valid_revision(&packet.promotion_base_sha)
        || packet.promoted_paths != promoted_paths()
        || packet.promoted_path_set_sha256 != promoted_path_set_sha256()
        || !valid_digest(&packet.promoted_content_sha256)
        || packet.changed_paths.is_empty()
        || packet.changed_path_set_sha256 != changed_path_set_sha256(&packet.changed_paths)
        || !valid_digest(&packet.changed_content_sha256)
        || !valid_path_classification(&packet.changed_paths, &packet.unchanged_paths)
        || packet
            .replacement_sha256
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            != packet
                .promoted_paths
                .iter()
                .map(String::as_str)
                .collect::<BTreeSet<_>>()
        || review_sha256(packet)? != packet.review_sha256
        || packet.witness_closure.recorded_sha256 != packet.witness_closure.recomputed_sha256
        || packet.replay_closure.recorded_sha256 != packet.replay_closure.recomputed_sha256
        || !packet.witness_closure.byte_identical
        || !packet.replay_closure.byte_identical
    {
        return Err(PromotionError::new(
            PromotionErrorKind::Schema,
            "review packet identity or closure contract is invalid",
        ));
    }
    validate_acquisition(&packet.acquisition)
}

pub(super) fn valid_path_classification(changed: &[String], unchanged: &[String]) -> bool {
    let changed = changed.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let unchanged = unchanged
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let promoted = PROMOTED_PATHS.into_iter().collect::<BTreeSet<_>>();
    changed.len() + unchanged.len() == PROMOTED_PATHS.len()
        && changed.is_disjoint(&unchanged)
        && changed.union(&unchanged).copied().collect::<BTreeSet<_>>() == promoted
}

pub(crate) fn validate_review_ack(
    packet: &ReviewPacket,
    maybe_acknowledgement: Option<&ReviewAcknowledgement>,
) -> Result<(), PromotionError> {
    let Some(acknowledgement) = maybe_acknowledgement else {
        return Err(PromotionError::new(
            PromotionErrorKind::Acknowledgement,
            "independent review acknowledgement is required",
        ));
    };
    if acknowledgement.schema_version != 2
        || acknowledgement.reviewer_id != packet.reviewer_id
        || acknowledgement.review_sha256 != packet.review_sha256
        || acknowledgement.acknowledgement.trim().is_empty()
        || !valid_utc_timestamp(&acknowledgement.reviewed_at)
    {
        return Err(PromotionError::new(
            PromotionErrorKind::Acknowledgement,
            "acknowledgement does not bind the exact reviewer and diff digest",
        ));
    }
    Ok(())
}

#[cfg(test)]
#[allow(
    dead_code,
    reason = "the path-included integration contract consumes this test constructor"
)]
pub(crate) fn review_packet_for_test(reviewer_id: &str, review_sha256: &str) -> ReviewPacket {
    let closure = ClosureReview {
        recorded_sha256: "a".repeat(64),
        recomputed_sha256: "a".repeat(64),
        byte_identical: true,
    };
    ReviewPacket {
        schema_version: 2,
        producer_sha: PRODUCER_SHA.to_owned(),
        bundle_sha256: BUNDLE_SHA256.to_owned(),
        promotion_base_sha: "b".repeat(40),
        reviewer_id: reviewer_id.to_owned(),
        acquisition: Acquisition {
            repository: PROVIDER_REPOSITORY.to_owned(),
            run_id: PROVIDER_RUN_ID,
            artifact_id: PROVIDER_ARTIFACT_ID,
            artifact_name: PROVIDER_ARTIFACT_NAME.to_owned(),
            provider_digest: PROVIDER_DIGEST.to_owned(),
            artifact_created_at: "2026-07-26T00:52:47Z".to_owned(),
            artifact_expires_at: "2026-10-24T00:50:42Z".to_owned(),
        },
        witness_closure: closure.clone(),
        replay_closure: closure,
        promoted_paths: promoted_paths(),
        promoted_path_set_sha256: promoted_path_set_sha256(),
        promoted_content_sha256: "c".repeat(64),
        changed_paths: promoted_paths(),
        unchanged_paths: Vec::new(),
        changed_path_set_sha256: changed_path_set_sha256(&promoted_paths()),
        changed_content_sha256: "d".repeat(64),
        replacement_sha256: BTreeMap::new(),
        staging_root: "target/test-stage".to_owned(),
        review_sha256: review_sha256.to_owned(),
        diff: "diff".to_owned(),
    }
}

pub(crate) fn validate_base_contract(
    producer_sha: &str,
    promotion_base_sha: &str,
    producer_is_ancestor: bool,
    witness_closure_equal: bool,
    replay_closure_equal: bool,
) -> Result<(), PromotionError> {
    if !valid_revision(producer_sha) || !valid_revision(promotion_base_sha) {
        return Err(PromotionError::new(
            PromotionErrorKind::Git,
            "P and R must be full lowercase Git identities",
        ));
    }
    if !producer_is_ancestor {
        return Err(PromotionError::new(
            PromotionErrorKind::Git,
            "producer P is not an ancestor of promotion base R",
        ));
    }
    if !witness_closure_equal || !replay_closure_equal {
        return Err(PromotionError::new(
            PromotionErrorKind::Closure,
            "producer-affecting closure drifted between P and R",
        ));
    }
    Ok(())
}

pub(super) fn tracked_reviewed_check(repository_root: &Path) -> Result<(), PromotionError> {
    let manifest: toml::Value = toml::from_str(
        &fs::read_to_string(repository_root.join(ARTIFACT_MANIFEST_PATH)).map_err(|error| {
            PromotionError::new(
                PromotionErrorKind::Filesystem,
                format!("failed to read artifact manifest: {error}"),
            )
        })?,
    )
    .map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Ledger,
            format!("invalid artifact manifest: {error}"),
        )
    })?;
    let records = manifest
        .get("artifact_schemas")
        .and_then(|value| value.get("phase13_evidence"))
        .and_then(|value| value.get("records"))
        .and_then(toml::Value::as_array)
        .ok_or_else(|| {
            PromotionError::new(
                PromotionErrorKind::Ledger,
                "reviewed Phase 13 artifact records are absent",
            )
        })?;
    if records.len() != 4 {
        return Err(PromotionError::new(
            PromotionErrorKind::Ledger,
            "reviewed Phase 13 artifact record set is incomplete",
        ));
    }
    let expected_record_paths = [
        WITNESS_PATH,
        WITNESS_PROVENANCE_PATH,
        REPLAY_EVIDENCE_PATH,
        RECEIPT_PATH,
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let mut actual_record_paths = BTreeSet::new();
    for record in records {
        let path = record
            .get("path")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| {
                PromotionError::new(PromotionErrorKind::Ledger, "artifact record path is absent")
            })?;
        let expected = record
            .get("sha256")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| {
                PromotionError::new(
                    PromotionErrorKind::Ledger,
                    "artifact record SHA-256 is absent",
                )
            })?;
        let digest_mode = record
            .get("digest_mode")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| {
                PromotionError::new(
                    PromotionErrorKind::Ledger,
                    "artifact record digest mode is absent",
                )
            })?;
        let required_mode = if path == RECEIPT_PATH {
            RECEIPT_SEMANTIC_DIGEST_MODE
        } else {
            EXACT_BYTES_DIGEST_MODE
        };
        let actual = if path == RECEIPT_PATH {
            receipt_semantic_sha256(
                &fs::read(repository_root.join(path)).map_err(filesystem_error)?,
            )?
        } else {
            file_sha256(&repository_root.join(path))?
        };
        if digest_mode != required_mode || actual != expected || !actual_record_paths.insert(path) {
            return Err(PromotionError::new(
                PromotionErrorKind::Ledger,
                format!("artifact ledger digest contract for `{path}` is stale"),
            ));
        }
    }
    if actual_record_paths != expected_record_paths {
        return Err(PromotionError::new(
            PromotionErrorKind::Ledger,
            "artifact ledger record paths are incomplete or unexpected",
        ));
    }
    let receipt: PromotionReceipt = read_json(&repository_root.join(RECEIPT_PATH))?;
    if receipt.producer_sha != PRODUCER_SHA || receipt.bundle_sha256 != BUNDLE_SHA256 {
        return Err(PromotionError::new(
            PromotionErrorKind::Ledger,
            "tracked promotion receipt is circular or has the wrong P/B",
        ));
    }
    validate_content_digest_claims(&reviewed_replacements_from_root(repository_root)?)?;
    Ok(())
}

pub(super) fn validate_promoted_worktree(
    repository_root: &Path,
    packet: &ReviewPacket,
    staging_root: &Path,
) -> Result<(), PromotionError> {
    for path in PROMOTED_PATHS {
        let current = fs::read(repository_root.join(path)).map_err(|error| {
            PromotionError::new(
                PromotionErrorKind::Transaction,
                format!("promoted path `{path}` is unreadable: {error}"),
            )
        })?;
        let staged = fs::read(staging_root.join(path)).map_err(|error| {
            PromotionError::new(
                PromotionErrorKind::Transaction,
                format!("staged path `{path}` is unreadable: {error}"),
            )
        })?;
        if current != staged {
            return Err(PromotionError::new(
                PromotionErrorKind::Transaction,
                format!("promoted path `{path}` differs from reviewed bytes"),
            ));
        }
    }
    let changed = git_text(
        repository_root,
        &["diff", "--name-only", &packet.promotion_base_sha, "--"],
    )?
    .lines()
    .map(str::to_owned)
    .chain(
        git_text(
            repository_root,
            &["ls-files", "--others", "--exclude-standard"],
        )?
        .lines()
        .map(str::to_owned),
    )
    .collect::<BTreeSet<_>>();
    let expected = packet
        .changed_paths
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if changed != expected {
        return Err(PromotionError::new(
            PromotionErrorKind::Transaction,
            "promoted worktree Git diff does not equal the reviewed changed subset",
        ));
    }
    for path in &packet.unchanged_paths {
        let baseline = git_file(repository_root, &packet.promotion_base_sha, path)?;
        let current = fs::read(repository_root.join(path)).map_err(filesystem_error)?;
        if current != baseline {
            return Err(PromotionError::new(
                PromotionErrorKind::Transaction,
                format!("unchanged reviewed path `{path}` differs from R"),
            ));
        }
    }
    Ok(())
}
