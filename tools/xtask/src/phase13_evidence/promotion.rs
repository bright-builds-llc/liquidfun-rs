//! Independent review and transactional promotion for canonical Phase 13 evidence.

#[path = "promotion/transaction.rs"]
mod transaction;

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt::{self, Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::phase13_evidence::bundle::{ClosureEntry, check_bundle, closure_digest};

const USAGE: &str = r"Usage:
  cargo xtask phase13 evidence prepare --bundle <path> --expected-producer-sha <P> --expected-bundle-sha256 <B> --reviewer-id <id> --review-packet <path>
  cargo xtask phase13 evidence review-ack check --review-packet <path> --ack <path>
  cargo xtask phase13 evidence promote --review-packet <path> --review-ack <path>
  cargo xtask phase13 evidence promotion-ready --review-packet <path> --review-ack <path>
  cargo xtask phase13 evidence check --tracked --require-reviewed";

const PRODUCER_SHA: &str = "dbf5044ea8750aa3eb7a3c7b95b6a36b326f3d7e";
const BUNDLE_SHA256: &str = "1eba915ed7cb634b54e0f8d89b0d2be4112bae6f3d3adac6e83ffc355217d775";
const UPSTREAM_REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const PROVIDER_REPOSITORY: &str = "bright-builds-llc/liquidfun-rs";
const PROVIDER_RUN_ID: u64 = 30_192_804_429;
const PROVIDER_ARTIFACT_ID: u64 = 8_629_151_708;
const PROVIDER_ARTIFACT_NAME: &str =
    "phase13-staged-30192804429-dbf5044ea8750aa3eb7a3c7b95b6a36b326f3d7e";
const PROVIDER_DIGEST: &str =
    "sha256:89870f5ea64842ccdaeb69be65b2f6fc3fff1660c4e1955db587d1ede8e22934";
const MATERIALS_MANIFEST: &str = "tools/reference/phase9-lifecycle-contact-witness.materials.json";
const REPLAY_EVIDENCE_PATH: &str =
    "reference/artifacts/catalog/rigid-stack-v1.replay-evidence.json";
const RECEIPT_PATH: &str = "reference/artifacts/phase13/promotion-receipt.json";
const WITNESS_PATH: &str = "reference/artifacts/phase9/lifecycle-contact-witnesses.json";
const WITNESS_PROVENANCE_PATH: &str =
    "reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json";
const CATALOG_PATH: &str = "crates/liquidfun-differential/src/fixtures/replay/catalog.rs";
const SOURCE_MAP_PATH: &str = "reference/source-map.toml";
const ARTIFACT_MANIFEST_PATH: &str = "reference/artifacts/manifest.toml";
const PROMOTED_PATHS: [&str; 7] = [
    CATALOG_PATH,
    REPLAY_EVIDENCE_PATH,
    RECEIPT_PATH,
    ARTIFACT_MANIFEST_PATH,
    WITNESS_PATH,
    WITNESS_PROVENANCE_PATH,
    SOURCE_MAP_PATH,
];
const WITNESS_REPOSITORY_PREFIXES: [&str; 3] = [
    MATERIALS_MANIFEST,
    "tools/xtask/src/phase13_evidence.rs",
    "tools/xtask/src/phase13_evidence/bundle.rs",
];
const REPLAY_REPOSITORY_PREFIXES: [&str; 16] = [
    "Cargo.lock",
    "Cargo.toml",
    "rust-toolchain.toml",
    "crates/liquidfun",
    "crates/liquidfun-differential",
    "crates/liquidfun-test-protocol",
    "protocol",
    "scenarios/catalog",
    "scenarios/regressions/catalog-manifest.json",
    "tools/xtask/Cargo.toml",
    "tools/xtask/src/phase13_evidence.rs",
    "tools/xtask/src/phase13_evidence/bundle.rs",
    "tools/reference/CMakeLists.txt",
    "tools/reference/CMakePresets.json",
    "tools/reference/src",
    "reference/upstream-lock.toml",
];

static NEXT_STAGE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PromotionErrorKind {
    Acknowledgement,
    Bundle,
    Closure,
    Diff,
    Filesystem,
    Git,
    Ledger,
    Path,
    Provider,
    Schema,
    Transaction,
    Usage,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PromotionError {
    kind: PromotionErrorKind,
    message: String,
}

impl PromotionError {
    pub(super) fn new(kind: PromotionErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    fn usage(message: impl Into<String>) -> Self {
        Self::new(
            PromotionErrorKind::Usage,
            format!("{}\n\n{USAGE}", message.into()),
        )
    }

    #[allow(dead_code, reason = "contract tests inspect stable failure categories")]
    pub(crate) const fn kind(&self) -> PromotionErrorKind {
        self.kind
    }
}

impl Display for PromotionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "phase13 promotion/{:?}: {}",
            self.kind, self.message
        )
    }
}

