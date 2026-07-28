//! Final promotion-descendant exact-head Phase 13 acceptance.

#[path = "phase13_acceptance/closure.rs"]
mod closure;
#[path = "phase13_acceptance/execution.rs"]
mod execution;
#[path = "phase13_acceptance/identity.rs"]
mod identity;

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt::{self, Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use execution::{assert_head, command_evidence, effect_steps, git_text, run_command};
use identity::load_identity;

#[allow(
    dead_code,
    reason = "integration contract tests inspect the locked command evidence"
)]
pub(crate) fn required_command_evidence() -> Vec<(AcceptanceStep, String)> {
    execution::required_command_evidence()
}

const USAGE: &str = "Usage: cargo xtask phase13 acceptance";
const RECEIPT_PATH: &str = "reference/artifacts/phase13/promotion-receipt.json";
const ARTIFACT_MANIFEST_PATH: &str = "reference/artifacts/manifest.toml";
const SOURCE_MAP_PATH: &str = "reference/source-map.toml";
const WITNESS_PROVENANCE_PATH: &str =
    "reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json";
const REPLAY_EVIDENCE_PATH: &str =
    "reference/artifacts/catalog/rigid-stack-v1.replay-evidence.json";
const MATERIALS_MANIFEST: &str = "tools/reference/phase9-lifecycle-contact-witness.materials.json";
const IDENTITY_PATH: &str = "target/phase13-acceptance/identity.json";
const ORACLE_REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const EXACT_BYTES_DIGEST_MODE: &str = "exact_bytes_sha256";
const RECEIPT_SEMANTIC_DIGEST_MODE: &str = "phase13_receipt_semantic_v2";

const PROMOTED_PATHS: [&str; 7] = [
    "crates/liquidfun-differential/src/fixtures/replay/catalog.rs",
    REPLAY_EVIDENCE_PATH,
    RECEIPT_PATH,
    ARTIFACT_MANIFEST_PATH,
    "reference/artifacts/phase9/lifecycle-contact-witnesses.json",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcceptanceErrorKind {
    Closure,
    Filesystem,
    Head,
    Identity,
    Ledger,
    Ordering,
    Process,
    Publication,
    Schema,
    Step,
    Usage,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct AcceptanceError {
    kind: AcceptanceErrorKind,
    message: String,
}

impl AcceptanceError {
    fn new(kind: AcceptanceErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    fn usage(message: impl Into<String>) -> Self {
        Self::new(
            AcceptanceErrorKind::Usage,
            format!("{}\n\n{USAGE}", message.into()),
        )
    }

    #[allow(
        dead_code,
        reason = "integration contract tests inspect stable categories"
    )]
    pub(crate) const fn kind(&self) -> AcceptanceErrorKind {
        self.kind
    }
}

impl Display for AcceptanceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "phase13 acceptance/{:?}: {}",
            self.kind, self.message
        )
    }
}

impl std::error::Error for AcceptanceError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AcceptanceStep {
    Identity,
    Provenance,
    ReviewedReplay,
    Diagnosis,
    Regression,
    OracleBuild,
    LiveReplay,
}

