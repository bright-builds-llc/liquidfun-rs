//! Read-only release readiness audit over existing evidence artifacts.

mod domain;
mod report;
mod validation;

use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};
use std::path::{Path, PathBuf};

const USAGE: &str = "Usage: cargo xtask release audit --manifest PATH \
    --candidate COMMIT --output human|json";

/// Stable categorized release-audit failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReleaseError {
    category: &'static str,
    message: String,
}

impl ReleaseError {
    pub(crate) fn new(category: &'static str, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
        }
    }

    fn usage(message: impl Into<String>) -> Self {
        Self::new("usage", format!("{}\n\n{USAGE}", message.into()))
    }
}

impl Display for ReleaseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "release/{}: {}", self.category, self.message)
    }
}

impl std::error::Error for ReleaseError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Human,
    Json,
}

struct AuditOptions {
    manifest: PathBuf,
    candidate: String,
    output: OutputFormat,
}

pub(crate) fn run(args: &[String]) -> Result<(), ReleaseError> {
    if matches!(args, [argument] if argument == "--help" || argument == "-h")
        || matches!(args, [command, argument] if command == "audit" && (argument == "--help" || argument == "-h"))
    {
        println!("{USAGE}");
        return Ok(());
    }
    let Some((command, command_args)) = args.split_first() else {
        return Err(ReleaseError::usage("missing release subcommand"));
    };
    if command != "audit" {
        return Err(ReleaseError::usage(format!(
            "unknown release subcommand `{command}`"
        )));
    }
    let options = parse_options(command_args)?;
    let root = repository_root()?;
    let readiness = validation::audit(&root, &options.manifest, &options.candidate)?;
    match options.output {
        OutputFormat::Human => print!("{}", report::human(&readiness)),
        OutputFormat::Json => print!(
            "{}",
            report::json(&readiness)
                .map_err(|error| ReleaseError::new("report", error.to_string()))?
        ),
    }
    Ok(())
}

fn parse_options(args: &[String]) -> Result<AuditOptions, ReleaseError> {
    if !args.len().is_multiple_of(2) {
        return Err(ReleaseError::usage(
            "release audit options require flag/value pairs",
        ));
    }
    let mut values = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        let option = pair[0].as_str();
        if !matches!(option, "--manifest" | "--candidate" | "--output")
            || pair[1].starts_with("--")
            || values.insert(option, pair[1].as_str()).is_some()
        {
            return Err(ReleaseError::usage(format!(
                "unknown, valueless, or duplicate option `{option}`"
            )));
        }
    }
    let manifest = required(&values, "--manifest")?;
    let candidate = required(&values, "--candidate")?;
    let output = match required(&values, "--output")? {
        "human" => OutputFormat::Human,
        "json" => OutputFormat::Json,
        _ => {
            return Err(ReleaseError::usage(
                "--output must be exactly `human` or `json`",
            ));
        }
    };
    Ok(AuditOptions {
        manifest: PathBuf::from(manifest),
        candidate: candidate.to_owned(),
        output,
    })
}

fn required<'a>(values: &'a BTreeMap<&str, &str>, option: &str) -> Result<&'a str, ReleaseError> {
    values
        .get(option)
        .copied()
        .ok_or_else(|| ReleaseError::usage(format!("missing `{option}`")))
}

fn repository_root() -> Result<PathBuf, ReleaseError> {
    let current = std::env::current_dir()
        .map_err(|error| ReleaseError::new("workspace", error.to_string()))?;
    current
        .ancestors()
        .find(|candidate| {
            candidate.join("Cargo.toml").is_file()
                && candidate.join("tools/xtask/Cargo.toml").is_file()
        })
        .map(Path::to_path_buf)
        .ok_or_else(|| ReleaseError::new("workspace", "could not find Cargo workspace root"))
}
