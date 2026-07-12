# Architecture

## Current status

Phase 2 establishes the permanent semantic-comparison seam. Phase 3 adds the
native-Rust object model, Phase 4 the consumer math/settings surface, and Phase
5 the safe shape/collision substrate. Phase 6 now adds a minimal rigid-world
vertical slice: checked bodies and fixtures, world-owned proxies and contacts,
and one bounded static/dynamic contact solve. Both required rigid lifecycle
timelines execute through the native adapter and pinned process-isolated C++
oracle in local debug and release builds. General rigid dynamics, broad solver
topologies, canonical-platform parity, particles, and performance parity remain
pending. The publishable `liquidfun` crate therefore remains version `0.0.0`, and the
generated [compatibility inventory](COMPATIBILITY.md) remains the authority for
maturity.

## Dependency direction

The implemented dependency graph is deliberately one-way:

```text
ordinary Cargo consumer -> crates/liquidfun

crates/liquidfun-differential -> crates/liquidfun-test-protocol
                              -> crates/liquidfun
                              -> subprocess: liquidfun-reference

liquidfun-reference -> repository CMake wrapper -> read-only LiquidFun C++

maintainer -> just -> cargo xtask -> Cargo, differential runner, evidence checks
```

`crates/liquidfun` is the only default member. Neither unpublished harness crate,
`tools/xtask`, CMake, the submodule, nor reference evidence is a dependency or
feature of the published crate. Plain `cargo build`, `cargo test`, and
`cargo doc` therefore remain Cargo-only consumer operations.

Production behavior must be native Rust. C++ is a development oracle, not a
runtime implementation. There is no FFI boundary in Phase 2: the supported
boundary is an external JSON Lines process. An in-process C ABI remains deferred
unless profiling later demonstrates a material bottleneck, and it may never
become a consumer dependency.

## Phase 4 math and numerical boundaries

The public `liquidfun::math` deep module owns the native consumer contract. It
maps the selected `b2Math.h` concepts to initialized safe Rust values:
`Vec2`, `Vec3`, `Vec4`, `Mat22`, `Mat33`, `Rotation`, `Transform`, and `Sweep`.
Vectors expose coordinates, while matrices, rotations, transforms, and sweeps
keep representation private behind initialized constructors and accessors.
Unlike the C++ source, there are no uninitialized defaults, raw indexing,
pointer/layout promises, allocator hooks, mutable global settings, or public
approximate-equality API. `Sweep` validates stored finite state and monotonic
fractions with typed errors before mutation.

The `math::settings` namespace maps the behavior-affecting `b2Settings.h`
constants to immutable Rust constants. Physics uses meters-kilograms-seconds
(MKS), angles use radians, and angular velocity uses radians per second. Matrix
operations are column-major; `Transform` maps local coordinates into a parent
frame by rotation then translation; `Sweep` interpolates between its initial
and final centers and angles at a checked normalized fraction. Rendering scale
and pixel conversion remain outside the engine.

Compatibility-sensitive arithmetic preserves the pinned source's operand
order, branch direction, and expression grouping. It deliberately avoids
`mul_add`, `sin_cos`, reassociation, and general-purpose math dependencies.
Raw math values can carry signed zero, subnormals, infinities, and NaNs for
probe fidelity, while physics-domain boundaries require finite values.

The numerical-policy and probe machinery points inward toward the public math
module but never leaks outward: `liquidfun-test-protocol` privately owns exact
bit transport, typed field policies, horizons, and evidence tiers;
`liquidfun-differential` privately owns comparison and diagnostics; and the
external C++ adapter executes only repository-defined, bounded probe operations
against the pinned oracle. None is a dependency or feature of `liquidfun`.

## Phase 5 collision boundaries

The public `liquidfun::collision` deep module owns immutable initialized values.
`CircleShape`, `EdgeShape`, `PolygonShape`, and `ChainShape` own their geometry;
the exhaustive `Shape` enum dispatches without public trait objects. Open and
closed chains are distinct, closed loops store semantic vertices once, and a
checked `ChildIndex` selects an owned adjacency-bearing child edge. Fallible
constructors reject non-finite data, invalid radii and densities, degenerate or
oversized polygons, invalid chains, reversed AABB bounds, invalid ray fractions,
and invalid TOI intervals before a kernel runs.

