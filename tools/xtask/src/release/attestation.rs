//! Frozen-source validation for a later, allowlisted release attestation.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output};

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::{ReleaseError, report, validation};

const SOURCE_SCHEMA_VERSION: u8 = 1;
const MAXIMUM_RECORD_BYTES: u64 = 4 * 1024 * 1024;
const WORKTREE_COMMAND: &str = "validate-worktree";
const COMMITTED_COMMAND: &str = "validate";
const ATTESTATION_PATHS: &[&str] = &[
    ".github/workflows/release.yml",
    "tools/xtask/src/release.rs",
    "tools/xtask/src/release/attestation.rs",
    "tools/xtask/src/main.rs",
    "tools/xtask/tests/release_attestation.rs",
    "tools/xtask/tests/release_cli.rs",
    "reference/release/source-candidate.json",
    "reference/release/candidate-manifest.json",
    "reference/release/audit-report.json",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceCandidate {
    schema_version: u8,
    ready: bool,
    #[serde(rename = "source_candidate_commit")]
    commit: String,
    source_tree_sha256: String,
    candidate_manifest_sha256: String,
    audit_report_sha256: String,
}

struct AttestationOptions {
    source: PathBuf,
    manifest: PathBuf,
    report: PathBuf,
    maybe_attestation_commit: Option<String>,
}

struct VerifiedInputs {
    source: SourceCandidate,
    manifest_path: PathBuf,
    report_bytes: Vec<u8>,
}

pub(super) fn run(repository_root: &Path, args: &[String]) -> Result<(), ReleaseError> {
    let Some((command, command_args)) = args.split_first() else {
        return Err(ReleaseError::new(
            "usage",
            "missing release attestation subcommand",
        ));
    };
    if matches!(command_args, [argument] if argument == "--help" || argument == "-h") {
        return Err(ReleaseError::new(
            "usage",
            "release attestation help is available from `cargo xtask release --help`",
        ));
    }
    let options = parse_options(command, command_args)?;
    let verified = validate_inputs(repository_root, &options)?;
    match command.as_str() {
        WORKTREE_COMMAND => {
            validate_worktree_paths(repository_root, &verified.source.commit)?;
        }
        COMMITTED_COMMAND => {
            let attestation_commit = options
                .maybe_attestation_commit
                .as_deref()
                .ok_or_else(|| ReleaseError::new("usage", "missing `--attestation-commit`"))?;
            validate_committed_paths(repository_root, &verified.source.commit, attestation_commit)?;
        }
        _ => {
            return Err(ReleaseError::new(
                "usage",
                format!("unknown release attestation subcommand `{command}`"),
            ));
        }
    }
    validate_audit(repository_root, &verified)?;
    println!(
        "release attestation: VALID\nsource candidate: {}",
        verified.source.commit
    );
    Ok(())
}

fn parse_options(command: &str, args: &[String]) -> Result<AttestationOptions, ReleaseError> {
    if !matches!(command, WORKTREE_COMMAND | COMMITTED_COMMAND) {
        return Err(ReleaseError::new(
            "usage",
            format!("unknown release attestation subcommand `{command}`"),
        ));
    }
    if !args.len().is_multiple_of(2) {
        return Err(ReleaseError::new(
            "usage",
            "release attestation options require flag/value pairs",
        ));
    }
    let mut values = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        let option = pair[0].as_str();
        let allowed = matches!(option, "--source" | "--manifest" | "--report")
            || command == COMMITTED_COMMAND && option == "--attestation-commit";
        if !allowed
            || pair[1].starts_with("--")
            || values.insert(option, pair[1].as_str()).is_some()
        {
            return Err(ReleaseError::new(
                "usage",
                format!("unknown, valueless, or duplicate option `{option}`"),
            ));
        }
    }
    let required = |option| {
        values
            .get(option)
            .copied()
            .ok_or_else(|| ReleaseError::new("usage", format!("missing `{option}`")))
    };
    let maybe_attestation_commit = values
        .get("--attestation-commit")
        .map(|value| (*value).to_owned());
    Ok(AttestationOptions {
        source: PathBuf::from(required("--source")?),
        manifest: PathBuf::from(required("--manifest")?),
        report: PathBuf::from(required("--report")?),
        maybe_attestation_commit,
    })
}

