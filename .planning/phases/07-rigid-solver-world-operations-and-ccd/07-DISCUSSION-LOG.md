# Phase 7: Rigid Solver, World Operations, and CCD - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-07-12T23:44:54.491Z
**Phase:** 7-rigid-solver-world-operations-and-ccd
**Mode:** Yolo
**Areas discussed:** Body and world control contract, island solver and sleeping semantics, CCD and sub-stepping contract, world queries/ray casts/origin shifting/evidence ordering

## Body and world control contract

| Option | Description | Selected |
| --- | --- | --- |
| Granular `World` methods, typed `WakePolicy`, no-effect as `Ok(())` | Preserves handle authority, recognizable direct methods, and source-compatible ignored branches. | ✓ |
| Granular methods returning `ControlOutcome` | Makes applied/no-op branches observable but exposes an unstable outcome taxonomy. | |
| Checked `BodyControl` command enum | Centralizes dispatch and scenarios but prematurely freezes a public control DSL. | |
| Borrow-scoped `BodyMut` capability | Improves repeated-mutation ergonomics but weakens the established authority model. | |

**Agent's choice:** Granular `World` methods with typed `WakePolicy` and source-compatible successful no-ops.
**Notes:** Private checked candidate helpers should remove repeated validation without changing the public contract.

## Island solver and sleeping semantics

| Option | Description | Selected |
| --- | --- | --- |
| Pinned ephemeral DFS with all-island staged commit | Preserves upstream order and prevents partial solver mutation after late failure. | ✓ |
| Pinned ephemeral DFS with per-island commit | Smaller transaction but allows earlier islands to mutate before a later failure. | |
| Canonically sorted connected components | Deterministic but changes pinned solver-visible seed and traversal order. | |
| Persistent incremental island graph | Potential optimization with substantial invalidation and ordering complexity. | |

**Agent's choice:** Source-faithful ephemeral DFS islands with checked configuration and staged all-island commit.
**Notes:** The current arena needs an explicit newest-first body-order lane; contacts already preserve useful newest-first adjacency.

## CCD and sub-stepping contract

| Option | Description | Selected |
| --- | --- | --- |
| Private state machine plus semantic completion status | Preserves pinned internals while making pending continuous work safely visible. | ✓ |
| Private state machine with opaque completion | Closest C++-shaped API but makes sub-step progress hard to observe and verify. | |
| Explicit continuation token | Clear cooperative scheduling but unfamiliar, mutation-sensitive, and larger than the upstream model. | |

**Agent's choice:** Private source-faithful CCD state machine with `Complete` / `ContinuousPending` report evidence.
**Notes:** Auto force clearing still occurs after every successful step call, including pending sub-step calls.

## World queries, ray casts, origin shifting, and evidence ordering

| Option | Description | Selected |
| --- | --- | --- |
| Streaming visitors, typed directives, evidence-only canonicalization | Preserves termination/clipping semantics and avoids a callback-order promise. | ✓ |
| Streaming API plus canonical owned helpers | Adds ergonomic deterministic collectors but expands and risks confusing the public ordering contract. | |
| Canonicalize before callbacks | Makes order stable but changes terminate/clip behavior and allocates every candidate. | |
| Rebuild and swap shifted broad phase | Simplifies rollback but changes topology, proxy identity, and later pair order. | |

**Agent's choice:** Borrow-scoped streaming visitors with typed controls; canonicalize only in declared evidence collectors.
**Notes:** Preserve repeated child occurrences, do not implicitly apply fixture filter masks, and shift the existing tree in place after full validation.

## the agent's Discretion

- Exact type, method, directive, error, and module names.
- Exact bounded budgets and field-specific numerical thresholds with pinned-source justification.
- Exact protocol action/checkpoint names and corpus sizes inside the closed required families.

## Deferred Ideas

- Persistent island graph optimization after profiling.
- Canonical owned query helper APIs after consumer demand.
- Joint traversal and shifted world anchors in Phase 8.
