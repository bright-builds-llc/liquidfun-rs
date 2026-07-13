//! Automatic checked rigid-world stepping, restricted hooks, and owned reports.

use std::error::Error;
use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{AggregateMassError, BodyId, DestructionRecord, FixtureId, HandleError, World};

use super::fixture::FixtureDestructionError;

use super::config::{StepCompletion, StepConfiguration};
use super::contact::{ContactPointSnapshot, ContactTransition, ManagedContactSnapshot};
use super::contact_manager::HookContactOccurrence;
use super::contact_solver::{ContactSolve, ContactSolveFailure};
use super::continuous::{ContinuousStepKey, ContinuousStepKind};

mod continuous;
pub use continuous::ContinuousProgress;

const MAX_STEP_EVENTS: usize = 4_096;
const MAX_STEP_COMMANDS: usize = 1_024;
const MAX_CONTINUOUS_WORK: usize = 1_024;

#[derive(Debug, Clone)]
pub(super) struct StepState {
    inner: Arc<StepStateInner>,
}

#[derive(Debug)]
struct StepStateInner {
    locked: AtomicBool,
    poisoned: AtomicBool,
}

impl StepState {
    pub(super) fn new() -> Self {
        Self {
            inner: Arc::new(StepStateInner {
                locked: AtomicBool::new(false),
                poisoned: AtomicBool::new(false),
            }),
        }
    }

    pub(super) fn is_poisoned(&self) -> bool {
        self.inner.poisoned.load(Ordering::Relaxed)
    }

    fn poison(&self) {
        self.inner.poisoned.store(true, Ordering::Relaxed);
    }

    pub(super) fn is_locked(&self) -> bool {
        self.inner.locked.load(Ordering::Relaxed)
    }

    #[cfg(test)]
    pub(super) fn set_locked_for_test(&self, locked: bool) {
        self.inner.locked.store(locked, Ordering::Relaxed);
    }

    #[cfg(test)]
    pub(super) fn set_poisoned_for_test(&self, poisoned: bool) {
        self.inner.poisoned.store(poisoned, Ordering::Relaxed);
    }
}

/// Reviewed finite limits for one automatic step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepLimits {
    max_events: usize,
    max_commands: usize,
    max_continuous_work: usize,
    maybe_failure_injection: Option<super::island::SolveFailureInjection>,
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
                super::island::SolveFailureInjection::LateIsland { solved_islands }
            }
            crate::rigid_differential::RigidStepFailureInjection::ProxyBounds { fixture } => {
                super::island::SolveFailureInjection::ProxyBounds { fixture }
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

/// A read-only view valid only for the duration of one hook call.
///
/// A hook cannot retain the view beyond its callback lifetime:
///
/// ```compile_fail
/// use liquidfun::{ContactView, StepHook};
///
/// struct RetainingHook {
///     retained: Option<ContactView<'static>>,
/// }
///
/// impl StepHook for RetainingHook {
///     fn observe(&mut self, contact: ContactView<'_>) {
///         self.retained = Some(contact);
///     }
/// }
/// ```
#[derive(Clone, Copy)]
pub struct ContactView<'step> {
    contact: &'step ManagedContactSnapshot,
}

impl<'step> ContactView<'step> {
    /// Returns owned typed fixture identities without exposing contact storage.
    #[must_use]
    pub const fn fixtures(self) -> [FixtureId; 2] {
        self.contact.fixtures()
    }

    /// Returns shape-child coordinates in oriented manager order.
    #[must_use]
    pub const fn child_indices(self) -> [crate::collision::ChildIndex; 2] {
        self.contact.child_indices()
    }

    /// Returns whether the occurrence is currently touching.
    #[must_use]
    pub const fn is_touching(self) -> bool {
        self.contact.is_touching()
    }

    /// Returns whether the occurrence bypasses pre-solve and constraints.
    #[must_use]
    pub const fn is_sensor(self) -> bool {
        self.contact.is_sensor()
    }

