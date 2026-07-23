---
phase: 12-performance-portability-and-release-hardening
plan: "01"
subsystem: private-testbed-rendering
tags: [eframe, egui, tiny-skia, passive-renderer, deterministic-image]
requires:
  - phase: 11-examples-headless-tooling-and-testbed
    provides: renderer-neutral catalog/controller architecture and bounded Macroquad advisory obligation
provides:
  - crate-private passive window, input, drawing, text, clipboard, and image-capture contracts
  - deterministic bounded tiny-skia CPU image capture with owned RGBA and PNG bytes
  - executable renderer-authority and published-dependency isolation tests
affects: [phase-12-renderer-migration, testbed, dependency-hardening, release-readiness]
tech-stack:
  added: [eframe 0.35.0, egui 0.35.0, tiny-skia 0.12.0]
  patterns: [owned semantic presentation frames, validate-before-allocation, caller-owned image persistence]
key-files:
  created:
    - crates/liquidfun-testbed/src/renderer.rs
    - crates/liquidfun-testbed/src/renderer/image.rs
    - crates/liquidfun-testbed/tests/renderer_contract.rs
  modified:
    - crates/liquidfun-testbed/Cargo.toml
    - crates/liquidfun-testbed/src/lib.rs
    - Cargo.lock
key-decisions:
  - "Keep the replacement renderer crate-private and accept only owned semantic presentation values so it cannot mutate simulation or comparison authority."
  - "Validate physical dimensions and checked RGBA byte counts before tiny-skia allocation, with a closed 4096x4096 maximum."
  - "Return encoded PNG bytes without filesystem access so output-path validation and persistence remain caller-owned effects."
patterns-established:
  - "Passive renderer boundary: window/input/clipboard traits and drawing/image traits exchange renderer-domain values only."
  - "Deterministic CPU capture: fixed presentation input produces byte-identical RGBA and PNG output across independent renderer instances."
requirements-completed: [API-12, DOCS-07]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-23T18:48:00Z
duration: 1h 34m
completed: 2026-07-23
---

# Phase 12 Plan 01: Passive Replacement Renderer Contract Summary

**A crate-private owned-value renderer boundary and bounded tiny-skia backend now produce deterministic RGBA/PNG captures without simulation authority or published-crate dependency leakage.**

## Performance

- **Duration:** 1h 34m
- **Started:** 2026-07-23T17:13:37Z
- **Completed:** 2026-07-23T18:48:00Z
- **Tasks:** 1
- **Files modified:** 6

## Accomplishments

- Added exact private eframe 0.35.0, egui 0.35.0, and tiny-skia 0.12.0 pins while retaining the compiling Macroquad shell for the incremental migration.
- Defined crate-private bounded colors, logical/physical dimensions, semantic input, clipboard, primitive/text drawing, passive window, and image-rendering contracts.
- Implemented deterministic tiny-skia CPU capture with 4096x4096 limits, checked RGBA byte-count arithmetic, allocation failure handling, and caller-owned PNG persistence.
- Proved invalid-dimension rejection, byte-stable 640x480 output, passive-state isolation, absence of forbidden renderer authority, and an unchanged `liquidfun` normal dependency graph.

## TDD Evidence

- **RED:** The new renderer contract suite failed because `src/renderer.rs` did not exist.
- **GREEN:** The owned presentation contracts and tiny-skia backend made all five focused tests pass.
- **REFACTOR:** Checked bounded conversions replaced lossy casts, and the module path was made explicit so the crate-private source can be compiled directly by the integration contract.
- The plan explicitly prohibited committing a failing RED state, so RED remained uncommitted.

## Task Commits

Each task was committed atomically:

1. **Task 1: Define and prove the passive replacement renderer contract** - `3e27d5d` (feat)

## Files Created/Modified

- `crates/liquidfun-testbed/src/renderer.rs` - Owns the passive renderer domain types and crate-private adapter traits.
- `crates/liquidfun-testbed/src/renderer/image.rs` - Renders owned presentation frames to deterministic tiny-skia RGBA and PNG bytes.
- `crates/liquidfun-testbed/tests/renderer_contract.rs` - Proves bounds, determinism, authority isolation, passive state, and dependency isolation.
- `crates/liquidfun-testbed/Cargo.toml` - Pins the private replacement renderer stack exactly.
- `crates/liquidfun-testbed/src/lib.rs` - Compiles the staged crate-private renderer module without exposing it publicly.
- `Cargo.lock` - Locks the replacement desktop and CPU-image dependency graph.

## Decisions Made

- Used owned presentation frames and owned text rather than borrowing controller or comparison state across the renderer boundary.
- Kept image persistence outside the backend; the backend returns PNG bytes and cannot choose or write a filesystem path.
- Retained Macroquad and both existing advisory ignores because the compiling migration is intentionally completed by Plan 12-18.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- Repository `target/` artifacts inherited a macOS provenance attribute that caused codesign/dyld stalls. Verification completed successfully with one Cargo job and the unprovenanced temporary target `/tmp/liquidfun-phase12.OJRc0w`; no repository target artifact was committed.
- Full testbed all-target Clippy reached an unrelated pre-existing uncommitted `src/bin/interactive.rs` field-name lint. Plan-owned library and renderer-contract targets pass warnings-as-errors Clippy; the unrelated file was preserved and not staged.
- The diagnostic `cargo deny --locked check` reports advisories, bans, and sources clean, while license policy rejects newly reachable BSD-2-Clause, BSL-1.0, OFL-1.1, Ubuntu-font-1.0, CC0-1.0, and ISC transitive licenses. The plan requires this as a diagnostic baseline; later Phase 12 dependency/license hardening owns explicit policy review.

## Known Stubs

None. The staged renderer is intentionally crate-private for incremental migration, but its contract and CPU capture path are fully implemented and tested.

## Verification

- `cargo test -p liquidfun-testbed --test renderer_contract` - 5 passed.
- `cargo clippy -p liquidfun-testbed --lib --test renderer_contract --all-features -- -D warnings` - passed.
- `cargo tree -p liquidfun --edges normal` - contains only `bitflags`; no renderer dependency leakage.
- Exact ordered commit gate passed: `cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo build --all-targets --all-features`; `cargo test --all-features`.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The passive contract and deterministic CPU capture backend are ready for incremental eframe/egui consumer migration.
- Plan 12-18 remains responsible for removing the compiling Macroquad shell and both bounded advisory ignores.
- Later Phase 12 dependency hardening must review and encode the replacement graph's transitive license policy.

## Self-Check: PASSED

- Confirmed the summary and every declared key file exist.
- Confirmed task commit `3e27d5d` exists.
- Confirmed the summary contains exactly two YAML frontmatter delimiters.
