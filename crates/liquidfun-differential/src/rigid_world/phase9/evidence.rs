//! Typed resolution and evaluation for the closed Phase 9 evidence bindings.

mod assertions;
mod proofs;

pub use proofs::{
    Phase9CaseEvidenceError, Phase9CrossRunProof, Phase9CrossRunProofRecord,
    Phase9EvidenceMismatch, Phase9EvidencePayloadRef, validate_phase9_cross_run_proofs,
};

use std::collections::{BTreeMap, BTreeSet};

use liquidfun_test_protocol::{
    HarnessLimits, Phase9OccurrenceKind, Phase9ParticleBufferMode, Phase9ParticleObservation,
    Phase9SemanticAssertion, Phase9WitnessBinding, RigidBodyKind, RigidWorldAction,
    RigidWorldObservation, RigidWorldRequestRecord, RigidWorldResultRecord, RigidWorldTimeline,
    RigidWorldTimelineResult, RigidWorldWitnessFamily, ScenarioId, Sha256Hex,
    decode_rigid_world_result_jsonl,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use assertions::{binding_error, evaluate_assertion, expected_action_id, unbound_error};

use super::{PHASE9_REQUIRED_POLICY_PATHS, Phase9ComparisonOutcome};
use crate::compare_complete_phase9_rigid_world_results;

/// A persisted Phase 9 witness did not resolve to or prove its declared semantic observation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Phase 9 evidence binding `{branch_id}` is invalid: {message}")]
pub struct Phase9EvidenceBindingError {
    branch_id: Box<str>,
    message: Box<str>,
}

/// Resolves and evaluates every closed Phase 9 witness against one decoded result.
///
/// Each binding must name the reviewed action for its branch, place that action inside the
/// selected checkpoint interval, resolve to the corresponding particle observation ordinal,
/// and satisfy its semantic assertion against the decoded request and result.
///
/// # Errors
///
/// Returns [`Phase9EvidenceBindingError`] when any indexed binding, observation variant, action,
/// or semantic value differs from the reviewed Phase 9 corpus contract.
pub fn validate_phase9_evidence_bindings(
    request: &RigidWorldRequestRecord,
    result: &RigidWorldResultRecord,
    bindings: &[Phase9WitnessBinding],
) -> Result<(), Phase9EvidenceBindingError> {
    if bindings
        .iter()
        .any(|binding| binding.branch_id.as_str() == "retained_phase6_through_phase8")
        && request.scenario().timelines().len() != RigidWorldWitnessFamily::ALL.len()
    {
        return Err(unbound_error(
            "retained Phase 6 through Phase 8 timelines are incomplete",
        ));
    }
    let timeline = request
        .scenario()
        .timelines()
        .first()
        .ok_or_else(|| unbound_error("missing Phase 9 request timeline"))?;
    let result_timeline = result
        .timelines()
        .first()
        .ok_or_else(|| unbound_error("missing Phase 9 result timeline"))?;
    for binding in bindings {
        if binding.semantic_assertion.requires_case_evidence() {
            validate_action_checkpoint(timeline, binding)?;
            continue;
        }
        let observation = resolve_observation(timeline, result_timeline, binding)?;
        if observation.witness_kind() != binding.observation_kind {
            return Err(binding_error(
                binding,
                format!(
                    "expected {:?} observation, resolved {:?}",
                    binding.observation_kind,
                    observation.witness_kind()
                ),
            ));
        }
        evaluate_assertion(timeline, result_timeline, binding, observation)?;
    }
    Ok(())
}

fn resolve_observation<'a>(
    timeline: &'a RigidWorldTimeline,
    result: &'a RigidWorldTimelineResult,
    binding: &Phase9WitnessBinding,
) -> Result<&'a Phase9ParticleObservation, Phase9EvidenceBindingError> {
    let action_start = validate_action_checkpoint(timeline, binding)?;
    let particle_ordinal = timeline.actions()[action_start..binding.action_index]
        .iter()
        .filter(|candidate| matches!(candidate.action(), RigidWorldAction::Particle { .. }))
        .count();
    let result_checkpoint = result
        .checkpoints
        .get(binding.checkpoint_index)
        .ok_or_else(|| binding_error(binding, "result checkpoint is absent"))?;
    result_checkpoint
        .observations
        .iter()
        .filter_map(|candidate| match candidate {
            RigidWorldObservation::Particle { observation } => Some(observation),
            _ => None,
        })
        .nth(particle_ordinal)
        .ok_or_else(|| binding_error(binding, "bound particle observation is absent"))
}

fn validate_action_checkpoint(
    timeline: &RigidWorldTimeline,
    binding: &Phase9WitnessBinding,
) -> Result<usize, Phase9EvidenceBindingError> {
    let action = timeline
        .actions()
        .get(binding.action_index)
        .ok_or_else(|| binding_error(binding, "action index is out of range"))?;
    let expected_action_id = expected_action_id(binding.branch_id.as_str());
    if action.action_id().as_str() != expected_action_id {
        return Err(binding_error(
            binding,
            format!(
                "expected action `{expected_action_id}`, found `{}`",
                action.action_id()
            ),
        ));
    }
    if !matches!(action.action(), RigidWorldAction::Particle { .. }) {
        return Err(binding_error(
            binding,
            "bound action is not a Phase 9 particle action",
        ));
    }
    let checkpoint = timeline
        .checkpoints()
        .get(binding.checkpoint_index)
        .ok_or_else(|| binding_error(binding, "checkpoint index is out of range"))?;
    let action_end = timeline
        .actions()
        .iter()
        .position(|candidate| candidate.action_id() == checkpoint.after_action_id())
        .ok_or_else(|| binding_error(binding, "checkpoint terminator action is absent"))?;
    let action_start = if binding.checkpoint_index == 0 {
        0
    } else {
        let previous = &timeline.checkpoints()[binding.checkpoint_index - 1];
        timeline
            .actions()
            .iter()
            .position(|candidate| candidate.action_id() == previous.after_action_id())
            .ok_or_else(|| binding_error(binding, "previous checkpoint terminator is absent"))?
            + 1
    };
    if !(action_start..=action_end).contains(&binding.action_index) {
        return Err(binding_error(
            binding,
            "bound action does not belong to the selected checkpoint",
        ));
    }
    Ok(action_start)
}
