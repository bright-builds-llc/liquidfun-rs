//! An independent Rust implementation of the `LiquidFun` physics engine.
//!
//! This crate is currently a safe, Cargo-only foundation. Its collision
//! namespace contains the Phase 5 shape and collision substrate, but it does
//! not yet contain a rigid-body world, contact manager, or solver.
//!
//! # Phase 5 collision foundation
//!
//! [`collision`] provides immutable owned circle, edge, polygon, and chain
//! shapes; checked child selection and unary queries; source-ordered distance,
//! overlap, manifold, broad-phase pair, and time-of-impact operations; and a
//! generic dynamic tree with opaque tree-scoped proxy identity. Invalid or
//! non-finite geometry is rejected by typed constructors instead of inheriting
//! the pinned C++ assertions, truncation, fallback hull, or arithmetic-NaN
//! behavior. Ordinary tree query and ray collection order is intentionally
//! unspecified, while broad-phase pairs that feed later contact creation are
//! ordered and deduplicated by private source-compatible coordinates.
//!
//! The optional `differential-internals` feature is non-default,
//! `#[doc(hidden)]`, and reserved for the unpublished workspace differential
//! harness. It transports only bounded owned typed diagnostics. It is not a
//! stable consumer API and provides no raw storage identity, mutable cache,
//! packed contact key, unchecked constructor, or public iteration surface.
//!
//! Phase 5 deliberately stops before bodies, fixtures, contact-manager
//! creation, contact persistence or destruction, waking, joint suppression,
//! listeners, impulses, and rigid stepping. Those world-owned behaviors remain
//! Phase 6 work.
//!
//! # Phase 3 object model
//!
//! [`World`] exclusively owns all object storage. Its typed handles are opaque,
//! world-scoped identity tokens, with particle identities additionally scoped
//! to their owning particle system. They confer no access by themselves, remain
//! invalid after destruction or slot reuse, and fail explicitly when used with
//! another owner. Destruction returns owned [`DestructionRecord`] values whose
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
//! World-local semantic diagnostic IDs use checked allocation. After the final
//! `u64` identity is issued, later creation returns
//! [`ArenaInsertError::DiagnosticIdExhausted`] before inserting an object.
//!
//! This foundation deliberately exposes no durable contact handle, raw object
//! constructor, particle dense index, arbitrary callback closure command, raw
//! pointer, or particle bulk/external-buffer API. Full particle solving and the
//! API-09/API-10 buffer surface remain Phase 9 work.
//!
//! Handle kinds cannot be substituted for one another:
//!
//! ```compile_fail
//! use liquidfun::collision::{FilterData, Shape};
//! use liquidfun::collision::shape::CircleShape;
//! use liquidfun::math::Vec2;
//! use liquidfun::{FixtureDef, World};
//!
//! let mut world = World::new().expect("world key should remain available");
//! let body = world.create_body(&liquidfun::BodyDef::default()).expect("body should fit");
//! let shape = Shape::from(CircleShape::new(Vec2::ZERO, 0.5).expect("valid circle"));
//! let definition = FixtureDef::new(shape, 0.0, 0.2, 0.0, false, FilterData::default())
//!     .expect("valid fixture definition");
//! let fixture = world.create_fixture(body, &definition).expect("fixture should fit");
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
pub mod collision;
mod error;
mod identity;
pub mod math;
mod particle;
mod world;

pub use association::{AssociationId, AssociationMap};
pub use error::{ArenaInsertError, HandleError, WorldKeyError};
pub use identity::{
    BodyId, FixtureId, JointId, ObjectKind, ParticleGroupId, ParticleId, ParticleSystemId,
};
pub use world::{
    BodyActivationError, BodyDef, BodyDefError, BodyMassData, BodyMassDataError, BodySnapshot,
    BodyTransformError, BodyType, CollisionDirective, CommandApplication, CommandError,
    ContactEvent, ContactPointSnapshot, ContactSnapshot, ContactTransition, ContactTransitionKind,
    ContactView, CreateObjectError, DestroyedId, DestructionCause, DestructionRecord,
    FixtureBoundsError, FixtureDef, FixtureDefError, FixtureMutationError, FixtureSnapshot,
    ManagedContactSnapshot, ObjectSnapshot, PreSolveDirective, StepError, StepHook,
    StepLifecycleEvent, StepLimits, StepReport, World, WorldCommand, WorldFixtureSnapshot,
};