impl std::error::Error for PromotionError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Acquisition {
    repository: String,
    run_id: u64,
    artifact_id: u64,
    artifact_name: String,
    provider_digest: String,
    artifact_created_at: String,
    artifact_expires_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClosureReview {
    recorded_sha256: String,
    recomputed_sha256: String,
    byte_identical: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReviewPacket {
    pub(crate) schema_version: u32,
    pub(crate) producer_sha: String,
    pub(crate) bundle_sha256: String,
    pub(crate) promotion_base_sha: String,
    pub(crate) reviewer_id: String,
    acquisition: Acquisition,
    witness_closure: ClosureReview,
    replay_closure: ClosureReview,
    pub(crate) promoted_paths: Vec<String>,
    pub(crate) promoted_path_set_sha256: String,
    pub(crate) changed_paths: Vec<String>,
    pub(crate) unchanged_paths: Vec<String>,
    pub(crate) changed_path_set_sha256: String,
    pub(crate) replacement_sha256: BTreeMap<String, String>,
    pub(crate) staging_root: String,
    pub(crate) review_sha256: String,
    pub(crate) diff: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReviewAcknowledgement {
    pub(crate) schema_version: u32,
    pub(crate) reviewer_id: String,
    pub(crate) review_sha256: String,
    pub(crate) acknowledgement: String,
    pub(crate) reviewed_at: String,
}

#[derive(Debug, Serialize)]
struct PromotionReceipt {
    schema_version: u32,
    producer_sha: String,
    bundle_sha256: String,
    promotion_base_sha: String,
    acquisition: Acquisition,
    independent_reviewer_id: String,
    promoted_paths: Vec<String>,
    promoted_path_set_sha256: String,
    changed_paths: Vec<String>,
    unchanged_paths: Vec<String>,
    changed_path_set_sha256: String,
    producer_closures: ProducerClosures,
    q_contract: PromotionCommitContract,
}

#[derive(Debug, Serialize)]
struct ProducerClosures {
    witness_sha256: String,
    replay_sha256: String,
    recomputed_at_r: bool,
}

#[derive(Debug, Serialize)]
struct PromotionCommitContract {
    required_first_parent: String,
    required_trailers: BTreeMap<String, String>,
    q_sha_recorded: bool,
    acceptance_sha_recorded: bool,
}

#[derive(Debug, Deserialize)]
struct BundleManifest {
    producer_sha: String,
    bundle_sha256: String,
    upstream_revision: String,
    witness_closure: BundleClosure,
    replay_closure: BundleClosure,
    sealed_input_sha256: String,
    d1_input_sha256: String,
    native_d0_repeat_sha256: [String; 2],
    d1_oracle_identity_sha256: String,
    d1_result: String,
    diagnosis: serde_json::Value,
    files: Vec<BundleFileEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct BundleClosure {
    schema_version: u32,
    label: String,
    digest: String,
    entries: Vec<ClosureEntry>,
}

#[derive(Debug, Deserialize)]
struct BundleFileEntry {
    path: String,
    sha256: String,
    record_class: String,
    source_revision: String,
    source_path: String,
    derivation_kind: String,
    alteration_summary: String,
    notice_refs: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ProviderArtifact {
    id: u64,
    name: String,
    digest: String,
    created_at: String,
    expires_at: String,
    workflow_run: ProviderWorkflowRun,
}

#[derive(Debug, Deserialize)]
struct ProviderWorkflowRun {
    id: u64,
    head_sha: String,
}

#[derive(Debug, Deserialize)]
struct MaterialsManifest {
    schema_version: u32,
    target: String,
    preset: String,
    materials: Vec<MaterialEntry>,
}

#[derive(Debug, Deserialize)]
struct MaterialEntry {
    kind: String,
    identity: String,
}

pub(crate) fn handles(args: &[String]) -> bool {
    matches!(
        args,
        [namespace, command, ..]
            if namespace == "evidence"
                && matches!(
                    command.as_str(),
                    "prepare" | "promote" | "promotion-ready" | "review-ack"
                )
    ) || matches!(
        args,
        [namespace, command, first, second]
            if namespace == "evidence"
                && command == "check"
                && first == "--tracked"
                && second == "--require-reviewed"
    )
}

pub(crate) fn run(args: &[String]) -> Result<(), PromotionError> {
    let [namespace, command, tail @ ..] = args else {
        return Err(PromotionError::usage("expected `evidence <command>`"));
    };
    if namespace != "evidence" {
        return Err(PromotionError::usage(
            "only the `evidence` namespace is available",
        ));
    }
    let repository_root = repository_root()?;
    match command.as_str() {
        "prepare" => prepare(&repository_root, &parse_options(tail)?),
        "promote" => promote(&repository_root, &parse_options(tail)?),
        "promotion-ready" => promotion_ready(&repository_root, &parse_options(tail)?),
        "review-ack" => {
            let [check, options @ ..] = tail else {
                return Err(PromotionError::usage(
                    "review-ack requires the `check` subcommand",
                ));
            };
            if check != "check" {
                return Err(PromotionError::usage(
                    "review-ack requires the `check` subcommand",
                ));
            }
            review_ack_check(&repository_root, &parse_options(options)?)
        }
        "check" if tail == ["--tracked", "--require-reviewed"] => {
            tracked_reviewed_check(&repository_root)
        }
        unknown => Err(PromotionError::usage(format!(
            "unknown promotion command `{unknown}`"
        ))),
    }
}

fn prepare(
    repository_root: &Path,
    options: &BTreeMap<String, String>,
) -> Result<(), PromotionError> {
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
    let producer_sha = required(options, "--expected-producer-sha")?;
    let bundle_sha256 = required(options, "--expected-bundle-sha256")?;
    if producer_sha != PRODUCER_SHA || bundle_sha256 != BUNDLE_SHA256 {
        return Err(PromotionError::new(
            PromotionErrorKind::Bundle,
            "prepare must consume the canonical P/B acquisition tuple",
        ));
    }
    let reviewer_id = required(options, "--reviewer-id")?;
    validate_reviewer_id(reviewer_id)?;
    require_clean_worktree(repository_root)?;
    let promotion_base_sha = git_text(repository_root, &["rev-parse", "HEAD"])?;
    validate_base_contract(
        producer_sha,
        &promotion_base_sha,
        git_success(
            repository_root,
            &[
                "merge-base",
                "--is-ancestor",
                producer_sha,
                &promotion_base_sha,
            ],
        )?,
        true,
        true,
    )?;

    let bundle_root = absolute_path(repository_root, required(options, "--bundle")?);
    check_bundle(&bundle_root, producer_sha, bundle_sha256, None, None)
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
        producer_sha,
        &promotion_base_sha,
        true,
        witness_at_r == manifest.witness_closure,
        replay_at_r == manifest.replay_closure,
    )?;
    validate_bundle_files(&bundle_root, &manifest)?;

    let staging_root = new_staging_root(repository_root, &promotion_base_sha)?;
    let baseline_sha256 = baseline_sha256(repository_root, &promotion_base_sha)?;
    let provisional_changed = promoted_paths();
    let provisional_receipt = render_receipt(
        producer_sha,
        bundle_sha256,
        &promotion_base_sha,
        reviewer_id,
        &acquisition,
        &manifest,
        &provisional_changed,
        &[],
    )?;
    let provisional_replacements = render_replacements(
        repository_root,
        &bundle_root,
        &manifest,
        &provisional_receipt,
        reviewer_id,
        &promotion_base_sha,
        &acquisition.artifact_created_at,
    )?;
    let provisional_sha256 = replacement_sha256(&provisional_replacements);
    let (changed_paths, unchanged_paths) =
        classify_reviewed_paths(&baseline_sha256, &provisional_sha256)?;
    if changed_paths.is_empty() {
        return Err(PromotionError::new(
            PromotionErrorKind::Diff,
            "incremental promotion must contain at least one mechanical change",
        ));
    }
    let receipt = render_receipt(
        producer_sha,
        bundle_sha256,
        &promotion_base_sha,
        reviewer_id,
        &acquisition,
        &manifest,
        &changed_paths,
        &unchanged_paths,
    )?;
    let replacements = render_replacements(
        repository_root,
        &bundle_root,
        &manifest,
        &receipt,
        reviewer_id,
        &promotion_base_sha,
        &acquisition.artifact_created_at,
    )?;
    let final_sha256 = replacement_sha256(&replacements);
    let final_classification = classify_reviewed_paths(&baseline_sha256, &final_sha256)?;
    if final_classification != (changed_paths.clone(), unchanged_paths.clone()) {
        return Err(PromotionError::new(
            PromotionErrorKind::Diff,
            "receipt rendering changed the incremental path classification",
        ));
    }
    write_replacements(&staging_root, &replacements)?;
    format_staged_catalog(&staging_root)?;
    let replacement_sha256 = validate_staged_tree(&staging_root)?;
    if classify_reviewed_paths(&baseline_sha256, &replacement_sha256)?
        != (changed_paths.clone(), unchanged_paths.clone())
    {
        return Err(PromotionError::new(
            PromotionErrorKind::Diff,
            "staged formatting changed the incremental path classification",
        ));
    }
    validate_staged_ledgers(&staging_root, &replacement_sha256)?;
    let diff = canonical_diff(
        repository_root,
        &promotion_base_sha,
        &staging_root,
        &changed_paths.iter().map(String::as_str).collect::<Vec<_>>(),
    )?;
    let mut packet = ReviewPacket {
        schema_version: 2,
        producer_sha: producer_sha.to_owned(),
        bundle_sha256: bundle_sha256.to_owned(),
        promotion_base_sha,
        reviewer_id: reviewer_id.to_owned(),
        acquisition,
        witness_closure: closure_review(&manifest.witness_closure, &witness_at_r),
        replay_closure: closure_review(&manifest.replay_closure, &replay_at_r),
        promoted_paths: promoted_paths(),
        promoted_path_set_sha256: promoted_path_set_sha256(),
        changed_path_set_sha256: changed_path_set_sha256(&changed_paths),
        changed_paths,
        unchanged_paths,
        replacement_sha256,
        staging_root: relative_path_text(repository_root, &staging_root)?,
        review_sha256: String::new(),
        diff,
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

fn promote(
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

fn promotion_ready(
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

fn review_ack_check(
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

fn validate_packet_and_ack(
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

fn validate_packet_identity(packet: &ReviewPacket) -> Result<(), PromotionError> {
    if packet.schema_version != 2
        || packet.producer_sha != PRODUCER_SHA
        || packet.bundle_sha256 != BUNDLE_SHA256
        || !valid_revision(&packet.promotion_base_sha)
        || packet.promoted_paths != promoted_paths()
        || packet.promoted_path_set_sha256 != promoted_path_set_sha256()
        || packet.changed_paths.is_empty()
        || packet.changed_path_set_sha256 != changed_path_set_sha256(&packet.changed_paths)
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

fn valid_path_classification(changed: &[String], unchanged: &[String]) -> bool {
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
        changed_paths: promoted_paths(),
        unchanged_paths: Vec::new(),
        changed_path_set_sha256: changed_path_set_sha256(&promoted_paths()),
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

fn tracked_reviewed_check(repository_root: &Path) -> Result<(), PromotionError> {
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
        let actual = file_sha256(&repository_root.join(path))?;
        if actual != expected {
            return Err(PromotionError::new(
                PromotionErrorKind::Ledger,
                format!("artifact ledger hash for `{path}` is stale"),
            ));
        }
    }
    let receipt: serde_json::Value = read_json(&repository_root.join(RECEIPT_PATH))?;
    if receipt
        .get("producer_sha")
        .and_then(serde_json::Value::as_str)
        != Some(PRODUCER_SHA)
        || receipt
            .get("bundle_sha256")
            .and_then(serde_json::Value::as_str)
            != Some(BUNDLE_SHA256)
        || receipt.get("q_sha").is_some()
        || receipt.get("acceptance_sha").is_some()
    {
        return Err(PromotionError::new(
            PromotionErrorKind::Ledger,
            "tracked promotion receipt is circular or has the wrong P/B",
        ));
    }
    Ok(())
}

fn validate_promoted_worktree(
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

fn render_replacements(
    repository_root: &Path,
    bundle_root: &Path,
    manifest: &BundleManifest,
    receipt: &[u8],
    reviewer_id: &str,
    promotion_base_sha: &str,
    generation_timestamp: &str,
) -> Result<BTreeMap<String, Vec<u8>>, PromotionError> {
    let mut replacements = BTreeMap::new();
    replacements.insert(
        WITNESS_PATH.to_owned(),
        fs::read(bundle_root.join("evidence/witness.json")).map_err(filesystem_error)?,
    );
    replacements.insert(
        WITNESS_PROVENANCE_PATH.to_owned(),
        render_witness_provenance(bundle_root, generation_timestamp)?,
    );
    replacements.insert(
        REPLAY_EVIDENCE_PATH.to_owned(),
        fs::read(bundle_root.join("evidence/replay.json")).map_err(filesystem_error)?,
    );
    replacements.insert(RECEIPT_PATH.to_owned(), receipt.to_vec());
    replacements.insert(
        CATALOG_PATH.to_owned(),
        render_catalog(
            &fs::read_to_string(repository_root.join(CATALOG_PATH)).map_err(|error| {
                PromotionError::new(
                    PromotionErrorKind::Filesystem,
                    format!("failed to read catalog replay source: {error}"),
                )
            })?,
        )?
        .into_bytes(),
    );
    replacements.insert(
        SOURCE_MAP_PATH.to_owned(),
        render_source_map(
            &fs::read_to_string(repository_root.join(SOURCE_MAP_PATH)).map_err(filesystem_error)?,
        )?
        .into_bytes(),
    );
    let replacement_digest = |path: &'static str| {
        replacements
            .get(path)
            .map(|bytes| sha256(bytes))
            .ok_or_else(|| {
                PromotionError::new(
                    PromotionErrorKind::Schema,
                    format!("replacement `{path}` is absent"),
                )
            })
    };
    let artifact_hashes = [
        (
            WITNESS_PATH,
            replacement_digest(WITNESS_PATH)?,
            "witness",
            manifest
                .files
                .iter()
                .find(|entry| entry.path == "evidence/witness.json"),
            PRODUCER_SHA,
        ),
        (
            WITNESS_PROVENANCE_PATH,
            replacement_digest(WITNESS_PROVENANCE_PATH)?,
            "witness",
            manifest
                .files
                .iter()
                .find(|entry| entry.path == "evidence/witness.provenance.json"),
            PRODUCER_SHA,
        ),
        (
            REPLAY_EVIDENCE_PATH,
            replacement_digest(REPLAY_EVIDENCE_PATH)?,
            "replay_evidence",
            manifest
                .files
                .iter()
                .find(|entry| entry.path == "evidence/replay.json"),
            PRODUCER_SHA,
        ),
        (
            RECEIPT_PATH,
            sha256(receipt),
            "promotion_receipt",
            None,
            promotion_base_sha,
        ),
    ];
    replacements.insert(
        ARTIFACT_MANIFEST_PATH.to_owned(),
        render_artifact_manifest(
            &fs::read_to_string(repository_root.join(ARTIFACT_MANIFEST_PATH))
                .map_err(filesystem_error)?,
            &artifact_hashes,
            reviewer_id,
        )?
        .into_bytes(),
    );
    Ok(replacements)
}

fn render_receipt(
    producer_sha: &str,
    bundle_sha256: &str,
    promotion_base_sha: &str,
    reviewer_id: &str,
    acquisition: &Acquisition,
    manifest: &BundleManifest,
    changed_paths: &[String],
    unchanged_paths: &[String],
) -> Result<Vec<u8>, PromotionError> {
    let required_trailers = BTreeMap::from([
        ("Phase13-Producer-SHA".to_owned(), producer_sha.to_owned()),
        ("Phase13-Bundle-SHA256".to_owned(), bundle_sha256.to_owned()),
        (
            "Phase13-Promotion-Base-SHA".to_owned(),
            promotion_base_sha.to_owned(),
        ),
    ]);
    json_bytes(&PromotionReceipt {
        schema_version: 2,
        producer_sha: producer_sha.to_owned(),
        bundle_sha256: bundle_sha256.to_owned(),
        promotion_base_sha: promotion_base_sha.to_owned(),
        acquisition: acquisition.clone(),
        independent_reviewer_id: reviewer_id.to_owned(),
        promoted_paths: promoted_paths(),
        promoted_path_set_sha256: promoted_path_set_sha256(),
        changed_paths: changed_paths.to_vec(),
        unchanged_paths: unchanged_paths.to_vec(),
        changed_path_set_sha256: changed_path_set_sha256(changed_paths),
        producer_closures: ProducerClosures {
            witness_sha256: manifest.witness_closure.digest.clone(),
            replay_sha256: manifest.replay_closure.digest.clone(),
            recomputed_at_r: true,
        },
        q_contract: PromotionCommitContract {
            required_first_parent: promotion_base_sha.to_owned(),
            required_trailers,
            q_sha_recorded: false,
            acceptance_sha_recorded: false,
        },
    })
}

fn render_witness_provenance(
    bundle_root: &Path,
    generation_timestamp: &str,
) -> Result<Vec<u8>, PromotionError> {
    let mut value: serde_json::Value =
        read_json(&bundle_root.join("evidence/witness.provenance.json"))?;
    let object = value.as_object_mut().ok_or_else(|| {
        PromotionError::new(
            PromotionErrorKind::Schema,
            "witness provenance is not an object",
        )
    })?;
    object.insert(
        "exact_argv".to_owned(),
        serde_json::json!([
            "target/reference/oracle-debug/phase9-lifecycle-contact-witness",
            "--output",
            WITNESS_PATH,
            "--provenance",
            WITNESS_PROVENANCE_PATH
        ]),
    );
    object.insert(
        "generation_timestamp".to_owned(),
        serde_json::Value::String(generation_timestamp.to_owned()),
    );
    json_bytes(&value)
}

fn render_catalog(current: &str) -> Result<String, PromotionError> {
    if current.contains("RIGID_STACK_REPLAY_EVIDENCE_PATH") {
        if current.contains("fn validate_rigid_stack_replay_evidence(")
            && current
                .contains("validate_rigid_stack_replay_evidence(&canonical_root, &manifest)?;")
        {
            return Ok(current.to_owned());
        }
        return Err(PromotionError::new(
            PromotionErrorKind::Schema,
            "existing Phase 13 catalog binding is incomplete",
        ));
    }
    let with_constant = current.replacen(
        "const MANIFEST_PATH: &str = \"scenarios/regressions/catalog-manifest.json\";",
        "const MANIFEST_PATH: &str = \"scenarios/regressions/catalog-manifest.json\";\n\
const RIGID_STACK_REPLAY_EVIDENCE_PATH: &str =\n\
    \"reference/artifacts/catalog/rigid-stack-v1.replay-evidence.json\";",
        1,
    );
    if with_constant == current {
        return Err(PromotionError::new(
            PromotionErrorKind::Schema,
            "catalog manifest constant insertion point is absent",
        ));
    }
    let with_call = with_constant.replacen(
        "    validate_manifest_header(&manifest)?;\n",
        "    validate_manifest_header(&manifest)?;\n\
    validate_rigid_stack_replay_evidence(&canonical_root, &manifest)?;\n",
        1,
    );
    if with_call == with_constant {
        return Err(PromotionError::new(
            PromotionErrorKind::Schema,
            "catalog replay-evidence validation insertion point is absent",
        ));
    }
    Ok(format!("{with_call}\n{CATALOG_VALIDATOR}"))
}

const CATALOG_VALIDATOR: &str = r#"#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RigidStackReplayEvidence {
    schema_version: u32,
    upstream_revision: String,
    resolved_scenario_path: String,
    sealed_input_sha256: String,
    native_d0_repeat_sha256: [String; 2],
    d1_oracle_identity_sha256: String,
    d1_result: String,
    diagnosis: serde_json::Value,
}

fn validate_rigid_stack_replay_evidence(
    canonical_root: &Path,
    manifest: &RegressionManifest,
) -> Result<(), CatalogRegressionError> {
    let bytes = read_regular_confined(
        canonical_root,
        Path::new(RIGID_STACK_REPLAY_EVIDENCE_PATH),
        MAXIMUM_MANIFEST_BYTES,
        CatalogRegressionErrorKind::InvalidManifest,
    )?;
    let evidence: RigidStackReplayEvidence =
        serde_json::from_slice(&bytes).map_err(|_error| {
            CatalogRegressionError::new(CatalogRegressionErrorKind::InvalidManifest)
        })?;
    let rigid_stack = manifest
        .entries
        .iter()
        .find(|entry| entry.fixture_id == "rigid-stack-v1")
        .ok_or_else(|| {
            CatalogRegressionError::new(CatalogRegressionErrorKind::InvalidManifest)
        })?;
    let diagnosis = &evidence.diagnosis;
    let reviewed_projection = diagnosis
        .pointer("/reviewed_schema/projection_version")
        .and_then(serde_json::Value::as_str);
    let current_projection = diagnosis
        .pointer("/current_schema/projection_version")
        .and_then(serde_json::Value::as_str);
    let reviewed_resolved = diagnosis
        .get("reviewed_resolved_sha256")
        .and_then(serde_json::Value::as_str);
    let current_resolved = diagnosis
        .get("current_resolved_sha256")
        .and_then(serde_json::Value::as_str);
    let valid_digest = |value: &str| {
        value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    };
    if evidence.schema_version != 1
        || evidence.upstream_revision != PINNED_UPSTREAM_REVISION
        || evidence.resolved_scenario_path != rigid_stack.path
        || evidence.sealed_input_sha256 != rigid_stack.resolved_sha256.as_str()
        || evidence.native_d0_repeat_sha256[0] != evidence.native_d0_repeat_sha256[1]
        || !valid_digest(&evidence.native_d0_repeat_sha256[0])
        || !valid_digest(&evidence.d1_oracle_identity_sha256)
        || evidence.d1_result != "match"
        || diagnosis.get("drift_class").and_then(serde_json::Value::as_str)
            != Some("capture_schema_drift")
        || reviewed_projection != Some("legacy_physics_v1")
        || current_projection != Some("expanded_checkpoint_v1")
        || reviewed_resolved != Some(rigid_stack.resolved_sha256.as_str())
        || current_resolved != Some(rigid_stack.resolved_sha256.as_str())
    {
        return Err(CatalogRegressionError::new(
            CatalogRegressionErrorKind::InvalidManifest,
        ));
    }
    Ok(())
}
"#;

fn render_source_map(current: &str) -> Result<String, PromotionError> {
    let replay_count = current.match_indices(REPLAY_EVIDENCE_PATH).count();
    let receipt_count = current.match_indices(RECEIPT_PATH).count();
    if replay_count == 1 && receipt_count == 1 {
        return Ok(current.to_owned());
    }
    if replay_count != 0 || receipt_count != 0 {
        return Err(PromotionError::new(
            PromotionErrorKind::Ledger,
            "source map contains an incomplete or duplicate Phase 13 mapping",
        ));
    }
    Ok(format!(
        "{current}\n[[mapping]]\n\
local_path = \"{REPLAY_EVIDENCE_PATH}\"\n\
upstream_revision = \"{UPSTREAM_REVISION}\"\n\
upstream_path = \".\"\n\
derivation_kind = \"repository-authored-replay-verification\"\n\
alteration_summary = \"Repository-authored canonical D0/D1 replay evidence preserving the reviewed legacy physics projection while validating expanded checkpoint diagnostics separately; no upstream source, raw object memory, or Rust-produced expectation is copied.\"\n\
notice_class = \"provenance-only\"\n\n\
[[mapping]]\n\
local_path = \"{RECEIPT_PATH}\"\n\
upstream_revision = \"{UPSTREAM_REVISION}\"\n\
upstream_path = \".\"\n\
derivation_kind = \"repository-authored-promotion-receipt\"\n\
alteration_summary = \"Repository-authored non-circular P/B/R acquisition and review contract for the exact promoted file set; no upstream source, raw object memory, or Rust-produced expectation is copied.\"\n\
notice_class = \"provenance-only\"\n"
    ))
}

fn render_artifact_manifest(
    current: &str,
    artifact_hashes: &[(&str, String, &str, Option<&BundleFileEntry>, &str); 4],
    reviewer_id: &str,
) -> Result<String, PromotionError> {
    const MARKER: &str = "[[artifact_schemas.phase13_evidence.records]]";
    let existing: toml::Value = toml::from_str(current).map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Ledger,
            format!("invalid artifact manifest before replacement: {error}"),
        )
    })?;
    let maybe_records = existing
        .get("artifact_schemas")
        .and_then(|value| value.get("phase13_evidence"))
        .and_then(|value| value.get("records"))
        .and_then(toml::Value::as_array);
    let mut rendered = match (current.find(MARKER), maybe_records) {
        (None, None) => current.to_owned(),
        (Some(index), Some(records)) if records.len() == 4 => {
            let suffix = &current[index..];
            if suffix.match_indices(MARKER).count() != 4 {
                return Err(PromotionError::new(
                    PromotionErrorKind::Ledger,
                    "existing Phase 13 artifact rows are not a closed four-record tail",
                ));
            }
            current[..index].trim_end().to_owned()
        }
        _ => {
            return Err(PromotionError::new(
                PromotionErrorKind::Ledger,
                "existing Phase 13 artifact rows are incomplete or duplicated",
            ));
        }
    };
    for (path, digest, record_class, maybe_entry, generator_revision) in artifact_hashes {
        let (
            source_revision,
            source_path,
            derivation_kind,
            alteration_summary,
            notice_refs,
        ) = maybe_entry.map_or_else(
            || {
                (
                    UPSTREAM_REVISION,
                    ".",
                    "repository-authored-promotion-receipt",
                    "Repository-authored review and promotion identity for a byte-exact staged evidence bundle; no upstream source, raw object memory, or Rust-produced expectations are copied.",
                    vec!["THIRD_PARTY_NOTICES.md".to_owned()],
                )
            },
            |entry| {
                (
                    entry.source_revision.as_str(),
                    entry.source_path.as_str(),
                    entry.derivation_kind.as_str(),
                    entry.alteration_summary.as_str(),
                    entry.notice_refs.clone(),
                )
            },
        );
        if notice_refs != ["THIRD_PARTY_NOTICES.md"] {
            return Err(PromotionError::new(
                PromotionErrorKind::Ledger,
                "bundle evidence record has incomplete notice metadata",
            ));
        }
        rendered.push_str(&format!(
            "\n[[artifact_schemas.phase13_evidence.records]]\n\
record_class = \"{record_class}\"\n\
path = \"{path}\"\n\
sha256 = \"{digest}\"\n\
generator_revision = \"{generator_revision}\"\n\
producer_sha = \"{PRODUCER_SHA}\"\n\
bundle_sha256 = \"{BUNDLE_SHA256}\"\n\
source_revision = \"{source_revision}\"\n\
source_path = \"{source_path}\"\n\
derivation_kind = \"{derivation_kind}\"\n\
alteration_summary = \"{alteration_summary}\"\n\
notice_refs = [\"THIRD_PARTY_NOTICES.md\"]\n\
reviewer = \"{reviewer_id}\"\n"
        ));
    }
    Ok(rendered)
}

fn validate_bundle_contract(manifest: &BundleManifest) -> Result<(), PromotionError> {
    if manifest.producer_sha != PRODUCER_SHA
        || manifest.bundle_sha256 != BUNDLE_SHA256
        || manifest.upstream_revision != UPSTREAM_REVISION
        || manifest.sealed_input_sha256 != manifest.d1_input_sha256
        || manifest.native_d0_repeat_sha256[0] != manifest.native_d0_repeat_sha256[1]
        || !valid_digest(&manifest.native_d0_repeat_sha256[0])
        || !valid_digest(&manifest.d1_oracle_identity_sha256)
        || manifest.d1_result != "match"
        || manifest
            .diagnosis
            .get("drift_class")
            .and_then(serde_json::Value::as_str)
            != Some("capture_schema_drift")
        || manifest
            .diagnosis
            .pointer("/reviewed_schema/projection_version")
            .and_then(serde_json::Value::as_str)
            != Some("legacy_physics_v1")
        || manifest
            .diagnosis
            .pointer("/current_schema/projection_version")
            .and_then(serde_json::Value::as_str)
            != Some("expanded_checkpoint_v1")
    {
        return Err(PromotionError::new(
            PromotionErrorKind::Bundle,
            "bundle does not satisfy the canonical diagnosis-selected D0/D1 contract",
        ));
    }
    Ok(())
}

fn validate_bundle_files(
    bundle_root: &Path,
    manifest: &BundleManifest,
) -> Result<(), PromotionError> {
    let expected = [
        "evidence/replay.json",
        "evidence/witness.json",
        "evidence/witness.provenance.json",
        "sealed/rigid-stack-v1.json",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let actual = manifest
        .files
        .iter()
        .map(|entry| entry.path.as_str())
        .collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(PromotionError::new(
            PromotionErrorKind::Bundle,
            "bundle evidence file set is not canonical",
        ));
    }
    for entry in &manifest.files {
        let actual = file_sha256(&bundle_root.join(&entry.path))?;
        if actual != entry.sha256
            || entry.source_revision != UPSTREAM_REVISION
            || entry.derivation_kind.trim().is_empty()
            || entry.alteration_summary.trim().is_empty()
            || entry.notice_refs != ["THIRD_PARTY_NOTICES.md"]
            || !matches!(entry.record_class.as_str(), "witness" | "replay_evidence")
        {
            return Err(PromotionError::new(
                PromotionErrorKind::Bundle,
                format!("bundle file `{}` has stale bytes or metadata", entry.path),
            ));
        }
    }
    let sealed =
        fs::read(bundle_root.join("sealed/rigid-stack-v1.json")).map_err(filesystem_error)?;
    let tracked = fs::read(
        bundle_root
            .ancestors()
            .find(|candidate| candidate.join("Cargo.toml").is_file())
            .unwrap_or(bundle_root)
            .join("scenarios/catalog/rigid-stack-v1.json"),
    )
    .unwrap_or_else(|_| sealed.clone());
    if sha256(&sealed) != manifest.sealed_input_sha256 || sealed != tracked {
        return Err(PromotionError::new(
            PromotionErrorKind::Bundle,
            "sealed rigid-stack input differs from the reviewed repository bytes",
        ));
    }
    Ok(())
}

fn derive_witness_closure(
    repository_root: &Path,
    revision: &str,
) -> Result<BundleClosure, PromotionError> {
    let manifest: MaterialsManifest = read_json(&repository_root.join(MATERIALS_MANIFEST))?;
    if manifest.schema_version != 1
        || manifest.target != "phase9-lifecycle-contact-witness"
        || manifest.preset != "oracle-debug"
    {
        return Err(PromotionError::new(
            PromotionErrorKind::Closure,
            "witness materials manifest identity is invalid",
        ));
    }
    let mut entries = derive_git_entries(repository_root, revision, &WITNESS_REPOSITORY_PREFIXES)?;
    for material in manifest.materials {
        if !matches!(material.kind.as_str(), "source" | "header" | "build_rule") {
            continue;
        }
        let candidate = repository_root.join(&material.identity);
        if candidate.is_file() {
            entries.insert(material.identity, file_sha256(&candidate)?);
        }
    }
    closure_from_entries("witness", entries)
}

fn derive_git_closure(
    repository_root: &Path,
    revision: &str,
    label: &str,
    prefixes: &[&str],
) -> Result<BundleClosure, PromotionError> {
    closure_from_entries(
        label,
        derive_git_entries(repository_root, revision, prefixes)?,
    )
}

fn derive_git_entries(
    repository_root: &Path,
    revision: &str,
    prefixes: &[&str],
) -> Result<BTreeMap<String, String>, PromotionError> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(repository_root)
        .args(["ls-tree", "-r", "--name-only", revision, "--"])
        .args(prefixes);
    let output = run_process(&mut command, "enumerate producer-affecting Git inputs")?;
    let names = String::from_utf8(output.stdout).map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Git,
            format!("Git returned non-UTF-8 paths: {error}"),
        )
    })?;
    let mut entries = BTreeMap::new();
    for path in names.lines() {
        validate_relative_path(path)?;
        entries.insert(
            path.to_owned(),
            sha256(&git_file(repository_root, revision, path)?),
        );
    }
    Ok(entries)
}

fn closure_from_entries(
    label: &str,
    entries: BTreeMap<String, String>,
) -> Result<BundleClosure, PromotionError> {
    if entries.is_empty() {
        return Err(PromotionError::new(
            PromotionErrorKind::Closure,
            format!("{label} closure is empty"),
        ));
    }
    let entries = entries
        .into_iter()
        .map(|(path, sha256)| ClosureEntry { path, sha256 })
        .collect::<Vec<_>>();
    Ok(BundleClosure {
        schema_version: 1,
        label: label.to_owned(),
        digest: closure_digest(label, &entries),
        entries,
    })
}

fn validate_staged_tree(staging_root: &Path) -> Result<BTreeMap<String, String>, PromotionError> {
    let actual = collect_regular_files(staging_root)?;
    let expected = PROMOTED_PATHS
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    validate_exact_paths(&actual, &expected)?;
    actual
        .into_iter()
        .map(|path| {
            let digest = file_sha256(&staging_root.join(&path))?;
            Ok((path, digest))
        })
        .collect()
}

fn baseline_sha256(
    repository_root: &Path,
    revision: &str,
) -> Result<BTreeMap<String, String>, PromotionError> {
    PROMOTED_PATHS
        .into_iter()
        .map(|path| {
            git_file(repository_root, revision, path).map(|bytes| (path.to_owned(), sha256(&bytes)))
        })
        .collect()
}

fn replacement_sha256(replacements: &BTreeMap<String, Vec<u8>>) -> BTreeMap<String, String> {
    replacements
        .iter()
        .map(|(path, bytes)| (path.clone(), sha256(bytes)))
        .collect()
}

pub(crate) fn validate_exact_paths(
    actual: &BTreeSet<String>,
    expected: &BTreeSet<String>,
) -> Result<(), PromotionError> {
    if actual == expected {
        return Ok(());
    }
    Err(PromotionError::new(
        PromotionErrorKind::Path,
        "staging tree must contain exactly the seven promoted paths",
    ))
}

pub(crate) fn validate_staged_ledgers(
    staging_root: &Path,
    replacement_sha256: &BTreeMap<String, String>,
) -> Result<(), PromotionError> {
    let contents =
        fs::read_to_string(staging_root.join(ARTIFACT_MANIFEST_PATH)).map_err(filesystem_error)?;
    let manifest: toml::Value = toml::from_str(&contents).map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Ledger,
            format!("invalid staged artifact manifest: {error}"),
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
                "staged artifact records are absent",
            )
        })?;
    if records.len() != 4 {
        return Err(PromotionError::new(
            PromotionErrorKind::Ledger,
            "staged artifact records are incomplete",
        ));
    }
    for record in records {
        let path = record
            .get("path")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| {
                PromotionError::new(PromotionErrorKind::Ledger, "staged record path is absent")
            })?;
        let digest = record
            .get("sha256")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| {
                PromotionError::new(PromotionErrorKind::Ledger, "staged record digest is absent")
            })?;
        if replacement_sha256.get(path).map(String::as_str) != Some(digest) {
            return Err(PromotionError::new(
                PromotionErrorKind::Ledger,
                format!("staged artifact ledger hash for `{path}` is stale"),
            ));
        }
    }
    Ok(())
}

