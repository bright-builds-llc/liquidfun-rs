//! Strict validation and recovery of persisted catalog replay authority.

use std::{fs, path::Path};

use liquidfun_test_protocol::{
    CanonicalCheckpoint, CheckpointDeclaration, HarnessLimits, ResolvedScenario, ScenarioActionId,
    Sha256Hex, decode_canonical_checkpoint_jsonl, decode_resolved_scenario,
};
use serde::Deserialize;

use crate::CatalogFailureKind;

use super::{
    MAXIMUM_FIELD_BYTES, Manifest, REQUIRED_FILES, SCHEMA_VERSION, digest, ensure_catalog_root,
    evidence_error, read_bounded, reject_symlink,
};
use crate::failure_bundle::FailureBundleError;

/// Exact replay authority recovered from a verified catalog bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogBundleReplay {
    resolved_bytes: Box<[u8]>,
    resolved_sha256: Sha256Hex,
}

impl CatalogBundleReplay {
    /// Returns exact persisted canonical resolved bytes.
    #[must_use]
    pub fn resolved_bytes(&self) -> &[u8] {
        &self.resolved_bytes
    }

    /// Returns the verified SHA-256 identity of those bytes.
    #[must_use]
    pub const fn resolved_sha256(&self) -> &Sha256Hex {
        &self.resolved_sha256
    }
}

pub(crate) fn replay_catalog_bundle(
    repository_root: &Path,
    directory: &Path,
) -> Result<CatalogBundleReplay, FailureBundleError> {
    let root = ensure_catalog_root(repository_root)?;
    reject_symlink(directory)?;
    let canonical = fs::canonicalize(directory)?;
    if !canonical.starts_with(&root) || canonical == root {
        return Err(evidence_error("catalog bundle path escaped evidence root"));
    }
    let manifest_bytes = read_bounded(&canonical.join("manifest.json"), MAXIMUM_FIELD_BYTES)?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)?;
    if manifest.schema_version != SCHEMA_VERSION
        || !matches!(
            manifest.result_kind,
            CatalogFailureKind::HarnessFailure | CatalogFailureKind::PhysicsMismatch
        )
        || !manifest_has_exact_files(&manifest)
    {
        return Err(evidence_error("unsupported catalog bundle schema"));
    }
    validate_directory_entries(&canonical, &manifest)?;
    for (name, entry) in &manifest.files {
        let path = canonical.join(name);
        reject_symlink(&path)?;
        let bytes = read_bounded(&path, MAXIMUM_FIELD_BYTES)?;
        if bytes.len() != entry.bytes || digest(&bytes) != entry.sha256 {
            return Err(evidence_error("catalog bundle file hash mismatch"));
        }
    }
    let resolved_bytes = read_bounded(&canonical.join("resolved.json"), MAXIMUM_FIELD_BYTES)?;
    let resolved = decode_resolved_scenario(&resolved_bytes, &manifest.resolved_sha256)
        .map_err(|_error| evidence_error("catalog bundle resolved bytes are invalid"))?;
    validate_replay_semantics(&canonical, &manifest, &resolved)?;
    Ok(CatalogBundleReplay {
        resolved_bytes: resolved_bytes.into_boxed_slice(),
        resolved_sha256: manifest.resolved_sha256,
    })
}

fn validate_directory_entries(
    directory: &Path,
    manifest: &Manifest,
) -> Result<(), FailureBundleError> {
    let mut expected = manifest.files.keys().cloned().collect::<Vec<_>>();
    expected.push("manifest.json".to_owned());
    expected.sort_unstable();
    let mut actual = Vec::with_capacity(expected.len());
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_name| evidence_error("catalog bundle filename is not UTF-8"))?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(evidence_error("catalog bundle entry is unsafe"));
        }
        actual.push(name);
    }
    actual.sort_unstable();
    if actual != expected {
        return Err(evidence_error(
            "catalog bundle contains unknown or missing entries",
        ));
    }
    Ok(())
}

fn manifest_has_exact_files(manifest: &Manifest) -> bool {
    let mut expected = REQUIRED_FILES
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<Vec<_>>();
    if manifest.result_kind == CatalogFailureKind::PhysicsMismatch {
        expected.push("first-divergence.json".to_owned());
    }
    expected.sort_unstable();
    manifest.files.keys().cloned().collect::<Vec<_>>() == expected
        && manifest.files.keys().all(|name| {
            !name.is_empty()
                && !name.contains('/')
                && !name.contains('\\')
                && name != "."
                && name != ".."
        })
}

fn validate_replay_semantics(
    directory: &Path,
    manifest: &Manifest,
    resolved: &ResolvedScenario,
) -> Result<(), FailureBundleError> {
    let action_log: Vec<ScenarioActionId> = read_json(directory, "action-log.json")?;
    if action_log
        != resolved
            .actions()
            .iter()
            .map(|action| action.action_id().clone())
            .collect::<Vec<_>>()
    {
        return Err(evidence_error(
            "catalog action log disagrees with resolved bytes",
        ));
    }
    let schedule: Vec<CheckpointDeclaration> = read_json(directory, "checkpoint-schedule.json")?;
    if schedule != resolved.checkpoints() {
        return Err(evidence_error(
            "catalog checkpoint schedule disagrees with resolved bytes",
        ));
    }
    let native = decode_checkpoint_array(directory, "native-checkpoints.json")?;
    let oracle = decode_checkpoint_array(directory, "oracle-checkpoints.json")?;
    if native.len() != schedule.len() || oracle.len() != schedule.len() {
        return Err(evidence_error(
            "catalog checkpoint count disagrees with schedule",
        ));
    }
    let request_id = liquidfun_test_protocol::RequestId::new(&manifest.request_id)
        .map_err(|_error| evidence_error("catalog bundle request identity is invalid"))?;
    for ((declaration, rust), cpp) in schedule.iter().zip(&native).zip(&oracle) {
        for checkpoint in [rust, cpp] {
            if checkpoint.request_id() != &request_id
                || checkpoint.resolved_sha256() != &manifest.resolved_sha256
                || checkpoint.checkpoint_id() != declaration.checkpoint_id()
            {
                return Err(evidence_error("catalog checkpoint identity mismatch"));
            }
        }
    }
    let comparison: serde_json::Value = read_json(directory, "comparison.json")?;
    if comparison.as_array().map(Vec::len) != Some(schedule.len()) {
        return Err(evidence_error("catalog comparison evidence is incomplete"));
    }
    for name in [
        "native-identity.json",
        "oracle-identity.json",
        "controller-state.json",
    ] {
        let value: serde_json::Value = read_json(directory, name)?;
        if !value.is_object() {
            return Err(evidence_error(
                "catalog structured evidence is not an object",
            ));
        }
    }
    Ok(())
}

fn read_json<T: for<'de> Deserialize<'de>>(
    directory: &Path,
    name: &str,
) -> Result<T, FailureBundleError> {
    serde_json::from_slice(&read_bounded(&directory.join(name), MAXIMUM_FIELD_BYTES)?)
        .map_err(FailureBundleError::from)
}

fn decode_checkpoint_array(
    directory: &Path,
    name: &str,
) -> Result<Vec<CanonicalCheckpoint>, FailureBundleError> {
    let values: Vec<serde_json::Value> = read_json(directory, name)?;
    values
        .into_iter()
        .map(|value| {
            let mut bytes = serde_json::to_vec(&value)?;
            bytes.push(b'\n');
            decode_canonical_checkpoint_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
                .map_err(|error| evidence_error(&error.to_string()))
        })
        .collect()
}
