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

    /// Returns typed body identities in oriented manager order.
    #[must_use]
    pub const fn bodies(self) -> [BodyId; 2] {
        self.contact.bodies()
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

    /// Returns the configured surface tangent speed.
    #[must_use]
    pub const fn tangent_speed(self) -> f32 {
        self.contact.tangent_speed()
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

#[derive(Debug, Clone, Copy)]
pub(super) struct FixturePairSnapshot {
    fixtures: [FixtureId; 2],
    bodies: [BodyId; 2],
    child_indices: [crate::collision::ChildIndex; 2],
}

/// Owned evidence for one source-timed collision-filter decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollisionFilterEvent {
    fixtures: [FixtureId; 2],
    bodies: [BodyId; 2],
    child_indices: [crate::collision::ChildIndex; 2],
    decision: CollisionDirective,
}

impl CollisionFilterEvent {
    const fn new(pair: FixturePairSnapshot, decision: CollisionDirective) -> Self {
        Self {
            fixtures: pair.fixtures,
            bodies: pair.bodies,
            child_indices: pair.child_indices,
            decision,
        }
    }

    /// Returns fixture identities in canonical pair order.
    #[must_use]
    pub const fn fixtures(self) -> [FixtureId; 2] {
        self.fixtures
    }

    /// Returns body identities in canonical pair order.
    #[must_use]
    pub const fn bodies(self) -> [BodyId; 2] {
        self.bodies
    }

    /// Returns shape-child coordinates in canonical pair order.
    #[must_use]
    pub const fn child_indices(self) -> [crate::collision::ChildIndex; 2] {
        self.child_indices
    }

    /// Returns the exact decision made at the admission or refilter point.
    #[must_use]
    pub const fn decision(self) -> CollisionDirective {
        self.decision
    }
}

impl FixturePairSnapshot {
    pub(super) const fn new(
        fixtures: [FixtureId; 2],
        bodies: [BodyId; 2],
        child_indices: [crate::collision::ChildIndex; 2],
    ) -> Self {
        Self {
            fixtures,
            bodies,
            child_indices,
        }
    }
}

/// Borrow-scoped semantic fixture pair evaluated before contact admission.
///
/// The view deliberately contains no reusable contact identity because a
/// rejected admission does not create a contact.
///
/// ```compile_fail
/// use liquidfun::ContactId;
/// ```
#[derive(Clone, Copy)]
pub struct FixturePairView<'hook> {
    pair: &'hook FixturePairSnapshot,
}

impl<'hook> FixturePairView<'hook> {
    pub(super) const fn new(pair: &'hook FixturePairSnapshot) -> Self {
        Self { pair }
    }

    /// Returns fixture identities in canonical pair order.
    #[must_use]
    pub const fn fixtures(self) -> [FixtureId; 2] {
        self.pair.fixtures
    }

    /// Returns body identities in canonical pair order.
    #[must_use]
    pub const fn bodies(self) -> [BodyId; 2] {
        self.pair.bodies
    }

    /// Returns shape-child coordinates in canonical pair order.
    #[must_use]
    pub const fn child_indices(self) -> [crate::collision::ChildIndex; 2] {
        self.pair.child_indices
    }
}

impl fmt::Debug for FixturePairView<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FixturePairView")
            .field("fixtures", &self.fixtures())
            .field("bodies", &self.bodies())
            .field("child_indices", &self.child_indices())
            .finish()
    }
}

/// A validated contact-control value was rejected before hook application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContactControlError {
    /// The value was NaN or infinite.
    NonFinite,
    /// Friction or restitution was negative.
    Negative,
}

impl fmt::Display for ContactControlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite => formatter.write_str("contact control must be finite"),
            Self::Negative => formatter.write_str("contact material control must be non-negative"),
        }
    }
}

impl Error for ContactControlError {}

/// Opaque validated material controls carried by [`PreSolveDirective`].
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[allow(
    clippy::struct_field_names,
    reason = "maybe_ prefixes make the three optional source controls explicit"
)]
pub struct PreSolveControls {
    maybe_friction: Option<f32>,
    maybe_restitution: Option<f32>,
    maybe_tangent_speed: Option<f32>,
}

impl PreSolveControls {
    const EMPTY: Self = Self {
        maybe_friction: None,
        maybe_restitution: None,
        maybe_tangent_speed: None,
    };
}

