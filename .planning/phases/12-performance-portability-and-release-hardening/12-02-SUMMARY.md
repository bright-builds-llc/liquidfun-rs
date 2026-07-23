---
phase: 12-performance-portability-and-release-hardening
plan: "02"
subsystem: private-testbed-rendering
tags: [eframe, egui, tiny-skia, capability-matrix, viewport-projection]
requires:
  - phase: 12-performance-portability-and-release-hardening
    provides: crate-private passive renderer contract and deterministic tiny-skia image backend
provides:
  - deterministic 640x480, 800x600, and 1280x960 replacement-renderer capability captures
  - renderer-contract projection for native and protocol semantic viewport primitives
  - Phase 12 adapter identity, capability profile, passive-state checks, and independent hash evidence
affects: [phase-12-desktop-shell-migration, testbed, capability-evidence, release-readiness]
tech-stack:
  added: []
  patterns: [semantic display-list projection, deterministic CPU capability capture, passive presentation effects]
key-files:
  created: []
  modified:
    - crates/liquidfun-testbed/src/capability/render.rs
    - crates/liquidfun-testbed/src/capability/input.rs
    - crates/liquidfun-testbed/src/capability/report.rs
    - crates/liquidfun-testbed/src/ui/viewport/draw.rs
    - crates/liquidfun-testbed/src/ui/protocol_viewport.rs
    - crates/liquidfun-testbed/tests/capability.rs
key-decisions:
  - "Keep the Phase 11 fixture profile immutable while recording the replacement capability run under the separate phase12-v1 profile."
  - "Express viewport drawing as source-ordered passive renderer commands so comparison opacity and semantic record identity remain outside simulation authority."
  - "Regenerate confidence from two independent replacement-renderer runs instead of rewriting the read-only Phase 11 capability report."
patterns-established:
  - "Capability capture: logical 640x480 presentation frames scale deterministically to each exact physical evidence size."
  - "Viewport migration: validated semantic records project in source order to crate-private passive draw commands."
requirements-completed: [API-12, DOCS-07]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-23T20:05:58Z
duration: 11min
completed: 2026-07-23
---

# Phase 12 Plan 02: Capability Capture and Viewport Projection Summary

**The Phase 11 capability matrix and semantic viewports now use deterministic tiny-skia capture and passive replacement-renderer commands with exact adapter identity, stable ordering, and 35% comparison opacity.**

## Performance

- **Duration:** 11 min
- **Started:** 2026-07-23T19:55:03Z
- **Completed:** 2026-07-23T20:05:58Z
- **Tasks:** 1
- **Files modified:** 7

## Accomplishments

- Replaced capability image, color, and keyboard dependencies with the Plan 12-01 semantic renderer contract and exact `eframe-egui-0.35.0+tiny-skia-0.12.0` adapter identity.
- Produced deterministic PNG evidence at 640x480, 800x600, and 1280x960 while preserving all 20 capability rows, five structural profile names without durations, accessibility evidence, and zero controller/capture effects.
- Converted native and protocol viewport primitives, labels, focus halos, synchronized overlays, and side-by-side differences into source-ordered passive draw commands.
- Preserved exact-match comparison opacity at 35% and retained semantic identity/order in the authoritative projected records.
- Proved independent capability runs produce identical artifact hashes and that the published `liquidfun` crate has no renderer dependency leakage.

## TDD Evidence

- **RED:** The replacement contract test failed on the old adapter identity and found legacy renderer imports in five migrated modules.
- **GREEN:** The replacement capture, input translation, report identity, and viewport command projection made all focused capability and interactive tests pass.
- **REFACTOR:** A small logical raster command builder keeps the established capability scene concise while delegating physical scaling and PNG encoding to tiny-skia.
- The plan prohibited committing a failing RED state, so RED remained uncommitted.

## Task Commits

Each task was committed atomically:

