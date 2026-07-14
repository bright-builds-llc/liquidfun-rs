//! An independent Rust implementation of the `LiquidFun` physics engine.
//!
//! This crate is currently a safe, Cargo-only early vertical slice. Its
//! collision namespace contains the Phase 5 substrate, while [`World`] owns the
//! checked Phase 7 body, fixture, contact, multi-contact island, continuous
//! collision, query, ray-cast, and origin-shift behavior described below.
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
//! Sensors use overlap-only touching and have no manifold, pre-solve callback,
//! or constraint. Persistent manifold points carry normal and tangent impulses
//! by semantic feature identity. Contacts expose only borrow-scoped
//! [`ContactView`] values and owned report snapshots; no reusable contact
//! handle or proxy coordinate is public.
//!
//! # Phase 7 checked rigid-world contract
//!
//! Granular velocity, force, impulse, damping, gravity-scale, awake, sleep,
//! bullet, and fixed-rotation controls validate a complete candidate before
//! replacing live body state. Non-finite input or derived arithmetic returns a
//! typed no-effect error. Forces and impulses apply only to dynamic bodies;
//! [`WakePolicy::Wake`] wakes before application, while
//! [`WakePolicy::PreserveSleep`] accepts an asleep body without applying the
//! control. Setting a body asleep atomically clears velocity, accumulated force
//! and torque, and sleep time.
//!
//! [`StepConfiguration`] validates timestep and solver iteration counts before
//! a step begins. [`World::step`] traverses bodies, contacts, manifold points,
//! and islands in deterministic source order, stages a complete island solve,
//! and commits body, impulse, and proxy state together. Warm-start impulses are
//! keyed by semantic contact features; disabling warm starting clears their
//! contribution for that call. Eligible quiet dynamic islands sleep together,
//! and relevant contacts, controls, or configuration changes wake them under
//! the documented policy. A successful step clears accumulated forces when
//! automatic clearing is enabled; applications that disable it use
//! [`World::clear_forces`] explicitly.
//!
//! Continuous-collision candidate, cache, and time-of-impact state is private and resumable.
//! Sub-stepping reports
//! [`StepCompletion::ContinuousPending`] after one accepted continuous event;
//! a matching later call resumes without repeating committed discrete work.
//! Exhausting the checked continuous-work budget returns semantic
//! [`ContinuousProgress`] through
//! [`StepError::ContinuousWorkLimitExceeded`], again preserving a coherent
//! resume point. Transient continuous solves are reported but do not overwrite
//! the persistent discrete warm-start lanes.
//!
//! [`World::query_aabb`] and [`World::ray_cast`] stream borrow-scoped semantic
//! fixture-child occurrences. Visitors may terminate, and ray visitors may
//! ignore, continue, or narrow the remaining interval. Their visitation order,
//! including equal-distance ray hits, is intentionally unspecified; fixture
//! multiplicity remains observable. [`World::shift_origin`] validates every
//! translated body, sweep, fixture bound, and broad-phase bound before one
//! atomic commit while preserving identities and local geometry.
//!
//! These APIs remain a bounded rigid-world slice. Joint solving, particle
//! solving, project-wide compatibility, reviewed platform coverage, and release
//! maturity remain later work.
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
pub mod joint;
pub mod math;
mod particle;
#[cfg(feature = "differential-internals")]
#[doc(hidden)]
pub mod rigid_differential;
pub mod rope;
mod world;

pub use association::{AssociationId, AssociationMap};
pub use error::{ArenaInsertError, HandleError, WorldKeyError};
pub use identity::{
    BodyId, FixtureId, JointId, ObjectKind, ParticleGroupId, ParticleId, ParticleSystemId,
};
pub use joint::{
    DistanceJointDef, DistanceJointSnapshot, FrictionJointDef, FrictionJointSnapshot, GearJointDef,
    GearJointSnapshot, JointDef, JointDefError, JointKind, JointLimitState, JointSnapshot,
    JointSpecificSnapshot, MotorJointDef, MotorJointSnapshot, MouseJointDef, MouseJointSnapshot,
    PrismaticJointDef, PrismaticJointSnapshot, PulleyJointDef, PulleyJointSnapshot,
    RevoluteJointDef, RevoluteJointSnapshot, RopeJointDef, RopeJointSnapshot, WeldJointDef,
    WeldJointSnapshot, WheelJointDef, WheelJointSnapshot,
};
pub use world::{
    AggregateMassError, BodyActivationError, BodyControlError, BodyDef, BodyDefError, BodyMassData,
    BodyMassDataError, BodyMassMutationError, BodyMassResetError, BodySnapshot, BodyTransformError,
    BodyType, BodyTypeChangeError, CollisionDirective, CommandApplication, CommandError,
    ContactEvent, ContactPointSnapshot, ContactSolve, ContactTransition, ContactTransitionKind,
    ContactView, ContinuousProgress, CreateObjectError, DestroyedId, DestructionCause,
    DestructionRecord, FixtureBoundsError, FixtureDef, FixtureDefError, FixtureDestructionError,
    FixtureMutationError, FixtureQueryOccurrence, FixtureSnapshot, JointCreationError,
    JointMutationError, JointQueryError, ManagedContactSnapshot, ObjectSnapshot, OriginShiftError,
    PreSolveDirective, QueryDirective, RayCastDirective, RayCastFraction, RayCastFractionError,
    StepCompletion, StepConfiguration, StepConfigurationError, StepError, StepHook,
    StepLifecycleEvent, StepLimits, StepPhase, StepReport, WakePolicy, World, WorldCommand,
    WorldConfigurationError, WorldFixtureSnapshot, WorldRayCastError, WorldRayHit,
};
