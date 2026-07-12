//! Restricted step hooks and owned reporting for the no-solver architecture spike.

use std::cell::Cell;
use std::error::Error;
use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};

use crate::{BodyId, DestructionRecord, FixtureId, HandleError, World};

use super::contact::ContactTransition;
use super::contact_solver::{ContactSolve, ContactSolveFailure};

#[cfg(test)]
use super::fixture::test_fixture_definition;
#[cfg(test)]
use crate::BodyDef;

const MAX_STEP_EVENTS: usize = 4_096;
const MAX_STEP_COMMANDS: usize = 1_024;

#[derive(Debug)]
pub(super) struct StepState {
    locked: Cell<bool>,
    poisoned: Cell<bool>,
}

impl StepState {
    pub(super) const fn new() -> Self {
        Self {
            locked: Cell::new(false),
            poisoned: Cell::new(false),
        }
    }

    pub(super) fn is_poisoned(&self) -> bool {
        self.poisoned.get()
    }

    fn poison(&self) {
        self.poisoned.set(true);
    }
}

/// Reviewed finite limits for one representative step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepLimits {
    max_events: usize,
    max_commands: usize,
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
}

impl Default for StepLimits {
    fn default() -> Self {
        Self {
            max_events: 256,
            max_commands: 64,
        }
    }
}

/// Owned fixture identities describing one transient contact occurrence.
///
/// This value is a semantic snapshot, not a durable contact identity. Supplying the same fixture
/// pair more than once represents distinct occurrences, and reports preserve those duplicates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContactSnapshot {
    fixtures: [FixtureId; 2],
}

impl ContactSnapshot {
    /// Creates an owned contact snapshot from two typed fixture identities.
    #[must_use]
    pub const fn new(first: FixtureId, second: FixtureId) -> Self {
        Self {
            fixtures: [first, second],
        }
    }

    /// Returns the fixture identities in occurrence order.
    #[must_use]
    pub const fn fixtures(self) -> [FixtureId; 2] {
        self.fixtures
    }
}

#[derive(Debug)]
struct TransientContact {
    snapshot: ContactSnapshot,
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
    contact: &'step TransientContact,
}

impl ContactView<'_> {
    /// Returns owned typed fixture identities without exposing contact storage.
    #[must_use]
    pub const fn fixtures(self) -> [FixtureId; 2] {
        self.contact.snapshot.fixtures()
    }

    fn snapshot(self) -> ContactSnapshot {
        self.contact.snapshot
    }
}

impl fmt::Debug for ContactView<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ContactView")
            .field("fixtures", &self.fixtures())
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
/// Hooks receive only borrow-scoped read-only contact views and return narrow decisions. They do
/// not receive mutable world access:
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

    /// Decides whether a non-filtered occurrence remains enabled for this step.
    fn pre_solve(&mut self, _contact: ContactView<'_>) -> PreSolveDirective {
        PreSolveDirective::Enable
    }

    /// Observes one non-filtered occurrence without receiving mutable world access.
    fn observe(&mut self, _contact: ContactView<'_>) {}

    /// Optionally requests one owned mutation after observing an occurrence.
    ///
    /// The returned command is queued while locked and revalidated only after the step unlocks.
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
    /// An internal lifecycle violation attempted application while the world was locked.
    Locked,
}

impl fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHandle(error) => write!(formatter, "invalid command handle: {error}"),
            Self::Locked => formatter.write_str("cannot apply command while world is locked"),
        }
    }
}

impl Error for CommandError {}

/// Owned deterministic result for one deferred command.
#[derive(Debug, PartialEq, Eq)]
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

/// Owned callback evidence for one occurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContactEvent {
    contact: ContactSnapshot,
    collision: CollisionDirective,
    maybe_pre_solve: Option<PreSolveDirective>,
}

impl ContactEvent {
    /// Returns the owned semantic contact snapshot.
    #[must_use]
    pub const fn contact(self) -> ContactSnapshot {
        self.contact
    }

