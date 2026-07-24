//! An independent Rust implementation of the `LiquidFun` physics engine.
//!
//! The published crate is a safe, Cargo-only, renderer-independent native Rust
//! engine. Its scalar implementation covers rigid bodies, contacts, joints,
//! rope, particles, queries, observations, and debug-draw data. Compatibility
//! claims remain evidence-scoped: consult the repository's generated
//! `COMPATIBILITY.md` instead of inferring whole-project parity from one API.
//!
//! Lengths use metres, mass uses kilograms, time uses seconds, and angles use
//! radians. Public constructors reject invalid or non-finite state with typed
//! errors. World mutation validates complete opaque handles before effects, and
//! owned reports preserve semantic event order without exposing storage.
//!
//! # API navigation
//!
//! - **Math and settings:** [`math`] contains vectors, matrices, transforms,
//!   sweeps, scalar helpers, and [`math::settings`] constants.
//! - **Collision and shapes:** [`collision`] contains immutable shapes,
//!   manifolds, distance and time-of-impact queries, broad phase, and dynamic
//!   tree APIs.
//! - **World, bodies, and fixtures:** [`World`], [`BodyDef`], [`FixtureDef`],
//!   their snapshots, and typed mutation errors form the owned simulation
//!   boundary.
//! - **Joints and rope:** [`joint`] contains all eleven world-owned joint
//!   definitions and snapshots; [`rope`] contains the independent rope model.
//! - **Particles and groups:** [`particle`] contains systems, stable particle
//!   and group identities, lifecycle, contacts, queries, editing, and owned
//!   buffer transfer.
//! - **Callbacks and events:** [`CollisionDecisionHook`], [`StepHook`],
//!   borrow-scoped views, typed directives, [`WorldCommand`], and owned
//!   [`StepReport`] values define the callback boundary.
//! - **Queries, observations, and profiles:** [`World::query_aabb`],
//!   [`World::ray_cast`], [`WorldObservation`], debug-draw primitives, and
//!   [`DiagnosticStepProfile`] expose renderer-neutral semantic state.
//! - **Handles and invalidation:** [`BodyId`], [`FixtureId`], [`JointId`],
//!   [`ParticleSystemId`], [`ParticleGroupId`], and [`ParticleId`] are opaque,
//!   owner-scoped identities; [`HandleError`] distinguishes foreign and stale
//!   use.
//! - **Errors and upstream naming:** recoverable failures are typed. Familiar
//!   `LiquidFun` nouns are retained where they clarify correspondence, while
//!   Rust APIs use owned values, checked constructors, and explicit mutation.
//!
//! A minimal headless world needs no C++ toolchain or renderer:
//!
//! ```
//! use liquidfun::math::Vec2;
//! use liquidfun::{BodyDef, BodyType, NoDecisionHook, StepConfiguration, StepLimits, World};
//!
//! let mut world = World::new()?;
//! let definition = BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)?;
//! let body = world.create_body(&definition)?;
//! let configuration = StepConfiguration::new(1.0 / 60.0, 8, 3)?;
//! let report = world.step(configuration, &mut NoDecisionHook, StepLimits::default())?;
//!
//! assert!(world.contains_body(body));
//! assert!(!report.phases().is_empty());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
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
//! # Phase 8 joints, rope, and callback contract
//!
//! [`World`] now owns checked definitions, snapshots, mutation, dependency
//! cascades, origin shifting, and transactional island solving for all eleven joint kinds.
//! Collision decisions, pre-solve controls, contact/destruction
//! events, and semantic reconstruction stay borrow-scoped or owned; no raw
//! contact, joint-storage, callback, or renderer authority escapes the world.
//! The standalone [`rope::Rope`] model remains independent of `World` and the
//! world-owned rope joint.
//!
//! The Phase 8 result has canonical scalar rigid-body and joint differential sign-off for the closed Phase 8 corpus.
//! That statement is limited to the
//! reviewed 19-family scalar Linux `x86_64` corpus and its named `phase8-v1`
//! policies. Later phases add particles, owned buffer transfer, bounded
//! renderer-neutral world observations, and diagnostic-only wall-clock
//! profiles. Broader compatibility, performance, platform, and release claims
//! require their own reviewed evidence.
//!
//! The checked-in compatibility ledger, not this historical milestone summary,
//! is authoritative for current subsystem evidence and explicit gaps.
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
//! This crate deliberately exposes no durable contact handle, raw object
//! constructor, particle dense index, arbitrary callback closure command, or
//! raw pointer. The safe external-buffer equivalent transfers uniquely owned
//! [`ParticleBufferBundle`] values into and out of a particle system.
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
pub mod debug_draw;
mod error;
mod identity;
pub mod joint;
pub mod math;
pub mod particle;
#[cfg(feature = "differential-internals")]
#[doc(hidden)]
pub mod rigid_differential;
pub mod rope;
mod world;