1. **Task 1: Port capability capture and semantic viewport drawing** - `0bbafa0` (feat)

## Files Created/Modified

- `crates/liquidfun-testbed/src/capability.rs` - Removes stale legacy-renderer module documentation required by the source exclusion check.
- `crates/liquidfun-testbed/src/capability/input.rs` - Maps the six capability shortcuts through semantic renderer keys.
- `crates/liquidfun-testbed/src/capability/render.rs` - Builds logical presentation commands and captures exact physical PNGs through tiny-skia.
- `crates/liquidfun-testbed/src/capability/report.rs` - Records the Phase 12 profile and exact replacement adapter/stack identity.
- `crates/liquidfun-testbed/src/ui/viewport/draw.rs` - Projects native semantic primitives to passive renderer commands.
- `crates/liquidfun-testbed/src/ui/protocol_viewport.rs` - Projects protocol primitives and comparison cues without immediate-mode renderer calls.
- `crates/liquidfun-testbed/tests/capability.rs` - Checks adapter identity, profile separation, artifact names, independent hashes, passive effects, and source exclusions.

## Decisions Made

- Kept `phase11-v1` as the immutable input fixture profile and introduced `phase12-v1` as the capability-run profile, avoiding a false claim that the inherited oracle fixture had changed.
- Used the existing tiny-skia command vocabulary for viewport migration; closed polylines preserve ordered outlines while richer fills remain a desktop-shell presentation concern.
- Left the Phase 11 `CAPABILITY.md` evidence read-only and established current determinism by comparing hashes from two independent Phase 12 runs.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Removed stale renderer references from the capability module entrypoint**

- **Found during:** Task 1 (Port capability capture and semantic viewport drawing)
- **Issue:** The required source exclusion scans the whole `src/capability` module, but unlisted `src/capability.rs` still described the old renderer in two documentation lines.
- **Fix:** Updated only those two documentation lines; no module wiring or behavior changed.
- **Files modified:** `crates/liquidfun-testbed/src/capability.rs`
- **Verification:** The exact source-exclusion command and focused integration test both pass.
- **Committed in:** `0bbafa0`

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Documentation-only scope expansion was necessary for the plan's explicit source exclusion and introduced no behavioral scope creep.

## Issues Encountered

- The first combined verification shell ended with status 1 because its dependency-count expression treated an empty `rg` result as an empty string. Re-running the dependency assertion with an explicit conditional passed; all Rust tests in the same command had already passed.

## Known Stubs

- The current legacy desktop shell invokes the migrated public draw entrypoints but cannot consume the crate-private renderer frame yet. The functions build passive display lists and intentionally leave shell presentation to the next desktop-shell migration plan; capability capture and viewport projection themselves are fully implemented and tested.

## Verification

- Exact ordered commit gate passed: `cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo build --all-targets --all-features`; `cargo test --all-features`.
- `cargo test -p liquidfun-testbed --test capability` - 4 passed.
- `cargo test -p liquidfun-testbed --test interactive` - 12 passed, including the pre-existing shared interactive-test additions.
- Source exclusion passed for `src/capability`, `src/ui/viewport/draw.rs`, and `src/ui/protocol_viewport.rs`.
- `cargo tree -p liquidfun --edges normal` contains no eframe, egui, tiny-skia, or legacy renderer dependency; the private testbed resolves exact eframe 0.35.0, egui 0.35.0, and tiny-skia 0.12.0.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The capability and viewport layers are renderer-independent, leaving the desktop event shell as the remaining interactive migration boundary.
- The pre-existing uncommitted interactive-test and shell work was preserved and excluded from this plan's commit.

## Self-Check: PASSED

- Confirmed the summary and all seven task files exist.
- Confirmed task commit `0bbafa0` exists.
- Confirmed the summary contains exactly two YAML frontmatter delimiters.

*Phase: 12-performance-portability-and-release-hardening*
*Completed: 2026-07-23*