    /// Returns the collision-filter decision.
    #[must_use]
    pub const fn collision(self) -> CollisionDirective {
        self.collision
    }

    /// Returns the pre-solve decision when collision filtering allowed the occurrence.
    #[must_use]
    pub const fn maybe_pre_solve(self) -> Option<PreSolveDirective> {
        self.maybe_pre_solve
    }
}

/// Owned results from one representative step.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum StepLifecycleEvent {
    /// A private manager contact began, persisted, or ended touching.
    Contact(ContactTransition),
    /// A world object was invalidated after dependent contact evidence.
    Destruction(DestructionRecord),
}

/// Owned results from one representative step.
#[derive(Debug, Default, PartialEq)]
pub struct StepReport {
    events: Vec<ContactEvent>,
    contact_transitions: Vec<ContactTransition>,
    contact_solves: Vec<ContactSolve>,
    lifecycle: Vec<StepLifecycleEvent>,
    destructions: Vec<DestructionRecord>,
    command_applications: Vec<CommandApplication>,
}

impl StepReport {
    /// Returns callback events in exact occurrence order, including duplicates.
    #[must_use]
    pub fn events(&self) -> &[ContactEvent] {
        &self.events
    }

    /// Returns automatic touching transitions in private manager occurrence order.
    #[must_use]
    pub fn contact_transitions(&self) -> &[ContactTransition] {
        &self.contact_transitions
    }

    /// Returns post-solve semantic state in fixed solver order.
    #[must_use]
    pub fn contact_solves(&self) -> &[ContactSolve] {
        &self.contact_solves
    }

    /// Returns contact and destruction evidence in exact production order.
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
    ///
    /// Recoverable stale and cross-world failures do not stop later commands.
    #[must_use]
    pub fn command_applications(&self) -> &[CommandApplication] {
        &self.command_applications
    }
}

/// A representative step-lifecycle failure.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum StepError {
    /// A prior hook panic left the representative world step poisoned.
    Poisoned,
    /// One of the contact fixture identities is foreign, stale, or destroyed.
    InvalidContact(HandleError),
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
    /// The world is already executing a step.
    Locked,
    /// A bounded per-step resource reached its configured limit.
    LimitExceeded {
        /// Name of the bounded resource.
        resource: &'static str,
        /// Configured finite limit.
        limit: usize,
    },
    /// Requested limits exceed the reviewed hard maxima.
    InvalidLimits {
        /// Requested event limit.
        max_events: usize,
        /// Requested command limit.
        max_commands: usize,
    },
}

impl fmt::Display for StepError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Poisoned => formatter.write_str("world is poisoned by a prior hook panic"),
            Self::InvalidContact(error) => write!(formatter, "invalid contact fixture: {error}"),
            Self::UnsupportedSolverTopology { .. } => {
                formatter.write_str("contact solver topology is deferred beyond Phase 6")
            }
            Self::NonFiniteSolverState { .. } => {
                formatter.write_str("contact solver produced non-finite state")
            }
            Self::Locked => formatter.write_str("world is locked by an active step"),
            Self::LimitExceeded { resource, limit } => {
                write!(formatter, "step {resource} limit of {limit} was exceeded")
            }
            Self::InvalidLimits {
                max_events,
                max_commands,
            } => write!(
                formatter,
                "step limits exceed hard maxima: events={max_events}, commands={max_commands}"
            ),
        }
    }
}

impl Error for StepError {}

struct StepLockGuard<'world> {
    state: &'world StepState,
}

impl<'world> StepLockGuard<'world> {
    fn acquire(state: &'world StepState) -> Result<Self, StepError> {
        if state.locked.replace(true) {
            return Err(StepError::Locked);
        }
        Ok(Self { state })
    }
}

impl Drop for StepLockGuard<'_> {
    fn drop(&mut self) {
        self.state.locked.set(false);
    }
}

