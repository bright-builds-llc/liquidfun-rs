use std::time::Duration;

use liquidfun_test_protocol::{
    HarnessLimits, RigidWorldRequestRecord, ScenarioReductionError, ScenarioSource,
    ValidatedScenarioV1, decode_rigid_world_request_jsonl,
};
use serde::Serialize;

use crate::FailureSignature;
use crate::RigidFailureSignature;

/// Deterministic bounded reduction configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinimizationBudget {
    max_attempts: usize,
    deadline: Duration,
}

impl MinimizationBudget {
    /// Creates named attempt and logical elapsed-time bounds.
    #[must_use]
    pub const fn new(max_attempts: usize, deadline: Duration) -> Self {
        Self {
            max_attempts,
            deadline,
        }
    }
}

/// Exhausted bound that stopped reduction before convergence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetExhausted {
    /// Candidate-attempt bound was reached.
    Attempts,
    /// Accumulated injected evaluation time reached the deadline.
    Deadline,
}

/// Completion state of a bounded minimization run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MinimizationStatus {
    /// No remaining stable transform retained the target signature.
    Complete,
    /// A bound stopped reduction with the best valid candidate found so far.
    Incomplete(BudgetExhausted),
}

/// One deterministic hierarchical candidate transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioTransform {
    /// Removes a half-open range of ordered checkpoint requests.
    RemoveCheckpoints {
        /// First removed checkpoint index.
        start: usize,
        /// Exclusive removed checkpoint index.
        end: usize,
    },
    /// Removes a half-open range of ordered step commands.
    RemoveCommands {
        /// First removed command index.
        start: usize,
        /// Exclusive removed command index.
        end: usize,
    },
}

/// Typed evaluator result plus deterministic logical elapsed time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    maybe_signature: Option<FailureSignature>,
    elapsed: Duration,
}

impl Evaluation {
    /// Creates one injected semantic evaluation result.
    #[must_use]
    pub const fn new(maybe_signature: Option<FailureSignature>, elapsed: Duration) -> Self {
        Self {
            maybe_signature,
            elapsed,
        }
    }

    /// Replaces logical elapsed time for deterministic evaluator fixtures.
    pub const fn set_elapsed(&mut self, elapsed: Duration) {
        self.elapsed = elapsed;
    }
}

/// Best valid same-signature scenario found within explicit budgets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinimizationResult {
    scenario: ValidatedScenarioV1,
    canonical_scenario_bytes: Box<[u8]>,
    original_source: ScenarioSource,
    status: MinimizationStatus,
    attempts: usize,
    evaluations: usize,
    rejected_invalid_candidates: usize,
    rejected_changed_signatures: usize,
    attempted_transforms: Box<[ScenarioTransform]>,
    accepted_transforms: Box<[ScenarioTransform]>,
}

impl MinimizationResult {
    fn new(state: ReductionState, status: MinimizationStatus) -> Result<Self, MinimizationError> {
        let canonical_scenario_bytes = serde_json::to_vec(&state.scenario)
            .map_err(MinimizationError::Serialize)?
            .into_boxed_slice();
        Ok(Self {
            scenario: state.scenario,
            canonical_scenario_bytes,
            original_source: state.original_source,
            status,
            attempts: state.attempts,
            evaluations: state.evaluations,
            rejected_invalid_candidates: state.rejected_invalid_candidates,
            rejected_changed_signatures: state.rejected_changed_signatures,
            attempted_transforms: state.attempted_transforms.into_boxed_slice(),
            accepted_transforms: state.accepted_transforms.into_boxed_slice(),
        })
    }

    /// Returns the best validated same-signature scenario.
    #[must_use]
    pub const fn scenario(&self) -> &ValidatedScenarioV1 {
        &self.scenario
    }

    /// Returns deterministic canonical bytes for the minimized scenario value.
    #[must_use]
    pub fn canonical_scenario_bytes(&self) -> &[u8] {
        &self.canonical_scenario_bytes
    }

