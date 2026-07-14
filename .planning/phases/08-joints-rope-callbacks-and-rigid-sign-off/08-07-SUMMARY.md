---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
plan: "07"
subsystem: standalone-rope
tags: [rust, rope, constraints, source-order, transactional]
requires:
  - phase: 04-math-settings-and-numerical-policy
    provides: source-ordered f32 math, tau mapping, and finite domain boundaries
  - phase: 08-01
    provides: explicit separation between world-owned RopeJointDef and standalone rope scope
provides:
  - checked owned standalone RopeDef and bounded RopeIterations contracts
  - source-ordered pure rope integration, stretch/bend solving, and velocity reconstruction
  - transactional construction, angle control, and stepping with borrow-scoped vertex inspection
affects: [08-11, 08-12, 08-13, standalone-rope-evidence]
tech-stack:
  added: []
  patterns: [candidate-first pure simulation, bounded invariant newtypes, pinned exact-bit oracle witness]
key-files:
  created:
    - crates/liquidfun/src/rope.rs
    - crates/liquidfun/src/rope/core.rs
    - crates/liquidfun/tests/standalone_rope.rs
  modified:
    - crates/liquidfun/src/lib.rs
    - .codex/tasks/lessons.md
key-decisions:
  - "Keep standalone Rope entirely separate from World, JointId, bodies, contacts, islands, and RopeJointDef."
  - "Clone the compact pure state for every positive step and replace it only after all source-ordered derived arithmetic stays finite."
patterns-established:
  - "Standalone rope: integrate vertices, solve stretch-bend-stretch per iteration, then reconstruct velocities in index order."
  - "Compatibility bits: derive exact expected values from the pinned C++ oracle, never from the Rust implementation under test."
requirements-completed: [JOIN-03, JOIN-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-14T00:00:00Z
duration: 16 min
completed: 2026-07-13
---

# Phase 8 Plan 07: Standalone Rope Summary

**World-independent checked rope simulation with pinned integrate/constraint/reconstruction order, bounded inputs, and transactional finite-state guarantees.**

## Performance

- **Duration:** 16 min
- **Started:** 2026-07-13T23:44:00Z
- **Completed:** 2026-07-14T00:00:00Z
- **Tasks:** 1
- **Files modified:** 5

## Accomplishments

- Added an owned public `rope::Rope` contract with checked definition lanes, fixed/free inverse-mass semantics, finite coefficients, bounded vertices, and a reviewed zero-inclusive iteration newtype.
- Translated the pinned `b2Rope` core in source order: indexed integration, stretch/bend/stretch per iteration, source angle wrapping, and indexed velocity reconstruction.
- Made every positive step candidate-first so non-finite arithmetic and angle-wrap exhaustion leave all live vertex and velocity bits unchanged.
- Added eleven focused consumer tests, including an exact one-iteration witness independently generated from pinned C++ `b2Rope` revision `7f20402173fd143a3988c921bc384459c6a858f2`.

## TDD Evidence

- **RED:** The focused target failed because `liquidfun::rope` did not exist.
- **GREEN:** The checked module and pure core made all eleven focused behaviors pass.
- **REFACTOR:** Added complete public error documentation, kept the public/core split cohesive, and reviewed source grouping against `b2Rope.cpp`.
- The failing RED state was not committed because repository Rust policy requires the complete passing pre-commit gate before every commit.

## Task Commits

Each task was committed atomically:

1. **Implement checked standalone rope initialization, stepping, angle control, and inspection** - `ab0f395` (feat)

## Files Created/Modified

- `crates/liquidfun/src/rope.rs` - Checked public definition, iteration, rope, error, and borrow-scoped inspection surface.
- `crates/liquidfun/src/rope/core.rs` - Private source-ordered owned simulation state and transactional constraint core.
- `crates/liquidfun/tests/standalone_rope.rs` - Fixed/free, damping, zero-step, iteration, oracle order, wrapping, bounds, and no-effect coverage.
- `crates/liquidfun/src/lib.rs` - Curates the standalone public `rope` deep module.
- `.codex/tasks/lessons.md` - Records that exact fixtures must be independently oracle-derived.

## Decisions Made

- Used a closed `RopeIterations` newtype with a reviewed maximum of 1024 while retaining source-supported zero iterations.
- Limited the owned definition to 4096 vertices to bound allocations and step work without exposing internal buffers.
- Preserved literal source angle wrapping by tau increments and added a bounded failure rather than permitting an unbounded loop for extreme finite targets.
- Kept the exact-bit order witness tied to a temporary driver linked against the repository's pinned debug `libliquidfun.a`; no Rust-produced output was used as authority.

## Deviations from Plan

None - plan scope and architecture were implemented as specified.

## Issues Encountered

- The first exact expected-bit values were provisional and therefore non-authoritative. They were replaced only after an independent pinned-C++ `b2Rope` driver produced the same four vertex bit pairs; the provenance is documented beside the assertion.
- Repository-wide `just markdown-check` stalled while its Python glob traversed generated/reference directories. The process was stopped and the only changed non-GSD Markdown file, `.codex/tasks/lessons.md`, passed targeted `mdformat --check`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 8 protocol and evidence plans can model standalone-rope requests and observations without introducing world, handle, body, joint, contact, island, or rendering dependencies.
- `JOIN-05` remains phase-level sign-off work: this plan supplies the native rope behavior but does not claim the later closed differential or canonical D1 gate by itself.

## Self-Check: PASSED

- All three planned artifacts exist and the task commit `ab0f395` is present.
- Focused standalone-rope tests pass 11/11, including independently pinned C++ exact bits.
- Formatting, warning-denied Clippy, all-target/all-feature build, all-feature tests, doctests, and diff checks pass using the clean temporary Cargo target directory.

***

*Phase: 08-joints-rope-callbacks-and-rigid-sign-off*
*Completed: 2026-07-13*