impl World {
    /// Runs a bounded no-solver step over supplied semantic contact occurrences.
    ///
    /// Every occurrence is validated and dispatched in slice order. Duplicate snapshots remain
    /// duplicate events. Filtering and pre-solve hooks execute while the world is locked; this
    /// representative lifecycle performs no physics integration.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid contact handles, nested stepping, or an exhausted event limit.
    pub fn step<H: StepHook>(
        &mut self,
        contacts: &[ContactSnapshot],
        hook: &mut H,
        limits: StepLimits,
    ) -> Result<StepReport, StepError> {
        if self.step_state.is_poisoned() {
            return Err(StepError::Poisoned);
        }
        self.find_new_contacts();
        self.update_contacts();
        let mut contact_transitions = self.contact_manager.drain_transitions();
        let contact_solves = self.solve_contacts().map_err(|error| match error {
            ContactSolveFailure::UnsupportedTopology => StepError::UnsupportedSolverTopology {
                contact_transitions: contact_transitions.clone(),
            },
            ContactSolveFailure::NonFinite => StepError::NonFiniteSolverState {
                contact_transitions: contact_transitions.clone(),
            },
        })?;
        let mut lifecycle = contact_transitions
            .iter()
            .cloned()
            .map(StepLifecycleEvent::Contact)
            .collect::<Vec<_>>();
        let (events, commands) = {
            let _lock = StepLockGuard::acquire(&self.step_state)?;
            self.dispatch_hooks(contacts, hook, limits)?
        };
        let (command_applications, destructions, command_transitions, command_lifecycle) =
            self.apply_commands(commands);
        contact_transitions.extend(command_transitions);
        lifecycle.extend(command_lifecycle);

        Ok(StepReport {
            events,
            contact_transitions,
            contact_solves,
            lifecycle,
            destructions,
            command_applications,
        })
    }

    fn dispatch_hooks<H: StepHook>(
        &self,
        contacts: &[ContactSnapshot],
        hook: &mut H,
        limits: StepLimits,
    ) -> Result<(Vec<ContactEvent>, Vec<WorldCommand>), StepError> {
        let mut events = Vec::with_capacity(contacts.len().min(limits.max_events));
        let mut commands = Vec::with_capacity(contacts.len().min(limits.max_commands));

        for snapshot in contacts {
            check_capacity(events.len(), limits.max_events, "event")?;
            for fixture in snapshot.fixtures() {
                self.validate_fixture(fixture)
                    .map_err(StepError::InvalidContact)?;
            }

            let transient = TransientContact {
                snapshot: *snapshot,
            };
            let view = ContactView {
                contact: &transient,
            };
            let callback = catch_unwind(AssertUnwindSafe(|| invoke_hook(hook, view)));
            let (collision, maybe_pre_solve, maybe_command) = match callback {
                Ok(output) => output,
                Err(payload) => {
                    self.step_state.poison();
                    resume_unwind(payload);
                }
            };
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
            lifecycle.extend(transitions.iter().cloned().map(StepLifecycleEvent::Contact));
            contact_transitions.extend(transitions);
            if let Ok(records) = &result {
                destructions.extend(records.iter().cloned());
                lifecycle.extend(records.iter().cloned().map(StepLifecycleEvent::Destruction));
            }
            applications.push(CommandApplication { command, result });
        }
        (applications, destructions, contact_transitions, lifecycle)
    }

    fn apply_command(
        &mut self,
        command: WorldCommand,
    ) -> Result<Vec<DestructionRecord>, CommandError> {
        if self.step_state.locked.get() {
            return Err(CommandError::Locked);
        }
        match command {
            WorldCommand::DestroyBody(body) => {
                self.destroy_body(body).map_err(CommandError::InvalidHandle)
            }
            WorldCommand::DestroyFixture(fixture) => self
                .destroy_fixture(fixture)
                .map(|record| vec![record])
                .map_err(CommandError::InvalidHandle),
        }
    }

