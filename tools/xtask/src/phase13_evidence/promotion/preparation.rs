use super::{
    Acquisition, BTreeMap, BUNDLE_SHA256, BundleClosure, BundleManifest, PRODUCER_SHA, Path,
    PathBuf, PromotionError, PromotionErrorKind, REPLAY_REPOSITORY_PREFIXES, ReceiptFields,
    ReviewPacket, absolute_path, acquire_provider_metadata, baseline_sha256, canonical_diff,
    changed_path_set_sha256, check_bundle, classify_reviewed_paths, closure_review,
    derive_git_closure, derive_witness_closure, format_staged_catalog, git_success, git_text,
    new_staging_root, promoted_path_set_sha256, promoted_paths, read_bundle_manifest,
    relative_path_text, render_receipt, render_replacements, replacement_sha256,
    require_clean_worktree, require_options, required, review_sha256, reviewed_content_digests,
    reviewed_content_digests_from_root, validate_base_contract, validate_bundle_contract,
    validate_bundle_files, validate_reviewer_id, validate_staged_ledgers, validate_staged_tree,
    write_json, write_replacements,
};

struct PreparationContext {
    producer_sha: String,
    bundle_sha256: String,
    reviewer_id: String,
    promotion_base_sha: String,
    bundle_root: PathBuf,
    manifest: BundleManifest,
    acquisition: Acquisition,
    witness_at_r: BundleClosure,
    replay_at_r: BundleClosure,
    staging_root: PathBuf,
    baseline_sha256: BTreeMap<String, String>,
}

struct ReviewedReplacements {
    changed_paths: Vec<String>,
    unchanged_paths: Vec<String>,
    promoted_content_sha256: String,
    changed_content_sha256: String,
    replacements: BTreeMap<String, Vec<u8>>,
}

struct StagedReview {
    replacement_sha256: BTreeMap<String, String>,
    diff: String,
}

pub(super) fn prepare(
    repository_root: &Path,
    options: &BTreeMap<String, String>,
) -> Result<(), PromotionError> {
    let context = load_context(repository_root, options)?;
    let reviewed = build_reviewed_replacements(repository_root, &context)?;
    let staged = stage_reviewed_replacements(repository_root, &context, &reviewed)?;
    let mut packet = ReviewPacket {
        schema_version: 2,
        producer_sha: context.producer_sha,
        bundle_sha256: context.bundle_sha256,
        promotion_base_sha: context.promotion_base_sha,
        reviewer_id: context.reviewer_id,
        acquisition: context.acquisition,
        witness_closure: closure_review(&context.manifest.witness_closure, &context.witness_at_r),
        replay_closure: closure_review(&context.manifest.replay_closure, &context.replay_at_r),
        promoted_paths: promoted_paths(),
        promoted_path_set_sha256: promoted_path_set_sha256(),
        promoted_content_sha256: reviewed.promoted_content_sha256,
        changed_path_set_sha256: changed_path_set_sha256(&reviewed.changed_paths),
        changed_content_sha256: reviewed.changed_content_sha256,
        changed_paths: reviewed.changed_paths,
        unchanged_paths: reviewed.unchanged_paths,
        replacement_sha256: staged.replacement_sha256,
        staging_root: relative_path_text(repository_root, &context.staging_root)?,
        review_sha256: String::new(),
        diff: staged.diff,
    };
    packet.review_sha256 = review_sha256(&packet)?;
    let review_packet = absolute_path(repository_root, required(options, "--review-packet")?);
    write_json(&review_packet, &packet, false)?;
    require_clean_worktree(repository_root)?;
    println!(
        "phase13 promotion prepared: P={} B={} R={} reviewer={} diff_sha256={} packet={}",
        packet.producer_sha,
        packet.bundle_sha256,
        packet.promotion_base_sha,
        packet.reviewer_id,
        packet.review_sha256,
        review_packet.display()
    );
    Ok(())
}

