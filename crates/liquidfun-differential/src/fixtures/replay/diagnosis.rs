//! Deterministic replay drift diagnosis over sealed inputs and semantic documents.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use liquidfun_test_protocol::{
    CanonicalCheckpoint, CheckpointId, CheckpointPosition, CheckpointProfileName,
    CheckpointSchemaVersion, CheckpointSet, DebugPrimitiveRecord, FloatBits, NumericObservation,
    OrderedOccurrence, ProtocolVersion, RequestId, Sha256Hex, StructuralObservation,
};

const SUPPORTED_CATALOG_SCHEMA_VERSION: u32 = 1;
const SUPPORTED_CHECKPOINT_SCHEMA_VERSION: u32 = 1;

/// Mutually exclusive authority boundaries for one replay divergence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayDriftClass {
    /// The current resolved scenario bytes differ from the reviewed sealed bytes.
    ResolvedScenarioDrift,
    /// Parity-bearing state or event semantics differ for identical resolved bytes.
    PhysicsDrift,
    /// Physics semantics match but the expanded checkpoint contract differs.
    CaptureSchemaDrift,
}

/// Closed semantic projections used to distinguish physics from capture expansion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplayProjectionVersion {
    /// Historical parity-bearing checkpoint fields, excluding later diagnostics.
    LegacyPhysicsV1,
    /// Complete current checkpoint fields, including renderer-neutral diagnostics.
    ExpandedCheckpointV1,
}

/// Version identity attached to both sides of every replay diagnosis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplaySchemaIdentity {
    #[serde(rename = "catalog_schema_version")]
    catalog_schema: u32,
    #[serde(rename = "checkpoint_schema_version")]
    checkpoint_schema: u32,
    #[serde(rename = "projection_version")]
    projection: ReplayProjectionVersion,
}

impl ReplaySchemaIdentity {
    /// Constructs a schema identity for fail-closed validation during diagnosis.
    #[must_use]
    pub const fn new(
        catalog_schema_version: u32,
        checkpoint_schema_version: u32,
        projection_version: ReplayProjectionVersion,
    ) -> Self {
        Self {
            catalog_schema: catalog_schema_version,
            checkpoint_schema: checkpoint_schema_version,
            projection: projection_version,
        }
    }

    /// Returns the catalog schema version.
    #[must_use]
    pub const fn catalog_schema_version(self) -> u32 {
        self.catalog_schema
    }

    /// Returns the checkpoint schema version.
    #[must_use]
    pub const fn checkpoint_schema_version(self) -> u32 {
        self.checkpoint_schema
    }

    /// Returns the semantic projection version.
    #[must_use]
    pub const fn projection_version(self) -> ReplayProjectionVersion {
        self.projection
    }
}

/// One side's versioned parity-bearing and expanded semantic documents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaySemanticDocument {
    schema: ReplaySchemaIdentity,
    physics_projection: Value,
    expanded_checkpoint: Value,
}

impl ReplaySemanticDocument {
    /// Constructs one already bounded semantic diagnosis input.
    #[must_use]
    pub const fn new(
        schema: ReplaySchemaIdentity,
        physics_projection: Value,
        expanded_checkpoint: Value,
    ) -> Self {
        Self {
            schema,
            physics_projection,
            expanded_checkpoint,
        }
    }

    /// Returns the document's schema identity.
    #[must_use]
    pub const fn schema(&self) -> &ReplaySchemaIdentity {
        &self.schema
    }
}

/// A present JSON value or explicit absence at one semantic path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "presence", content = "value", rename_all = "snake_case")]
pub enum ReplaySemanticValue {
    /// The semantic path is absent on this side.
    Missing,
    /// The semantic path contains this engine-neutral JSON value.
    Json(Value),
}

/// The deterministic first changed semantic path and both compared values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayFirstDivergence {
    semantic_path: Box<str>,
    reviewed_value: ReplaySemanticValue,
    current_value: ReplaySemanticValue,
}

impl ReplayFirstDivergence {
    /// Returns the stable JSON-style semantic path.
    #[must_use]
    pub fn semantic_path(&self) -> &str {
        &self.semantic_path
    }

    /// Returns the reviewed value or explicit absence.
    #[must_use]
    pub const fn reviewed_value(&self) -> &ReplaySemanticValue {
        &self.reviewed_value
    }

    /// Returns the current value or explicit absence.
    #[must_use]
    pub const fn current_value(&self) -> &ReplaySemanticValue {
        &self.current_value
    }
}