    /// Returns the canonical manifold when this is a solid touching occurrence.
    #[must_use]
    pub const fn maybe_manifold(self) -> Option<&'step crate::collision::Manifold> {
        self.contact.maybe_manifold()
    }

    /// Returns warm-start points in canonical manifold order.
    #[must_use]
    pub fn points(self) -> &'step [ContactPointSnapshot] {
        self.contact.points()
    }

    /// Returns the creation-time mixed friction coefficient.
    #[must_use]
    pub const fn friction(self) -> f32 {
        self.contact.friction()
    }

    /// Returns the creation-time mixed restitution coefficient.
    #[must_use]
    pub const fn restitution(self) -> f32 {
        self.contact.restitution()
    }

    fn snapshot(self) -> ManagedContactSnapshot {
        self.contact.clone()
    }
}

impl fmt::Debug for ContactView<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ContactView")
            .field("fixtures", &self.fixtures())
            .field("child_indices", &self.child_indices())
            .field("touching", &self.is_touching())
            .field("sensor", &self.is_sensor())
            .finish_non_exhaustive()
    }
}

/// Narrow collision-filter result returned by a hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionDirective {
    /// Continue processing the occurrence.
    Collide,
    /// Ignore this occurrence before pre-solve processing.
    Ignore,
}

/// Narrow pre-solve result returned by a hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreSolveDirective {
    /// Keep the occurrence enabled.
    Enable,
    /// Disable the occurrence for this step.
    Disable,
}

/// Restricted synchronous step hooks.
///
/// Hooks receive only borrow-scoped read-only contact views and return narrow
/// decisions. They do not receive mutable world access:
///
/// ```compile_fail
/// use liquidfun::{ContactView, StepHook, World};
///
/// struct MutatingHook;
///
/// impl StepHook for MutatingHook {
///     fn observe(&mut self, _world: &mut World, _contact: ContactView<'_>) {}
/// }
/// ```
pub trait StepHook {
    /// Decides whether one contact occurrence should proceed.
    fn filter(&mut self, _contact: ContactView<'_>) -> CollisionDirective {
        CollisionDirective::Collide
    }

    /// Decides whether one supported solid occurrence remains enabled.
    fn pre_solve(&mut self, _contact: ContactView<'_>) -> PreSolveDirective {
        PreSolveDirective::Enable
    }

    /// Observes one non-filtered occurrence without mutable world access.
    fn observe(&mut self, _contact: ContactView<'_>) {}

    /// Optionally requests one owned mutation after observing an occurrence.
    fn command(&mut self, _contact: ContactView<'_>) -> Option<WorldCommand> {
        None
    }
}

/// A supported owned mutation requested by a step hook.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WorldCommand {
    /// Destroy a body and its dependents.
    DestroyBody(BodyId),
    /// Destroy one fixture.
    DestroyFixture(FixtureId),
}

/// Why one deferred command could not be applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CommandError {
    /// A typed command operand was foreign, stale, or destroyed at application time.
    InvalidHandle(HandleError),
    /// A fixture destruction would produce an invalid aggregate body mass.
    InvalidAggregateMass(AggregateMassError),
    /// An internal lifecycle violation attempted application while the world was locked.
    Locked,
}

impl fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHandle(error) => write!(formatter, "invalid command handle: {error}"),
            Self::InvalidAggregateMass(error) => {
                write!(formatter, "invalid aggregate body mass: {error}")
            }
            Self::Locked => formatter.write_str("cannot apply command while world is locked"),
        }
    }
}

impl Error for CommandError {}

/// Owned deterministic result for one deferred command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandApplication {
    command: WorldCommand,
    result: Result<Vec<DestructionRecord>, CommandError>,
}

impl CommandApplication {
    /// Returns the exact command requested by the hook.
    #[must_use]
    pub const fn command(&self) -> WorldCommand {
        self.command
    }

