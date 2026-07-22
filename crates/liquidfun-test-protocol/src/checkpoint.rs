//! Strict resolved-run checkpoint wire contract.

mod observation;
mod primitive;

pub use observation::*;
pub use primitive::*;

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{
    CheckpointId, CodecError, CodecErrorKind, FloatBits, HarnessLimits, ProtocolVersion,
    RecordLimit, RequestId, ScenarioActionId, ScenarioId, Sha256Hex, codec::BoundedVec,
    decode_jsonl, encode_jsonl,
};

const CHECKPOINT_MAXIMUM_OBSERVATIONS: usize = 128;
const CHECKPOINT_MAXIMUM_OCCURRENCES: usize = 4_096;
const CHECKPOINT_MAXIMUM_SETS: usize = 128;
const CHECKPOINT_MAXIMUM_PRIMITIVES: usize = 8_192;
const CHECKPOINT_MAXIMUM_TOTAL_VERTICES: usize = 65_536;
const CHECKPOINT_MAXIMUM_PROFILES: usize = 6;

/// Validated canonical-checkpoint schema version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CheckpointSchemaVersion(u32);

impl CheckpointSchemaVersion {
    /// The only checkpoint schema version supported by this contract.
    pub const CURRENT: Self = Self(1);

    /// Validates the closed checkpoint schema version.
    ///
    /// # Errors
    ///
    /// Returns [`CheckpointSchemaVersionError`] unless `value` is exactly one.
    pub const fn new(value: u32) -> Result<Self, CheckpointSchemaVersionError> {
        if value == 1 {
            return Ok(Self(value));
        }
        Err(CheckpointSchemaVersionError { value })
    }

    /// Returns the validated integer version.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl Serialize for CheckpointSchemaVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.0)
    }
}

impl<'de> Deserialize<'de> for CheckpointSchemaVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::new(u32::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// Error returned for unsupported checkpoint schema versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("unsupported checkpoint schema version {value}; supported version is 1")]
pub struct CheckpointSchemaVersionError {
    value: u32,
}

/// Stable semantic validation categories for run and checkpoint records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointErrorKind {
    /// Asserted bytes do not match their SHA-256 identity.
    HashMismatch,
    /// Redundant run identity disagrees with the exact resolved bytes.
    IdentityMismatch,
    /// A checkpoint identity disagrees with its declared semantic boundary.
    CheckpointMismatch,
    /// A semantic identity appears more than once in a unique namespace.
    DuplicateSemanticId,
    /// A collection violates its declared ordering policy.
    OrderingViolation,
    /// A numeric field contains NaN or infinity.
    InvalidFloat,
    /// A reviewed count or aggregate bound was exceeded.
    BoundaryLimitExceeded,
    /// A primitive has invalid geometry, style, kind, or inert label data.
    InvalidPrimitive,
    /// A profile name collection is incomplete, duplicated, or unordered.
    InvalidProfile,
}

/// Redacted semantic validation error.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("checkpoint validation failure: {kind:?}")]
pub struct CheckpointValidationError {
    kind: CheckpointErrorKind,
}

impl CheckpointValidationError {
    /// Returns the stable, non-sensitive failure category.
    #[must_use]
    pub const fn kind(&self) -> CheckpointErrorKind {
        self.kind
    }
}

pub(super) const fn validation(kind: CheckpointErrorKind) -> CheckpointValidationError {
    CheckpointValidationError { kind }
}

/// Error returned while strictly decoding a run request or checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CheckpointDecodeError {
    /// Strict JSONL framing or shape validation failed.
    #[error(transparent)]
    Codec(#[from] CodecError),
    /// Semantic validation failed after bounded decoding.
    #[error(transparent)]
    Validation(#[from] CheckpointValidationError),
}

impl CheckpointDecodeError {
    /// Returns the codec category when decoding failed before semantic acceptance.
    #[must_use]
    pub const fn codec_kind(&self) -> Option<CodecErrorKind> {
        match self {
            Self::Codec(error) => Some(error.kind()),
            Self::Validation(_) => None,
        }
    }

    /// Returns the semantic category when bounded decoding succeeded.
    #[must_use]
    pub const fn validation_kind(&self) -> Option<CheckpointErrorKind> {
        match self {
            Self::Codec(_) => None,
            Self::Validation(error) => Some(error.kind()),
        }
    }
}

/// Semantic boundary captured by a canonical checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CheckpointPosition {
    /// State immediately after one stable action occurrence.
    Action {
        /// Stable action identity.
        after_action_id: ScenarioActionId,
        /// Zero-based action ordinal in the resolved schedule.
        ordinal: u32,
    },
    /// State after one one-based logical simulation step.
    LogicalStep {
        /// One-based logical-step ordinal.
        ordinal: u32,
    },
}

