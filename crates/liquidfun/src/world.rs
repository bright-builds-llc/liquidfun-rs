//! World ownership and minimal non-solver object storage.

mod body;
mod fixture;
pub(crate) mod object;
mod proxy;
mod step;

pub use body::{
    BodyActivationError, BodyDef, BodyDefError, BodyMassData, BodyMassDataError, BodySnapshot,
    BodyTransformError, BodyType,
};
pub use fixture::{
    FixtureBoundsError, FixtureDef, FixtureDefError, FixtureMutationError, FixtureSnapshot,
    WorldFixtureSnapshot,
};
pub use object::{
    CreateObjectError, DestroyedId, DestructionCause, DestructionRecord, ObjectSnapshot, World,
};
pub use step::{
    CollisionDirective, CommandApplication, CommandError, ContactEvent, ContactSnapshot,
    ContactView, PreSolveDirective, StepError, StepHook, StepLimits, StepReport, WorldCommand,
};
