---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 5-2026-07-11T14-53-25
generated_at: 2026-07-11T15:00:40.038Z
---

# Phase 5: Shapes and Collision Foundation - Context

**Gathered:** 2026-07-11
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Implement the complete native-Rust shape and collision substrate required by later rigid worlds and particle-body coupling: invariant-bearing circle, edge, polygon, and chain shapes; unary shape queries; distance, overlap, clipping, manifolds, and supported shape-pair collision; dynamic tree and broad-phase proxy behavior; pure filtering/refiltering semantics; and time-of-impact kernels. Prove these with focused unit, property, and bounded differential evidence against the pinned LiquidFun revision. World-owned body/fixture/contact lifecycle, warm-start impulses, listener timing, and rigid stepping remain Phase 6 or later.

</domain>

<decisions>
## Implementation Decisions

### Shape values, validation, and ownership

- **D-01:** Expose public `CircleShape`, `EdgeShape`, `PolygonShape`, and `ChainShape` value types plus an exhaustive public `Shape` enum with static internal dispatch. Do not use public trait objects, raw layout access, or a separate published collision crate.
- **D-02:** Keep shape representation private and immutable after construction. Shapes own their data, ordinary `Clone` is deep, chains own canonical vertices exactly once, and future fixtures take immutable shape snapshots by value.
- **D-03:** Use fallible constructors and typed non-exhaustive errors so malformed geometry cannot enter collision storage. Require finite inputs and reject unsupported degeneracy instead of reproducing upstream debug assertions, truncation, or fallback-box behavior.
- **D-04:** Preserve pinned algorithms, expression grouping, support selection, weld tests, hull order, normals, centroid, and mass behavior for every accepted input. Any deliberate safe-Rust difference for invalid or pathological input must be named, probed, and documented rather than hidden by a broad tolerance.
- **D-05:** Model open and closed chains explicitly. Ghost vertices apply only to open chains; child indices are checked; `child_edge` returns an owned adjacency-bearing edge; loops do not expose a duplicated closing vertex.
- **D-06:** Use owned typed query values such as `Aabb`, `MassData`, `RayCastInput`, and `RayCastHit`. Ray casts return a checked optional hit; edge/chain point tests remain false; ray-start-inside behavior follows upstream. Explicitly research and record the exact circle-center distance normal rather than silently leaking or normalizing arithmetic NaN.

### Distance, clipping, manifolds, and pair dispatch

- **D-07:** Implement one source-ordered GJK distance path over validated shape-child proxies, with radii enabled or disabled explicitly. Overlap uses the pinned strict `distance < 10.0 * EPSILON` predicate; tests straddle that threshold.
- **D-08:** Represent reusable distance state as an initialized `DistanceCache`, never raw unchecked arrays. Return the updated cache and expose a semantic snapshot of count, ordered support-index pairs, and metric. Invalid cross-topology reuse fails or resets explicitly.
- **D-09:** Keep clipping as a private pure kernel with a dedicated differential operation. Preserve retained input order, append crossing vertices in pinned order, compare output count/order/feature identity exactly, and compare coordinates under a named numeric policy.
- **D-10:** Represent contact features semantically as typed vertex/face identities. Never serialize or compare the packed C++ union key. Manifold type, point count, point order, reference-face choice, flip/orientation, and add/persist/remove identity states are exact ordered evidence; inactive or uninitialized fields are omitted.
- **D-11:** Use a closed supported-pair registry with canonical primary ordering and explicit reversed-input orientation. Supported manifold pairs are circle-circle, polygon-circle, polygon-polygon, edge-circle, edge-polygon, chain-child-circle, and chain-child-polygon. Distinguish unsupported, separated, and touching outcomes.
- **D-12:** Keep Phase 5 manifold evidence free of solver impulses. Feature-based point persistence may be proven as a pure state transition, while warm-start impulse semantics belong to Phase 6.

### Dynamic tree, broad phase, and filtering

- **D-13:** Expose a cohesive public `DynamicTree<T>` with opaque, tree-scoped, generational `ProxyId` values. Public IDs expose no raw slot, constructor, serialization, or ordering; a separate private node coordinate preserves pinned allocation and ordering behavior.
- **D-14:** Reproduce the pinned node pool/free list, fat-AABB creation and movement, move/touch buffer behavior, insertion/rotation tie branches, and LIFO traversal details. Pair candidates sort and deduplicate by private `(min_node_slot, max_node_slot)` exactly as upstream.
- **D-15:** Treat pair callbacks and any sequence feeding future contact creation as exact `Ordered` evidence. Ordinary collect-all AABB query and ray results have unspecified consumer order and compare as unique sets; raw traversal order may appear only as diagnostic topology evidence.
- **D-16:** Use typed borrow-scoped query and ray visitors with explicit continue, stop, ignore, terminate, and validated clip controls. Owned collection helpers may wrap the visitor API but must not create an order promise.
- **D-17:** Implement the pure upstream filter rule and broad-phase refilter/touch behavior in Phase 5, including duplicate suppression and reconsideration of newly eligible pairs. Actual contact-manager insertion, persistence, destruction, waking, joint suppression, and callbacks remain Phase 6 or later; compatibility reporting must not overstate the completed portion of `COLL-05`.