    /// Returns owned destruction evidence or the explicit per-command rejection.
    ///
    /// # Errors
    ///
    /// Returns the command's recoverable application error when its operand was invalid.
    pub fn result(&self) -> Result<&[DestructionRecord], CommandError> {
        self.result
            .as_ref()
            .map(Vec::as_slice)
            .map_err(|error| *error)
    }
}

/// Owned callback evidence for one manager occurrence.
#[derive(Debug, Clone, PartialEq)]
pub struct ContactEvent {
    contact: ManagedContactSnapshot,
    collision: CollisionDirective,
    maybe_pre_solve: Option<PreSolveDirective>,
}

impl ContactEvent {
    /// Returns the owned semantic contact snapshot.
    #[must_use]
    pub const fn contact(&self) -> &ManagedContactSnapshot {
        &self.contact
    }

    /// Returns the collision-filter decision.
    #[must_use]
    pub const fn collision(&self) -> CollisionDirective {
        self.collision
    }

    /// Returns the pre-solve decision for a supported non-sensor occurrence.
    #[must_use]
    pub const fn maybe_pre_solve(&self) -> Option<PreSolveDirective> {
        self.maybe_pre_solve
    }
}

/// One named Phase 6 step seam in exact execution order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StepPhase {
    /// Discover broad-phase pairs and admit new private contacts.
    FindPairs,
    /// Refilter and update all admitted contact occurrences.
    UpdateContacts,
    /// Invoke restricted hooks for touching manager occurrences.
    Hook,
    /// Execute the reviewed bounded constraint solve.
    Solve,
    /// Release the world lock before requested mutation.
    Unlock,
    /// Apply all queued commands sequentially after unlock.
    ApplyCommands,
}

/// Owned results from one step occurrence in exact production order.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum StepLifecycleEvent {
    /// A private manager contact began, persisted, or ended touching.
    Contact(ContactTransition),
    /// A restricted hook observed one owned manager occurrence.
    Hook(ContactEvent),
    /// One private manager occurrence completed bounded solving.
    Solve(ContactSolve),
    /// One requested mutation completed after unlock.
    Command(CommandApplication),
    /// A world object was invalidated after dependent contact evidence.
    Destruction(DestructionRecord),
}

/// Owned results from one automatic step.
#[derive(Debug, Default, PartialEq)]
pub struct StepReport {
    completion: StepCompletion,
    time_step_ratio: f32,
    phases: Vec<StepPhase>,
    events: Vec<ContactEvent>,
    contact_transitions: Vec<ContactTransition>,
    contact_solves: Vec<ContactSolve>,
    continuous_contact_solves: Vec<ContactSolve>,
    lifecycle: Vec<StepLifecycleEvent>,
    destructions: Vec<DestructionRecord>,
    command_applications: Vec<CommandApplication>,
}

impl StepReport {
    /// Returns whether continuous work completed or remains pending.
    #[must_use]
    pub const fn completion(&self) -> StepCompletion {
        self.completion
    }

    /// Returns the source-ordered warm-start ratio for this call.
    #[must_use]
    pub const fn time_step_ratio(&self) -> f32 {
        self.time_step_ratio
    }

    /// Returns named execution seams in exact phase order.
    #[must_use]
    pub fn phases(&self) -> &[StepPhase] {
        &self.phases
    }

    /// Returns callback events in exact manager occurrence order.
    #[must_use]
    pub fn events(&self) -> &[ContactEvent] {
        &self.events
    }

    /// Returns automatic touching transitions in manager occurrence order.
    #[must_use]
    pub fn contact_transitions(&self) -> &[ContactTransition] {
        &self.contact_transitions
    }

    /// Returns post-solve semantic state in fixed solver order.
    #[must_use]
    pub fn contact_solves(&self) -> &[ContactSolve] {
        &self.contact_solves
    }

    /// Returns transient post-solve state for committed continuous events.
    ///
    /// These snapshots are reported in TOI solver order and are not stored in
    /// the persistent warm-start impulse lanes.
    #[must_use]
    pub fn continuous_contact_solves(&self) -> &[ContactSolve] {
        &self.continuous_contact_solves
    }

