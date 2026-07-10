//! Confined filesystem, hashing, diff, and manifest primitives.

use std::{
    collections::BTreeSet,
    fmt::Write as _,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Component, Path, PathBuf},
};

use sha2::{Digest, Sha256};

use super::domain::{
    ArtifactKind, ArtifactManifest, ArtifactRecord, CandidateMetadata, FixtureError,
    MANIFEST_FIELDS, ManifestReviewStatus, REQUIRED_FILES, ReviewMetadata,
};

pub(super) fn destination_path(
    repository_root: &Path,
    metadata: &CandidateMetadata,
) -> Result<PathBuf, FixtureError> {
    validate_identifier(&metadata.scenario_id, "scenario")?;
    let relative = match metadata.artifact_kind {
        ArtifactKind::ReviewedTrace => {
            format!("reference/artifacts/traces/{}.jsonl", metadata.scenario_id)
        }
        ArtifactKind::MinimizedRegression => {
            format!("scenarios/regressions/{}.json", metadata.scenario_id)
        }
    };
    let path = repository_root.join(&relative);
    if Path::new(&relative)
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(FixtureError::PathEscape { path });
    }
    Ok(path)
}

pub(super) fn update_manifest_atomically(
    repository_root: &Path,
    metadata: &CandidateMetadata,
    destination: &Path,
    artifact_hash: &str,
    artifact_id: &str,
) -> Result<(), FixtureError> {
    let lock_path = repository_root.join("reference/artifacts/manifest.toml.lock");
    write_create_new(&lock_path, artifact_id.as_bytes())?;
    let result = update_manifest_locked(
        repository_root,
        metadata,
        destination,
        artifact_hash,
        artifact_id,
    );
    let cleanup = fs::remove_file(lock_path);
    match (result, cleanup) {
        (Err(error), _) => Err(error),
        (Ok(()), Ok(())) => Ok(()),
        (Ok(()), Err(error)) => Err(FixtureError::Io(error)),
    }
}

fn update_manifest_locked(
    repository_root: &Path,
    metadata: &CandidateMetadata,
    destination: &Path,
    artifact_hash: &str,
    artifact_id: &str,
) -> Result<(), FixtureError> {
    let mut manifest = read_manifest(repository_root)?;
    let canonical_root = fs::canonicalize(repository_root)?;
    let relative =
        destination
            .strip_prefix(canonical_root)
            .map_err(|_| FixtureError::PathEscape {
                path: destination.to_path_buf(),
            })?;
    let path = relative.to_string_lossy().replace('\\', "/");
    if manifest.artifacts.iter().any(|record| record.path == path) {
        return Err(FixtureError::DestinationExists {
            path: destination.to_path_buf(),
        });
    }
    manifest.artifacts.push(ArtifactRecord {
        path,
        sha256: artifact_hash.to_owned(),
        generator_revision: metadata.generator_revision.clone(),
        oracle_revision: metadata.oracle_revision.clone(),
        preset: metadata.preset.clone(),
        compiler: metadata.compiler.clone(),
        target: metadata.target.clone(),
        flags: metadata.flags.clone(),
        notice_refs: vec!["THIRD_PARTY_NOTICES.md".to_owned()],
        review_status: ManifestReviewStatus::Reviewed,
    });
    manifest
        .artifacts
        .sort_by(|left, right| left.path.cmp(&right.path));
    let manifest_path = repository_root.join("reference/artifacts/manifest.toml");
    let temporary = manifest_path.with_file_name(format!("manifest.toml.{artifact_id}.tmp"));
    write_create_new(&temporary, toml::to_string_pretty(&manifest)?.as_bytes())?;
    fs::rename(&temporary, &manifest_path)?;
    sync_directory(
        manifest_path
            .parent()
            .ok_or_else(|| FixtureError::PathEscape {
                path: manifest_path.clone(),
            })?,
    )
}

pub(super) fn read_manifest(repository_root: &Path) -> Result<ArtifactManifest, FixtureError> {
    let path = repository_root.join("reference/artifacts/manifest.toml");
    reject_symlink_chain(repository_root, &path)?;
    let text = fs::read_to_string(path)?;
    let manifest: ArtifactManifest = toml::from_str(&text)?;
    if manifest.schema_version != 1
        || manifest.record_schema_version != 1
        || manifest.record_fields != MANIFEST_FIELDS
        || !is_revision(&manifest.oracle_revision)
    {
        return Err(FixtureError::Manifest(
            "schema, fields, or oracle revision mismatch".to_owned(),
        ));
    }
    Ok(manifest)
}