Those checks are deliberate safe-Rust differences from the pinned source's
debug assertions, polygon truncation or fallback box, unchecked child access,
and arithmetic-NaN results. The circle-center point-distance case returns a
finite zero normal. Edge and chain point tests remain false, and ray casts that
start inside retain the pinned no-hit convention. `Aabb`, `RayCastInput`,
`RayCastHit`, and `MassData` expose initialized semantic values without layout
or raw-parts promises.

One source-ordered GJK implementation serves distance and overlap. A
`DistanceCache` begins cold, binds to the ordered shape-child topology that
created it, and exposes only an owned semantic snapshot of count, ordered
support-index pairs, and metric. Cross-topology reuse is rejected before index
access. Source cache validation uses the inclusive metric-ratio window
`[0.5, 2.0]` and the strict `metric < EPSILON` reset; overlap uses the strict
`distance < 10.0 * EPSILON` predicate. These are named operation rules, not a
repository-wide tolerance.

Contact identity is semantic: manifold kind, active point order, reference
orientation, and typed vertex/face features are observable. The C++ packed
feature union is never public or compared. The closed pair registry covers
circle-circle, polygon-circle, polygon-polygon, edge-circle, edge-polygon,
chain-child-circle, and chain-child-polygon, with explicit reversed-input
orientation. Outcomes distinguish unsupported, separated, and touching;
inactive fields and solver impulses are absent.

`DynamicTree<T>` uses opaque tree-scoped generational `ProxyId` values. It
publishes no slot coordinate, raw constructor, serialization, ordering, or
whole-tree iterator. Borrow-scoped visitors control AABB queries and rays.
Collect-all query and ray membership is set-like and its consumer order is
unspecified. `BroadPhase<T>` preserves exact ordered pair generation: move and
touch occurrences are queried, candidates sort by private node coordinates,
and adjacent duplicates are removed. `FilterData` implements the pure group and
symmetric mask rule; changing a filter touches the proxy so newly eligible
pairs are reconsidered.

`TimeOfImpactInput` validates both shape children, copies checked `Sweep`
values, and bounds `t_max` to `0.0..=1.0`. `TimeOfImpactOutput` exposes only
`Overlapped`, `Touching`, `Separated`, or `Failed` plus time. The fixed kernel
retains the source target/tolerance formulas, outer cap 20, root cap 50,
push-back cap `MAX_POLYGON_VERTICES`, and alternating bisection/secant sequence;
support, branch, and iteration details remain bounded diagnostics.

The `differential-internals` feature is non-default, `#[doc(hidden)]`, and
enabled solely by the unpublished `liquidfun-differential` workspace crate. It
is development-only, unstable evidence plumbing carrying bounded owned typed
records. It exposes no raw storage, mutation, packed keys, private node
coordinates, or unchecked constructors. Ordinary builds, default rustdoc, and
packaged consumers do not include its diagnostic module.

Phase 5 proves immutable shape snapshots, semantic manifold identity, ordered
broad-phase pairs, and pure filter/refilter reconsideration for Phase 6. It has
no bodies or fixtures, contact-manager insertion, contact persistence or
destruction, waking, joint suppression, listener timing, warm-start impulses,
or rigid stepping. In particular, the Phase 5 portion of `COLL-05` is only the
pair/filter/refilter substrate.

This layout follows the repository guidance and Bright Builds architecture,
code-shape, testing, verification, and Rust standards: invariant-bearing values
form the pure core; the differential process is an imperative shell; cohesive
child modules keep the public seam deep; focused tests lock branch behavior;
and repository-native checks gate evidence. `standards-overrides.md` contains no
substantive active exception to those rules.

## Phase 6 rigid-world boundaries

The public rigid-world seam is `World` plus checked, reusable `BodyDef` and `FixtureDef`
values. `BodyType` is closed over static, kinematic, and dynamic bodies.
`FixtureDef` owns an immutable `Shape` snapshot by value; neither a
definition nor a live fixture exposes mutable geometry or storage authority.
Every public operation validates the complete world-scoped handle before an
effect. The supported body/fixture operations are `World::create_body`,
`World::body_snapshot`, `World::set_body_type`,
`World::set_body_transform`, `World::set_body_active`,
`World::create_fixture`, `World::fixture_snapshot`, and the corresponding
destruction methods. Non-finite transforms, invalid material or mass values,
invalid topology, and foreign or stale handles return typed errors rather than
being clamped or partially applied.

