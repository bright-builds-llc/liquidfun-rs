//! Automatic checked rigid-world stepping, restricted hooks, and owned reports.

use std::error::Error;
use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
    AggregateMassError, BodyId, DestructionRecord, FixtureId, HandleError, ParticleBodyContact,
    ParticleBodyContactEffect, ParticleContact, ParticleContactEffect, ParticleId, World,
};

use super::fixture::FixtureDestructionError;

use super::config::{StepCompletion, StepConfiguration};
use super::contact::{ContactPointSnapshot, ContactTransition, ManagedContactSnapshot};
use super::contact_solver::{ContactSolve, ContactSolveFailure};
use super::continuous::{ContinuousStepKey, ContinuousStepKind};
use super::observation::{
    DiagnosticProfileParent, DiagnosticStepPhase, DiagnosticStepProfile, DiagnosticStepProfiler,
};

mod continuous;
pub use continuous::ContinuousProgress;

const MAX_STEP_EVENTS: usize = 4_096;
const MAX_STEP_COMMANDS: usize = 1_024;
const MAX_CONTINUOUS_WORK: usize = 1_024;

mod contact_view;
mod execution;
mod hook;
mod report;
mod state;
#[cfg(test)]
mod tests;

pub(super) use contact_view::FixturePairSnapshot;
pub use contact_view::{
    CollisionDirective, CollisionFilterEvent, ContactControlError, ContactView, FixturePairView,
    FixtureParticleView, ParticlePairContactView, PreSolveDirective, PreSolveView,
};
use execution::check_capacity;
pub(super) use execution::solver_step_error;
pub(super) use hook::ContactHookRun;
pub use hook::{CollisionDecisionHook, NoDecisionHook, StepHook};
pub use report::{
    CommandApplication, CommandError, ContactEvent, LifecycleEvent, StepError, StepLifecycleEvent,
    StepPhase, StepReport, WorldCommand,
};
pub use state::StepLimits;
pub(super) use state::StepState;
