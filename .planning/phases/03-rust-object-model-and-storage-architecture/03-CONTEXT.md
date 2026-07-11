---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 3-2026-07-11T01-23-59
generated_at: 2026-07-11T01:28:27.592Z
---

# Phase 3: Rust Object Model and Storage Architecture - Context

**Gathered:** 2026-07-10
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Prove the safe native-Rust object and storage architecture that later physics subsystems will depend on: typed world-scoped identity, explicit invalidation and destruction cascades, transient contact access, restricted step hooks, deferred mutation, owned events, safe user associations, and a representative dense-storage permutation spike. This phase establishes and tests those contracts without implementing broad rigid-body or particle-solver behavior. Full particle bulk-mutation and external-buffer APIs remain Phase 9 work; Phase 3 locks the architecture they must follow.

</domain>

<decisions>
## Implementation Decisions

### Typed identity and invalidation

- **D-01:** Implement private custom generational arenas and distinct opaque public handle types for bodies, fixtures, joints, particle systems, particle groups, and stable particles. Every handle identity includes a checked process-unique world key, a private slot, and a `u64` generation.
- **D-02:** Generation increments are checked. A slot whose generation would overflow is permanently retired; generation wrap may never resurrect an ancient handle. World-key, slot-capacity, and generation exhaustion fail explicitly.
- **D-03:** Lookup distinguishes wrong-world and stale/destroyed failures. Wrong object kinds are compile-time errors for typed APIs; heterogeneous internal paths use an explicit wrong-kind error rather than reinterpretation.
- **D-04:** Equality and hashing cover the complete typed identity. Handles expose no raw constructors, dense indices, serialization contract, or `Ord` guarantee initially. `Debug` may show the handle kind and an opaque diagnostic token, not stable storage layout.
- **D-05:** Integer-only handles are `Send + Sync` because they confer no access. `World` and callback/user-data containers receive auto traits only from their actual fields; no unsafe manual `Send` or `Sync` implementation is permitted.

### Destruction, hooks, events, and mutation

- **D-06:** Centralize destruction in `World`: validate once, execute the pinned-upstream cascade in a documented order, update every adjacency/storage structure transactionally, invalidate affected handles, and return or emit owned destruction records carrying semantic IDs, cause, and required snapshots.
- **D-07:** Contacts are transient internal records exposed only as borrow-scoped views during hooks or as owned snapshots/events. There is no durable public contact handle.
- **D-08:** Preserve synchronous upstream decision points with borrow-scoped read-only hook views and narrow return directives such as collision filtering or pre-solve control. Hooks never receive `&mut World` or retain internal references.
- **D-09:** Collect an owned `StepReport` for polling consumers. Callback and destruction events preserve occurrence order and multiplicity exactly; they are not deduplicated or globally sorted.
- **D-10:** Application mutations requested during hooks become typed commands. Apply them sequentially only after the world unlocks, validating every referenced handle at application time and reporting stale-command failures explicitly.
- **D-11:** Use RAII to restore the lock flag during unwinding. If a hook panics, discard unapplied commands, mark the partially stepped world poisoned, resume the panic, and make later world operations fail explicitly rather than pretending the state is coherent.

### Dense particle storage and stable identity

- **D-12:** Keep particle state in a dense structure-of-arrays layout with group-contiguous ranges. Public `ParticleId` is world- and particle-system-scoped and stable across lane rotations, reorderings, and compaction; private `ParticleIndex` is ephemeral and never crosses the API boundary.
- **D-13:** Maintain `dense_to_id` and `id_to_dense` mappings with explicit `Live`, `PendingDelete`, and `Vacant` identity states. Marking deletion rejects ordinary mutation while retaining enough row state to construct owned destruction information; compaction then invalidates the ID and advances its generation.
- **D-14:** One private authoritative permutation operation must update required and optional lanes, identity maps, proxies, contacts, pairs, triads, lifetime indices, and group ranges as one invariant-preserving transaction. Solver-visible order never depends on hash-map iteration.
- **D-15:** The Phase 3 spike uses representative required/optional lanes and remappable derived-index fixtures, then property-tests arbitrary create, reorder, mark-delete, compact, stale-access, and capacity-failure sequences. It does not implement particle solver passes.

### User associations and future external buffers

- **D-16:** Provide safe typed application-owned `AssociationMap<Id, T>` side tables keyed by stable handles, with destruction-report cleanup helpers. Do not put `Any`, arbitrary raw pointers, or a user-data generic parameter into `World`; small copyable correlation tags may be stored internally only when a callback contract requires them.
- **D-17:** The future safe external-buffer equivalent accepts ownership of a validated lane bundle at particle-system construction, tracks declared fixed capacity separately from allocation capacity, returns explicit `CapacityExceeded` instead of reallocating beyond the contract, exposes borrow-scoped slices only, and returns owned buffers on teardown. Borrowed lifetime-long raw arrays are not part of the safe API.

### Agent's Discretion

- Exact private module, field, error-variant, event-variant, and directive names within the locked contracts.
- Exact opaque `Debug` token format and initial arena/free-list representation, provided no stable layout or ordering leaks publicly.
- Exact representative lanes, property-test operation weights, and bounded sequence counts used by the Phase 3 spike.
- Whether the association cleanup helper consumes a `StepReport`, an iterator of destruction events, or both, provided cleanup remains explicit and deterministic.

