//! Restricted step hooks and owned reporting for the no-solver architecture spike.

use std::cell::Cell;
use std::error::Error;
use std::fmt;

use crate::{DestructionRecord, FixtureId, HandleError, World};

const MAX_STEP_EVENTS: usize = 4_096;
const MAX_STEP_COMMANDS: usize = 1_024;

#[derive(Debug)]
pub(super) struct StepState {
    locked: Cell<bool>,
}

impl StepState {
    pub(super) const fn new() -> Self {
        Self {
            locked: Cell::new(false),
        }
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
#[derive(Debug, Default, PartialEq, Eq)]
pub struct StepReport {
    events: Vec<ContactEvent>,
    destructions: Vec<DestructionRecord>,
}

impl StepReport {
    /// Returns callback events in exact occurrence order, including duplicates.
    #[must_use]
    pub fn events(&self) -> &[ContactEvent] {
        &self.events
    }

    /// Returns owned destruction evidence in command-application order.
    #[must_use]
    pub fn destructions(&self) -> &[DestructionRecord] {
        &self.destructions
    }
}

/// A representative step-lifecycle failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StepError {
    /// One of the contact fixture identities is foreign, stale, or destroyed.
    InvalidContact(HandleError),
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
            Self::InvalidContact(error) => write!(formatter, "invalid contact fixture: {error}"),
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
        let _lock = StepLockGuard::acquire(&self.step_state)?;
        let mut events = Vec::with_capacity(contacts.len().min(limits.max_events));

        for snapshot in contacts {
            if events.len() == limits.max_events {
                return Err(StepError::LimitExceeded {
                    resource: "event",
                    limit: limits.max_events,
                });
            }
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
            let collision = hook.filter(view);
            let maybe_pre_solve = if collision == CollisionDirective::Collide {
                let directive = hook.pre_solve(view);
                hook.observe(view);
                Some(directive)
            } else {
                None
            };
            events.push(ContactEvent {
                contact: view.snapshot(),
                collision,
                maybe_pre_solve,
            });
        }

        Ok(StepReport {
            events,
            destructions: Vec::new(),
        })
    }
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
        let body = world.create_body().expect("body should fit");
        let first = world.create_fixture(body).expect("fixture should fit");
        let second = world.create_fixture(body).expect("fixture should fit");
        (world, ContactSnapshot::new(first, second))
    }

    #[test]
    fn reports_preserve_occurrence_order_and_multiplicity() {
        // Arrange
        let (mut world, first) = world_with_contact();
        let body = world.create_body().expect("body should fit");
        let third = world.create_fixture(body).expect("fixture should fit");
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