fn load_context(
    repository_root: &Path,
    options: &BTreeMap<String, String>,
) -> Result<PreparationContext, PromotionError> {
    require_options(
        options,
        &[
            "--bundle",
            "--expected-producer-sha",
            "--expected-bundle-sha256",
            "--reviewer-id",
            "--review-packet",
        ],
    )?;
    let producer_sha = required(options, "--expected-producer-sha")?.to_owned();
    let bundle_sha256 = required(options, "--expected-bundle-sha256")?.to_owned();
    if producer_sha != PRODUCER_SHA || bundle_sha256 != BUNDLE_SHA256 {
        return Err(PromotionError::new(
            PromotionErrorKind::Bundle,
            "prepare must consume the canonical P/B acquisition tuple",
        ));
    }
    let reviewer_id = required(options, "--reviewer-id")?.to_owned();
    validate_reviewer_id(&reviewer_id)?;
    require_clean_worktree(repository_root)?;
    let promotion_base_sha = git_text(repository_root, &["rev-parse", "HEAD"])?;
    validate_base_contract(
        &producer_sha,
        &promotion_base_sha,
        git_success(
            repository_root,
            &[
                "merge-base",
                "--is-ancestor",
                &producer_sha,
                &promotion_base_sha,
            ],
        )?,
        true,
        true,
    )?;
    let bundle_root = absolute_path(repository_root, required(options, "--bundle")?);
    check_bundle(&bundle_root, &producer_sha, &bundle_sha256, None, None)
        .map_err(|error| PromotionError::new(PromotionErrorKind::Bundle, error.to_string()))?;
    let manifest = read_bundle_manifest(&bundle_root)?;
    validate_bundle_contract(&manifest)?;
    let acquisition = acquire_provider_metadata()?;
    let witness_at_r = derive_witness_closure(repository_root, &promotion_base_sha)?;
    let replay_at_r = derive_git_closure(
        repository_root,
        &promotion_base_sha,
        "replay",
        &REPLAY_REPOSITORY_PREFIXES,
    )?;
    validate_base_contract(
        &producer_sha,
        &promotion_base_sha,
        true,
        witness_at_r == manifest.witness_closure,
        replay_at_r == manifest.replay_closure,
    )?;
    validate_bundle_files(&bundle_root, &manifest)?;
    Ok(PreparationContext {
        producer_sha,
        bundle_sha256,
        reviewer_id,
        promotion_base_sha: promotion_base_sha.clone(),
        bundle_root,
        manifest,
        acquisition,
        witness_at_r,
        replay_at_r,
        staging_root: new_staging_root(repository_root, &promotion_base_sha)?,
        baseline_sha256: baseline_sha256(repository_root, &promotion_base_sha)?,
    })
}

fn build_reviewed_replacements(
    repository_root: &Path,
    context: &PreparationContext,
) -> Result<ReviewedReplacements, PromotionError> {
    let provisional_changed = promoted_paths();
    let provisional_receipt = render_context_receipt(context, &provisional_changed, &[], "", "")?;
    let provisional_replacements =
        render_context_replacements(repository_root, context, &provisional_receipt)?;
    let provisional_sha256 = replacement_sha256(&provisional_replacements);
    let (changed_paths, unchanged_paths) =
        classify_reviewed_paths(&context.baseline_sha256, &provisional_sha256)?;
    if changed_paths.is_empty() {
        return Err(PromotionError::new(
            PromotionErrorKind::Diff,
            "incremental promotion must contain at least one mechanical change",
        ));
    }
    let classified_receipt =
        render_context_receipt(context, &changed_paths, &unchanged_paths, "", "")?;
    let classified_replacements =
        render_context_replacements(repository_root, context, &classified_receipt)?;
    let classified_sha256 = replacement_sha256(&classified_replacements);
    require_stable_classification(
        &context.baseline_sha256,
        &classified_sha256,
        &changed_paths,
        &unchanged_paths,
        "receipt rendering changed the incremental path classification",
    )?;
    let (promoted_content_sha256, changed_content_sha256) =
        reviewed_content_digests(&classified_replacements, &changed_paths)?;
    let receipt = render_context_receipt(
        context,
        &changed_paths,
        &unchanged_paths,
        &promoted_content_sha256,
        &changed_content_sha256,
    )?;
    let replacements = render_context_replacements(repository_root, context, &receipt)?;
    let final_sha256 = replacement_sha256(&replacements);
    require_stable_classification(
        &context.baseline_sha256,
        &final_sha256,
        &changed_paths,
        &unchanged_paths,
        "final receipt content claims changed the incremental path classification",
    )?;
    if reviewed_content_digests(&replacements, &changed_paths)?
        != (
            promoted_content_sha256.clone(),
            changed_content_sha256.clone(),
        )
    {
        return Err(PromotionError::new(
            PromotionErrorKind::Schema,
            "normalized receipt content digests did not stabilize",
        ));
    }
    Ok(ReviewedReplacements {
        changed_paths,
        unchanged_paths,
        promoted_content_sha256,
        changed_content_sha256,
        replacements,
    })
}