/// Narrow pre-solve result returned by a hook.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[non_exhaustive]
pub enum PreSolveDirective {
    /// Keep the occurrence enabled.
    #[default]
    Enable,
    /// Disable the occurrence for this step.
    Disable,
    /// Keep the occurrence enabled and apply validated source-supported controls.
    Configure {
        /// Whether this occurrence remains enabled for the current update.
        enabled: bool,
        /// Validated source-supported material controls.
        controls: PreSolveControls,
    },
}

impl PreSolveDirective {
    /// Returns this directive with a finite non-negative friction override.
    ///
    /// # Errors
    ///
    /// Rejects non-finite or negative values.
    pub fn with_friction(self, friction: f32) -> Result<Self, ContactControlError> {
        validate_material_control(friction)?;
        let (enabled, mut controls) = self.parts();
        controls.maybe_friction = Some(friction);
        Ok(Self::Configure { enabled, controls })
    }

    /// Returns this directive with a finite non-negative restitution override.
    ///
    /// # Errors
    ///
    /// Rejects non-finite or negative values.
    pub fn with_restitution(self, restitution: f32) -> Result<Self, ContactControlError> {
        validate_material_control(restitution)?;
        let (enabled, mut controls) = self.parts();
        controls.maybe_restitution = Some(restitution);
        Ok(Self::Configure { enabled, controls })
    }

    /// Returns this directive with a finite tangent-speed override.
    ///
    /// # Errors
    ///
    /// Rejects NaN and infinity.
    pub fn with_tangent_speed(self, tangent_speed: f32) -> Result<Self, ContactControlError> {
        if !tangent_speed.is_finite() {
            return Err(ContactControlError::NonFinite);
        }
        let (enabled, mut controls) = self.parts();
        controls.maybe_tangent_speed = Some(tangent_speed);
        Ok(Self::Configure { enabled, controls })
    }

    const fn parts(self) -> (bool, PreSolveControls) {
        match self {
            Self::Enable => (true, PreSolveControls::EMPTY),
            Self::Disable => (false, PreSolveControls::EMPTY),
            Self::Configure { enabled, controls } => (enabled, controls),
        }
    }

    pub(super) const fn enabled(self) -> bool {
        self.parts().0
    }

    pub(super) const fn material_controls(self) -> (Option<f32>, Option<f32>, Option<f32>) {
        let controls = self.parts().1;
        (
            controls.maybe_friction,
            controls.maybe_restitution,
            controls.maybe_tangent_speed,
        )
    }
}

fn validate_material_control(value: f32) -> Result<(), ContactControlError> {
    if !value.is_finite() {
        return Err(ContactControlError::NonFinite);
    }
    if value < 0.0 {
        return Err(ContactControlError::Negative);
    }
    Ok(())
}

/// Borrow-scoped semantic state available at the pinned pre-solve point.
#[derive(Clone, Copy)]
pub struct PreSolveView<'hook> {
    current: &'hook ManagedContactSnapshot,
    current_manifold: &'hook crate::collision::Manifold,
    maybe_previous_manifold: Option<&'hook crate::collision::Manifold>,
}

impl<'hook> PreSolveView<'hook> {
    pub(super) const fn new(
        current: &'hook ManagedContactSnapshot,
        maybe_previous_manifold: Option<&'hook crate::collision::Manifold>,
    ) -> Self {
        Self {
            current,
            current_manifold: current
                .maybe_manifold()
                .expect("pre-solve construction requires a touching solid manifold"),
            maybe_previous_manifold,
        }
    }

    /// Returns fixture identities in oriented manager order.
    #[must_use]
    pub const fn fixtures(self) -> [FixtureId; 2] {
        self.current.fixtures()
    }

    /// Returns body identities in oriented manager order.
    #[must_use]
    pub const fn bodies(self) -> [BodyId; 2] {
        self.current.bodies()
    }

    /// Returns child indices in oriented manager order.
    #[must_use]
    pub const fn child_indices(self) -> [crate::collision::ChildIndex; 2] {
        self.current.child_indices()
    }

    /// Returns the current touching manifold.
    #[must_use]
    pub fn current_manifold(self) -> &'hook crate::collision::Manifold {
        self.current_manifold
    }

    /// Returns the owned semantic manifold captured before this update.
    #[must_use]
    pub const fn maybe_previous_manifold(self) -> Option<&'hook crate::collision::Manifold> {
        self.maybe_previous_manifold
    }

    /// Returns the current semantic contact through the legacy read-only view.
    #[must_use]
    pub const fn contact(self) -> ContactView<'hook> {
        ContactView {
            contact: self.current,
        }
    }
}