    /// Returns all owned lifecycle evidence in exact production order.
    #[must_use]
    pub fn lifecycle(&self) -> &[StepLifecycleEvent] {
        &self.lifecycle
    }

    /// Returns owned destruction evidence in command-application order.
    #[must_use]
    pub fn destructions(&self) -> &[DestructionRecord] {
        &self.destructions
    }

    /// Returns one result per requested command in exact request order.
    #[must_use]
    pub fn command_applications(&self) -> &[CommandApplication] {
        &self.command_applications
    }
}

/// A checked step-lifecycle failure.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum StepError {
    /// A prior hook panic left coherent world operations poisoned.
    Poisoned,
    /// Contact lifecycle completed, but its active solver topology is deferred.
    UnsupportedSolverTopology {
        /// Owned lifecycle evidence committed before fail-closed preflight.
        contact_transitions: Vec<ContactTransition>,
    },
    /// Contact lifecycle completed, but solver construction produced non-finite state.
    NonFiniteSolverState {
        /// Owned lifecycle evidence committed before fail-closed construction.
        contact_transitions: Vec<ContactTransition>,
    },
    /// Staged body motion would produce invalid broad-phase synchronization bounds.
    InvalidSolverProxyBounds {
        /// Owned lifecycle evidence committed before fail-closed solver staging.
        contact_transitions: Vec<ContactTransition>,
    },
    /// The world is already executing a step.
    Locked,
    /// A bounded per-step resource reached its configured limit.
    LimitExceeded {
        /// Name of the bounded resource.
        resource: &'static str,
        /// Configured finite limit.
        limit: usize,
    },
    /// A coherent continuous checkpoint reached its configured work budget.
    ContinuousWorkLimitExceeded {
        /// Configured finite limit for this call.
        limit: usize,
        /// Semantic progress retained for a matching continuation.
        progress: ContinuousProgress,
    },
    /// Requested limits exceed the reviewed hard maxima.
    InvalidLimits {
        /// Requested event limit.
        max_events: usize,
        /// Requested command limit.
        max_commands: usize,
    },
    /// A requested continuous-work limit exceeds the reviewed hard maximum.
    InvalidContinuousWorkLimit {
        /// Rejected continuous-event budget.
        requested: usize,
        /// Largest accepted continuous-event budget.
        maximum: usize,
    },
}

impl fmt::Display for StepError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Poisoned => formatter.write_str("world is poisoned by a prior hook panic"),
            Self::UnsupportedSolverTopology { .. } => {
                formatter.write_str("contact solver topology is deferred beyond Phase 6")
            }
            Self::NonFiniteSolverState { .. } => {
                formatter.write_str("contact solver produced non-finite state")
            }
            Self::InvalidSolverProxyBounds { .. } => {
                formatter.write_str("contact solver produced invalid proxy bounds")
            }
            Self::Locked => formatter.write_str("world is locked by an active step"),
            Self::LimitExceeded { resource, limit } => {
                write!(formatter, "step {resource} limit of {limit} was exceeded")
            }
            Self::ContinuousWorkLimitExceeded { limit, progress } => write!(
                formatter,
                "continuous work limit of {limit} was reached after {} committed events",
                progress.completed_events()
            ),
            Self::InvalidLimits {
                max_events,
                max_commands,
            } => write!(
                formatter,
                "step limits exceed hard maxima: events={max_events}, commands={max_commands}"
            ),
            Self::InvalidContinuousWorkLimit { requested, maximum } => write!(
                formatter,
                "continuous work limit must be within 0..={maximum}, got {requested}"
            ),
        }
    }
}

impl Error for StepError {}

struct StepLockGuard {
    state: StepState,
}

impl StepLockGuard {
    fn acquire(state: &StepState) -> Result<Self, StepError> {
        if state.inner.locked.swap(true, Ordering::Relaxed) {
            return Err(StepError::Locked);
        }
        Ok(Self {
            state: state.clone(),
        })
    }
}

