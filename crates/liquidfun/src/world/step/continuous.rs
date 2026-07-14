use super::{
    CollisionDecisionHook, ContactHookRun, ContactTransition, StepCompletion, StepConfiguration,
    StepError,
};
use crate::World;

use crate::world::contact_solver::{ContactSolve, ContactSolveFailure};
use crate::world::continuous::{ContinuousEventError, ContinuousScanError, ContinuousStepKey};
use crate::world::island::{IslandBuildError, ToiIslandLimits};

/// Semantic progress retained after one continuous-work budget is exhausted.
#[derive(Debug, Clone, PartialEq)]
pub struct ContinuousProgress {
    discrete_completed: bool,
    completed_events: usize,
    contact_solves: Vec<ContactSolve>,
}

impl ContinuousProgress {
    const fn after_discrete(completed_events: usize, contact_solves: Vec<ContactSolve>) -> Self {
        Self {
            discrete_completed: true,
            completed_events,
            contact_solves,
        }
    }

    /// Returns whether the discrete stage committed before the checkpoint.
    #[must_use]
    pub const fn discrete_completed(&self) -> bool {
        self.discrete_completed
    }

    /// Returns the number of continuous events committed by this call.
    #[must_use]
    pub const fn completed_events(&self) -> usize {
        self.completed_events
    }

    /// Returns transient post-solve state for committed continuous events.
    ///
    /// These snapshots do not populate persistent warm-start impulse lanes.
    #[must_use]
    pub fn contact_solves(&self) -> &[ContactSolve] {
        &self.contact_solves
    }
}

pub(super) struct ContinuousStageResult {
    pub(super) completion: StepCompletion,
    pub(super) contact_solves: Vec<ContactSolve>,
}

impl World {
    pub(super) fn run_continuous_stage<H: CollisionDecisionHook>(
        &mut self,
        configuration: StepConfiguration,
        key: ContinuousStepKey,
        work_limit: usize,
        contact_transitions: &[ContactTransition],
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<ContinuousStageResult, StepError> {
        let mut completed_events = 0;
        let mut contact_solves = Vec::new();
        loop {
            if completed_events == work_limit {
                self.continuous_step_state.mark_pending(key);
                return Err(StepError::ContinuousWorkLimitExceeded {
                    limit: work_limit,
                    progress: ContinuousProgress::after_discrete(completed_events, contact_solves),
                });
            }
            let maybe_event = self
                .solve_next_continuous_event_with_hook(
                    configuration,
                    ToiIslandLimits::REVIEWED,
                    false,
                    hook_run,
                )
                .map_err(|error| {
                    self.continuous_step_state.mark_pending(key);
                    continuous_step_error(error, contact_transitions)
                })?;
            let Some(event) = maybe_event else {
                self.continuous_step_state.invalidate();
                return Ok(ContinuousStageResult {
                    completion: StepCompletion::Complete,
                    contact_solves,
                });
            };
            contact_solves.extend(event.contact_solves);
            completed_events += 1;
            if self.is_sub_stepping_enabled() {
                self.continuous_step_state.mark_pending(key);
                return Ok(ContinuousStageResult {
                    completion: StepCompletion::ContinuousPending,
                    contact_solves,
                });
            }
        }
    }
}

fn continuous_step_error(
    error: ContinuousEventError,
    contact_transitions: &[ContactTransition],
) -> StepError {
    match error {
        ContinuousEventError::Scan(ContinuousScanError::Hook(error)) => error,
        ContinuousEventError::Scan(ContinuousScanError::CapacityExceeded { resource, limit })
        | ContinuousEventError::Island(IslandBuildError::CapacityExceeded { resource, limit })
        | ContinuousEventError::Solve(ContactSolveFailure::CapacityExceeded { resource, limit }) => {
            StepError::LimitExceeded { resource, limit }
        }
        ContinuousEventError::Scan(ContinuousScanError::InvalidGraph)
        | ContinuousEventError::Island(IslandBuildError::InvalidGraph)
        | ContinuousEventError::Solve(ContactSolveFailure::UnsupportedTopology) => {
            StepError::UnsupportedSolverTopology {
                contact_transitions: contact_transitions.to_vec(),
            }
        }
        ContinuousEventError::Solve(ContactSolveFailure::InvalidProxyBounds)
        | ContinuousEventError::ProxyBounds => StepError::InvalidSolverProxyBounds {
            contact_transitions: contact_transitions.to_vec(),
        },
        ContinuousEventError::Scan(
            ContinuousScanError::Collision(_)
            | ContinuousScanError::Sweep(_)
            | ContinuousScanError::ToiCountLimit,
        )
        | ContinuousEventError::Solve(ContactSolveFailure::NonFinite)
        | ContinuousEventError::InjectedFailure => StepError::NonFiniteSolverState {
            contact_transitions: contact_transitions.to_vec(),
        },
    }
}
