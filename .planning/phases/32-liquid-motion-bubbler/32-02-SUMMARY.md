---
phase: 32-liquid-motion-bubbler
plan: "02"
subsystem: ui
tags: [catalog, solidjs, portrait-frame, readme-svg]

requires:
  - phase: 32-liquid-motion-bubbler
    provides: Native liquid-bubbler vessel with a static waist, a motor-off wheel, and a raised side-shaft plate
provides:
  - Watch-first liquid-bubbler catalog record after wave-tank
  - Static amber waist-and-wheel preview, phone frame, and empty-cue README plan
affects: [32-03-browser]

tech-stack:
  added: []
  patterns:
    - "Watch-first catalog scenes use withGravitySlider([]) and a center-click capture stub"
    - "Portrait bounds match viewBounds and include the plate at rest and at full stroke"

key-files:
  created: []
  modified:
    - web/src/catalog/scenes.ts
    - web/src/catalog/scene-records.ts
    - web/src/catalog/previews.tsx
    - web/src/catalog/portrait-bounds.ts
    - web/src/player/runtime.ts
    - web/scripts/readme-svg/plans.ts
    - web/scripts/demo-media/model.ts
    - web/e2e/player-helpers.ts
    - web/tests/scenes.test.ts
    - web/tests/scene-catalog-controls.test.ts
    - web/tests/scene-catalog-credits.test.ts
    - web/tests/portrait-bounds.test.ts
    - web/tests/demo-media-model.test.ts

key-decisions:
  - "Kept the plan view rectangle because gap, stroke, and walls were unchanged; the raised plate still sits inside it"
  - "Passing catalog unit tests are evidence only and do not approve the work"

patterns-established:
  - "liquid-bubbler follows wave-tank in SCENE_IDS, README plans, and capture plans"

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 32-2026-09-27T18-56-14
generated_at: 2026-09-27T19:58:59Z

duration: 7 min
completed: 2026-09-27
---

# Phase 32 Plan 02: Liquid Bubbler Catalog Summary

**Watch-first Liquid Bubbler catalog entry with an amber waist-and-wheel preview, a phone frame around the raised plate, and showcase-only credit**

## Performance

- **Duration:** 7 min
- **Started:** 2026-09-27T19:51:35Z
- **Completed:** 2026-09-27T19:58:59Z
- **Tasks:** 2
- **Files modified:** 13

## Accomplishments

- The catalog lists twenty-two ready scenes. `liquid-bubbler` follows `wave-tank`. The water-wheel record and its preview stay the jet-driven wheel.
- Liquid Bubbler is watch-first aside from the shared gravity slider. The static preview shows a narrow waist, an amber drip, and a wheel. The caption `Static preview` stays in `DemoNavigation`.
- The phone frame and `viewBounds` are `{ minX: -0.71, minY: -0.16, maxX: 1.12, maxY: 1.98 }`. The README plan is `{ id: "liquid-bubbler", cues: [] }`. Credits point at `scene/liquid_bubbler.rs` and the LiquidFun showcase link only.

## Task Commits

Each task was committed atomically:

1. **Task 1: Append the Liquid Bubbler catalog entry** - `37506bf` (feat)
2. **Task 2: Extend the catalog unit contracts** - `4f84fd7` (test)

**Plan metadata:** docs commit with this summary, STATE.md, and ROADMAP.md

## Files Created/Modified

- `web/src/catalog/scenes.ts` - `liquid-bubbler` after `wave-tank`
- `web/src/catalog/scene-records.ts` - Watch-first Liquid Bubbler record and showcase-only credit
- `web/src/catalog/previews.tsx` - Still waist, amber drip, and wheel
- `web/src/catalog/portrait-bounds.ts` - Phone rectangle matching `viewBounds`
- `web/src/player/runtime.ts` - Page summary says twenty-two demos
- `web/scripts/readme-svg/plans.ts` - `{ id: "liquid-bubbler", cues: [] }`
- `web/scripts/demo-media/model.ts` - Center-click capture stub
- `web/e2e/player-helpers.ts` - Hash path `#/scene/liquid-bubbler`
- `web/tests/scenes.test.ts` - Twenty-two scene order, title, ready count, and watch-first hint
- `web/tests/scene-catalog-controls.test.ts` - Gravity-only controls, including the recreating-id lock
- `web/tests/scene-catalog-credits.test.ts` - Scene path and showcase href
- `web/tests/portrait-bounds.test.ts` - Height fraction `0.40` and finished wall, wheel, and plate points
- `web/tests/demo-media-model.test.ts` - Capture plans end with `liquid-bubbler`

## Decisions Made

- Kept the plan rectangle `{ minX: -0.71, minY: -0.16, maxX: 1.12, maxY: 1.98 }` for `viewBounds` and the phone frame. Plan 01 changed wheel density and raised the plate `0.02` m. Gap, stroke, and walls stayed put, and the raised plate corners still sit inside that rectangle. Portrait wall points use the finished rest pose `(0.58, 0.02)` to `(0.94, 0.10)` and the stroke pose `(0.58, 1.22)` to `(0.94, 1.30)`.
- Did not approve this work. D-11 leaves independent review eligible. Passing Vitest is evidence only.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Extended the recreating-control lock for the new gravity slider**
- **Found during:** Task 2 (Extend the catalog unit contracts)
- **Issue:** `RECREATING_CONTROL_IDS` still listed the previous scene count. Liquid Bubbler's shared gravity slider recreates the scene, so the actual id list was one `gravity` longer.
- **Fix:** Appended one `"gravity"` entry. Did not add waist, color, or wheel controls.
- **Files modified:** `web/tests/scene-catalog-controls.test.ts`
- **Verification:** `bun run test:unit -- tests/scene-catalog-controls.test.ts` exits 0
- **Committed in:** `4f84fd7` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** The extra expected id keeps the existing recreates lock honest. No scene controls were added.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 32-03. The hash route `#/scene/liquid-bubbler` is on the catalog allowlist. Chromium play, pause, and reset smoke is still Plan 03. Passing catalog unit tests here are evidence only and are not an approval.

## Known Stubs

None.

---
*Phase: 32-liquid-motion-bubbler*
*Completed: 2026-09-27*

## Self-Check: PASSED

- FOUND: web/src/catalog/scenes.ts
- FOUND: web/src/catalog/scene-records.ts
- FOUND: web/src/catalog/previews.tsx
- FOUND: web/src/catalog/portrait-bounds.ts
- FOUND: web/scripts/readme-svg/plans.ts
- FOUND: .planning/phases/32-liquid-motion-bubbler/32-02-SUMMARY.md
- FOUND: 37506bf
- FOUND: 4f84fd7