impl fmt::Debug for PreSolveView<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreSolveView")
            .field("fixtures", &self.fixtures())
            .field("bodies", &self.bodies())
            .field("child_indices", &self.child_indices())
            .finish_non_exhaustive()
    }
}

/// Restricted synchronous step hooks.
///
/// Hooks receive only borrow-scoped read-only contact views and return narrow
/// decisions. They do not receive mutable world access:
///
/// ```compile_fail
/// use liquidfun::{CollisionDecisionHook, PreSolveDirective, PreSolveView, World};
///
/// struct MutatingHook;
///
/// impl CollisionDecisionHook for MutatingHook {
///     fn pre_solve(
///         &mut self,
///         _world: &mut World,
///         _contact: PreSolveView<'_>,
///     ) -> PreSolveDirective {
///         PreSolveDirective::Enable
///     }
/// }
/// ```
pub trait CollisionDecisionHook {
    /// Decides whether a broad-phase pair may create or retain a contact.
    fn should_collide(&mut self, _pair: FixturePairView<'_>) -> CollisionDirective {
        CollisionDirective::Collide
    }

    /// Decides source-supported controls immediately after one solid update.
    fn pre_solve(&mut self, _contact: PreSolveView<'_>) -> PreSolveDirective {
        PreSolveDirective::Enable
    }

    /// Observes one non-filtered occurrence without mutable world access.
    fn observe(&mut self, _contact: ContactView<'_>) {}

    /// Optionally requests one owned mutation after observing an occurrence.
    fn command(&mut self, _contact: ContactView<'_>) -> Option<WorldCommand> {
        None
    }
}

/// Explicit no-op decision maker used to replace or unregister prior behavior.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoDecisionHook;

impl CollisionDecisionHook for NoDecisionHook {}

/// Legacy observation and deferred-command hook contract.
///
/// Implementations receive the new source-timed behavior through the blanket
/// [`CollisionDecisionHook`] adapter. New filtering code should implement
/// `CollisionDecisionHook` directly so it can inspect [`FixturePairView`].
pub trait StepHook {
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

impl<H: StepHook + ?Sized> CollisionDecisionHook for H {
    fn pre_solve(&mut self, contact: PreSolveView<'_>) -> PreSolveDirective {
        StepHook::pre_solve(self, contact.contact())
    }

    fn observe(&mut self, contact: ContactView<'_>) {
        StepHook::observe(self, contact);
    }

    fn command(&mut self, contact: ContactView<'_>) -> Option<WorldCommand> {
        StepHook::command(self, contact)
    }
}

pub(super) struct ContactHookRun<'hook, H> {
    hook: &'hook mut H,
    limits: StepLimits,
    lifecycle: Vec<LifecycleEvent>,
    commands: Vec<WorldCommand>,
}

impl<'hook, H: CollisionDecisionHook> ContactHookRun<'hook, H> {
    pub(super) fn new(hook: &'hook mut H, limits: StepLimits) -> Self {
        Self {
            hook,
            limits,
            lifecycle: Vec::new(),
            commands: Vec::new(),
        }
    }

    pub(super) fn should_collide(&mut self, pair: &FixturePairSnapshot) -> Result<bool, StepError> {
        let decision = self.hook.should_collide(FixturePairView::new(pair));
        self.push_lifecycle(LifecycleEvent::Filter(CollisionFilterEvent::new(
            *pair, decision,
        )))?;
        Ok(decision == CollisionDirective::Collide)
    }

    pub(super) fn contact_updated(
        &mut self,
        current: &ManagedContactSnapshot,
        maybe_previous_manifold: Option<&crate::collision::Manifold>,
        allow_pre_solve: bool,
    ) -> Result<PreSolveDirective, StepError> {
        let contact = ContactView { contact: current };
        let directive = if allow_pre_solve {
            self.hook
                .pre_solve(PreSolveView::new(current, maybe_previous_manifold))
        } else {
            PreSolveDirective::Enable
        };
        self.hook.observe(contact);
        if let Some(command) = self.hook.command(contact) {
            check_capacity(self.commands.len(), self.limits.max_commands, "command")?;
            self.commands.push(command);
        }
        self.push_lifecycle(LifecycleEvent::Hook(ContactEvent {
            contact: current.clone(),
            collision: CollisionDirective::Collide,
            maybe_pre_solve: allow_pre_solve.then_some(directive),
        }))?;
        Ok(directive)
    }

