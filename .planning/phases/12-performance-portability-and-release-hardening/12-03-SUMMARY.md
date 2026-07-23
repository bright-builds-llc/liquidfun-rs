---
phase: 12-performance-portability-and-release-hardening
plan: "03"
subsystem: desktop-testbed
tags: [eframe, egui, session-command, comparison-ui, screenshots]
requires:
  - phase: 12-02
    provides: pinned eframe/egui stack and passive replacement-renderer contracts
  - phase: 11-30
    provides: compiled comparison-diagnostics lifecycle and generic-error isolation
provides:
  - eframe/egui desktop launcher with no legacy renderer imports
  - passive render-then-submit SessionCommand boundary
  - explicit fixed-time logical driver independent from UI repaint timing
  - retained catalog, comparison, provenance, settings, viewport, and screenshot behavior
affects: [phase-12-release-audit, testbed, desktop-ui, dependency-cleanup]
tech-stack:
  added: []
  patterns: [passive immediate-mode shell, begin-command then explicit submission, non-paint logical driver]
key-files:
  created: []
  modified:
    - crates/liquidfun-testbed/src/bin/interactive.rs
    - crates/liquidfun-testbed/src/interactive.rs
    - crates/liquidfun-testbed/tests/comparison_lifecycle.rs
    - crates/liquidfun-testbed/tests/controller_ui.rs
    - crates/liquidfun-testbed/tests/interactive.rs
key-decisions:
  - "Render one immutable egui frame, queue at most one UI effect, then translate it through AppShell and submit the resulting SessionCommand after rendering."
  - "Drive Running sessions from eframe's non-paint logic callback through the existing fixed-time accumulator; repaint requests never directly execute a physics step."
  - "Keep exact 35 percent matching-primitive opacity in the shell painter while reusing protocol projection and comparison authority."
patterns-established:
  - "Desktop command boundary: begin_action emits a closed SessionCommand without controller mutation, and submit_command is the only execution step."
  - "Desktop timing boundary: eframe UI paints and requests repaints, while drive_logical_time owns bounded deterministic catch-up."
requirements-completed: [API-12, DOCS-07, DOCS-09]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-23T22:18:35Z
duration: 29m
completed: 2026-07-23
---

# Phase 12 Plan 03: Eframe Desktop Shell Summary

**The private semantic testbed now runs through eframe/egui while all simulation mutations remain validated, controller-owned `SessionCommand` submissions and logical time remains separate from repaint cadence.**

## Performance

- **Duration:** 29m
- **Started:** 2026-07-23T21:49:35Z
- **Completed:** 2026-07-23T22:18:35Z
- **Tasks:** 1
- **Files modified:** 7

## Accomplishments

- Replaced the legacy desktop attribute, async frame loop, input polling, drawing, clipboard, close, and screenshot plumbing with `eframe::run_native`, one `eframe::App`, and egui APIs.
- Split UI admission from controller mutation: catalog selection, ordinary controls, and staged settings first emit a closed `SessionCommand`, pass it through `AppShell`, and only then submit it under the controller's monotonic identity.
- Preserved stable catalog identity, Pause/Run/Step/Restart/Capture meanings, validated Apply & Restart settings, typed scenario actions, semantic hit testing, pointer-anchored zoom, pan, responsive minimum-window behavior, comparison lifecycle, Overlay/Side-by-side modes, and exact 35% matching opacity.
- Preserved HTTPS-allowlisted provenance actions with visible copy fallback and target-confined diagnostic PNG capture.
- Reduced the launcher from 2,484 to 1,618 lines while keeping comparison and controller authority in existing modules.

## TDD Evidence

- **RED:** New controller tests failed because passive `begin_*`, explicit `submit_command`, and `drive_logical_time` APIs did not exist; the new launcher-source regression failed because the legacy shell had no eframe implementation.
- **GREEN:** Controller UI now proves emission does not mutate state before explicit submission and Running advances only after an explicit logical-driver call; launcher regression proves eframe wiring and rejects legacy/frame-time/direct-step tokens.
- **REFACTOR:** Consolidated command execution through one `dispatch_pending` path, retained compatibility convenience methods, and replaced ad hoc alpha math with exact integer 35% scaling.
- The plan prohibited a failing RED commit, so RED remained uncommitted.

## Task Commits

Each task was committed atomically:

1. **Task 1: Port the desktop shell without changing semantic authority** - `924dfe0` (feat)

## Files Created/Modified

