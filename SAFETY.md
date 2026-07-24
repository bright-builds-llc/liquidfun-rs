# Safety

`liquidfun` is safe Rust throughout its production source. The workspace sets
`unsafe_code = "forbid"`, and the crate repeats `#![forbid(unsafe_code)]` at its
public root. The repository's test-only source scanner strips comments and
string literals before rejecting constructible unsafe blocks, functions,
traits, implementations, extern declarations, and unsafe attributes.

This policy is a checked property of the current source, not a claim that every
possible physics input is useful. Public constructors and mutations still
validate numerical and ownership invariants and return typed errors.

## Identity and invalidation

`World` owns bodies, fixtures, joints, particle systems, particle groups, and
particles. Their public handles are opaque values containing private owner,
slot, and generation identity; particle identities also include their particle
system scope. A handle grants no access without validation by its owning world.

Foreign handles return `HandleError::WrongWorld`, particle-owner mismatches
return `HandleError::WrongParticleSystem`, and destroyed or reused identities
return `HandleError::StaleOrDestroyed`. Generations never wrap into a live
identity: an exhausted slot is retired. Handles contain no pointers and expose
no raw-parts constructor, dense index, serialization, or storage order.

Destruction is centralized in `World`. It validates the root before mutation,
invalidates every removed handle, and returns owned `DestructionRecord` values
whose snapshots remain usable after invalidation and slot reuse.

## Contacts and callbacks

Contacts are private, transient world state. Hooks receive only borrow-scoped,
read-only contact and particle-contact views. The public API has no durable
contact handle and no callback receives `&mut World`.

Collision and pre-solve hooks return closed typed directives. Step hooks may
request only the bounded typed `WorldCommand` surface. Owned `StepReport`,
contact-transition, solve, lifecycle, and destruction values are the supported
way to retain callback results after a step.

## World locking and deferred commands

`World::step` acquires an internal RAII lock before contact discovery and
callback execution. Re-entrant mutation is unavailable through hook
signatures. Commands are collected under the lock, then applied sequentially
after unlock and revalidated at application time. A stale or foreign command
operand is reported for that command without concealing later results.

Event, command, and continuous-work limits are explicit and bounded. A limit
failure discards unapplied commands or preserves a coherent private continuous
resume point according to the documented `StepError` variant.

## Application user data

Application values do not enter physics storage. `AssociationMap<Id, T>` is an
application-owned, typed side table sealed to one exact handle kind. Consumers
clean it up explicitly from owned destruction records. The crate does not use
`Any`, raw pointers, lifetime-long world borrows, or implicit callbacks into
application memory.

## Owned particle buffers

The external-buffer equivalent uses `ParticleBufferLanes` and
`ParticleBufferBundle`, never borrowed or raw foreign memory. Adoption consumes
all supplied vectors, validates lane lengths and declared capacity before
mutation, and returns ownership on failure. Fixed bundles reject growth beyond
their declared capacity. Destroying a system through
`World::destroy_particle_system_with_buffers` returns the complete owned lane
bundle.

No alias to a supplied lane can survive safe ownership transfer. Dense particle
positions and derived solver lanes remain private.

## Owned events and observations

Destruction reports, step reports, world observations, profiles, and debug-draw
primitives own their semantic data. They preserve documented order and
multiplicity but carry no pointers into world storage. Query and ray-cast
visitors receive borrow-scoped occurrences; their visitation order is
unspecified unless a narrower API contract says otherwise.

## Renderer and oracle isolation

The published `liquidfun` crate is headless and contains no renderer, window,
GPU, C++, FFI, protocol, or process dependency. Renderer adapters consume owned
observations and debug primitives outside the engine. The pinned C++ LiquidFun
tree is a private out-of-process development oracle; it never executes as part
of ordinary library use.

## Panic policy

Recoverable validation, capacity, ownership, and configuration failures use
typed `Result` values. Public APIs do not use panics as normal error handling.

If user hook code panics, the step lock is restored through RAII, queued
commands are discarded, the world is marked poisoned, and the original panic
resumes. Diagnostic lock, poison, and handle-liveness queries remain available;
later coherent-state mutation fails explicitly rather than treating partially
progressed state as valid.

Introducing unsafe code, SIMD, parallel stepping, or another structural fast
path requires a separate reviewed architecture decision, a safe behavioral
baseline, a narrow documented invariant, focused tests, measured need, and all
compatibility gates. The current release contract does not weaken the
prohibition.