impl AcceptanceStep {
    pub(crate) const ORDERED: [Self; 7] = [
        Self::Identity,
        Self::Provenance,
        Self::ReviewedReplay,
        Self::Diagnosis,
        Self::Regression,
        Self::OracleBuild,
        Self::LiveReplay,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct StepCompletion {
    pub(crate) step: AcceptanceStep,
    pub(crate) command: String,
    pub(crate) succeeded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "the integration-test contract exposes each independently falsifiable acceptance proof"
)]
pub(crate) struct IdentityContract {
    pub(crate) producer_sha: String,
    pub(crate) bundle_sha256: String,
    pub(crate) promotion_base_sha: String,
    pub(crate) promotion_sha: String,
    pub(crate) acceptance_sha: String,
    pub(crate) producer_is_ancestor_of_base: bool,
    pub(crate) witness_closure_at_r: String,
    pub(crate) replay_closure_at_r: String,
    pub(crate) witness_closure_at_a: String,
    pub(crate) replay_closure_at_a: String,
    pub(crate) expected_witness_closure: String,
    pub(crate) expected_replay_closure: String,
    pub(crate) promotion_first_parent: String,
    pub(crate) required_trailers: BTreeMap<String, String>,
    pub(crate) actual_trailers: BTreeMap<String, String>,
    pub(crate) expected_promoted_path_set_sha256: String,
    pub(crate) actual_promoted_path_set_sha256: String,
    pub(crate) expected_promoted_content_sha256: String,
    pub(crate) actual_promoted_content_sha256: String,
    pub(crate) expected_changed_path_set_sha256: String,
    pub(crate) actual_changed_path_set_sha256: String,
    pub(crate) expected_changed_content_sha256: String,
    pub(crate) actual_changed_content_sha256: String,
    pub(crate) changed_paths_match: bool,
    pub(crate) unchanged_paths_equal_base: bool,
    pub(crate) all_promoted_paths_equal_at_acceptance: bool,
    pub(crate) promotion_is_ancestor_of_acceptance: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HeadSnapshot {
    pub(crate) expected_sha: String,
    pub(crate) observed_sha: String,
    pub(crate) clean: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct AcceptanceState {
    ordered_steps: Vec<StepCompletion>,
    failed: bool,
}

impl AcceptanceState {
    pub(crate) fn new() -> Self {
        Self {
            ordered_steps: Vec::with_capacity(AcceptanceStep::ORDERED.len()),
            failed: false,
        }
    }

    pub(crate) fn record(&mut self, completion: StepCompletion) -> Result<(), AcceptanceError> {
        if self.failed {
            return Err(AcceptanceError::new(
                AcceptanceErrorKind::Step,
                "acceptance already failed; later work is forbidden",
            ));
        }
        let maybe_expected = AcceptanceStep::ORDERED.get(self.ordered_steps.len());
        if maybe_expected != Some(&completion.step) {
            self.failed = true;
            return Err(AcceptanceError::new(
                AcceptanceErrorKind::Ordering,
                "acceptance step was missing, duplicated, or reordered",
            ));
        }
        if !completion.succeeded {
            self.failed = true;
            return Err(AcceptanceError::new(
                AcceptanceErrorKind::Step,
                format!("{:?} did not complete successfully", completion.step),
            ));
        }
        self.ordered_steps.push(completion);
        Ok(())
    }

    pub(crate) fn publish(
        self,
        contract: IdentityContract,
    ) -> Result<TerminalIdentity, AcceptanceError> {
        if self.failed || self.ordered_steps.len() != AcceptanceStep::ORDERED.len() {
            return Err(AcceptanceError::new(
                AcceptanceErrorKind::Publication,
                "terminal identity requires every ordered step to succeed",
            ));
        }
        validate_identity_contract(&contract)?;
        Ok(TerminalIdentity {
            schema_version: 2,
            producer_sha: contract.producer_sha,
            bundle_sha256: contract.bundle_sha256,
            promotion_base_sha: contract.promotion_base_sha,
            promotion_sha: contract.promotion_sha,
            acceptance_sha: contract.acceptance_sha,
            promoted_path_set_sha256: contract.expected_promoted_path_set_sha256,
            promoted_content_sha256: contract.expected_promoted_content_sha256,
            changed_path_set_sha256: contract.expected_changed_path_set_sha256,
            changed_content_sha256: contract.expected_changed_content_sha256,
            upstream_revision: String::new(),
            oracle_build_identity_sha256: String::new(),
            reviewed_evidence_sha256: BTreeMap::new(),
            ordered_steps: self.ordered_steps,
            invocation: "cargo xtask phase13 acceptance".to_owned(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct TerminalIdentity {
    schema_version: u32,
    pub(crate) producer_sha: String,
    pub(crate) bundle_sha256: String,
    pub(crate) promotion_base_sha: String,
    pub(crate) promotion_sha: String,
    pub(crate) acceptance_sha: String,
    promoted_path_set_sha256: String,
    promoted_content_sha256: String,
    changed_path_set_sha256: String,
    changed_content_sha256: String,
    upstream_revision: String,
    oracle_build_identity_sha256: String,
    reviewed_evidence_sha256: BTreeMap<String, String>,
    pub(crate) ordered_steps: Vec<StepCompletion>,
    invocation: String,
}

pub(crate) fn validate_head_snapshot(snapshot: &HeadSnapshot) -> Result<(), AcceptanceError> {
    if !valid_revision(&snapshot.expected_sha)
        || snapshot.observed_sha != snapshot.expected_sha
        || !snapshot.clean
    {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Head,
            "acceptance A changed or the source worktree became dirty",
        ));
    }
    Ok(())
}

pub(crate) fn validate_identity_contract(
    contract: &IdentityContract,
) -> Result<(), AcceptanceError> {
    let revisions = [
        &contract.producer_sha,
        &contract.promotion_base_sha,
        &contract.promotion_sha,
        &contract.acceptance_sha,
        &contract.promotion_first_parent,
    ];
    let digests = [
        &contract.bundle_sha256,
        &contract.witness_closure_at_r,
        &contract.replay_closure_at_r,
        &contract.witness_closure_at_a,
        &contract.replay_closure_at_a,
        &contract.expected_witness_closure,
        &contract.expected_replay_closure,
        &contract.expected_promoted_path_set_sha256,
        &contract.actual_promoted_path_set_sha256,
        &contract.expected_promoted_content_sha256,
        &contract.actual_promoted_content_sha256,
        &contract.expected_changed_path_set_sha256,
        &contract.actual_changed_path_set_sha256,
        &contract.expected_changed_content_sha256,
        &contract.actual_changed_content_sha256,
    ];
    if !revisions.into_iter().all(|value| valid_revision(value))
        || !digests.into_iter().all(|value| valid_digest(value))
        || !contract.producer_is_ancestor_of_base
        || contract.promotion_first_parent != contract.promotion_base_sha
        || contract.actual_trailers != contract.required_trailers
        || contract.actual_promoted_path_set_sha256 != contract.expected_promoted_path_set_sha256
        || contract.actual_promoted_content_sha256 != contract.expected_promoted_content_sha256
        || contract.actual_changed_path_set_sha256 != contract.expected_changed_path_set_sha256
        || contract.actual_changed_content_sha256 != contract.expected_changed_content_sha256
        || !contract.changed_paths_match
        || !contract.unchanged_paths_equal_base
        || !contract.all_promoted_paths_equal_at_acceptance
        || !contract.promotion_is_ancestor_of_acceptance
    {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Identity,
            "P/B/R/Q/A ancestry, trailers, or promoted path-set identity is invalid",
        ));
    }
    if contract.witness_closure_at_r != contract.expected_witness_closure
        || contract.replay_closure_at_r != contract.expected_replay_closure
        || contract.witness_closure_at_a != contract.expected_witness_closure
        || contract.replay_closure_at_a != contract.expected_replay_closure
    {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Closure,
            format!(
                "producer-affecting closure drift: witness(R={}, A={}, P={}), replay(R={}, A={}, P={})",
                contract.witness_closure_at_r,
                contract.witness_closure_at_a,
                contract.expected_witness_closure,
                contract.replay_closure_at_r,
                contract.replay_closure_at_a,
                contract.expected_replay_closure
            ),
        ));
    }
    Ok(())
}

pub(crate) fn run(args: &[String]) -> Result<(), AcceptanceError> {
    if args != ["acceptance"] {
        return Err(AcceptanceError::usage(
            "phase13 acceptance accepts no additional arguments",
        ));
    }
    let repository_root = repository_root()?;
    let acceptance_sha = git_text(&repository_root, &["rev-parse", "HEAD"])?;
    assert_head(&repository_root, &acceptance_sha)?;

    let loaded = load_identity(&repository_root, &acceptance_sha)?;
    validate_identity_contract(&loaded.contract)?;
    let mut state = AcceptanceState::new();
    state.record(StepCompletion {
        step: AcceptanceStep::Identity,
        command: "internal P/B/R/Q/A and closure validation".to_owned(),
        succeeded: true,
    })?;

    for (step, commands) in effect_steps() {
        assert_head(&repository_root, &acceptance_sha)?;
        for command in &commands {
            run_command(&repository_root, command)?;
            assert_head(&repository_root, &acceptance_sha)?;
        }
        state.record(StepCompletion {
            step,
            command: command_evidence(&commands),
            succeeded: true,
        })?;
    }
    assert_head(&repository_root, &acceptance_sha)?;

    let mut identity = state.publish(loaded.contract)?;
    identity.upstream_revision = loaded.upstream_revision;
    identity.oracle_build_identity_sha256 = loaded.oracle_build_identity_sha256;
    identity.reviewed_evidence_sha256 = loaded.reviewed_evidence_sha256;
    write_identity(&repository_root, &identity)?;
    println!(
        "phase13 acceptance passed: P={} B={} R={} Q={} A={}",
        identity.producer_sha,
        identity.bundle_sha256,
        identity.promotion_base_sha,
        identity.promotion_sha,
        identity.acceptance_sha
    );
    Ok(())
}

#[allow(
    dead_code,
    reason = "integration contract tests exercise repository history without running effects"
)]
pub(crate) fn validate_repository_identity_at(
    repository_root: &Path,
    acceptance_sha: &str,
) -> Result<(), AcceptanceError> {
    let loaded = load_identity(repository_root, acceptance_sha)?;
    validate_identity_contract(&loaded.contract)
}