- `crates/liquidfun-testbed/src/bin/interactive.rs` - Eframe/egui launcher, passive command dispatch, fixed-time logical driving, semantic painting, provenance, and screenshots.
- `crates/liquidfun-testbed/src/interactive.rs` - Separates typed command emission from explicit controller submission and names the logical-time driver.
- `crates/liquidfun-testbed/tests/comparison_lifecycle.rs` - Proves eframe wiring and excludes legacy/frame-time/direct-step authority.
- `crates/liquidfun-testbed/tests/controller_ui.rs` - Proves passive command emission and explicit logical-driver advancement.
- `crates/liquidfun-testbed/tests/interactive.rs` - Updates two launcher-source contracts from legacy toolkit details to eframe/controller behavior.
- `crates/liquidfun-testbed/src/capability/render.rs` - Mechanically centralizes reviewed fixture integer-to-renderer conversion for strict Clippy.
- `crates/liquidfun-testbed/src/ui/protocol_viewport.rs` - Mechanically collapses one nested optional-fill branch for strict Clippy.

## Decisions Made

- Kept eframe 0.35's `logic` and `ui` callbacks separate: `ui` renders and emits commands, while `logic` invokes only the existing bounded logical driver and comparison refresh.
- Preserved existing `InteractiveTestbed` convenience methods by composing the new begin/submit seam, avoiding a breaking change for headless and integration callers.
- Used egui's native screenshot event and the existing `DiagnosticScreenshotPath` confinement contract; screenshots remain explicitly diagnostic and cannot authorize compatibility claims.
- Reused the existing protocol projection and comparison model instead of moving physics or comparison authority into the renderer.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Replaced stale legacy-toolkit launcher assertions**

- **Found during:** Task 1 focused interactive verification.
- **Issue:** Two pre-existing static launcher tests required the removed legacy entry attribute and toolkit-specific focus/pointer tokens, directly contradicting the plan's no-legacy-source acceptance.
- **Fix:** With explicit parent approval, updated only those two assertions to require eframe lifecycle, egui input/accessibility, passive command submission, confined screenshots, and minimum-window actions.
- **Files modified:** `crates/liquidfun-testbed/tests/interactive.rs`
- **Verification:** All 12 interactive integration tests pass.
- **Committed in:** `924dfe0`

**2. [Rule 3 - Blocking] Cleared committed predecessor Clippy failures**

- **Found during:** Task 1 package-wide strict Clippy verification.
- **Issue:** Six reviewed fixture integer conversions triggered `cast_precision_loss`, and one passive AABB fill branch triggered `collapsible_if`, preventing the mandatory warnings-denied gate.
- **Fix:** With explicit parent approval, isolated the intended bounded fixture conversion behind one documented helper and mechanically collapsed the optional-fill branch without changing behavior.
- **Files modified:** `crates/liquidfun-testbed/src/capability/render.rs`, `crates/liquidfun-testbed/src/ui/protocol_viewport.rs`
- **Verification:** Capability 4/4, renderer contract 5/5, package-wide Clippy, and the exact full workspace gate pass.
- **Committed in:** `924dfe0`

**Total deviations:** 2 auto-fixed (2 blocking issues)

**Impact on plan:** Both deviations were narrowly approved and required to make the planned migration verifiable; neither added simulation authority or expanded the dependency surface.

## Issues Encountered

- Eframe 0.35 uses separate `App::logic` and `App::ui` callbacks plus the unified `Panel` API. The shell uses `logic` solely for the explicit fixed-time driver and keeps immediate-mode rendering in `ui`.
- Plan 12-18 still owns removing the now-unused legacy dependency and advisory waivers from the private dependency graph; this plan removes all migrated-shell imports and source usage.

## Known Stubs

None.

## Verification

- `cargo test -p liquidfun-testbed --test comparison_lifecycle --test controller_ui --test interactive --test renderer_contract --test capability` - 43 passed.
- `cargo clippy -p liquidfun-testbed --all-targets --all-features -- -D warnings` - passed.
- Source scan found no legacy renderer import, render-frame timing call, `next_frame`, or direct `World::step` in the migrated shell.
- `cargo tree -p liquidfun --edges normal` contains no eframe, egui, tiny-skia, or legacy renderer dependency.
- `cargo build --all-targets --all-features` compiled the native interactive binary.
- Exact ordered commit gate passed after final changes: `cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo build --all-targets --all-features`; `cargo test --all-features`.
- `git diff --cached --check` passed and implementation commit `924dfe0` contains exactly the seven approved task/deviation files.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The native shell is fully migrated and ready for Plan 12-18 to remove the unused private legacy dependency and both advisory waivers.
- Release and UI audits can rely on compiled command-boundary, comparison-lifecycle, renderer, capability, and integration regressions.

## Self-Check: PASSED

- Confirmed all seven implementation files and this summary exist.
- Confirmed task commit `924dfe0` exists.
- Confirmed the summary contains exactly two YAML frontmatter delimiters.
