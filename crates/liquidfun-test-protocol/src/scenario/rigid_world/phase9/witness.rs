use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::{
    PHASE9_MAXIMUM_WITNESS_BINDINGS, PHASE9_MAXIMUM_WITNESS_CHECKPOINTS,
    PHASE9_REQUIRED_BRANCH_IDS, Phase9ObservationKind, Phase9SemanticAssertion,
};
use crate::{RIGID_WORLD_MAXIMUM_ACTIONS, ScenarioId};

/// Exact action/checkpoint/observation/assertion binding for one Phase 9 branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase9WitnessBinding {
    pub branch_id: ScenarioId,
    pub action_index: usize,
    pub checkpoint_index: usize,
    pub observation_kind: Phase9ObservationKind,
    pub semantic_assertion: Phase9SemanticAssertion,
}

/// Stable validation category for an invalid Phase 9 witness binding registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase9WitnessBindingErrorKind {
    TooManyBindings,
    DuplicateBranch,
    MissingBranch,
    ExtraBranch,
    BranchAssertionMismatch,
    ActionIndexOutOfRange,
    CheckpointIndexOutOfRange,
    ObservationKindMismatch,
    InvalidSemanticAssertion,
}

/// Error returned when a Phase 9 witness registry is not closed and semantic.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid Phase 9 witness binding: {kind:?}")]
pub struct Phase9WitnessBindingError {
    kind: Phase9WitnessBindingErrorKind,
}

impl Phase9WitnessBindingError {
    /// Returns the stable witness-validation failure category.
    #[must_use]
    pub const fn kind(&self) -> Phase9WitnessBindingErrorKind {
        self.kind
    }
}

/// Validates a complete Phase 9 witness registry before any indexed evaluation.
///
/// # Errors
///
/// Returns [`Phase9WitnessBindingError`] when the registry is oversized,
/// incomplete, duplicated, out of range, or not bound to a typed semantic
/// assertion and its exact observation kind.
pub fn validate_phase9_witness_bindings(
    bindings: &[Phase9WitnessBinding],
    action_count: usize,
    checkpoint_count: usize,
) -> Result<(), Phase9WitnessBindingError> {
    if bindings.len() > PHASE9_MAXIMUM_WITNESS_BINDINGS {
        return Err(witness_error(
            Phase9WitnessBindingErrorKind::TooManyBindings,
        ));
    }
    let mut branches = HashSet::with_capacity(bindings.len());
    for binding in bindings {
        let branch_id = binding.branch_id.as_str();
        if !PHASE9_REQUIRED_BRANCH_IDS
            .lines()
            .any(|required| required == branch_id)
        {
            return Err(witness_error(Phase9WitnessBindingErrorKind::ExtraBranch));
        }
        if !branches.insert(branch_id) {
            return Err(witness_error(
                Phase9WitnessBindingErrorKind::DuplicateBranch,
            ));
        }
        if binding.semantic_assertion.branch_id() != branch_id {
            return Err(witness_error(
                Phase9WitnessBindingErrorKind::BranchAssertionMismatch,
            ));
        }
        if requires_specific_assertion(branch_id)
            && matches!(
                binding.semantic_assertion,
                Phase9SemanticAssertion::ObservedSemantic { .. }
            )
        {
            return Err(witness_error(
                Phase9WitnessBindingErrorKind::BranchAssertionMismatch,
            ));
        }
        if !binding.semantic_assertion.is_valid() {
            return Err(witness_error(
                Phase9WitnessBindingErrorKind::InvalidSemanticAssertion,
            ));
        }
        if binding.action_index >= action_count
            || binding.action_index >= RIGID_WORLD_MAXIMUM_ACTIONS
        {
            return Err(witness_error(
                Phase9WitnessBindingErrorKind::ActionIndexOutOfRange,
            ));
        }
        if binding.checkpoint_index >= checkpoint_count
            || binding.checkpoint_index >= PHASE9_MAXIMUM_WITNESS_CHECKPOINTS
        {
            return Err(witness_error(
                Phase9WitnessBindingErrorKind::CheckpointIndexOutOfRange,
            ));
        }
        if binding.observation_kind != binding.semantic_assertion.expected_observation_kind() {
            return Err(witness_error(
                Phase9WitnessBindingErrorKind::ObservationKindMismatch,
            ));
        }
    }
    if PHASE9_REQUIRED_BRANCH_IDS
        .lines()
        .any(|branch_id| !branches.contains(branch_id))
    {
        return Err(witness_error(Phase9WitnessBindingErrorKind::MissingBranch));
    }
    Ok(())
}

pub(super) fn requires_specific_assertion(branch_id: &str) -> bool {
    matches!(
        branch_id,
        "finite_lifetime"
            | "infinite_lifetime"
            | "equal_lifetime"
            | "strict_contact_enabled"
            | "strict_contact_disabled"
            | "listener_flag_enabled"
            | "listener_flag_disabled"
            | "filter_flag_enabled"
            | "filter_flag_disabled"
            | "collision_energy"
            | "stuck_candidates"
            | "replay_identity"
            | "minimization_identity"
            | "first_divergence_stability"
            | "d0_byte_identity"
            | "debug_release_agreement"
    )
}

pub(super) fn observed_branch_kind(branch_id: &str) -> Option<Phase9ObservationKind> {
    match branch_id {
        "multiple_systems" | "newest_first" | "stable_ids_sort" => {
            Some(Phase9ObservationKind::System)
        }
        "paused_system" | "fixed_buffer" | "growable_buffer" | "fixed_full"
        | "maximum_lifetime" => Some(Phase9ObservationKind::Statistics),
        "stable_ids_compact" => Some(Phase9ObservationKind::MixedState),
        "optional_lanes" | "force_range" | "impulse_range" => Some(Phase9ObservationKind::Particle),
        "teardown" | "oldest_lifetime" | "requested_destruction_callback" | "capacity_eviction" => {
            Some(Phase9ObservationKind::Lifecycle)
        }
        "unrequested_destruction_callback" | "zombie_pending" => {
            Some(Phase9ObservationKind::MixedState)
        }
        "particle_contact" | "contact_order" | "contact_multiplicity" => {
            Some(Phase9ObservationKind::ParticleContact)
        }
        "body_contact" | "coupling_fields" => Some(Phase9ObservationKind::BodyContact),
        "dynamic_body_reaction" | "static_body_no_reaction" | "statistics_counts" => {
            Some(Phase9ObservationKind::Statistics)
        }
        "system_aabb" | "world_aabb" | "system_culling" | "query_continue" | "query_terminate" => {
            Some(Phase9ObservationKind::Query)
        }
        "system_ray"
        | "world_ray"
        | "ray_culling"
        | "ray_start_inside_exclusion"
        | "ray_ignore"
        | "ray_continue"
        | "ray_clip"
        | "ray_terminate" => Some(Phase9ObservationKind::RayCast),
        "retained_phase6_through_phase8" | "phase10_rejection" | "closed_policy_registry" => {
            Some(Phase9ObservationKind::Particle)
        }
        _ => None,
    }
}

const fn witness_error(kind: Phase9WitnessBindingErrorKind) -> Phase9WitnessBindingError {
    Phase9WitnessBindingError { kind }
}