/// Complete evidence-derived replay diagnosis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayDiagnosis {
    drift_class: ReplayDriftClass,
    first_divergence: ReplayFirstDivergence,
    reviewed_schema: ReplaySchemaIdentity,
    current_schema: ReplaySchemaIdentity,
    reviewed_resolved_sha256: Sha256Hex,
    current_resolved_sha256: Sha256Hex,
    rationale: Box<str>,
}

impl ReplayDiagnosis {
    /// Returns the mutually exclusive drift classification.
    #[must_use]
    pub const fn drift_class(&self) -> ReplayDriftClass {
        self.drift_class
    }

    /// Returns the deterministic first changed semantic path and values.
    #[must_use]
    pub const fn first_divergence(&self) -> &ReplayFirstDivergence {
        &self.first_divergence
    }

    /// Returns the reviewed semantic schema identity.
    #[must_use]
    pub const fn reviewed_schema(&self) -> &ReplaySchemaIdentity {
        &self.reviewed_schema
    }

    /// Returns the current semantic schema identity.
    #[must_use]
    pub const fn current_schema(&self) -> &ReplaySchemaIdentity {
        &self.current_schema
    }

    /// Returns the digest of the reviewed sealed resolved bytes.
    #[must_use]
    pub const fn reviewed_resolved_sha256(&self) -> &Sha256Hex {
        &self.reviewed_resolved_sha256
    }

    /// Returns the digest of the current sealed resolved bytes.
    #[must_use]
    pub const fn current_resolved_sha256(&self) -> &Sha256Hex {
        &self.current_resolved_sha256
    }

    /// Returns the deterministic evidence rationale for the classification.
    #[must_use]
    pub fn rationale(&self) -> &str {
        &self.rationale
    }
}

/// Stable fail-closed diagnosis error categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayDiagnosisErrorKind {
    /// A catalog or checkpoint schema version is not supported.
    UnsupportedSchema,
    /// The reviewed/current projection transition cannot be compared safely.
    IncomparableSchema,
    /// Changed resolved bytes are not valid engine-neutral JSON.
    MalformedResolvedDocument,
    /// A validated checkpoint could not be encoded into a semantic document.
    SemanticEncoding,
}

/// A harness/schema failure that cannot become a drift classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("replay diagnosis failure: {kind:?}")]
pub struct ReplayDiagnosisError {
    kind: ReplayDiagnosisErrorKind,
}