    /// Returns original named or seeded reproducibility metadata.
    #[must_use]
    pub const fn original_source(&self) -> &ScenarioSource {
        &self.original_source
    }

    /// Returns complete or explicitly incomplete status.
    #[must_use]
    pub const fn status(&self) -> MinimizationStatus {
        self.status
    }

    /// Returns every bounded candidate attempt, including invalid candidates.
    #[must_use]
    pub const fn attempts(&self) -> usize {
        self.attempts
    }

    /// Returns valid candidates passed to the injected evaluator.
    #[must_use]
    pub const fn evaluations(&self) -> usize {
        self.evaluations
    }

    /// Returns candidates rejected by typed scenario revalidation.
    #[must_use]
    pub const fn rejected_invalid_candidates(&self) -> usize {
        self.rejected_invalid_candidates
    }

    /// Returns valid mismatches rejected for changing the failure signature.
    #[must_use]
    pub const fn rejected_changed_signatures(&self) -> usize {
        self.rejected_changed_signatures
    }

    /// Returns the exact stable transform-attempt order.
    #[must_use]
    pub fn attempted_transforms(&self) -> &[ScenarioTransform] {
        &self.attempted_transforms
    }

    /// Returns transforms retained because they reproduced the exact signature.
    #[must_use]
    pub fn accepted_transforms(&self) -> &[ScenarioTransform] {
        &self.accepted_transforms
    }
}

/// Error produced while serializing a fully validated minimized scenario.
#[derive(Debug, thiserror::Error)]
pub enum MinimizationError {
    /// Canonical typed scenario serialization failed.
    #[error("minimized scenario serialization failed: {0}")]
    Serialize(serde_json::Error),
}

struct ReductionState {
    scenario: ValidatedScenarioV1,
    original_source: ScenarioSource,
    attempts: usize,
    evaluations: usize,
    elapsed: Duration,
    rejected_invalid_candidates: usize,
    rejected_changed_signatures: usize,
    attempted_transforms: Vec<ScenarioTransform>,
    accepted_transforms: Vec<ScenarioTransform>,
}

impl ReductionState {
    fn new(scenario: &ValidatedScenarioV1) -> Self {
        Self {
            scenario: scenario.clone(),
            original_source: scenario.source().clone(),
            attempts: 0,
            evaluations: 0,
            elapsed: Duration::ZERO,
            rejected_invalid_candidates: 0,
            rejected_changed_signatures: 0,
            attempted_transforms: Vec::new(),
            accepted_transforms: Vec::new(),
        }
    }
}

/// Minimizes a validated scenario while retaining exactly one target failure signature.
///
/// The evaluator is injected and receives only candidates that passed typed protocol
/// revalidation. Its elapsed value is logical evidence, keeping this core independent of clocks,
/// processes, and file systems.
///
/// # Errors
///
/// Returns [`MinimizationError`] only if canonical serialization of the final validated value
/// fails.
pub fn minimize<F>(
    scenario: &ValidatedScenarioV1,
    target: &FailureSignature,
    budget: MinimizationBudget,
    mut evaluator: F,
) -> Result<MinimizationResult, MinimizationError>
where
    F: FnMut(&ValidatedScenarioV1) -> Evaluation,
{
    let limits = HarnessLimits::phase2_default_v1();
    let mut state = ReductionState::new(scenario);
    if budget.max_attempts == 0 {
        return MinimizationResult::new(
            state,
            MinimizationStatus::Incomplete(BudgetExhausted::Attempts),
        );
    }
    if budget.deadline.is_zero() {
        return MinimizationResult::new(
            state,
            MinimizationStatus::Incomplete(BudgetExhausted::Deadline),
        );
    }

    loop {
        let transforms = candidate_transforms(&state.scenario);
        let mut accepted = false;
        for transform in transforms {
            if state.attempts >= budget.max_attempts {
                return MinimizationResult::new(
                    state,
                    MinimizationStatus::Incomplete(BudgetExhausted::Attempts),
                );
            }

            state.attempts += 1;
            state.attempted_transforms.push(transform);
            let Ok(candidate) = apply_transform(&state.scenario, transform, &limits) else {
                state.rejected_invalid_candidates += 1;
                continue;
            };

            state.evaluations += 1;
            let evaluation = evaluator(&candidate);
            state.elapsed = state.elapsed.saturating_add(evaluation.elapsed);
            if evaluation.maybe_signature.as_ref() == Some(target) {
                state.scenario = candidate;
                state.accepted_transforms.push(transform);
                accepted = true;
            } else if evaluation.maybe_signature.is_some() {
                state.rejected_changed_signatures += 1;
            }

            if state.elapsed >= budget.deadline {
                return MinimizationResult::new(
                    state,
                    MinimizationStatus::Incomplete(BudgetExhausted::Deadline),
                );
            }
            if accepted {
                break;
            }
        }

        if !accepted {
            return MinimizationResult::new(state, MinimizationStatus::Complete);
        }
    }
}

