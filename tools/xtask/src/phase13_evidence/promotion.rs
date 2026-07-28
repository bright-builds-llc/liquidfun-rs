//! Independent review and transactional promotion for canonical Phase 13 evidence.

#[path = "promotion/operations.rs"]
mod operations;
#[path = "promotion/preparation.rs"]
mod preparation;
#[path = "promotion/rendering.rs"]
mod rendering;
#[path = "promotion/review.rs"]
mod review;
#[path = "promotion/support.rs"]
mod support;
#[path = "promotion/transaction.rs"]
mod transaction;
#[path = "promotion/validation.rs"]
mod validation;

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
pub(crate) use operations::classify_reviewed_paths;
use operations::{
    acquire_provider_metadata, canonical_diff, changed_path_set_sha256, closure_review,
    format_staged_catalog, promoted_path_set_sha256, promoted_paths, read_bundle_manifest,
    validate_acquisition, write_replacements,
};
#[cfg(test)]
#[allow(
    unused_imports,
    reason = "path-included integration contracts consume the stable test surface"
)]
pub(crate) use operations::{replace_with_failing_validation, replace_with_injected_failure};
use preparation::prepare;
use rendering::{ReceiptFields, render_receipt, render_replacements};
pub(crate) use review::validate_base_contract;
use review::{promote, promotion_ready, review_ack_check, tracked_reviewed_check};
#[cfg(test)]
#[allow(
    unused_imports,
    reason = "path-included integration contracts consume the stable test surface"
)]
pub(crate) use review::{review_packet_for_test, validate_review_ack};
#[cfg(test)]
#[allow(
    unused_imports,
    reason = "path-included integration contracts consume the stable test surface"
)]
pub(crate) use support::review_sha256_for_test;
use support::{
    absolute_path, collect_regular_files, file_sha256, filesystem_error, git_file, git_maybe_file,
    git_success, git_text, json_bytes, new_staging_root, parse_options, read_json,
    relative_path_text, repository_root, require_clean_worktree, require_options, required,
    review_sha256, run_process, sha256, update_field, valid_digest, valid_revision,
    valid_utc_timestamp, validate_relative_path, validate_reviewer_id, write_json, write_new_file,
};
use validation::{
    baseline_sha256, derive_git_closure, derive_witness_closure, receipt_semantic_sha256,
    replacement_sha256, reviewed_content_digests, reviewed_content_digests_from_root,
    reviewed_replacements_from_root, validate_bundle_contract, validate_bundle_files,
    validate_content_digest_claims, validate_staged_tree,
};
#[cfg(test)]
#[allow(
    unused_imports,
    reason = "path-included integration contracts consume the stable test surface"
)]
pub(crate) use validation::{
    promotion_receipt_for_test, reviewed_content_digests_for_test,
    validate_content_digest_claims_for_test,
};
pub(crate) use validation::{validate_exact_paths, validate_staged_ledgers};

const USAGE: &str = r"Usage:
  cargo xtask phase13 evidence prepare --bundle <path> --expected-producer-sha <P> --expected-bundle-sha256 <B> --reviewer-id <id> --review-packet <path>
  cargo xtask phase13 evidence review-ack check --review-packet <path> --ack <path>
  cargo xtask phase13 evidence promote --review-packet <path> --review-ack <path>
  cargo xtask phase13 evidence promotion-ready --review-packet <path> --review-ack <path>
  cargo xtask phase13 evidence check --tracked --require-reviewed";

const PRODUCER_SHA: &str = "6e8261a66a67a05bf3fadb4ad9d818121c395324";
const BUNDLE_SHA256: &str = "fd7fa1a857c0b8cab3ee02fc1d61a45290b632173a4a1f80a790d4334c7453b2";
const UPSTREAM_REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const PROVIDER_REPOSITORY: &str = "bright-builds-llc/liquidfun-rs";
const PROVIDER_RUN_ID: u64 = 30_232_297_731;
const PROVIDER_ARTIFACT_ID: u64 = 8_640_500_578;
const PROVIDER_ARTIFACT_NAME: &str =
    "phase13-staged-30232297731-6e8261a66a67a05bf3fadb4ad9d818121c395324";
const PROVIDER_DIGEST: &str =
    "sha256:040d7f02c32c40ef6b208f3daf63fb1d458c0cb8cc78cc3d8ccd13e21488e0a7";
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
const EXACT_BYTES_DIGEST_MODE: &str = "exact_bytes_sha256";
const RECEIPT_SEMANTIC_DIGEST_MODE: &str = "phase13_receipt_semantic_v2";
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
    pub(crate) promoted_content_sha256: String,
    pub(crate) changed_paths: Vec<String>,
    pub(crate) unchanged_paths: Vec<String>,
    pub(crate) changed_path_set_sha256: String,
    pub(crate) changed_content_sha256: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PromotionReceipt {
    schema_version: u32,
    producer_sha: String,
    bundle_sha256: String,
    promotion_base_sha: String,
    acquisition: Acquisition,
    independent_reviewer_id: String,
    promoted_paths: Vec<String>,
    promoted_path_set_sha256: String,
    promoted_content_sha256: String,
    changed_paths: Vec<String>,
    unchanged_paths: Vec<String>,
    changed_path_set_sha256: String,
    changed_content_sha256: String,
    producer_closures: ProducerClosures,
    q_contract: PromotionCommitContract,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProducerClosures {
    witness_sha256: String,
    replay_sha256: String,
    recomputed_at_r: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