fn validate_inputs(
    repository_root: &Path,
    options: &AttestationOptions,
) -> Result<VerifiedInputs, ReleaseError> {
    let source_bytes = read_confined(repository_root, &options.source, "attestation-input")?;
    let source: SourceCandidate = serde_json::from_slice(&source_bytes)
        .map_err(|error| ReleaseError::new("attestation-source", error.to_string()))?;
    validate_source_record(&source)?;

    let manifest_bytes = read_confined(repository_root, &options.manifest, "attestation-input")?;
    let report_bytes = read_confined(repository_root, &options.report, "attestation-input")?;
    validate_hash(
        &manifest_bytes,
        &source.candidate_manifest_sha256,
        "attestation-manifest-hash",
    )?;
    validate_hash(
        &report_bytes,
        &source.audit_report_sha256,
        "attestation-report-hash",
    )?;
    validate_record_candidates(&source.commit, &manifest_bytes, &report_bytes)?;
    validate_source_tree(repository_root, &source)?;
    Ok(VerifiedInputs {
        source,
        manifest_path: options.manifest.clone(),
        report_bytes,
    })
}

fn validate_source_record(source: &SourceCandidate) -> Result<(), ReleaseError> {
    if source.schema_version != SOURCE_SCHEMA_VERSION
        || !source.ready
        || !is_full_sha(&source.commit)
        || !is_sha256(&source.source_tree_sha256)
        || !is_sha256(&source.candidate_manifest_sha256)
        || !is_sha256(&source.audit_report_sha256)
    {
        return Err(ReleaseError::new(
            "attestation-source",
            "source record must be strict, ready, and carry full SHA identities",
        ));
    }
    Ok(())
}

fn validate_record_candidates(
    candidate: &str,
    manifest_bytes: &[u8],
    report_bytes: &[u8],
) -> Result<(), ReleaseError> {
    let manifest: Value = serde_json::from_slice(manifest_bytes)
        .map_err(|error| ReleaseError::new("attestation-manifest", error.to_string()))?;
    let report: Value = serde_json::from_slice(report_bytes)
        .map_err(|error| ReleaseError::new("attestation-report", error.to_string()))?;
    if manifest.get("candidate_commit").and_then(Value::as_str) != Some(candidate)
        || report.get("candidate_commit").and_then(Value::as_str) != Some(candidate)
        || report.get("decision").and_then(Value::as_str) != Some("ready")
    {
        return Err(ReleaseError::new(
            "attestation-candidate",
            "source, manifest, and ready report candidate identities differ",
        ));
    }
    Ok(())
}

fn validate_source_tree(
    repository_root: &Path,
    source: &SourceCandidate,
) -> Result<(), ReleaseError> {
    let output = git(
        repository_root,
        &["ls-tree", "-r", "-z", "--full-tree", &source.commit],
        "attestation-source-tree",
    )?;
    validate_hash(
        &output.stdout,
        &source.source_tree_sha256,
        "attestation-source-tree",
    )
}

fn validate_worktree_paths(repository_root: &Path, candidate: &str) -> Result<(), ReleaseError> {
    require_ancestor(repository_root, candidate, "HEAD")?;
    let tracked = git(
        repository_root,
        &["diff", "--name-only", "-z", candidate, "--"],
        "attestation-diff",
    )?;
    let untracked = git(
        repository_root,
        &["ls-files", "--others", "--exclude-standard", "-z", "--"],
        "attestation-diff",
    )?;
    let mut paths = parse_nul_paths(&tracked.stdout)?;
    paths.extend(parse_nul_paths(&untracked.stdout)?);
    validate_changed_paths(&paths)
}

fn validate_committed_paths(
    repository_root: &Path,
    source_candidate: &str,
    attestation_commit: &str,
) -> Result<(), ReleaseError> {
    let resolved = resolve_commit(repository_root, attestation_commit)?;
    if resolved == source_candidate {
        return Err(ReleaseError::new(
            "attestation-diff",
            "attestation commit must be later than the frozen source candidate",
        ));
    }
    require_ancestor(repository_root, source_candidate, &resolved)?;
    let range = format!("{source_candidate}..{resolved}");
    let output = git(
        repository_root,
        &["diff", "--name-only", "-z", &range, "--"],
        "attestation-diff",
    )?;
    validate_changed_paths(&parse_nul_paths(&output.stdout)?)
}

fn resolve_commit(repository_root: &Path, revision: &str) -> Result<String, ReleaseError> {
    let commit_expression = format!("{revision}^{{commit}}");
    let output = git(
        repository_root,
        &["rev-parse", "--verify", &commit_expression],
        "attestation-commit",
    )?;
    let resolved = String::from_utf8(output.stdout)
        .map_err(|error| ReleaseError::new("attestation-commit", error.to_string()))?;
    let resolved = resolved.trim();
    if !is_full_sha(resolved) {
        return Err(ReleaseError::new(
            "attestation-commit",
            "attestation revision did not resolve to one full commit SHA",
        ));
    }
    Ok(resolved.to_owned())
}