#[allow(
    dead_code,
    reason = "integration tests inject a bounded transaction failure"
)]
pub(crate) fn replace_with_injected_failure(
    repository_root: &Path,
    staging_root: &Path,
    paths: &[&str],
    maybe_fail_after: Option<usize>,
) -> Result<(), PromotionError> {
    transaction::replace_all(repository_root, staging_root, paths, maybe_fail_after)
}

#[allow(
    dead_code,
    reason = "integration tests inject a failing post-write validator"
)]
pub(crate) fn replace_with_failing_validation(
    repository_root: &Path,
    staging_root: &Path,
    paths: &[&str],
) -> Result<(), PromotionError> {
    transaction::replace_all_and_validate(repository_root, staging_root, paths, None, || {
        Err(PromotionError::new(
            PromotionErrorKind::Transaction,
            "injected post-write validation failure",
        ))
    })
}

fn write_replacements(
    staging_root: &Path,
    replacements: &BTreeMap<String, Vec<u8>>,
) -> Result<(), PromotionError> {
    let actual = replacements.keys().cloned().collect::<BTreeSet<_>>();
    let expected = PROMOTED_PATHS
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    validate_exact_paths(&actual, &expected)?;
    for (path, bytes) in replacements {
        write_new_file(&staging_root.join(path), bytes)?;
    }
    Ok(())
}

