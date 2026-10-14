---
phase: 45-curiosity-groundwork-and-feasibility-spikes
plan: "04"
subsystem: web-catalog-tests
tags: [scene-catalog, web, e2e, prep-02, d-08, d-09, d-10]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 45-2026-10-14T15-08-35
generated_at: "2026-10-14T16:14:52.288Z"
requires:
  - "45-01: pinned scene fingerprint baseline (web-only change; engine and bridge untouched)"
provides:
  - "Number-free PAGE_SUMMARY footer copy ('Every demo runs ...')"
  - "Web unit tests and shell e2e derive scene counts from SCENE_IDS.length and the last scene from the catalog"
  - "shell.spec.ts FOCUSABLE_COUNT = SCENE_IDS.length + 2 and LAST_SCENE_TITLE from SCENES.at(-1)"
affects: [46, 47, 48, 49, 50, 51]
tech-stack:
  added: []
  patterns:
    - "e2e imports copy constants from src instead of re-typing user-facing text"
    - "Module-level SCENES.at(-1) with an explicit undefined guard under noUncheckedIndexedAccess"
key-files:
  created: []
  modified:
    - web/src/player/runtime.ts
    - web/src/components/PlaygroundShell.tsx
    - web/tests/scenes.test.ts
    - web/tests/demo-media-model.test.ts
    - web/e2e/shell.spec.ts
key-decisions:
  - "45-04: PAGE_SUMMARY is number-free ('Every demo runs this repository's engine through WebAssembly.'); shell e2e imports it; web unit/e2e counts derive from SCENE_IDS.length, the drawer focus cycle is SCENE_IDS.length + 2 and the last link title comes from SCENES.at(-1)"
requirements-completed: []
duration: 3min
completed: 2026-10-14
---

# Phase 45 Plan 04: Number-free Web Copy and Catalog-derived Counts Summary

**The footer copy no longer names a scene count, and every web unit and shell e2e count now comes from `SCENE_IDS` / `SCENES`. Adding a scene needs no edit to "twenty-five" or `25` anywhere in the web tree.**

## Performance

- **Duration:** about 3 min
- **Started:** 2026-10-14T16:12Z
- **Completed:** 2026-10-14T16:15Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- `PAGE_SUMMARY` now reads "Play experimental Rust physics scenes in the browser. Every demo runs this repository's engine through WebAssembly." (D-09). `SiteFooter.tsx` is still the only importer in `src`.
- The `PlaygroundShell.tsx` comment now says "so the scene list preview counts hold (D-03)" and no longer mentions "25-preview".
- `scenes.test.ts`: the two test titles no longer contain a number. `toHaveLength`, `readyCount` and the `Array.from` length now use `SCENE_IDS.length`. The explicit `expectedIds`, title lists, `UI_SPEC_DESCRIPTIONS` and `UI_SPEC_HINTS` stay as they are because they are UI-SPEC contracts (D-08).
- `demo-media-model.test.ts`: the length check uses `SCENE_IDS.length`, and the last id is `SCENE_IDS.at(-1)` instead of `"tesla-valve"`.
- `shell.spec.ts`:
  - It imports `PAGE_SUMMARY`, `SCENE_IDS` and `SCENES`. The header and footer assertions use `PAGE_SUMMARY`.
  - At module level, `const FOCUSABLE_COUNT = SCENE_IDS.length + 2;` sits under a comment that says what the count covers: Dismiss, the Puzzles link, and one link per scene. Both focus-trap loops run `FOCUSABLE_COUNT + 1` times, and their comments no longer contain numbers.
  - `LAST_SCENE_TITLE` comes from `SCENES.at(-1)`, with an explicit `undefined` guard that throws. `lastLink` matches `new RegExp(`${escapeRegExp(LAST_SCENE_TITLE)}\\b`)`.
  - The sidebar preview test is now titled "shows one static preview per scene in the desktop sidebar", and both counts use `toHaveCount(SCENE_IDS.length)`.

## Task Commits

1. **Task 1: number-free copy and catalog-derived unit-test counts.** `a92898bbf` (refactor)
1. **Task 2: catalog-derived shell e2e counts and imported copy.** `63b4679eb` (test)

## Verification

