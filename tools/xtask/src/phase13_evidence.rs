//! Canonical Phase 13 evidence production without tracked promotion.

#[path = "phase13_evidence/bundle.rs"]
pub(crate) mod bundle;

use std::collections::BTreeMap;
use std::env;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output};

use bundle::{
    BundleDraft, BundleFile, ClosureEntry, ClosureIdentity, EvidenceMetadata, check_bundle,
    closure_digest, write_bundle,
};
use liquidfun_differential::{
    CatalogOracleSupervisor, CatalogRunOutcome, ComparisonState, OracleExecutable, OraclePreset,
    ReplayDriftClass, ReplayProjectionVersion, SessionProfile, compare_catalog_physics_projection,
    execute_catalog_native, legacy_physics_checkpoint_sha256, replay_catalog_regressions,
};
use liquidfun_test_protocol::{
    BuildEvidenceTier, CatalogDefinition, CatalogRunRequest, CatalogSlug, EvidenceTier, RequestId,
    ResolveRequest, RunProvenanceRequirements, ScenarioCatalog, resolve_catalog,
    reviewed_scenario_catalog,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const USAGE: &str = r"Usage: cargo xtask phase13 evidence <command> [arguments]

Commands:
  produce --staging-root <path> --producer-sha <full-sha>
  check --staging-root <path> --expected-producer-sha <full-sha> --expected-bundle-sha256 <sha256> [--expected-witness-closure <sha256> --expected-replay-closure <sha256>]
  acquire-check --staging-root <path> --run-id <decimal> --artifact-id <decimal> --artifact-name <name> --provider-digest <sha256:hex> --expected-producer-sha <full-sha> --expected-bundle-sha256 <sha256>";

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
    let options = parse_options(tail)?;
    let repository_root = repository_root()?;
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
    let replay = produce_replay(repository_root)?;
    let witness_closure = derive_witness_closure(repository_root, &producer_sha)?;
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
        materials_sha256: witness_closure.digest.clone(),
        materials_count: witness_closure.entries.len(),
        probe_source_sha256: file_sha256(&repository_root.join(PROBE_SOURCE))?,
        compiler_id: "Clang".to_owned(),
        compiler_version: "22.1.8".to_owned(),
        target: TARGET_TRIPLE.to_owned(),
        cmake_preset: ORACLE_PRESET.to_owned(),
        cmake_target: "phase9-lifecycle-contact-witness".to_owned(),
        exact_argv: witness.invocation.clone(),
        witness_sha256: witness.repeat_sha256[0].clone(),
    };
    let replay_record = ReplayEvidenceRecord {
        schema_version: 1,
        upstream_revision: UPSTREAM_REVISION.to_owned(),
        resolved_scenario_path: RIGID_STACK_FIXTURE.to_owned(),
        sealed_input_sha256: replay.sealed_input_sha256,
        native_d0_repeat_sha256: replay.native_repeat_sha256,
        d1_oracle_identity_sha256: replay.oracle_identity_sha256.clone(),
        d1_result: "match".to_owned(),
        diagnosis: replay.diagnosis.clone(),
    };
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

struct WitnessOutput {
    bytes: Vec<u8>,
    repeat_sha256: [String; 2],
    invocation: Vec<String>,
}

fn produce_witness(
    repository_root: &Path,
    temporary_root: &Path,
) -> Result<WitnessOutput, Phase13EvidenceError> {
    let executable = repository_root.join(WITNESS_EXECUTABLE);
    let output_path = temporary_root.join("witness.json");
    let provenance_path = temporary_root.join("witness.provenance.raw.json");
    let invocation = vec![
        WITNESS_EXECUTABLE.to_owned(),
        "--output".to_owned(),
        path_text(&output_path)?,
        "--provenance".to_owned(),
        path_text(&provenance_path)?,
    ];
    let mut repeat_sha256 = [String::new(), String::new()];
    let mut bytes = Vec::new();
    for digest in &mut repeat_sha256 {
        run_process(
            Command::new(&executable)
                .current_dir(repository_root)
                .args(&invocation[1..]),
            "run the Phase 9 witness oracle",
        )?;
        bytes = fs::read(&output_path).map_err(|error| {
            Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Filesystem,
                format!("failed to read generated witness: {error}"),
            )
        })?;
        *digest = sha256(&bytes);
    }
    Ok(WitnessOutput {
        bytes,
        repeat_sha256,
        invocation,
    })
}