fn format_staged_catalog(staging_root: &Path) -> Result<(), PromotionError> {
    run_process(
        Command::new("rustfmt")
            .args(["--edition", "2024"])
            .arg(staging_root.join(CATALOG_PATH)),
        "format staged catalog binding",
    )
    .map(|_output| ())
}

fn canonical_diff(
    repository_root: &Path,
    base_revision: &str,
    staging_root: &Path,
    paths: &[&str],
) -> Result<String, PromotionError> {
    let scratch = repository_root
        .join("target/phase13/promotion-diff")
        .join(format!(
            "{}-{}",
            std::process::id(),
            NEXT_STAGE.fetch_add(1, Ordering::Relaxed)
        ));
    fs::create_dir_all(&scratch).map_err(filesystem_error)?;
    let mut complete = String::new();
    for path in paths {
        let maybe_old = git_maybe_file(repository_root, base_revision, path)?;
        let staged = relative_path_text(repository_root, &staging_root.join(path))?;
        let old = if let Some(ref bytes) = maybe_old {
            let old_path = scratch.join(path);
            write_new_file(&old_path, &bytes)?;
            relative_path_text(repository_root, &old_path)?
        } else {
            "/dev/null".to_owned()
        };
        let output = Command::new("git")
            .current_dir(repository_root)
            .args([
                "diff",
                "--no-index",
                "--no-ext-diff",
                "--text",
                "--src-prefix=a/",
                "--dst-prefix=b/",
                "--",
                &old,
                &staged,
            ])
            .output()
            .map_err(|error| {
                PromotionError::new(
                    PromotionErrorKind::Diff,
                    format!("failed to render review diff: {error}"),
                )
            })?;
        if output.status.code() != Some(1) {
            return Err(PromotionError::new(
                PromotionErrorKind::Diff,
                format!("promoted path `{path}` did not produce one changed-file diff"),
            ));
        }
        let text = String::from_utf8(output.stdout).map_err(|error| {
            PromotionError::new(
                PromotionErrorKind::Diff,
                format!("review diff is not UTF-8: {error}"),
            )
        })?;
        complete.push_str(&normalize_diff_headers(
            &text,
            &old,
            &staged,
            path,
            maybe_old.is_none(),
        ));
    }
    Ok(complete)
}

