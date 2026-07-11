# Phase 5: Shapes and Collision Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `05-CONTEXT.md`; this log preserves alternatives considered by the yolo advisor pass.

**Date:** 2026-07-11
**Phase:** 5-shapes-and-collision-foundation
**Mode:** Yolo
**Areas discussed:** Shape values and validation, narrow-phase observables, broad-phase identity and ordering, TOI and differential evidence

## Shape Representation and Ownership

| Option | Description | Selected |
| --- | --- | --- |
| Concrete values plus `Shape` enum | Immutable owned concrete types with exhaustive static dispatch and deep clone | ✓ |
| Public shape trait objects | Heap allocation, dynamic dispatch, object-safe clone, and downcasting | |
| Concrete types only | No heterogeneous public representation until fixture integration | |

**Agent choice:** Concrete values plus `Shape` enum.
**Notes:** Keeps the crate cohesive, fits future fixture snapshots, and avoids exposing allocator or storage details.

## Invalid Geometry Policy

| Option | Description | Selected |
| --- | --- | --- |
| Fallible invariant-bearing constructors | Reject malformed/unsupported input; preserve exact upstream behavior for accepted input | ✓ |
| Reproduce asserts and fallback shapes | Preserve invalid C++ behavior, including build-mode differences and polygon fallback | |
| Permissive values plus `validate()` | Allow temporarily invalid shapes and require repeated checks | |

**Agent choice:** Fallible invariant-bearing constructors.
**Notes:** Invalid-input differences must be explicit compatibility dispositions, not silent changes.

## Distance Cache and Manifold Identity

| Option | Description | Selected |
| --- | --- | --- |
| Typed semantic state | Initialized cache snapshots and typed vertex/face identities with exact order | ✓ |
| Raw arrays and packed keys | Directly mirror implementation layout and unchecked combinations | |
| Hide all state | Compare only final geometry and lose warm-start/first-divergence evidence | |

**Agent choice:** Typed semantic state.
**Notes:** Packed C++ keys are not portable semantic identity; inactive manifold fields and solver impulses are omitted in Phase 5.

## Supported Pair Dispatch

| Option | Description | Selected |
| --- | --- | --- |
| Closed exact registry | Seven pinned manifold pairs, canonical primary ordering, explicit reversal | ✓ |
| Invent every pair | Produce manifolds even where upstream has no registered path | |
| Named primary functions only | Leave future generic dispatch duplicated across callers | |

**Agent choice:** Closed exact registry.
**Notes:** Unsupported, separated, and touching are distinct outcomes; chain pairs require checked child indices.

## Proxy Identity and Pair Ordering

| Option | Description | Selected |
| --- | --- | --- |
| Opaque proxy ID plus private pinned node order | Safe tree-scoped identity while matching solver-relevant pair order | ✓ |
| Raw integer node IDs | Mirror upstream slots publicly | |
| Sort by public semantic IDs | Stable consumer order that differs from pinned pair creation | |

**Agent choice:** Opaque proxy ID plus private pinned node order.
**Notes:** Pair callbacks are ordered evidence; ordinary collect-all query and ray results are unspecified-order sets.

## Filtering Boundary

| Option | Description | Selected |
| --- | --- | --- |
| Pure filter/refilter in Phase 5 | Prove mask/group and touch/reconsideration semantics; leave world contacts to Phase 6 | ✓ |
| Full contact manager in Phase 5 | Pull world ownership, persistence, listeners, and waking into this phase | |
| Defer all filtering | Leave explicit Phase 5 broad-phase behavior unproven | |

**Agent choice:** Pure filter/refilter in Phase 5.
**Notes:** Compatibility reporting must state the partial `COLL-05` boundary truthfully.

## TOI Contract and Evidence

| Option | Description | Selected |
| --- | --- | --- |
| Checked result plus private diagnostic trace | Closed public state/time and bounded exact termination/branch evidence | ✓ |
| Expose TOI internals publicly | Make caches, separation state, iterations, and globals stable API | |
| Compare final time only | Use one broad epsilon and omit cause/branch evidence | |

**Agent choice:** Checked result plus private diagnostic trace.
**Notes:** Preserve the pinned kernel and start exact; add only demonstrated named numeric policies.

## the agent's Discretion

- Exact private module and diagnostic type names.
- Typed error variant granularity.
- Optional owned query collection helpers that do not change visitor or ordering contracts.

## Deferred Ideas

- World contact lifecycle, warm-start impulses, joint suppression, waking, and listener timing.
- Mutable fixture geometry and topology updates.
- Independent collision crate or `no_std` extraction.