    pub(super) fn record_contact(
        &mut self,
        transition: ContactTransition,
    ) -> Result<(), StepError> {
        self.push_lifecycle(LifecycleEvent::Contact(transition))
    }

    pub(super) fn record_contact_destruction(
        &mut self,
        transition: ContactTransition,
    ) -> Result<(), StepError> {
        self.push_lifecycle(LifecycleEvent::ContactDestruction(transition))
    }

    pub(super) fn record_particle_destruction(
        &mut self,
        record: DestructionRecord,
    ) -> Result<(), StepError> {
        self.push_lifecycle(LifecycleEvent::ParticleDestruction(record))
    }

    pub(super) fn record_discrete_solve(&mut self, solve: ContactSolve) -> Result<(), StepError> {
        self.push_lifecycle(LifecycleEvent::Solve(solve))
    }

    pub(super) fn record_continuous_solve(&mut self, solve: ContactSolve) -> Result<(), StepError> {
        self.push_lifecycle(LifecycleEvent::ContinuousSolve(solve))
    }

    pub(super) fn ensure_lifecycle_capacity(&self, additional: usize) -> Result<(), StepError> {
        let required =
            self.lifecycle
                .len()
                .checked_add(additional)
                .ok_or(StepError::LimitExceeded {
                    resource: "event",
                    limit: self.limits.max_events,
                })?;
        if required > self.limits.max_events {
            return Err(StepError::LimitExceeded {
                resource: "event",
                limit: self.limits.max_events,
            });
        }
        Ok(())
    }

    fn push_lifecycle(&mut self, event: LifecycleEvent) -> Result<(), StepError> {
        check_capacity(self.lifecycle.len(), self.limits.max_events, "event")?;
        self.lifecycle.push(event);
        Ok(())
    }

    fn finish(self) -> (Vec<LifecycleEvent>, Vec<WorldCommand>) {
        (self.lifecycle, self.commands)
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
    /// A fixture pair was accepted or rejected at its source filter point.
    Filter(CollisionFilterEvent),
    /// A private manager contact began, persisted, or ended touching.
    Contact(ContactTransition),
    /// A touching contact ended because its private manager occurrence was destroyed.
    ContactDestruction(ContactTransition),
    /// A restricted hook observed one owned manager occurrence.
    Hook(ContactEvent),
    /// One private manager occurrence completed bounded solving.
    Solve(ContactSolve),
    /// One private manager occurrence completed a transient TOI solve.
    ContinuousSolve(ContactSolve),
    /// An implicitly destroyed joint emitted source-compatible goodbye evidence.
    JointGoodbye(DestructionRecord),
    /// An implicitly destroyed fixture emitted source-compatible goodbye evidence.
    FixtureGoodbye(DestructionRecord),
    /// A requested particle listener occurrence was journaled before invalidation.
    ParticleDestruction(DestructionRecord),
    /// One requested mutation completed after unlock.
    Command(CommandApplication),
    /// A world object was invalidated after dependent contact evidence.
    Destruction(DestructionRecord),
}

/// The authoritative owned lifecycle vocabulary shared by step and mutation reports.
pub type LifecycleEvent = StepLifecycleEvent;

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
    /// Particle lifetime advancement could not represent the requested step.
    ParticleLifetime(crate::ParticleLifetimeError),
    /// Authoritative particle storage violated an internal lifecycle invariant.
    ParticleLifecycleInvariant,
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
            Self::ParticleLifetime(error) => write!(formatter, "particle lifetime failed: {error}"),
            Self::ParticleLifecycleInvariant => {
                formatter.write_str("particle lifecycle invariant was violated")
            }
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

struct StepLimitBackup {
    bodies: crate::arena::Arena<super::object::Body, BodyId>,
    fixtures: crate::arena::Arena<super::object::Fixture, FixtureId>,
    joints: crate::arena::Arena<super::joint::JointRecord, crate::JointId>,
    particle_systems: crate::arena::Arena<super::object::ParticleSystem, crate::ParticleSystemId>,
    broad_phase: crate::collision::BroadPhase<super::proxy::FixtureProxy>,
    contact_manager: super::contact_manager::ContactManager,
    continuous_step_state: super::continuous::ContinuousStepState,
    configuration: super::config::WorldConfiguration,
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
    fn backup_step_limit_state(&self) -> StepLimitBackup {
        StepLimitBackup {
            bodies: self.bodies.clone(),
            fixtures: self.fixtures.clone(),
            joints: self.joints.clone(),
            particle_systems: self.particle_systems.clone(),
            broad_phase: self.broad_phase.clone(),
            contact_manager: self.contact_manager.clone(),
            continuous_step_state: self.continuous_step_state,
            configuration: self.configuration,
        }
    }

