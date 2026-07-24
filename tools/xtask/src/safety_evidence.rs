//! Closed CLI shell around the shared Phase 12 safety-evidence contracts.

pub(crate) mod contract;

use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use contract::{
    differential_leaf_coverage, render_execution_list, sha256, validate_coverage_contract_bytes,
    validate_regression_manifest_bytes, validate_regression_result_bytes,
};
use serde::Serialize;

const USAGE: &str = "Usage: cargo xtask safety-evidence validate-regressions \
    [--emit-execution-list]\n       cargo xtask safety-evidence \
    validate-regression-results --candidate FULL_SHA --results \
    target/phase12-regressions/FULL_SHA\n       cargo xtask safety-evidence validate-coverage\n       \
    cargo xtask safety-evidence validate-differential-leaves \
    --expected PATH --observed PATH --output PATH";
const REGRESSION_MANIFEST: &str = "reference/regressions/manifest.toml";
const COVERAGE_CONTRACT: &str = "reference/coverage/contract.json";
const COMPLETION_FILE: &str = "completion.json";
const IDENTITY_FILE: &str = "identity.json";
const MAXIMUM_RESULT_BYTES: usize = 1024 * 1024;

/// Fail-closed safety-evidence CLI error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SafetyEvidenceError {
    category: &'static str,
    message: String,
}

impl SafetyEvidenceError {
    fn new(category: &'static str, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
        }
    }

    fn usage(message: impl Into<String>) -> Self {
        Self::new("usage", format!("{}\n\n{USAGE}", message.into()))
    }
}

impl Display for SafetyEvidenceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "safety-evidence/{}: {}",
            self.category, self.message
        )
    }
}

impl std::error::Error for SafetyEvidenceError {}

#[derive(Debug)]
struct ResultOptions {
    candidate: String,
    results: PathBuf,
}

#[derive(Serialize)]
struct ResultIdentity<'a> {
    schema_version: u32,
    candidate_sha: &'a str,
    regression_manifest_sha256: String,
    completion_sha256: String,
}

pub(crate) fn run(args: &[String]) -> Result<(), SafetyEvidenceError> {
    let Some((command, command_args)) = args.split_first() else {
        return Err(SafetyEvidenceError::usage("missing subcommand"));
    };
    match command.as_str() {
        "--help" | "-h" => {
            println!("{USAGE}");
            Ok(())
        }
        "validate-regressions" => validate_regressions(command_args),
        "validate-regression-results" => validate_regression_results(command_args),
        "validate-coverage" => validate_coverage(command_args),
        "validate-differential-leaves" => validate_differential_leaves(command_args),
        unknown => Err(SafetyEvidenceError::usage(format!(
            "unknown subcommand `{unknown}`"
        ))),
    }
}

fn validate_differential_leaves(args: &[String]) -> Result<(), SafetyEvidenceError> {
    let mut values = BTreeMap::<String, String>::new();
    for pair in args.chunks(2) {
        let [option, value] = pair else {
            return Err(SafetyEvidenceError::usage(
                "every differential-leaf option requires one value",
            ));
        };
        if !matches!(option.as_str(), "--expected" | "--observed" | "--output")
            || value.starts_with("--")
            || values.insert(option.clone(), value.clone()).is_some()
        {
            return Err(SafetyEvidenceError::usage(format!(
                "unknown, valueless, or duplicated option `{option}`"
            )));
        }
    }
    let expected = required_path(&mut values, "--expected")?;
    let observed = required_path(&mut values, "--observed")?;
    let output = required_path(&mut values, "--output")?;
    if !values.is_empty() {
        return Err(SafetyEvidenceError::usage(
            "unexpected differential-leaf options",
        ));
    }
    let expected_bytes = read_regular(&expected, "differential leaves")?;
    let observed_bytes = read_regular(&observed, "differential leaves")?;
    let report = differential_leaf_coverage(&expected_bytes, &observed_bytes)
        .map_err(|error| SafetyEvidenceError::new("coverage", error.to_string()))?;
    let mut bytes = serde_json::to_vec_pretty(&report)
        .map_err(|error| SafetyEvidenceError::new("coverage", error.to_string()))?;
    bytes.push(b'\n');
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)
        .map_err(|error| SafetyEvidenceError::new("coverage", error.to_string()))?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|error| SafetyEvidenceError::new("coverage", error.to_string()))?;
    if !report.missed().is_empty() {
        return Err(SafetyEvidenceError::new(
            "coverage",
            format!(
                "{} required differential leaves were missed",
                report.missed().len()
            ),
        ));
    }
    println!(
        "safety evidence differential coverage verified: {} leaves",
        serde_json::from_slice::<Vec<String>>(&observed_bytes)
            .map_err(|error| SafetyEvidenceError::new("coverage", error.to_string()))?
            .len()
    );
    Ok(())
}

