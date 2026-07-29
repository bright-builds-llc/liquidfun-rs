//! Canonical Phase 13 evidence production without tracked promotion.

#[path = "phase13_evidence/acquisition.rs"]
mod acquisition;
#[path = "phase13_evidence/bundle.rs"]
pub(crate) mod bundle;
#[path = "phase13_evidence/replay.rs"]
mod replay;
#[path = "phase13_evidence/support.rs"]
mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt::{self, Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output};

use bundle::{
    BundleDraft, BundleFile, ClosureEntry, ClosureIdentity, EvidenceMetadata, check_bundle,
    closure_digest, write_bundle,
};
use liquidfun_differential::{
    CatalogComparisonSurface, CatalogFailureBundleRequest, CatalogFailureKind,
    CatalogOracleSupervisor, CatalogRunOutcome, ComparisonState, OracleExecutable, OraclePreset,
    ReplayDriftClass, ReplayProjectionVersion, SessionProfile, compare_catalog_physics_projection,
    execute_catalog_native, legacy_physics_checkpoint_sha256, persist_catalog_failure_bundle,
    replay_catalog_regressions,
};
use liquidfun_test_protocol::{
    BuildEvidenceTier, CatalogDefinition, CatalogRunRequest, CatalogSlug, EvidenceTier, RequestId,
    ResolveRequest, RunProvenanceRequirements, ScenarioCatalog, resolve_catalog,
    reviewed_scenario_catalog,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use acquisition::{
    acquire_check, acquire_replay, derive_git_closure, derive_witness_closure, produce_witness,
};
use replay::{WitnessProvenance, live_check, replay_record};
use support::{
    absolute_path, canonical_environment, file_sha256, git_text, json_bytes, lexical_absolute,
    metadata, parse_options, repository_root, require_allowed_options, require_options,
    require_upstream_revision, required, run_xtask, valid_digest, valid_revision,
};

pub(crate) fn select_rigid_stack_definition(
    catalog: &ScenarioCatalog,
) -> Result<(&CatalogDefinition, CatalogSlug), Phase13EvidenceError> {
    acquisition::select_rigid_stack_definition(catalog)
}

pub(crate) fn witness_materials_identity(
    repository_root: &Path,
) -> Result<(String, usize), Phase13EvidenceError> {
    acquisition::witness_materials_identity(repository_root)
}

pub(crate) fn compare_live_replay_records(
    reviewed: &serde_json::Value,
    current: &serde_json::Value,
) -> Result<(), replay::LiveReplayMismatch> {
    replay::compare_live_replay_records(reviewed, current)
}

pub(crate) fn persist_live_check_failure(
    repository_root: &Path,
    reviewed: &serde_json::Value,
    current: &serde_json::Value,
    mismatch: &replay::LiveReplayMismatch,
) -> Result<PathBuf, Phase13EvidenceError> {
    replay::persist_live_check_failure(repository_root, reviewed, current, mismatch)
}

const USAGE: &str = r"Usage: cargo xtask phase13 evidence <command> [arguments]

Commands:
  produce --staging-root <path> --producer-sha <full-sha>
  check --staging-root <path> --expected-producer-sha <full-sha> --expected-bundle-sha256 <sha256> [--expected-witness-closure <sha256> --expected-replay-closure <sha256>]
  acquire-check --staging-root <path> --run-id <decimal> --artifact-id <decimal> --artifact-name <name> --provider-digest <sha256:hex> --expected-producer-sha <full-sha> --expected-bundle-sha256 <sha256>
  live-check --tracked --require-reviewed";

const UPSTREAM_REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const MATERIALS_MANIFEST: &str = "tools/reference/phase9-lifecycle-contact-witness.materials.json";
const PROBE_SOURCE: &str = "tools/reference/src/phase9_lifecycle_contact_witness.cpp";
const TOLERANCE_PROFILE: &str = "protocol/tolerances/phase4-v1.toml";
const RIGID_STACK_FIXTURE: &str = "scenarios/catalog/rigid-stack-v1.json";
const RIGID_STACK_CATALOG_SLUG: &str = "rigid-stack-stability";
const WITNESS_EXECUTABLE: &str = "target/reference/oracle-debug/phase9-lifecycle-contact-witness";
const ORACLE_PRESET: &str = "oracle-debug";
const TARGET_TRIPLE: &str = "x86_64-unknown-linux-gnu";

const WITNESS_REPOSITORY_PREFIXES: &[&str] = &[
    MATERIALS_MANIFEST,
    "tools/xtask/src/phase13_evidence.rs",
    "tools/xtask/src/phase13_evidence/bundle.rs",
];
const REPLAY_REPOSITORY_PREFIXES: &[&str] = &[
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
pub(crate) enum ProductionGateErrorKind {
    D1Failure,
    D1InputMismatch,
    Environment,
    Identity,
    NativeRepeatMismatch,
    WitnessRepeatMismatch,
}

#[derive(Debug)]
pub(crate) struct ProductionGateError {
    kind: ProductionGateErrorKind,
    message: &'static str,
}

impl ProductionGateError {
    const fn new(kind: ProductionGateErrorKind, message: &'static str) -> Self {
        Self { kind, message }
    }

    #[allow(
        dead_code,
        reason = "integration contract tests inspect stable categories"
    )]
    pub(crate) const fn kind(&self) -> ProductionGateErrorKind {
        self.kind
    }
}

