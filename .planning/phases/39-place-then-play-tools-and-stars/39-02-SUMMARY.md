---
phase: 39-place-then-play-tools-and-stars
plan: "02"
subsystem: web-puzzle-editor-core
tags: [typescript, puzzle, editor, hit-test, copy, functional-core, TOOL-01, TOOL-02, TOOL-03]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 39-2026-10-12T22-34-34
generated_at: "2026-10-13T00:10:05Z"
requires:
  - phase: 39-place-then-play-tools-and-stars
    provides: Rust PlacementStatus wire codes 0..12 and 8-float outline lane (39-01)
  - phase: 37-puzzle-core-in-the-bridge
    provides: web/src/puzzle/placements.ts PUZZLE_TOOL_CODES, PuzzlePlacement, encodePlacementLanes
provides:
  - web/src/puzzle/placement-status.ts PLACEMENT_STATUS, PlacementStatusCode, Quad, PlacementInspection, isPlacementStatusCode, isGeometricStatus, placementReason
  - web/src/puzzle/editor.ts BuildingEditor, EMPTY_EDITOR and pure edit operations (arm, place, select, move, rotate, nudge, remove, clear, applyInspection, validity gates, cycleSelection, canonicalPlacements, grid and angle helpers)
  - web/src/puzzle/hit-test.ts HIT_SLOP_CSS_PX, distanceToQuad, pickTool
  - web/src/puzzle/editor-format.ts trayItems, statusLine, buildingMessage, editor announcements
affects:
  - 39-03 and later shell plans (reducer, overlay, pointer, HUD wire these pure functions)
  - 39-06 (status-code parity test against Rust)
tech-stack:
  added: []
  patterns:
    - pure editor operations return the same object when nothing changes
    - two inspections, a live one for tint, outlines and hit-testing and a committed one for copy and Play (UI-D14)
key-files:
  created:
    - web/src/puzzle/placement-status.ts
    - web/src/puzzle/editor.ts
    - web/src/puzzle/hit-test.ts
    - web/src/puzzle/editor-format.ts
    - web/tests/puzzle-placement-status.test.ts
    - web/tests/puzzle-editor.test.ts
    - web/tests/puzzle-hit-test.test.ts
    - web/tests/puzzle-editor-format.test.ts
  modified: []
key-decisions:
  - "TS PLACEMENT_STATUS mirrors the 39-01 Rust codes exactly (0 valid .. 12 overlapsNoPlace); the plan literals already matched, so there was no deviation"
  - "applyInspection drops answers whose revision, status count or outline count does not match the editor (T-39-07)"
  - "clearTools on an editor with no tools only clears selection and arm, and returns the same object when neither is set"
  - "validityChangeAnnouncement throws when a tool becomes invalid without a reason instead of inventing copy"
requirements-completed: []
duration: 3min
completed: 2026-10-13
---

# Phase 39 Plan 02: Pure Editor Core Summary

**The pure TypeScript editing core now exists: status codes 0 to 12 match Rust, a budget-aware building editor keeps separate live and committed inspections, a 22 px nearest-outline hit test is in place, and the tray, status line and announcements use the UI-SPEC copy. Nothing touches the DOM, wasm or frames.**

## Performance

- **Duration:** about 3 min
- **Started:** 2026-10-13T00:06:54Z
- **Completed:** 2026-10-13T00:10:05Z
- **Tasks:** 2 of 2
- **Files created:** 8 (4 source, 4 test)

## Accomplishments

- `placement-status.ts`: `PLACEMENT_STATUS` holds codes 0 to 12, matching Rust `validate.rs` from 39-01. `placementReason` maps each invalid code to its UI-SPEC phrase and throws for 0. `isGeometricStatus` is false only for 1, 4 and 5, the codes whose outline lane is zeros.
- `editor.ts`: a `BuildingEditor` record and pure operations:
  - `armKind` toggles, and does nothing when the kind has none left.
  - `placeTool` places at angle 0, selects the new tool, disarms and bumps `revision`.
  - Move, rotate (mod 24) and nudge accept only integer cells. A non-integer throws "Invalid editor move".
  - `applyInspection` in live mode replaces only `maybeInspection`; in commit mode it replaces both inspections.
  - `allPlacementsValid` is the Play gate and needs a fresh commit. `committedPlacementsValid` is for display and ignores the revision.
  - `canonicalPlacements` sorts by (kind code, x, y, angle).
  - `cycleSelection` returns undefined past either end.
  - Grid snapping and the build-area centre helpers work in 0.05 m cells.
