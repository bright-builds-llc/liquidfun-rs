//! Fail-closed validation for local and exact-ref Phase 10 evidence.

mod authority;
mod content;
mod paths;

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

use authority::{ExactRun, parse_exact_run, validate_exact_pair};
use content::{EvidenceKind, validate_directory, validate_generated_directory};
use paths::{checked_relative_path, read_json, repository_root, resolve_target_path};

const USAGE: &str = "Usage: cargo xtask phase10-evidence validate \
    --mode <local|exact-ref> --canonical-dir <target/path> \
    --sanitizer-dir <target/path> [--run-json <target/path>] \
    [--deny-run-id <id>]... [--deny-artifact-id <id>]...\n       cargo xtask \
    phase10-evidence validate-content <canonical|sanitizer> <target/path>";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Phase10EvidenceError {
    category: &'static str,
    message: String,
}

impl Phase10EvidenceError {
    pub(super) fn new(category: &'static str, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
        }
    }
}

impl Display for Phase10EvidenceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "phase10-evidence/{}: {}",
            self.category, self.message
        )
    }
}

impl std::error::Error for Phase10EvidenceError {}

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

pub(crate) fn run(args: &[String]) -> Result<(), Phase10EvidenceError> {
    if args
        .first()
        .is_some_and(|value| value == "validate-content")
    {
        return validate_content_command(args);
    }
    let options = parse_options(args)?;
    let root = repository_root()?;
    let maybe_run = load_exact_run(&root, &options)?;
    let canonical = validate_directory(
        &root,
        &options.canonical_dir,
        EvidenceKind::Canonical,
        maybe_run.as_ref(),
    )?;
    let sanitizer = validate_directory(
        &root,
        &options.sanitizer_dir,
        EvidenceKind::Sanitizer,
        maybe_run.as_ref(),
    )?;
    if canonical.semantic_manifest_sha256 != sanitizer.semantic_manifest_sha256 {
        return Err(Phase10EvidenceError::new(
            "semantic-manifest",
            "canonical and sanitizer semantic manifests differ",
        ));
    }
    if let Some(run) = maybe_run.as_ref() {
        validate_exact_pair(
            &root,
            run,
            &canonical,
            &sanitizer,
            &options.denied_artifact_ids,
        )?;
    }
    println!(
        "Phase 10 evidence verified: 5 cases, 80 semantic leaves, mode {:?}",
        options.mode
    );
    Ok(())
}

fn validate_content_command(args: &[String]) -> Result<(), Phase10EvidenceError> {
    let [_, kind, directory] = args else {
        return Err(usage(
            "`validate-content` requires one evidence kind and directory",
        ));
    };
    let kind = EvidenceKind::parse(kind)?;
    let root = repository_root()?;
    let relative = checked_relative_path(directory)?;
    validate_generated_directory(&root, &relative, kind)?;
    println!("Phase 10 generated {kind:?} content verified before identity");
    Ok(())
}

fn load_exact_run(
    root: &std::path::Path,
    options: &Options,
) -> Result<Option<ExactRun>, Phase10EvidenceError> {
    if options.mode == ValidationMode::Local {
        return Ok(None);
    }
    let relative = options
        .maybe_run_json
        .as_ref()
        .expect("exact-ref options require run-json");
    let path = resolve_target_path(root, relative, "run-json")?;
    let value: serde_json::Value = read_json(&path, "run-json")?;
    let run_id = value
        .get("run_id")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| Phase10EvidenceError::new("run", "run_id is absent or invalid"))?;
    if options.denied_run_ids.contains(&run_id) {
        return Err(Phase10EvidenceError::new(
            "run",
            format!("run {run_id} is denylisted"),
        ));
    }
    parse_exact_run(value, &options.denied_run_ids).map(Some)
}

fn parse_options(args: &[String]) -> Result<Options, Phase10EvidenceError> {
    let Some((command, rest)) = args.split_first() else {
        return Err(usage("missing phase10-evidence subcommand"));
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
        .map(|value| checked_relative_path(&value))
        .transpose()?;
    if (mode == ValidationMode::ExactRef) != maybe_run_json.is_some() {
        return Err(usage(
            "exact-ref requires run-json and local mode forbids it",
        ));
    }
    Ok(Options {
        mode,
        canonical_dir: checked_relative_path(&one(&values, "--canonical-dir")?)?,
        sanitizer_dir: checked_relative_path(&one(&values, "--sanitizer-dir")?)?,
        maybe_run_json,
        denied_run_ids: parse_ids(&values, "--deny-run-id")?,
        denied_artifact_ids: parse_ids(&values, "--deny-artifact-id")?,
    })
}

fn one(
    values: &BTreeMap<String, Vec<String>>,
    option: &str,
) -> Result<String, Phase10EvidenceError> {
    optional_one(values, option)?.ok_or_else(|| usage(format!("missing `{option}`")))
}

fn optional_one(
    values: &BTreeMap<String, Vec<String>>,
    option: &str,
) -> Result<Option<String>, Phase10EvidenceError> {
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
) -> Result<BTreeSet<u64>, Phase10EvidenceError> {
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

fn usage(message: impl Into<String>) -> Phase10EvidenceError {
    Phase10EvidenceError::new("usage", format!("{}\n\n{USAGE}", message.into()))
}
