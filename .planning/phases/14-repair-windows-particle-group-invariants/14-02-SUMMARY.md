---
phase: 14-repair-windows-particle-group-invariants
plan: "02"
subsystem: particles
tags: [rust, windows, allocation, transaction, rollback]
requires:
  - phase: 14-01
    provides: Exact Windows scratch allocation failure and valid candidate trace
provides:
  - Scratch reservation proportional to actual particle rows
  - Permanent original-seed successful append contract on all platforms
  - Complete private group rollback and next-allocation evidence
  - Candidate-only typed scratch failure classification
affects: [14-03, 14-04, 15]
tech-stack:
  added: []
  patterns: [bounded resource-budget regression, private structural transaction assertions]
key-files:
  created:
    - crates/liquidfun/src/particle/storage/transaction_test_support.rs
    - crates/liquidfun/src/world/object/tests/particle_group_transactions.rs
  modified:
    - crates/liquidfun/src/particle/storage/solver_state.rs
    - crates/liquidfun/src/particle/storage/solver_state/tests.rs
    - crates/liquidfun/tests/particle_group_properties.rs
    - crates/liquidfun/src/world/particle_object.rs
    - crates/liquidfun/src/arena.rs
    - crates/liquidfun/src/particle/storage.rs
    - crates/liquidfun/src/world/object/tests.rs
key-decisions:
  - Preserve the existing clone-plan-commit owner; D-03 does not justify a second prepared payload.
  - Preserve logical count guards while reserving scratch only for actual rows.
  - Map InvalidLaneBundle only within unpublished group candidates to the existing topology error; preserve shared authoritative assertions.
  - Keep complete storage and arena comparison support behind cfg(test).
requirements-completed: []
requirements-addressed: [PART-03, PART-04, PART-09, PART-10]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 14-2026-07-27T16-15-14
generated_at: 2026-09-14T03:04:00Z
duration: 16min
completed: 2026-09-14
---

# Phase 14 Plan 02: Bounded scratch and complete rollback Summary

**Particle scratch now reserves actual rows, the original audited append must succeed on every platform, and private tests prove full group rejection rollback and allocation continuity.**

## Performance

- Started after coordinator handoff at approximately 2026-09-14T02:48Z.
- Two planned tasks completed with three implementation commits; nine Rust files changed.
- The coordinator owns STATE.md, ROADMAP.md and task-ledger updates. Requirement completion remains deferred until the phase's full platform and stress evidence is accepted.

## Task commits

1. Task 1 sizing repair and original replay: `e58e028` — `fix(14-02): bound particle scratch reservation to actual rows`.
1. Task 2 complete private rollback: `88643e0` — `test(14-02): prove complete particle group rollback and allocation continuity`.
1. Task 1 candidate-error follow-up: `800a51c` — `fix(14-02): classify unpublished group scratch failures without panic`.

## Repair and D-03 disposition

The only allocation-sizing change is `zeroed_lane` reserving `particle_count` instead of `declared_capacity`, after the existing declared-capacity and signed-index guards. Copy, append, permutation, zero initialization and lazy optional-lane behavior remain intact. No capacity limit, float tolerance, identity or ordering rule changes.

The original seed, controls, generator version, fourteen expanded operations and separate vocabulary regression remain unchanged. Initialization and all prefix calls execute normally; operation 14 must return `Applied { created: 0, lifecycle: 0 }`, retain all existing groups and target members, and add exactly one member/particle. The model independently asserts the returned target identity. Windows-specific panic acceptance and its catcher are removed.

The existing clone-plan-commit transaction remains sufficient. The proven oversized allocation was repaired first. A second bounded RED test then confirmed that a residual scratch allocation/finite-value rejection (`InvalidLaneBundle`) still reached the old shared panic. Only unpublished group storage/lifecycle preparation now maps that category to `InvalidParticleGroupTopology`; ordinary creation/lifecycle mappers and other categories retain their existing behavior. Tests preserve the shared corruption assertion and capacity error identity. No second owner or new public error API was introduced.

## Verification evidence

All local Cargo execution used the coordinator's verified Linux ARM64 Docker image `sha256:5f3d56072c0c734ebb2381c59f0a20db1330b8884134c4e3a9d369de2d810ea8`, Rust 1.97.0, offline registry, read-only rustup volume and isolated Cargo target volume. This is local Linux evidence, not native Windows/macOS proof.

Evidence is retained beneath `target/phase14-repair/`:

| Record | Result |
| --- | --- |
| `attempt-20260914-task1/red.log` | Exactly one test fails before repair: ten rows reserve the 4,096-row logical maximum and exceed the 20-row resource budget. No GiB allocation occurs. |
| `attempt-20260914-task1/green-gate.log` | Solver module 11 pass, original exact replay 1 pass, creation 7 pass and mutation 6 pass. First Clippy attempt then catches two redundant float-array assertions; retained as failed. |
| `attempt-20260914-task1/gate-corrected.log` | Ordered fmt, Clippy, build, all-feature tests pass; 991 total tests including doctests. |
| `attempt-20260914-task2/focused-01..04` | Actual filenames are `focused.log`, `focused-02.log`, `focused-03.log`, `focused-04.log`; retained setup/visibility failures are not passing evidence. |
| `attempt-20260914-task2/focused-05.log` | Three private rollback tests pass with populated fixture assertions. |
| `attempt-20260914-task2/gate.log` | Retained Clippy semicolon rejection. |
| `attempt-20260914-task2/gate-02.log` | Ordered gate passes 994 tests; inventory lists 416 default lib tests; explicit transaction filter runs 3 passing tests. |
| `attempt-20260914-task1-candidate/red.log` | Exactly one direct candidate error-seam test fails with the previous authoritative-storage panic. |
| `attempt-20260914-task1-candidate/gate.log` | Ordered gate passes 998 tests, then 4 explicit error-classification tests pass. |
| `attempt-20260914-metadata/gate.log` | Final ordered metadata commit gate passes all 998 tests. |

