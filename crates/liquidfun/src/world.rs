//! World ownership and minimal non-solver object storage.

pub(crate) mod object;
mod step;

pub use object::{
    CreateObjectError, DestroyedId, DestructionCause, DestructionRecord, ObjectSnapshot, World,
};
pub use step::{
    CollisionDirective, CommandApplication, CommandError, ContactEvent, ContactSnapshot,
    ContactView, PreSolveDirective, StepError, StepHook, StepLimits, StepReport, WorldCommand,
};