pub(super) fn validate_candidate_entries(directory: &Path) -> Result<(), FixtureError> {
    let mut expected = REQUIRED_FILES.into_iter().collect::<BTreeSet<_>>();
    expected.insert("candidate.toml");
    expected.insert("review.toml");
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.file_type().is_symlink() {
            return Err(FixtureError::Symlink { path: entry.path() });
        }
        let name = entry.file_name();
        let name = name
            .to_str()
            .ok_or_else(|| FixtureError::Replay("candidate filename is not UTF-8".to_owned()))?;
        if !metadata.is_file() || !expected.contains(name) {
            return Err(FixtureError::Replay(format!(
                "unexpected candidate entry `{name}`"
            )));
        }
    }
    Ok(())
}

pub(super) fn ensure_directory_chain(
    repository_root: &Path,
    components: &[&str],
) -> Result<PathBuf, FixtureError> {
    let canonical_root = fs::canonicalize(repository_root)?;
    let mut path = repository_root.to_path_buf();
    for component in components {
        validate_identifier(component, "path component")?;
        path.push(component);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(FixtureError::Symlink { path });
            }
            Ok(metadata) if !metadata.is_dir() => return Err(FixtureError::PathEscape { path }),
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => fs::create_dir(&path)?,
            Err(error) => return Err(FixtureError::Io(error)),
        }
    }
    let canonical = fs::canonicalize(path)?;
    if !canonical.starts_with(canonical_root) {
        return Err(FixtureError::PathEscape { path: canonical });
    }
    Ok(canonical)
}

pub(super) fn reject_symlink_chain(
    repository_root: &Path,
    path: &Path,
) -> Result<(), FixtureError> {
    let canonical_root = fs::canonicalize(repository_root)?;
    let relative = path
        .strip_prefix(repository_root)
        .map_err(|_| FixtureError::PathEscape {
            path: path.to_path_buf(),
        })?;
    let mut cursor = repository_root.to_path_buf();
    for component in relative.components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(FixtureError::PathEscape {
                path: path.to_path_buf(),
            });
        }
        cursor.push(component.as_os_str());
        match fs::symlink_metadata(&cursor) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(FixtureError::Symlink { path: cursor });
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => break,
            Err(error) => return Err(FixtureError::Io(error)),
        }
    }
    if let Some(existing) = path.ancestors().find(|ancestor| ancestor.exists()) {
        let canonical = fs::canonicalize(existing)?;
        if !canonical.starts_with(canonical_root) {
            return Err(FixtureError::PathEscape { path: canonical });
        }
    }
    Ok(())
}

pub(super) fn read_required(
    directory: &Path,
    name: &'static str,
    limit: usize,
) -> Result<Vec<u8>, FixtureError> {
    let path = directory.join(name);
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(FixtureError::Symlink { path });
        }
        Ok(metadata) if !metadata.is_file() => {
            return Err(FixtureError::MissingCandidateFile { file: name });
        }
        Ok(metadata) if metadata.len() > limit as u64 => {
            return Err(FixtureError::SizeLimit { field: name, limit });
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(FixtureError::MissingCandidateFile { file: name });
        }
        Err(error) => return Err(FixtureError::Io(error)),
    }
    Ok(fs::read(path)?)
}

pub(super) fn read_confined_file(
    path: &Path,
    limit: usize,
    field: &'static str,
) -> Result<Vec<u8>, FixtureError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(FixtureError::Symlink {
            path: path.to_path_buf(),
        });
    }
    if !metadata.is_file() {
        return Err(FixtureError::MissingCandidateFile { file: field });
    }
    if metadata.len() > limit as u64 {
        return Err(FixtureError::SizeLimit { field, limit });
    }
    Ok(fs::read(path)?)
}

pub(super) fn write_create_new(path: &Path, bytes: &[u8]) -> Result<(), FixtureError> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

pub(super) fn sync_directory(path: &Path) -> Result<(), FixtureError> {
    match File::open(path).and_then(|directory| directory.sync_all()) {
        Ok(()) => Ok(()),
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::PermissionDenied
                    | io::ErrorKind::InvalidInput
                    | io::ErrorKind::Other
            ) =>
        {
            Ok(())
        }
        Err(error) => Err(FixtureError::Io(error)),
    }
}

