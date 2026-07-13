---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-13T21:32:39.399Z
---

# Phase 8: Joints, Rope, Callbacks, and Rigid Sign-Off - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-07-13
**Phase:** 8-joints-rope-callbacks-and-rigid-sign-off
**Mode:** Yolo
**Areas discussed:** Joint and rope consumer contract, callback/filter/listener timing, diagnostics and broad rigid sign-off

## Joint and rope consumer contract

| Option | Description | Selected |
| --- | --- | --- |
| Tagged definitions/snapshots plus one `JointId` | Extend the established identity model with checked per-kind state and exhaustive inspection. | ✓ |
| Eleven typed joint IDs and APIs | Maximize compile-time kind safety at the cost of gear, generic destruction, event, and association complexity. | |
| Eleven create methods plus a generic property bag | Minimize initial scaffolding while weakening invariants and inspection. | |

**User's choice:** Auto-selected the recommended tagged-definition contract with one `JointId`; kept standalone rope independent of `World` and rope joint state.

**Notes:** Gear dependencies become explicit safe-Rust edges and deterministic cascades. Every source solver, mutation, reaction, origin-shift, dump, and differential path remains independently represented.

## Callback, filter, and listener timing

| Option | Description | Selected |
| --- | --- | --- |
| Borrowed synchronous decisions plus owned lifecycle reports | Preserve source decision points while making observation and fan-out safe after coherent completion. | ✓ |
| Borrowed full callback bundle per operation | Mirror more C++ callback timing but expose a wider panic-sensitive locked surface. | |
| World-registered boxed filter/listener slots | Offer familiar persistence with ownership, replacement, trait-object, and poisoning complexity. | |
| Multi-listener registry | Add application fan-out while inventing ordering and decision-combination semantics absent upstream. | |

**User's choice:** Auto-selected borrowed synchronous filter/pre-solve decisions and one authoritative owned lifecycle timeline.

**Notes:** Persistent registration and multi-listener semantics are rejected. Commands remain deferred; contacts remain authority-free; only synchronous decision panics poison the world.

## Diagnostics and broad rigid sign-off

| Option | Description | Selected |
| --- | --- | --- |
| Full public diagnostics/debug-draw in Phase 8 | Pull the complete `RIGD-10` and renderer/profile surface forward. | |
| Byte-for-byte C++ dump clone | Compare formatting while coupling to locale, indices, and unsupported output details. | |
| Defer every diagnostic surface | Avoid premature APIs but fail Phase 8 dump coverage. | |
| Semantic reconstruction dump plus bounded headless diagnostics | Satisfy Phase 8 evidence without claiming persistence or completing later debug-draw/profile scope. | ✓ |

**User's choice:** Auto-selected semantic reconstruction plus closed headless diagnostic evidence; deferred the full public draw/profile contract.

**Notes:** The `phase8-v1` registry remains fail-closed, preserves order/multiplicity only where semantic, excludes wall-clock timing, and permits the scoped rigid sign-off claim only after actual canonical D1 evidence.

## the agent's Discretion

- Exact naming and private module boundaries.
- Plan decomposition across joint families and shared solver infrastructure.
- Reviewed field thresholds, capacities, and witness corpus sizes.
- Inclusion of inventory-proven typed material/tangent-speed pre-solve controls.

## Deferred Ideas

- Complete public renderer-neutral debug drawing, timing profiles, particle drawing, and `RIGD-10` completion remain outside Phase 8.
