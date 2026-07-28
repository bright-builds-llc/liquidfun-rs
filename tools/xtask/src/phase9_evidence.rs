//! Fail-closed validation for local and exact-ref Phase 9 evidence.

#[path = "phase9_evidence/archive.rs"]
mod archive;
#[path = "phase9_evidence/identity.rs"]
mod identity;
#[path = "phase9_evidence/manifest.rs"]
mod manifest;

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Display, Formatter},
    fs,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
    process::Command,
};

use liquidfun_differential::{
    Phase9ComparisonOutcome, Phase9CrossRunProof, Phase9CrossRunProofRecord,
    Phase9EvidencePayloadRef, compare_complete_phase9_rigid_world_results,
    validate_phase9_cross_run_proofs, validate_phase9_evidence_bindings,
};
use liquidfun_test_protocol::{
    HarnessLimits, PHASE9_REQUIRED_BRANCH_IDS, Phase9WitnessBinding, RigidWorldRequestRecord,
    RigidWorldResultRecord, decode_rigid_world_request_jsonl, decode_rigid_world_result_jsonl,
    validate_phase9_witness_bindings, validate_rigid_world_result_against_request,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use archive::{ExactRun, expected_evidence_files, parse_exact_run, validate_archive};
use identity::{
    EvidenceIdentity, canonical_sha256, is_sha256, parse_json_bytes, read_json_absolute,
    read_regular_file, regular_files, require_digest, sha256, validate_exact_file_set,
    validate_identity, validate_trace,
};
use manifest::{cross_run_payload_refs, validate_manifest};

const USAGE: &str = "Usage: cargo xtask phase9-evidence validate \
    --mode <local|exact-ref> --canonical-dir <target/path> \
    --sanitizer-dir <target/path> [--run-json <target/path>] \
    [--deny-run-id <id>]...\n       cargo xtask phase9-evidence \
    validate-content <canonical|sanitizer> <target/path>";
const MANIFEST_FILE: &str = "phase9-manifest.json";
const IDENTITY_FILE: &str = "identity.json";
const TRACE_FILE: &str = "phase9-trace.log";
const PROVENANCE_FILE: &str = "provenance.log";
const INVENTORY_FILE: &str = "inventory.log";
const READ_ONLY_FILE: &str = "read-only.log";
const UPSTREAM_REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const PHASE6_POLICY_SHA256: &str =
    "7f10df148852866fd20d11b8d27adcddc0ad463ac3d3d716a8946ca5c8f1c63a";
const PHASE7_POLICY_SHA256: &str =
    "fd772b2cf523a6d40bf978bc4d0da18a4564181a93e6b2bdeb8e4d40d5613311";
const PHASE8_POLICY_SHA256: &str =
    "2843ca40bec5b1c680135664c58c12a8388a7a9e86ad77f8ef5a268f3f15a6bf";
const MAXIMUM_JSON_BYTES: u64 = 16 * 1024 * 1024;
const MAXIMUM_LOG_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Phase9EvidenceError {
    category: &'static str,
    message: String,
}

impl Phase9EvidenceError {
    fn new(category: &'static str, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
        }
    }
}

impl Display for Phase9EvidenceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "phase9-evidence/{}: {}",
            self.category, self.message
        )
    }
}

impl std::error::Error for Phase9EvidenceError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValidationMode {
    Local,
    ExactRef,
}

#[derive(Debug)]
struct ValidationOptions {
    mode: ValidationMode,
    canonical_dir: PathBuf,
    sanitizer_dir: PathBuf,
    maybe_run_json: Option<PathBuf>,
    denied_run_ids: BTreeSet<u64>,
}

