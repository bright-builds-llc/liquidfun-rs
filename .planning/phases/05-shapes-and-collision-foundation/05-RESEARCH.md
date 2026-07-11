---
generated_by: gsd-phase-researcher
lifecycle_mode: yolo
phase_lifecycle_id: 5-2026-07-11T14-53-25
generated_at: 2026-07-11T15:12:00.000Z
---

# Phase 5: Shapes and Collision Foundation - Research

## Research Question

What must the planner know to implement and verify the pinned LiquidFun shape and collision substrate without leaking C++ layout, invalid state, or accidental ordering promises into the Rust API?

## Executive Recommendation

Implement Phase 5 as one deep `liquidfun::collision` module in dependency order, not as parallel translations of upstream files. The safe public surface should consist of invariant-bearing owned shapes, semantic query/manifold values, a checked distance cache, a generic dynamic tree with opaque proxy identity, pure filtering, and a checked TOI result. Source-faithful kernels, allocation coordinates, packed-key translation, and diagnostic traces remain private.

Use the existing Phase 2 protocol and Phase 4 evidence path for one closed collision-probe extension. Differential work must be planned alongside each kernel rather than postponed to one final integration task: exact branch, identity, and order evidence is what prevents later rigid-world work from hardening a subtly wrong substrate.

Recommended dependency order:

1. Collision domain types and invariant-bearing shapes.
2. AABB, unary shape queries, mass properties, and ray casting.
3. Shape-child distance proxies, GJK, overlap, and cache semantics.
4. Clipping, manifolds, feature identity, and supported pair dispatch.
5. Dynamic tree, broad phase, pure filtering, and refilter/touch behavior.
6. TOI and separation/root-finding diagnostics.
7. Closed protocol/oracle/comparator operations, evidence corpus, docs, and sign-off.

## Existing Foundations to Reuse

### Production Rust

- `crates/liquidfun/src/math.rs` and `crates/liquidfun/src/math/` already provide source-ordered `Vec2`, `Rotation`, `Transform`, `Sweep`, matrices, scalar helpers, and exact settings.
- `crates/liquidfun/src/error.rs` establishes the crate's hand-written non-exhaustive typed-error pattern.
- `crates/liquidfun/src/lib.rs` forbids unsafe code and curates the public API.
- `crates/liquidfun/src/arena.rs` is a precedent for generation retirement and stale-ID rejection, but the tree needs a separate pool because private node coordinates participate in pinned ordering.
- `crates/liquidfun/src/world/` provides the future fixture/contact seams. Phase 5 must not import world ownership into pure collision kernels.

### Differential Infrastructure

- `crates/liquidfun-test-protocol` supplies bounded, versioned requests, exact float-bit transport, provenance, and closed tolerance policies.
- `crates/liquidfun-differential` supplies first-divergence comparison, typed failures, set/multiset canonicalization, replay, and Phase 4 evidence aggregation.
- `tools/reference/src/math_probe.*` supplies the external C++ probe pattern; extend this adapter instead of adding another executable or protocol.
- `tools/xtask` supplies registered command shapes and fail-closed local evidence entrypoints.

## Recommended Production Architecture

Use `crates/liquidfun/src/collision.rs` as the curated entrypoint with cohesive children under `crates/liquidfun/src/collision/`. A likely decomposition is:

- `types.rs`: AABB, ray input/hit/control, mass data, child index, feature identity, manifold values, typed errors.
- `shape.rs` plus `shape/`: concrete owned shapes, `Shape`, construction, validation, unary queries.
- `distance.rs` plus `distance/`: shape-child proxy, simplex/cache, source-ordered GJK, overlap.
- `narrow.rs` plus `narrow/`: clipping, manifold point state, pair dispatcher, circle/polygon/edge kernels.
- `tree.rs` plus `tree/`: opaque `ProxyId`, private node pool, balancing, query/ray traversal, metrics.
- `broad_phase.rs`: move/touch buffers, exact candidate sort/dedup, pure filter/refilter registry.
- `toi.rs` plus `toi/`: checked input/output, separation functions, root finding, bounded diagnostics.

This is a planning guide, not a requirement to create every file immediately. Split only when responsibilities or repository length triggers justify it.

## Shape Research

