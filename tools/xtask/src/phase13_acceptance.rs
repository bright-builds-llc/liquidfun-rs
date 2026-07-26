//! Final promotion-descendant exact-head Phase 13 acceptance.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt::{self, Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
    Comparison,
}

impl AcceptanceStep {
    pub(crate) const ORDERED: [Self; 7] = [
        Self::Identity,
        Self::Provenance,
        Self::ReviewedReplay,
        Self::Diagnosis,
        Self::Regression,
        Self::OracleBuild,
        Self::Comparison,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct StepCompletion {
    pub(crate) step: AcceptanceStep,
    pub(crate) command: String,
    pub(crate) succeeded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
            schema_version: 1,
            producer_sha: contract.producer_sha,
            bundle_sha256: contract.bundle_sha256,
            promotion_base_sha: contract.promotion_base_sha,
            promotion_sha: contract.promotion_sha,
            acceptance_sha: contract.acceptance_sha,
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
    ];
    if !revisions.into_iter().all(|value| valid_revision(value))
        || !digests.into_iter().all(|value| valid_digest(value))
        || !contract.producer_is_ancestor_of_base
        || contract.promotion_first_parent != contract.promotion_base_sha
        || contract.actual_trailers != contract.required_trailers
        || contract.actual_promoted_path_set_sha256 != contract.expected_promoted_path_set_sha256
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
            command: commands
                .iter()
                .map(CommandSpec::display)
                .collect::<Vec<_>>()
                .join(" && "),
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

#[derive(Debug)]
struct LoadedIdentity {
    contract: IdentityContract,
    upstream_revision: String,
    oracle_build_identity_sha256: String,
    reviewed_evidence_sha256: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema_version: u32,
    producer_sha: String,
    bundle_sha256: String,
    promotion_base_sha: String,
    acquisition: serde_json::Value,
    independent_reviewer_id: String,
    promoted_paths: Vec<String>,
    promoted_path_set_sha256: String,
    producer_closures: ProducerClosures,
    q_contract: PromotionContract,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProducerClosures {
    witness_sha256: String,
    replay_sha256: String,
    recomputed_at_r: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PromotionContract {
    required_first_parent: String,
    required_trailers: BTreeMap<String, String>,
    q_sha_recorded: bool,
    acceptance_sha_recorded: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MaterialsManifest {
    schema_version: u32,
    target: String,
    preset: String,
    materials: Vec<Material>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Material {
    kind: String,
    identity: String,
}

#[derive(Debug, Deserialize)]
struct WitnessProvenance {
    repository_revision: String,
    oracle_revision: String,
    compiler_id: String,
    compiler_version: String,
    target: String,
    cmake_preset: String,
}

#[derive(Debug, Deserialize)]
struct ReplayEvidence {
    upstream_revision: String,
    d1_oracle_identity_sha256: String,
}

fn load_identity(
    repository_root: &Path,
    acceptance_sha: &str,
) -> Result<LoadedIdentity, AcceptanceError> {
    let receipt: Receipt = read_json(&repository_root.join(RECEIPT_PATH))?;
    validate_receipt(&receipt)?;
    let reviewed_evidence_sha256 = validate_ledgers(repository_root, &receipt)?;
    let witness: WitnessProvenance = read_json(&repository_root.join(WITNESS_PROVENANCE_PATH))?;
    let replay: ReplayEvidence = read_json(&repository_root.join(REPLAY_EVIDENCE_PATH))?;
    if witness.repository_revision != receipt.producer_sha
        || witness.oracle_revision != ORACLE_REVISION
        || replay.upstream_revision != ORACLE_REVISION
        || !valid_digest(&replay.d1_oracle_identity_sha256)
        || witness.compiler_id != "Clang"
        || witness.compiler_version != "22.1.8"
        || witness.target != "x86_64-unknown-linux-gnu"
        || witness.cmake_preset != "oracle-debug"
    {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Identity,
            "reviewed evidence disagrees with receipt P or the pinned oracle",
        ));
    }

    let producer_is_ancestor_of_base = is_ancestor(
        repository_root,
        &receipt.producer_sha,
        &receipt.promotion_base_sha,
    )?;
    let witness_closure_at_r =
        derive_witness_closure(repository_root, &receipt.promotion_base_sha)?;
    let replay_closure_at_r = derive_replay_closure(
        repository_root,
        &receipt.promotion_base_sha,
        &receipt.promotion_base_sha,
    )?;
    let witness_closure_at_a = derive_witness_closure(repository_root, acceptance_sha)?;
    let replay_closure_at_a =
        derive_replay_closure(repository_root, acceptance_sha, &receipt.promotion_base_sha)?;
    let promotion_sha = discover_promotion(repository_root, acceptance_sha, &receipt)?;
    let promotion_first_parent = first_parent(repository_root, &promotion_sha)?;
    let actual_trailers = commit_trailers(repository_root, &promotion_sha)?;
    let actual_paths = changed_paths(repository_root, &promotion_sha)?;
    validate_q_paths_and_tree(
        repository_root,
        &promotion_sha,
        acceptance_sha,
        &actual_paths,
    )?;
    let actual_promoted_path_set_sha256 = promoted_path_set_sha256(&actual_paths)?;

    Ok(LoadedIdentity {
        contract: IdentityContract {
            producer_sha: receipt.producer_sha,
            bundle_sha256: receipt.bundle_sha256,
            promotion_base_sha: receipt.promotion_base_sha,
            promotion_sha: promotion_sha.clone(),
            acceptance_sha: acceptance_sha.to_owned(),
            producer_is_ancestor_of_base,
            witness_closure_at_r,
            replay_closure_at_r,
            witness_closure_at_a,
            replay_closure_at_a,
            expected_witness_closure: receipt.producer_closures.witness_sha256,
            expected_replay_closure: receipt.producer_closures.replay_sha256,
            promotion_first_parent,
            required_trailers: receipt.q_contract.required_trailers,
            actual_trailers,
            expected_promoted_path_set_sha256: receipt.promoted_path_set_sha256,
            actual_promoted_path_set_sha256,
            promotion_is_ancestor_of_acceptance: is_ancestor(
                repository_root,
                &promotion_sha,
                acceptance_sha,
            )?,
        },
        upstream_revision: replay.upstream_revision,
        oracle_build_identity_sha256: replay.d1_oracle_identity_sha256,
        reviewed_evidence_sha256,
    })
}

fn validate_receipt(receipt: &Receipt) -> Result<(), AcceptanceError> {
    let expected_paths = PROMOTED_PATHS.map(str::to_owned).to_vec();
    if receipt.schema_version != 1
        || !valid_revision(&receipt.producer_sha)
        || !valid_digest(&receipt.bundle_sha256)
        || !valid_revision(&receipt.promotion_base_sha)
        || receipt.promoted_paths != expected_paths
        || receipt.promoted_path_set_sha256 != promoted_path_set_sha256(&receipt.promoted_paths)?
        || !receipt.producer_closures.recomputed_at_r
        || receipt.q_contract.required_first_parent != receipt.promotion_base_sha
        || receipt.q_contract.q_sha_recorded
        || receipt.q_contract.acceptance_sha_recorded
        || receipt.acquisition.is_null()
        || receipt.independent_reviewer_id.trim().is_empty()
    {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Schema,
            "tracked receipt is incomplete, circular, or has the wrong promoted set",
        ));
    }
    Ok(())
}

fn validate_ledgers(
    repository_root: &Path,
    receipt: &Receipt,
) -> Result<BTreeMap<String, String>, AcceptanceError> {
    let manifest: toml::Value = toml::from_str(
        &fs::read_to_string(repository_root.join(ARTIFACT_MANIFEST_PATH)).map_err(filesystem)?,
    )
    .map_err(|error| {
        AcceptanceError::new(
            AcceptanceErrorKind::Ledger,
            format!("invalid artifact manifest: {error}"),
        )
    })?;
    let records = manifest
        .get("artifact_schemas")
        .and_then(|value| value.get("phase13_evidence"))
        .and_then(|value| value.get("records"))
        .and_then(toml::Value::as_array)
        .ok_or_else(|| {
            AcceptanceError::new(
                AcceptanceErrorKind::Ledger,
                "Phase 13 artifact records are absent",
            )
        })?;
    let mut digests = BTreeMap::new();
    for record in records {
        let path = toml_string(record, "path")?;
        let digest = toml_string(record, "sha256")?;
        let producer = toml_string(record, "producer_sha")?;
        let bundle = toml_string(record, "bundle_sha256")?;
        if producer != receipt.producer_sha
            || bundle != receipt.bundle_sha256
            || file_sha256(&repository_root.join(path))? != digest
            || digests.insert(path.to_owned(), digest.to_owned()).is_some()
        {
            return Err(AcceptanceError::new(
                AcceptanceErrorKind::Ledger,
                "artifact ledger disagrees with P/B or reviewed bytes",
            ));
        }
    }
    let source_map: toml::Value = toml::from_str(
        &fs::read_to_string(repository_root.join(SOURCE_MAP_PATH)).map_err(filesystem)?,
    )
    .map_err(|error| {
        AcceptanceError::new(
            AcceptanceErrorKind::Ledger,
            format!("invalid source map: {error}"),
        )
    })?;
    let mapped = source_map
        .get("mapping")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| AcceptanceError::new(AcceptanceErrorKind::Ledger, "source map is empty"))?
        .iter()
        .filter_map(|entry| entry.get("local_path").and_then(toml::Value::as_str))
        .collect::<BTreeSet<_>>();
    if digests.len() != 4 || !digests.keys().all(|path| mapped.contains(path.as_str())) {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Ledger,
            "reviewed evidence is incomplete in the artifact or FND-04 source ledger",
        ));
    }
    Ok(digests)
}

