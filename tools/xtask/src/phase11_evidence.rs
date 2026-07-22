//! Fail-closed local and exact-reference Phase 11 evidence validation.

mod authority;
mod content;
mod paths;

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
};

use authority::{ExactRun, parse_exact_run, validate_exact_pair};
use content::{
    AcceptedContent, EvidenceKind, GENERATOR_VERSION, IDENTITY_FILE, PROTOCOL_VERSION,
    UPSTREAM_REVISION, evaluate_directory, evaluate_generated_before_identity,
    render_source_records,
};
use paths::{
    MAX_JSON_BYTES, checked_input_path, checked_target_path, read_json, read_regular,
    repository_root, require_sha256, resolve_input, sha256,
};
use serde::Deserialize;

const USAGE: &str = "Usage: cargo xtask phase11-evidence validate \
    --mode <local|exact-ref> --canonical-dir <path> --sanitizer-dir <path> \
    [--run-json <target/path>] [--deny-run-id <id>]... \
    [--deny-artifact-id <id>]...\n       cargo xtask phase11-evidence \
    validate-content <canonical|sanitizer> <target/path>\n       cargo xtask \
    phase11-evidence render-records <debug|release|replay|sanitizer>";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Phase11EvidenceError {
    category: &'static str,
    message: String,
}

impl Phase11EvidenceError {
    pub(super) fn new(category: &'static str, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
        }
    }
}

impl Display for Phase11EvidenceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "phase11-evidence/{}: {}",
            self.category, self.message
        )
    }
}

impl std::error::Error for Phase11EvidenceError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValidationMode {
    Local,
    ExactRef,
}

