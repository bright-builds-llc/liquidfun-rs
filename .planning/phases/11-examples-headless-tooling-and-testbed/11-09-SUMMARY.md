---
phase: 11-examples-headless-tooling-and-testbed
plan: "09"
subsystem: renderer-neutral-debug-draw
tags: [debug-draw, observations, stable-identity, bounded-collections, headless]
requires:
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "08"
    provides: Bounded owned world observations with stable semantic identities
provides:
  - One authoritative owned renderer-neutral debug-primitive collection
  - Stable semantic keys for shapes, joints, contacts, particles, AABBs, centers, and labels
  - A narrow passive sink adapter over the already-collected primitive model
affects: [phase11-headless-runner, phase11-testbed, phase11-screenshots, phase11-evidence]
tech-stack:
  added: []
  patterns:
    - Build display geometry only from one bounded owned public world observation
    - Preserve source-significant order and canonicalize only declared unordered AABB observations
key-files:
  created:
    - crates/liquidfun/src/debug_draw.rs
    - crates/liquidfun/src/debug_draw/primitive.rs
    - crates/liquidfun/src/debug_draw/collector.rs
    - crates/liquidfun/src/debug_draw/collector/layers.rs
    - crates/liquidfun/src/debug_draw/collector/support.rs
    - crates/liquidfun/tests/debug_draw.rs
  modified:
    - crates/liquidfun/src/world/observation.rs
    - crates/liquidfun/src/world.rs
    - crates/liquidfun/src/lib.rs
key-decisions:
  - "Extend the public owned observation with body, fixture, joint, and particle snapshots so debug collection never traverses private storage."
  - "Expose one owned primitive collection as authority and make the optional sink a passive replay adapter rather than a second collection path."
  - "Use semantic owner, layer, kind, child, and occurrence/canonical ordinal for primitive identity; never expose private arena, proxy, or dense-row coordinates."
patterns-established:
  - "Primitive boundary: observe once, validate finite geometry and aggregate bounds before publishing, then optionally replay exact borrowed records."
  - "Frame boundary: every collection starts empty and owns all geometry, preventing stale-frame leakage."
requirements-completed: [RIGD-10, EXMP-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-22T01:13:00Z
duration: 18 min
completed: 2026-07-21
---

# Phase 11 Plan 09: Renderer-Neutral Debug Primitives Summary

**The engine now produces one bounded owned debug frame with stable semantic keys and explicit finite geometry for rigid bodies, joints, contacts, particles, broad-phase bounds, centers of mass, and inert labels without any renderer or private-storage dependency.**

## Performance

- **Duration:** 18 min
- **Started:** 2026-07-22T00:54:35Z
- **Completed:** 2026-07-22T01:13:00Z
- **Tasks:** 1
- **Files modified:** 9

## Accomplishments

- Added a closed public primitive vocabulary for points, segments, polylines, circles, transform axes, AABBs, arrows, and inert text labels with explicit style metadata.
- Added stable semantic primitive keys covering owner, layer, kind, shape child, source occurrence, and canonical ordinal without exposing arena slots, tree proxies, contact-manager positions, or dense particle rows.
- Added `World::collect_debug_primitives` as the one authoritative observation-to-frame path, with finite geometry validation and inclusive reviewed limits for primitive count, aggregate vertices, per-primitive vertices, label bytes, and aggregate text bytes.
- Added a passive sink that replays the exact owned collection without recollecting or granting mutation authority.
- Added public behavior coverage for all required layers, repeated deterministic collection, finite geometry, canonical AABB ordinals, exact-limit acceptance, first-excess rejection, sink equivalence, fresh empty frames, inert labels, and renderer-free dependencies.

## TDD Evidence

- **RED:** No valid implementation RED was captured because the production collector patch landed before the test file. Initial focused runs exposed two test compilation defects, followed by a behavior failure where the test joint's default `collide_connected = false` suppressed the expected rigid contact.
- **GREEN:** The fixture explicitly enabled connected-body collision, after which all three focused debug-draw behavior tests passed. The exact ordered full-workspace format, Clippy, build, test, and doctest gate also passed.
- **REFACTOR:** The original 882-line collector was split into a 247-line public API, a 481-line layer converter, and a 214-line finite/key/style support module while preserving one observation call and one collector traversal.

## Task Commits

Each task was committed atomically:

1. **Task 1: Build one authoritative renderer-neutral debug primitive layer** - `6cf9319` (feat)

**Plan metadata:** committed separately with this summary.

## Files Created/Modified

- `crates/liquidfun/src/debug_draw/primitive.rs` - Defines stable semantic owners, layer/kind keys, explicit style metadata, and the closed owned primitive vocabulary.
- `crates/liquidfun/src/debug_draw/collector.rs` - Defines reviewed limits, options, typed errors, the authoritative collection, passive sink, and public world entrypoint.
- `crates/liquidfun/src/debug_draw/collector/layers.rs` - Converts one owned observation into required source-ordered and canonicalized primitive layers.
- `crates/liquidfun/src/debug_draw/collector/support.rs` - Centralizes finite validation, semantic lookup, styling, stable canonical hashing, and bounded accounting helpers.
- `crates/liquidfun/src/world/observation.rs` - Adds owned body, fixture, joint, and particle observations required to avoid private collector traversal.
- `crates/liquidfun/tests/debug_draw.rs` - Proves complete stable behavior through the public all-feature API.
- `crates/liquidfun/src/debug_draw.rs`, `crates/liquidfun/src/world.rs`, and `crates/liquidfun/src/lib.rs` - Curate the new public surface.

## Decisions Made

- The collector accepts no callback that can influence simulation. Consumers either inspect the owned slice or pass it to a passive borrowed-record sink.
- Source-significant fixture, particle, joint, rigid-contact, and particle-contact order is retained. Broad-phase observations are the only declared unordered input and receive deterministic canonical ordinals.
- Body, fixture, joint, and particle geometry first crosses the bounded public observation boundary as owned semantic state; the debug module never reads world arenas, contact manager storage, broad-phase proxies, or particle rows.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical Functionality] Extended the owned observation boundary**

