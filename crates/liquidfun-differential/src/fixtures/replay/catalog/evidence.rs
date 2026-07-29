use std::path::Path;

use serde::Deserialize;

use super::{
    CatalogRegressionError, CatalogRegressionErrorKind, MAXIMUM_MANIFEST_BYTES,
    PINNED_UPSTREAM_REVISION, RIGID_STACK_REPLAY_EVIDENCE_PATH, RegressionManifest,
    read_regular_confined,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RigidStackReplayEvidence {
    schema_version: u32,
    upstream_revision: String,
    resolved_scenario_path: String,
    sealed_input_sha256: String,
    native_d0_repeat_sha256: [String; 2],
    d1_oracle_identity_sha256: String,
    d1_result: String,
    diagnosis: serde_json::Value,
}

pub(super) fn validate_rigid_stack_replay_evidence(
    canonical_root: &Path,
    manifest: &RegressionManifest,
) -> Result<(), CatalogRegressionError> {
    let bytes = read_regular_confined(
        canonical_root,
        Path::new(RIGID_STACK_REPLAY_EVIDENCE_PATH),
        MAXIMUM_MANIFEST_BYTES,
        CatalogRegressionErrorKind::InvalidManifest,
    )?;
    let evidence: RigidStackReplayEvidence = serde_json::from_slice(&bytes).map_err(|_error| {
        CatalogRegressionError::new(CatalogRegressionErrorKind::InvalidManifest)
    })?;
    let rigid_stack = manifest
        .entries
        .iter()
        .find(|entry| entry.fixture_id == "rigid-stack-v1")
        .ok_or_else(|| CatalogRegressionError::new(CatalogRegressionErrorKind::InvalidManifest))?;
    let diagnosis = &evidence.diagnosis;
    let reviewed_projection = diagnosis
        .pointer("/reviewed_schema/projection_version")
        .and_then(serde_json::Value::as_str);
    let current_projection = diagnosis
        .pointer("/current_schema/projection_version")
        .and_then(serde_json::Value::as_str);
    let reviewed_resolved = diagnosis
        .get("reviewed_resolved_sha256")
        .and_then(serde_json::Value::as_str);
    let current_resolved = diagnosis
        .get("current_resolved_sha256")
        .and_then(serde_json::Value::as_str);
    let valid_digest = |value: &str| {
        value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    };
    if evidence.schema_version != 1
        || evidence.upstream_revision != PINNED_UPSTREAM_REVISION
        || evidence.resolved_scenario_path != rigid_stack.path
        || evidence.sealed_input_sha256 != rigid_stack.resolved_sha256.as_str()
        || evidence.native_d0_repeat_sha256[0] != evidence.native_d0_repeat_sha256[1]
        || !valid_digest(&evidence.native_d0_repeat_sha256[0])
        || !valid_digest(&evidence.d1_oracle_identity_sha256)
        || evidence.d1_result != "match"
        || diagnosis
            .get("drift_class")
            .and_then(serde_json::Value::as_str)
            != Some("capture_schema_drift")
        || reviewed_projection != Some("legacy_physics_v1")
        || current_projection != Some("expanded_checkpoint_v1")
        || reviewed_resolved != Some(rigid_stack.resolved_sha256.as_str())
        || current_resolved != Some(rigid_stack.resolved_sha256.as_str())
    {
        return Err(CatalogRegressionError::new(
            CatalogRegressionErrorKind::InvalidManifest,
        ));
    }
    Ok(())
}