#[derive(Debug)]
struct Options {
    mode: ValidationMode,
    canonical_dir: PathBuf,
    sanitizer_dir: PathBuf,
    maybe_run_json: Option<PathBuf>,
    denied_run_ids: BTreeSet<u64>,
    denied_artifact_ids: BTreeSet<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct EvidenceIdentity {
    schema_version: u32,
    pub(super) mode: String,
    pub(super) run_id: u64,
    pub(super) head_sha: String,
    pub(super) job_name: String,
    pub(super) artifact_id: u64,
    pub(super) artifact_name: String,
    pub(super) platform: String,
    pub(super) rust_version: String,
    pub(super) clang_version: String,
    upstream_revision: String,
    protocol_version: String,
    generator_version: String,
    pub(super) semantic_sha256: String,
    files: Vec<FileReference>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileReference {
    path: String,
    sha256: String,
}

pub(crate) fn run(args: &[String]) -> Result<(), Phase11EvidenceError> {
    if args
        .first()
        .is_some_and(|argument| argument == "render-records")
    {
        return render_records_command(args);
    }
    if args
        .first()
        .is_some_and(|argument| argument == "validate-content")
    {
        return validate_content_command(args);
    }
    let options = parse_options(args)?;
    let root = repository_root()?;
    let maybe_run = load_exact_run(&root, &options)?;
    let canonical = evaluate_directory(&root, &options.canonical_dir)?;
    let sanitizer = evaluate_directory(&root, &options.sanitizer_dir)?;
    if canonical.semantic_sha256 != sanitizer.semantic_sha256 {
        return Err(Phase11EvidenceError::new(
            "semantic",
            "canonical and sanitizer semantic content differs",
        ));
    }
    let maybe_canonical_identity = load_identity(&canonical)?;
    let maybe_sanitizer_identity = load_identity(&sanitizer)?;
    match options.mode {
        ValidationMode::Local => {
            validate_local_identity(
                &canonical,
                maybe_canonical_identity.as_ref(),
                EvidenceKind::Canonical,
            )?;
            validate_local_identity(
                &sanitizer,
                maybe_sanitizer_identity.as_ref(),
                EvidenceKind::Sanitizer,
            )?;
            println!("Phase 11 evidence verified: 3 cases, local D2 non-promotable authority");
        }
        ValidationMode::ExactRef => {
            let run = maybe_run
                .as_ref()
                .expect("exact-ref option parsing requires a run envelope");
            let canonical_identity = maybe_canonical_identity.as_ref().ok_or_else(|| {
                Phase11EvidenceError::new("identity", "canonical identity is missing")
            })?;
            let sanitizer_identity = maybe_sanitizer_identity.as_ref().ok_or_else(|| {
                Phase11EvidenceError::new("identity", "sanitizer identity is missing")
            })?;
            validate_exact_pair(
                &root,
                run,
                &canonical,
                canonical_identity,
                &sanitizer,
                sanitizer_identity,
                &options.denied_artifact_ids,
            )?;
            println!("Phase 11 evidence verified: 3 cases, exact-ref same-run D1 authority");
        }
    }
    Ok(())
}

fn validate_content_command(args: &[String]) -> Result<(), Phase11EvidenceError> {
    let [_, kind, directory] = args else {
        return Err(usage(
            "`validate-content` requires one evidence kind and directory",
        ));
    };
    let kind = EvidenceKind::parse(kind)?;
    let relative = checked_target_path(directory)?;
    let root = repository_root()?;
    let accepted = evaluate_generated_before_identity(&root, &relative)?;
    println!(
        "Phase 11 generated {kind:?} content verified before identity; semantic-sha256={}",
        accepted.semantic_sha256
    );
    Ok(())
}

fn render_records_command(args: &[String]) -> Result<(), Phase11EvidenceError> {
    let [_, role] = args else {
        return Err(usage("`render-records` requires one closed proof role"));
    };
    let root = repository_root()?;
    for record in render_source_records(&root, role)? {
        println!("{record}");
    }
    Ok(())
}

fn load_identity(
    content: &AcceptedContent,
) -> Result<Option<EvidenceIdentity>, Phase11EvidenceError> {
    if content.source_only {
        return Ok(None);
    }
    let identity: EvidenceIdentity = read_json(&content.root.join(IDENTITY_FILE), "identity")?;
    if identity.schema_version != 1
        || identity.upstream_revision != UPSTREAM_REVISION
        || identity.protocol_version != PROTOCOL_VERSION
        || identity.generator_version != GENERATOR_VERSION
        || identity.semantic_sha256 != content.semantic_sha256
    {
        return Err(Phase11EvidenceError::new(
            "identity",
            "identity schema or semantic provenance differs from accepted content",
        ));
    }
    let expected = content
        .expected_files
        .iter()
        .filter(|path| path.as_str() != IDENTITY_FILE)
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let actual = identity
        .files
        .iter()
        .map(|file| file.path.as_str())
        .collect::<BTreeSet<_>>();
    if actual != expected || identity.files.len() != expected.len() {
        return Err(Phase11EvidenceError::new(
            "identity",
            "identity inventory is incomplete, duplicated, or contains extras",
        ));
    }
    for file in &identity.files {
        let path = resolve_input(
            &content.root,
            checked_input_path(&file.path)?.as_path(),
            "identity",
        )?;
        require_sha256(
            "identity file",
            &file.sha256,
            &sha256(&read_regular(&path, "identity file", MAX_JSON_BYTES)?),
        )?;
    }
    Ok(Some(identity))
}

fn validate_local_identity(
    content: &AcceptedContent,
    maybe_identity: Option<&EvidenceIdentity>,
    kind: EvidenceKind,
) -> Result<(), Phase11EvidenceError> {
    if content.source_only {
        if maybe_identity.is_some() {
            return Err(Phase11EvidenceError::new(
                "identity",
                "tracked source corpus unexpectedly carries authority",
            ));
        }
        return Ok(());
    }
    let identity = maybe_identity
        .ok_or_else(|| Phase11EvidenceError::new("identity", "local identity is missing"))?;
    if identity.mode != "local"
        || identity.run_id != 0
        || identity.head_sha != "local"
        || identity.job_name != kind.local_name()
        || identity.artifact_id != 0
        || identity.artifact_name != kind.local_name()
        || identity.platform != "local"
        || identity.rust_version != "local"
        || identity.clang_version != "local"
    {
        return Err(Phase11EvidenceError::new(
            "identity",
            "local evidence carries promotable or substituted authority",
        ));
    }
    Ok(())
}

fn load_exact_run(
    root: &Path,
    options: &Options,
) -> Result<Option<ExactRun>, Phase11EvidenceError> {
    if options.mode == ValidationMode::Local {
        return Ok(None);
    }
    let relative = options
        .maybe_run_json
        .as_ref()
        .expect("exact-ref options require run-json");
    let path = resolve_input(root, relative, "run-json")?;
    let value: serde_json::Value = read_json(&path, "run-json")?;
    let run_id = value
        .get("run_id")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| Phase11EvidenceError::new("run", "run_id is absent or invalid"))?;
    if options.denied_run_ids.contains(&run_id) {
        return Err(Phase11EvidenceError::new(
            "run",
            format!("run {run_id} is denylisted"),
        ));
    }
    parse_exact_run(value, &options.denied_run_ids).map(Some)
}

fn parse_options(args: &[String]) -> Result<Options, Phase11EvidenceError> {
    let Some((command, rest)) = args.split_first() else {
        return Err(usage("missing phase11-evidence subcommand"));
    };
    if command != "validate" {
        return Err(usage(format!("unknown subcommand `{command}`")));
    }
    let mut values = BTreeMap::<String, Vec<String>>::new();
    for pair in rest.chunks(2) {
        let [option, value] = pair else {
            return Err(usage("every option requires a value"));
        };
        if !matches!(
            option.as_str(),
            "--mode"
                | "--canonical-dir"
                | "--sanitizer-dir"
                | "--run-json"
                | "--deny-run-id"
                | "--deny-artifact-id"
        ) || value.starts_with("--")
        {
            return Err(usage(format!("unknown or valueless option `{option}`")));
        }
        values
            .entry(option.clone())
            .or_default()
            .push(value.clone());
    }
    let mode = match one(&values, "--mode")?.as_str() {
        "local" => ValidationMode::Local,
        "exact-ref" => ValidationMode::ExactRef,
        value => return Err(usage(format!("unsupported mode `{value}`"))),
    };
    let maybe_run_json = optional_one(&values, "--run-json")?
        .map(|value| checked_target_path(&value))
        .transpose()?;
    if (mode == ValidationMode::ExactRef) != maybe_run_json.is_some() {
        return Err(usage(
            "exact-ref requires run-json and local mode forbids it",
        ));
    }
    Ok(Options {
        mode,
        canonical_dir: checked_input_path(&one(&values, "--canonical-dir")?)?,
        sanitizer_dir: checked_input_path(&one(&values, "--sanitizer-dir")?)?,
        maybe_run_json,
        denied_run_ids: parse_ids(&values, "--deny-run-id")?,
        denied_artifact_ids: parse_ids(&values, "--deny-artifact-id")?,
    })
}

fn one(
    values: &BTreeMap<String, Vec<String>>,
    option: &str,
) -> Result<String, Phase11EvidenceError> {
    optional_one(values, option)?.ok_or_else(|| usage(format!("missing `{option}`")))
}

fn optional_one(
    values: &BTreeMap<String, Vec<String>>,
    option: &str,
) -> Result<Option<String>, Phase11EvidenceError> {
    let Some(entries) = values.get(option) else {
        return Ok(None);
    };
    if entries.len() != 1 {
        return Err(usage(format!("`{option}` may appear only once")));
    }
    Ok(entries.first().cloned())
}

fn parse_ids(
    values: &BTreeMap<String, Vec<String>>,
    option: &str,
) -> Result<BTreeSet<u64>, Phase11EvidenceError> {
    values
        .get(option)
        .into_iter()
        .flatten()
        .map(|value| {
            value
                .parse()
                .map_err(|_| usage(format!("invalid ID `{value}`")))
        })
        .collect()
}

fn usage(message: impl Into<String>) -> Phase11EvidenceError {
    Phase11EvidenceError::new("usage", format!("{}\n\n{USAGE}", message.into()))
}