### Safe public contract

- Public concrete value types plus an exhaustive `Shape` enum are preferable to trait objects. Fixtures and particle-group creation will need owned heterogeneous shape values, while collision dispatch is a closed pinned set.
- All fields remain private. Derived polygon normals, centroid, topology, and chain adjacency cannot be allowed to become stale through field mutation.
- Constructors reject non-finite input and unsupported degeneracy before core kernels run. Do not reproduce upstream debug-assert/release-fallback differences as public Rust behavior.
- Ordinary `Clone` is a deep value clone. Do not reproduce allocator-owned virtual cloning.

### Pinned behaviors that require focused probes

- Polygon construction preserves the upstream weld test, rightmost start, gift-wrapping order, strict collinearity/farthest-point branch, normals, centroid, and mass expression order.
- The pinned weld test compares squared distance with `0.5 * LINEAR_SLOP`; do not replace it with a dimensionally cleaner squared-slop expression without evidence.
- Upstream truncation above `MAX_POLYGON_VERTICES` and fallback to `SetAsBox(1,1)` for a failed hull are invalid-input behaviors. The safe Rust API should reject these and record the difference.
- Open chains have optional previous/next ghost vertices. Closed loops preserve semantic vertices once and derive the closing child edge; the duplicated upstream storage vertex is not public state.
- Edge/chain point tests remain false. Ray casts beginning inside a shape preserve the pinned no-hit behavior.
- Audit the circle-center distance case explicitly. Do not let a broad NaN policy conceal a deliberate public-safe behavior choice.

## Distance and Overlap Research

- Use a single source-ordered GJK path over a validated shape-child proxy. Pair-specific replacements would lose cache semantics and create avoidable divergence.
- Support indices use strict `>` replacement, retaining the first vertex on ties.
- `DistanceCache::empty()` creates valid cold state. A call returns the updated cache and semantic snapshot: count, ordered `(index_a, index_b)` pairs, and metric.
- Cache validation must reproduce the pinned ratio window `[0.5, 2.0]` and `metric < EPSILON` flush rule while still rejecting or explicitly resetting cross-topology reuse before unchecked indexing.
- Overlap uses the pinned strict predicate `distance < 10.0 * EPSILON`; plan witnesses on both sides of the threshold.
- Exact evidence includes simplex size, support indices/order, duplicate-support and termination branch, iteration count, radii mode, and overlap predicate. Witness points, metric, and distance use named field policies.

## Clipping, Manifold, and Pair Research

- Keep clipping as a private pure function with a dedicated probe operation. It retains inside inputs in input order and appends a crossing vertex afterward.
- Replace the packed `b2ContactFeature`/union key with semantic `{index_a, index_b, kind_a, kind_b}` fields. The packed integer is endianness/layout-shaped evidence, not portable identity.
- Manifold type, active point count, feature identity, point order, reference-face choice, flip/orientation, and add/persist/remove states are exact ordered observables.
- Empty/inactive upstream manifold fields and Phase 5 impulses are omitted. Collision kernels do not initialize solver impulses; Phase 6 owns warm-start values.
- The closed supported registry is circle-circle, polygon-circle, polygon-polygon, edge-circle, edge-polygon, chain-child-circle, and chain-child-polygon. Reversed inputs canonicalize like `b2Contact::Create` while retaining explicit orientation.
- Polygon reference-face selection has hysteresis around `0.1 * LINEAR_SLOP`; plan boundary witnesses.
- Edge/chain collision needs isolated, convex-adjacent, concave-adjacent, front, back, and reversal cases.

## Dynamic Tree and Broad-Phase Research

### Identity boundary

- `DynamicTree<T>` may be public and independently useful, but `ProxyId` must be tree-scoped and generational with no raw constructor, raw slot, stable serialization, or `Ord` promise.
- The internal node coordinate is separate from public identity because node allocation and pair order are behavioral oracle facts.
- Do not reuse world `BodyId`/`FixtureId` or the world arena unchanged.

### Pinned ordering and lifecycle