fn normalize_diff_headers(
    diff: &str,
    old: &str,
    staged: &str,
    promoted_path: &str,
    is_new: bool,
) -> String {
    let old_label = if is_new {
        "/dev/null".to_owned()
    } else {
        format!("a/{promoted_path}")
    };
    diff.lines()
        .map(|line| {
            if line.starts_with("diff --git ") {
                return format!("diff --git a/{promoted_path} b/{promoted_path}");
            }
            if line == format!("--- a/{old}") || line == "--- /dev/null" {
                return format!("--- {old_label}");
            }
            if line == format!("+++ b/{staged}") {
                return format!("+++ b/{promoted_path}");
            }
            line.to_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn acquire_provider_metadata() -> Result<Acquisition, PromotionError> {
    let endpoint = format!("repos/{PROVIDER_REPOSITORY}/actions/artifacts/{PROVIDER_ARTIFACT_ID}");
    let output = run_process(
        Command::new("gh").args(["api", &endpoint]),
        "read immutable artifact provider metadata",
    )?;
    let artifact: ProviderArtifact = serde_json::from_slice(&output.stdout).map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Provider,
            format!("invalid provider artifact metadata: {error}"),
        )
    })?;
    let acquisition = Acquisition {
        repository: PROVIDER_REPOSITORY.to_owned(),
        run_id: artifact.workflow_run.id,
        artifact_id: artifact.id,
        artifact_name: artifact.name,
        provider_digest: artifact.digest,
        artifact_created_at: artifact.created_at,
        artifact_expires_at: artifact.expires_at,
    };
    if artifact.workflow_run.head_sha != PRODUCER_SHA {
        return Err(PromotionError::new(
            PromotionErrorKind::Provider,
            "provider artifact head SHA does not equal P",
        ));
    }
    validate_acquisition(&acquisition)?;
    Ok(acquisition)
}