/// Closed instrumentation profile names; durations are deliberately absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointProfileName {
    /// Contact lifecycle instrumentation was enabled.
    ContactLifecycle,
    /// Particle solve instrumentation was enabled.
    ParticleSolve,
    /// Rigid solve instrumentation was enabled.
    RigidSolve,
    /// Continuous solve instrumentation was enabled.
    ContinuousSolve,
    /// Typed command application instrumentation was enabled.
    ApplyCommands,
    /// Checkpoint finalization instrumentation was enabled.
    Finalize,
}

/// One versioned canonical semantic checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CanonicalCheckpoint {
    protocol_version: ProtocolVersion,
    record_kind: &'static str,
    checkpoint_schema_version: CheckpointSchemaVersion,
    request_id: RequestId,
    resolved_sha256: Sha256Hex,
    checkpoint_id: CheckpointId,
    position: CheckpointPosition,
    simulation_time_bits: FloatBits,
    observations: Box<[StructuralObservation]>,
    numeric_observations: Box<[NumericObservation]>,
    ordered_occurrences: Box<[OrderedOccurrence]>,
    unordered_sets: Box<[CheckpointSet]>,
    debug_primitives: Box<[DebugPrimitiveRecord]>,
    profile_names: Box<[CheckpointProfileName]>,
}

impl CanonicalCheckpoint {
    /// Constructs and validates one canonical checkpoint.
    ///
    /// # Errors
    ///
    /// Returns [`CheckpointValidationError`] for invalid identities, numbers, order, or bounds.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        request_id: RequestId,
        resolved_sha256: Sha256Hex,
        checkpoint_id: CheckpointId,
        position: CheckpointPosition,
        simulation_time_bits: FloatBits,
        observations: Vec<StructuralObservation>,
        numeric_observations: Vec<NumericObservation>,
        ordered_occurrences: Vec<OrderedOccurrence>,
        unordered_sets: Vec<CheckpointSet>,
        debug_primitives: Vec<DebugPrimitiveRecord>,
        mut profile_names: Vec<CheckpointProfileName>,
    ) -> Result<Self, CheckpointValidationError> {
        profile_names.sort_unstable();
        let checkpoint = Self {
            protocol_version: ProtocolVersion::CURRENT,
            record_kind: "canonical_checkpoint",
            checkpoint_schema_version: CheckpointSchemaVersion::CURRENT,
            request_id,
            resolved_sha256,
            checkpoint_id,
            position,
            simulation_time_bits,
            observations: observations.into_boxed_slice(),
            numeric_observations: numeric_observations.into_boxed_slice(),
            ordered_occurrences: ordered_occurrences.into_boxed_slice(),
            unordered_sets: unordered_sets.into_boxed_slice(),
            debug_primitives: debug_primitives.into_boxed_slice(),
            profile_names: profile_names.into_boxed_slice(),
        };
        checkpoint.validate()?;
        Ok(checkpoint)
    }

    /// Returns the closed checkpoint schema version.
    #[must_use]
    pub const fn schema_version(&self) -> CheckpointSchemaVersion {
        self.checkpoint_schema_version
    }

    /// Returns explicitly unordered sets in stable set-ID order.
    #[must_use]
    pub fn unordered_sets(&self) -> &[CheckpointSet] {
        &self.unordered_sets
    }

    /// Returns enabled profile names without timing values.
    #[must_use]
    pub fn profile_names(&self) -> &[CheckpointProfileName] {
        &self.profile_names
    }

    fn validate(&self) -> Result<(), CheckpointValidationError> {
        require_finite(self.simulation_time_bits)?;
        validate_position(&self.checkpoint_id, &self.position)?;
        validate_bounds(self)?;
        validate_strict_ids(
            self.observations
                .iter()
                .map(StructuralObservation::observation_id),
        )?;
        validate_strict_ids(
            self.numeric_observations
                .iter()
                .map(NumericObservation::observation_id),
        )?;
        for observation in &self.numeric_observations {
            require_finite(observation.value_bits())?;
        }
        validate_unique_ids(
            self.ordered_occurrences
                .iter()
                .map(OrderedOccurrence::occurrence_id),
        )?;
        validate_strict_ids(self.unordered_sets.iter().map(CheckpointSet::set_id))?;

        let mut primitive_keys = HashSet::with_capacity(self.debug_primitives.len());
        let mut maybe_previous_canonical = None;
        let mut total_vertices = 0_usize;
        for record in &self.debug_primitives {
            if !primitive_keys.insert(record.key()) {
                return Err(validation(CheckpointErrorKind::DuplicateSemanticId));
            }
            if record.ordering() == DebugPrimitiveOrder::Canonicalized {
                if maybe_previous_canonical.is_some_and(|previous| previous >= record.key()) {
                    return Err(validation(CheckpointErrorKind::OrderingViolation));
                }
                maybe_previous_canonical = Some(record.key());
            } else {
                maybe_previous_canonical = None;
            }
            total_vertices = total_vertices
                .checked_add(record.primitive().validate()?)
                .ok_or_else(|| validation(CheckpointErrorKind::BoundaryLimitExceeded))?;
            if total_vertices > CHECKPOINT_MAXIMUM_TOTAL_VERTICES {
                return Err(validation(CheckpointErrorKind::BoundaryLimitExceeded));
            }
        }
        if self.profile_names.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(validation(CheckpointErrorKind::InvalidProfile));
        }
        Ok(())
    }
}