impl Display for ProductionGateError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "phase13 producer/{:?}: {}",
            self.kind, self.message
        )
    }
}

impl std::error::Error for ProductionGateError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CanonicalEnvironment {
    pub(crate) operating_system: String,
    pub(crate) architecture: String,
    pub(crate) rust_target: String,
    pub(crate) rust_version: String,
    pub(crate) cmake_version: String,
    pub(crate) ninja_version: String,
    pub(crate) clang_version: String,
    pub(crate) cmake_preset: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProductionGate {
    pub(crate) producer_sha: String,
    pub(crate) upstream_revision: String,
    pub(crate) environment: CanonicalEnvironment,
    pub(crate) witness_repeat_sha256: [String; 2],
    pub(crate) native_d0_repeat_sha256: [String; 2],
    pub(crate) d1_oracle_passed: bool,
    pub(crate) sealed_input_sha256: String,
    pub(crate) d1_input_sha256: String,
}

impl ProductionGate {
    pub(crate) fn validate(&self) -> Result<(), ProductionGateError> {
        if !valid_revision(&self.producer_sha)
            || !valid_revision(&self.upstream_revision)
            || self.upstream_revision != UPSTREAM_REVISION
            || !self
                .witness_repeat_sha256
                .iter()
                .all(|value| valid_digest(value))
            || !self
                .native_d0_repeat_sha256
                .iter()
                .all(|value| valid_digest(value))
            || !valid_digest(&self.sealed_input_sha256)
            || !valid_digest(&self.d1_input_sha256)
        {
            return Err(ProductionGateError::new(
                ProductionGateErrorKind::Identity,
                "P, upstream, input, and output identities must be complete lowercase hashes",
            ));
        }
        if self.environment.operating_system != "linux"
            || self.environment.architecture != "x86_64"
            || self.environment.rust_target != TARGET_TRIPLE
            || self.environment.rust_version != "1.97.0"
            || self.environment.cmake_version != "4.3.3"
            || self.environment.ninja_version != "1.13.2"
            || self.environment.clang_version != "22.1.8"
            || self.environment.cmake_preset != ORACLE_PRESET
        {
            return Err(ProductionGateError::new(
                ProductionGateErrorKind::Environment,
                "canonical evidence requires pinned scalar x86_64 Linux tools",
            ));
        }
        if self.witness_repeat_sha256[0] != self.witness_repeat_sha256[1] {
            return Err(ProductionGateError::new(
                ProductionGateErrorKind::WitnessRepeatMismatch,
                "pinned-oracle witness repeats were not byte-identical",
            ));
        }
        if self.native_d0_repeat_sha256[0] != self.native_d0_repeat_sha256[1] {
            return Err(ProductionGateError::new(
                ProductionGateErrorKind::NativeRepeatMismatch,
                "native D0 repeats were not byte-identical",
            ));
        }
        if self.sealed_input_sha256 != self.d1_input_sha256 {
            return Err(ProductionGateError::new(
                ProductionGateErrorKind::D1InputMismatch,
                "D1 did not consume the exact sealed D0 input",
            ));
        }
        if !self.d1_oracle_passed {
            return Err(ProductionGateError::new(
                ProductionGateErrorKind::D1Failure,
                "pinned-oracle D1 comparison did not pass",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase13EvidenceErrorKind {
    Bundle,
    Environment,
    Filesystem,
    Git,
    Oracle,
    Process,
    Protocol,
    Usage,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Phase13EvidenceError {
    kind: Phase13EvidenceErrorKind,
    message: String,
}

impl Phase13EvidenceError {
    fn new(kind: Phase13EvidenceErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    fn usage(message: impl Into<String>) -> Self {
        Self::new(
            Phase13EvidenceErrorKind::Usage,
            format!("{}\n\n{USAGE}", message.into()),
        )
    }
}

impl Display for Phase13EvidenceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "phase13 evidence/{:?}: {}",
            self.kind, self.message
        )
    }
}

impl std::error::Error for Phase13EvidenceError {}

pub(crate) fn run(args: &[String]) -> Result<(), Phase13EvidenceError> {
    let [namespace, command, tail @ ..] = args else {
        return Err(Phase13EvidenceError::usage("expected `evidence <command>`"));
    };
    if namespace != "evidence" {
        return Err(Phase13EvidenceError::usage(
            "only the `evidence` namespace is available",
        ));
    }
    let repository_root = repository_root()?;
    if command == "live-check" {
        if tail != ["--tracked", "--require-reviewed"] {
            return Err(Phase13EvidenceError::usage(
                "live-check requires exactly `--tracked --require-reviewed`",
            ));
        }
        return live_check(&repository_root);
    }
    let options = parse_options(tail)?;
    match command.as_str() {
        "produce" => {
            require_options(&options, &["--staging-root", "--producer-sha"])?;
            let staging_root =
                absolute_path(&repository_root, required(&options, "--staging-root")?);
            validate_staging_root(&repository_root, &staging_root)?;
            let identity = produce(
                &repository_root,
                &staging_root,
                required(&options, "--producer-sha")?,
            )?;
            println!(
                "phase13 evidence produced: P={} B={}",
                identity.producer_sha, identity.bundle_sha256
            );
            Ok(())
        }
        "check" => {
            require_allowed_options(
                &options,
                &[
                    "--staging-root",
                    "--expected-producer-sha",
                    "--expected-bundle-sha256",
                ],
                &["--expected-witness-closure", "--expected-replay-closure"],
            )?;
            let staging_root =
                absolute_path(&repository_root, required(&options, "--staging-root")?);
            let identity = check_bundle(
                &staging_root,
                required(&options, "--expected-producer-sha")?,
                required(&options, "--expected-bundle-sha256")?,
                options
                    .get("--expected-witness-closure")
                    .map(String::as_str),
                options.get("--expected-replay-closure").map(String::as_str),
            )
            .map_err(|error| {
                Phase13EvidenceError::new(Phase13EvidenceErrorKind::Bundle, error.to_string())
            })?;
            println!(
                "phase13 evidence verified: P={} B={}",
                identity.producer_sha, identity.bundle_sha256
            );
            Ok(())
        }
        "acquire-check" => acquire_check(&repository_root, &options),
        unknown => Err(Phase13EvidenceError::usage(format!(
            "unknown evidence command `{unknown}`"
        ))),
    }
}

pub(crate) fn validate_staging_root(
    repository_root: &Path,
    staging_root: &Path,
) -> Result<(), Phase13EvidenceError> {
    let root = lexical_absolute(repository_root)?;
    let staging = lexical_absolute(staging_root)?;
    if staging == root
        || staging.starts_with(root.join(".git"))
        || staging.starts_with(root.join(".github"))
        || staging.starts_with(root.join("crates"))
        || staging.starts_with(root.join("protocol"))
        || staging.starts_with(root.join("reference"))
        || staging.starts_with(root.join("scenarios"))
        || staging.starts_with(root.join("tools"))
        || staging.starts_with(root.join(".planning"))
        || staging.starts_with(root.join("docs"))
        || (staging.starts_with(&root) && !staging.starts_with(root.join("target")))
    {
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Filesystem,
            "staging root may be outside the repository or beneath target only",
        ));
    }
    Ok(())
}

#[allow(
    clippy::too_many_lines,
    reason = "the fail-closed production transaction keeps identity checks and staged outputs in one reviewable sequence"
)]
fn produce(
    repository_root: &Path,
    staging_root: &Path,
    expected_producer_sha: &str,
) -> Result<bundle::BundleIdentity, Phase13EvidenceError> {
    let producer_sha = git_text(repository_root, &["rev-parse", "HEAD"])?;
    if producer_sha != expected_producer_sha || !valid_revision(&producer_sha) {
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Git,
            "checked-out HEAD does not equal the requested full producer SHA",
        ));
    }
    let status = git_text(
        repository_root,
        &["status", "--porcelain=v1", "--untracked-files=no"],
    )?;
    if !status.is_empty() {
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Git,
            "canonical production requires a clean tracked worktree",
        ));
    }
    require_upstream_revision(repository_root)?;
    let environment = canonical_environment()?;
    run_xtask(
        repository_root,
        &["upstream", "configure", "--preset", ORACLE_PRESET],
    )?;
    run_xtask(
        repository_root,
        &[
            "upstream",
            "build",
            "--preset",
            ORACLE_PRESET,
            "--target",
            "phase9-lifecycle-contact-witness",
        ],
    )?;
    run_xtask(
        repository_root,
        &["upstream", "build", "--preset", ORACLE_PRESET],
    )?;

    let temporary_root = repository_root
        .join("target/phase13/producer")
        .join(&producer_sha);
    if temporary_root.exists() {
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Filesystem,
            "producer scratch path already exists; remove the prior failed run before retrying",
        ));
    }
    fs::create_dir_all(&temporary_root).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Filesystem,
            format!("failed to create producer scratch path: {error}"),
        )
    })?;
    let witness = produce_witness(repository_root, &temporary_root)?;
    let replay = acquire_replay(repository_root, false)?.output;
    let witness_closure = derive_witness_closure(repository_root, &producer_sha)?;
    let (materials_sha256, materials_count) = witness_materials_identity(repository_root)?;
    let replay_closure = derive_git_closure(
        repository_root,
        &producer_sha,
        "replay",
        REPLAY_REPOSITORY_PREFIXES,
    )?;

    let gate = ProductionGate {
        producer_sha: producer_sha.clone(),
        upstream_revision: UPSTREAM_REVISION.to_owned(),
        environment,
        witness_repeat_sha256: witness.repeat_sha256.clone(),
        native_d0_repeat_sha256: replay.native_repeat_sha256.clone(),
        d1_oracle_passed: replay.d1_passed,
        sealed_input_sha256: replay.sealed_input_sha256.clone(),
        d1_input_sha256: replay.sealed_input_sha256.clone(),
    };
    gate.validate().map_err(|error| {
        let message = replay.d1_diagnostic.as_ref().map_or_else(
            || error.to_string(),
            |diagnostic| format!("{error}; {diagnostic}"),
        );
        Phase13EvidenceError::new(Phase13EvidenceErrorKind::Environment, message)
    })?;

    let witness_provenance = WitnessProvenance {
        schema_version: 2,
        repository_revision: producer_sha,
        oracle_revision: UPSTREAM_REVISION.to_owned(),
        materials_manifest_sha256: file_sha256(&repository_root.join(MATERIALS_MANIFEST))?,
        materials_sha256,
        materials_count,
        probe_source_sha256: file_sha256(&repository_root.join(PROBE_SOURCE))?,
        compiler_id: "Clang".to_owned(),
        compiler_version: "22.1.8".to_owned(),
        target: TARGET_TRIPLE.to_owned(),
        cmake_preset: ORACLE_PRESET.to_owned(),
        cmake_target: "phase9-lifecycle-contact-witness".to_owned(),
        exact_argv: witness.invocation.clone(),
        witness_sha256: witness.repeat_sha256[0].clone(),
    };
    let replay_record = replay_record(&replay)?;
    let witness_provenance_bytes = json_bytes(&witness_provenance)?;
    let replay_bytes = json_bytes(&replay_record)?;
    let materials_manifest_sha256 = file_sha256(&repository_root.join(MATERIALS_MANIFEST))?;
    let probe_source_sha256 = file_sha256(&repository_root.join(PROBE_SOURCE))?;
    let tolerance_identity = file_sha256(&repository_root.join(TOLERANCE_PROFILE))?;

    let draft = BundleDraft {
        producer: gate,
        witness_closure,
        replay_closure,
        materials_manifest_sha256,
        materials_sha256: witness_provenance.materials_sha256.clone(),
        probe_source_sha256,
        schema_identity: "phase13-evidence-v1".to_owned(),
        tolerance_identity,
        witness_invocation: witness.invocation,
        replay_invocations: vec![
            "execute_catalog_native(rigid-stack-v1,repeat=1)".to_owned(),
            "execute_catalog_native(rigid-stack-v1,repeat=2)".to_owned(),
            "CatalogOracleSupervisor(oracle-debug,reuse,rigid-stack-v1)".to_owned(),
        ],
        d1_oracle_identity_sha256: replay.oracle_identity_sha256,
        d1_result: "match".to_owned(),
        diagnosis: replay.diagnosis,
        bundle_metadata: metadata("staged_bundle", "."),
    };
    let files = vec![
        BundleFile {
            path: "evidence/replay.json".to_owned(),
            bytes: replay_bytes,
            metadata: metadata("replay_evidence", "."),
        },
        BundleFile {
            path: "evidence/witness.json".to_owned(),
            bytes: witness.bytes,
            metadata: metadata(
                "witness",
                "liquidfun/Box2D/Box2D/Particle/b2ParticleSystem.cpp",
            ),
        },
        BundleFile {
            path: "evidence/witness.provenance.json".to_owned(),
            bytes: witness_provenance_bytes,
            metadata: metadata("witness", "."),
        },
        BundleFile {
            path: "sealed/rigid-stack-v1.json".to_owned(),
            bytes: fs::read(repository_root.join(RIGID_STACK_FIXTURE)).map_err(|error| {
                Phase13EvidenceError::new(
                    Phase13EvidenceErrorKind::Filesystem,
                    format!("failed to read sealed input: {error}"),
                )
            })?,
            metadata: metadata("replay_evidence", "."),
        },
    ];
    write_bundle(staging_root, draft, files).map_err(|error| {
        Phase13EvidenceError::new(Phase13EvidenceErrorKind::Bundle, error.to_string())
    })
}
