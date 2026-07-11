//! World ownership and minimal non-solver object storage.

pub(crate) mod object;

pub use object::{
    CreateObjectError, DestroyedId, DestructionCause, DestructionRecord, ObjectSnapshot, World,
};