pub(crate) fn run(args: &[String]) -> Result<(), Phase9EvidenceError> {
    if args
        .first()
        .is_some_and(|value| value == "validate-content")
    {
        return validate_generated_content(args);
    }
    let options = parse_options(args)?;
    let repository_root = repository_root()?;
    let maybe_run = if options.mode == ValidationMode::ExactRef {
        let run_path = options
            .maybe_run_json
            .as_ref()
            .expect("exact-ref parsing requires run-json");
        let run_path = resolve_existing_target_path(&repository_root, run_path, "run-json")?;
        let run_value: serde_json::Value =
            read_json_absolute(&run_path, "run-json", MAXIMUM_JSON_BYTES)?;
        let run_id = run_value
            .get("run_id")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| Phase9EvidenceError::new("run", "run_id is absent or invalid"))?;
        if options.denied_run_ids.contains(&run_id) {
            return Err(Phase9EvidenceError::new(
                "run",
                format!("run {run_id} is denylisted"),
            ));
        }
        Some(parse_exact_run(run_value, &options.denied_run_ids)?)
    } else {
        None
    };

    let canonical = validate_directory(
        &repository_root,
        &options.canonical_dir,
        EvidenceKind::Canonical,
        maybe_run.as_ref(),
        &options.denied_run_ids,
    )?;
    let sanitizer = validate_directory(
        &repository_root,
        &options.sanitizer_dir,
        EvidenceKind::Sanitizer,
        maybe_run.as_ref(),
        &options.denied_run_ids,
    )?;
    if canonical.manifest != sanitizer.manifest {
        return Err(Phase9EvidenceError::new(
            "semantic-manifest",
            "canonical and sanitizer semantic manifests differ",
        ));
    }
    if canonical.identity.job == sanitizer.identity.job {
        return Err(Phase9EvidenceError::new(
            "identity",
            "canonical evidence cannot substitute for sanitizer evidence",
        ));
    }
    println!(
        "Phase 9 evidence verified: 7 cases, 58 semantic bindings, mode {:?}",
        options.mode
    );
    Ok(())
}

fn validate_generated_content(args: &[String]) -> Result<(), Phase9EvidenceError> {
    let [_, kind, directory] = args else {
        return Err(usage(
            "`validate-content` requires one evidence kind and directory",
        ));
    };
    let kind = match kind.as_str() {
        "canonical" => EvidenceKind::Canonical,
        "sanitizer" => EvidenceKind::Sanitizer,
        value => return Err(usage(format!("unsupported evidence kind `{value}`"))),
    };
    let relative_dir = checked_relative_path(directory)?;
    let repository_root = repository_root()?;
    let root = resolve_existing_target_path(&repository_root, &relative_dir, "evidence root")?;
    let manifest: EvidenceManifest =
        read_json_absolute(&root.join(MANIFEST_FILE), "manifest", MAXIMUM_JSON_BYTES)?;
    validate_manifest(&root, &manifest)?;
    validate_trace(&root.join(TRACE_FILE))?;
    let mut expected = expected_evidence_files(&manifest);
    expected.remove(IDENTITY_FILE);
    let actual = regular_files(&root)?;
    if actual != expected {
        return Err(Phase9EvidenceError::new(
            "files",
            format!(
                "generated evidence file set mismatch: expected {expected:?}, actual {actual:?}"
            ),
        ));
    }
    println!("Phase 9 generated {kind:?} content verified before identity");
    Ok(())
}