fn validate_position(
    checkpoint_id: &CheckpointId,
    position: &CheckpointPosition,
) -> Result<(), CheckpointValidationError> {
    let expected = match position {
        CheckpointPosition::LogicalStep { ordinal } if *ordinal > 0 => {
            format!("checkpoint-{ordinal:04}")
        }
        CheckpointPosition::Action {
            after_action_id,
            ordinal,
        } if after_action_id.as_str() == format!("action-{ordinal:04}") => {
            format!("checkpoint-action-{ordinal:04}")
        }
        _ => return Err(validation(CheckpointErrorKind::CheckpointMismatch)),
    };
    if checkpoint_id.as_str() != expected {
        return Err(validation(CheckpointErrorKind::CheckpointMismatch));
    }
    Ok(())
}

fn validate_bounds(checkpoint: &CanonicalCheckpoint) -> Result<(), CheckpointValidationError> {
    let valid = checkpoint.observations.len() <= CHECKPOINT_MAXIMUM_OBSERVATIONS
        && checkpoint.numeric_observations.len() <= CHECKPOINT_MAXIMUM_OBSERVATIONS
        && checkpoint.ordered_occurrences.len() <= CHECKPOINT_MAXIMUM_OCCURRENCES
        && checkpoint.unordered_sets.len() <= CHECKPOINT_MAXIMUM_SETS
        && checkpoint.debug_primitives.len() <= CHECKPOINT_MAXIMUM_PRIMITIVES
        && checkpoint.profile_names.len() <= CHECKPOINT_MAXIMUM_PROFILES;
    if !valid {
        return Err(validation(CheckpointErrorKind::BoundaryLimitExceeded));
    }
    Ok(())
}

fn validate_strict_ids<'a>(
    ids: impl Iterator<Item = &'a ScenarioId>,
) -> Result<(), CheckpointValidationError> {
    let mut maybe_previous: Option<&ScenarioId> = None;
    for id in ids {
        if maybe_previous.is_some_and(|previous| previous >= id) {
            return Err(validation(CheckpointErrorKind::OrderingViolation));
        }
        maybe_previous = Some(id);
    }
    Ok(())
}