- **Found during:** Task 1 collector design
- **Issue:** Plan 11-08 exposed contacts, AABBs, and particle statistics but not owned body transforms, fixture shapes, joint anchors, or particle positions required to construct the complete debug frame without private traversal.
- **Fix:** Added bounded owned body, fixture, joint, and particle observation records and collected them through the existing public observation transaction.
- **Files modified:** `crates/liquidfun/src/world/observation.rs`, `crates/liquidfun/src/world.rs`, `crates/liquidfun/src/lib.rs`
- **Commit:** `6cf9319`

### Process Deviations

**2. TDD production-before-test ordering**

- The initial collector patch landed before the focused test file, so an implementation RED was not fabricated. The later observed failures and final GREEN status are recorded exactly above.
- The plan still has a single atomic task commit, matching its explicit no-intentionally-failing-commit instruction.

## Issues Encountered

- The first behavior run lacked rigid contact primitives because the test distance joint used the engine's correct default of suppressing collision between connected bodies. Enabling `collide_connected` in the fixture produced the intended contact witness.
- The shared worktree contained four unrelated pre-existing edits. They remained unstaged and were not committed or reverted.

## Security Verification

- Every output record is owned, finite, explicitly typed, and bounded before publication; failures expose only closed semantic resource/layer categories and configured limits.
- Stable keys contain only public semantic handles and closed coordinates. No pointer, arena slot, proxy ID, contact-manager index, dense particle row, renderer command, markup, path, process, network, or secret crosses the boundary.
- Labels are fixed bounded plain text with control characters rejected by primitive validation.
- The published crate gained no dependency, renderer integration, file access, network endpoint, authentication path, foreign code, or `unsafe` surface.
- No unresolved high-severity OWASP ASVS L1 or STRIDE finding remains.

## Known Stubs

None.

## Requirements Status

Plan 11-09's `RIGD-10` and `EXMP-05` mappings are achieved by the implementation and retained in summary frontmatter. Their global requirement checkboxes remain intentionally unchanged until later Phase 11 integration and evidence plans verify the complete end-to-end requirement scope.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Headless and visual consumers can share the exact same stable owned primitive collection without recollection or simulation authority.
- Later testbed and evidence plans can add rendering, camera controls, screenshots, and inspection UI strictly outside the engine crate.
- Phase execution state must continue to respect the earliest incomplete wave-order plan rather than treating Plan 11-09's out-of-order completion as phase completion.

## Self-Check: PASSED

- Confirmed all six created files and three modified source files exist.
- Confirmed task commit `6cf9319` exists and contains only the nine Plan 11-09 artifacts.
- Confirmed the focused three-test target and exact ordered full-workspace format, Clippy, build, test, and doctest gate pass with `/tmp/liquidfun-rs-phase11-11-09`.
- Confirmed no known stub or unplanned threat surface was introduced and all four unrelated shared-tree edits remain unstaged.

*Phase: 11-examples-headless-tooling-and-testbed*
*Completed: 2026-07-21*