fn toml_string<'a>(value: &'a toml::Value, key: &str) -> Result<&'a str, AcceptanceError> {
    value.get(key).and_then(toml::Value::as_str).ok_or_else(|| {
        AcceptanceError::new(
            AcceptanceErrorKind::Ledger,
            format!("artifact record omitted `{key}`"),
        )
    })
}

fn discover_promotion(
    repository_root: &Path,
    acceptance_sha: &str,
    receipt: &Receipt,
) -> Result<String, AcceptanceError> {
    let revisions = git_text(repository_root, &["rev-list", acceptance_sha])?;
    let mut matches = Vec::new();
    for revision in revisions.lines() {
        if commit_trailers(repository_root, revision)? == receipt.q_contract.required_trailers {
            matches.push(revision.to_owned());
        }
    }
    if matches.len() != 1 {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Identity,
            "A history must contain exactly one Q with the required P/B/R trailers",
        ));
    }
    Ok(matches.remove(0))
}

fn commit_trailers(
    repository_root: &Path,
    revision: &str,
) -> Result<BTreeMap<String, String>, AcceptanceError> {
    let body = git_text(repository_root, &["show", "-s", "--format=%B", revision])?;
    let required_names = [
        "Phase13-Bundle-SHA256",
        "Phase13-Producer-SHA",
        "Phase13-Promotion-Base-SHA",
    ];
    let mut trailers = BTreeMap::new();
    for line in body.lines() {
        let Some((name, value)) = line.split_once(": ") else {
            continue;
        };
        if required_names.contains(&name)
            && trailers.insert(name.to_owned(), value.to_owned()).is_some()
        {
            return Err(AcceptanceError::new(
                AcceptanceErrorKind::Identity,
                "Q contains a duplicate Phase 13 trailer",
            ));
        }
    }
    Ok(trailers)
}

