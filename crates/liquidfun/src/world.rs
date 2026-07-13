//! World ownership and minimal non-solver object storage.

mod body;
mod config;
mod contact;
mod contact_manager;
mod contact_solver;
mod fixture;
pub(crate) mod object;
mod proxy;
mod step;

pub use body::{
    AggregateMassError, BodyActivationError, BodyControlError, BodyDef, BodyDefError, BodyMassData,
    BodyMassDataError, BodyMassResetError, BodySnapshot, BodyTransformError, BodyType,
    BodyTypeChangeError, WakePolicy,
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
pub use object::{
    CreateObjectError, DestroyedId, DestructionCause, DestructionRecord, ObjectSnapshot, World,
};
pub use step::{
    CollisionDirective, CommandApplication, CommandError, ContactEvent, ContactView,
    PreSolveDirective, StepError, StepHook, StepLifecycleEvent, StepLimits, StepPhase, StepReport,
    WorldCommand,
};
