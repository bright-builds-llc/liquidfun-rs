//! World ownership and minimal non-solver object storage.

mod body;
pub(crate) mod object;
mod step;

pub use body::{BodyDef, BodyDefError, BodyMassData, BodyMassDataError, BodySnapshot, BodyType};
pub use object::{
    CreateObjectError, DestroyedId, DestructionCause, DestructionRecord, ObjectSnapshot, World,
};
pub use step::{
    CollisionDirective, CommandApplication, CommandError, ContactEvent, ContactSnapshot,
    ContactView, PreSolveDirective, StepError, StepHook, StepLimits, StepReport, WorldCommand,
};
