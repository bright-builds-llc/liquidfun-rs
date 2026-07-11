//! An independent Rust implementation of the `LiquidFun` physics engine.
//!
//! This crate is currently a safe, Cargo-only foundation. It does not yet
//! implement physics behavior or claim behavioral parity with `LiquidFun`.

#![forbid(unsafe_code)]

mod arena;
mod association;
mod error;
mod identity;
mod world;

pub use association::{AssociationId, AssociationMap};
pub use error::{ArenaInsertError, HandleError, WorldKeyError};
pub use identity::{
    BodyId, FixtureId, JointId, ObjectKind, ParticleGroupId, ParticleId, ParticleSystemId,
};
pub use world::{
    CollisionDirective, CommandApplication, CommandError, ContactEvent, ContactSnapshot,
    ContactView, CreateObjectError, DestroyedId, DestructionCause, DestructionRecord,
    ObjectSnapshot, PreSolveDirective, StepError, StepHook, StepLimits, StepReport, World,
    WorldCommand,
};