The managed checker reports zero findings after staging the new modules (934 scanned files). Diff whitespace checks and scoped review pass. Every touched Rust file remains below 629 lines. No non-GSD Markdown was changed. Source inspection found no new stubs, unsafe blocks, public debug surface, endpoints, dependencies or trust-boundary changes outside the plan's threat model.

## D-06 assertion inventory

| State category | Evidence |
| --- | --- |
| Every required/optional lane, particle identity entries, dense maps, free/retired state, group metadata/order, pair/triad rest data, proxies, contacts, weights, forces, solver caches | `Before::assert_unchanged` compares full `ParticleStorage` structural equality in every live system. No serialized memory or raw pointer comparison. |
| Lifetime clock, expiration-dirty flag, destruction-by-age and count policy | Full `ParticleLifetimeState` equality separate from storage expiration lanes/order. |
| System definition, group vector, timestamp, system diagnostic ID | Named assertions for every ParticleSystem field outside storage/lifetime. |
| Every arena occupied/vacant/retired slot, generation/scope, free-list order, retired count, capacity | `Arena::assert_same_state_for_test` compares both particle-system and group arenas plus every occupied value. |
| Group shell identity/diagnostic/system, world system order and next diagnostic allocation | Full arena comparison and explicit order/next-diagnostic assertions; world remains unlocked and unpoisoned. |
| Populated categories | Fixture uses real public group/particle creation, destruction and stepping for colored SOLID/SPRING/ELASTIC/STATIC_PRESSURE/TENSILE particles, contacts, weights, pairs/triads, expiration, pending/ZOMBIE/listener state and vacant identities. Test-only helper asserts category presence, seeds inaccessible association metadata and one impractical retired generation, and populates finite scratch/forces. |
| Next allocations | Preflighted same-world next group and particle identities match the successful creation after rejected new and append requests. Shell/particle diagnostic IDs are exactly the preflighted ID and ID+1; world next is ID+2. |
| No extra lifecycle output | Rejections return only typed errors and preserve all output-producing state. The next step's full ordered particle lifecycle kind, slot/generation, diagnostic and cause matches a separately constructed no-rejection control world, normalizing world identity only. The pre-existing pending listener is nonempty, so this is not vacuous; a following step is empty in both worlds. |

Separate tests cover new rejection, append rejection and next allocation/lifecycle continuity. Full same-world structural equality is distinct from normalized cross-world control evidence.

## Legitimate late topology rejection

The executed fixture contains 64 ELASTIC positions spaced 1,000 units apart and a bounded finite lifetime. Both public destinations return exactly `InvalidParticleGroupTopology`. Source localization identifies Voronoi `GridLimitExceeded` at the existing 4,096-cell limit: generated topology computes a grid for the necessary 63,000-unit span using the default diameter, stride-derived radius and margin. This is a source-supported localization, not a newly captured raw-error trace, and is not a non-finite-derived failure. Public planning has already cloned/appended particles, initialized lifetime and refreshed contacts before calling this topology stage; no commit runs on the error.

## Deviations and issues

1. **[Rule 2 — private proof completeness]** Task 2 ownership was amended before edits to include the storage helper/registration and arena comparison seam, keeping the task at five Rust files. Helpers are cfg(test), with no production equality/debug or public accessor.
1. **[Rule 2 — verification constraints]** Failing RED commits were not created because repository instructions forbid committing unless the ordered gate passes. Both real RED logs were preserved, then tested repairs were committed. Task 2 verifies the already-correct transaction boundary; fixture construction failures are not represented as a pre-existing rollback defect.
1. **[Rule 2 — candidate classification]** The Task 1 error audit required a separately gated follow-up commit after Task 2. It fixes only the candidate-specific InvalidLaneBundle classification, after the root sizing repair; a mapper-only patch would fail the resource-budget and original-success tests.

Initial test-support visibility mistakes, an invalid empty-source fixture, insufficient triad spacing and lint findings were corrected and their failed logs retained. The final fixture asserts nonempty triads rather than weakening coverage. Standing authorization in AGENTS.md permits these corrective iterations; no new permission was requested. No authentication gates occurred.

## Next phase readiness

Ready for Plan 14-03 stress/rollback expansion and Plan 14-04 exact platform verification. The coordinator published `e58e028a31ffe845c5465e8d72dd5d933c9d80f5`; [Cargo CI run 34800684869](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34800684869) has successful Windows, Linux and macOS default jobs. Windows job `103842637964` records the original named regression passing at `2026-09-14T02:55:42`. Retained logs, metadata and digests are under `target/phase14-platform/e58e028a31ffe845c5465e8d72dd5d933c9d80f5/attempt-20260914-initial-repair/retained-proof.json`; the Windows raw log SHA-256 is `8a862a4ba8a1c1e47ddae9ac606e2873e07d55b8869d5450dd69316566151a0b`. These results cover the sizing repair and original-success test, and exclude later private tests/error classification. The run's quality job was still running when the coordinator reported those results, so no whole-workflow success is claimed. No final full-source Windows result or phase-completion claim is made here.

## Self-Check: PASSED

All nine listed Rust files exist, and all three recorded task commits resolve in git history. The exact transaction filter runs three tests, the candidate error filter runs four, and the original replay filter runs one. Scoped review confirms only actual-row reservation plus candidate-only error classification changes production behavior. No known stubs or new threat flags were found.