struct ReplayOutput {
    sealed_input_sha256: String,
    native_repeat_sha256: [String; 2],
    oracle_identity_sha256: String,
    d1_passed: bool,
    d1_diagnostic: Option<String>,
    diagnosis: serde_json::Value,
}

pub(crate) fn select_rigid_stack_definition(
    catalog: &ScenarioCatalog,
) -> Result<(&CatalogDefinition, CatalogSlug), Phase13EvidenceError> {
    let slug = CatalogSlug::new(RIGID_STACK_CATALOG_SLUG.to_owned()).map_err(|error| {
        Phase13EvidenceError::new(Phase13EvidenceErrorKind::Protocol, error.to_string())
    })?;
    let definition = catalog
        .definitions()
        .iter()
        .find(|candidate| candidate.slug() == &slug)
        .ok_or_else(|| {
            Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Protocol,
                "rigid-stack-v1 is absent from the reviewed catalog",
            )
        })?;
    Ok((definition, slug))
}

#[allow(
    clippy::too_many_lines,
    reason = "the D0/D1 authority sequence remains linear so no comparison gate can be skipped"
)]
fn produce_replay(repository_root: &Path) -> Result<ReplayOutput, Phase13EvidenceError> {
    let catalog = reviewed_scenario_catalog().map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            format!("reviewed catalog is invalid: {error}"),
        )
    })?;
    let (definition, slug) = select_rigid_stack_definition(&catalog)?;
    let metadata = definition.metadata().ok_or_else(|| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            "rigid-stack-v1 has no reviewed metadata",
        )
    })?;
    let resolved = resolve_catalog(
        catalog.definitions(),
        &ResolveRequest::new(slug, None, metadata.default_settings()),
    )
    .map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            format!("failed to resolve rigid-stack-v1: {error}"),
        )
    })?;
    let executable =
        OracleExecutable::resolve(repository_root, OraclePreset::Debug).map_err(|error| {
            Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Oracle,
                format!("canonical oracle is unavailable: {error}"),
            )
        })?;
    let mut supervisor =
        CatalogOracleSupervisor::new(executable, SessionProfile::Reuse, UPSTREAM_REVISION);
    let oracle_identity = supervisor.discover_identity().map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Oracle,
            format!("failed to discover oracle identity: {error}"),
        )
    })?;
    if oracle_identity.evidence_tier() != BuildEvidenceTier::D1Canonical {
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Environment,
            "oracle build identity is not canonical D1",
        ));
    }
    let request_id = RequestId::new(format!(
        "phase13-{}",
        &resolved.identity().content_sha256().as_str()[..16]
    ))
    .map_err(|error| {
        Phase13EvidenceError::new(Phase13EvidenceErrorKind::Protocol, error.to_string())
    })?;
    let request = CatalogRunRequest::new(
        request_id,
        resolved,
        RunProvenanceRequirements::new(
            oracle_identity.identity_sha256().clone(),
            supervisor.limits_profile_sha256(),
            EvidenceTier::D1Canonical,
        ),
    )
    .map_err(|error| {
        Phase13EvidenceError::new(Phase13EvidenceErrorKind::Protocol, error.to_string())
    })?;
    let first = execute_catalog_native(&request).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            format!("first native D0 failed: {error}"),
        )
    })?;
    let second = execute_catalog_native(&request).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            format!("second native D0 failed: {error}"),
        )
    })?;
    let replay = replay_catalog_regressions(repository_root).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            format!("reviewed replay diagnosis failed: {error}"),
        )
    })?;
    let replay_entry = replay
        .entries()
        .iter()
        .find(|entry| entry.fixture_id() == "rigid-stack-v1")
        .ok_or_else(|| {
            Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Protocol,
                "rigid-stack-v1 replay result is absent",
            )
        })?;
    let diagnosis = replay_entry.maybe_diagnosis().ok_or_else(|| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            "rigid-stack-v1 capture-schema diagnosis is absent",
        )
    })?;
    if diagnosis.drift_class() != ReplayDriftClass::CaptureSchemaDrift
        || diagnosis.reviewed_schema().projection_version()
            != ReplayProjectionVersion::LegacyPhysicsV1
        || diagnosis.current_schema().projection_version()
            != ReplayProjectionVersion::ExpandedCheckpointV1
    {
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            "rigid-stack-v1 did not select the reviewed legacy physics projection",
        ));
    }
    let native_repeat_sha256 = [
        legacy_physics_checkpoint_sha256(first.checkpoints())
            .map_err(|error| {
                Phase13EvidenceError::new(
                    Phase13EvidenceErrorKind::Protocol,
                    format!("first legacy D0 projection failed: {error}"),
                )
            })?
            .as_str()
            .to_owned(),
        legacy_physics_checkpoint_sha256(second.checkpoints())
            .map_err(|error| {
                Phase13EvidenceError::new(
                    Phase13EvidenceErrorKind::Protocol,
                    format!("second legacy D0 projection failed: {error}"),
                )
            })?
            .as_str()
            .to_owned(),
    ];
    let oracle = supervisor.execute(&request).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Oracle,
            format!("pinned-oracle D1 execution failed: {error}"),
        )
    })?;
    let outcome =
        compare_catalog_physics_projection(&first, oracle.capture()).map_err(|error| {
            Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Protocol,
                format!("D1 comparison failed: {error}"),
            )
        })?;
    let d1_passed = matches!(outcome, CatalogRunOutcome::Match(_));
    let d1_diagnostic = match &outcome {
        CatalogRunOutcome::Match(_) => None,
        CatalogRunOutcome::PhysicsMismatch(mismatch) => {
            let comparison = mismatch.first_mismatch();
            let maybe_entry = comparison.entries().iter().find(|entry| {
                !matches!(
                    entry.state(),
                    ComparisonState::ExactMatch | ComparisonState::WithinPolicy
                )
            });
            Some(maybe_entry.map_or_else(
                || {
                    format!(
                        "D1 physics mismatch at checkpoint {}",
                        comparison.checkpoint_id().as_str()
                    )
                },
                |entry| {
                    format!(
                        "D1 physics mismatch at checkpoint {} path {} ({:?}): Rust={:?}, C++={:?}",
                        comparison.checkpoint_id().as_str(),
                        entry.semantic_path(),
                        entry.state(),
                        entry.maybe_rust_value(),
                        entry.maybe_oracle_value()
                    )
                },
            ))
        }
        CatalogRunOutcome::HarnessFailure(kind) => {
            Some(format!("D1 comparison reported harness failure {kind:?}"))
        }
    };
    Ok(ReplayOutput {
        sealed_input_sha256: request
            .resolved()
            .identity()
            .content_sha256()
            .as_str()
            .to_owned(),
        native_repeat_sha256,
        oracle_identity_sha256: oracle_identity.identity_sha256().as_str().to_owned(),
        d1_passed,
        d1_diagnostic,
        diagnosis: serde_json::to_value(diagnosis).map_err(|error| {
            Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Protocol,
                format!("failed to encode replay diagnosis: {error}"),
            )
        })?,
    })
}