    fn restore_step_limit_state(&mut self, backup: StepLimitBackup) {
        self.bodies = backup.bodies;
        self.fixtures = backup.fixtures;
        self.joints = backup.joints;
        self.particle_systems = backup.particle_systems;
        self.broad_phase = backup.broad_phase;
        self.contact_manager = backup.contact_manager;
        self.continuous_step_state = backup.continuous_step_state;
        self.configuration = backup.configuration;
    }

    /// Runs one automatic bounded rigid-world lifecycle and solve.
    ///
    /// Checked configuration is supplied by [`StepConfiguration`]. Discrete
    /// islands are staged and committed transactionally in deterministic source
    /// order. Continuous work remains private and resumable: sub-stepping uses
    /// [`StepCompletion::ContinuousPending`], while budget exhaustion returns
    /// [`StepError::ContinuousWorkLimitExceeded`] with [`ContinuousProgress`].
    /// Ordinary event and command limit failures restore the exact pre-call
    /// rigid-world state so the caller may retry with larger limits.
    /// Accumulated forces clear only after a successful call when automatic
    /// clearing is enabled.
    ///
    /// # Errors
    ///
    /// Returns an error for poisoned or nested stepping, exhausted limits,
    /// unsupported topology, or non-finite solver state.
    #[allow(
        clippy::too_many_lines,
        reason = "the locked source-ordered step lifecycle is kept visible as one transaction"
    )]
    pub fn step<H: CollisionDecisionHook>(
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
        let step_limit_backup = self.backup_step_limit_state();
        let step_kind = if continuous_enabled {
            self.continuous_step_state
                .begin_step(continuous_key, &mut self.contact_manager)
        } else {
            self.continuous_step_state.invalidate();
            ContinuousStepKind::Fresh
        };
        let mut hook_run = ContactHookRun::new(hook, limits);
        let mut contact_transitions = self.contact_manager.drain_transitions();
        let locked_result = (|| -> Result<StepCompletion, StepError> {
            let _lock = step_lock;
            let contact_lifecycle_result = (|| {
                for transition in contact_transitions.iter().cloned() {
                    hook_run.record_contact_destruction(transition)?;
                }
                phases.push(StepPhase::FindPairs);
                let hook_result = catch_unwind(AssertUnwindSafe(|| {
                    self.find_pairs_with_hook(&mut hook_run)?;
                    phases.push(StepPhase::UpdateContacts);
                    self.update_contacts_for_step(&mut hook_run)
                }));
                match hook_result {
                    Ok(result) => result,
                    Err(payload) => {
                        self.step_state.poison();
                        resume_unwind(payload);
                    }
                }
            })();
            contact_lifecycle_result?;
            contact_transitions.extend(self.contact_manager.drain_transitions());
            self.run_particle_lifecycle_step(configuration.time_step(), &mut hook_run)?;
            if step_kind == ContinuousStepKind::Fresh && configuration.time_step() > 0.0 {
                self.preflight_contact_solver()
                    .map_err(|error| solver_step_error(error, &contact_transitions))?;
            }

            phases.push(StepPhase::Hook);
            if step_kind == ContinuousStepKind::Fresh && configuration.time_step() > 0.0 {
                phases.push(StepPhase::Solve);
                self.solve_contact_constraints(
                    configuration,
                    timing,
                    limits.maybe_failure_injection,
                    &contact_transitions,
                    &mut hook_run,
                )?;
            }

            let completion = if continuous_enabled {
                let hook_result = catch_unwind(AssertUnwindSafe(|| {
                    self.run_continuous_stage(
                        configuration,
                        continuous_key,
                        limits.max_continuous_work,
                        &contact_transitions,
                        &mut hook_run,
                    )
                }));
                let continuous = match hook_result {
                    Ok(result) => result?,
                    Err(payload) => {
                        self.step_state.poison();
                        resume_unwind(payload);
                    }
                };
                drop(continuous.contact_solves);
                drop(self.contact_manager.drain_transitions());
                continuous.completion
            } else {
                StepCompletion::Complete
            };
            Ok(completion)
        })();
        let completion = match locked_result {
            Ok(completion) => completion,
            Err(error @ StepError::LimitExceeded { .. }) => {
                self.restore_step_limit_state(step_limit_backup);
                return Err(error);
            }
            Err(error) => return Err(error),
        };
        let (mut lifecycle, commands) = hook_run.finish();
        phases.push(StepPhase::Unlock);
        if !commands.is_empty() {
            phases.push(StepPhase::ApplyCommands);
        }
        lifecycle.extend(self.apply_commands(commands));

        let contact_transitions = lifecycle
            .iter()
            .filter_map(|event| match event {
                LifecycleEvent::Contact(transition)
                | LifecycleEvent::ContactDestruction(transition) => Some(transition.clone()),
                _ => None,
            })
            .collect();
        let events = lifecycle
            .iter()
            .filter_map(|event| match event {
                LifecycleEvent::Hook(event) => Some(event.clone()),
                _ => None,
            })
            .collect();
        let contact_solves = lifecycle
            .iter()
            .filter_map(|event| match event {
                LifecycleEvent::Solve(solve) => Some(solve.clone()),
                _ => None,
            })
            .collect();
        let continuous_contact_solves = lifecycle
            .iter()
            .filter_map(|event| match event {
                LifecycleEvent::ContinuousSolve(solve) => Some(solve.clone()),
                _ => None,
            })
            .collect();
        let destructions = lifecycle
            .iter()
            .filter_map(|event| match event {
                LifecycleEvent::JointGoodbye(record)
                | LifecycleEvent::FixtureGoodbye(record)
                | LifecycleEvent::ParticleDestruction(record)
                | LifecycleEvent::Destruction(record) => Some(record.clone()),
                _ => None,
            })
            .collect();
        let command_applications = lifecycle
            .iter()
            .filter_map(|event| match event {
                LifecycleEvent::Command(application) => Some(application.clone()),
                _ => None,
            })
            .collect();

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

    fn find_pairs_with_hook<H: CollisionDecisionHook>(
        &mut self,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<(), StepError> {
        self.find_new_contacts_with_hook(hook_run)
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

    fn update_contacts_for_step<H: CollisionDecisionHook>(
        &mut self,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<(), StepError> {
        self.update_contacts_with_hook(hook_run)
    }

    fn solve_contact_constraints<H: CollisionDecisionHook>(
        &mut self,
        configuration: StepConfiguration,
        timing: super::config::StepTiming,
        maybe_failure_injection: Option<super::island::SolveFailureInjection>,
        contact_transitions: &[ContactTransition],
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<Vec<ContactSolve>, StepError> {
        self.solve_contacts(
            configuration,
            timing,
            maybe_failure_injection,
            contact_transitions,
            hook_run,
        )
    }

    fn apply_commands(&mut self, commands: Vec<WorldCommand>) -> Vec<StepLifecycleEvent> {
        let mut lifecycle = Vec::new();
        for command in commands {
            let result = self.apply_command(command);
            let (owned_result, mutation_lifecycle) = match result {
                Ok(report) => {
                    let mutation_lifecycle = report.lifecycle().to_vec();
                    (Ok(report.into_value()), mutation_lifecycle)
                }
                Err(error) => (Err(error), Vec::new()),
            };
            lifecycle.extend(mutation_lifecycle);
            let application = CommandApplication {
                command,
                result: owned_result,
            };
            lifecycle.push(StepLifecycleEvent::Command(application.clone()));
        }
        lifecycle
    }

    fn apply_command(
        &mut self,
        command: WorldCommand,
    ) -> Result<crate::DestructionReport, CommandError> {
        if self.step_state.is_locked() {
            return Err(CommandError::Locked);
        }
        match command {
            WorldCommand::DestroyBody(body) => {
                self.destroy_body(body).map_err(CommandError::InvalidHandle)
            }
            WorldCommand::DestroyFixture(fixture) => {
                self.destroy_fixture(fixture).map_err(|error| match error {
                    FixtureDestructionError::InvalidHandle(error) => {
                        CommandError::InvalidHandle(error)
                    }
                    FixtureDestructionError::InvalidAggregateMass(error) => {
                        CommandError::InvalidAggregateMass(error)
                    }
                })
            }
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

pub(super) fn solver_step_error(
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