    /// Returns whether this world is currently inside its step lock.
    ///
    /// This diagnostic remains available after poisoning so callers can verify unwind cleanup.
    #[must_use]
    pub fn is_locked(&self) -> bool {
        self.step_state.locked.get()
    }

    /// Returns whether a hook panic permanently poisoned coherent-state operations.
    #[must_use]
    pub fn is_poisoned(&self) -> bool {
        self.step_state.is_poisoned()
    }
}

fn invoke_hook<H: StepHook>(
    hook: &mut H,
    view: ContactView<'_>,
) -> (
    CollisionDirective,
    Option<PreSolveDirective>,
    Option<WorldCommand>,
) {
    let collision = hook.filter(view);
    if collision == CollisionDirective::Ignore {
        return (collision, None, None);
    }

    let directive = hook.pre_solve(view);
    hook.observe(view);
    let maybe_command = hook.command(view);
    (collision, Some(directive), maybe_command)
}

fn check_capacity(current: usize, limit: usize, resource: &'static str) -> Result<(), StepError> {
    if current == limit {
        return Err(StepError::LimitExceeded { resource, limit });
    }
    Ok(())
}

#[cfg(test)]
pub(super) mod hooks {
    use super::*;

    struct RecordingHook {
        observed: Vec<[FixtureId; 2]>,
    }

    impl StepHook for RecordingHook {
        fn observe(&mut self, contact: ContactView<'_>) {
            self.observed.push(contact.fixtures());
        }
    }

    fn world_with_contact() -> (World, ContactSnapshot) {
        let mut world = World::new().expect("test world key should remain available");
        let body = world
            .create_body(&BodyDef::default())
            .expect("body should fit");
        let first = world
            .create_fixture(body, &test_fixture_definition())
            .expect("fixture should fit");
        let second = world
            .create_fixture(body, &test_fixture_definition())
            .expect("fixture should fit");
        (world, ContactSnapshot::new(first, second))
    }

    #[test]
    fn reports_preserve_occurrence_order_and_multiplicity() {
        // Arrange
        let (mut world, first) = world_with_contact();
        let body = world
            .create_body(&BodyDef::default())
            .expect("body should fit");
        let third = world
            .create_fixture(body, &test_fixture_definition())
            .expect("fixture should fit");
        let second = ContactSnapshot::new(first.fixtures()[1], third);
        let contacts = [first, second, first];
        let mut hook = RecordingHook {
            observed: Vec::new(),
        };

        // Act
        let report = world
            .step(&contacts, &mut hook, StepLimits::default())
            .expect("bounded contacts should step");

        // Assert
        assert_eq!(
            report
                .events()
                .iter()
                .map(|event| event.contact())
                .collect::<Vec<_>>(),
            contacts
        );
        assert_eq!(hook.observed, contacts.map(ContactSnapshot::fixtures));
    }

    #[test]
    fn filtering_returns_a_narrow_directive_and_skips_later_hooks() {
        struct FilteringHook {
            pre_solve_calls: usize,
            observe_calls: usize,
        }

        impl StepHook for FilteringHook {
            fn filter(&mut self, _contact: ContactView<'_>) -> CollisionDirective {
                CollisionDirective::Ignore
            }

            fn pre_solve(&mut self, _contact: ContactView<'_>) -> PreSolveDirective {
                self.pre_solve_calls += 1;
                PreSolveDirective::Enable
            }

            fn observe(&mut self, _contact: ContactView<'_>) {
                self.observe_calls += 1;
            }
        }

        // Arrange
        let (mut world, contact) = world_with_contact();
        let mut hook = FilteringHook {
            pre_solve_calls: 0,
            observe_calls: 0,
        };

        // Act
        let report = world
            .step(&[contact], &mut hook, StepLimits::default())
            .expect("contact should be valid");

        // Assert
        assert_eq!(report.events()[0].collision(), CollisionDirective::Ignore);
        assert_eq!(report.events()[0].maybe_pre_solve(), None);
        assert_eq!(hook.pre_solve_calls, 0);
        assert_eq!(hook.observe_calls, 0);
    }

