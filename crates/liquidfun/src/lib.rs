//! An independent Rust implementation of the `LiquidFun` physics engine.
//!
//! This crate is currently a safe, Cargo-only foundation. It does not yet
//! implement physics behavior or claim behavioral parity with `LiquidFun`.
//!
//! # Phase 3 object model
//!
//! [`World`] exclusively owns all object storage. Its typed handles are opaque,
//! world-scoped identity tokens: they confer no access by themselves, remain
//! invalid after destruction or slot reuse, and fail explicitly when used with
//! another world. Destruction returns owned [`DestructionRecord`] values whose
//! snapshots outlive the invalidated objects. Application data belongs in an
//! application-owned [`AssociationMap`], with cleanup driven explicitly by
//! those records.
//!
//! A representative [`World::step`] accepts caller-owned contact snapshots,
//! exposes contacts to [`StepHook`] only through borrow-scoped read-only views,
//! and records ordered, non-deduplicated owned events. Hooks return narrow
//! directives and at most one typed command per contact. Commands are bounded,
//! applied sequentially after unlock, and report stale or foreign operands per
//! command without hiding later results. A hook panic restores the lock,
//! discards queued commands, poisons coherent-state operations, and resumes the
//! original panic.
//!
//! This foundation deliberately exposes no durable contact handle, raw object
//! constructor, particle dense index, arbitrary callback closure command, raw
//! pointer, or particle bulk/external-buffer API. Full particle solving and the
//! API-09/API-10 buffer surface remain Phase 9 work.
//!
//! Handle kinds cannot be substituted for one another:
//!
//! ```compile_fail
//! use liquidfun::World;
//!
//! let mut world = World::new().expect("world key should remain available");
//! let body = world.create_body().expect("body should fit");
//! let fixture = world.create_fixture(body).expect("fixture should fit");
//! world.destroy_body(fixture);
//! ```
//!
//! Handles have no public raw-parts constructor, and dense particle positions
//! are not part of the consumer API:
//!
//! ```compile_fail
//! use liquidfun::BodyId;
//!
//! let _body = BodyId::from_raw_parts(1, 2, 3);
//! ```
//!
//! ```compile_fail
//! use liquidfun::ParticleIndex;
//! ```

#![forbid(unsafe_code)]

mod arena;
mod association;
mod error;
mod identity;
mod particle;
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
