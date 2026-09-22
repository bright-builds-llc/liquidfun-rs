---
phase: 28-interaction-seams
plan: "07"
subsystem: web-testing
tags: [vitest, catalog, SCENE_CAPTURE_PLANS, ACT-01, ACT-02, ACT-03, ACT-04, ACT-05]

requires:
  - phase: 28-interaction-seams
    provides: Sixteen-scene catalog records, controls, and pinned credits from Plan 06
provides:
  - Sixteen-scene Vitest catalog contract with interaction controls and credits
  - SCENE_CAPTURE_PLANS synchronized with sixteen SCENE_IDS
affects:
  - 28-08 Chromium player-smoke

tech-stack:
  added: []
  patterns:
    - Watch-first soup/wave-machine use center-click SceneAction stubs in capture plans
    - Interactive stirrer/impulse use click gestures; theo keeps a click stub because SceneAction has no control kind

key-files:
  created: []
  modified:
    - web/tests/scenes.test.ts
    - web/scripts/demo-media/model.ts
    - web/tests/demo-media-model.test.ts

key-decisions:
  - "navigation.test.ts needed no edits — it already maps SCENE_IDS dynamically"
  - "Used bun run test:unit because package.json has no test script"

patterns-established:
  - "Pattern: Sixteen-scene Vitest asserts order, UI-SPEC copy, control ids, and pin 7f204021…"
  - "Pattern: Capture coverage test asserts last five interaction ids plus click stubs"

requirements-completed: [ACT-01, ACT-02, ACT-03, ACT-04, ACT-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 28-2026-09-22T12-56-07
generated_at: 2026-09-22T14:34:42Z

duration: 3min
completed: 2026-09-22
---

# Phase 28 Plan 07: Sixteen-Scene Catalog Contract Summary

**Vitest locks sixteen SCENE_IDS with interaction controls and pinned credits; SCENE_CAPTURE_PLANS covers all five new scenes in order**

## Performance

- **Duration:** 3 min
- **Started:** 2026-09-22T14:31:27Z
- **Completed:** 2026-09-22T14:34:42Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Expanded `scenes.test.ts` from eleven to sixteen locked ids, titles, ready count, and watch-first set including soup and wave-machine
- Asserted Soup Stirrer `toggle-paddle-rail`, Impulse `push-mode` force/impulse, Theo `motor-direction` forward/reverse, and ten pinned interaction credit URLs at `7f204021…`
- Appended capture plans for soup, soup-stirrer, impulse, wave-machine, and theo-jansen with `interactionStep: 180` click SceneActions
- Strengthened `demo-media-model.test.ts` to require length 16 and last-five interaction coverage

## Task Commits

Each task was committed atomically:

1. **Task 1: Expand scenes.test.ts to sixteen scenes and controls** - `9404c48` (test)
2. **Task 2: Sync SCENE_CAPTURE_PLANS for five interaction scenes** - `60b8ee7` (feat)

**Plan metadata:** `6e391ee` (docs: complete plan)

## Files Created/Modified

- `web/tests/scenes.test.ts` - Sixteen-scene order, controls, credits, watch-first set
- `web/scripts/demo-media/model.ts` - Five interaction SCENE_CAPTURE_PLANS entries
- `web/tests/demo-media-model.test.ts` - Explicit sixteen-coverage and last-five asserts

## Decisions Made

- Left `navigation.test.ts` unchanged — routes already follow `SCENE_IDS`
- Verification used `bun run test:unit` (plan text said `bun run test`, which is not a package script)
- Catalog implementation stayed Plan 06; this plan only locked tests and capture stubs

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Used test:unit instead of nonexistent test script**
- **Found during:** Task 1
- **Issue:** `bun run test` hits the shell `test` builtin / missing script and fails
- **Fix:** Ran `bun run test:unit` matching Phase 27 Plan 05 evidence
- **Files modified:** none (command only)
- **Verification:** Unit suites exit 0
- **Committed in:** n/a (docs note only)

---

**Total deviations:** 1 auto-fixed (1× Rule 3)
**Impact on plan:** Command alias only; acceptance criteria otherwise met.

## Issues Encountered

None beyond the script-name mismatch above.

## Known Stubs

- Capture SceneActions for watch-first soup/wave-machine and control-only theo-jansen are intentional center/stub clicks so the capture script still emits a required `SceneAction` (WASM pointer no-op or control-driven scenes). Not product UI stubs.

## Threat Flags

None — implementation paths stay host-locked; tests assert no `google/liquidfun` in implementation paths; inspiration URLs remain pinned.

## Self-Check: PASSED

- FOUND: `web/tests/scenes.test.ts`
- FOUND: `web/scripts/demo-media/model.ts`
- FOUND: `web/tests/demo-media-model.test.ts`
- FOUND: `.planning/phases/28-interaction-seams/28-07-SUMMARY.md`
- FOUND: commit `9404c48`
- FOUND: commit `60b8ee7`

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Catalog Vitest contract and capture plans ready for Plan 08 Chromium smoke
- Plan 08 must update `SCENE_HASH_PATHS`, shell sixteen-demo asserts, and player-smoke timeouts as needed
- Keep `MAX_ADVANCE_STEPS` at 4

---
*Phase: 28-interaction-seams*
*Completed: 2026-09-22*