</decisions>

<canonical_refs>

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Scope and acceptance

- `.planning/ROADMAP.md` § Phase 3 — fixed goal, dependency, requirements, success criteria, and research/ADR flags.
- `.planning/PROJECT.md` — native-Rust, safety, deterministic ordering, API, architecture, oracle-isolation, and renderer-independence constraints.
- `.planning/REQUIREMENTS.md` — API-01 through API-08 and DOCS-02 acceptance requirements; API-09 and API-10 remain assigned to Phase 9.
- `.planning/phases/02-semantic-protocol-and-oracle-round-trip/02-CONTEXT.md` — locked process-isolated oracle, semantic-ID, ordering, and adapter-boundary decisions inherited from Phase 2.

### Reconciled architecture research

- `.planning/research/SUMMARY.md` — prescribed Phase 3 risk-retirement spike and object/storage recommendations.
- `.planning/research/ARCHITECTURE.md` — world-scoped identity, transient contacts, restricted hooks, dense particle storage, ownership boundaries, dependency direction, and unresolved design questions.
- `.planning/research/FEATURES.md` — upstream object, callback, particle-handle, and buffer capability inventory.
- `.planning/research/PITFALLS.md` — stale identity, callback mutation, ephemeral indices, group/buffer corruption, ordering, and parity failure modes.

### Pinned upstream behavior

- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2World.cpp` — world locking, step phases, creation/destruction guards, and cascade implementation.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2WorldCallbacks.h` — listener/filter/query contracts and mutation warnings.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Documentation/Programmers-Guide/Chapter09_Contacts.md` — contact callback timing, multiplicity, and mutation restrictions.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Documentation/Programmers-Guide/Chapter10_World.md` — world ownership, locking, query behavior, and destruction semantics.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Documentation/Programmers-Guide/Chapter11_Particles.md` — self-compacting particle indices and buffer behavior.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Particle/b2Particle.h` — upstream particle-handle semantics.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Particle/b2ParticleSystem.cpp` — SoA permutations, compaction, group rotation, and derived-index updates.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Particle/b2ParticleSystem.h` — particle lifecycle and external-buffer surface.

### Existing repository and standards

- `ARCHITECTURE.md` — current dependency direction and the Phase 2 boundary that Phase 3 must extend without leaking harness or C++ concerns into `liquidfun`.
- `crates/liquidfun/src/lib.rs` — safe Cargo-only production scaffold and integration root for the new deep modules.
- `AGENTS.md` and `AGENTS.bright-builds.md` — repository constraints, Rust gates, deep-module guidance, and required workflow.
- `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/testing.md`, `standards/core/verification.md`, and `standards/languages/rust.md` — invariant-bearing domain types, functional-core guidance, focused tests, module shape, and verification requirements.

</canonical_refs>

<code_context>

## Existing Code Insights

### Reusable Assets

- `crates/liquidfun`: the sole publishable/default crate, currently a safe scaffold with `unsafe_code` forbidden; Phase 3 can establish private `handle`, `arena`, `world`, and representative `particle` modules without migration baggage.
- `crates/liquidfun-test-protocol`: existing strict typed boundary and semantic ID vocabulary for test scenarios and traces; adapters may map semantic IDs to engine handles without making protocol IDs public engine identity.
- `crates/liquidfun-differential`: existing ordered trace/report infrastructure that later phase tests can extend with handle, callback, and destruction observables while remaining private.

### Established Patterns

- One deep published crate owns production state; private harness crates depend inward and must not shape the public object model.
- Boundary values are parsed into invariant-bearing types, ordering is explicit, machine-readable evidence is authoritative, and unsafe code is forbidden in the current production crate.
- Cargo-only consumer paths remain independent of the C++ oracle and reference data.

### Integration Points

- Add the object-model and storage modules below `crates/liquidfun/src/` and curate only stable public types through `lib.rs`.
- Extend `ARCHITECTURE.md` with enforceable object, callback, storage, step-order, oracle, and renderer boundaries.
- Use compile-fail/type tests, unit/property tests, and a small semantic differential scenario to prove identity and event behavior without broad solver implementation.

</code_context>

<specifics>
## Specific Ideas

- Prefer a never-resurrect guarantee over accepting a library key's remote wraparound risk; retire exhausted slots even though exhaustion is practically unreachable.
- Treat handles as authority-free identity tokens: they may cross threads, but they never grant access without a validated `World` operation.
- Preserve callback/destruction order and multiplicity as compatibility evidence while giving polling users the same occurrences as owned data.
- Keep stable particle identity and dense solver location deliberately separate, then make the permutation transaction the single auditable invariant boundary.

</specifics>

<deferred>
## Deferred Ideas

- Complete particle bulk-view mutation, external-buffer construction/teardown APIs, and performance sign-off — Phase 9 (API-09 and API-10).
- Exact rigid and particle callback event payloads that require implemented contacts or solvers — Phases 6 through 10; Phase 3 defines their ownership and timing envelope.
- Public handle serialization or cross-process persistence — post-parity unless a later reviewed requirement establishes a stable persistence contract.
- Unsafe raw-pointer buffer interoperability — excluded from the initial safe API and considered only in an explicitly unsafe low-level module after measured need and focused safety review.

</deferred>

***

*Phase: 03-rust-object-model-and-storage-architecture*
*Context gathered: 2026-07-10*
