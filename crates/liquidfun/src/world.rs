//! World ownership and minimal non-solver object storage.

mod body;
mod config;
mod contact;
mod contact_manager;
mod contact_solver;
mod continuous;
#[cfg(feature = "differential-internals")]
mod diagnostics;
mod fixture;
mod island;
mod joint;
pub(crate) mod object;
mod origin;
mod particle_coupling;
mod particle_lifecycle;
pub(crate) mod particle_object;
mod proxy;
mod query;
mod step;

pub use body::{
    AggregateMassError, BodyActivationError, BodyControlError, BodyDef, BodyDefError, BodyMassData,
    BodyMassDataError, BodyMassMutationError, BodyMassResetError, BodySnapshot, BodyTransformError,
    BodyType, BodyTypeChangeError, WakePolicy,
};
pub use config::{
    StepCompletion, StepConfiguration, StepConfigurationError, WorldConfigurationError,
};
pub use contact::{
    ContactPointSnapshot, ContactTransition, ContactTransitionKind, ManagedContactSnapshot,
};
pub use contact_solver::ContactSolve;
#[cfg(feature = "differential-internals")]
pub use diagnostics::{
    BodyReconstruction, FixtureReconstruction, JointReconstruction, ReconstructionIndex,
    ReconstructionSupport, ReconstructionUnsupported, WorldDiagnostics, WorldReconstruction,
    WorldReconstructionError, WorldReconstructionLimits,
};
pub use fixture::{
    FixtureBoundsError, FixtureDef, FixtureDefError, FixtureDestructionError, FixtureMutationError,
    FixtureSnapshot, WorldFixtureSnapshot,
};
pub use joint::{JointCreationError, JointMutationError, JointQueryError};
pub use object::{
    CreateObjectError, DestroyedId, DestructionCause, DestructionRecord, DestructionReport,
    MutationReport, ObjectSnapshot, World,
};
pub use origin::OriginShiftError;
pub use query::{
    FixtureQueryOccurrence, QueryDirective, RayCastDirective, RayCastFraction,
    RayCastFractionError, WorldRayCastError, WorldRayHit,
};
pub use step::{
    CollisionDecisionHook, CollisionDirective, CollisionFilterEvent, CommandApplication,
    CommandError, ContactControlError, ContactEvent, ContactView, ContinuousProgress,
    FixturePairView, FixtureParticleView, LifecycleEvent, NoDecisionHook, PreSolveDirective,
    PreSolveView, StepError, StepHook, StepLifecycleEvent, StepLimits, StepPhase, StepReport,
    WorldCommand,
};