Fixture children have private world-owned broad-phase proxies. A transform
change synchronizes those proxies immediately, but discovery and contact update
wait for the next `World::step`. Deactivation destroys contacts before proxies;
activation recreates proxies and likewise waits for stepping to discover
contacts. A type change destroys contacts, resets source-compatible body state,
and touches proxies for later reconsideration. These transitions run through
centralized validate-then-commit paths so contact-end evidence is captured while
its body and fixture semantics are still live.

Mass behavior deliberately preserves the pinned asymmetry. Creating a fixture
with positive density resets mass, and fixture destruction always resets it.
`World::set_fixture_density` changes density without recomputing mass;
`World::reset_body_mass_data` performs the explicit recomputation.
`World::set_body_mass_data` is a current dynamic-body override and a
source-compatible no-op for static or kinematic bodies. A later reset-triggering
fixture or body-type operation replaces the override.

Private contact-manager occurrences consume Phase 5 ordered broad-phase pairs,
use canonical manifold feature identity, and preserve manager, manifold-point,
hook, report, and destruction order. Friction uses the pinned geometric mean
and restitution the pinned maximum when a contact is created. Those
creation-time mixed friction and restitution values persist when fixture
material changes and change only when the contact is recreated. Sensor changes
wake the parent for the next update through `World::set_fixture_sensor`;
sensors use overlap-only touching with no manifold, pre-solve call, or
constraint. `World::set_fixture_filter` flags current contacts and touches every
active proxy for the next update.

`World::step` exposes the reviewed order through `StepPhase` and `StepReport`:
`FindPairs`, `UpdateContacts`, `Hook`, `Solve`, `Unlock`, followed by
`ApplyCommands` only when hooks requested deferred work. Reports own ordered
begin, persist, end, hook, solve, command, and destruction evidence. Hooks see
only a borrow-scoped `ContactView`, receive no mutable `World`, and may request
one bounded typed command per occurrence. No durable contact identity crosses
the public boundary; the harness-only occurrence ordinal is available solely
under the non-default hidden `differential-internals` feature.

The solver boundary is intentionally one static/dynamic contact with at most
two canonical manifold points. It carries normal and tangent impulses by
semantic feature identity and fails closed before velocity or impulse mutation
for unsupported topology or non-finite derived state. Phase 7 owns forces,
torques, public velocity control, damping, gravity scale, fixed rotation,
sleeping, the general island solver, multi-contact stacks, CCD/TOI world
orchestration, queries, ray casts, origin shifting, and broad world
configuration. Phase 8 owns joint solving. Public contact handles and mutable
shape storage remain intentionally excluded rather than deferred.

The private evidence boundary is one declaration-first rigid timeline using
the `phase6-v1` closed field policy. The
`non_colliding_body_fixture_lifecycle` family covers all body types, fixture and
mass mutations, zero contacts, and explicit teardown. The
`single_contact_lifecycle` family covers creation, persistence, warm-start
carry, sensor/filter/activation changes, recreation, and ordered destruction.
Each engine must satisfy the declared witnesses and counts before the two
results are compared. Exact transport, field-specific policy, D0 byte identity,
D1 canonical authority, and local D2 evidence remain separate dimensions; a
local pass cannot promote a canonical fixture or platform claim.

## Private protocol and domain core

`crates/liquidfun-test-protocol` is an unpublished functional core. It owns:

- independent protocol, scenario-schema, trace-schema, and tolerance-profile
  version types;
- strict bounded JSON Lines framing and raw-to-domain parsing;
- semantic request, entity, command, checkpoint, provenance, and trace types;
- authoritative `f32` transport as exact `u32` bit patterns;
- named immutable limits and session profiles;
- typed harness failures and deterministic schema/tolerance presentations.

Raw JSON never becomes a generic comparison surface. The Rust and C++ codecs
reject partial records, duplicate or unknown members, unknown record kinds,
unsupported versions, invalid references, and values outside named limits
before execution. Once parsed, the rest of the system receives invariant-bearing
domain values rather than unchecked strings, paths, or maps.