impl ReplayDiagnosisError {
    const fn new(kind: ReplayDiagnosisErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the stable harness/schema failure category.
    #[must_use]
    pub const fn kind(self) -> ReplayDiagnosisErrorKind {
        self.kind
    }
}

/// Diagnoses resolved-input, physics, then capture-schema drift in authority order.
///
/// Exact resolved bytes are compared before either checkpoint projection. Identical inputs then
/// compare the parity-bearing projection before the expanded capture contract, ensuring new
/// diagnostics cannot conceal a physics difference.
///
/// # Errors
///
/// Returns [`ReplayDiagnosisError`] for unsupported or incomparable schemas and for malformed
/// changed resolved documents.
pub fn diagnose_replay_drift(
    reviewed_resolved_bytes: &[u8],
    current_resolved_bytes: &[u8],
    reviewed: &ReplaySemanticDocument,
    current: &ReplaySemanticDocument,
) -> Result<Option<ReplayDiagnosis>, ReplayDiagnosisError> {
    validate_schema_pair(reviewed.schema, current.schema)?;
    let reviewed_resolved_sha256 = digest(reviewed_resolved_bytes);
    let current_resolved_sha256 = digest(current_resolved_bytes);

    if reviewed_resolved_bytes != current_resolved_bytes {
        let reviewed_value = serde_json::from_slice(reviewed_resolved_bytes).map_err(|_error| {
            ReplayDiagnosisError::new(ReplayDiagnosisErrorKind::MalformedResolvedDocument)
        })?;
        let current_value = serde_json::from_slice(current_resolved_bytes).map_err(|_error| {
            ReplayDiagnosisError::new(ReplayDiagnosisErrorKind::MalformedResolvedDocument)
        })?;
        let first_divergence =
            first_divergence(&reviewed_value, &current_value).unwrap_or_else(root_byte_drift);
        return Ok(Some(ReplayDiagnosis {
            drift_class: ReplayDriftClass::ResolvedScenarioDrift,
            first_divergence,
            reviewed_schema: reviewed.schema,
            current_schema: current.schema,
            reviewed_resolved_sha256,
            current_resolved_sha256,
            rationale: "sealed resolved bytes differ before checkpoint comparison".into(),
        }));
    }

    if let Some(first_divergence) =
        first_divergence(&reviewed.physics_projection, &current.physics_projection)
    {
        return Ok(Some(ReplayDiagnosis {
            drift_class: ReplayDriftClass::PhysicsDrift,
            first_divergence,
            reviewed_schema: reviewed.schema,
            current_schema: current.schema,
            reviewed_resolved_sha256,
            current_resolved_sha256,
            rationale: "identical sealed inputs diverge in the parity-bearing projection".into(),
        }));
    }

    if let Some(first_divergence) =
        first_divergence(&reviewed.expanded_checkpoint, &current.expanded_checkpoint)
    {
        return Ok(Some(ReplayDiagnosis {
            drift_class: ReplayDriftClass::CaptureSchemaDrift,
            first_divergence,
            reviewed_schema: reviewed.schema,
            current_schema: current.schema,
            reviewed_resolved_sha256,
            current_resolved_sha256,
            rationale:
                "identical sealed inputs and physics projection diverge only in expanded capture fields"
                    .into(),
        }));
    }

    Ok(None)
}

fn validate_schema_pair(
    reviewed: ReplaySchemaIdentity,
    current: ReplaySchemaIdentity,
) -> Result<(), ReplayDiagnosisError> {
    for schema in [reviewed, current] {
        if schema.catalog_schema != SUPPORTED_CATALOG_SCHEMA_VERSION
            || schema.checkpoint_schema != SUPPORTED_CHECKPOINT_SCHEMA_VERSION
        {
            return Err(ReplayDiagnosisError::new(
                ReplayDiagnosisErrorKind::UnsupportedSchema,
            ));
        }
    }
    if matches!(
        (reviewed.projection, current.projection),
        (
            ReplayProjectionVersion::ExpandedCheckpointV1,
            ReplayProjectionVersion::LegacyPhysicsV1
        )
    ) {
        return Err(ReplayDiagnosisError::new(
            ReplayDiagnosisErrorKind::IncomparableSchema,
        ));
    }
    Ok(())
}

fn digest(bytes: &[u8]) -> Sha256Hex {
    Sha256Hex::from_digest(Sha256::digest(bytes).into())
}

fn first_divergence(reviewed: &Value, current: &Value) -> Option<ReplayFirstDivergence> {
    first_divergence_at("$", Some(reviewed), Some(current))
}

fn first_divergence_at(
    path: &str,
    maybe_reviewed: Option<&Value>,
    maybe_current: Option<&Value>,
) -> Option<ReplayFirstDivergence> {
    match (maybe_reviewed, maybe_current) {
        (Some(reviewed), Some(current)) if reviewed == current => None,
        (Some(Value::Object(reviewed)), Some(Value::Object(current))) => {
            for key in ordered_keys(reviewed, current) {
                let child_path = format!("{path}.{key}");
                if let Some(divergence) =
                    first_divergence_at(&child_path, reviewed.get(key), current.get(key))
                {
                    return Some(divergence);
                }
            }
            None
        }
        (Some(Value::Array(reviewed)), Some(Value::Array(current))) => {
            if reviewed.len() != current.len() {
                return Some(ReplayFirstDivergence {
                    semantic_path: format!("{path}.length").into(),
                    reviewed_value: ReplaySemanticValue::Json(Value::from(reviewed.len())),
                    current_value: ReplaySemanticValue::Json(Value::from(current.len())),
                });
            }
            for index in 0..reviewed.len() {
                let child_path = format!("{path}[{index}]");
                if let Some(divergence) =
                    first_divergence_at(&child_path, reviewed.get(index), current.get(index))
                {
                    return Some(divergence);
                }
            }
            None
        }
        (maybe_reviewed, maybe_current) => Some(ReplayFirstDivergence {
            semantic_path: path.into(),
            reviewed_value: semantic_value(maybe_reviewed),
            current_value: semantic_value(maybe_current),
        }),
    }
}

fn ordered_keys<'a>(
    reviewed: &'a serde_json::Map<String, Value>,
    current: &'a serde_json::Map<String, Value>,
) -> Vec<&'a str> {
    const SEMANTIC_ORDER: [&str; 18] = [
        "identity",
        "entities",
        "actions",
        "checkpoints",
        "protocol_version",
        "record_kind",
        "checkpoint_schema_version",
        "request_id",
        "resolved_sha256",
        "checkpoint_id",
        "position",
        "simulation_time_bits",
        "observations",
        "numeric_observations",
        "ordered_occurrences",
        "unordered_sets",
        "debug_primitives",
        "profile_names",
    ];
    let keys = reviewed
        .keys()
        .chain(current.keys())
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut ordered = SEMANTIC_ORDER
        .into_iter()
        .filter(|key| keys.contains(key))
        .collect::<Vec<_>>();
    ordered.extend(keys.into_iter().filter(|key| !SEMANTIC_ORDER.contains(key)));
    ordered
}

