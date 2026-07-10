use std::ops::Range;

use crate::{HarnessLimits, RecordLimit, codec::decode_jsonl};

use super::{RawScenarioV1, ScenarioDecodeError, ValidatedScenarioV1, validate_scenario};

impl ValidatedScenarioV1 {
    /// Creates and revalidates a candidate with one checkpoint range removed.
    ///
    /// # Errors
    ///
    /// Returns [`ScenarioReductionError`] for an invalid range, serialization failure, or a
    /// candidate that violates scenario invariants.
    pub fn without_checkpoint_range(
        &self,
        range: Range<usize>,
        limits: &HarnessLimits,
    ) -> Result<Self, ScenarioReductionError> {
        validate_reduction_range(&range, self.checkpoints.len())?;
        let mut candidate = self.clone();
        candidate.checkpoints.drain(range);
        revalidate_candidate(&candidate, limits)
    }

    /// Creates and revalidates a candidate with one command range removed.
    ///
    /// # Errors
    ///
    /// Returns [`ScenarioReductionError`] when removal empties the command list, leaves a broken
    /// checkpoint reference, uses an invalid range, or cannot be serialized.
    pub fn without_command_range(
        &self,
        range: Range<usize>,
        limits: &HarnessLimits,
    ) -> Result<Self, ScenarioReductionError> {
        validate_reduction_range(&range, self.commands.len())?;
        let mut candidate = self.clone();
        candidate.commands.drain(range);
        revalidate_candidate(&candidate, limits)
    }
}

/// Error produced while building and revalidating one typed reduction candidate.
#[derive(Debug, thiserror::Error)]
pub enum ScenarioReductionError {
    /// The requested half-open range was empty or outside the collection.
    #[error("scenario reduction range is empty or out of bounds")]
    InvalidRange,
    /// A typed candidate could not be serialized for boundary revalidation.
    #[error("scenario reduction serialization failed: {0}")]
    Serialization(serde_json::Error),
    /// The candidate failed the ordinary strict scenario validator.
    #[error(transparent)]
    Validation(#[from] ScenarioDecodeError),
}

/// Strictly reparses one canonical scenario value without a transport envelope.
///
/// # Errors
///
/// Returns [`ScenarioDecodeError`] unless the canonical JSON value satisfies the same typed
/// bounds, references, ordering, and empty-world invariants as a request scenario.
pub fn decode_scenario_json(
    bytes: &[u8],
    limits: &HarnessLimits,
) -> Result<ValidatedScenarioV1, ScenarioDecodeError> {
    let mut jsonl = Vec::with_capacity(bytes.len() + 1);
    jsonl.extend_from_slice(bytes);
    jsonl.push(b'\n');
    let raw = decode_jsonl::<RawScenarioV1>(&jsonl, limits, RecordLimit::Input)?;
    validate_scenario(raw)
}

fn validate_reduction_range(
    range: &Range<usize>,
    length: usize,
) -> Result<(), ScenarioReductionError> {
    if range.start >= range.end || range.end > length {
        return Err(ScenarioReductionError::InvalidRange);
    }
    Ok(())
}

fn revalidate_candidate(
    candidate: &ValidatedScenarioV1,
    limits: &HarnessLimits,
) -> Result<ValidatedScenarioV1, ScenarioReductionError> {
    let bytes = serde_json::to_vec(candidate).map_err(ScenarioReductionError::Serialization)?;
    decode_scenario_json(&bytes, limits).map_err(ScenarioReductionError::Validation)
}