fn validate_acquisition(acquisition: &Acquisition) -> Result<(), PromotionError> {
    if acquisition.repository != PROVIDER_REPOSITORY
        || acquisition.run_id != PROVIDER_RUN_ID
        || acquisition.artifact_id != PROVIDER_ARTIFACT_ID
        || acquisition.artifact_name != PROVIDER_ARTIFACT_NAME
        || acquisition.provider_digest != PROVIDER_DIGEST
        || !valid_utc_timestamp(&acquisition.artifact_created_at)
        || !valid_utc_timestamp(&acquisition.artifact_expires_at)
    {
        return Err(PromotionError::new(
            PromotionErrorKind::Provider,
            "artifact provider metadata does not equal the canonical acquisition tuple",
        ));
    }
    Ok(())
}

fn read_bundle_manifest(root: &Path) -> Result<BundleManifest, PromotionError> {
    read_json(&root.join("phase13-bundle.json"))
}

fn closure_review(recorded: &BundleClosure, recomputed: &BundleClosure) -> ClosureReview {
    ClosureReview {
        recorded_sha256: recorded.digest.clone(),
        recomputed_sha256: recomputed.digest.clone(),
        byte_identical: recorded == recomputed,
    }
}

fn promoted_paths() -> Vec<String> {
    PROMOTED_PATHS
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>()
}

