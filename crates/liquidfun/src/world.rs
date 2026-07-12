//! World ownership and minimal non-solver object storage.

mod body;
mod contact;
mod contact_manager;
mod contact_solver;
mod fixture;
pub(crate) mod object;
mod proxy;
mod step;

pub use body::{
    BodyActivationError, BodyDef, BodyDefError, BodyMassData, BodyMassDataError, BodySnapshot,
    BodyTransformError, BodyType,
};
pub use contact::{
    ContactPointSnapshot, ContactTransition, ContactTransitionKind, ManagedContactSnapshot,
};
pub use contact_solver::ContactSolve;
pub use fixture::{
    FixtureBoundsError, FixtureDef, FixtureDefError, FixtureMutationError, FixtureSnapshot,
    WorldFixtureSnapshot,
};
pub use object::{
    CreateObjectError, DestroyedId, DestructionCause, DestructionRecord, ObjectSnapshot, World,
};
pub use step::{
    CollisionDirective, CommandApplication, CommandError, ContactEvent, ContactView,
    PreSolveDirective, StepError, StepHook, StepLifecycleEvent, StepLimits, StepPhase, StepReport,
    WorldCommand,
};