fn candidate_transforms(scenario: &ValidatedScenarioV1) -> Vec<ScenarioTransform> {
    let mut transforms = removal_ranges(scenario.checkpoints().len())
        .into_iter()
        .map(|(start, end)| ScenarioTransform::RemoveCheckpoints { start, end })
        .collect::<Vec<_>>();
    transforms.extend(
        removal_ranges(scenario.commands().len())
            .into_iter()
            .map(|(start, end)| ScenarioTransform::RemoveCommands { start, end }),
    );
    transforms
}

fn removal_ranges(length: usize) -> Vec<(usize, usize)> {
    if length == 0 {
        return Vec::new();
    }

    let mut ranges = Vec::new();
    let mut chunk_size = length.div_ceil(2);
    loop {
        let mut start = 0;
        while start < length {
            ranges.push((start, (start + chunk_size).min(length)));
            start += chunk_size;
        }
        if chunk_size == 1 {
            break;
        }
        chunk_size = chunk_size.div_ceil(2);
    }
    ranges
}

fn apply_transform(
    scenario: &ValidatedScenarioV1,
    transform: ScenarioTransform,
    limits: &HarnessLimits,
) -> Result<ValidatedScenarioV1, ScenarioReductionError> {
    match transform {
        ScenarioTransform::RemoveCheckpoints { start, end } => {
            scenario.without_checkpoint_range(start..end, limits)
        }
        ScenarioTransform::RemoveCommands { start, end } => {
            scenario.without_command_range(start..end, limits)
        }
    }
}

/// One deterministic validity-preserving rigid timeline transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidScenarioTransform {
    /// Removes a half-open range of ordered timeline actions.
    RemoveActions {
        /// Declaration-ordered timeline index.
        timeline_index: usize,
        /// First removed action index.
        start: usize,
        /// Exclusive removed action index.
        end: usize,
    },
}

/// Typed rigid evaluator result plus deterministic logical elapsed time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RigidEvaluation {
    maybe_signature: Option<RigidFailureSignature>,
    elapsed: Duration,
}

impl RigidEvaluation {
    /// Creates one injected semantic rigid evaluation.
    #[must_use]
    pub const fn new(maybe_signature: Option<RigidFailureSignature>, elapsed: Duration) -> Self {
        Self {
            maybe_signature,
            elapsed,
        }
    }
}

/// Best valid same-signature rigid request found within explicit budgets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RigidMinimizationResult {
    request: RigidWorldRequestRecord,
    canonical_request_bytes: Box<[u8]>,
    status: MinimizationStatus,
    attempts: usize,
    evaluations: usize,
    rejected_invalid_candidates: usize,
    rejected_changed_signatures: usize,
    attempted_transforms: Box<[RigidScenarioTransform]>,
    accepted_transforms: Box<[RigidScenarioTransform]>,
}