- Match node-pool/free-list reuse and every source-visible tie branch. Equal insertion costs descend to child 2; equal rotation heights choose the second grandchild.
- Query/ray traversal pushes child 1 and then child 2, making the LIFO stack visit child 2 first.
- Creation fattens by `AABB_EXTENSION`. Movement is a no-op when the old fat AABB contains the new tight AABB; otherwise extend and add sign-directed `AABB_MULTIPLIER * displacement` before reinsertion.
- Duplicate move/touch entries are allowed. Destruction tombstones every occurrence. Updating queries every live occurrence is intentional; final sort/dedup removes duplicate pairs.
- Broad-phase pairs sort by private `(min_node_slot, max_node_slot)` and deduplicate adjacent equals. This sequence feeds future contact creation and is exact ordered evidence.
- Ordinary collect-all AABB query and ray results remain unspecified-order consumer results and compare as unique sets. Do not accidentally publish traversal order.

### Query and filtering APIs

- Use borrow-scoped query and ray visitors with closed controls. Validate clip fractions as finite and inside the current interval.
- Collection helpers may allocate and collect results, but must not become the authoritative hot-path or ordering contract.
- Implement the exact filter defaults and rule: category `0x0001`, mask `0xffff`, group `0`; equal nonzero group overrides masks, positive collides, negative does not, otherwise both symmetric intersections must be nonzero.
- Phase 5 proves dirty/touch/reconsideration and duplicate suppression. World contact creation, persistence, removal, wake state, joint suppression, and listener timing remain Phase 6 or later.

## Time-of-Impact Research

- Public input consists of two validated shape-child proxies, checked sweeps, and finite `t_max` in `0.0..=1.0`. Copy and normalize sweeps internally.
- Public output is a closed state plus time. Iteration counts, support indices, separation kind, root method, branch path, and termination cause are bounded private diagnostics.
- Preserve the pinned target `max(LINEAR_SLOP, total_radius - 3 * LINEAR_SLOP)` and tolerance `0.25 * LINEAR_SLOP`.
- Preserve the outer cap 20, root cap 50, push-back cap `MAX_POLYGON_VERTICES`, bisection-first/secant-alternating root sequence, and strict support ties.
- Exclude wall-clock TOI globals from compatibility evidence.
- Required witnesses include initial overlap, touching at zero, separated through `t_max`, translation impact, rotation, nearly tangent motion, zero relative translation with rotation, chain/edge children, support ties, large-angle normalization, and every reachable cap/failure path. Reuse the pinned `TimeOfImpact.h` large-angle witness.

## Differential Contract

Extend the existing request/trace schema with a closed Phase 5 collision-probe family. Each request must remain bounded and use semantic IDs, child indices, exact float bits, pinned build identity, and a declared operation. Suggested operations are:

- shape construction and unary query;
- distance/overlap with cold or supplied cache;
- clipping;
- supported manifold collision and feature-state transition;
- tree lifecycle/query/ray/metrics;
- broad-phase move/touch/pair/filter/refilter;
- TOI with bounded diagnostic trace.

Keep stdout protocol-only and stderr diagnostic-only. Unknown operations, malformed shape state, invalid child indices, resource-limit overflow, wrong build identity, and nonzero/crash/timeout outcomes remain harness failures, distinct from physics mismatches.

### Comparison policy

- Exact: type/state tags, child indices, feature identities, point/cache/pair order, support indices, branch/termination causes, iteration counts, and collection policy.
- Exact-bit first: all transported floats in canonical D1 probes.
- Named ULP policy: bounded scalar kernels only after canonical evidence proves exact bits are not stable.
- Named absolute or absolute-relative policy: dimensioned transformed geometry only after evidence.
- Set: unique ordinary query/ray results whose callback order is not a public promise.
- Arithmetic NaN is a mismatch; signed zero remains distinct unless a specific reviewed field policy says otherwise.
- Never use a global epsilon, runtime-adaptive widening, or iteration-count-scaled tolerance.

## Recommended Plan Decomposition

### Wave 1: Public domain and shapes

- Plan 05-01: collision domain values, AABB/ray/mass types, errors, public module seam.
- Plan 05-02: circle/edge/polygon/chain construction, topology, clone, unary queries, focused unit/property tests.

These plans may overlap on shared entrypoints; the planner should either keep them sequential or assign exclusive file ownership.

### Wave 2: Pure narrow-phase foundations