    #[test]
    fn event_limit_fails_without_mutating_world_objects() {
        // Arrange
        let (mut world, contact) = world_with_contact();
        let fixtures = contact.fixtures();
        let mut hook = RecordingHook {
            observed: Vec::new(),
        };
        let limits = StepLimits::new(1, 0).expect("limits are below hard maxima");

        // Act
        let result = world.step(&[contact, contact], &mut hook, limits);

        // Assert
        assert_eq!(
            result,
            Err(StepError::LimitExceeded {
                resource: "event",
                limit: 1,
            })
        );
        assert!(world.contains_fixture(fixtures[0]));
        assert!(world.contains_fixture(fixtures[1]));
    }
}

#[cfg(test)]
mod commands {
    use super::*;

    struct CommandHook {
        commands: std::collections::VecDeque<WorldCommand>,
    }

    impl StepHook for CommandHook {
        fn command(&mut self, _contact: ContactView<'_>) -> Option<WorldCommand> {
            self.commands.pop_front()
        }
    }

    fn world_with_contact() -> (World, ContactSnapshot) {
        let mut world = World::new().expect("test world key should remain available");
        let body = world
            .create_body(&BodyDef::default())
            .expect("body should fit");
        let first = world
            .create_fixture(body, &test_fixture_definition())
            .expect("fixture should fit");
        let second = world
            .create_fixture(body, &test_fixture_definition())
            .expect("fixture should fit");
        (world, ContactSnapshot::new(first, second))
    }

    #[test]
    fn commands_apply_after_unlock_in_request_order() {
        // Arrange
        let (mut world, contact) = world_with_contact();
        let first = world
            .create_body(&BodyDef::default())
            .expect("body should fit");
        let second = world
            .create_body(&BodyDef::default())
            .expect("body should fit");
        let mut hook = CommandHook {
            commands: [
                WorldCommand::DestroyBody(first),
                WorldCommand::DestroyBody(second),
            ]
            .into(),
        };

        // Act
        let report = world
            .step(&[contact, contact], &mut hook, StepLimits::default())
            .expect("commands should be bounded");

        // Assert
        assert_eq!(report.command_applications().len(), 2);
        assert!(report.command_applications()[0].result().is_ok());
        assert!(report.command_applications()[1].result().is_ok());
        assert_eq!(
            report
                .destructions()
                .iter()
                .map(DestructionRecord::destroyed)
                .collect::<Vec<_>>(),
            vec![
                crate::DestroyedId::Body(first),
                crate::DestroyedId::Body(second)
            ]
        );
    }

    #[test]
    fn stale_command_does_not_stop_later_commands() {
        // Arrange
        let (mut world, contact) = world_with_contact();
        let invalidated = world
            .create_body(&BodyDef::default())
            .expect("body should fit");
        let survivor = world
            .create_body(&BodyDef::default())
            .expect("body should fit");
        let mut hook = CommandHook {
            commands: [
                WorldCommand::DestroyBody(invalidated),
                WorldCommand::DestroyBody(invalidated),
                WorldCommand::DestroyBody(survivor),
            ]
            .into(),
        };

        // Act
        let report = world
            .step(
                &[contact, contact, contact],
                &mut hook,
                StepLimits::default(),
            )
            .expect("stale command is a per-command result");

        // Assert
        assert!(report.command_applications()[0].result().is_ok());
        assert_eq!(
            report.command_applications()[1].result(),
            Err(CommandError::InvalidHandle(HandleError::StaleOrDestroyed))
        );
        assert!(report.command_applications()[2].result().is_ok());
        assert!(!world.contains_body(survivor));
    }