impl Drop for StepLockGuard {
    fn drop(&mut self) {
        self.state.inner.locked.store(false, Ordering::Relaxed);
    }
}

impl World {
    /// Runs one automatic bounded rigid-world lifecycle and solve.
    ///
    /// Checked configuration is supplied by [`StepConfiguration`]. Discrete
    /// islands are staged and committed transactionally in deterministic source
    /// order. Continuous work remains private and resumable: sub-stepping uses
    /// [`StepCompletion::ContinuousPending`], while budget exhaustion returns
    /// [`StepError::ContinuousWorkLimitExceeded`] with [`ContinuousProgress`].
    /// Accumulated forces clear only after a successful call when automatic
    /// clearing is enabled.
    ///
    /// # Errors
    ///
    /// Returns an error for poisoned or nested stepping, exhausted limits,
    /// unsupported topology, or non-finite solver state.
    pub fn step<H: StepHook>(
        &mut self,
        configuration: StepConfiguration,
        hook: &mut H,
        limits: StepLimits,
    ) -> Result<StepReport, StepError> {
        if self.step_state.is_poisoned() {
            return Err(StepError::Poisoned);
        }
        let timing = self.prepare_step_timing(configuration);
        let mut phases = Vec::with_capacity(6);
        let continuous_enabled =
            self.is_continuous_physics_enabled() && configuration.time_step() > 0.0;
        let continuous_key = ContinuousStepKey::from_configuration(configuration);
        let step_lock = StepLockGuard::acquire(&self.step_state)?;
        let step_kind = if continuous_enabled {
            self.continuous_step_state
                .begin_step(continuous_key, &mut self.contact_manager)
        } else {
            self.continuous_step_state.invalidate();
            ContinuousStepKind::Fresh
        };
        let mut contact_transitions;
        let events;
        let commands;
        let mut contact_solves = Vec::new();
        let mut continuous_contact_solves = Vec::new();
        let completion = {
            let _lock = step_lock;
            phases.push(StepPhase::FindPairs);
            self.find_pairs();
            phases.push(StepPhase::UpdateContacts);
            self.update_contacts_for_step();
            contact_transitions = self.contact_manager.drain_transitions();
            if step_kind == ContinuousStepKind::Fresh && configuration.time_step() > 0.0 {
                self.preflight_contact_solver()
                    .map_err(|error| solver_step_error(error, &contact_transitions))?;
            }

            phases.push(StepPhase::Hook);
            let occurrences = self.contact_manager.hook_contacts();
            (events, commands) = self.run_contact_hooks(&occurrences, hook, limits)?;
            if step_kind == ContinuousStepKind::Fresh && configuration.time_step() > 0.0 {
                phases.push(StepPhase::Solve);
                contact_solves = self
                    .solve_contact_constraints(
                        configuration,
                        timing,
                        limits.maybe_failure_injection,
                    )
                    .map_err(|error| solver_step_error(error, &contact_transitions))?;
            }

            if continuous_enabled {
                let continuous = self.run_continuous_stage(
                    configuration,
                    continuous_key,
                    limits.max_continuous_work,
                    &contact_transitions,
                )?;
                continuous_contact_solves = continuous.contact_solves;
                contact_transitions.extend(self.contact_manager.drain_transitions());
                continuous.completion
            } else {
                StepCompletion::Complete
            }
        };
        phases.push(StepPhase::Unlock);

        let mut lifecycle = contact_transitions
            .iter()
            .cloned()
            .map(StepLifecycleEvent::Contact)
            .collect::<Vec<_>>();
        lifecycle.extend(events.iter().cloned().map(StepLifecycleEvent::Hook));
        lifecycle.extend(
            contact_solves
                .iter()
                .cloned()
                .map(StepLifecycleEvent::Solve),
        );
        lifecycle.extend(
            continuous_contact_solves
                .iter()
                .cloned()
                .map(StepLifecycleEvent::Solve),
        );
        if !commands.is_empty() {
            phases.push(StepPhase::ApplyCommands);
        }
        let (command_applications, destructions, command_transitions, command_lifecycle) =
            self.apply_commands(commands);
        contact_transitions.extend(command_transitions);
        lifecycle.extend(command_lifecycle);

        let completion = self.finish_successful_step(timing, completion);
        Ok(StepReport {
            completion,
            time_step_ratio: timing.time_step_ratio(),
            phases,
            events,
            contact_transitions,
            contact_solves,
            continuous_contact_solves,
            lifecycle,
            destructions,
            command_applications,
        })
    }

