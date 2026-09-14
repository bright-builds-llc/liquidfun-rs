---
phase: 14-repair-windows-particle-group-invariants
plan: "03"
subsystem: particles
tags: [rust, windows, regression, topology, transactional-rejection]
requires:
  - phase: 14-02
    provides: Actual-row scratch reservation and complete private rollback proof
provides:
  - Precise public zero-rest-pair rejection without masking storage corruption
  - Both literal Windows replays with required successful outcomes
  - Typed bounded property outcomes and complete public rollback observations
  - Nine populated rejection cases with successful neighboring operations
affects: [14-04, 15]
tech-stack:
  added: []
  patterns: [private cause preservation, actual-identity rollback, normalized fresh-world replay]
key-files:
  created:
    - crates/liquidfun/tests/particle_group_properties.proptest-regressions
    - crates/liquidfun/tests/particle_groups/transaction_support.rs
    - crates/liquidfun/tests/particle_groups/transactional_rejection.rs
    - crates/liquidfun/tests/particle_group_mutation/transactional_rejection.rs
    - .planning/phases/14-repair-windows-particle-group-invariants/14-REACTIVE-TOPOLOGY-DIAGNOSIS.md
  modified:
    - crates/liquidfun/src/particle/storage/group.rs
    - crates/liquidfun/src/particle/storage/group/tests.rs
    - crates/liquidfun/src/particle/solver/preparation.rs
    - crates/liquidfun/src/world/particle_coupling/executor.rs
    - crates/liquidfun/src/world/step/report.rs
    - crates/liquidfun/tests/particle_group_properties.rs
    - crates/liquidfun/tests/particle_group_properties/model.rs
    - crates/liquidfun/tests/particle_group_properties/snapshot.rs
    - crates/liquidfun/tests/particle_groups.rs
    - crates/liquidfun/tests/particle_group_mutation.rs
key-decisions:
  - Classify only ConstraintError::ZeroLengthPairDistance as StepError::InvalidParticleGroupTopology; all other failures retain their existing invariant boundary.
  - Preserve literal seeds and generator bounds; require valid operations to apply and keep every newly exposed minimized input.
  - Compare actual identities and lifecycle records for rollback while retaining normalized replay across separate worlds.