### Time of impact and evidence policy

- **D-18:** Provide checked TOI input from two validated shape-child proxies, two checked sweeps, and finite `t_max` in `0.0..=1.0`. Return a closed state (`Overlapped`, `Touching`, `Separated`, or `Failed`) plus time; normalize copied sweeps internally without mutating callers.
- **D-19:** Preserve the pinned TOI kernel exactly, including target/tolerance formulas, iteration caps, bisection/secant alternation, push-back cap, support ties, and termination branches. Wall-clock globals are not compatibility observables.
- **D-20:** Build one closed Phase 5 collision-probe extension on the existing semantic protocol and C++ adapter. Do not create a second oracle path. Persist first-divergence, replay, exact build identity, and D0 two-run byte-identity behavior.
- **D-21:** Compare discrete structure, feature identity, cache indices, order, branch choices, termination causes, and iteration counts exactly. Transport all floats by exact bits, start canonical D1 probes with exact-bit comparison, and introduce a named ULP or dimensioned absolute/relative policy only from demonstrated canonical evidence. Never use a global epsilon or widen tolerance by iteration count.
- **D-22:** Cover cold/warm/invalidated distance caches, all simplex sizes and support ties, every supported manifold pair and reversal, edge adjacency, feature transitions, tree ties and lifecycle, refilter/touch, query/ray controls, and TOI overlap/touch/separation/rotation/tangent/cap witnesses before sign-off.

### the agent's Discretion

- Exact internal file decomposition below `collision.rs`, provided the module remains cohesive and files/functions stay within repository readability triggers.
- Concrete names for private kernels and bounded diagnostic record types.
- Whether collection convenience helpers are added in Phase 5, provided the borrow-scoped visitor surface and ordering contracts remain authoritative.
- The precise typed error variant split, provided invalid states remain unrepresentable and errors do not expose private storage details.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and locked architecture

- `.planning/ROADMAP.md` § Phase 5 — fixed boundary, requirements, success criteria, and research flags.
- `.planning/REQUIREMENTS.md` § `COLL-02` through `COLL-07` — shape, narrow-phase, broad-phase, filter/refilter, TOI, and evidence requirements.
- `.planning/phases/02-semantic-protocol-and-oracle-round-trip/02-CONTEXT.md` — closed protocol, exact identity, ordering, replay, and failure-classification decisions.
- `.planning/phases/03-rust-object-model-and-storage-architecture/03-CONTEXT.md` — opaque scoped identity, invalidation, ownership, transient contacts, and no raw storage exposure.
- `.planning/phases/04-math-settings-and-numerical-policy/04-CONTEXT.md` — source-ordered math, exact settings, non-finite/signed-zero rules, collection policies, horizons, and D0-D3 evidence authority.
- `.planning/research/ARCHITECTURE.md` — deep collision module, dependency direction, pure kernels, and native-Rust boundary.
- `docs/decisions/0001-oracle-selection.md` — immutable upstream revision and selection rationale.
- `reference/upstream-lock.toml` — machine-readable pinned oracle identity.

### Pinned upstream shapes and collision

- `third_party/liquidfun/liquidfun/Box2D/Box2D/Collision/Shapes/` — canonical circle, edge, polygon, chain, and base-shape behavior.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Collision/b2Collision.h` and `b2Collision.cpp` — AABB, clipping, manifolds, feature identity, and overlap behavior.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Collision/b2Distance.h` and `b2Distance.cpp` — proxy, simplex cache, GJK, witness, and overlap semantics.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Collision/b2CollideCircle.cpp` — circle pair and polygon-circle manifolds.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Collision/b2CollidePolygon.cpp` — polygon-polygon separation, clipping, hysteresis, identity, and point order.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Collision/b2CollideEdge.cpp` — adjacency-aware edge/chain child collision behavior.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/Contacts/b2Contact.cpp` — supported pair registration, input reversal, and feature-state transition semantics.

### Pinned upstream broad phase and TOI