pub use association::{AssociationId, AssociationMap};
pub use debug_draw::{
    DebugCollectionError, DebugCollectionResource, DebugColor, DebugDrawLimits, DebugDrawOptions,
    DebugFill, DebugLayer, DebugOwnerKey, DebugPrimitive, DebugPrimitiveCollection,
    DebugPrimitiveKey, DebugPrimitiveKind, DebugPrimitiveMetadata, DebugPrimitiveSink, DebugStroke,
};
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
pub use particle::{
    ParticleBodyContact, ParticleBodyContactEffect, ParticleBodyContactUpdate,
    ParticleBodyContactView, ParticleBufferAdoptionError, ParticleBufferAdoptionErrorKind,
    ParticleBufferBundle, ParticleBufferError, ParticleBufferErrorKind, ParticleBufferLanes,
    ParticleBufferMode, ParticleBufferTeardown, ParticleCapacity, ParticleColor, ParticleContact,
    ParticleContactEffect, ParticleContactError, ParticleContactUpdate, ParticleContactView,
    ParticleCreationReceipt, ParticleDef, ParticleDefError, ParticleDestructionOccurrence,
    ParticleEditError, ParticleEditor, ParticleFlags, ParticleForceError,
    ParticleGroupMutationError, ParticleLifetimeClock, ParticleLifetimeError,
    ParticleLifetimeOrder, ParticleNeighborPair, ParticleNeighborhood, ParticlePairView,
    ParticleProxyError, ParticleQueryError, ParticleQueryOccurrence, ParticleRayCastError,
    ParticleRayHit, ParticleSnapshot, ParticleSystemDef, ParticleSystemDefError,
    ParticleSystemSnapshot, ParticleSystemStatistics, ParticleSystemView, ParticleTriadView,
    ParticleWorldStatistics,
};
pub use world::{
    AggregateMassError, BodyActivationError, BodyControlError, BodyDef, BodyDefError, BodyMassData,
    BodyMassDataError, BodyMassMutationError, BodyMassResetError, BodyObservation, BodySnapshot,
    BodyTransformError, BodyType, BodyTypeChangeError, BroadPhaseObservation,
    CollisionDecisionHook, CollisionDirective, CollisionFilterEvent, CommandApplication,
    CommandError, ContactControlError, ContactEvent, ContactObservation, ContactPointSnapshot,
    ContactSolve, ContactTransition, ContactTransitionKind, ContactView, ContinuousProgress,
    CreateObjectError, DestroyedId, DestructionCause, DestructionRecord, DestructionReport,
    DiagnosticProfileChild, DiagnosticProfileParent, DiagnosticProfileSchema, DiagnosticStepPhase,
    DiagnosticStepPhaseTiming, DiagnosticStepProfile, FixtureBoundsError, FixtureDef,
    FixtureDefError, FixtureDestructionError, FixtureMutationError, FixtureObservation,
    FixturePairView, FixtureParticleView, FixtureQueryOccurrence, FixtureSnapshot,
    JointCreationError, JointMutationError, JointObservation, JointQueryError, LifecycleEvent,
    ManagedContactSnapshot, MutationReport, NoDecisionHook, ObjectSnapshot, OriginShiftError,
    ParticleBodyContactObservation, ParticleContactObservation, ParticleObservation,
    ParticlePairContactView, PreSolveDirective, PreSolveView, QueryDirective, RayCastDirective,
    RayCastFraction, RayCastFractionError, StepCompletion, StepConfiguration,
    StepConfigurationError, StepError, StepHook, StepLifecycleEvent, StepLimits, StepPhase,
    StepReport, WakePolicy, World, WorldCommand, WorldConfigurationError, WorldDiagnostics,
    WorldFixtureSnapshot, WorldObservation, WorldObservationError, WorldObservationLimitError,
    WorldObservationLimits, WorldObservationResource, WorldQueryOccurrence, WorldRayCastError,
    WorldRayCastOccurrence, WorldRayCastWithParticlesError, WorldRayHit,
};
#[cfg(feature = "differential-internals")]
pub use world::{ReconstructionSupport, ReconstructionUnsupported};