Exact transport bits and comparison policy are separate responsibilities.
Every authoritative float crosses the process boundary exactly. The comparator
then applies the reviewed field policy: exact bits, absolute, relative, or ULP
only where that field's typed policy permits it. This separation prevents a JSON
formatter or repository-wide numeric tolerance from silently changing evidence.

## Differential functional core and imperative shell

`crates/liquidfun-differential` is also unpublished. Its comparator,
canonicalization, first-divergence reporting, failure signatures, and scenario
reducer form the functional core. Typed exhaustive matches compare discrete
values exactly, apply field-specific float policy, and canonicalize only
collections explicitly modeled as sets or multisets. Checkpoints and future
solver, callback, and destruction sequences remain ordered.

The process supervisor, fixture storage, and CLI are the imperative shell. One
synchronous supervisor serves one-shot, bounded reuse, and sanitizer profiles;
there are not separate implementations. Standard-library threads drain stdout
and stderr concurrently. Requests are sequential with exactly one request in
flight. A request budget of one maximizes reproduction isolation, while finite
reviewed budgets exercise reset and bounded process reuse.

Every request constructs isolated adapter state. A complete `trace_end` proves
destruction, cleared semantic mappings, reset verification, and an incremented
reset epoch. Timeout, malformed or oversized output, sanitizer evidence,
unexpected exit, identity mismatch, or reset failure poisons the session. The
supervisor kills when needed, waits and reaps, joins drains, retains bounded
first/last stderr evidence, and never reuses or silently retries a poisoned
child.

The Phase-2 native adapter is private and intentionally supports only the
empty-world lifecycle needed to prove the seam. It is not the public `World`
API; Phase 3 owns the native handle, invalidation, callback, and storage
contracts without importing protocol or oracle types into `liquidfun`.

## Native Rust object and module boundaries

The `liquidfun` crate is one deep, safe Rust module. `lib.rs` is the only public
curation boundary; implementation modules remain private unless a consumer
contract requires a type:

- `identity.rs` owns six distinct opaque typed handles. `arena.rs` owns the
  private deterministic generational storage used by world objects.
- `world/object.rs` exclusively owns bodies, fixtures, joints, particle
  systems, particle groups, particles, adjacency, and every destruction path.
- `world/body.rs`, `world/fixture.rs`, and `world/proxy.rs` own checked rigid
  definitions, private live state, immutable shape ownership, and world-owned
  broad-phase entries.
- `world/contact.rs`, `world/contact_manager.rs`, and
  `world/contact_solver.rs` own private automatic contacts, semantic manifold
  persistence, creation-time material, and the bounded one-contact solve.
- `world/step.rs` owns the automatic step lock, restricted hook calls, bounded
  event and command collection, ordered reports, command application, and
  poison state.
- `association.rs` owns the sealed typed application-side-table abstraction;
  association values never enter `World`.
- `particle/storage.rs` and its children are a private representative SoA
  architecture spike. They are executable evidence for later particle work,
  not a consumer bulk-storage API.

All production modules compile with `unsafe_code` forbidden. Public handles
contain no raw pointers and grant no access without validation by their owning
world. Harness crates may depend on `liquidfun`; no protocol, differential,
reference, serialization, CMake, C++, or renderer concern may depend inward
from `liquidfun`.

## World ownership, typed handles, and destruction

One `World` owns every object arena. A complete handle identity contains a
private process-unique world key, private slot, and checked `u64` generation;
particle identity additionally contains the complete owning particle-system
scope. Equality and hashing cover that complete identity, while public
constructors, slot values, serialization, ordering, and dense positions remain
unavailable. Handles are authority-free integer identities and receive `Send`
and `Sync` only through Rust auto traits.

Every lookup validates handle kind internally, then world scope, slot,
generation, and particle-system scope where applicable. Public typed signatures
make wrong-kind substitution a compile-time error. Foreign handles return
`WrongWorld`; same-world particle owner mismatches return
`WrongParticleSystem`; destroyed or reused-slot identities return
`StaleOrDestroyed`. Removing an arena entry advances its generation before
reuse, and a generation that cannot advance retires its slot permanently rather
than wrapping. Capacity, world-key, and generation exhaustion are explicit
failures. World-local semantic diagnostic IDs likewise advance with checked
arithmetic: `u64::MAX` may be issued once, after which creation returns
`DiagnosticIdExhausted` before inserting an object.