- `bun scripts/bright-builds-check.ts all`: findings=0 (before each commit).
- `cd web && bun run typecheck`: clean for both the app and the e2e projects.
- `cd web && bun run test:unit`: 119 files and 1844 tests passed. That is the same file count as at HEAD (119).
- Shell e2e (`target/phase45/shell-e2e-04.log`): `just web-build && (cd web && CI=true bunx playwright test e2e/shell.spec.ts)` gave 14 passed and 0 failed.
- Task 1 grep: `twenty-five|toHaveLength(25)|toBe(25)|length: 25` has no hits in the three edited files, and `25-preview` has no hits in `PlaygroundShell.tsx`.
- Task 2 grep: `twenty-five|toHaveCount(25)|< 28|Tesla Valve` has no hits in `shell.spec.ts`. `toHaveCount(SCENE_IDS.length)` appears 2 times.

### D-10 grep (roots: crates/liquidfun-wasm/src, web/src, web/tests, web/e2e, web/scripts)

- Words (`twenty-five|twenty five`, case-insensitive): `target/phase45/d10-words-04.txt` is **empty**.
- Count patterns (`target/phase45/d10-grep-04.txt`) have one remaining line:

| Hit | Justification |
| --- | --- |
| `web/tests/animated-svg.test.ts:143: expect(percent).toBe(25);` | Known false positive. It checks a progress percent, not a scene count. |

Plan 03 has already run, so none of its Rust sites appear.

As an extra check beyond the plan regex, a word-bounded sweep for `25`, `26` and `27` over the same roots found only values that are not scene counts:

- the camera pan offset `-25` in `camera.test.ts`
- `15:27:03` timestamps in `build-info.test.ts`
- `T-41-25` and `T-41-26` threat ids in `puzzle-progress.spec.ts`
- SVG `cy="26"` in `previews.tsx`
- valve ticks and poses in the puzzle validate/tools tests
- the `GridRect` cell `-26` in `grand_finale.rs`
- the Dam Break particle grid `SMALL_PARTICLE_COLUMNS: u8 = 25` and `SMALL_PARTICLE_ROWS: u8 = 26` (a known false positive)

The "D-25" comments and decimal geometry such as `3.25` were filtered out as known false positives.

## Deviations from Plan

### Environment (not a code change)

**1. Shell e2e build command: `just web-build` instead of `just web-wasm && bun run build:app`**

- **Found during:** Task 2, first e2e run. That log is kept as `target/phase45/shell-e2e-04-attempt1-no-provenance.log`.
- **Issue:** With the plan's command, 13 tests passed and 1 failed. The failing test was `build timestamp › shows the build timestamp in local time with an approximate age`. The footer showed "Built at: Unavailable" because `bun run build:app` does not inject `VITE_BUILT_AT`. Only `scripts/web-build.ts` sets the provenance env (`VITE_APP_VERSION`, `VITE_GIT_SHA`, `VITE_BUILD_ID`, `VITE_BUILT_AT`). The failure has nothing to do with this plan's edits.
- **Fix:** Rebuilt with `just web-build`, which runs the wasm build and the app build with provenance, then reran the spec: 14 of 14 passed. No source change was needed.
- **Commit:** none (build environment only).

### Formatting

- No formatter is configured for `web/`. I wrapped the lines that went over the width by hand, in the existing prettier-like style: the `Array.from` assertion, the `lastLink` locator, the reverse-tab `for` header and the preview test signature.
- `FOCUSABLE_COUNT` has its explanation on the line above rather than as a trailing comment, to keep the line short. The acceptance grep `const FOCUSABLE_COUNT = SCENE_IDS.length + 2` still matches.

### Frontmatter provenance

- `generated_by` is `gsd-execute-phase`, as the orchestrator prompt asked. The plan's output section said `gsd-executor`. The lifecycle tooling only checks that the field is present.

## Requirement Status

PREP-02 is not ticked yet. This plan finishes the web half, and Plan 03 finished the Rust half. Plans 05 and 09 still list PREP-02, so the checkbox stays open until they are done.

## Known Stubs

None.

## Threat Flags

None. T-45-08 (stale public claim) is mitigated: the copy has no number, and the e2e imports the constant instead of re-typing it.

## Self-Check: PASSED

- FOUND: web/src/player/runtime.ts ("Every demo runs this repository's engine through WebAssembly." count 1)
- FOUND: web/e2e/shell.spec.ts (`import { PAGE_SUMMARY } from "../src/player/runtime";`, `SCENES.at(-1)` followed by the undefined guard)
- FOUND: target/phase45/shell-e2e-04.log, target/phase45/d10-words-04.txt (empty), target/phase45/d10-grep-04.txt
- FOUND: commit a92898bbf
- FOUND: commit 63b4679eb