- Plan 05-03: proxy/cache/simplex/GJK/overlap.
- Plan 05-04: clipping, manifold identity, supported pair kernels and transitions.

Plan 05-04 depends on shapes and distance primitives. Parallelism is safe only if file ownership is separated explicitly.

### Wave 3: Spatial acceleration and continuous collision

- Plan 05-05: dynamic tree, broad phase, visitors, metrics, filtering/refiltering.
- Plan 05-06: TOI kernels and bounded diagnostics.

Both depend on earlier collision values; TOI depends directly on distance/proxy behavior. They can be parallel only with disjoint files and stable shared APIs.

### Wave 4: Cross-language evidence and sign-off

- Plan 05-07: protocol schema/types, Rust adapter/comparator, C++ probe, fixed corpus, D0 replay, registered xtask verification.
- Plan 05-08: compatibility inventory, testing/architecture/API docs, evidence report, truthful requirement disposition, full verification.

The planner may split evidence by subsystem if one plan would exceed file/function readability or execution-context limits. Every production plan should add its native unit/property witnesses before the final cross-language wave.

## Risks and Mitigations

| Risk | Consequence | Mitigation |
| --- | --- | --- |
| Translating a newer Box2D implementation | Subtle branch/order/API drift | Read only the pinned sources and record exact line-level quirks in tasks/tests |
| Parallel plans share collision entry files | Merge conflicts or inconsistent public types | Assign waves and exclusive `files_modified`; serialize overlapping plans |
| Invalid-input compatibility leaks into public API | Build-mode differences and invalid physics state | Fallible constructors plus named compatibility differences |
| Packed C++ keys/raw node IDs become evidence | Nonportable identity and unsafe API commitments | Semantic feature types and private node coordinates |
| Query order is accidentally documented | Future optimization becomes breaking | Ordered only for pair/contact-feeding paths; set policy for ordinary queries |
| Differential evidence postponed | Wrong kernels become dependencies | Add unit/property probes per kernel and cross-language operations before sign-off |
| Full `COLL-05` marked complete too early | False parity claim | Record Phase 5 filter/refilter portion; leave world contacts Phase 6 |
| Broad numeric epsilon hides branch defects | Later solver divergence | Exact discrete evidence plus closed field-specific policies |

## Validation Architecture

### Fast feedback per task

- Focused unit tests in the touched collision module.
- Public integration tests for constructibility, typed errors, checked child/proxy identities, and ordering promises.
- Property tests for hull invariants, AABB containment, GJK symmetry where upstream semantics permit, tree consistency, pair deduplication, and bounded query correctness.
- `cargo fmt --all`, targeted `cargo test -p liquidfun`, and targeted clippy while iterating.

### Plan completion gates

- Every task has a directly named test command and exact acceptance criteria.
- Tests use Arrange, Act, Assert and cover one concern each.
- Source-branch witnesses cover equality, strictness, and cap boundaries, not only nominal examples.
- No new unsafe code or production dependency.
- Consumer Cargo-only build/package paths remain independent from C++.

### Phase verification gates

Run repository-native verification plus the mandatory Rust sequence:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`

Also require the closed Phase 5 xtask/oracle commands created by the plans, fixed debug/release collision-probe corpus, two-run D0 byte identity, package isolation, docs checks, and compatibility inventory regeneration/check. Canonical promotion remains limited by the Phase 4 D0-D3 authority rules.

## Planning Checklist

- Every `COLL-02` through `COLL-07` requirement appears in plan frontmatter.
- Every locked D-01 through D-22 decision maps to at least one task or verification criterion.
- Plans declare exact pinned upstream files in `read_first`.
- Actions state exact branch/tie/threshold/cap behavior, not “port upstream.”
- Public API and private diagnostics are separated explicitly.
- `files_modified` lists are disjoint inside a parallel wave.
- Threat models cover untrusted geometry/protocol input, resource exhaustion, stale proxy/cache identity, and C++ oracle isolation.
- Evidence and documentation claims stop at the verified Phase 5 boundary.

## RESEARCH COMPLETE

The phase is ready for goal-backward planning. No external dependency or new architecture decision is required before planning; the remaining uncertainty is implementation-level source auditing that should be encoded directly into each plan's `read_first`, action, tests, and differential witnesses.
