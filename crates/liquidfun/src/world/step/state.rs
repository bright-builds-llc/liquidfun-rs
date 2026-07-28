use super::{
    Arc, AtomicBool, MAX_CONTINUOUS_WORK, MAX_STEP_COMMANDS, MAX_STEP_EVENTS, Ordering, StepError,
};

#[derive(Debug, Clone)]
pub(in crate::world) struct StepState {
    pub(super) inner: Arc<StepStateInner>,
}

#[derive(Debug)]
pub(super) struct StepStateInner {
    pub(super) locked: AtomicBool,
    pub(super) poisoned: AtomicBool,
}

impl StepState {
    pub(in crate::world) fn new() -> Self {
        Self {
            inner: Arc::new(StepStateInner {
                locked: AtomicBool::new(false),
                poisoned: AtomicBool::new(false),
            }),
        }
    }

    pub(in crate::world) fn is_poisoned(&self) -> bool {
        self.inner.poisoned.load(Ordering::Relaxed)
    }

    pub(super) fn poison(&self) {
        self.inner.poisoned.store(true, Ordering::Relaxed);
    }

    pub(in crate::world) fn is_locked(&self) -> bool {
        self.inner.locked.load(Ordering::Relaxed)
    }

    #[cfg(test)]
    pub(in crate::world) fn set_locked_for_test(&self, locked: bool) {
        self.inner.locked.store(locked, Ordering::Relaxed);
    }

    #[cfg(test)]
    pub(in crate::world) fn set_poisoned_for_test(&self, poisoned: bool) {
        self.inner.poisoned.store(poisoned, Ordering::Relaxed);
    }
}

/// Reviewed finite limits for one automatic step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepLimits {
    pub(super) max_events: usize,
    pub(super) max_commands: usize,
    pub(super) max_continuous_work: usize,
    pub(super) maybe_failure_injection: Option<crate::world::island::SolveFailureInjection>,
}

impl StepLimits {
    /// Creates limits within the crate's reviewed hard maxima.
    ///
    /// # Errors
    ///
    /// Returns [`StepError::InvalidLimits`] when either value exceeds its hard maximum.
    pub const fn new(max_events: usize, max_commands: usize) -> Result<Self, StepError> {
        if max_events > MAX_STEP_EVENTS || max_commands > MAX_STEP_COMMANDS {
            return Err(StepError::InvalidLimits {
                max_events,
                max_commands,
            });
        }
        Ok(Self {
            max_events,
            max_commands,
            max_continuous_work: 64,
            maybe_failure_injection: None,
        })
    }

    /// Returns the maximum number of owned contact events.
    #[must_use]
    pub const fn max_events(self) -> usize {
        self.max_events
    }

    /// Returns the maximum number of deferred commands.
    #[must_use]
    pub const fn max_commands(self) -> usize {
        self.max_commands
    }

    /// Returns the maximum number of continuous events accepted by one call.
    #[must_use]
    pub const fn max_continuous_work(self) -> usize {
        self.max_continuous_work
    }

    /// Returns limits with a checked per-call continuous-event budget.
    ///
    /// Zero is valid and creates a coherent checkpoint immediately after the
    /// discrete stage. A matching later call resumes without repeating that
    /// stage.
    ///
    /// # Errors
    ///
    /// Returns [`StepError::InvalidContinuousWorkLimit`] when `maximum`
    /// exceeds the reviewed hard maximum.
    pub const fn with_continuous_work_limit(mut self, maximum: usize) -> Result<Self, StepError> {
        if maximum > MAX_CONTINUOUS_WORK {
            return Err(StepError::InvalidContinuousWorkLimit {
                requested: maximum,
                maximum: MAX_CONTINUOUS_WORK,
            });
        }
        self.max_continuous_work = maximum;
        Ok(self)
    }

    /// Returns limits with one bounded transactional-solver failure injection.
    #[cfg(feature = "differential-internals")]
    #[doc(hidden)]
    #[must_use]
    pub fn with_rigid_failure_injection(
        mut self,
        injection: crate::rigid_differential::RigidStepFailureInjection,
    ) -> Self {
        self.maybe_failure_injection = Some(match injection {
            crate::rigid_differential::RigidStepFailureInjection::LateIsland { solved_islands } => {
                crate::world::island::SolveFailureInjection::LateIsland { solved_islands }
            }
            crate::rigid_differential::RigidStepFailureInjection::ProxyBounds { fixture } => {
                crate::world::island::SolveFailureInjection::ProxyBounds { fixture }
            }
        });
        self
    }
}

impl Default for StepLimits {
    fn default() -> Self {
        Self {
            max_events: 256,
            max_commands: 64,
            max_continuous_work: 64,
            maybe_failure_injection: None,
        }
    }
}