fn stage_reviewed_replacements(
    repository_root: &Path,
    context: &PreparationContext,
    reviewed: &ReviewedReplacements,
) -> Result<StagedReview, PromotionError> {
    write_replacements(&context.staging_root, &reviewed.replacements)?;
    format_staged_catalog(&context.staging_root)?;
    let replacement_sha256 = validate_staged_tree(&context.staging_root)?;
    require_stable_classification(
        &context.baseline_sha256,
        &replacement_sha256,
        &reviewed.changed_paths,
        &reviewed.unchanged_paths,
        "staged formatting changed the incremental path classification",
    )?;
    if reviewed_content_digests_from_root(&context.staging_root, &reviewed.changed_paths)?
        != (
            reviewed.promoted_content_sha256.clone(),
            reviewed.changed_content_sha256.clone(),
        )
    {
        return Err(PromotionError::new(
            PromotionErrorKind::Diff,
            "staged promoted or changed content digest changed after review",
        ));
    }
    validate_staged_ledgers(&context.staging_root, &replacement_sha256)?;
    let changed_paths = reviewed
        .changed_paths
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let diff = canonical_diff(
        repository_root,
        &context.promotion_base_sha,
        &context.staging_root,
        &changed_paths,
    )?;
    Ok(StagedReview {
        replacement_sha256,
        diff,
    })
}

fn render_context_receipt(
    context: &PreparationContext,
    changed_paths: &[String],
    unchanged_paths: &[String],
    promoted_content_sha256: &str,
    changed_content_sha256: &str,
) -> Result<Vec<u8>, PromotionError> {
    render_receipt(&ReceiptFields {
        producer_sha: &context.producer_sha,
        bundle_sha256: &context.bundle_sha256,
        promotion_base_sha: &context.promotion_base_sha,
        reviewer_id: &context.reviewer_id,
        acquisition: &context.acquisition,
        manifest: &context.manifest,
        changed_paths,
        unchanged_paths,
        promoted_content_sha256,
        changed_content_sha256,
    })
}

fn render_context_replacements(
    repository_root: &Path,
    context: &PreparationContext,
    receipt: &[u8],
) -> Result<BTreeMap<String, Vec<u8>>, PromotionError> {
    render_replacements(
        repository_root,
        &context.bundle_root,
        &context.manifest,
        receipt,
        &context.reviewer_id,
        &context.promotion_base_sha,
        &context.acquisition.artifact_created_at,
    )
}

fn require_stable_classification(
    baseline_sha256: &BTreeMap<String, String>,
    replacement_sha256: &BTreeMap<String, String>,
    changed_paths: &[String],
    unchanged_paths: &[String],
    message: &str,
) -> Result<(), PromotionError> {
    if classify_reviewed_paths(baseline_sha256, replacement_sha256)?
        == (changed_paths.to_vec(), unchanged_paths.to_vec())
    {
        return Ok(());
    }
    Err(PromotionError::new(PromotionErrorKind::Diff, message))
}