fn repository_root() -> Result<PathBuf, AcceptanceError> {
    let current = env::current_dir().map_err(AcceptanceError::from)?;
    current
        .ancestors()
        .find(|candidate| {
            candidate.join("Cargo.toml").is_file()
                && candidate.join("crates/liquidfun/Cargo.toml").is_file()
        })
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            AcceptanceError::new(
                AcceptanceErrorKind::Filesystem,
                "repository root is unavailable",
            )
        })
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, AcceptanceError> {
    serde_json::from_slice(&fs::read(path).map_err(AcceptanceError::from)?).map_err(|error| {
        AcceptanceError::new(
            AcceptanceErrorKind::Schema,
            format!("invalid {}: {error}", path.display()),
        )
    })
}

fn write_identity(
    repository_root: &Path,
    identity: &TerminalIdentity,
) -> Result<(), AcceptanceError> {
    let path = repository_root.join(IDENTITY_PATH);
    fs::create_dir_all(path.parent().ok_or_else(|| {
        AcceptanceError::new(
            AcceptanceErrorKind::Filesystem,
            "identity path has no parent",
        )
    })?)
    .map_err(AcceptanceError::from)?;
    let mut bytes = serde_json::to_vec_pretty(identity).map_err(|error| {
        AcceptanceError::new(
            AcceptanceErrorKind::Schema,
            format!("failed to encode terminal identity: {error}"),
        )
    })?;
    bytes.push(b'\n');
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(AcceptanceError::from)?;
    file.write_all(&bytes).map_err(AcceptanceError::from)?;
    file.sync_all().map_err(AcceptanceError::from)
}

fn file_sha256(path: &Path) -> Result<String, AcceptanceError> {
    fs::read(path)
        .map(|bytes| sha256(&bytes))
        .map_err(AcceptanceError::from)
}

fn validate_relative_path(value: &str) -> Result<(), AcceptanceError> {
    if !value.is_empty()
        && !value.contains('\\')
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Ok(());
    }
    Err(AcceptanceError::new(
        AcceptanceErrorKind::Filesystem,
        format!("unsafe repository path `{value}`"),
    ))
}

fn update_field(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn valid_revision(value: &str) -> bool {
    value.len() == 40 && lower_hex(value)
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64 && lower_hex(value)
}

fn lower_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

impl From<std::io::Error> for AcceptanceError {
    fn from(error: std::io::Error) -> Self {
        Self::new(AcceptanceErrorKind::Filesystem, error.to_string())
    }
}