fn derive_witness_closure(
    repository_root: &Path,
    producer_sha: &str,
) -> Result<ClosureIdentity, Phase13EvidenceError> {
    let manifest_bytes = fs::read(repository_root.join(MATERIALS_MANIFEST)).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Filesystem,
            format!("failed to read witness materials: {error}"),
        )
    })?;
    let manifest: MaterialsManifest = serde_json::from_slice(&manifest_bytes).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            format!("invalid witness materials: {error}"),
        )
    })?;
    if manifest.schema_version != 1
        || manifest.target != "phase9-lifecycle-contact-witness"
        || manifest.preset != ORACLE_PRESET
    {
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            "witness materials manifest has the wrong schema, target, or preset",
        ));
    }
    let mut entries =
        derive_git_entries(repository_root, producer_sha, WITNESS_REPOSITORY_PREFIXES)?;
    for material in manifest.materials {
        let path = material.identity;
        if !matches!(material.kind.as_str(), "source" | "header" | "build_rule") {
            continue;
        }
        let candidate = repository_root.join(&path);
        if candidate.is_file() {
            entries.insert(path, file_sha256(&candidate)?);
        }
    }
    closure_from_entries("witness", entries)
}

fn derive_git_closure(
    repository_root: &Path,
    producer_sha: &str,
    label: &str,
    prefixes: &[&str],
) -> Result<ClosureIdentity, Phase13EvidenceError> {
    closure_from_entries(
        label,
        derive_git_entries(repository_root, producer_sha, prefixes)?,
    )
}