fn required_path(
    values: &mut BTreeMap<String, String>,
    option: &str,
) -> Result<PathBuf, SafetyEvidenceError> {
    values
        .remove(option)
        .map(PathBuf::from)
        .ok_or_else(|| SafetyEvidenceError::usage(format!("missing {option}")))
}

fn validate_regressions(args: &[String]) -> Result<(), SafetyEvidenceError> {
    let emit_execution_list = match args {
        [] => false,
        [argument] if argument == "--emit-execution-list" => true,
        _ => {
            return Err(SafetyEvidenceError::usage(
                "validate-regressions accepts only --emit-execution-list",
            ));
        }
    };
    let root = repository_root()?;
    let bytes = read_regular(&root.join(REGRESSION_MANIFEST), "regression manifest")?;
    let manifest = validate_regression_manifest_bytes(&root, &bytes)
        .map_err(|error| SafetyEvidenceError::new("regressions", error.to_string()))?;
    if emit_execution_list {
        print!(
            "{}",
            render_execution_list(&manifest)
                .map_err(|error| SafetyEvidenceError::new("regressions", error.to_string()))?
        );
    } else {
        println!(
            "safety evidence regressions verified: {} reviewed entries",
            manifest.regressions().len()
        );
    }
    Ok(())
}

fn validate_coverage(args: &[String]) -> Result<(), SafetyEvidenceError> {
    if !args.is_empty() {
        return Err(SafetyEvidenceError::usage(
            "validate-coverage does not accept arguments",
        ));
    }
    let root = repository_root()?;
    let bytes = read_regular(&root.join(COVERAGE_CONTRACT), "coverage contract")?;
    validate_coverage_contract_bytes(&bytes)
        .map_err(|error| SafetyEvidenceError::new("coverage", error.to_string()))?;
    println!("safety evidence coverage contract verified: parity_authority=false");
    Ok(())
}

fn validate_regression_results(args: &[String]) -> Result<(), SafetyEvidenceError> {
    if matches!(args, [argument] if argument == "--help" || argument == "-h") {
        println!("{USAGE}");
        return Ok(());
    }
    let options = parse_result_options(args)?;
    let root = repository_root()?;
    let result_directory = checked_result_directory(&root, &options)?;
    let identity_path = result_directory.join(IDENTITY_FILE);
    if fs::symlink_metadata(&identity_path).is_ok() {
        return Err(SafetyEvidenceError::new(
            "identity",
            "result identity already exists; validation must precede identity publication",
        ));
    }
    require_exact_result_inventory(&result_directory)?;

    let manifest_bytes = read_regular(&root.join(REGRESSION_MANIFEST), "regression manifest")?;
    let manifest = validate_regression_manifest_bytes(&root, &manifest_bytes)
        .map_err(|error| SafetyEvidenceError::new("regressions", error.to_string()))?;
    let completion_bytes = read_regular(
        &result_directory.join(COMPLETION_FILE),
        "result-set completion marker",
    )?;
    validate_regression_result_bytes(&manifest, &options.candidate, &completion_bytes)
        .map_err(|error| SafetyEvidenceError::new("results", error.to_string()))?;

    write_identity_last(
        &identity_path,
        &options.candidate,
        &manifest_bytes,
        &completion_bytes,
    )?;
    println!(
        "safety evidence regression results verified: candidate {}",
        options.candidate
    );
    Ok(())
}

fn parse_result_options(args: &[String]) -> Result<ResultOptions, SafetyEvidenceError> {
    let mut values = BTreeMap::<String, String>::new();
    for pair in args.chunks(2) {
        let [option, value] = pair else {
            return Err(SafetyEvidenceError::usage(
                "every results option requires one value",
            ));
        };
        if !matches!(option.as_str(), "--candidate" | "--results")
            || value.starts_with("--")
            || values.insert(option.clone(), value.clone()).is_some()
        {
            return Err(SafetyEvidenceError::usage(format!(
                "unknown, valueless, or duplicated option `{option}`"
            )));
        }
    }
    let candidate = values
        .remove("--candidate")
        .ok_or_else(|| SafetyEvidenceError::usage("missing --candidate"))?;
    if !is_full_sha(&candidate) {
        return Err(SafetyEvidenceError::usage(
            "candidate must be one canonical lowercase 40-hex SHA",
        ));
    }
    let results = values
        .remove("--results")
        .ok_or_else(|| SafetyEvidenceError::usage("missing --results"))?;
    if !values.is_empty() {
        return Err(SafetyEvidenceError::usage("unexpected results options"));
    }
    let expected = format!("target/phase12-regressions/{candidate}");
    if results != expected {
        return Err(SafetyEvidenceError::usage(format!(
            "results path must equal `{expected}`"
        )));
    }
    Ok(ResultOptions {
        candidate,
        results: PathBuf::from(results),
    })
}