fn promoted_path_set_sha256() -> String {
    path_set_sha256("phase13-promoted-path-set-v2", &promoted_paths())
}

fn changed_path_set_sha256(paths: &[String]) -> String {
    path_set_sha256("phase13-changed-path-set-v2", paths)
}

fn path_set_sha256(domain: &str, paths: &[String]) -> String {
    let mut hasher = Sha256::new();
    update_field(&mut hasher, domain.as_bytes());
    for path in paths {
        update_field(&mut hasher, path.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

pub(crate) fn classify_reviewed_paths(
    baseline_sha256: &BTreeMap<String, String>,
    replacement_sha256: &BTreeMap<String, String>,
) -> Result<(Vec<String>, Vec<String>), PromotionError> {
    if baseline_sha256.keys().collect::<Vec<_>>() != replacement_sha256.keys().collect::<Vec<_>>() {
        return Err(PromotionError::new(
            PromotionErrorKind::Path,
            "baseline and replacement path sets differ",
        ));
    }
    let (changed, unchanged): (Vec<_>, Vec<_>) = replacement_sha256
        .iter()
        .partition(|(path, replacement)| baseline_sha256.get(*path) != Some(*replacement));
    let changed = changed
        .into_iter()
        .map(|(path, _digest)| path.clone())
        .collect();
    let unchanged = unchanged
        .into_iter()
        .map(|(path, _digest)| path.clone())
        .collect();
    Ok((changed, unchanged))
}

#[derive(Serialize)]
struct ReviewSubject<'a> {
    schema_version: u32,
    producer_sha: &'a str,
    bundle_sha256: &'a str,
    promotion_base_sha: &'a str,
    acquisition: &'a Acquisition,
    witness_closure: &'a ClosureReview,
    replay_closure: &'a ClosureReview,
    promoted_paths: &'a [String],
    promoted_path_set_sha256: &'a str,
    changed_paths: &'a [String],
    unchanged_paths: &'a [String],
    changed_path_set_sha256: &'a str,
    replacement_sha256: &'a BTreeMap<String, String>,
    diff: &'a str,
}

fn review_sha256(packet: &ReviewPacket) -> Result<String, PromotionError> {
    let subject = ReviewSubject {
        schema_version: 2,
        producer_sha: &packet.producer_sha,
        bundle_sha256: &packet.bundle_sha256,
        promotion_base_sha: &packet.promotion_base_sha,
        acquisition: &packet.acquisition,
        witness_closure: &packet.witness_closure,
        replay_closure: &packet.replay_closure,
        promoted_paths: &packet.promoted_paths,
        promoted_path_set_sha256: &packet.promoted_path_set_sha256,
        changed_paths: &packet.changed_paths,
        unchanged_paths: &packet.unchanged_paths,
        changed_path_set_sha256: &packet.changed_path_set_sha256,
        replacement_sha256: &packet.replacement_sha256,
        diff: &packet.diff,
    };
    serde_json::to_vec(&subject)
        .map(|bytes| sha256(&bytes))
        .map_err(|error| {
            PromotionError::new(
                PromotionErrorKind::Schema,
                format!("failed to encode review subject: {error}"),
            )
        })
}

#[allow(
    dead_code,
    reason = "integration contract tests hash a packet subject without running promotion"
)]
pub(crate) fn review_sha256_for_test(packet: &ReviewPacket) -> Result<String, PromotionError> {
    review_sha256(packet)
}

fn collect_regular_files(root: &Path) -> Result<BTreeSet<String>, PromotionError> {
    let mut pending = vec![root.to_path_buf()];
    let mut owned = BTreeSet::new();
    while let Some(directory) = pending.pop() {
        reject_symlink(&directory)?;
        for entry in fs::read_dir(&directory).map_err(filesystem_error)? {
            let entry = entry.map_err(filesystem_error)?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).map_err(filesystem_error)?;
            if metadata.file_type().is_symlink() {
                return Err(PromotionError::new(
                    PromotionErrorKind::Path,
                    "staging tree contains a symbolic link",
                ));
            }
            if metadata.is_dir() {
                pending.push(path);
            } else if metadata.is_file() {
                let relative = path.strip_prefix(root).map_err(|_error| {
                    PromotionError::new(
                        PromotionErrorKind::Path,
                        "staged file escaped the staging root",
                    )
                })?;
                owned.insert(path_text(relative)?);
            } else {
                return Err(PromotionError::new(
                    PromotionErrorKind::Path,
                    "staging tree contains a non-regular entry",
                ));
            }
        }
    }
    Ok(owned)
}

fn new_staging_root(
    repository_root: &Path,
    promotion_base_sha: &str,
) -> Result<PathBuf, PromotionError> {
    let parent = repository_root.join("target/phase13/promotion-staged");
    fs::create_dir_all(&parent).map_err(filesystem_error)?;
    reject_symlink(&parent)?;
    let ordinal = NEXT_STAGE.fetch_add(1, Ordering::Relaxed);
    let root = parent.join(format!(
        "{promotion_base_sha}-{}-{ordinal}",
        std::process::id()
    ));
    fs::create_dir(&root).map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Filesystem,
            format!("failed to create new promotion staging root: {error}"),
        )
    })?;
    Ok(root)
}

fn require_clean_worktree(repository_root: &Path) -> Result<(), PromotionError> {
    let status = git_text(
        repository_root,
        &["status", "--porcelain=v1", "--untracked-files=all"],
    )?;
    if status.is_empty() {
        return Ok(());
    }
    Err(PromotionError::new(
        PromotionErrorKind::Git,
        "promotion preparation requires a completely clean worktree",
    ))
}

