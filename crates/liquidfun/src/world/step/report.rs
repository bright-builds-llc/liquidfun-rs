use super::{
    AggregateMassError, BodyId, CollisionDirective, CollisionFilterEvent, ContactSolve,
    ContactTransition, ContinuousProgress, DestructionRecord, Error, FixtureId, HandleError,
    ManagedContactSnapshot, ParticleBodyContactEffect, ParticleContactEffect, PreSolveDirective,
    StepCompletion, fmt,
};

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
    pub(super) command: WorldCommand,
    pub(super) result: Result<Vec<DestructionRecord>, CommandError>,
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
    pub(super) contact: ManagedContactSnapshot,
    pub(super) collision: CollisionDirective,
    pub(super) maybe_pre_solve: Option<PreSolveDirective>,
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
    /// One flag-gated fixture-particle contact began or ended in source order.
    ParticleBodyContact(ParticleBodyContactEffect),
    /// One flag-gated particle-pair contact began or ended in source order.
    ParticleContact(ParticleContactEffect),
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
    pub(super) completion: StepCompletion,
    pub(super) time_step_ratio: f32,
    pub(super) phases: Vec<StepPhase>,
    pub(super) events: Vec<ContactEvent>,
    pub(super) contact_transitions: Vec<ContactTransition>,
    pub(super) contact_solves: Vec<ContactSolve>,
    pub(super) continuous_contact_solves: Vec<ContactSolve>,
    pub(super) lifecycle: Vec<StepLifecycleEvent>,
    pub(super) destructions: Vec<DestructionRecord>,
    pub(super) command_applications: Vec<CommandApplication>,
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
    /// Particle proxy preparation rejected a spatial state.
    ParticleProxy(crate::ParticleProxyError),
    /// Particle-pair contact preparation rejected inconsistent semantic input.
    ParticleContact(crate::ParticleContactError),
    /// Phase 9 rigid reaction produced an invalid body candidate.
    ParticleCoupling(crate::BodyControlError),
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
            Self::ParticleProxy(error) => {
                write!(formatter, "particle proxy preparation failed: {error:?}")
            }
            Self::ParticleContact(error) => {
                write!(formatter, "particle contact preparation failed: {error:?}")
            }
            Self::ParticleCoupling(error) => write!(formatter, "particle coupling failed: {error}"),
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