fn parse_options(args: &[String]) -> Result<ValidationOptions, Phase9EvidenceError> {
    let Some((subcommand, rest)) = args.split_first() else {
        return Err(usage("missing phase9-evidence subcommand"));
    };
    if subcommand != "validate" {
        return Err(usage(format!(
            "unknown phase9-evidence subcommand `{subcommand}`"
        )));
    }
    let mut values = BTreeMap::<String, Vec<String>>::new();
    let mut index = 0;
    while index < rest.len() {
        let option = &rest[index];
        if !option.starts_with("--") {
            return Err(usage(format!("unexpected argument `{option}`")));
        }
        let Some(value) = rest.get(index + 1) else {
            return Err(usage(format!("missing value for `{option}`")));
        };
        if value.starts_with("--") {
            return Err(usage(format!("missing value for `{option}`")));
        }
        if !matches!(
            option.as_str(),
            "--mode" | "--canonical-dir" | "--sanitizer-dir" | "--run-json" | "--deny-run-id"
        ) {
            return Err(usage(format!("unknown option `{option}`")));
        }
        values
            .entry(option.clone())
            .or_default()
            .push(value.clone());
        index += 2;
    }

    let mode = match one(&values, "--mode")?.as_str() {
        "local" => ValidationMode::Local,
        "exact-ref" => ValidationMode::ExactRef,
        value => return Err(usage(format!("unsupported validation mode `{value}`"))),
    };
    let canonical_dir = checked_relative_path(&one(&values, "--canonical-dir")?)?;
    let sanitizer_dir = checked_relative_path(&one(&values, "--sanitizer-dir")?)?;
    let maybe_run_json = values
        .get("--run-json")
        .map(|entries| {
            if entries.len() != 1 {
                return Err(usage("`--run-json` may appear only once"));
            }
            checked_relative_path(&entries[0])
        })
        .transpose()?;
    if mode == ValidationMode::Local && maybe_run_json.is_some() {
        return Err(usage("`--run-json` is exact-ref-only"));
    }
    if mode == ValidationMode::ExactRef && maybe_run_json.is_none() {
        return Err(usage("exact-ref mode requires `--run-json`"));
    }
    let denied_run_ids = values
        .get("--deny-run-id")
        .into_iter()
        .flatten()
        .map(|value| {
            value
                .parse::<u64>()
                .map_err(|_| usage(format!("invalid run ID `{value}`")))
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    Ok(ValidationOptions {
        mode,
        canonical_dir,
        sanitizer_dir,
        maybe_run_json,
        denied_run_ids,
    })
}

fn one(
    values: &BTreeMap<String, Vec<String>>,
    option: &str,
) -> Result<String, Phase9EvidenceError> {
    let Some(entries) = values.get(option) else {
        return Err(usage(format!("missing required option `{option}`")));
    };
    if entries.len() != 1 {
        return Err(usage(format!("`{option}` may appear only once")));
    }
    Ok(entries[0].clone())
}

fn usage(message: impl Into<String>) -> Phase9EvidenceError {
    Phase9EvidenceError::new("usage", format!("{}\n\n{USAGE}", message.into()))
}

fn checked_relative_path(value: &str) -> Result<PathBuf, Phase9EvidenceError> {
    let path = PathBuf::from(value);
    if path.is_absolute()
        || !path.starts_with("target")
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(usage(format!(
            "path `{value}` must be normalized, relative, and under target/"
        )));
    }
    Ok(path)
}

fn repository_root() -> Result<PathBuf, Phase9EvidenceError> {
    let current = std::env::current_dir()
        .map_err(|error| Phase9EvidenceError::new("root", error.to_string()))?;
    let root = current
        .ancestors()
        .find(|candidate| candidate.join("Cargo.toml").is_file())
        .map(Path::to_path_buf)
        .ok_or_else(|| Phase9EvidenceError::new("root", "workspace root not found"))?;
    fs::canonicalize(root).map_err(|error| Phase9EvidenceError::new("root", error.to_string()))
}

fn resolve_existing_target_path(
    repository_root: &Path,
    relative: &Path,
    label: &'static str,
) -> Result<PathBuf, Phase9EvidenceError> {
    let target_relative = relative
        .strip_prefix("target")
        .map_err(|_| Phase9EvidenceError::new(label, "path must remain beneath the target root"))?;
    let target_root = resolve_existing_descendant(repository_root, Path::new("target"), label)?;
    let path = resolve_existing_descendant(&target_root, target_relative, label)?;
    if !path.starts_with(&target_root) {
        return Err(Phase9EvidenceError::new(
            label,
            "path escapes the canonical target root",
        ));
    }
    Ok(path)
}

fn resolve_existing_descendant(
    root: &Path,
    relative: &Path,
    label: &'static str,
) -> Result<PathBuf, Phase9EvidenceError> {
    let root_metadata = fs::symlink_metadata(root)
        .map_err(|error| Phase9EvidenceError::new(label, error.to_string()))?;
    if root_metadata.file_type().is_symlink() {
        return Err(Phase9EvidenceError::new(
            label,
            format!("symlink component `{}` is forbidden", root.display()),
        ));
    }
    let canonical_root = fs::canonicalize(root)
        .map_err(|error| Phase9EvidenceError::new(label, error.to_string()))?;
    let mut current = canonical_root.clone();
    for component in relative.components() {
        let Component::Normal(component) = component else {
            return Err(Phase9EvidenceError::new(
                label,
                format!("unsafe path component in `{}`", relative.display()),
            ));
        };
        current.push(component);
        let metadata = fs::symlink_metadata(&current)
            .map_err(|error| Phase9EvidenceError::new(label, error.to_string()))?;
        if metadata.file_type().is_symlink() {
            return Err(Phase9EvidenceError::new(
                label,
                format!("symlink component `{}` is forbidden", current.display()),
            ));
        }
    }
    let canonical = fs::canonicalize(&current)
        .map_err(|error| Phase9EvidenceError::new(label, error.to_string()))?;
    if !canonical.starts_with(&canonical_root) {
        return Err(Phase9EvidenceError::new(
            label,
            format!("path `{}` escapes its canonical root", current.display()),
        ));
    }
    Ok(canonical)
}

#[derive(Debug, Clone, Copy)]
enum EvidenceKind {
    Canonical,
    Sanitizer,
}

impl EvidenceKind {
    const fn identity_job(self, exact_ref: bool) -> &'static str {
        match (self, exact_ref) {
            (Self::Canonical, true) => "canonical-linux",
            (Self::Sanitizer, true) => "sanitizer-linux",
            (Self::Canonical, false) => "canonical-local",
            (Self::Sanitizer, false) => "sanitizer-local",
        }
    }

    const fn artifact_prefix(self) -> &'static str {
        match self {
            Self::Canonical => "phase9-canonical",
            Self::Sanitizer => "phase9-sanitizer",
        }
    }
}