fn parse_options(args: &[String]) -> Result<BTreeMap<String, String>, PromotionError> {
    if !args.len().is_multiple_of(2) {
        return Err(PromotionError::usage(
            "every option requires exactly one value",
        ));
    }
    let mut options = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        if !pair[0].starts_with("--") || options.insert(pair[0].clone(), pair[1].clone()).is_some()
        {
            return Err(PromotionError::usage(
                "options must be unique option/value pairs",
            ));
        }
    }
    Ok(options)
}

fn require_options(
    options: &BTreeMap<String, String>,
    required_names: &[&str],
) -> Result<(), PromotionError> {
    if options.len() == required_names.len()
        && required_names
            .iter()
            .all(|name| options.contains_key(*name))
    {
        return Ok(());
    }
    Err(PromotionError::usage(
        "command options do not match the closed contract",
    ))
}

fn required<'a>(
    options: &'a BTreeMap<String, String>,
    name: &str,
) -> Result<&'a str, PromotionError> {
    options
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| PromotionError::usage(format!("missing `{name}`")))
}

fn validate_reviewer_id(value: &str) -> Result<(), PromotionError> {
    let valid = !value.is_empty()
        && value.len() <= 80
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'@'))
        && !value.to_ascii_lowercase().contains("codex");
    if valid {
        return Ok(());
    }
    Err(PromotionError::new(
        PromotionErrorKind::Acknowledgement,
        "reviewer ID must identify an independent human",
    ))
}

fn validate_relative_path(value: &str) -> Result<(), PromotionError> {
    if !value.is_empty()
        && !value.contains('\\')
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Ok(());
    }
    Err(PromotionError::new(
        PromotionErrorKind::Path,
        format!("unsafe relative path `{value}`"),
    ))
}

fn repository_root() -> Result<PathBuf, PromotionError> {
    let current = env::current_dir().map_err(filesystem_error)?;
    current
        .ancestors()
        .find(|candidate| {
            candidate.join("Cargo.toml").is_file()
                && candidate.join("crates/liquidfun/Cargo.toml").is_file()
        })
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            PromotionError::new(
                PromotionErrorKind::Filesystem,
                "repository root is unavailable",
            )
        })
}

fn absolute_path(repository_root: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repository_root.join(path)
    }
}

fn relative_path_text(repository_root: &Path, path: &Path) -> Result<String, PromotionError> {
    path.strip_prefix(repository_root)
        .map_err(|_error| {
            PromotionError::new(
                PromotionErrorKind::Path,
                "generated path is outside the repository",
            )
        })
        .and_then(path_text)
}

fn git_text(repository_root: &Path, args: &[&str]) -> Result<String, PromotionError> {
    let output = run_process(
        Command::new("git")
            .arg("-C")
            .arg(repository_root)
            .args(args),
        "query Git identity",
    )?;
    String::from_utf8(output.stdout)
        .map(|text| text.trim().to_owned())
        .map_err(|error| {
            PromotionError::new(
                PromotionErrorKind::Git,
                format!("Git returned non-UTF-8 output: {error}"),
            )
        })
}

fn git_success(repository_root: &Path, args: &[&str]) -> Result<bool, PromotionError> {
    let status = Command::new("git")
        .arg("-C")
        .arg(repository_root)
        .args(args)
        .status()
        .map_err(|error| {
            PromotionError::new(
                PromotionErrorKind::Git,
                format!("failed to query Git ancestry: {error}"),
            )
        })?;
    Ok(status.success())
}

fn git_file(repository_root: &Path, revision: &str, path: &str) -> Result<Vec<u8>, PromotionError> {
    git_maybe_file(repository_root, revision, path)?.ok_or_else(|| {
        PromotionError::new(
            PromotionErrorKind::Git,
            format!("`{path}` is absent at revision `{revision}`"),
        )
    })
}

fn git_maybe_file(
    repository_root: &Path,
    revision: &str,
    path: &str,
) -> Result<Option<Vec<u8>>, PromotionError> {
    let object = format!("{revision}:{path}");
    let output = Command::new("git")
        .arg("-C")
        .arg(repository_root)
        .args(["show", &object])
        .output()
        .map_err(|error| {
            PromotionError::new(
                PromotionErrorKind::Git,
                format!("failed to read Git object `{object}`: {error}"),
            )
        })?;
    if output.status.success() {
        return Ok(Some(output.stdout));
    }
    if output.status.code() == Some(128) {
        return Ok(None);
    }
    Err(PromotionError::new(
        PromotionErrorKind::Git,
        format!(
            "failed to read Git object `{object}`: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ),
    ))
}

fn run_process(command: &mut Command, action: &str) -> Result<Output, PromotionError> {
    let output = command.output().map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Filesystem,
            format!("failed to {action}: {error}"),
        )
    })?;
    if !output.status.success() {
        return Err(PromotionError::new(
            PromotionErrorKind::Filesystem,
            format!(
                "failed to {action} with {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    Ok(output)
}

fn write_json(path: &Path, value: &impl Serialize, create_new: bool) -> Result<(), PromotionError> {
    let bytes = json_bytes(value)?;
    if create_new {
        return write_new_file(path, &bytes);
    }
    let parent = path.parent().ok_or_else(|| {
        PromotionError::new(PromotionErrorKind::Path, "JSON output has no parent")
    })?;
    fs::create_dir_all(parent).map_err(filesystem_error)?;
    fs::write(path, bytes).map_err(filesystem_error)
}

fn write_new_file(path: &Path, bytes: &[u8]) -> Result<(), PromotionError> {
    let parent = path.parent().ok_or_else(|| {
        PromotionError::new(PromotionErrorKind::Path, "output file has no parent")
    })?;
    fs::create_dir_all(parent).map_err(filesystem_error)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(filesystem_error)?;
    file.write_all(bytes).map_err(filesystem_error)
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, PromotionError> {
    let bytes = fs::read(path).map_err(filesystem_error)?;
    serde_json::from_slice(&bytes).map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Schema,
            format!("invalid JSON `{}`: {error}", path.display()),
        )
    })
}

fn json_bytes(value: &impl Serialize) -> Result<Vec<u8>, PromotionError> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| {
        PromotionError::new(
            PromotionErrorKind::Schema,
            format!("failed to encode JSON: {error}"),
        )
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn reject_symlink(path: &Path) -> Result<(), PromotionError> {
    let metadata = fs::symlink_metadata(path).map_err(filesystem_error)?;
    if metadata.file_type().is_symlink() {
        return Err(PromotionError::new(
            PromotionErrorKind::Path,
            format!("symbolic link is forbidden: {}", path.display()),
        ));
    }
    Ok(())
}

fn file_sha256(path: &Path) -> Result<String, PromotionError> {
    fs::read(path)
        .map(|bytes| sha256(&bytes))
        .map_err(filesystem_error)
}

fn path_text(path: &Path) -> Result<String, PromotionError> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| PromotionError::new(PromotionErrorKind::Path, "path is not valid UTF-8"))
}

fn filesystem_error(error: std::io::Error) -> PromotionError {
    PromotionError::new(PromotionErrorKind::Filesystem, error.to_string())
}

fn valid_revision(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(lower_hex)
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(lower_hex)
}

fn lower_hex(byte: u8) -> bool {
    byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
}

fn valid_utc_timestamp(value: &str) -> bool {
    value.len() == 20
        && value.as_bytes().get(4) == Some(&b'-')
        && value.as_bytes().get(7) == Some(&b'-')
        && value.as_bytes().get(10) == Some(&b'T')
        && value.as_bytes().get(13) == Some(&b':')
        && value.as_bytes().get(16) == Some(&b':')
        && value.ends_with('Z')
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn update_field(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update(u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_be_bytes());
    hasher.update(bytes);
}