fn derive_git_entries(
    repository_root: &Path,
    producer_sha: &str,
    prefixes: &[&str],
) -> Result<BTreeMap<String, String>, Phase13EvidenceError> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(repository_root)
        .args(["ls-tree", "-r", "--name-only", producer_sha, "--"])
        .args(prefixes);
    let output = run_process(&mut command, "enumerate producer-affecting Git inputs")?;
    let names = String::from_utf8(output.stdout).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Git,
            format!("Git returned non-UTF-8 paths: {error}"),
        )
    })?;
    let mut entries = BTreeMap::new();
    for path in names.lines() {
        validate_relative_path(path)?;
        let object = format!("{producer_sha}:{path}");
        let output = run_process(
            Command::new("git")
                .arg("-C")
                .arg(repository_root)
                .args(["show", &object]),
            "read producer-affecting Git input",
        )?;
        entries.insert(path.to_owned(), sha256(&output.stdout));
    }
    Ok(entries)
}

fn closure_from_entries(
    label: &str,
    entries: BTreeMap<String, String>,
) -> Result<ClosureIdentity, Phase13EvidenceError> {
    if entries.is_empty() {
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Git,
            format!("{label} producer-affecting closure is empty"),
        ));
    }
    let entries = entries
        .into_iter()
        .map(|(path, sha256)| ClosureEntry { path, sha256 })
        .collect::<Vec<_>>();
    let digest = closure_digest(label, &entries);
    Ok(ClosureIdentity {
        schema_version: 1,
        label: label.to_owned(),
        digest,
        entries,
    })
}

fn acquire_check(
    repository_root: &Path,
    options: &BTreeMap<String, String>,
) -> Result<(), Phase13EvidenceError> {
    require_options(
        options,
        &[
            "--staging-root",
            "--run-id",
            "--artifact-id",
            "--artifact-name",
            "--provider-digest",
            "--expected-producer-sha",
            "--expected-bundle-sha256",
        ],
    )?;
    for key in ["--run-id", "--artifact-id"] {
        if required(options, key)?
            .parse::<u64>()
            .ok()
            .is_none_or(|value| value == 0)
        {
            return Err(Phase13EvidenceError::usage(format!(
                "`{key}` must be a positive decimal identifier"
            )));
        }
    }
    let producer_sha = required(options, "--expected-producer-sha")?;
    let artifact_name = required(options, "--artifact-name")?;
    if !artifact_name.contains(producer_sha)
        || !artifact_name.contains(required(options, "--run-id")?)
        || !provider_digest(required(options, "--provider-digest")?)
    {
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Bundle,
            "artifact provider metadata does not bind the expected run and P",
        ));
    }
    let staging_root = absolute_path(repository_root, required(options, "--staging-root")?);
    check_bundle(
        &staging_root,
        producer_sha,
        required(options, "--expected-bundle-sha256")?,
        None,
        None,
    )
    .map_err(|error| {
        Phase13EvidenceError::new(Phase13EvidenceErrorKind::Bundle, error.to_string())
    })?;
    println!(
        "phase13 acquisition verified: run={} artifact={} name={artifact_name}",
        required(options, "--run-id")?,
        required(options, "--artifact-id")?
    );
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MaterialsManifest {
    schema_version: u32,
    target: String,
    preset: String,
    materials: Vec<MaterialEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MaterialEntry {
    kind: String,
    identity: String,
}

#[derive(Serialize)]
struct WitnessProvenance {
    schema_version: u32,
    repository_revision: String,
    oracle_revision: String,
    materials_manifest_sha256: String,
    materials_sha256: String,
    materials_count: usize,
    probe_source_sha256: String,
    compiler_id: String,
    compiler_version: String,
    target: String,
    cmake_preset: String,
    cmake_target: String,
    exact_argv: Vec<String>,
    witness_sha256: String,
}

#[derive(Serialize)]
struct ReplayEvidenceRecord {
    schema_version: u32,
    upstream_revision: String,
    resolved_scenario_path: String,
    sealed_input_sha256: String,
    native_d0_repeat_sha256: [String; 2],
    d1_oracle_identity_sha256: String,
    d1_result: String,
    diagnosis: serde_json::Value,
}

fn canonical_environment() -> Result<CanonicalEnvironment, Phase13EvidenceError> {
    if env::consts::OS != "linux" || env::consts::ARCH != "x86_64" {
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Environment,
            "canonical producer runs on x86_64 Linux only",
        ));
    }
    let rust = command_text(Command::new("rustc").arg("-vV"), "read rustc identity")?;
    let cmake = command_text(
        Command::new("cmake").arg("--version"),
        "read CMake identity",
    )?;
    let ninja = command_text(
        Command::new("ninja").arg("--version"),
        "read Ninja identity",
    )?;
    let clang = command_text(
        Command::new(env::var_os("CXX").unwrap_or_else(|| "clang++-22".into())).arg("--version"),
        "read Clang identity",
    )?;
    let rust_version = field_after(&rust, "release: ")?;
    let rust_target = field_after(&rust, "host: ")?;
    let environment = CanonicalEnvironment {
        operating_system: env::consts::OS.to_owned(),
        architecture: env::consts::ARCH.to_owned(),
        rust_target,
        rust_version,
        cmake_version: token_after(&cmake, "cmake version ")?,
        ninja_version: ninja.lines().next().unwrap_or_default().trim().to_owned(),
        clang_version: token_after(&clang, "clang version ")?,
        cmake_preset: ORACLE_PRESET.to_owned(),
    };
    let dummy = ProductionGate {
        producer_sha: "a".repeat(40),
        upstream_revision: UPSTREAM_REVISION.to_owned(),
        environment: environment.clone(),
        witness_repeat_sha256: ["a".repeat(64), "a".repeat(64)],
        native_d0_repeat_sha256: ["a".repeat(64), "a".repeat(64)],
        d1_oracle_passed: true,
        sealed_input_sha256: "a".repeat(64),
        d1_input_sha256: "a".repeat(64),
    };
    dummy.validate().map_err(|error| {
        Phase13EvidenceError::new(Phase13EvidenceErrorKind::Environment, error.to_string())
    })?;
    Ok(environment)
}