All object destruction is centralized in `World`, validates the root before
mutation, updates both sides of adjacency, invalidates each affected handle,
and returns owned semantic records. Body cascades emit attached joints, then
fixtures, then the body. Particle-system cascades emit groups, then particles,
then the system. Body joint and fixture adjacency is prepended on creation so
records, snapshots, and association cleanup preserve the pinned upstream
newest-first list order within those categories. Particle-system categories
preserve creation/occurrence order. A particle-system cascade captures the root
membership and every particle's optional group as one transaction before group
cleanup begins, while still emitting group records before particle records.
Owned snapshots therefore retain the required pre-invalidation adjacency,
owner, group, and diagnostic state and remain usable after slot reuse. Direct
group destruction clears membership without destroying its particles.

## Transient contacts, restricted hooks, and step order

Contacts have no durable public identity. The private contact manager owns
their lifecycle; hooks receive only a borrow-scoped read-only `ContactView`,
and polling consumers receive owned snapshots through `ContactEvent`,
`ContactTransition`, `ContactSolve`, and `StepReport`. Rust lifetimes prevent
retaining an internal contact view, and hook trait signatures provide no
`&mut World`.

The Phase 6 step follows one enforceable sequence:

1. Reject a poisoned or already locked world, then acquire the RAII step lock.
1. Discover ordered broad-phase pairs and admit eligible private contacts.
1. Refilter, update, or destroy contacts and capture touching transitions.
1. Preflight the supported topology, invoke restricted hooks, and solve the
   one reviewed occurrence with semantic impulse carry.
1. Restore the lock before applying any command.
1. Apply bounded typed commands sequentially in request order, revalidating
   every operand at application time. A stale or foreign operand becomes that
   command's owned failure and does not suppress later applications.
1. Return one owned `StepReport` containing exact phase, transition, hook,
   solve, destruction, and command-result order.

`StepLimits` is caller-configurable only up to reviewed hard maxima of 4,096
events and 1,024 commands. Limit failure discards the pending command queue;
hooks cannot return arbitrary closures or build an unreviewed command surface.
If a hook panics, the step restores the lock through RAII, discards unapplied
commands, marks the world poisoned, and resumes the original panic. Diagnostic
lock/poison and handle-liveness queries remain available, but later step,
creation, and destruction operations fail explicitly instead of treating
partially progressed state as coherent.

## Safe application associations

`AssociationMap<Id, T>` is an application-owned typed side table sealed to one
exact public handle kind. It does not use `Any`, raw pointers, lifetime-long
borrows, or a `World<T>` generic. Destruction cannot mutate application memory
implicitly: consumers pass owned destruction records to explicit cleanup
helpers, which remove matching identities in record occurrence order and leave
other kinds or worlds untouched.

## Particle storage and future buffer boundary

Public `ParticleId` is stable, world-scoped, and particle-system-scoped; the
private dense particle index is ephemeral and never crosses `lib.rs`. The
representative dense SoA keeps required and materialized optional lanes aligned
and embeds the complete owning `ParticleSystemId` scope in every stable identity,
so different systems may safely reuse the same local slot and generation ranges.
Every lookup checks that owner before dense resolution. Stable-to-dense and
dense-to-stable mappings use explicit live, pending-delete, vacant, and retired
states. Pending deletion rejects ordinary mutation while retaining an owned row
snapshot. Compaction then advances or retires the identity generation, so
removed IDs cannot alias surviving rows.

One private validate-then-commit permutation is authoritative for lane reorder
and compaction. It validates the complete candidate before changing required or
optional lanes, both identity directions, proxies, contacts, pairs, triads,
deterministic lifetime order, or contiguous group ranges. Invalid duplicates,
lane lengths, or derived references leave the prior state unchanged. Vectors
and explicit ordering, rather than hash iteration, determine solver-visible
storage order. Focused tests and a bounded 128-case model state machine cover
create, reorder, mark-delete, compact, stale access, and capacity failure.

The future API-09/API-10 particle bulk-mutation and external-buffer surface is
not public or complete. Phase 3 proves only the locked safe direction with a
private owned lane bundle: validate ownership and lane lengths at construction,
track declared fixed capacity separately from allocation capacity, reject
growth with `CapacityExceeded`, expose no raw pointer, and return owned buffers
on teardown. Full API design, solver integration, compatibility evidence, and
performance sign-off remain Phase 9 or later.