    fn find_pairs(&mut self) {
        self.find_new_contacts();
    }

    fn finish_successful_step(
        &mut self,
        timing: super::config::StepTiming,
        completion: StepCompletion,
    ) -> StepCompletion {
        self.commit_step_timing(timing);
        if self.is_automatic_force_clearing_enabled() {
            self.clear_force_accumulators();
        }
        completion
    }

    fn update_contacts_for_step(&mut self) {
        self.update_contacts();
    }

    fn solve_contact_constraints(
        &mut self,
        configuration: StepConfiguration,
        timing: super::config::StepTiming,
        maybe_failure_injection: Option<super::island::SolveFailureInjection>,
    ) -> Result<Vec<ContactSolve>, ContactSolveFailure> {
        self.solve_contacts(configuration, timing, maybe_failure_injection)
    }

    fn run_contact_hooks<H: StepHook>(
        &mut self,
        occurrences: &[HookContactOccurrence],
        hook: &mut H,
        limits: StepLimits,
    ) -> Result<(Vec<ContactEvent>, Vec<WorldCommand>), StepError> {
        let mut events = Vec::with_capacity(occurrences.len().min(limits.max_events));
        let mut commands = Vec::with_capacity(occurrences.len().min(limits.max_commands));
        for occurrence in occurrences {
            check_capacity(events.len(), limits.max_events, "event")?;
            let view = ContactView {
                contact: &occurrence.snapshot,
            };
            let callback = catch_unwind(AssertUnwindSafe(|| {
                invoke_hook(hook, view, !view.is_sensor())
            }));
            let (collision, maybe_pre_solve, maybe_command) = match callback {
                Ok(output) => output,
                Err(payload) => {
                    self.step_state.poison();
                    resume_unwind(payload);
                }
            };
            let enabled = collision == CollisionDirective::Collide
                && maybe_pre_solve != Some(PreSolveDirective::Disable);
            self.contact_manager
                .set_hook_enabled(occurrence.ordinal, enabled);
            if let Some(command) = maybe_command {
                check_capacity(commands.len(), limits.max_commands, "command")?;
                commands.push(command);
            }
            events.push(ContactEvent {
                contact: view.snapshot(),
                collision,
                maybe_pre_solve,
            });
        }
        Ok((events, commands))
    }

    fn apply_commands(
        &mut self,
        commands: Vec<WorldCommand>,
    ) -> (
        Vec<CommandApplication>,
        Vec<DestructionRecord>,
        Vec<ContactTransition>,
        Vec<StepLifecycleEvent>,
    ) {
        let mut applications = Vec::with_capacity(commands.len());
        let mut destructions = Vec::new();
        let mut contact_transitions = Vec::new();
        let mut lifecycle = Vec::new();
        for command in commands {
            let result = self.apply_command(command);
            let transitions = self.contact_manager.drain_transitions();
            let application = CommandApplication { command, result };
            lifecycle.push(StepLifecycleEvent::Command(application.clone()));
            lifecycle.extend(transitions.iter().cloned().map(StepLifecycleEvent::Contact));
            contact_transitions.extend(transitions);
            if let Ok(records) = &application.result {
                destructions.extend(records.iter().cloned());
                lifecycle.extend(records.iter().cloned().map(StepLifecycleEvent::Destruction));
            }
            applications.push(application);
        }
        (applications, destructions, contact_transitions, lifecycle)
    }