fn run_xtask(repository_root: &Path, args: &[&str]) -> Result<(), Phase13EvidenceError> {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    run_process(
        Command::new(cargo)
            .current_dir(repository_root)
            .arg("xtask")
            .args(args),
        "run nested repository orchestration",
    )
    .map(|_output| ())
}

fn require_upstream_revision(repository_root: &Path) -> Result<(), Phase13EvidenceError> {
    let revision = git_text(
        &repository_root.join("third_party/liquidfun"),
        &["rev-parse", "HEAD"],
    )?;
    if revision != UPSTREAM_REVISION {
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Git,
            "upstream checkout does not equal the pinned revision",
        ));
    }
    Ok(())
}

fn parse_options(args: &[String]) -> Result<BTreeMap<String, String>, Phase13EvidenceError> {
    if !args.len().is_multiple_of(2) {
        return Err(Phase13EvidenceError::usage(
            "every option requires exactly one value",
        ));
    }
    let mut options = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        if !pair[0].starts_with("--") || options.insert(pair[0].clone(), pair[1].clone()).is_some()
        {
            return Err(Phase13EvidenceError::usage(
                "options must be unique option/value pairs",
            ));
        }
    }
    Ok(options)
}

fn require_options(
    options: &BTreeMap<String, String>,
    required_names: &[&str],
) -> Result<(), Phase13EvidenceError> {
    require_allowed_options(options, required_names, &[])
}

fn require_allowed_options(
    options: &BTreeMap<String, String>,
    required_names: &[&str],
    optional_names: &[&str],
) -> Result<(), Phase13EvidenceError> {
    let exact = required_names
        .iter()
        .all(|name| options.contains_key(*name))
        && options.keys().all(|name| {
            required_names.contains(&name.as_str()) || optional_names.contains(&name.as_str())
        });
    if exact {
        return Ok(());
    }
    Err(Phase13EvidenceError::usage(
        "command options do not match the closed contract",
    ))
}

fn required<'a>(
    options: &'a BTreeMap<String, String>,
    name: &str,
) -> Result<&'a str, Phase13EvidenceError> {
    options
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| Phase13EvidenceError::usage(format!("missing `{name}`")))
}

fn repository_root() -> Result<PathBuf, Phase13EvidenceError> {
    let current = env::current_dir().map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Filesystem,
            format!("failed to read current directory: {error}"),
        )
    })?;
    current
        .ancestors()
        .find(|candidate| {
            candidate.join("Cargo.toml").is_file()
                && candidate.join("crates/liquidfun/Cargo.toml").is_file()
        })
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Filesystem,
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