## Renderer independence

The simulation and object/storage modules are headless and expose no renderer,
window, input, UI, GPU, or framework dependency. Future debug-draw data and
traits must remain renderer-neutral. A private testbed may translate those
values into a renderer, but testbed scheduling, frame pacing, and storage may
not dictate `World`, handle, callback, or particle layout.

## Phase 3 decision sign-off

| Decision | Disposition | Executable evidence |
| --- | --- | --- |
| D-01 | Implemented: six opaque typed identities use private world/slot/generation coordinates and custom arenas. | `identity.rs`; `arena.rs`; `tests/object_model.rs::public_handle_kinds_are_distinct_types` |
| D-02 | Implemented: checked generation advance permanently retires exhausted slots, checked diagnostic allocation rejects insertion after `u64::MAX`, and finite-space failures remain explicit. | `arena.rs::tests::maximum_generation_retires_permanently`; `world/object.rs::tests::diagnostic_identity_exhaustion_rejects_insertion`; seeded arena model test |
| D-03 | Implemented: typed signatures reject wrong kinds; runtime lookup distinguishes foreign from stale identities. | crate compile-fail doctest; `tests/object_model.rs` stale/reuse and cross-world tests |
| D-04 | Implemented: complete identity, including particle-system scope for `ParticleId`, controls equality/hash while constructors, coordinates, serialization, and ordering stay private. | `identity.rs` equality/scope/debug tests; crate raw-parts compile-fail doctest |
| D-05 | Implemented: handles use auto traits only and production forbids unsafe code. | `identity.rs::tests::handles_are_send_and_sync_through_auto_traits`; `lib.rs` crate lint |
| D-06 | Implemented for the Phase-3 object graph: centralized cascades preserve pinned upstream newest-first body adjacency order and owned snapshots. | `world/object.rs`; `tests/object_model.rs::body_destruction_returns_owned_ordered_cascade_evidence`; `tests/object_model.rs::typed_association_cleanup_follows_destruction_records` |
| D-07 | Implemented: contacts are borrow-scoped views or owned snapshots/events with no durable handle. | `world/step.rs` contact-view compile-fail doctest; `tests/hook_contract.rs` |
| D-08 | Implemented for representative hooks: read-only views return narrow filter/pre-solve directives and optional typed commands. | `world/step.rs` hook-signature compile-fail doctest; owned-directive integration test |
| D-09 | Implemented: owned reports preserve event occurrence order and multiplicity without deduplication. | `tests/hook_contract.rs::owned_events_preserve_hook_order_multiplicity_and_directives` |
| D-10 | Implemented: bounded commands apply sequentially after unlock with explicit per-command stale/foreign results. | `tests/hook_contract.rs` deferred and stale-command tests |
| D-11 | Implemented: RAII unlock, command discard, persistent poison, and resumed unwind are consumer-visible. | `tests/hook_contract.rs::hook_panic_restores_lock_and_poison_gates_later_operations` |
| D-12 | Implemented privately and exposed only as stable system-scoped identity: overlapping system-local slot/generation ranges cannot alias, while dense positions remain ephemeral and group-contiguous. | `particle/storage/identity.rs::cross_system_id_is_rejected_before_dense_lookup`; `tests/object_model.rs::particle_group_owner_mismatch_reports_particle_system_scope`; `tests/particle_identity.rs` |
| D-13 | Implemented privately: live, pending-delete, vacant, and retired mappings preserve snapshots then invalidate on compaction. | `particle/storage/identity.rs` pending, compaction, and retirement tests |
| D-14 | Implemented privately: one validate-then-commit permutation updates every representative lane, map, derived index, and group range. | `particle/storage/permutation.rs` remapping and unchanged-on-error tests |
| D-15 | Implemented as a bounded architecture spike only; no particle solver pass is present. | `particle/storage/properties.rs::bounded_state_machine_matches_independent_model` |
| D-16 | Implemented: sealed application-owned typed side tables clean up explicitly from owned destruction records. | `association.rs` compile-fail/unit tests; `tests/object_model.rs` cleanup test |
| D-17 | Direction locked and proved privately, deferred publicly: owned validated lanes, fixed declared capacity, and owned teardown. | `particle/storage/properties.rs` capacity/teardown tests; public API completion deferred to Phase 9 |

