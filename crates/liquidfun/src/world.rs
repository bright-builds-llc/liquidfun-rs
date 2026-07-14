//! World ownership and minimal non-solver object storage.

mod body;
mod config;
mod contact;
mod contact_manager;
mod contact_solver;
mod continuous;
mod fixture;
mod island;
mod joint;
pub(crate) mod object;
mod origin;
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
pub use fixture::{
    FixtureBoundsError, FixtureDef, FixtureDefError, FixtureDestructionError, FixtureMutationError,
    FixtureSnapshot, WorldFixtureSnapshot,
};
pub use joint::{JointCreationError, JointMutationError, JointQueryError};
pub use object::{
    CreateObjectError, DestroyedId, DestructionCause, DestructionRecord, ObjectSnapshot, World,
};
pub use origin::OriginShiftError;
pub use query::{
    FixtureQueryOccurrence, QueryDirective, RayCastDirective, RayCastFraction,
    RayCastFractionError, WorldRayCastError, WorldRayHit,
};
pub use step::{
    CollisionDecisionHook, CollisionDirective, CommandApplication, CommandError,
    ContactControlError, ContactEvent, ContactView, ContinuousProgress, FixturePairView,
    NoDecisionHook, PreSolveDirective, PreSolveView, StepError, StepHook, StepLifecycleEvent,
    StepLimits, StepPhase, StepReport, WorldCommand,
};