fn first_parent(repository_root: &Path, revision: &str) -> Result<String, AcceptanceError> {
    let parents = git_text(repository_root, &["show", "-s", "--format=%P", revision])?;
    let values = parents.split_whitespace().collect::<Vec<_>>();
    if values.len() != 1 {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Identity,
            "Q must have exactly one parent",
        ));
    }
    Ok(values[0].to_owned())
}

fn changed_paths(repository_root: &Path, revision: &str) -> Result<Vec<String>, AcceptanceError> {
    let output = git_text(
        repository_root,
        &["diff-tree", "--no-commit-id", "--name-only", "-r", revision],
    )?;
    Ok(output.lines().map(str::to_owned).collect())
}

fn validate_q_paths_and_tree(
    repository_root: &Path,
    promotion_sha: &str,
    acceptance_sha: &str,
    actual_paths: &[String],
) -> Result<(), AcceptanceError> {
    let actual = actual_paths
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let expected = PROMOTED_PATHS.into_iter().collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Identity,
            "Q does not change exactly the promoted seven-path tree",
        ));
    }
    let output = run_process(
        Command::new("git")
            .arg("-C")
            .arg(repository_root)
            .args(["diff", "--quiet", promotion_sha, acceptance_sha, "--"])
            .args(PROMOTED_PATHS),
        "compare Q promoted bytes with A",
    )?;
    if !output.status.success() {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Identity,
            "A changed promoted evidence after Q",
        ));
    }
    Ok(())
}

