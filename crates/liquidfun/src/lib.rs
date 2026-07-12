//! An independent Rust implementation of the `LiquidFun` physics engine.
//!
//! This crate is currently a safe, Cargo-only early vertical slice. Its
//! collision namespace contains the Phase 5 substrate, and [`World`] now owns
//! the Phase 6 checked body, fixture, broad-phase proxy, automatic contact, and
//! bounded one-contact solver lifecycle.
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
//! Phase 6 now adds checked body and fixture ownership, automatic contact
//! lifecycle, and one bounded static/dynamic contact solve. General islands,
//! joints, sleeping, forces, and continuous world stepping remain later work.
//!
//! # Phase 6 rigid-world contract
//!
//! [`BodyDef`] and [`FixtureDef`] are reusable checked definitions.
//! [`FixtureDef`] owns an immutable [`collision::Shape`] snapshot by value.
//! [`World::create_body`], [`World::body_snapshot`],
//! [`World::set_body_type`], [`World::set_body_transform`],
//! [`World::set_body_active`], [`World::create_fixture`], and
//! [`World::fixture_snapshot`] validate complete world-scoped handles before
//! effects and return owned state rather than storage borrows.
//!
//! Creating a positive-density fixture and destroying any fixture reset body
//! mass. [`World::set_fixture_density`] does not reset it;
//! [`World::reset_body_mass_data`] is explicit. [`World::set_body_mass_data`]
//! is a current dynamic-body override and is a no-op for static and kinematic
//! bodies. Later reset-triggering fixture or body-type changes replace it.
//! Fixture friction and restitution affect contacts created afterward, while
//! existing contacts retain their creation-time mixed values. Sensor and
//! filter changes are observed by the next contact update.
//!
//! [`World::step`] discovers pairs, updates contacts, invokes restricted hooks,
//! solves at most one supported static/dynamic occurrence, unlocks, then
//! applies bounded deferred commands. Sensors use overlap-only touching and
//! have no manifold, pre-solve callback, or constraint. Persistent manifold
//! points carry normal and tangent impulses by semantic feature identity.
//! Contacts expose only borrow-scoped [`ContactView`] values and owned report
//! snapshots; no reusable contact handle or proxy coordinate is public.
//!
//! Phase 7 retains forces, public velocity controls, damping, sleeping, the
//! general island solver, multi-contact stacks, CCD/TOI world orchestration,
//! queries, ray casts, and broad world configuration. Joint solving follows in
//! Phase 8.
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
//! [`World::step`] discovers and updates private manager contacts, exposes them
//! to [`StepHook`] only through borrow-scoped read-only views, solves the one
//! supported rigid occurrence, and records ordered owned evidence. Hooks
//! return narrow directives and at most one typed command per occurrence.
//! Commands are bounded, applied sequentially after unlock, and report stale
//! or foreign operands without hiding later results. A hook panic restores the
//! lock, discards queued commands, poisons coherent-state operations, and
//! resumes the original panic.
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
#[cfg(feature = "differential-internals")]
#[doc(hidden)]
pub mod rigid_differential;
mod world;

pub use association::{AssociationId, AssociationMap};
pub use error::{ArenaInsertError, HandleError, WorldKeyError};
pub use identity::{
    BodyId, FixtureId, JointId, ObjectKind, ParticleGroupId, ParticleId, ParticleSystemId,
};
pub use world::{
    AggregateMassError, BodyActivationError, BodyDef, BodyDefError, BodyMassData,
    BodyMassDataError, BodyMassResetError, BodySnapshot, BodyTransformError, BodyType,
    CollisionDirective, CommandApplication, CommandError, ContactEvent, ContactPointSnapshot,
    ContactSolve, ContactTransition, ContactTransitionKind, ContactView, CreateObjectError,
    DestroyedId, DestructionCause, DestructionRecord, FixtureBoundsError, FixtureDef,
    FixtureDefError, FixtureMutationError, FixtureSnapshot, ManagedContactSnapshot, ObjectSnapshot,
    PreSolveDirective, StepError, StepHook, StepLifecycleEvent, StepLimits, StepPhase, StepReport,
    World, WorldCommand, WorldFixtureSnapshot,
};