requirements-completed: []
requirements-addressed: [PART-03, PART-04, PART-09, PART-10, TEST-02, TEST-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 14-2026-07-27T16-15-14
generated_at: 2026-09-14T05:02:00Z
duration: approximately 2h including host-recovery pause
completed: 2026-09-14
---

# Phase 14 Plan 03: Typed replays and public rollback Summary

**Both Windows inputs now require real successful behavior, zero-rest topology rejection has a precise public category, and populated public tests protect identities, topology, optional data and lifecycle continuity.**

## Execution and commits

Three amended tasks completed. Changes cover thirteen Rust files, one persisted property-input file, and plan/diagnosis metadata. Work began around 03:08 UTC and resumed finalization after the coordinator recovered Docker; elapsed time includes the host-recovery pause. The coordinator owns STATE, ROADMAP, requirements and the task ledger.

1. Task 0 — `3953157`: `fix(14-03): distinguish zero-rest topology rejection from storage corruption`.
1. Task 1 — `6761b26`: `test(14-03): preserve Windows replays and exact operation outcomes`.
1. Task 2 — `9d826aa`: `test(14-03): prove populated public rollback and valid followups`.

The independent CI fixture repair `82ba701` belongs to the coordinator and is not a Plan 03 task change. These task commits remain local for Plan 14-04's combined publication.

## Behavior and evidence boundaries

The original seed `4149329052036581951`, its fourteen controls/expanded operations, and the separate vocabulary seed remain unchanged. The additional seed `190752942043209832` retains all fifteen controls and explicit expanded operations. Each replay checks generator equality and determinism; every prefix operation must apply except the intentional wrong-system join. Both final appends require `Applied { created: 0, lifecycle: 0 }`. The model also checks returned target identity, exact original-member order, and a one-particle/member increase for every successful append.

Generator version 1, 128 property cases, and limits of 24 operations, 8 groups and 32 particles are unchanged. Rejections carry specific test categories. Create/append accept `PendingDelete` only with a nonzero pending-row witness. Mutation topology errors remain explicit; destroy/compact errors are unexpected. Step accepts only the new precise topology variant. Every unexpected error includes the operation index and input; `ParticleLifecycleInvariant` remains a failure.

Strict matching exposed the persisted seed `9340296914046120310`. Independent exact and minimal diagnosis showed coincident reactive particles producing a deliberately rejected zero-rest spring pair, with valid unchanged storage. Phase 10's pinned `zero_length_pair` witness already requires `typed_error`. Task 0 preserves that rejection and its private cause through solver preparation, then maps only `ConstraintError::ZeroLengthPairDistance` to `StepError::InvalidParticleGroupTopology`. Corrupt-lane and other-generation-error controls reject broader classification. No geometry, tolerance, allocation policy or global step-transaction guarantee changes. Named coincident-rejection and shifted-success tests protect this boundary.

## Final local verification

Every task commit followed ordered `cargo fmt --all`, Clippy with all targets/features and denied warnings, all-target/all-feature build, and all-feature tests. Execution used Rust 1.97.0 in the pinned Linux ARM64 Docker image `sha256:5f3d56072c0c734ebb2381c59f0a20db1330b8884134c4e3a9d369de2d810ea8`, offline dependencies, read-only rustup volume and the isolated warm Cargo target volume. This is local Linux evidence, not final supported-platform proof.

| Retained log under `target/phase14-public/` | Result |
| --- | --- |
| `attempt-final-task0/gate.log` | Ordered gate: 1,012 tests pass including doctests; separate private group module: 16 pass. |
| `attempt-final-task1/gate.log` | Ordered gate: 1,012 pass; inventory lists six property tests; each original/current Windows exact filter runs one passing test; complete property target: six pass with 128 random cases and persisted inputs. |
| `attempt-final-task2/gate.log` | Ordered gate: 1,012 pass; mutation target: ten pass; creation target: twelve pass; property target: six pass; explicit complete all-feature liquidfun package: 1,012 pass. |
| `attempt-final-summary/gate.log` | Summary commit's ordered gate: 1,012 tests pass; its managed checker also reports zero findings. |
| `attempt-final-task0/managed.log`, `attempt-final-task1/managed.log`, `attempt-final-task2/managed.log` | Managed checks pass after staging task files. Final source scan includes all three new Rust children: 937 files, zero findings. |

Scoped diff review and whitespace checks pass. All touched Rust files remain below 629 physical lines. Only parser-owned GSD Markdown changed; no mdformat pass was applied. No known stubs, dependencies, unsafe blocks or unplanned trust boundaries were introduced.

## Combined D-06 assertion inventory

| State category | Public evidence in this plan | Complete private evidence from Plan 14-02 |
| --- | --- | --- |
| Stable identities and next allocations | Actual particle/group IDs, stale shells, ordered memberships and pending errors; successful append retains its target; split retains the first component and allocates the next; creation and join demonstrate continued usable identities. | Full storage/arena identity, generation, free and retired state; preflighted next group/particle IDs and exact diagnostic allocation continuity. |
| Group metadata | Flags, exact transform/rotation, angle, ordered members, center, velocities, mass, inertia and optional depth bits. | Full group records, order, strength, shell metadata and cached statistics. |
| Required particle lanes | Actual IDs, positions, velocities, flags, group IDs, weights and forces; floating lanes use bit representations. Nonzero force/weight witnesses prevent empty-data coverage. | Structural equality of every authoritative required lane and identity mapping. |
| Optional and lifetime data | Allocated nonempty colors and expiration order, system definition/statistics, pending snapshot errors and retained-empty shell behavior. | Complete optional scratch/lifetime lanes, clock, expiration dirty state, policy, listener journals and sequence counters. |
| Pairs, triads and rest data | Nonempty pairs/triads with actual endpoints, flags, strength, pair distance and all eleven triad floating fields. | Exact full topology/rest records, including private retained state. |
| Derived data | Nonempty particle/body contacts, exact endpoints/owners, normals, weights and masses; forces, statistics and stuck identities. | Proxies, contacts, solver caches, scratch lanes and timestamps. |
| Pending, zombie and retired state | A live fixture carries a pending listener particle, ZOMBIE flag and stale group before each focused rejection. | Full pending snapshots, identity states, free/retired slots and arena allocation state. |
| Lifecycle output | Rejections return precise errors without reports; the next step emits exactly the existing pending particle listener with its actual ID, cause, system and group snapshot; the following step is empty. Successful joins emit exactly one expected shell destruction. | Complete output-producing journals remain unchanged; normalized no-rejection control matches deferred output and diagnostic identities. |

`RollbackSnapshot` additionally retains actual lifecycle records, known shell identities and the complete owned system snapshot. `SemanticSnapshot` remains normalized for independent-world replay. The new shared focused helper uses actual identities throughout. No normalized comparison substitutes for same-world rollback.

Nine new focused cases cover capacity/new rejection, late new topology, late append topology, stale append, wrong-system append, stale join, cross-system join, stale split and stale flag change. Both systems are compared where an operation crosses system ownership. Every family includes a successful follow-up, while the existing successful recipe, retained-empty, rigid/solid, lifecycle and mutation suites remain intact.

## Deviations and recovery

1. **[Rule 1 — error classification]** Strict properties exposed a misleading invariant category for an existing supported typed-rejection contract. The coordinator added the bounded five-file Task 0 before integration and obtained an independent amendment review. The reviewed correction classifies only zero-rest pairs; the original broader isolated patch remains preserved as historical evidence.
1. **[Rule 2 — proof completeness]** Ownership was amended before adding shared snapshot/fixture and mutation child modules, plus the three persisted newly exposed random inputs. This preserves full public coverage and file limits without adding public inspection APIs.
1. **[Rule 2 — verification constraints]** Failing RED commits were not created because repository instructions require a passing ordered gate before every commit. Failed strict-model observations, diagnostic RED logs and test-development failures remain retained. No seed, case count, bound, operation or success assertion was removed to obtain a pass.
1. **[Rule 3 — environment]** Docker image/blob and metadata I/O failures interrupted final verification. The coordinator preserved WIP and diagnostics, obtained explicit user approval to restart Docker Desktop, verified the pinned image/toolchain after restart, and reran checks. Failed records were not relabeled as passing.

Earlier retained logs include `attempt-task1-01/focused.log` and `focused-02.log` for the pending-delete category discoveries, `attempt-task1-gate/gate.log` for the unexpected step category, and `attempt-task2-01/` for fixture/lint corrections. The fixture originally expected two step events; source inspection confirmed that a retained shell yields only the requested particle listener. That assertion was corrected without changing engine behavior. The oversized snapshot helper was split by group observation, keeping the shared helper cohesive.

## Next plan readiness

Plan 14-03 is complete locally. Plan 14-04 must publish the final combined candidate and obtain matching Windows, Linux and macOS focused/full-package results. Earlier platform results at `e58e028` cover only the prior scratch repair and cannot certify these later changes. Requirements remain addressed, not marked phase-complete; no canonical fixture promotion, acceptance waiver, package release or Phase 15 evidence claim is made.

## Self-Check: PASSED

All sixteen task source/fixture/metadata paths and this summary exist. All three task commits resolve in git history. The thirteen touched Rust files remain below 629 lines. Exact Windows filters each execute one test, all focused suites have nonzero passing counts, and complete package verification passes. No known stubs or unplanned threat surfaces were found. Coordinator-owned state, roadmap and checkpoint files are excluded from these commits.