- `third_party/liquidfun/liquidfun/Box2D/Box2D/Collision/b2DynamicTree.h` and `b2DynamicTree.cpp` — proxy/node lifecycle, insertion, balancing, query/ray traversal, metrics, and exact ties.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Collision/b2BroadPhase.h` and `b2BroadPhase.cpp` — move/touch buffering, pair sorting/deduplication, proxy lifecycle, and metrics.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Collision/b2TimeOfImpact.h` and `b2TimeOfImpact.cpp` — checked input model, separation functions, root finding, caps, and termination states.
- `third_party/liquidfun/liquidfun/Box2D/Testbed/Tests/DynamicTreeTest.h`, `RayCast.h`, `CollisionFiltering.h`, and `TimeOfImpact.h` — required upstream witness families.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2Fixture.cpp`, `b2ContactManager.cpp`, and `b2WorldCallbacks.cpp` — source boundary between Phase 5 filtering/refiltering and Phase 6 contact lifecycle.

### Existing Rust evidence and standards

- `crates/liquidfun/src/math.rs` and `crates/liquidfun/src/math/` — Phase 4 math, sweep, transform, settings, and exact operation-order foundation.
- `crates/liquidfun-test-protocol/src/tolerance/policy.rs` — closed field/collection/horizon/non-finite comparison policy.
- `crates/liquidfun-differential/src/canonical.rs`, `comparator.rs`, and `phase4_evidence.rs` — canonicalization, typed comparison, and bounded phase-local evidence patterns.
- `tools/reference/src/math_probe.hpp` and `math_probe.cpp` — existing pinned C++ probe seam to extend.
- `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/testing.md`, `standards/core/verification.md`, and `standards/languages/rust.md` — required architecture, code-shape, test, verification, and Rust rules.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `crates/liquidfun/src/math/`: `Vec2`, `Rotation`, `Transform`, `Sweep`, scalar helpers, and exact collision/TOI settings already establish the required `f32` vocabulary.
- `crates/liquidfun/src/error.rs`: existing hand-written, non-exhaustive typed error pattern.
- `crates/liquidfun-test-protocol`: exact float-bit transport, bounded schemas, build identity, and closed comparison policy.
- `crates/liquidfun-differential`: first-divergence comparison, set/multiset canonicalization, D0 replay, and Phase 4 evidence-report structure.
- `tools/reference/src/math_probe.*`: external C++ probe pattern with stdout protocol discipline and build provenance.
- `crates/liquidfun/src/world/`: Phase 3 scoped handles, arena precedent, transient contact view, and collision hook integration points.

### Established Patterns

- Production behavior remains safe native Rust in one publishable crate; private tooling and the pinned C++ oracle depend inward and never shape consumer builds.
- Public inputs are parsed into invariant-bearing types, implementation storage remains private, and ordered versus set-like observables are declared per field.
- Cross-language evidence is closed, bounded, reproducible, bit-preserving, provenance-checked, and truthful about D0-D3 authority.

### Integration Points

- Add the production seam as `crates/liquidfun/src/collision.rs` plus `crates/liquidfun/src/collision/` children and curate exports through `crates/liquidfun/src/lib.rs`.
- Extend the existing semantic protocol, differential runner, xtask commands, and `tools/reference` C++ adapter with collision-specific bounded operations and evidence rather than adding another harness.
- Phase 6 consumes immutable `Shape` snapshots, pure filter rules, ordered broad-phase pairs, and manifold feature identities without changing the public Phase 3 object model.

</code_context>

<specifics>
## Specific Ideas

- Preserve source-level quirks that affect valid behavior, including exact polygon weld/tie logic, GJK cache-flush thresholds, polygon reference-face hysteresis, tree equal-cost branches, LIFO traversal, pair sorting by private node coordinates, and TOI alternating root method.
- Keep semantic evidence portable: typed feature identity replaces packed C++ keys, while private diagnostic traces may retain exact support indices and branch sequences.
- Any safe public departure from invalid-input or arithmetic-NaN upstream behavior is a named compatibility difference with a focused regression and probe, never an undocumented cleanup.

</specifics>

<deferred>
## Deferred Ideas

- World contact creation, persistence, destruction, awake-state effects, joint collision suppression, warm-start impulses, and listener timing — Phase 6 and later.
- Fixture-side topology mutation or mutable shape editing — deferred until an explicit world/update contract proves it is needed.
- Public simplex, separation-function, packed manifold-key, tree-node-slot, and TOI iteration APIs — intentionally excluded as private implementation/diagnostic state.
- Independent collision crate or `no_std` extraction — only after a later allocation, API, and independent-consumer audit.

</deferred>

*Phase: 05-shapes-and-collision-foundation*
*Context gathered: 2026-07-11*
