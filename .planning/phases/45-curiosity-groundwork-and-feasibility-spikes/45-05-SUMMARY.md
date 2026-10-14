---
phase: 45-curiosity-groundwork-and-feasibility-spikes
plan: "05"
subsystem: web-catalog
tags: [scene-catalog, web, refactor, file-length, prep-02, d-11, d-12]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 45-2026-10-14T15-08-35
generated_at: "2026-10-14T16:18:27.114Z"
requires:
  - "45-01: pinned scene fingerprint baseline (web-only change; engine and bridge untouched)"
provides:
  - "scene-view-bounds.ts plus scene-records-classic/-testbed/-machines.ts; scene-records.ts spreads CLASSIC, TESTBED, MACHINE arrays into SCENES"
  - "preview-kit.tsx plus previews-classic/-testbed/-machines.tsx; previews.tsx is the ScenePreview switch only"
  - "never exhaustiveness guard in ScenePreview: a catalog id without a case fails to compile (W5)"
affects: [46, 47, 48, 49, 50, 51]
tech-stack:
  added: []
  patterns:
    - "Catalog family modules: each family exports a readonly SceneRecord[] array and the entry spreads them in catalog order"
    - "Exhaustive switch over SceneId closes with `const unreachable: never = props.sceneId; return unreachable;`"
key-files:
  created:
    - web/src/catalog/scene-view-bounds.ts
    - web/src/catalog/scene-records-classic.ts
    - web/src/catalog/scene-records-testbed.ts
    - web/src/catalog/scene-records-machines.ts
    - web/src/catalog/preview-kit.tsx
    - web/src/catalog/previews-classic.tsx
    - web/src/catalog/previews-testbed.tsx
    - web/src/catalog/previews-machines.tsx
  modified:
    - web/src/catalog/scene-records.ts
    - web/src/catalog/previews.tsx
key-decisions:
  - "Families: classic 7 (wave-machine..water-wheel), testbed 12 (particles..sparky), machines 6 (hydraulic-fountain..tesla-valve); v1.6 scenes go into scene-records-curiosity.ts / previews-curiosity.tsx created by Phase 46"
  - "Pure move proven by SCENES sha256 d64119508d2669a206c0837084ae8b3794e2f41db25f2896bd24cfb337a70ff3 before and after; scenes.ts and DemoNavigation.tsx unchanged"
  - "D-12: no other near-limit file is crossed by the first new scene, so no further split (controls.test.ts 529 is left for Phase 47 or 48)"
requirements-completed: []
duration: 5min
completed: 2026-10-14
---

# Phase 45 Plan 05: Catalog file splits Summary

**`scene-records.ts` (617 lines) and `previews.tsx` (562 lines) split by catalog family into modules of 20 to 247 lines. SCENES is byte-identical (same sha256). ScenePreview now fails to compile if a catalog id has no case.**

## Performance

- **Duration:** about 5 min
- **Started:** 2026-10-14T16:16:26Z
- **Completed:** 2026-10-14T16:21Z
- **Tasks:** 2
- **Files modified:** 10 (8 created, 2 reduced)

## Accomplishments

- `scene-records.ts` re-exports the five `*_VIEW_BOUNDS` from `scene-view-bounds.ts` and builds `SCENES` from `[...CLASSIC_SCENE_RECORDS, ...TESTBED_SCENE_RECORDS, ...MACHINE_SCENE_RECORDS]`. A comment there says new families append their arrays in catalog order. Record bodies moved verbatim.
- `previews.tsx` keeps `ScenePreviewProps` and the 25-case switch in the same order. The family modules export the preview components and `preview-kit.tsx` exports the palette, `PreviewFrame`, `VerticalWallBasin` and `SoupBasin`.
- Added the `never` guard. A negative check (temporarily removing `case "sparky"`) failed `tsc` with `TS2322: Type '"sparky"' is not assignable to type 'never'`. The file was then restored.

## SCENES pure-move proof

| When | sha256 of `JSON.stringify(SCENES)` | File |
| --- | --- | --- |
| Before | `d64119508d2669a206c0837084ae8b3794e2f41db25f2896bd24cfb337a70ff3` | `target/phase45/scenes-sha-before.txt` |
| After | `d64119508d2669a206c0837084ae8b3794e2f41db25f2896bd24cfb337a70ff3` | `target/phase45/scenes-sha-after.txt` |

## Module line counts (limit 628, target at most 560)