## Process-isolated C++ oracle

`tools/reference/liquidfun-reference` is a repository-owned executable built by
the external CMake wrapper and linked to the pinned read-only submodule. Adapter
sources stay outside `third_party/liquidfun`; generated builds stay below
`target/reference/<preset>`.

The executable emits a startup handshake before accepting scenarios. Its build
identity includes the pinned oracle revision, adapter source digest, preset,
compiler, target, effective flags, sanitizer mode, and stable identity hash.
The runner checks that identity independently before comparing semantic values.
Stdout is protocol-only; diagnostics use stderr.

The C++ side parses bounded JSON events directly into closed domain structs,
constructs a fresh `b2World` per request, emits ordered semantic checkpoints,
destroys request state, verifies reset, and only then emits `trace_end`. C++
pointers, STL objects, dense indices, padding, and raw memory never cross the
protocol.

## Comparison and diagnosis boundary

Only two complete schema-, request-, tolerance-, provenance-, payload-, and
reset-validated traces may reach comparison. A `PhysicsMismatch` therefore means
two trustworthy traces differ under a reviewed typed policy. Process crashes,
timeouts, sanitizer reports, malformed output, wrong provenance, unsupported
schemas, output limits, and reset failures remain `HarnessFailure` outcomes.

The primary diagnostic stops at the first divergent checkpoint, phase, semantic
path, and mismatch kind while retaining adjacent identities and exact-bit plus
human-readable float evidence. A failure signature is stable across reduction;
a candidate that changes the first divergence is a different failure.

Ordering is never globally normalized. Only payloads explicitly typed as
unordered use stable set or multiset canonicalization. NaN, infinities, and
signed zero follow explicit field policy and are never silently normalized.

## Reviewed artifact boundary

Protocol fixtures, named scenarios, and reviewed comparison evidence are
different artifact classes:

1. `protocol/fixtures` proves strict transport acceptance and rejection.
1. `scenarios/phase-02` owns hand-reviewed semantic input.
1. `reference/artifacts` and `scenarios/regressions` own provenance-bound traces
   and minimized same-signature regressions.

Generation writes only below `target/differential/staging`. The lifecycle
replays and validates the candidate, renders a reviewable diff, binds explicit
review metadata, and promotes through a confined no-clobber atomic rename.
`reference/artifacts/manifest.toml` records content/request/scenario/payload
hashes, all four versions, tolerance identity, oracle/adapter/build identity,
compiler/target/flags, source or seed, notices, and review status. Checks are
read-only and never regenerate or hand-mutate golden data.

## Contributor orchestration

`tools/xtask` is a private imperative shell. Its `differential` commands accept
only registered scenarios, presets, session profiles, and lifecycle shapes,
verify upstream identity for oracle-dependent work, and invoke the runner with
canonical structured arguments. `cargo xtask docs check`, protocol fixture
tests, provenance checks, and package verification enforce the checked-in
contracts without generating them.

The root `justfile` remains a transparent discoverability facade. Each
differential or lifecycle recipe is one visible `cargo xtask` command; it owns
no validation, loops, retry policy, evidence mutation, or C++ logic.

## Enforced invariants

- `liquidfun` stays the sole published default member and has no harness,
  serialization, C++, reference-data, or build-script leakage.
- Protocol and comparison decisions live in typed functional cores; process,
  filesystem, Cargo, CMake, and Git effects stay in thin imperative shells.
- One bounded supervisor owns one-shot, reuse, and sanitizer execution with one
  request in flight, reset proof, poison, kill, wait, and reap semantics.
- Exact bits preserve transport fidelity; only reviewed typed field policy may
  permit numeric tolerance.
- Solver-visible and callback/destruction order is preserved; there is no global
  sort or generic JSON-path comparison policy.
- C++ remains process-isolated, read-only, and external to all consumer paths;
  there is no FFI or runtime delegation.
- Artifact checks are read-only. Stage, replay, diff, explicit review, and atomic
  promote are the only accepted mutation path.
- Phase 2 proves the empty-world harness seam; Phase 3 proves the public object
  contract and private particle-storage architecture only. Broad physics
  behavior, full particle APIs, and final tolerance policy remain later work.