impl RigidMinimizationResult {
    fn new(
        state: RigidReductionState,
        status: MinimizationStatus,
    ) -> Result<Self, RigidMinimizationError> {
        let mut canonical_request_bytes =
            serde_json::to_vec(&state.request).map_err(RigidMinimizationError::Serialize)?;
        canonical_request_bytes.push(b'\n');
        Ok(Self {
            request: state.request,
            canonical_request_bytes: canonical_request_bytes.into_boxed_slice(),
            status,
            attempts: state.attempts,
            evaluations: state.evaluations,
            rejected_invalid_candidates: state.rejected_invalid_candidates,
            rejected_changed_signatures: state.rejected_changed_signatures,
            attempted_transforms: state.attempted_transforms.into_boxed_slice(),
            accepted_transforms: state.accepted_transforms.into_boxed_slice(),
        })
    }

    /// Returns the best validated same-signature rigid request.
    #[must_use]
    pub const fn request(&self) -> &RigidWorldRequestRecord {
        &self.request
    }

    /// Returns deterministic newline-complete minimized request bytes.
    #[must_use]
    pub fn canonical_request_bytes(&self) -> &[u8] {
        &self.canonical_request_bytes
    }

    /// Returns complete or explicitly incomplete status.
    #[must_use]
    pub const fn status(&self) -> MinimizationStatus {
        self.status
    }

    /// Returns every bounded candidate attempt, including invalid candidates.
    #[must_use]
    pub const fn attempts(&self) -> usize {
        self.attempts
    }

    /// Returns valid candidates passed to the injected evaluator.
    #[must_use]
    pub const fn evaluations(&self) -> usize {
        self.evaluations
    }

    /// Returns candidates rejected by typed rigid request revalidation.
    #[must_use]
    pub const fn rejected_invalid_candidates(&self) -> usize {
        self.rejected_invalid_candidates
    }

    /// Returns valid mismatches rejected for changing exact signature identity.
    #[must_use]
    pub const fn rejected_changed_signatures(&self) -> usize {
        self.rejected_changed_signatures
    }

    /// Returns exact stable transform-attempt order.
    #[must_use]
    pub fn attempted_transforms(&self) -> &[RigidScenarioTransform] {
        &self.attempted_transforms
    }

    /// Returns transforms retained because they reproduced the exact signature.
    #[must_use]
    pub fn accepted_transforms(&self) -> &[RigidScenarioTransform] {
        &self.accepted_transforms
    }
}

/// Error produced while serializing a fully validated minimized rigid request.
#[derive(Debug, thiserror::Error)]
pub enum RigidMinimizationError {
    /// Canonical typed request serialization failed.
    #[error("minimized rigid request serialization failed: {0}")]
    Serialize(serde_json::Error),
}

struct RigidReductionState {
    request: RigidWorldRequestRecord,
    attempts: usize,
    evaluations: usize,
    elapsed: Duration,
    rejected_invalid_candidates: usize,
    rejected_changed_signatures: usize,
    attempted_transforms: Vec<RigidScenarioTransform>,
    accepted_transforms: Vec<RigidScenarioTransform>,
}

impl RigidReductionState {
    fn new(request: &RigidWorldRequestRecord) -> Self {
        Self {
            request: request.clone(),
            attempts: 0,
            evaluations: 0,
            elapsed: Duration::ZERO,
            rejected_invalid_candidates: 0,
            rejected_changed_signatures: 0,
            attempted_transforms: Vec::new(),
            accepted_transforms: Vec::new(),
        }
    }
}