pub(super) fn candidate_sha256(metadata: &CandidateMetadata) -> String {
    let mut digest = Sha256::new();
    let artifact_kind = match metadata.artifact_kind {
        ArtifactKind::ReviewedTrace => "reviewed_trace",
        ArtifactKind::MinimizedRegression => "minimized_regression",
    };
    for value in [
        metadata.artifact_id.as_str(),
        artifact_kind,
        metadata.scenario_id.as_str(),
        metadata.scenario_sha256.as_str(),
        metadata.source_json.as_str(),
        metadata.tolerance_profile_sha256.as_str(),
        metadata.oracle_revision.as_str(),
        metadata.preset.as_str(),
        metadata.session_profile.as_str(),
        metadata.compiler.as_str(),
        metadata.target.as_str(),
        metadata.generator_revision.as_str(),
        metadata.review_status.as_str(),
        metadata.request_sha256.as_str(),
        metadata.trace_sha256.as_str(),
        metadata.report_sha256.as_str(),
        metadata.identity_sha256.as_str(),
        metadata.stderr_sha256.as_str(),
        metadata.scenario_bytes_sha256.as_str(),
        metadata.failure_signature_json.as_deref().unwrap_or(""),
    ] {
        digest.update(value.len().to_be_bytes());
        digest.update(value.as_bytes());
    }
    for version in [
        metadata.schema_version,
        metadata.protocol_version,
        metadata.scenario_schema_version,
        metadata.trace_schema_version,
        metadata.tolerance_profile_version,
    ] {
        digest.update(version.to_be_bytes());
    }
    for flag in &metadata.flags {
        digest.update(flag.len().to_be_bytes());
        digest.update(flag.as_bytes());
    }
    format!("{:x}", digest.finalize())
}

pub(super) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn deterministic_diff(accepted: &[u8], candidate: &[u8]) -> String {
    if accepted == candidate {
        return "no differences\n".to_owned();
    }
    let accepted = String::from_utf8_lossy(accepted);
    let candidate = String::from_utf8_lossy(candidate);
    let mut diff = String::from("--- accepted\n+++ candidate\n");
    let accepted_lines = accepted.lines().collect::<Vec<_>>();
    let candidate_lines = candidate.lines().collect::<Vec<_>>();
    for index in 0..accepted_lines.len().max(candidate_lines.len()).min(256) {
        let left = accepted_lines.get(index).copied().unwrap_or("");
        let right = candidate_lines.get(index).copied().unwrap_or("");
        if left != right {
            write!(diff, "@@ line {} @@\n-{left}\n+{right}\n", index + 1)
                .expect("writing into String cannot fail");
        }
    }
    diff
}

pub(super) fn validate_review(review: ReviewMetadata<'_>) -> Result<(), FixtureError> {
    validate_nonempty(review.reviewer, "reviewer")?;
    validate_nonempty(review.reviewed_at, "reviewed_at")?;
    if !review.reviewed_at.contains('T') || !review.reviewed_at.ends_with('Z') {
        return Err(FixtureError::Replay(
            "review timestamp must be explicit UTC RFC3339 form".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn validate_identifier(value: &str, field: &'static str) -> Result<(), FixtureError> {
    let valid = !value.is_empty()
        && value.len() <= 128
        && value.bytes().enumerate().all(|(index, byte)| {
            (index > 0 && matches!(byte, b'.' | b'_' | b'-'))
                || byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
        });
    if valid {
        return Ok(());
    }
    Err(FixtureError::InvalidIdentifier {
        field,
        value: value.to_owned(),
    })
}

fn validate_nonempty(value: &str, field: &'static str) -> Result<(), FixtureError> {
    if !value.trim().is_empty() && !value.chars().any(char::is_control) {
        return Ok(());
    }
    Err(FixtureError::InvalidIdentifier {
        field,
        value: value.to_owned(),
    })
}

pub(super) fn validate_revision(value: &str) -> Result<(), FixtureError> {
    if is_revision(value) {
        return Ok(());
    }
    Err(FixtureError::InvalidIdentifier {
        field: "generator revision",
        value: value.to_owned(),
    })
}

fn is_revision(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

pub(super) fn validate_preset_profile(preset: &str, profile: &str) -> Result<(), FixtureError> {
    if !matches!(
        preset,
        "oracle-debug" | "oracle-release" | "oracle-asan-ubsan"
    ) {
        return Err(FixtureError::InvalidIdentifier {
            field: "preset",
            value: preset.to_owned(),
        });
    }
    if !matches!(profile, "one-shot" | "reuse" | "sanitizer") {
        return Err(FixtureError::InvalidIdentifier {
            field: "session profile",
            value: profile.to_owned(),
        });
    }
    if profile == "sanitizer" && preset != "oracle-asan-ubsan" {
        return Err(FixtureError::Replay(
            "sanitizer profile requires the sanitizer preset".to_owned(),
        ));
    }
    Ok(())
}

pub(super) fn enforce_size(
    field: &'static str,
    bytes: &[u8],
    limit: usize,
) -> Result<(), FixtureError> {
    if bytes.len() <= limit {
        return Ok(());
    }
    Err(FixtureError::SizeLimit { field, limit })
}