fn validate_unique_ids<'a>(
    ids: impl Iterator<Item = &'a ScenarioId>,
) -> Result<(), CheckpointValidationError> {
    let mut seen = HashSet::new();
    for id in ids {
        if !seen.insert(id) {
            return Err(validation(CheckpointErrorKind::DuplicateSemanticId));
        }
    }
    Ok(())
}

pub(super) fn require_finite(value: FloatBits) -> Result<(), CheckpointValidationError> {
    if !value.to_f32().is_finite() {
        return Err(validation(CheckpointErrorKind::InvalidFloat));
    }
    Ok(())
}

pub(super) fn require_nonnegative_finite(
    value: FloatBits,
) -> Result<(), CheckpointValidationError> {
    require_finite(value)?;
    if value.to_f32().is_sign_negative() {
        return Err(validation(CheckpointErrorKind::InvalidPrimitive));
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCanonicalCheckpoint {
    protocol_version: ProtocolVersion,
    #[serde(rename = "record_kind")]
    _record_kind: CanonicalCheckpointRecordKind,
    checkpoint_schema_version: CheckpointSchemaVersion,
    request_id: RequestId,
    resolved_sha256: Sha256Hex,
    checkpoint_id: CheckpointId,
    position: CheckpointPosition,
    simulation_time_bits: FloatBits,
    observations: BoundedVec<StructuralObservation, CHECKPOINT_MAXIMUM_OBSERVATIONS>,
    numeric_observations: BoundedVec<NumericObservation, CHECKPOINT_MAXIMUM_OBSERVATIONS>,
    ordered_occurrences: BoundedVec<OrderedOccurrence, CHECKPOINT_MAXIMUM_OCCURRENCES>,
    unordered_sets: BoundedVec<CheckpointSet, CHECKPOINT_MAXIMUM_SETS>,
    debug_primitives: BoundedVec<DebugPrimitiveRecord, CHECKPOINT_MAXIMUM_PRIMITIVES>,
    profile_names: BoundedVec<CheckpointProfileName, CHECKPOINT_MAXIMUM_PROFILES>,
}

#[derive(Deserialize)]
enum CanonicalCheckpointRecordKind {
    #[serde(rename = "canonical_checkpoint")]
    CanonicalCheckpoint,
}

/// Encodes one canonical checkpoint as strict newline-complete JSON.
///
/// # Errors
///
/// Returns [`CodecError`] when serialization or the output byte bound fails.
pub fn encode_canonical_checkpoint_jsonl(
    checkpoint: &CanonicalCheckpoint,
    limits: &HarnessLimits,
) -> Result<Vec<u8>, CodecError> {
    encode_jsonl(checkpoint, limits, RecordLimit::Output)
}

/// Strictly decodes and semantically validates one canonical checkpoint.
///
/// # Errors
///
/// Returns [`CheckpointDecodeError`] before accepting malformed or contradictory input.
pub fn decode_canonical_checkpoint_jsonl(
    bytes: &[u8],
    limits: &HarnessLimits,
) -> Result<CanonicalCheckpoint, CheckpointDecodeError> {
    let raw: RawCanonicalCheckpoint = decode_jsonl(bytes, limits, RecordLimit::Output)?;
    let checkpoint = CanonicalCheckpoint {
        protocol_version: raw.protocol_version,
        record_kind: "canonical_checkpoint",
        checkpoint_schema_version: raw.checkpoint_schema_version,
        request_id: raw.request_id,
        resolved_sha256: raw.resolved_sha256,
        checkpoint_id: raw.checkpoint_id,
        position: raw.position,
        simulation_time_bits: raw.simulation_time_bits,
        observations: raw.observations.into_vec().into_boxed_slice(),
        numeric_observations: raw.numeric_observations.into_vec().into_boxed_slice(),
        ordered_occurrences: raw.ordered_occurrences.into_vec().into_boxed_slice(),
        unordered_sets: raw.unordered_sets.into_vec().into_boxed_slice(),
        debug_primitives: raw.debug_primitives.into_vec().into_boxed_slice(),
        profile_names: raw.profile_names.into_vec().into_boxed_slice(),
    };
    checkpoint.validate()?;
    Ok(checkpoint)
}