/// Minimizes timeline actions while retaining request validity and one exact rigid signature.
///
/// The divergent action and its same-timeline setup prefix remain bit-identical. Required witness
/// families, declaration identities, transition witnesses, lifecycle order, checkpoint
/// references, and bounds are additionally preserved by decoding every candidate through the
/// strict rigid request boundary before evaluation.
///
/// # Errors
///
/// Returns [`RigidMinimizationError`] only when final canonical serialization fails.
pub fn minimize_rigid_world_request<F>(
    request: &RigidWorldRequestRecord,
    target: &RigidFailureSignature,
    budget: MinimizationBudget,
    mut evaluator: F,
) -> Result<RigidMinimizationResult, RigidMinimizationError>
where
    F: FnMut(&RigidWorldRequestRecord) -> RigidEvaluation,
{
    let limits = HarnessLimits::phase2_default_v1();
    let mut state = RigidReductionState::new(request);
    if budget.max_attempts == 0 {
        return RigidMinimizationResult::new(
            state,
            MinimizationStatus::Incomplete(BudgetExhausted::Attempts),
        );
    }
    if budget.deadline.is_zero() {
        return RigidMinimizationResult::new(
            state,
            MinimizationStatus::Incomplete(BudgetExhausted::Deadline),
        );
    }

    loop {
        let transforms = rigid_candidate_transforms(&state.request, target);
        let mut accepted = false;
        for transform in transforms {
            if state.attempts >= budget.max_attempts {
                return RigidMinimizationResult::new(
                    state,
                    MinimizationStatus::Incomplete(BudgetExhausted::Attempts),
                );
            }
            state.attempts += 1;
            state.attempted_transforms.push(transform);
            let Some(candidate) = apply_rigid_transform(&state.request, transform, &limits) else {
                state.rejected_invalid_candidates += 1;
                continue;
            };

            state.evaluations += 1;
            let evaluation = evaluator(&candidate);
            state.elapsed = state.elapsed.saturating_add(evaluation.elapsed);
            if evaluation.maybe_signature.as_ref() == Some(target) {
                state.request = candidate;
                state.accepted_transforms.push(transform);
                accepted = true;
            } else if evaluation.maybe_signature.is_some() {
                state.rejected_changed_signatures += 1;
            }
            if state.elapsed >= budget.deadline {
                return RigidMinimizationResult::new(
                    state,
                    MinimizationStatus::Incomplete(BudgetExhausted::Deadline),
                );
            }
            if accepted {
                break;
            }
        }
        if !accepted {
            return RigidMinimizationResult::new(state, MinimizationStatus::Complete);
        }
    }
}

fn rigid_candidate_transforms(
    request: &RigidWorldRequestRecord,
    target: &RigidFailureSignature,
) -> Vec<RigidScenarioTransform> {
    request
        .scenario()
        .timelines()
        .iter()
        .enumerate()
        .flat_map(|(timeline_index, timeline)| {
            let maybe_protected_prefix_end = timeline
                .actions()
                .iter()
                .position(|action| action.action_id().as_str() == target.action_id());
            removal_ranges(timeline.actions().len())
                .into_iter()
                .filter(move |(start, _end)| {
                    maybe_protected_prefix_end.is_none_or(|protected| *start > protected)
                })
                .map(move |(start, end)| RigidScenarioTransform::RemoveActions {
                    timeline_index,
                    start,
                    end,
                })
        })
        .collect()
}

fn apply_rigid_transform(
    request: &RigidWorldRequestRecord,
    transform: RigidScenarioTransform,
    limits: &HarnessLimits,
) -> Option<RigidWorldRequestRecord> {
    let mut value = serde_json::to_value(request).ok()?;
    let RigidScenarioTransform::RemoveActions {
        timeline_index,
        start,
        end,
    } = transform;
    let actions = value
        .get_mut("scenario")?
        .get_mut("timelines")?
        .as_array_mut()?
        .get_mut(timeline_index)?
        .get_mut("actions")?
        .as_array_mut()?;
    if start >= end || end > actions.len() {
        return None;
    }
    actions.drain(start..end);
    let mut bytes = serde_json::to_vec(&value).ok()?;
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, limits).ok()
}
