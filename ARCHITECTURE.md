# Architecture

## Current status

Phase 2 establishes the permanent semantic-comparison seam and proves one
bounded empty-world scenario through native Rust and the pinned,
process-isolated C++ oracle. Phase 3 adds a public native-Rust object-model
foundation plus a private representative particle-storage spike. It proves
identity, invalidation, destruction, callback, step-lifecycle, association, and
storage-remapping contracts; it does not implement rigid-body or particle
solver behavior, broad parity, or final subsystem tolerances. The publishable
`liquidfun` crate therefore remains version `0.0.0`, and the
[compatibility inventory](COMPATIBILITY.md) remains the authority for maturity.

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
formatter or global epsilon from silently changing compatibility evidence.

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
- `world/step.rs` owns the representative no-solver step lock, restricted hook
  calls, bounded event and command collection, command application, and poison
  state.
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
preserve creation/occurrence order. Owned snapshots capture the required
pre-invalidation adjacency, owner, group, and diagnostic state and remain usable
after slot reuse. Direct group destruction clears membership without destroying
its particles.

## Transient contacts, restricted hooks, and step order

Contacts have no durable public identity. Callers supply owned semantic
`ContactSnapshot` values to the representative step; hooks receive only a
borrow-scoped read-only `ContactView`, and polling consumers receive owned
`ContactEvent` values. Rust lifetimes prevent retaining an internal contact
view, and hook trait signatures provide no `&mut World`.

The no-solver Phase-3 step follows one enforceable sequence:

1. Reject a poisoned or already locked world, then acquire the RAII step lock.
2. Validate each supplied fixture pair in caller order and invoke collision
   filtering, optional pre-solve control, observation, and optional command
   request while locked.
3. Preserve every owned event in exact occurrence order and multiplicity. An
   ignored collision skips later hooks for that occurrence.
4. Restore the lock before applying any command.
5. Apply bounded typed commands sequentially in request order, revalidating
   every operand at application time. A stale or foreign operand becomes that
   command's owned failure and does not suppress later applications.
6. Return one owned `StepReport` containing events, destruction records, and
   per-command results in their documented orders.

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
