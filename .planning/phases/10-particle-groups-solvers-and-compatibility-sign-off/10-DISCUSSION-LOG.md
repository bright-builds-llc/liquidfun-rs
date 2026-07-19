# Phase 10: Particle Groups, Solvers, and Compatibility Sign-Off - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-07-19T05:17:27.914Z
**Phase:** 10-particle-groups-solvers-and-compatibility-sign-off
**Mode:** Yolo
**Areas discussed:** Particle-group public contract and mutation semantics, topology and mutation ordering, solver graph and compatibility sign-off

***

## Particle-group Public Contract and Mutation Semantics

| Option | Description | Selected |
| --- | --- | --- |
| C++-shaped `ParticleGroupDef` with nullable fields | Maximizes surface familiarity but permits contradictory combinations, lifetime-heavy references, and partial-state hazards. | |
| Typed fixed-order recipe plus explicit target plus opaque group view | Uses owned invariant-bearing sources, separates new versus append targets, exposes stable IDs without dense indices, and supports one transactional planner. | ✓ |
| Operation-specific constructors over a shared transaction core | Makes simple sources ergonomic but proliferates public methods and risks inconsistent sequencing or validation. | |

**User's choice:** Auto-selected the recommended typed fixed-order recipe, explicit target, and opaque borrow-scoped view.
**Notes:** Preserve source order, stable group and particle identities, transactional failure, can-be-empty behavior, source-equivalent joins and splits, and aligned depth/rigid inspection.

***

## Topology and Mutation Ordering

| Option | Description | Selected |
| --- | --- | --- |
| Storage-owned source-order mutation transaction | Extends the Phase 9 authority and commits permutations, group metadata, depth, rigid caches, pairs, and triads together. | ✓ |
| Separate topology component with coordinated atomic commit | Gives topology a hard module boundary but risks split-brain authority and revision skew across group ranges and dense remaps. | |
| Stable-ID topology graph with dense solver projection | Makes external identity direct but risks projection reorder, regenerated rest-state drift, invalidation complexity, and repeated lookup cost. | |

**User's choice:** Auto-selected the recommended storage-owned source-order mutation transaction.
**Notes:** Pure topology planning kernels may live in a cohesive child module, but `ParticleStorage` owns the candidate and commit. Encode create, join, split, rotation, reactive regeneration, depth, and rigid-state operations explicitly.

***

## Solver Graph and Compatibility Sign-Off

| Option | Description | Selected |
| --- | --- | --- |
| Closed leaf ledger plus pinned pass graph | Makes pass gates/order machine-checkable and gives every flag, pass, and group behavior an independently promotable evidence leaf. | ✓ |
| Black-box flag and interaction witness lattice | Keeps evidence entirely semantic but can miss reordered or skipped passes with equivalent end states. | |
| Aggregate subsystem rows with attached coverage manifest | Minimizes ledger churn but can hide uncovered leaves and overclaim aggregate parity. | |

**User's choice:** Auto-selected the recommended closed leaf ledger plus pinned pass graph.
**Notes:** Native tests prove exact private pass IDs, gates, multiplicity, and order; canonical D1 scenarios prove semantic outcomes. Every leaf needs control and activation witnesses, named numeric policies, and truthful promotion authority.

## the agent's Discretion

- Exact naming and plan/module decomposition within the locked single-authority, pass-graph, and leaf-ledger contracts.
- Exact bounded corpus sizes and field-specific numerical policies when source analysis and canonical evidence justify them.
- The unpublished mechanism used to expose private pass traces to native verification.

## Deferred Ideas

- Phase 11 examples, renderer-neutral scenarios, headless tooling, debug drawing, and testbed work.
- Phase 12 performance, broader portability, fuzzing, coverage, packaging, and release hardening.
- Unmeasured allocator, GPU, unsafe-buffer, SIMD, parallel, and alternate-precision extensions.
