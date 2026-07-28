use super::acquisition::{ReplayOutput, acquire_replay};
use super::support::{bounded_text, json_bytes, sha256, valid_digest};
use super::{
    CatalogFailureBundleRequest, CatalogFailureKind, CatalogRunRequest, Deserialize, OpenOptions,
    Path, PathBuf, Phase13EvidenceError, Phase13EvidenceErrorKind, RIGID_STACK_FIXTURE, Serialize,
    UPSTREAM_REVISION, Write, fs, persist_catalog_failure_bundle,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MaterialsManifest {
    pub(super) schema_version: u32,
    pub(super) target: String,
    pub(super) preset: String,
    pub(super) materials: Vec<MaterialEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MaterialEntry {
    pub(super) kind: String,
    pub(super) identity: String,
}

#[derive(Serialize)]
pub(super) struct WitnessProvenance {
    pub(super) schema_version: u32,
    pub(super) repository_revision: String,
    pub(super) oracle_revision: String,
    pub(super) materials_manifest_sha256: String,
    pub(super) materials_sha256: String,
    pub(super) materials_count: usize,
    pub(super) probe_source_sha256: String,
    pub(super) compiler_id: String,
    pub(super) compiler_version: String,
    pub(super) target: String,
    pub(super) cmake_preset: String,
    pub(super) cmake_target: String,
    pub(super) exact_argv: Vec<String>,
    pub(super) witness_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReplayEvidenceRecord {
    schema_version: u32,
    upstream_revision: String,
    resolved_scenario_path: String,
    sealed_input_sha256: String,
    native_d0_repeat_sha256: [String; 2],
    d1_oracle_identity_sha256: String,
    d1_result: String,
    diagnosis: ReplayDiagnosis,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplayDiagnosis {
    current_resolved_sha256: String,
    current_schema: ReplaySchemaIdentity,
    drift_class: String,
    first_divergence: ReplayFirstDivergence,
    rationale: String,
    reviewed_resolved_sha256: String,
    reviewed_schema: ReplaySchemaIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplaySchemaIdentity {
    #[serde(rename = "catalog_schema_version")]
    catalog_schema: u32,
    #[serde(rename = "checkpoint_schema_version")]
    checkpoint_schema: u32,
    #[serde(rename = "projection_version")]
    projection: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplayFirstDivergence {
    current_value: ReplayDiagnosticValue,
    reviewed_value: ReplayDiagnosticValue,
    semantic_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplayDiagnosticValue {
    presence: String,
    value: serde_json::Value,
}

impl ReplayEvidenceRecord {
    fn validate(&self) -> Result<(), Phase13EvidenceError> {
        let valid = self.schema_version == 1
            && self.upstream_revision == UPSTREAM_REVISION
            && self.resolved_scenario_path == RIGID_STACK_FIXTURE
            && valid_digest(&self.sealed_input_sha256)
            && self
                .native_d0_repeat_sha256
                .iter()
                .all(|digest| valid_digest(digest))
            && self.native_d0_repeat_sha256[0] == self.native_d0_repeat_sha256[1]
            && valid_digest(&self.d1_oracle_identity_sha256)
            && self.d1_result == "match"
            && self.diagnosis.current_resolved_sha256 == self.sealed_input_sha256
            && self.diagnosis.reviewed_resolved_sha256 == self.sealed_input_sha256
            && self.diagnosis.drift_class == "capture_schema_drift"
            && self.diagnosis.current_schema.catalog_schema == 1
            && self.diagnosis.current_schema.checkpoint_schema == 1
            && self.diagnosis.current_schema.projection == "expanded_checkpoint_v1"
            && self.diagnosis.reviewed_schema.catalog_schema == 1
            && self.diagnosis.reviewed_schema.checkpoint_schema == 1
            && self.diagnosis.reviewed_schema.projection == "legacy_physics_v1"
            && self.diagnosis.first_divergence.semantic_path
                == "$.checkpoints[0].debug_primitives.length";
        if valid {
            return Ok(());
        }
        Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            "replay evidence violates the reviewed live-check contract",
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LiveReplayMismatch {
    path: String,
    reviewed: serde_json::Value,
    current: serde_json::Value,
}

impl LiveReplayMismatch {
    #[must_use]
    pub(crate) fn path(&self) -> &str {
        &self.path
    }
}

pub(super) fn compare_live_replay_records(
    reviewed: &serde_json::Value,
    current: &serde_json::Value,
) -> Result<(), LiveReplayMismatch> {
    let maybe_mismatch = first_json_mismatch("", reviewed, current);
    let reviewed_record = serde_json::from_value::<ReplayEvidenceRecord>(reviewed.clone())
        .ok()
        .filter(|record| record.validate().is_ok());
    let current_record = serde_json::from_value::<ReplayEvidenceRecord>(current.clone())
        .ok()
        .filter(|record| record.validate().is_ok());
    if reviewed_record.is_none() || current_record.is_none() {
        return Err(maybe_mismatch.unwrap_or_else(|| LiveReplayMismatch {
            path: String::new(),
            reviewed: reviewed.clone(),
            current: current.clone(),
        }));
    }
    maybe_mismatch.map_or(Ok(()), Err)
}

fn first_json_mismatch(
    path: &str,
    reviewed: &serde_json::Value,
    current: &serde_json::Value,
) -> Option<LiveReplayMismatch> {
    match (reviewed, current) {
        (serde_json::Value::Object(reviewed), serde_json::Value::Object(current)) => {
            for (key, reviewed_value) in reviewed {
                let child_path = format!("{path}/{}", json_pointer_token(key));
                let Some(current_value) = current.get(key) else {
                    return Some(LiveReplayMismatch {
                        path: child_path,
                        reviewed: reviewed_value.clone(),
                        current: serde_json::Value::Null,
                    });
                };
                if let Some(mismatch) =
                    first_json_mismatch(&child_path, reviewed_value, current_value)
                {
                    return Some(mismatch);
                }
            }
            current
                .keys()
                .find(|key| !reviewed.contains_key(*key))
                .map(|key| LiveReplayMismatch {
                    path: format!("{path}/{}", json_pointer_token(key)),
                    reviewed: serde_json::Value::Null,
                    current: current[key].clone(),
                })
        }
        (serde_json::Value::Array(reviewed), serde_json::Value::Array(current)) => {
            for (index, reviewed_value) in reviewed.iter().enumerate() {
                let child_path = format!("{path}/{index}");
                let Some(current_value) = current.get(index) else {
                    return Some(LiveReplayMismatch {
                        path: child_path,
                        reviewed: reviewed_value.clone(),
                        current: serde_json::Value::Null,
                    });
                };
                if let Some(mismatch) =
                    first_json_mismatch(&child_path, reviewed_value, current_value)
                {
                    return Some(mismatch);
                }
            }
            (current.len() > reviewed.len()).then(|| LiveReplayMismatch {
                path: format!("{path}/{}", reviewed.len()),
                reviewed: serde_json::Value::Null,
                current: current[reviewed.len()].clone(),
            })
        }
        _ if reviewed == current => None,
        _ => Some(LiveReplayMismatch {
            path: path.to_owned(),
            reviewed: reviewed.clone(),
            current: current.clone(),
        }),
    }
}

fn json_pointer_token(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}

pub(super) fn persist_live_check_failure(
    repository_root: &Path,
    reviewed: &serde_json::Value,
    current: &serde_json::Value,
    mismatch: &LiveReplayMismatch,
) -> Result<PathBuf, Phase13EvidenceError> {
    let path = repository_root.join("target/phase13-acceptance/failures/live-check-failure.json");
    let reviewed_bytes = serde_json::to_vec(reviewed).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            format!("failed to encode reviewed replay identity: {error}"),
        )
    })?;
    let current_bytes = serde_json::to_vec(current).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            format!("failed to encode current replay identity: {error}"),
        )
    })?;
    let record = serde_json::json!({
        "schema_version": 1,
        "failure_kind": "reviewed_live_replay_mismatch",
        "request_identity": {
            "upstream_revision": current.get("upstream_revision"),
            "resolved_scenario_path": current.get("resolved_scenario_path"),
            "sealed_input_sha256": current.get("sealed_input_sha256"),
        },
        "first_json_pointer": mismatch.path,
        "reviewed_value": bounded_json_value(&mismatch.reviewed),
        "current_value": bounded_json_value(&mismatch.current),
        "expected_record": bounded_json_value(reviewed),
        "observed_record": bounded_json_value(current),
        "reviewed_record_sha256": sha256(&reviewed_bytes),
        "current_record_sha256": sha256(&current_bytes),
    });
    write_failure_record(&path, &record)?;
    Ok(path)
}

pub(super) fn replay_record(
    replay: &ReplayOutput,
) -> Result<ReplayEvidenceRecord, Phase13EvidenceError> {
    let record = ReplayEvidenceRecord {
        schema_version: 1,
        upstream_revision: UPSTREAM_REVISION.to_owned(),
        resolved_scenario_path: RIGID_STACK_FIXTURE.to_owned(),
        sealed_input_sha256: replay.sealed_input_sha256.clone(),
        native_d0_repeat_sha256: replay.native_repeat_sha256.clone(),
        d1_oracle_identity_sha256: replay.oracle_identity_sha256.clone(),
        d1_result: if replay.d1_passed {
            "match"
        } else {
            "mismatch"
        }
        .to_owned(),
        diagnosis: serde_json::from_value(replay.diagnosis.clone()).map_err(|error| {
            Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Protocol,
                format!("live replay diagnosis violates the closed schema: {error}"),
            )
        })?,
    };
    Ok(record)
}

pub(super) fn live_check(repository_root: &Path) -> Result<(), Phase13EvidenceError> {
    let tracked_path =
        repository_root.join("reference/artifacts/catalog/rigid-stack-v1.replay-evidence.json");
    let tracked_bytes = fs::read(&tracked_path).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Filesystem,
            format!("failed to read reviewed replay evidence: {error}"),
        )
    })?;
    if tracked_bytes.len() > 64 * 1024 {
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            "reviewed replay evidence exceeds the live-check bound",
        ));
    }
    let reviewed: serde_json::Value = serde_json::from_slice(&tracked_bytes).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            format!("reviewed replay evidence is invalid JSON: {error}"),
        )
    })?;
    serde_json::from_value::<ReplayEvidenceRecord>(reviewed.clone())
        .map_err(|error| {
            Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Protocol,
                format!("reviewed replay evidence violates the closed schema: {error}"),
            )
        })?
        .validate()?;

    let acquisition = acquire_replay(repository_root, true).map_err(|error| {
        match persist_live_acquisition_failure(repository_root, &error) {
            Ok(_path) => error,
            Err(persist_error) => Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Bundle,
                format!("{error}; additionally failed to persist evidence: {persist_error}"),
            ),
        }
    })?;
    let current_record = replay_record(&acquisition.output)?;
    let current = serde_json::to_value(&current_record).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            format!("failed to encode current replay evidence: {error}"),
        )
    })?;
    if let Err(mismatch) = super::compare_live_replay_records(&reviewed, &current) {
        let path =
            super::persist_live_check_failure(repository_root, &reviewed, &current, &mismatch)?;
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            format!(
                "live replay differs from reviewed evidence at {}; failure evidence: {}",
                mismatch.path(),
                path.display()
            ),
        ));
    }
    println!(
        "phase13 live replay verified: sealed={} D0={} D1={} drift={}",
        current_record.sealed_input_sha256,
        current_record.native_d0_repeat_sha256[0],
        current_record.d1_oracle_identity_sha256,
        current_record.diagnosis.drift_class
    );
    Ok(())
}