fn lexical_absolute(path: &Path) -> Result<PathBuf, Phase13EvidenceError> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .map_err(|error| {
                Phase13EvidenceError::new(
                    Phase13EvidenceErrorKind::Filesystem,
                    format!("failed to read current directory: {error}"),
                )
            })?
            .join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(Phase13EvidenceError::new(
                        Phase13EvidenceErrorKind::Filesystem,
                        "path normalization escaped its root",
                    ));
                }
            }
        }
    }
    Ok(normalized)
}

fn git_text(repository_root: &Path, args: &[&str]) -> Result<String, Phase13EvidenceError> {
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
            Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Git,
                format!("Git returned non-UTF-8 output: {error}"),
            )
        })
}

fn run_process(command: &mut Command, action: &str) -> Result<Output, Phase13EvidenceError> {
    let output = command.output().map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Process,
            format!("failed to {action}: {error}"),
        )
    })?;
    if !output.status.success() {
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Process,
            format!(
                "failed to {action} with {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    Ok(output)
}

fn command_text(command: &mut Command, action: &str) -> Result<String, Phase13EvidenceError> {
    let output = run_process(command, action)?;
    String::from_utf8(output.stdout).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Process,
            format!("{action} returned non-UTF-8 output: {error}"),
        )
    })
}

fn field_after(text: &str, prefix: &str) -> Result<String, Phase13EvidenceError> {
    text.lines()
        .find_map(|line| line.strip_prefix(prefix))
        .map(str::to_owned)
        .ok_or_else(|| {
            Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Environment,
                format!("tool identity omitted `{prefix}`"),
            )
        })
}

fn token_after(text: &str, prefix: &str) -> Result<String, Phase13EvidenceError> {
    text.lines()
        .find_map(|line| line.split_once(prefix).map(|(_, tail)| tail))
        .and_then(|tail| tail.split_whitespace().next())
        .map(str::to_owned)
        .ok_or_else(|| {
            Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Environment,
                format!("tool identity omitted `{prefix}`"),
            )
        })
}

fn file_sha256(path: &Path) -> Result<String, Phase13EvidenceError> {
    fs::read(path).map(|bytes| sha256(&bytes)).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Filesystem,
            format!("failed to hash {}: {error}", path.display()),
        )
    })
}

fn metadata(record_class: &str, source_path: &str) -> EvidenceMetadata {
    let (derivation_kind, alteration_summary) = match record_class {
        "witness" => (
            "generated-semantic-oracle-witness",
            "Repository-authored semantic observations generated from the pinned upstream oracle without copying source, raw object memory, or Rust-produced expectations.",
        ),
        "replay_evidence" => (
            "repository-authored-replay-verification",
            "Repository-authored replay results derived from a canonical oracle bundle; no upstream source, raw object memory, or Rust-produced expectations are copied.",
        ),
        _ => (
            "repository-authored-staged-evidence-bundle",
            "Repository-authored immutable bundle metadata assembling oracle evidence and provenance records; no upstream source or raw object memory is copied.",
        ),
    };
    EvidenceMetadata {
        record_class: record_class.to_owned(),
        source_revision: UPSTREAM_REVISION.to_owned(),
        source_path: source_path.to_owned(),
        derivation_kind: derivation_kind.to_owned(),
        alteration_summary: alteration_summary.to_owned(),
        notice_refs: vec!["THIRD_PARTY_NOTICES.md".to_owned()],
    }
}

fn json_bytes(value: &impl Serialize) -> Result<Vec<u8>, Phase13EvidenceError> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            format!("failed to encode evidence record: {error}"),
        )
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn validate_relative_path(value: &str) -> Result<(), Phase13EvidenceError> {
    if !value.is_empty()
        && !value.contains('\\')
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Ok(());
    }
    Err(Phase13EvidenceError::new(
        Phase13EvidenceErrorKind::Filesystem,
        format!("unsafe producer-affecting path `{value}`"),
    ))
}

fn provider_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(valid_digest)
}

fn path_text(path: &Path) -> Result<String, Phase13EvidenceError> {
    path.to_str().map(str::to_owned).ok_or_else(|| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Filesystem,
            "path is not valid UTF-8",
        )
    })
}

pub(super) fn valid_revision(value: &str) -> bool {
    value.len() == 40 && lower_hex(value)
}

pub(super) fn valid_digest(value: &str) -> bool {
    value.len() == 64 && lower_hex(value)
}

fn lower_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