    #[test]
    fn cross_world_and_reused_slot_commands_fail_at_application_time() {
        // Arrange
        let (mut world, contact) = world_with_contact();
        let stale = world
            .create_body(&BodyDef::default())
            .expect("body should fit");
        world.destroy_body(stale).expect("body should be live");
        let replacement = world
            .create_body(&BodyDef::default())
            .expect("reused slot should fit");
        let mut other = World::new().expect("test world key should remain available");
        let foreign = other
            .create_body(&BodyDef::default())
            .expect("body should fit");
        let mut hook = CommandHook {
            commands: [
                WorldCommand::DestroyBody(stale),
                WorldCommand::DestroyBody(foreign),
            ]
            .into(),
        };

        // Act
        let report = world
            .step(&[contact, contact], &mut hook, StepLimits::default())
            .expect("invalid commands are recoverable results");

        // Assert
        assert_eq!(
            report.command_applications()[0].result(),
            Err(CommandError::InvalidHandle(HandleError::StaleOrDestroyed))
        );
        assert_eq!(
            report.command_applications()[1].result(),
            Err(CommandError::InvalidHandle(HandleError::WrongWorld))
        );
        assert!(world.contains_body(replacement));
    }

    #[test]
    fn command_overflow_discards_all_queued_commands() {
        // Arrange
        let (mut world, contact) = world_with_contact();
        let body = world
            .create_body(&BodyDef::default())
            .expect("body should fit");
        let mut hook = CommandHook {
            commands: [
                WorldCommand::DestroyBody(body),
                WorldCommand::DestroyBody(body),
            ]
            .into(),
        };
        let limits = StepLimits::new(2, 1).expect("limits are below hard maxima");

        // Act
        let result = world.step(&[contact, contact], &mut hook, limits);

        // Assert
        assert_eq!(
            result,
            Err(StepError::LimitExceeded {
                resource: "command",
                limit: 1,
            })
        );
        assert!(world.contains_body(body));
    }
}

#[cfg(test)]
mod panic {
    use std::collections::VecDeque;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use super::*;

    struct PanickingHook {
        calls: usize,
        commands: VecDeque<WorldCommand>,
    }

    impl StepHook for PanickingHook {
        fn observe(&mut self, _contact: ContactView<'_>) {
            self.calls += 1;
            assert!(self.calls < 2, "intentional hook panic");
        }

        fn command(&mut self, _contact: ContactView<'_>) -> Option<WorldCommand> {
            self.commands.pop_front()
        }
    }

    #[test]
    fn hook_panic_restores_lock_discards_commands_and_poisons_world() {
        // Arrange
        let mut world = World::new().expect("test world key should remain available");
        let contact_body = world
            .create_body(&BodyDef::default())
            .expect("body should fit");
        let first = world
            .create_fixture(contact_body, &test_fixture_definition())
            .expect("fixture should fit");
        let second = world
            .create_fixture(contact_body, &test_fixture_definition())
            .expect("fixture should fit");
        let contact = ContactSnapshot::new(first, second);
        let command_body = world
            .create_body(&BodyDef::default())
            .expect("body should fit");
        let mut hook = PanickingHook {
            calls: 0,
            commands: [WorldCommand::DestroyBody(command_body)].into(),
        };

        // Act
        let panic = catch_unwind(AssertUnwindSafe(|| {
            let _report = world.step(&[contact, contact], &mut hook, StepLimits::default());
        }));

        // Assert
        assert!(panic.is_err());
        assert!(!world.is_locked());
        assert!(world.is_poisoned());
        assert!(world.contains_body(command_body));
        assert_eq!(
            world.destroy_body(command_body),
            Err(HandleError::WorldPoisoned)
        );
        assert_eq!(
            world.create_body(&BodyDef::default()),
            Err(crate::ArenaInsertError::WorldPoisoned)
        );
        assert_eq!(
            world.step(&[], &mut hook, StepLimits::default()),
            Err(StepError::Poisoned)
        );
    }
}
