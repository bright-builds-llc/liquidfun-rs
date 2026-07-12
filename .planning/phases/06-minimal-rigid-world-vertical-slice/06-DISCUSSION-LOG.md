# Phase 6: Minimal Rigid World Vertical Slice - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-11
**Phase:** 6-minimal-rigid-world-vertical-slice
**Mode:** Yolo
**Areas discussed:** Body and fixture contract, automatic contact lifecycle, minimal rigid-world evidence

## Body and fixture contract

| Option | Description | Selected |
| --- | --- | --- |
| Checked definitions plus handle-oriented `World` methods | Invariant-bearing owned definitions, immutable shape snapshots, centralized world side effects, typed pre-mutation errors | ✓ |
| Plain public definition structs | Familiar and compact, but invalid intermediate state and public-field compatibility harden early | |
| World-scoped mutable façades | Object-style ergonomics, but hides whole-world effects and complicates borrow/side-effect centralization | |
| General mutation patches | Atomic and traceable, but over-engineers direct setters and can change pinned per-call ordering | |

**User's choice:** Auto-selected checked owned definitions plus granular handle-oriented `World` methods.
**Notes:** This follows Phase 3 authority-free handles, Phase 5 immutable shape snapshots, and pinned asymmetric mass/material/filter side effects.

## Automatic contact lifecycle

| Option | Description | Selected |
| --- | --- | --- |
| Private automatic manager plus one-contact solve | Own pair admission, persistence, sensors, mixing, warm-start transfer, ordered events, and one deterministic discrete solve | ✓ |
| Lifecycle without any solve | Smaller, but fails the Phase 6 warm-start/solve requirement and cannot prove impulses are consumed | |
| Complete island solver now | Broad parity, but captures Phase 7 forces/islands/sleeping/CCD scope and weakens the vertical-slice boundary | |

**User's choice:** Auto-selected the private automatic manager with a narrowly bounded one-contact solver witness.
**Notes:** Contacts remain transient with harness-private semantic occurrence identity; complete islands and world dynamics remain Phase 7.

## Minimal rigid-world evidence

| Option | Description | Selected |
| --- | --- | --- |
| Dedicated bounded lifecycle timelines | Closed ordered actions and checkpoints prove one world across body/fixture/contact transitions and reuse existing evidence paths | ✓ |
| Independent atomic cases | Simple, but weak for persistence, warm-start carry, refiltering, activation, and cascade order | |
| General future-world DSL | Extensible, but freezes Phase 7/8 semantics prematurely and expands validation/minimization scope | |

**User's choice:** Auto-selected dedicated bounded lifecycle timelines with a closed witness registry.
**Notes:** Two mandatory families cover non-colliding body/fixture lifecycle and a single-contact lifecycle; D0/D2/D1 authority remains unchanged.

## the agent's Discretion

- Exact private module decomposition, checked error/accessor names, bounded scenario record names, and evidence-derived field thresholds.

## Deferred Ideas

- Complete rigid dynamics, sleeping, CCD, queries, joints, and a future general scenario language remain assigned to later phases.