| Module | Lines | Contents |
| --- | --- | --- |
| `web/src/catalog/scene-records.ts` | 20 | entry: bounds re-export, SCENES spread |
| `web/src/catalog/scene-view-bounds.ts` | 71 | 5 view-bound constants |
| `web/src/catalog/scene-records-classic.ts` | 206 | 7 records |
| `web/src/catalog/scene-records-testbed.ts` | 244 | 12 records |
| `web/src/catalog/scene-records-machines.ts` | 127 | 6 records |
| `web/src/catalog/previews.tsx` | 95 | ScenePreview switch plus never guard |
| `web/src/catalog/preview-kit.tsx` | 60 | palette, frame, basins |
| `web/src/catalog/previews-classic.tsx` | 107 | 7 previews |
| `web/src/catalog/previews-testbed.tsx` | 247 | 12 previews |
| `web/src/catalog/previews-machines.tsx` | 114 | 6 previews |

## D-12 assessment (other near-limit files a new scene touches)

| File | Lines | Growth per new scene | Verdict |
| --- | --- | --- | --- |
| `web/tests/controls.test.ts` | 529 | about 25-35 per new live HUD control test | no split needed for the first new scene; crosses around the third control, so Phase 47 or 48 splits it if needed |
| `web/e2e/player-helpers.ts` | 580 | 1 | no split needed for the first new scene |
| `crates/liquidfun-wasm/src/session/tests.rs` | 580 | 1 | no split needed for the first new scene |
| `web/tests/scenes.test.ts` | 482 | about 6-8 | no split needed for the first new scene |
| `web/tests/portrait-bounds.test.ts` | 461 | about 10-15 | no split needed for the first new scene |

## Task Commits

1. **Task 1: Split scene-records.ts into view bounds plus three family modules** - `4bc65fc86` (refactor)
1. **Task 2: Split previews.tsx into a kit plus three family modules with an exhaustive switch** - `596132e8c` (refactor)

## Verification

- `bun run typecheck` (app and e2e projects) passed after each task.
- `bun run test:unit`: 119 files, 1844 tests passed after each task.
- `bun scripts/bright-builds-check.ts all`: `findings=0` (file-lengths scanned=1565 after Task 1).
- `git diff --color-moved=dimmed-zebra` review: the only non-moved lines are module docs, imports, the added `export` keywords, the SCENES spread and the never guard.
- `just web-build`, then `CI=true bunx playwright test e2e/shell.spec.ts`: 14 passed, including "shows one static preview per scene in the desktop sidebar" (`target/phase45/shell-e2e-05.log`).
- `web/src/catalog/scenes.ts` and `web/src/components/DemoNavigation.tsx` are unchanged.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Theo Jansen view-bounds doc comment re-attached to its constant**
- **Found during:** Task 1
- **Issue:** In the original `scene-records.ts`, the "Camera frame for Theo Jansen" doc block sat directly above the washing-machine doc block. It documented nothing, and `THEO_JANSEN_VIEW_BOUNDS` had no doc comment.
- **Fix:** Moved the same comment text, unchanged, onto `THEO_JANSEN_VIEW_BOUNDS` in `scene-view-bounds.ts`. It is a doc-only change with no runtime effect; the SCENES hash is unchanged.
- **Files modified:** `web/src/catalog/scene-view-bounds.ts`
- **Commit:** `4bc65fc86`

### Notes

- The acceptance check `grep -c 'id: "'` on the record modules gives 11 + 12 + 6 = 29, not 25. That is because four inline preset controls in the classic records also have `id:` fields (`water-amount`, `mix-strength`, `shape`, `softness`). The original file also gave 29. Counting record-level ids (`grep -c '^    id: "'`) gives 7 + 12 + 6 = 25, as intended.
- The plan's runtime check uses `just web-build` instead of `just web-wasm && bun run build:app`, following the 45-04 finding that e2e builds need `VITE_BUILT_AT`.

## Known Stubs

None.

## Next Phase Readiness

- Phase 46 adds `scene-records-curiosity.ts` (spread after `MACHINE_SCENE_RECORDS`) and `previews-curiosity.tsx`. It also adds the new `case`, which the never guard now requires.
- PREP-02 is not ticked here. Plan 10 or phase completion ticks it.

## Self-Check: PASSED

- All 10 module files exist; commits `4bc65fc86` and `596132e8c` are in `git log`.