fn derive_witness_closure(
    repository_root: &Path,
    revision: &str,
) -> Result<String, AcceptanceError> {
    let bytes = git_file(repository_root, revision, MATERIALS_MANIFEST)?;
    let manifest: MaterialsManifest = serde_json::from_slice(&bytes).map_err(|error| {
        AcceptanceError::new(
            AcceptanceErrorKind::Schema,
            format!("invalid witness materials at {revision}: {error}"),
        )
    })?;
    if manifest.schema_version != 1
        || manifest.target != "phase9-lifecycle-contact-witness"
        || manifest.preset != "oracle-debug"
    {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Schema,
            "witness materials have the wrong schema, target, or preset",
        ));
    }
    let mut entries = git_entries(repository_root, revision, &WITNESS_REPOSITORY_PREFIXES)?;
    for material in manifest.materials {
        if !matches!(material.kind.as_str(), "source" | "header" | "build_rule") {
            continue;
        }
        let maybe_bytes = if material.identity.starts_with("third_party/liquidfun/") {
            require_pinned_upstream(repository_root, revision)?;
            Some(fs::read(repository_root.join(&material.identity)).map_err(filesystem)?)
        } else if git_file_exists(repository_root, revision, &material.identity)? {
            Some(git_file(repository_root, revision, &material.identity)?)
        } else {
            None
        };
        if let Some(bytes) = maybe_bytes {
            entries.insert(material.identity, sha256(&bytes));
        }
    }
    closure_digest("witness", entries)
}

fn derive_replay_closure(
    repository_root: &Path,
    revision: &str,
    promotion_base_sha: &str,
) -> Result<String, AcceptanceError> {
    let mut entries = git_entries(repository_root, revision, &REPLAY_REPOSITORY_PREFIXES)?;
    // Q's seven reviewed paths are promotion outputs, not fresh producer inputs. Project those
    // paths back to R while every other input remains read from the revision under acceptance.
    for path in PROMOTED_PATHS {
        if entries.contains_key(path) {
            let base_bytes = git_file(repository_root, promotion_base_sha, path)?;
            entries.insert(path.to_owned(), sha256(&base_bytes));
        }
    }
    closure_digest("replay", entries)
}

fn require_pinned_upstream(repository_root: &Path, revision: &str) -> Result<(), AcceptanceError> {
    let gitlink = git_text(
        repository_root,
        &["ls-tree", revision, "third_party/liquidfun"],
    )?;
    let expected = format!("160000 commit {ORACLE_REVISION}\tthird_party/liquidfun");
    let checked_out = git_text(
        &repository_root.join("third_party/liquidfun"),
        &["rev-parse", "HEAD"],
    )?;
    if gitlink != expected || checked_out != ORACLE_REVISION {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Identity,
            "witness closure requires the exact pinned and initialized oracle gitlink",
        ));
    }
    Ok(())
}