pub(super) fn persist_acquisition_failure(
    repository_root: &Path,
    persist_failures: bool,
    request: &CatalogRunRequest,
    kind: CatalogFailureKind,
    message: &str,
    stderr: &[u8],
) -> Phase13EvidenceError {
    let original = Phase13EvidenceError::new(
        Phase13EvidenceErrorKind::Protocol,
        bounded_text(message, 4_096),
    );
    if !persist_failures {
        return original;
    }
    let category = if matches!(
        kind,
        CatalogFailureKind::PhysicsMismatch | CatalogFailureKind::HarnessFailure
    ) {
        CatalogFailureKind::Evidence
    } else {
        kind
    };
    let mut controller = match serde_json::to_vec(&serde_json::json!({
        "controller_state": "acquisition_failure",
        "failure_kind": category,
        "diagnostic": bounded_text(message, 4_096),
    })) {
        Ok(bytes) => bytes,
        Err(error) => {
            return Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Bundle,
                format!("{original}; failed to encode controller evidence: {error}"),
            );
        }
    };
    controller.push(b'\n');
    let bundle = match CatalogFailureBundleRequest::from_harness_failure(
        category,
        request,
        stderr,
        &controller,
    ) {
        Ok(bundle) => bundle,
        Err(error) => {
            return Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Bundle,
                format!("{original}; failed to construct catalog evidence: {error}"),
            );
        }
    };
    if let Err(error) = persist_catalog_failure_bundle(repository_root, &bundle) {
        return Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Bundle,
            format!("{original}; failed to persist catalog evidence: {error}"),
        );
    }
    match persist_live_acquisition_failure(repository_root, &original) {
        Ok(_path) => original,
        Err(error) => Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Bundle,
            format!("{original}; failed to persist acceptance evidence: {error}"),
        ),
    }
}