#[derive(Debug)]
struct ValidatedDirectory {
    manifest: EvidenceManifest,
    identity: EvidenceIdentity,
}

fn validate_directory(
    repository_root: &Path,
    relative_dir: &Path,
    kind: EvidenceKind,
    maybe_run: Option<&ExactRun>,
    denied_run_ids: &BTreeSet<u64>,
) -> Result<ValidatedDirectory, Phase9EvidenceError> {
    let root = resolve_existing_target_path(repository_root, relative_dir, "evidence root")?;
    let manifest: EvidenceManifest =
        read_json_absolute(&root.join(MANIFEST_FILE), "manifest", MAXIMUM_JSON_BYTES)?;
    validate_manifest(&root, &manifest)?;
    let identity: EvidenceIdentity =
        read_json_absolute(&root.join(IDENTITY_FILE), "identity", MAXIMUM_JSON_BYTES)?;
    validate_identity(&root, kind, &identity, maybe_run, denied_run_ids, &manifest)?;
    validate_exact_file_set(&root, &manifest, &identity)?;
    validate_trace(&root.join(TRACE_FILE))?;
    if let Some(run) = maybe_run {
        validate_archive(repository_root, kind, run, &manifest)?;
    }
    Ok(ValidatedDirectory { manifest, identity })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceManifest {
    schema_version: u32,
    case_record_schema_version: u32,
    profile: String,
    upstream_revision: String,
    semantic_manifest_sha256: String,
    cases: Vec<EvidenceCaseRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceCaseRecord {
    case_id: String,
    reached_branches: Vec<String>,
    witnesses: Vec<Phase9WitnessBinding>,
    witness_binding_sha256: String,
    consumed_policy_paths: Vec<String>,
    retained_rigid: RetainedRigidRecord,
    request_path: String,
    request_sha256: String,
    native_result_path: String,
    native_result_sha256: String,
    oracle_result_path: String,
    oracle_result_sha256: String,
    complete_comparison_path: String,
    complete_comparison_sha256: String,
    cross_run_proofs: Vec<Phase9CrossRunProofRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RetainedRigidRecord {
    comparator: String,
    phase6_policy_sha256: String,
    phase7_policy_sha256: String,
    phase8_policy_sha256: String,
    outcome: String,
    comparison_sha256: String,
}

#[derive(Serialize)]
struct RetainedRigidPayload<'a> {
    comparator: &'a str,
    phase6_policy_sha256: &'a str,
    phase7_policy_sha256: &'a str,
    phase8_policy_sha256: &'a str,
    outcome: &'a str,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CompleteComparisonPayload {
    outcome: String,
    consumed_policy_paths: Vec<String>,
}