fn git_entries(
    repository_root: &Path,
    revision: &str,
    prefixes: &[&str],
) -> Result<BTreeMap<String, String>, AcceptanceError> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(repository_root)
        .args(["ls-tree", "-r", "--name-only", revision, "--"])
        .args(prefixes);
    let output = successful_output(&mut command, "enumerate closure inputs")?;
    let names = String::from_utf8(output.stdout).map_err(|error| {
        AcceptanceError::new(
            AcceptanceErrorKind::Identity,
            format!("Git returned non-UTF-8 paths: {error}"),
        )
    })?;
    let mut entries = BTreeMap::new();
    for name in names.lines() {
        validate_relative_path(name)?;
        entries.insert(
            name.to_owned(),
            sha256(&git_file(repository_root, revision, name)?),
        );
    }
    Ok(entries)
}

fn closure_digest(
    label: &str,
    entries: BTreeMap<String, String>,
) -> Result<String, AcceptanceError> {
    if entries.is_empty() {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Closure,
            format!("{label} closure is empty"),
        ));
    }
    let mut hasher = Sha256::new();
    update_field(&mut hasher, b"phase13-closure-v1");
    update_field(&mut hasher, label.as_bytes());
    for (path, digest) in entries {
        update_field(&mut hasher, path.as_bytes());
        update_field(&mut hasher, digest.as_bytes());
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn promoted_path_set_sha256(paths: &[String]) -> Result<String, AcceptanceError> {
    let actual = paths.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = PROMOTED_PATHS.into_iter().collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(AcceptanceError::new(
            AcceptanceErrorKind::Identity,
            "promoted path set is incomplete or contains extra paths",
        ));
    }
    let mut hasher = Sha256::new();
    update_field(&mut hasher, b"phase13-promoted-path-set-v1");
    for path in PROMOTED_PATHS {
        update_field(&mut hasher, path.as_bytes());
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[derive(Debug)]
struct CommandSpec {
    program: &'static str,
    args: &'static [&'static str],
}

impl CommandSpec {
    fn display(&self) -> String {
        std::iter::once(self.program)
            .chain(self.args.iter().copied())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn effect_steps() -> Vec<(AcceptanceStep, Vec<CommandSpec>)> {
    vec![
        (
            AcceptanceStep::Provenance,
            vec![
                xtask(&[
                    "phase13",
                    "evidence",
                    "check",
                    "--tracked",
                    "--require-reviewed",
                ]),
                xtask(&["provenance", "check"]),
                xtask(&["inventory", "check"]),
            ],
        ),
        (
            AcceptanceStep::ReviewedReplay,
            vec![cargo(&[
                "test",
                "-p",
                "liquidfun-differential",
                "--test",
                "catalog_regressions",
                "tracked_catalog_regressions_replay_byte_identically_without_writes",
            ])],
        ),
        (
            AcceptanceStep::Diagnosis,
            vec![cargo(&[
                "test",
                "-p",
                "liquidfun-differential",
                "--test",
                "catalog_regressions",
                "diagnosis",
            ])],
        ),
        (
            AcceptanceStep::Regression,
            vec![cargo(&[
                "test",
                "-p",
                "liquidfun-differential",
                "--test",
                "catalog_regressions",
            ])],
        ),
        (
            AcceptanceStep::OracleBuild,
            vec![
                xtask(&["upstream", "verify"]),
                xtask(&["upstream", "configure", "--preset", "oracle-debug"]),
                xtask(&["upstream", "build", "--preset", "oracle-debug"]),
            ],
        ),
        (
            AcceptanceStep::Comparison,
            vec![xtask(&[
                "catalog",
                "compare",
                "--scenario",
                "rigid-stack-stability",
                "--timestep",
                "0.016666668",
                "--velocity-iterations",
                "8",
                "--position-iterations",
                "3",
                "--particle-iterations",
                "1",
                "--oracle-preset",
                "oracle-debug",
                "--session-profile",
                "one-shot",
                "--output",
                "human",
                "--commands",
                "auto",
            ])],
        ),
    ]
}

const fn xtask(args: &'static [&'static str]) -> CommandSpec {
    CommandSpec {
        program: "xtask",
        args,
    }
}

const fn cargo(args: &'static [&'static str]) -> CommandSpec {
    CommandSpec {
        program: "cargo",
        args,
    }
}

fn run_command(repository_root: &Path, spec: &CommandSpec) -> Result<(), AcceptanceError> {
    let mut command = if spec.program == "xtask" {
        let executable = env::current_exe().map_err(filesystem)?;
        Command::new(executable)
    } else {
        Command::new(env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
    };
    successful_output(
        command.current_dir(repository_root).args(spec.args),
        &spec.display(),
    )
    .map(|_output| ())
}

fn assert_head(repository_root: &Path, expected_sha: &str) -> Result<(), AcceptanceError> {
    let observed_sha = git_text(repository_root, &["rev-parse", "HEAD"])?;
    let status = git_text(
        repository_root,
        &["status", "--porcelain=v1", "--untracked-files=normal"],
    )?;
    validate_head_snapshot(&HeadSnapshot {
        expected_sha: expected_sha.to_owned(),
        observed_sha,
        clean: status.is_empty(),
    })
}

fn is_ancestor(repository_root: &Path, older: &str, newer: &str) -> Result<bool, AcceptanceError> {
    let output = run_process(
        Command::new("git").arg("-C").arg(repository_root).args([
            "merge-base",
            "--is-ancestor",
            older,
            newer,
        ]),
        "check Git ancestry",
    )?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(process_failure("check Git ancestry", &output)),
    }
}

fn git_file_exists(
    repository_root: &Path,
    revision: &str,
    path: &str,
) -> Result<bool, AcceptanceError> {
    validate_relative_path(path)?;
    let object = format!("{revision}:{path}");
    let output = run_process(
        Command::new("git")
            .arg("-C")
            .arg(repository_root)
            .args(["cat-file", "-e", &object]),
        "inspect closure input",
    )?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1 | 128) => Ok(false),
        _ => Err(process_failure("inspect closure input", &output)),
    }
}