fn semantic_value(maybe_value: Option<&Value>) -> ReplaySemanticValue {
    maybe_value.map_or(ReplaySemanticValue::Missing, |value| {
        ReplaySemanticValue::Json(value.clone())
    })
}

fn root_byte_drift() -> ReplayFirstDivergence {
    ReplayFirstDivergence {
        semantic_path: "$".into(),
        reviewed_value: ReplaySemanticValue::Missing,
        current_value: ReplaySemanticValue::Missing,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CheckpointSemanticDocuments {
    pub(crate) legacy_physics_sha256: Sha256Hex,
    pub(crate) physics_projection: Value,
    pub(crate) expanded_checkpoint: Value,
}

pub(crate) fn checkpoint_semantic_documents(
    checkpoints: &[CanonicalCheckpoint],
) -> Result<CheckpointSemanticDocuments, ReplayDiagnosisError> {
    let mut legacy_hasher = Sha256::new();
    let mut physics_projection = Vec::with_capacity(checkpoints.len());
    let mut expanded_checkpoint = Vec::with_capacity(checkpoints.len());
    for checkpoint in checkpoints {
        let projection = LegacyCheckpointProjection::from(checkpoint);
        let mut bytes = serde_json::to_vec(&projection).map_err(|_error| {
            ReplayDiagnosisError::new(ReplayDiagnosisErrorKind::SemanticEncoding)
        })?;
        bytes.push(b'\n');
        legacy_hasher.update(bytes);
        physics_projection.push(serde_json::to_value(projection).map_err(|_error| {
            ReplayDiagnosisError::new(ReplayDiagnosisErrorKind::SemanticEncoding)
        })?);
        expanded_checkpoint.push(serde_json::to_value(checkpoint).map_err(|_error| {
            ReplayDiagnosisError::new(ReplayDiagnosisErrorKind::SemanticEncoding)
        })?);
    }
    Ok(CheckpointSemanticDocuments {
        legacy_physics_sha256: Sha256Hex::from_digest(legacy_hasher.finalize().into()),
        physics_projection: serde_json::json!({"checkpoints": physics_projection}),
        expanded_checkpoint: serde_json::json!({"checkpoints": expanded_checkpoint}),
    })
}

/// Returns the reviewed legacy physics identity for canonical checkpoints.
///
/// This preserves the historical empty debug-primitive array while retaining every other
/// checkpoint field exactly.
///
/// # Errors
///
/// Returns [`ReplayDiagnosisError`] when a checkpoint cannot be encoded into the versioned
/// legacy projection.
pub fn legacy_physics_checkpoint_sha256(
    checkpoints: &[CanonicalCheckpoint],
) -> Result<Sha256Hex, ReplayDiagnosisError> {
    checkpoint_semantic_documents(checkpoints).map(|documents| documents.legacy_physics_sha256)
}

#[derive(Serialize)]
struct LegacyCheckpointProjection<'a> {
    protocol_version: ProtocolVersion,
    record_kind: &'static str,
    checkpoint_schema_version: CheckpointSchemaVersion,
    request_id: &'a RequestId,
    resolved_sha256: &'a Sha256Hex,
    checkpoint_id: &'a CheckpointId,
    position: &'a CheckpointPosition,
    simulation_time_bits: FloatBits,
    observations: &'a [StructuralObservation],
    numeric_observations: &'a [NumericObservation],
    ordered_occurrences: &'a [OrderedOccurrence],
    unordered_sets: &'a [CheckpointSet],
    debug_primitives: &'static [DebugPrimitiveRecord],
    profile_names: &'a [CheckpointProfileName],
}

impl<'a> From<&'a CanonicalCheckpoint> for LegacyCheckpointProjection<'a> {
    fn from(checkpoint: &'a CanonicalCheckpoint) -> Self {
        Self {
            protocol_version: checkpoint.protocol_version(),
            record_kind: checkpoint.record_kind(),
            checkpoint_schema_version: checkpoint.schema_version(),
            request_id: checkpoint.request_id(),
            resolved_sha256: checkpoint.resolved_sha256(),
            checkpoint_id: checkpoint.checkpoint_id(),
            position: checkpoint.position(),
            simulation_time_bits: checkpoint.simulation_time_bits(),
            observations: checkpoint.observations(),
            numeric_observations: checkpoint.numeric_observations(),
            ordered_occurrences: checkpoint.ordered_occurrences(),
            unordered_sets: checkpoint.unordered_sets(),
            debug_primitives: &[],
            profile_names: checkpoint.profile_names(),
        }
    }
}
