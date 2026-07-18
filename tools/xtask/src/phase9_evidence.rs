//! Fail-closed validation for local and exact-ref Phase 9 evidence.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Display, Formatter},
    fs,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
    process::Command,
};

use liquidfun_test_protocol::{
    HarnessLimits, PHASE9_REQUIRED_BRANCH_IDS, Phase9ParticleObservation, Phase9SemanticAssertion,
    Phase9WitnessBinding, RigidWorldObservation, RigidWorldRequestRecord, RigidWorldResultRecord,
    decode_rigid_world_request_jsonl, decode_rigid_world_result_jsonl,
    validate_phase9_witness_bindings, validate_rigid_world_result_against_request,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
    let relative_dir = checked_relative_path(directory.clone())?;
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
    println!(
        "Phase 9 generated {:?} content verified before identity",
        kind
    );
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
    let canonical_dir = checked_relative_path(one(&values, "--canonical-dir")?)?;
    let sanitizer_dir = checked_relative_path(one(&values, "--sanitizer-dir")?)?;
    let maybe_run_json = values
        .get("--run-json")
        .map(|entries| {
            if entries.len() != 1 {
                return Err(usage("`--run-json` may appear only once"));
            }
            checked_relative_path(entries[0].clone())
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

fn checked_relative_path(value: String) -> Result<PathBuf, Phase9EvidenceError> {
    let path = PathBuf::from(&value);
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

fn validate_manifest(root: &Path, manifest: &EvidenceManifest) -> Result<(), Phase9EvidenceError> {
    if manifest.schema_version != 2
        || manifest.case_record_schema_version != 1
        || manifest.profile != "phase9-v1"
        || manifest.upstream_revision != UPSTREAM_REVISION
        || manifest.cases.len() != 7
    {
        return Err(Phase9EvidenceError::new(
            "manifest",
            "manifest header or case cardinality is invalid",
        ));
    }
    require_digest(
        "semantic manifest",
        &manifest.semantic_manifest_sha256,
        &canonical_sha256(&manifest.cases)?,
    )?;
    let mut case_ids = BTreeSet::new();
    let mut all_bindings = Vec::new();
    let mut all_branches = BTreeSet::new();
    let required_policies = liquidfun_differential::PHASE9_REQUIRED_POLICY_PATHS
        .iter()
        .map(|path| (*path).to_owned())
        .collect::<Vec<_>>();
    let mut maximum_actions = 0;
    let mut maximum_checkpoints = 0;
    for case in &manifest.cases {
        if !case_ids.insert(case.case_id.as_str()) || case.case_id.is_empty() {
            return Err(Phase9EvidenceError::new(
                "case",
                "case IDs must be nonempty and unique",
            ));
        }
        if case.reached_branches
            != case
                .witnesses
                .iter()
                .map(|binding| binding.branch_id.as_str().to_owned())
                .collect::<Vec<_>>()
        {
            return Err(Phase9EvidenceError::new(
                "bindings",
                format!(
                    "case `{}` branch list does not match witnesses",
                    case.case_id
                ),
            ));
        }
        for branch in &case.reached_branches {
            if !all_branches.insert(branch.as_str()) {
                return Err(Phase9EvidenceError::new(
                    "bindings",
                    format!("duplicate semantic branch `{branch}`"),
                ));
            }
        }
        if case.consumed_policy_paths != required_policies {
            return Err(Phase9EvidenceError::new(
                "policies",
                format!("case `{}` has an incomplete policy array", case.case_id),
            ));
        }
        validate_retained(&case.retained_rigid)?;
        require_digest(
            "witness binding",
            &case.witness_binding_sha256,
            &canonical_sha256(&case.witnesses)?,
        )?;
        let request_bytes =
            read_payload(root, &case.request_path, &case.request_sha256, "request")?;
        let request = decode_request(&request_bytes)?;
        let timeline = request
            .scenario()
            .timelines()
            .first()
            .ok_or_else(|| Phase9EvidenceError::new("request", "missing Phase 9 timeline"))?;
        maximum_actions = maximum_actions.max(timeline.actions().len());
        maximum_checkpoints = maximum_checkpoints.max(timeline.checkpoints().len());
        for binding in &case.witnesses {
            if binding.action_index >= timeline.actions().len()
                || binding.checkpoint_index >= timeline.checkpoints().len()
            {
                return Err(Phase9EvidenceError::new(
                    "bindings",
                    format!("case `{}` has an out-of-range binding", case.case_id),
                ));
            }
        }
        let native_bytes = read_payload(
            root,
            &case.native_result_path,
            &case.native_result_sha256,
            "native result",
        )?;
        let oracle_bytes = read_payload(
            root,
            &case.oracle_result_path,
            &case.oracle_result_sha256,
            "oracle result",
        )?;
        let native = validate_result(&request, &native_bytes, "native")?;
        let oracle = validate_result(&request, &oracle_bytes, "oracle")?;
        validate_semantic_outcomes(case, &native, "native")?;
        validate_semantic_outcomes(case, &oracle, "oracle")?;
        let comparison_bytes = read_payload(
            root,
            &case.complete_comparison_path,
            &case.complete_comparison_sha256,
            "complete comparison",
        )?;
        let comparison: CompleteComparisonPayload =
            parse_json_bytes(&comparison_bytes, "complete comparison")?;
        if comparison.outcome != "match" || comparison.consumed_policy_paths != required_policies {
            return Err(Phase9EvidenceError::new(
                "comparison",
                format!("case `{}` did not record a complete match", case.case_id),
            ));
        }
        all_bindings.extend(case.witnesses.iter().cloned());
    }
    if all_branches.len() != 58
        || all_branches != PHASE9_REQUIRED_BRANCH_IDS.lines().collect::<BTreeSet<_>>()
    {
        return Err(Phase9EvidenceError::new(
            "bindings",
            "manifest must contain exactly the 58 reviewed branches",
        ));
    }
    validate_phase9_witness_bindings(&all_bindings, maximum_actions, maximum_checkpoints)
        .map_err(|error| Phase9EvidenceError::new("bindings", error.to_string()))
}

fn validate_retained(record: &RetainedRigidRecord) -> Result<(), Phase9EvidenceError> {
    if record.comparator != "phase8-v1"
        || record.phase6_policy_sha256 != PHASE6_POLICY_SHA256
        || record.phase7_policy_sha256 != PHASE7_POLICY_SHA256
        || record.phase8_policy_sha256 != PHASE8_POLICY_SHA256
        || record.outcome != "match"
    {
        return Err(Phase9EvidenceError::new(
            "retained-rigid",
            "retained-rigid comparator, policy, or outcome mismatch",
        ));
    }
    let payload = RetainedRigidPayload {
        comparator: &record.comparator,
        phase6_policy_sha256: &record.phase6_policy_sha256,
        phase7_policy_sha256: &record.phase7_policy_sha256,
        phase8_policy_sha256: &record.phase8_policy_sha256,
        outcome: &record.outcome,
    };
    require_digest(
        "retained-rigid comparison",
        &record.comparison_sha256,
        &canonical_sha256(&payload)?,
    )
}

fn decode_request(bytes: &[u8]) -> Result<RigidWorldRequestRecord, Phase9EvidenceError> {
    decode_rigid_world_request_jsonl(bytes, &HarnessLimits::phase2_default_v1())
        .map_err(|error| Phase9EvidenceError::new("request", error.to_string()))
}

fn validate_result(
    request: &RigidWorldRequestRecord,
    bytes: &[u8],
    side: &'static str,
) -> Result<RigidWorldResultRecord, Phase9EvidenceError> {
    let mut jsonl = bytes.to_vec();
    if !jsonl.ends_with(b"\n") {
        jsonl.push(b'\n');
    }
    let result = decode_rigid_world_result_jsonl(&jsonl, &HarnessLimits::phase2_default_v1())
        .map_err(|error| Phase9EvidenceError::new(side, error.to_string()))?;
    validate_rigid_world_result_against_request(request, &result)
        .map_err(|error| Phase9EvidenceError::new(side, error.to_string()))?;
    Ok(result)
}

fn validate_semantic_outcomes(
    case: &EvidenceCaseRecord,
    result: &RigidWorldResultRecord,
    side: &'static str,
) -> Result<(), Phase9EvidenceError> {
    let timeline = result
        .timelines()
        .first()
        .ok_or_else(|| Phase9EvidenceError::new(side, "missing Phase 9 result timeline"))?;
    for binding in &case.witnesses {
        let assertion_requires_statistics = matches!(
            binding.semantic_assertion,
            Phase9SemanticAssertion::CollisionEnergyPositiveFinite { .. }
                | Phase9SemanticAssertion::StuckCandidatesNonempty { .. }
        );
        if !assertion_requires_statistics {
            continue;
        }
        let checkpoint = timeline
            .checkpoints
            .get(binding.checkpoint_index)
            .ok_or_else(|| Phase9EvidenceError::new(side, "missing bound checkpoint"))?;
        let statistics =
            checkpoint
                .observations
                .iter()
                .filter_map(|observation| match observation {
                    RigidWorldObservation::Particle {
                        observation: Phase9ParticleObservation::Statistics { statistics },
                    } => Some(statistics),
                    _ => None,
                });
        let mut saw_statistics = false;
        let matches = statistics.into_iter().any(|statistics| {
            saw_statistics = true;
            match &binding.semantic_assertion {
                Phase9SemanticAssertion::CollisionEnergyPositiveFinite { minimum_bits } => {
                    let energy = statistics.collision_energy_bits.to_f32();
                    energy.is_finite() && energy >= minimum_bits.to_f32() && energy > 0.0
                }
                Phase9SemanticAssertion::StuckCandidatesNonempty { particle_ids } => {
                    !statistics.stuck_particle_ids.is_empty()
                        && particle_ids
                            .iter()
                            .all(|particle_id| statistics.stuck_particle_ids.contains(particle_id))
                }
                _ => true,
            }
        });
        if !saw_statistics {
            return Err(Phase9EvidenceError::new(
                side,
                format!(
                    "case `{}` lacks bound statistics for `{}`",
                    case.case_id, binding.branch_id
                ),
            ));
        }
        if !matches {
            return Err(Phase9EvidenceError::new(
                side,
                format!(
                    "case `{}` does not satisfy semantic assertion `{}`",
                    case.case_id, binding.branch_id
                ),
            ));
        }
    }
    Ok(())
}

fn read_payload(
    root: &Path,
    relative: &str,
    expected_digest: &str,
    label: &'static str,
) -> Result<Vec<u8>, Phase9EvidenceError> {
    let path = checked_payload_path(root, relative)?;
    let bytes = read_regular_file(&path, label, MAXIMUM_JSON_BYTES)?;
    require_digest(label, expected_digest, &sha256(&bytes))?;
    Ok(bytes)
}

fn checked_payload_path(root: &Path, value: &str) -> Result<PathBuf, Phase9EvidenceError> {
    let relative = Path::new(value);
    if relative.is_absolute()
        || !relative.starts_with("cases")
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(Phase9EvidenceError::new(
            "path",
            format!("unsafe evidence payload path `{value}`"),
        ));
    }
    resolve_existing_descendant(root, relative, "path")
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceIdentity {
    run_id: u64,
    job: String,
    head_sha: String,
    upstream_revision: String,
    rust: String,
    cmake: String,
    ninja: String,
    clang: String,
    target: String,
    policy: String,
    trace: IdentityFile,
    manifest: IdentityFile,
    files: Vec<IdentityFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct IdentityFile {
    path: String,
    sha256: String,
}

fn validate_identity(
    root: &Path,
    kind: EvidenceKind,
    identity: &EvidenceIdentity,
    maybe_run: Option<&ExactRun>,
    denied_run_ids: &BTreeSet<u64>,
    manifest: &EvidenceManifest,
) -> Result<(), Phase9EvidenceError> {
    if denied_run_ids.contains(&identity.run_id) {
        return Err(Phase9EvidenceError::new(
            "identity",
            format!("identity run {} is denylisted", identity.run_id),
        ));
    }
    let exact_ref = maybe_run.is_some();
    if identity.job != kind.identity_job(exact_ref)
        || identity.upstream_revision != UPSTREAM_REVISION
        || identity.rust != "1.97.0"
        || identity.cmake != "4.3.3"
        || identity.ninja != "1.13.2"
        || identity.clang != "22.1.8"
        || identity.target != "x86_64-unknown-linux-gnu"
        || identity.policy != "phase9-v1"
        || manifest.profile != identity.policy
    {
        return Err(Phase9EvidenceError::new(
            "identity",
            "identity does not match the reviewed job, toolchain, target, or policy",
        ));
    }
    if let Some(run) = maybe_run {
        if identity.run_id != run.run_id || identity.head_sha != run.approved_sha {
            return Err(Phase9EvidenceError::new(
                "identity",
                "identity does not match exact-ref run and approved head",
            ));
        }
    } else if identity.run_id != 0 || identity.head_sha != "local" {
        return Err(Phase9EvidenceError::new(
            "identity",
            "local evidence must use run 0 and local head identity",
        ));
    }
    if identity.trace.path != TRACE_FILE || identity.manifest.path != MANIFEST_FILE {
        return Err(Phase9EvidenceError::new(
            "identity",
            "identity trace or manifest path mismatch",
        ));
    }
    require_file_digest(root, &identity.trace)?;
    require_file_digest(root, &identity.manifest)
}

fn validate_exact_file_set(
    root: &Path,
    manifest: &EvidenceManifest,
    identity: &EvidenceIdentity,
) -> Result<(), Phase9EvidenceError> {
    let mut expected = BTreeSet::from([
        IDENTITY_FILE.to_owned(),
        MANIFEST_FILE.to_owned(),
        TRACE_FILE.to_owned(),
        PROVENANCE_FILE.to_owned(),
        INVENTORY_FILE.to_owned(),
        READ_ONLY_FILE.to_owned(),
    ]);
    for case in &manifest.cases {
        expected.insert(case.request_path.clone());
        expected.insert(case.native_result_path.clone());
        expected.insert(case.oracle_result_path.clone());
        expected.insert(case.complete_comparison_path.clone());
    }
    let actual = regular_files(root)?;
    if actual != expected {
        return Err(Phase9EvidenceError::new(
            "files",
            format!("evidence regular-file set mismatch: expected {expected:?}, actual {actual:?}"),
        ));
    }
    let expected_identity_files = expected
        .iter()
        .filter(|path| path.as_str() != IDENTITY_FILE)
        .cloned()
        .collect::<BTreeSet<_>>();
    let actual_identity_files = identity
        .files
        .iter()
        .map(|file| file.path.clone())
        .collect::<BTreeSet<_>>();
    if identity.files.len() != actual_identity_files.len()
        || actual_identity_files != expected_identity_files
    {
        return Err(Phase9EvidenceError::new(
            "identity",
            "identity file inventory is incomplete, duplicated, or substituted",
        ));
    }
    for file in &identity.files {
        require_file_digest(root, file)?;
    }
    Ok(())
}

fn regular_files(root: &Path) -> Result<BTreeSet<String>, Phase9EvidenceError> {
    let metadata = fs::symlink_metadata(root)
        .map_err(|error| Phase9EvidenceError::new("files", error.to_string()))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(Phase9EvidenceError::new(
            "files",
            "evidence root must be an ordinary directory",
        ));
    }
    let mut pending = vec![(root.to_path_buf(), PathBuf::new())];
    let mut files = BTreeSet::new();
    while let Some((directory, relative_directory)) = pending.pop() {
        for entry in fs::read_dir(&directory)
            .map_err(|error| Phase9EvidenceError::new("files", error.to_string()))?
        {
            let entry =
                entry.map_err(|error| Phase9EvidenceError::new("files", error.to_string()))?;
            let name = entry.file_name();
            let relative = relative_directory.join(name);
            let metadata = fs::symlink_metadata(entry.path())
                .map_err(|error| Phase9EvidenceError::new("files", error.to_string()))?;
            if metadata.file_type().is_symlink() {
                return Err(Phase9EvidenceError::new(
                    "files",
                    format!("symlink `{}` is forbidden", relative.display()),
                ));
            }
            if metadata.is_dir() {
                pending.push((entry.path(), relative));
            } else if metadata.is_file() {
                files.insert(
                    relative
                        .to_str()
                        .ok_or_else(|| {
                            Phase9EvidenceError::new("files", "non-UTF-8 evidence path")
                        })?
                        .to_owned(),
                );
            } else {
                return Err(Phase9EvidenceError::new(
                    "files",
                    format!("non-regular entry `{}` is forbidden", relative.display()),
                ));
            }
        }
    }
    Ok(files)
}

fn validate_trace(path: &Path) -> Result<(), Phase9EvidenceError> {
    let bytes = read_regular_file(path, "trace", MAXIMUM_LOG_BYTES)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|error| Phase9EvidenceError::new("trace", error.to_string()))?;
    if !text.contains("test result: ok.") || text.contains("FAILED") {
        return Err(Phase9EvidenceError::new(
            "trace",
            "trace lacks a passing marker or contains FAILED",
        ));
    }
    Ok(())
}

fn require_file_digest(root: &Path, file: &IdentityFile) -> Result<(), Phase9EvidenceError> {
    let relative = Path::new(&file.path);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(Phase9EvidenceError::new(
            "identity",
            format!("unsafe identity file path `{}`", file.path),
        ));
    }
    let path = resolve_existing_descendant(root, relative, "identity")?;
    let bytes = read_regular_file(&path, "identity file", MAXIMUM_LOG_BYTES)?;
    require_digest("identity file", &file.sha256, &sha256(&bytes))
}

fn require_digest(
    label: &'static str,
    expected: &str,
    actual: &str,
) -> Result<(), Phase9EvidenceError> {
    if !is_sha256(expected) || expected != actual {
        return Err(Phase9EvidenceError::new(
            "digest",
            format!("{label} SHA-256 mismatch"),
        ));
    }
    Ok(())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn canonical_sha256(value: &impl Serialize) -> Result<String, Phase9EvidenceError> {
    serde_json::to_vec(value)
        .map(|bytes| sha256(&bytes))
        .map_err(|error| Phase9EvidenceError::new("json", error.to_string()))
}

fn read_json_absolute<T: for<'de> Deserialize<'de>>(
    path: &Path,
    label: &'static str,
    maximum: u64,
) -> Result<T, Phase9EvidenceError> {
    let bytes = read_regular_file(path, label, maximum)?;
    parse_json_bytes(&bytes, label)
}

fn parse_json_bytes<T: for<'de> Deserialize<'de>>(
    bytes: &[u8],
    label: &'static str,
) -> Result<T, Phase9EvidenceError> {
    serde_json::from_slice(bytes)
        .map_err(|error| Phase9EvidenceError::new(label, error.to_string()))
}

fn read_regular_file(
    path: &Path,
    label: &'static str,
    maximum: u64,
) -> Result<Vec<u8>, Phase9EvidenceError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        let detail = if error.kind() == ErrorKind::NotFound {
            "is missing".to_owned()
        } else {
            error.to_string()
        };
        Phase9EvidenceError::new(label, format!("{} {detail}", path.display()))
    })?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > maximum {
        return Err(Phase9EvidenceError::new(
            label,
            format!("{} must be a bounded regular file", path.display()),
        ));
    }
    fs::read(path).map_err(|error| Phase9EvidenceError::new(label, error.to_string()))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExactRun {
    repository: String,
    branch: String,
    approved_sha: String,
    head_sha: String,
    dispatched_at: String,
    run_id: u64,
    run_url: String,
    workflow_name: String,
    event: String,
    conclusion: String,
    created_at: String,
    updated_at: String,
    jobs: ExactJobs,
    artifacts: ExactArtifacts,
    live_run: LiveRun,
    live_jobs: Vec<LiveJob>,
    live_artifacts: Vec<LiveArtifact>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExactJobs {
    canonical: ExactJob,
    sanitizer: ExactJob,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExactJob {
    id: u64,
    name: String,
    url: String,
    conclusion: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExactArtifacts {
    canonical: ExactArtifact,
    sanitizer: ExactArtifact,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExactArtifact {
    id: u64,
    name: String,
    api_url: String,
    archive_download_url: String,
    digest: String,
    size_in_bytes: u64,
    expired: bool,
    created_at: String,
    expires_at: String,
    archive_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveRun {
    id: u64,
    head_sha: String,
    name: String,
    event: String,
    conclusion: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveJob {
    id: u64,
    name: String,
    conclusion: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveArtifact {
    id: u64,
    name: String,
    digest: String,
    expired: bool,
}

fn parse_exact_run(
    value: serde_json::Value,
    denied_run_ids: &BTreeSet<u64>,
) -> Result<ExactRun, Phase9EvidenceError> {
    let run: ExactRun = serde_json::from_value(value)
        .map_err(|error| Phase9EvidenceError::new("run", error.to_string()))?;
    if run.run_id == 0 || denied_run_ids.contains(&run.run_id) {
        return Err(Phase9EvidenceError::new(
            "run",
            format!("run {} is absent or denylisted", run.run_id),
        ));
    }
    if run.repository != "bright-builds-llc/liquidfun-rs"
        || run.branch != "main"
        || !is_full_sha(&run.approved_sha)
        || run.approved_sha != run.head_sha
        || run.workflow_name != "Oracle CI"
        || run.event != "workflow_dispatch"
        || run.conclusion != "success"
    {
        return Err(Phase9EvidenceError::new(
            "run",
            "run does not match the approved head or Oracle CI dispatch authority",
        ));
    }
    let _metadata = (
        &run.dispatched_at,
        &run.run_url,
        &run.created_at,
        &run.updated_at,
    );
    validate_exact_job(
        &run.jobs.canonical,
        "Canonical Linux oracle",
        &run.live_jobs,
    )?;
    validate_exact_job(
        &run.jobs.sanitizer,
        "Scheduled fail-fast sanitizer and reset corpus",
        &run.live_jobs,
    )?;
    if run.jobs.canonical.id == run.jobs.sanitizer.id {
        return Err(Phase9EvidenceError::new(
            "jobs",
            "canonical and sanitizer job IDs must be unique",
        ));
    }
    validate_exact_artifact(
        run.run_id,
        &run.approved_sha,
        EvidenceKind::Canonical,
        &run.artifacts.canonical,
        &run.live_artifacts,
    )?;
    validate_exact_artifact(
        run.run_id,
        &run.approved_sha,
        EvidenceKind::Sanitizer,
        &run.artifacts.sanitizer,
        &run.live_artifacts,
    )?;
    if run.artifacts.canonical.id == run.artifacts.sanitizer.id {
        return Err(Phase9EvidenceError::new(
            "artifacts",
            "canonical and sanitizer artifact IDs must be unique",
        ));
    }
    if run.live_jobs.len() != 2 || run.live_artifacts.len() != 2 {
        return Err(Phase9EvidenceError::new(
            "run",
            "live metadata must contain exactly two jobs and two artifacts",
        ));
    }
    if run.live_run.id != run.run_id
        || run.live_run.head_sha != run.approved_sha
        || run.live_run.name != run.workflow_name
        || run.live_run.event != run.event
        || run.live_run.conclusion != run.conclusion
    {
        return Err(Phase9EvidenceError::new(
            "run",
            "live run snapshot does not match run.json",
        ));
    }
    Ok(run)
}

fn validate_exact_job(
    job: &ExactJob,
    expected_name: &str,
    live_jobs: &[LiveJob],
) -> Result<(), Phase9EvidenceError> {
    let _url = &job.url;
    if job.name != expected_name || job.conclusion != "success" {
        return Err(Phase9EvidenceError::new(
            "jobs",
            format!("required successful job `{expected_name}` is absent"),
        ));
    }
    let matches = live_jobs
        .iter()
        .filter(|live| {
            live.id == job.id && live.name == job.name && live.conclusion == job.conclusion
        })
        .count();
    if matches != 1
        || live_jobs
            .iter()
            .filter(|live| live.name == expected_name)
            .count()
            != 1
    {
        return Err(Phase9EvidenceError::new(
            "jobs",
            format!("live job `{expected_name}` is missing or duplicated"),
        ));
    }
    Ok(())
}

fn validate_exact_artifact(
    run_id: u64,
    approved_sha: &str,
    kind: EvidenceKind,
    artifact: &ExactArtifact,
    live_artifacts: &[LiveArtifact],
) -> Result<(), Phase9EvidenceError> {
    let expected_name = format!("{}-{run_id}-{approved_sha}", kind.artifact_prefix());
    let _metadata = (
        &artifact.api_url,
        &artifact.archive_download_url,
        artifact.size_in_bytes,
        &artifact.created_at,
        &artifact.expires_at,
    );
    if artifact.name != expected_name
        || artifact.expired
        || artifact
            .digest
            .strip_prefix("sha256:")
            .is_none_or(|digest| !is_sha256(digest))
    {
        return Err(Phase9EvidenceError::new(
            "artifacts",
            format!("artifact `{expected_name}` is absent, expired, or malformed"),
        ));
    }
    let matches = live_artifacts
        .iter()
        .filter(|live| {
            live.id == artifact.id
                && live.name == artifact.name
                && live.digest == artifact.digest
                && live.expired == artifact.expired
        })
        .count();
    if matches != 1
        || live_artifacts
            .iter()
            .filter(|live| live.name == expected_name)
            .count()
            != 1
    {
        return Err(Phase9EvidenceError::new(
            "artifacts",
            format!("live artifact `{expected_name}` is missing or duplicated"),
        ));
    }
    Ok(())
}

fn validate_archive(
    repository_root: &Path,
    kind: EvidenceKind,
    run: &ExactRun,
    manifest: &EvidenceManifest,
) -> Result<(), Phase9EvidenceError> {
    let artifact = match kind {
        EvidenceKind::Canonical => &run.artifacts.canonical,
        EvidenceKind::Sanitizer => &run.artifacts.sanitizer,
    };
    let archive_relative = checked_relative_path(artifact.archive_path.clone())?;
    let archive = resolve_existing_target_path(repository_root, &archive_relative, "archive")?;
    let bytes = read_regular_file(&archive, "archive", MAXIMUM_LOG_BYTES)?;
    if u64::try_from(bytes.len()).ok() != Some(artifact.size_in_bytes) {
        return Err(Phase9EvidenceError::new(
            "archive",
            "archive size does not match recorded artifact metadata",
        ));
    }
    let expected = artifact
        .digest
        .strip_prefix("sha256:")
        .expect("artifact digest validated during run parsing");
    require_digest("archive", expected, &sha256(&bytes))?;
    let output = Command::new("unzip")
        .arg("-Z1")
        .arg(&archive)
        .output()
        .map_err(|error| Phase9EvidenceError::new("archive", error.to_string()))?;
    if !output.status.success() {
        return Err(Phase9EvidenceError::new(
            "archive",
            "unzip could not inspect the artifact archive",
        ));
    }
    let entries = std::str::from_utf8(&output.stdout)
        .map_err(|error| Phase9EvidenceError::new("archive", error.to_string()))?;
    let expected_files = expected_evidence_files(manifest);
    let mut archive_files = BTreeSet::new();
    for entry in entries.lines() {
        if entry.ends_with('/') {
            validate_archive_entry(entry.trim_end_matches('/'))?;
            continue;
        }
        validate_archive_entry(entry)?;
        archive_files.insert(entry.to_owned());
    }
    if archive_files != expected_files {
        return Err(Phase9EvidenceError::new(
            "archive",
            "archive entries do not match extracted evidence files",
        ));
    }
    let modes = Command::new("unzip")
        .args(["-Z", "-l"])
        .arg(&archive)
        .output()
        .map_err(|error| Phase9EvidenceError::new("archive", error.to_string()))?;
    if !modes.status.success()
        || std::str::from_utf8(&modes.stdout)
            .map_err(|error| Phase9EvidenceError::new("archive", error.to_string()))?
            .lines()
            .any(|line| line.starts_with('l'))
    {
        return Err(Phase9EvidenceError::new(
            "archive",
            "archive contains a symlink or unreadable mode listing",
        ));
    }
    Ok(())
}

fn validate_archive_entry(entry: &str) -> Result<(), Phase9EvidenceError> {
    let path = Path::new(entry);
    if entry.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(Phase9EvidenceError::new(
            "archive",
            format!("unsafe archive entry `{entry}`"),
        ));
    }
    Ok(())
}

fn expected_evidence_files(manifest: &EvidenceManifest) -> BTreeSet<String> {
    let mut files = BTreeSet::from([
        IDENTITY_FILE.to_owned(),
        MANIFEST_FILE.to_owned(),
        TRACE_FILE.to_owned(),
        PROVENANCE_FILE.to_owned(),
        INVENTORY_FILE.to_owned(),
        READ_ONLY_FILE.to_owned(),
    ]);
    for case in &manifest.cases {
        files.insert(case.request_path.clone());
        files.insert(case.native_result_path.clone());
        files.insert(case.oracle_result_path.clone());
        files.insert(case.complete_comparison_path.clone());
    }
    files
}

fn is_full_sha(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
