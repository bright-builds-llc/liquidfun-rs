use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    BuildEvidenceTier, BuildIdentity, BuildIdentityFields, CheckpointId, CodecError, FloatBits,
    HarnessFailureKind, HarnessLimits, MathProbeHorizon, MathProbeOperation, MathProbePolicyPath,
    Phase4BuildIdentityFields, ProtocolVersion, RecordLimit, RequestId, ScenarioId,
    ScenarioRequestRecord, ScenarioSchemaVersion, ScenarioSource, Sha256Hex,
    ToleranceProfileVersion, TraceSchemaVersion,
    codec::{BoundedString, BoundedVec, decode_jsonl},
};

const MAXIMUM_ID_BYTES: usize = 128;
const MAXIMUM_STRING_BYTES: usize = 4 * 1024;
const MAXIMUM_SUPPORTED_VERSIONS: usize = 16;

/// Error produced while validating handshake or trace semantics.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("trace validation failed: {kind:?}: {message}")]
pub struct TraceValidationError {
    kind: HarnessFailureKind,
    message: Box<str>,
}

impl TraceValidationError {
    fn new(kind: HarnessFailureKind, message: impl Into<Box<str>>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// Returns the exact non-physics harness classification.
    #[must_use]
    pub const fn kind(&self) -> HarnessFailureKind {
        self.kind
    }
}

/// Error produced while hashing canonical trace payload bytes.
#[derive(Debug, thiserror::Error)]
#[error("trace payload serialization failed: {0}")]
pub struct TraceHashError(serde_json::Error);

/// Error produced while strictly decoding a handshake or trace record.
#[derive(Debug, thiserror::Error)]
pub enum TraceDecodeError {
    /// Strict JSONL framing or shape validation failed.
    #[error(transparent)]
    Codec(#[from] CodecError),
    /// Typed provenance or record validation failed.
    #[error(transparent)]
    Validation(#[from] TraceValidationError),
}

mod math_probe;
mod records;
mod session;
mod validation;
mod wire;

pub use math_probe::*;
pub use records::*;
pub use session::*;
pub use validation::*;
pub use wire::*;

#[cfg(test)]
mod tests;