    fn apply_command(
        &mut self,
        command: WorldCommand,
    ) -> Result<Vec<DestructionRecord>, CommandError> {
        if self.step_state.is_locked() {
            return Err(CommandError::Locked);
        }
        match command {
            WorldCommand::DestroyBody(body) => {
                self.destroy_body(body).map_err(CommandError::InvalidHandle)
            }
            WorldCommand::DestroyFixture(fixture) => self
                .destroy_fixture(fixture)
                .map(|record| vec![record])
                .map_err(|error| match error {
                    FixtureDestructionError::InvalidHandle(error) => {
                        CommandError::InvalidHandle(error)
                    }
                    FixtureDestructionError::InvalidAggregateMass(error) => {
                        CommandError::InvalidAggregateMass(error)
                    }
                }),
        }
    }

    /// Returns whether this world is currently inside its step lock.
    #[must_use]
    pub fn is_locked(&self) -> bool {
        self.step_state.is_locked()
    }

    /// Returns whether a hook panic permanently poisoned coherent operations.
    #[must_use]
    pub fn is_poisoned(&self) -> bool {
        self.step_state.is_poisoned()
    }
}

fn invoke_hook<H: StepHook>(
    hook: &mut H,
    view: ContactView<'_>,
    allow_pre_solve: bool,
) -> (
    CollisionDirective,
    Option<PreSolveDirective>,
    Option<WorldCommand>,
) {
    let collision = hook.filter(view);
    if collision == CollisionDirective::Ignore {
        return (collision, None, None);
    }
    let maybe_directive = allow_pre_solve.then(|| hook.pre_solve(view));
    hook.observe(view);
    let maybe_command = hook.command(view);
    (collision, maybe_directive, maybe_command)
}

fn solver_step_error(
    error: ContactSolveFailure,
    contact_transitions: &[ContactTransition],
) -> StepError {
    match error {
        ContactSolveFailure::UnsupportedTopology => StepError::UnsupportedSolverTopology {
            contact_transitions: contact_transitions.to_vec(),
        },
        ContactSolveFailure::NonFinite => StepError::NonFiniteSolverState {
            contact_transitions: contact_transitions.to_vec(),
        },
        ContactSolveFailure::CapacityExceeded { resource, limit } => {
            StepError::LimitExceeded { resource, limit }
        }
        ContactSolveFailure::InvalidProxyBounds => StepError::InvalidSolverProxyBounds {
            contact_transitions: contact_transitions.to_vec(),
        },
    }
}

fn check_capacity(current: usize, limit: usize, resource: &'static str) -> Result<(), StepError> {
    if current == limit {
        return Err(StepError::LimitExceeded { resource, limit });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::math::Vec2;
    use crate::{BodyDef, BodyType, StepConfiguration, WakePolicy, World};

    use super::StepCompletion;

    #[test]
    fn successful_continuous_pending_path_clears_forces() {
        // Arrange
        let mut world = World::new().expect("world key should remain available");
        let definition = BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)
            .expect("test body definition should be valid");
        let body = world
            .create_body(&definition)
            .expect("test body should fit");
        world
            .apply_body_force_to_center(body, Vec2::new(f32::MAX, 0.0), WakePolicy::Wake)
            .expect("first finite force should be accepted");
        let configuration = StepConfiguration::new(1.0 / 60.0, 8, 3)
            .expect("test step configuration should be valid");
        let timing = world.prepare_step_timing(configuration);

        // Act
        let completion = world.finish_successful_step(timing, StepCompletion::ContinuousPending);

        // Assert
        assert_eq!(completion, StepCompletion::ContinuousPending);
        world
            .apply_body_force_to_center(body, Vec2::new(f32::MAX, 0.0), WakePolicy::Wake)
            .expect("successful pending path should clear the force accumulator");
    }
}
