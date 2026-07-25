use std::collections::BTreeSet;
use std::env;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Display, Formatter};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use serde::Deserialize;
use sha2::{Digest, Sha256};

mod artifact;
mod evidence_schema;
mod phase9_witness;

const USAGE: &str = "Usage: cargo xtask provenance check";
const SCHEMA_VERSION: u64 = 1;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ProvenanceError {
    category: &'static str,
    message: String,
}

impl ProvenanceError {
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

impl Display for ProvenanceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "provenance/{}: {}", self.category, self.message)
    }
}

impl Error for ProvenanceError {}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UpstreamLock {
    schema_version: u64,
    repository: String,
    revision: String,
    release_tag: String,
    release_tag_object: String,
    release_commit: String,
    submodule_path: String,
    patch_set: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceMap {
    schema_version: u64,
    mapping: Vec<SourceMapping>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceMapping {
    local_path: String,
    upstream_revision: String,
    upstream_path: String,
    derivation_kind: String,
    alteration_summary: String,
    notice_class: String,
}

#[derive(Debug, Deserialize)]
struct RevisionIdentity {
    schema_version: u64,
    oracle_revision: String,
}

struct ConfinedPaths {
    repository_root: PathBuf,
    canonical_root: PathBuf,
}

impl ConfinedPaths {
    fn new(repository_root: &Path) -> Result<Self, ProvenanceError> {
        let canonical_root = fs::canonicalize(repository_root).map_err(|error| {
            ProvenanceError::new(
                "path",
                format!(
                    "failed to canonicalize repository root {}: {error}",
                    repository_root.display()
                ),
            )
        })?;
        Ok(Self {
            repository_root: repository_root.to_path_buf(),
            canonical_root,
        })
    }

    fn file(&self, value: &str, field: &str) -> Result<PathBuf, ProvenanceError> {
        validate_relative_path(value, field, false)?;
        let mut candidate = self.repository_root.clone();
        for component in Path::new(value).components() {
            candidate.push(component.as_os_str());
            let metadata = fs::symlink_metadata(&candidate).map_err(|error| {
                ProvenanceError::new(
                    "path",
                    format!("failed to inspect {field} `{value}`: {error}"),
                )
            })?;
            if metadata.file_type().is_symlink() {
                return Err(ProvenanceError::new(
                    "path",
                    format!(
                        "{field} `{value}` traverses symlink {}",
                        candidate.display()
                    ),
                ));
            }
        }

        let canonical_candidate = fs::canonicalize(&candidate).map_err(|error| {
            ProvenanceError::new(
                "path",
                format!("failed to canonicalize {field} `{value}`: {error}"),
            )
        })?;
        if !canonical_candidate.starts_with(&self.canonical_root) {
            return Err(ProvenanceError::new(
                "path",
                format!("{field} `{value}` resolves outside the repository"),
            ));
        }
        if !canonical_candidate.is_file() {
            return Err(ProvenanceError::new(
                "path",
                format!("{field} `{value}` must resolve to a regular file"),
            ));
        }
        Ok(canonical_candidate)
    }
}

pub(crate) fn run(args: &[String]) -> Result<(), ProvenanceError> {
    if args != ["check"] {
        return Err(ProvenanceError::usage("expected `check`"));
    }
    let repository_root = repository_root()?;
    check(&repository_root)
}

pub(crate) fn check_artifacts(repository_root: &Path) -> Result<(), ProvenanceError> {
    let confined_paths = ConfinedPaths::new(repository_root)?;
    let upstream_lock: UpstreamLock = read_toml(
        &repository_root.join("reference/upstream-lock.toml"),
        "upstream lock",
    )?;
    validate_lock(&upstream_lock)?;
    let source_map: SourceMap = read_toml(
        &repository_root.join("reference/source-map.toml"),
        "source map",
    )?;
    validate_source_map(&confined_paths, &source_map, &upstream_lock.revision)?;
    let artifact_count = artifact::validate_manifest(
        repository_root,
        &confined_paths,
        &source_map,
        &upstream_lock.revision,
    )?;
    phase9_witness::validate(repository_root, &upstream_lock.revision)?;
    println!(
        "artifact provenance verified: oracle {} with {} artifact records",
        upstream_lock.revision, artifact_count
    );
    Ok(())
}

fn check(repository_root: &Path) -> Result<(), ProvenanceError> {
    let confined_paths = ConfinedPaths::new(repository_root)?;
    let upstream_lock: UpstreamLock = read_toml(
        &repository_root.join("reference/upstream-lock.toml"),
        "upstream lock",
    )?;
    validate_lock(&upstream_lock)?;
    verify_git_identities(repository_root, &upstream_lock)?;

    let source_map: SourceMap = read_toml(
        &repository_root.join("reference/source-map.toml"),
        "source map",
    )?;
    validate_source_map(&confined_paths, &source_map, &upstream_lock.revision)?;
    for relative in ["reference/discovery.json", "reference/compatibility.json"] {
        let identity: RevisionIdentity = read_json(&repository_root.join(relative), relative)?;
        require_schema(identity.schema_version, relative)?;
        require_revision(relative, &upstream_lock.revision, &identity.oracle_revision)?;
    }

    let artifact_count = artifact::validate_manifest(
        repository_root,
        &confined_paths,
        &source_map,
        &upstream_lock.revision,
    )?;
    phase9_witness::validate(repository_root, &upstream_lock.revision)?;
    println!(
        "provenance verified: oracle {} with {} artifact records",
        upstream_lock.revision, artifact_count
    );
    Ok(())
}

fn validate_lock(lock: &UpstreamLock) -> Result<(), ProvenanceError> {
    require_schema(lock.schema_version, "upstream lock")?;
    for (field, value) in [
        ("repository", lock.repository.as_str()),
        ("release_tag", lock.release_tag.as_str()),
        ("patch_set", lock.patch_set.as_str()),
    ] {
        require_nonempty(field, value)?;
    }
    for (field, revision) in [
        ("revision", lock.revision.as_str()),
        ("release_tag_object", lock.release_tag_object.as_str()),
        ("release_commit", lock.release_commit.as_str()),
    ] {
        require_revision_format(field, revision)?;
    }
    validate_relative_path(&lock.submodule_path, "submodule_path", false)
}

fn verify_git_identities(
    repository_root: &Path,
    lock: &UpstreamLock,
) -> Result<(), ProvenanceError> {
    let git = env::var_os("LIQUIDFUN_XTASK_GIT").unwrap_or_else(|| OsString::from("git"));
    let gitlink = run_git(
        &git,
        [
            OsStr::new("-C"),
            repository_root.as_os_str(),
            OsStr::new("ls-tree"),
            OsStr::new("HEAD"),
            OsStr::new("--"),
            OsStr::new(&lock.submodule_path),
        ],
        "read upstream gitlink",
    )?;
    let gitlink_revision = gitlink
        .split_whitespace()
        .nth(2)
        .ok_or_else(|| ProvenanceError::new("gitlink", "malformed git ls-tree output"))?;
    require_revision("gitlink", &lock.revision, gitlink_revision)?;

    let submodule = repository_root.join(&lock.submodule_path);
    if !submodule.is_dir() {
        return Err(ProvenanceError::new(
            "checkout",
            format!("missing submodule directory {}", submodule.display()),
        ));
    }
    let checkout = run_git(
        &git,
        [
            OsStr::new("-C"),
            submodule.as_os_str(),
            OsStr::new("rev-parse"),
            OsStr::new("HEAD"),
        ],
        "read upstream checkout",
    )?;
    require_revision("checkout", &lock.revision, checkout.trim())
}

fn validate_source_map(
    confined_paths: &ConfinedPaths,
    source_map: &SourceMap,
    oracle_revision: &str,
) -> Result<(), ProvenanceError> {
    require_schema(source_map.schema_version, "source map")?;
    let mut local_paths = BTreeSet::new();
    for mapping in &source_map.mapping {
        validate_relative_path(&mapping.local_path, "source-map local_path", false)?;
        validate_relative_path(&mapping.upstream_path, "source-map upstream_path", true)?;
        require_revision(
            &format!("source-map entry `{}`", mapping.local_path),
            oracle_revision,
            &mapping.upstream_revision,
        )?;
        if !local_paths.insert(mapping.local_path.as_str()) {
            return Err(ProvenanceError::new(
                "schema",
                format!("duplicate source-map local_path `{}`", mapping.local_path),
            ));
        }
        for (field, value) in [
            ("derivation_kind", mapping.derivation_kind.as_str()),
            ("alteration_summary", mapping.alteration_summary.as_str()),
            ("notice_class", mapping.notice_class.as_str()),
        ] {
            require_nonempty(field, value)?;
        }
        let _local_path = confined_paths.file(&mapping.local_path, "source-map local_path")?;
    }
    Ok(())
}

fn sha256(path: &Path) -> Result<String, ProvenanceError> {
    let mut file = File::open(path).map_err(|error| {
        ProvenanceError::new(
            "hash",
            format!("failed to open {}: {error}", path.display()),
        )
    })?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let count = file.read(&mut buffer).map_err(|error| {
            ProvenanceError::new(
                "hash",
                format!("failed to read {}: {error}", path.display()),
            )
        })?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn run_git<'a>(
    program: &OsStr,
    args: impl IntoIterator<Item = &'a OsStr>,
    action: &str,
) -> Result<String, ProvenanceError> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| ProvenanceError::new("process", format!("failed to {action}: {error}")))?;
    if !output.status.success() {
        return Err(ProvenanceError::new(
            "process",
            format!(
                "failed to {action} with {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    String::from_utf8(output.stdout).map_err(|error| {
        ProvenanceError::new(
            "process",
            format!("{action} returned non-UTF-8 output: {error}"),
        )
    })
}

fn read_toml<T: for<'de> Deserialize<'de>>(path: &Path, label: &str) -> Result<T, ProvenanceError> {
    let contents = read_text(path)?;
    toml::from_str(&contents).map_err(|error| {
        ProvenanceError::new(
            "schema",
            format!("invalid {label} in {}: {error}", path.display()),
        )
    })
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path, label: &str) -> Result<T, ProvenanceError> {
    let contents = read_text(path)?;
    serde_json::from_str(&contents).map_err(|error| {
        ProvenanceError::new(
            "schema",
            format!("invalid {label} in {}: {error}", path.display()),
        )
    })
}

fn read_text(path: &Path) -> Result<String, ProvenanceError> {
    fs::read_to_string(path).map_err(|error| {
        ProvenanceError::new(
            "filesystem",
            format!("failed to read {}: {error}", path.display()),
        )
    })
}

fn require_schema(actual: u64, label: &str) -> Result<(), ProvenanceError> {
    if actual == SCHEMA_VERSION {
        return Ok(());
    }
    Err(ProvenanceError::new(
        "schema",
        format!("{label} schema version must be {SCHEMA_VERSION}, actual {actual}"),
    ))
}

fn require_revision(label: &str, expected: &str, actual: &str) -> Result<(), ProvenanceError> {
    if actual == expected {
        return Ok(());
    }
    Err(ProvenanceError::new(
        "revision",
        format!("{label} revision mismatch: expected `{expected}`, actual `{actual}`"),
    ))
}

fn require_revision_format(label: &str, revision: &str) -> Result<(), ProvenanceError> {
    if revision.len() == 40 && is_lower_hex(revision) {
        return Ok(());
    }
    Err(ProvenanceError::new(
        "revision",
        format!("{label} must be a lowercase 40-hex revision"),
    ))
}

fn is_lower_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn require_nonempty(field: &str, value: &str) -> Result<(), ProvenanceError> {
    if !value.trim().is_empty() {
        return Ok(());
    }
    Err(ProvenanceError::new(
        "schema",
        format!("{field} cannot be empty"),
    ))
}

fn validate_relative_path(
    value: &str,
    field: &str,
    allow_dot: bool,
) -> Result<(), ProvenanceError> {
    if allow_dot && value == "." {
        return Ok(());
    }
    if value.is_empty()
        || value.contains('\\')
        || Path::new(value)
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(ProvenanceError::new(
            "path",
            format!("{field} `{value}` must be a normalized confined relative path"),
        ));
    }
    Ok(())
}

fn repository_root() -> Result<PathBuf, ProvenanceError> {
    if let Some(root) = env::var_os("LIQUIDFUN_XTASK_ROOT") {
        return Ok(PathBuf::from(root));
    }
    let current_dir = env::current_dir().map_err(|error| {
        ProvenanceError::new(
            "filesystem",
            format!("failed to read current directory: {error}"),
        )
    })?;
    let Some(root) = current_dir.ancestors().find(|candidate| {
        candidate.join("reference/upstream-lock.toml").is_file()
            && candidate.join(".gitmodules").is_file()
    }) else {
        return Err(ProvenanceError::new(
            "repository",
            "could not find reference/upstream-lock.toml and .gitmodules",
        ));
    };
    Ok(root.to_path_buf())
}