pub(super) fn persist_live_acquisition_failure(
    repository_root: &Path,
    error: &Phase13EvidenceError,
) -> Result<PathBuf, Phase13EvidenceError> {
    let path = repository_root.join("target/phase13-acceptance/failures/acquisition-failure.json");
    if path.is_file() {
        return Ok(path);
    }
    let record = serde_json::json!({
        "schema_version": 1,
        "failure_kind": "live_replay_acquisition_failure",
        "diagnostic": bounded_text(&error.to_string(), 4_096),
    });
    write_failure_record(&path, &record)?;
    Ok(path)
}

pub(super) fn bounded_json_value(value: &serde_json::Value) -> serde_json::Value {
    match serde_json::to_vec(value) {
        Ok(bytes) if bytes.len() <= 4_096 => value.clone(),
        Ok(bytes) => serde_json::json!({
            "presence": "sha256",
            "bytes": bytes.len(),
            "sha256": sha256(&bytes),
        }),
        Err(error) => serde_json::json!({
            "presence": "encoding_error",
            "diagnostic": bounded_text(&error.to_string(), 256),
        }),
    }
}

pub(super) fn write_failure_record(
    path: &Path,
    record: &serde_json::Value,
) -> Result<(), Phase13EvidenceError> {
    let parent = path.parent().ok_or_else(|| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Filesystem,
            "failure evidence path has no parent",
        )
    })?;
    fs::create_dir_all(parent).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Filesystem,
            format!("failed to create failure evidence root: {error}"),
        )
    })?;
    let bytes = json_bytes(record)?;
    if bytes.len() > 64 * 1024 {
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Bundle,
            "failure evidence exceeds the 64 KiB contract",
        ));
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| {
            Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Filesystem,
                format!("failed to create new failure evidence: {error}"),
            )
        })?;
    file.write_all(&bytes).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Filesystem,
            format!("failed to write failure evidence: {error}"),
        )
    })?;
    file.sync_all().map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Filesystem,
            format!("failed to sync failure evidence: {error}"),
        )
    })
}
