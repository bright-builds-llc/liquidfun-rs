# Phase 9: Particle Storage, Lifecycle, and Coupling - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-07-14
**Phase:** 09-particle-storage-lifecycle-and-coupling
**Mode:** Yolo
**Areas discussed:** Safe storage and buffer ownership, lifecycle and deferred compaction, contacts/queries/rigid coupling

## Safe storage and buffer ownership

| Option | Description | Selected |
| --- | --- | --- |
| Unified owned SoA storage with growable or fixed-capacity mode | Deepen the existing owned lane bundle and one permutation authority; expose borrow-scoped views and typed mutation scopes. | ✓ |
| Per-lane owned overrides with smallest-capacity clamping | Mirror upstream partial lane replacement more closely, at the cost of cross-lane ownership and capacity complexity. | |
| Generic buffer backend behind the stable-ID/permutation core | Abstract growable, fixed, and future specialized backing strategies behind one backend interface. | |

**User's choice:** Auto-selected the unified owned SoA design.
**Notes:** It preserves Phase 3 stable-ID and transaction contracts while reproducing fixed-capacity behavior without raw pointers or unsafe aliasing.

## Lifecycle and deferred compaction

| Option | Description | Selected |
| --- | --- | --- |
| Storage-authoritative two-phase zombie lifecycle | Mark pending-delete, retain snapshots through callbacks, then atomically compact and invalidate. | ✓ |
| World-arena identity with dense-storage tombstones | Keep placeholder arena identity and synchronize it with dense storage. | |
| Eager ID invalidation with deferred row reclamation | Invalidate handles immediately and reclaim dense rows later through a separate scheduler. | |

**User's choice:** Auto-selected the storage-authoritative two-phase lifecycle.
**Notes:** This matches pinned mark-now/remove-later behavior and avoids split identity authority. Equal expiration ties remain an oracle research item.

## Contacts, queries, and rigid coupling

| Option | Description | Selected |
| --- | --- | --- |
| Unified source-timed particle slice inside `World::step` | Execute contacts, callbacks, queries, and rigid coupling at the pinned world-step point with one lifecycle journal. | ✓ |
| Particle-specific hook and report invoked from `World::step` | Keep particle API types separate while retaining a shared internal journal. | |
| Explicit particle contact refresh outside `World::step` | Provide a separate public refresh or particle-only tick. | |

**User's choice:** Auto-selected the unified source-timed `World::step` slice.
**Notes:** This prevents stale rigid coupling, reuses the Phase 8 hook/poisoning contract, and keeps callback order observable at its true source point.

## the agent's Discretion

- Exact type/module names and plan decomposition.
- Exact bounded capacities, corpus sizes, thresholds, and field-specific numerical policies after pinned-source research.
- Exact mutation editor shape, provided derived state is valid when the borrow ends.

## Deferred Ideas

- Particle groups, topology, pairs/triads, and solver behaviors remain Phase 10.
- Generic or unsafe external backing stores remain deferred until measured need.