fn require_ancestor(
    repository_root: &Path,
    ancestor: &str,
    descendant: &str,
) -> Result<(), ReleaseError> {
    git(
        repository_root,
        &["merge-base", "--is-ancestor", ancestor, descendant],
        "attestation-diff",
    )?;
    Ok(())
}

fn validate_changed_paths(paths: &BTreeSet<String>) -> Result<(), ReleaseError> {
    let maybe_rejected = paths
        .iter()
        .find(|path| !ATTESTATION_PATHS.contains(&path.as_str()));
    if let Some(rejected) = maybe_rejected {
        return Err(ReleaseError::new(
            "attestation-diff",
            format!("non-attestation path changed: {rejected}"),
        ));
    }
    Ok(())
}

fn validate_audit(repository_root: &Path, verified: &VerifiedInputs) -> Result<(), ReleaseError> {
    let readiness = validation::audit(
        repository_root,
        &verified.manifest_path,
        &verified.source.commit,
    )?;
    let expected = report::json(&readiness)
        .map_err(|error| ReleaseError::new("attestation-report", error.to_string()))?;
    if expected.as_bytes() != verified.report_bytes {
        return Err(ReleaseError::new(
            "attestation-report",
            "audit report bytes differ from the independently recomputed report",
        ));
    }
    Ok(())
}

fn read_confined(
    repository_root: &Path,
    relative: &Path,
    category: &'static str,
) -> Result<Vec<u8>, ReleaseError> {
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(ReleaseError::new(
            category,
            "input path must be normalized and repository-relative",
        ));
    }
    let path = repository_root.join(relative);
    reject_symlink_components(&path, category)?;
    let canonical_root = fs::canonicalize(repository_root)
        .map_err(|error| ReleaseError::new(category, error.to_string()))?;
    let canonical =
        fs::canonicalize(&path).map_err(|error| ReleaseError::new(category, error.to_string()))?;
    if !canonical.starts_with(canonical_root) {
        return Err(ReleaseError::new(category, "input escaped the repository"));
    }
    let metadata = fs::symlink_metadata(&canonical)
        .map_err(|error| ReleaseError::new(category, error.to_string()))?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() > MAXIMUM_RECORD_BYTES
    {
        return Err(ReleaseError::new(
            category,
            "input must be a bounded ordinary file",
        ));
    }
    fs::read(canonical).map_err(|error| ReleaseError::new(category, error.to_string()))
}

fn reject_symlink_components(path: &Path, category: &'static str) -> Result<(), ReleaseError> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        if let Ok(metadata) = fs::symlink_metadata(&current)
            && metadata.file_type().is_symlink()
        {
            return Err(ReleaseError::new(
                category,
                "input path contains a symbolic link",
            ));
        }
    }
    Ok(())
}

fn git(
    repository_root: &Path,
    args: &[&str],
    category: &'static str,
) -> Result<Output, ReleaseError> {
    let output = Command::new("git")
        .current_dir(repository_root)
        .args(args)
        .output()
        .map_err(|error| ReleaseError::new(category, error.to_string()))?;
    if !output.status.success() {
        return Err(ReleaseError::new(
            category,
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok(output)
}

fn parse_nul_paths(bytes: &[u8]) -> Result<BTreeSet<String>, ReleaseError> {
    let mut paths = BTreeSet::new();
    for bytes in bytes
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
    {
        let path = std::str::from_utf8(bytes)
            .map_err(|error| ReleaseError::new("attestation-diff", error.to_string()))?;
        if Path::new(path)
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
        {
            return Err(ReleaseError::new(
                "attestation-diff",
                "Git returned a non-normalized path",
            ));
        }
        paths.insert(path.to_owned());
    }
    Ok(paths)
}

fn validate_hash(bytes: &[u8], expected: &str, category: &'static str) -> Result<(), ReleaseError> {
    if sha256(bytes) != expected {
        return Err(ReleaseError::new(category, "SHA-256 identity differs"));
    }
    Ok(())
}

fn is_full_sha(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changed_path_allowlist_accepts_only_attestation_owned_paths() {
        // Arrange
        let allowed = BTreeSet::from([
            "reference/release/source-candidate.json".to_owned(),
            "tools/xtask/src/release/attestation.rs".to_owned(),
        ]);
        let rejected = BTreeSet::from(["README.md".to_owned()]);

        // Act
        let allowed_result = validate_changed_paths(&allowed);
        let rejected_result = validate_changed_paths(&rejected);

        // Assert
        assert!(allowed_result.is_ok());
        assert_eq!(
            rejected_result
                .expect_err("README is outside the allowlist")
                .category,
            "attestation-diff"
        );
    }
}