fn checked_result_directory(
    repository_root: &Path,
    options: &ResultOptions,
) -> Result<PathBuf, SafetyEvidenceError> {
    let canonical_root = fs::canonicalize(repository_root).map_err(|error| {
        SafetyEvidenceError::new("path", format!("failed to resolve repository: {error}"))
    })?;
    let mut current = repository_root.to_path_buf();
    for component in &options.results {
        current.push(component);
        let metadata = fs::symlink_metadata(&current).map_err(|error| {
            SafetyEvidenceError::new(
                "path",
                format!("failed to inspect {}: {error}", current.display()),
            )
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(SafetyEvidenceError::new(
                "path",
                format!("{} must be an ordinary directory", current.display()),
            ));
        }
    }
    let canonical = fs::canonicalize(&current).map_err(|error| {
        SafetyEvidenceError::new(
            "path",
            format!("failed to resolve {}: {error}", current.display()),
        )
    })?;
    let expected = canonical_root
        .join("target")
        .join("phase12-regressions")
        .join(&options.candidate);
    if canonical != expected || !canonical.starts_with(&canonical_root) {
        return Err(SafetyEvidenceError::new(
            "path",
            "regression results escaped or substituted the exact candidate directory",
        ));
    }
    Ok(canonical)
}

fn require_exact_result_inventory(directory: &Path) -> Result<(), SafetyEvidenceError> {
    let mut names = fs::read_dir(directory)
        .map_err(|error| {
            SafetyEvidenceError::new(
                "results",
                format!("failed to read {}: {error}", directory.display()),
            )
        })?
        .map(|entry| {
            entry
                .map_err(|error| SafetyEvidenceError::new("results", error.to_string()))
                .and_then(|entry| {
                    entry.file_name().into_string().map_err(|_name| {
                        SafetyEvidenceError::new("results", "result filename is not UTF-8")
                    })
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();
    if names != [COMPLETION_FILE] {
        return Err(SafetyEvidenceError::new(
            "results",
            "result directory must contain only completion.json before validation",
        ));
    }
    Ok(())
}

fn write_identity_last(
    identity_path: &Path,
    candidate: &str,
    manifest_bytes: &[u8],
    completion_bytes: &[u8],
) -> Result<(), SafetyEvidenceError> {
    let identity = ResultIdentity {
        schema_version: 1,
        candidate_sha: candidate,
        regression_manifest_sha256: sha256(manifest_bytes),
        completion_sha256: sha256(completion_bytes),
    };
    let mut bytes = serde_json::to_vec_pretty(&identity).map_err(|error| {
        SafetyEvidenceError::new("identity", format!("failed to render identity: {error}"))
    })?;
    bytes.push(b'\n');
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(identity_path)
        .map_err(|error| {
            SafetyEvidenceError::new(
                "identity",
                format!("failed to create {}: {error}", identity_path.display()),
            )
        })?;
    file.write_all(&bytes).map_err(|error| {
        SafetyEvidenceError::new(
            "identity",
            format!("failed to write {}: {error}", identity_path.display()),
        )
    })?;
    file.sync_all().map_err(|error| {
        SafetyEvidenceError::new(
            "identity",
            format!("failed to sync {}: {error}", identity_path.display()),
        )
    })
}

fn repository_root() -> Result<PathBuf, SafetyEvidenceError> {
    let current_dir = std::env::current_dir().map_err(|error| {
        SafetyEvidenceError::new("path", format!("failed to read current directory: {error}"))
    })?;
    let Some(root) = current_dir.ancestors().find(|candidate| {
        candidate.join("Cargo.toml").is_file()
            && candidate.join("crates/liquidfun/Cargo.toml").is_file()
    }) else {
        return Err(SafetyEvidenceError::new(
            "path",
            "could not find the liquidfun Cargo workspace",
        ));
    };
    Ok(root.to_path_buf())
}

fn read_regular(path: &Path, field: &'static str) -> Result<Vec<u8>, SafetyEvidenceError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        SafetyEvidenceError::new(
            field,
            format!("failed to inspect {}: {error}", path.display()),
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(SafetyEvidenceError::new(
            field,
            format!("{} must be an ordinary file", path.display()),
        ));
    }
    let length = usize::try_from(metadata.len())
        .map_err(|_error| SafetyEvidenceError::new(field, "file length exceeds usize"))?;
    if length > MAXIMUM_RESULT_BYTES {
        return Err(SafetyEvidenceError::new(
            field,
            "file exceeds the reviewed one-mebibyte bound",
        ));
    }
    fs::read(path).map_err(|error| {
        SafetyEvidenceError::new(field, format!("failed to read {}: {error}", path.display()))
    })
}

fn is_full_sha(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