fn git_file(
    repository_root: &Path,
    revision: &str,
    path: &str,
) -> Result<Vec<u8>, AcceptanceError> {
    validate_relative_path(path)?;
    let object = format!("{revision}:{path}");
    successful_output(
        Command::new("git")
            .arg("-C")
            .arg(repository_root)
            .args(["show", &object]),
        "read closure input",
    )
    .map(|output| output.stdout)
}

fn git_text(repository_root: &Path, args: &[&str]) -> Result<String, AcceptanceError> {
    let output = successful_output(
        Command::new("git")
            .arg("-C")
            .arg(repository_root)
            .args(args),
        "query Git",
    )?;
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_owned())
        .map_err(|error| {
            AcceptanceError::new(
                AcceptanceErrorKind::Identity,
                format!("Git returned non-UTF-8 output: {error}"),
            )
        })
}

fn run_process(command: &mut Command, action: &str) -> Result<Output, AcceptanceError> {
    command.output().map_err(|error| {
        AcceptanceError::new(
            AcceptanceErrorKind::Process,
            format!("failed to {action}: {error}"),
        )
    })
}

fn successful_output(command: &mut Command, action: &str) -> Result<Output, AcceptanceError> {
    let output = run_process(command, action)?;
    if output.status.success() {
        return Ok(output);
    }
    Err(process_failure(action, &output))
}

fn process_failure(action: &str, output: &Output) -> AcceptanceError {
    AcceptanceError::new(
        AcceptanceErrorKind::Process,
        format!(
            "`{action}` failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ),
    )
}

fn repository_root() -> Result<PathBuf, AcceptanceError> {
    let current = env::current_dir().map_err(filesystem)?;
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
    serde_json::from_slice(&fs::read(path).map_err(filesystem)?).map_err(|error| {
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
    .map_err(filesystem)?;
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
        .map_err(filesystem)?;
    file.write_all(&bytes).map_err(filesystem)?;
    file.sync_all().map_err(filesystem)
}

fn file_sha256(path: &Path) -> Result<String, AcceptanceError> {
    fs::read(path)
        .map(|bytes| sha256(&bytes))
        .map_err(filesystem)
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

fn filesystem(error: std::io::Error) -> AcceptanceError {
    AcceptanceError::new(AcceptanceErrorKind::Filesystem, error.to_string())
}