- `hit-test.ts`: `distanceToQuad` is 0 inside a quad of either winding. `pickTool` picks the nearest outline within `22 / pixelsPerMeter` m. Ties go to the selected tool, then to the highest id. Tools with no outline are skipped, and a non-finite or non-positive scale returns undefined.
- `editor-format.ts`:
  - `trayItems` is generic over the kind string and was tested with a fake plank/pin budget (D-06).
  - `statusLine` reads only the committed snapshot (UI-D14).
  - `buildingMessage` checks, in order: status, armed hint, empty hint, goal.
  - Every editor announcement uses the UI-SPEC copy, with U+2212 for negative angles.

## Task Commits

1. **Task 1 RED:** `cbd05c67a` (test). Failing tests for the status codes and the building editor.
1. **Task 1 GREEN:** `b62c92435` (feat). Placement status codes and the pure building editor.
1. **Task 2 RED:** `dfe13581c` (test). Failing tests for hit-testing and the editor copy.
1. **Task 2 GREEN:** `0262e8b5e` (feat). Tool hit-testing and the editor view models.

No refactor commits were needed.

## Verification

- `bunx vitest run tests/puzzle-placement-status.test.ts tests/puzzle-editor.test.ts`: 78 passed.
- `bunx vitest run tests/puzzle-hit-test.test.ts tests/puzzle-editor-format.test.ts`: 48 passed.
- Full `bunx vitest run`: 78 files, 915 tests passed.
- `bunx tsc --noEmit`: exit 0 (strict, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`).
- `bun scripts/bright-builds-check.ts all`: 0 findings.
- Acceptance greps all matched:
  - `overlapsNoPlace: 12`
  - `"In a no-build zone"`
  - `export function canonicalPlacements`
  - `HIT_SLOP_CSS_PX = 22`
  - both status-line and empty-hint strings
  - `"pin"` appears 4 times in the format test
  - the `innerHTML|Date.now|performance.now|setTimeout|setInterval` grep over `web/src/puzzle/*.ts` printed nothing
- RED runs failed because the modules did not exist yet, as expected.
- The web package has no lint, format or build script. Type checking and Vitest are its applicable checks.

## Decisions Made

See `key-decisions` in the frontmatter.

## Deviations from Plan

None. The plan ran as written. The plan's status-code literals already matched the 39-01 Rust codes.

Small choices the plan left open:

- `clearTools` on an empty editor goes through `clearInteraction`. An empty editor with nothing selected or armed returns the same object, which the plan requires. An empty editor that still has an armed kind gets disarmed.
- `snapWorldToGrid` throws when the grid is not a positive fraction of a meter, and it folds `-0` to `0`.
- `validityChangeAnnouncement(true, false, undefined)` throws instead of returning made-up copy, so it fails visibly.

## Requirements

TOOL-01, TOOL-02 and TOOL-03 get pure, tested logic here. The tray, gestures and Play gating are not wired into the UI yet; later Phase 39 plans do that. TOOL-01 and TOOL-02 therefore stay Pending in REQUIREMENTS.md, and TOOL-03 was already marked Complete by 39-01.

## Issues Encountered

None.

## Known Stubs

None. These modules are not imported by the app yet; later shell plans in this phase wire them.

## Threat Flags

None. T-39-06 is mitigated: positions must be safe integers, angles wrap mod 24 and `placeTool` enforces the budget. T-39-07 is mitigated: `applyInspection` drops stale or mismatched answers. Neither file adds a network, storage or DOM surface.

## Next Phase Readiness

- The reducer, overlay, pointer and HUD plans can import `editor.ts`, `hit-test.ts` and `editor-format.ts` directly.
- Decoding inspector output means mapping `statuses()` through `isPlacementStatusCode` and building a `Quad` per 8-float stride when `isGeometricStatus` holds.

## Self-Check: PASSED
