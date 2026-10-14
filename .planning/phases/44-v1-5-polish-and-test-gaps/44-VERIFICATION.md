---
phase: 44-v1-5-polish-and-test-gaps
verified: 2026-10-14T02:20:46Z
status: passed
score: 4/4 roadmap success criteria verified (13/13 merged must-haves)
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 44-2026-10-14T01-34-05
generated_at: 2026-10-14T02:20:46Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 44: v1.5 Polish and Test Gaps Verification Report

**Phase Goal:** Close the owner-selected tech debt from the v1.5 milestone audit so the puzzle game reads well on phones, is accessible, and its e2e code and remaining flows are covered by checks.
**Verified:** 2026-10-14T02:20:46Z on HEAD `de4253283`, after the code-review fixes
**Status:** passed
**Re-verification:** No. This is the initial verification.

The 44-03 gate run (`target/phase44/checks-03.results`) ran against `813516547`, before the 8 review-fix commits. I re-ran the same 10 gates on the final tree (`de4253283`) with a copy of `target/phase44/checks-03.sh` that writes to the session scratchpad, so the executor's evidence files were not overwritten. I also re-ran the two specs that the review fixes touched but no gate covers (`live-worker`, `canvas-stage`).

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| SC1 | On a 390×844 phone viewport, every tool name in every level's tray (including grand-finale's planks and valve) is shown in full, and the tray keeps its fixed HUD obstacle box and 44 px touch targets. | ✓ VERIFIED | The CSS diff in `web/src/styles/puzzle-editor.css` is additions only: 4 rules scoped to `[data-puzzle-layout="portrait"] .puzzle-tray[data-kind-count="2"]`, with the tray box block unchanged. `PuzzleToolTray.tsx` is unchanged since `9e6efcc23`. The e2e tests `shows every tray name in full at 390x844` and `…360x780` loop over every `PUZZLE_IDS` level. They assert `scrollWidth <= clientWidth` for each visible name, 44 px item height, equal item widths, a tray box of 218/188 × 44, and 14 px name / 12 px count on the two-kind level. They also assert the accessible names "Plank, 2 left" / "Valve, 1 left" and the texts `["Plank","Valve"]` / `["×2","×1"]`. Both pass in the final-tree puzzle-e2e gate (104/104), and hud-safe-frame passes 64/64. The screenshot regenerated on the final tree, `target/phase44/screens/level-grand-finale-390x844.png`, shows "Plank" over "×2" and "Valve" over "×1" with no ellipsis. |
| SC2 | A selected valve's opening time is announced to assistive technology (not `aria-hidden`), and the TypeScript valve tick is capped at 255 to match the Rust lane. | ✓ VERIFIED | In `PuzzleSelectionBar.tsx`, the readout is `<span id={VALVE_TIME_ID} …>` with no `aria-hidden`. The group has `aria-describedby={valve().enabled ? VALVE_TIME_ID : undefined}`, where `enabled` holds only while building with a valve selected (`hud-format.ts` `valveStepperModel`). There is no `aria-live`. The e2e test `plays the timed-gate reference at 2× and 4× with the 1× outcome` asserts `toHaveAccessibleDescription(valveSecondsText(9))` while a valve is selected, and no `aria-describedby` after Escape. In `valve-format.ts`, `maxOpenTick` returns `Math.min(Math.floor(stepLimit / 30), MAX_OPEN_TICK=255)`, which mirrors `spec.rs:148` `u8::try_from(…).unwrap_or(u8::MAX)`. Both consumers (`hud-format.ts:128`, `reduce-puzzle.ts:254`) go through it. Unit tests at 7,650, 7,680 and 9,000 steps pass in vitest (1844/1844). |
| SC3 | An e2e test wins a level, clicks Next and lands on the next level ready to build. Another runs the timed-gate reference at 2× and 4× and reaches the same outcome step as 1×. | ✓ VERIFIED | `puzzle-levels.spec.ts:231` wins first-pour (it asserts `data-puzzle-outcome="won"`), deletes the seam global, and clicks the "Next level" link. It then asserts the URL `#/puzzle/thick-syrup`, `data-puzzle="thick-syrup"`, the building phase, empty placements, the title, no failure, and no canvas fallback. `puzzle-valve.spec.ts:225` shares one build, then plays at 1× and retries at 2× and 4×. It asserts a won outcome on each run, `outcomeStep` 429, `peakHeld` 582, equal fingerprints, and final speed 4. Both tests ran and passed in the final-tree puzzle-e2e gate (log lines 126 and 194). |
| SC4 | `web/playwright.config.ts` and the `web/e2e` helpers are inside the TypeScript typecheck, and the typecheck passes. Puzzle outcomes and the 25 scene fingerprints are unchanged. | ✓ VERIFIED | `web/tsconfig.e2e.json` extends the strict base and overrides only `types` and `include`. `--showConfig` confirms `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes` and `noUnused*`. `--listFilesOnly` lists 26 files: all 25 `e2e/*.ts` plus `playwright.config.ts`. `package.json` `typecheck` = `tsc --noEmit && tsc --noEmit -p tsconfig.e2e.json`, and `scripts/web-build.ts:292` runs `bun run typecheck`. The typecheck gate exits 0. The escape-hatch grep (`@ts-ignore`, `@ts-expect-error`, `@ts-nocheck`, `any`) over `e2e/` and the config finds nothing. Puzzle outcomes: test:puzzle 104/104, including the replay specs. `git diff --quiet 9e6efcc23 -- crates/` exits 0 and there are no untracked files under `crates/`, so the 25 scene fingerprints cannot have changed. |

**Score:** 4/4 roadmap success criteria verified.

Plan must-haves merged into the criteria above all hold:

- **44-01:** 5 truths. The tray fit holds at 390 and 360 wide. The tray box stays 218/188 × 44. The ARIA id and conditional describedby are in place. The 255 cap holds. A seam valve without `openTick` reads 4 and a plank reads 0 (`placements.ts` `readSeamOpenTick(entry, kind)`, with the kind checked first). The unit test passes.
- **44-02:** 3 truths. The two-project typecheck passes. All 16 errors are fixed at the root with no escape hatches. The touched specs pass: canvas-stage plus live-worker 17/17 and water 13/13 on the final tree.
- **44-03:** 4 truths. The Next e2e, the 1×/2×/4× e2e and the ARIA description e2e pass, and all 10 gates pass on the final tree.

Note on SC4 wording: ROADMAP.md says "`npx tsc --noEmit` passes". Under locked decision D-09, the gate command is `cd web && bun run typecheck`, which runs `tsc --noEmit` (app project, which passes) and then the e2e project. Plain `npx tsc --noEmit` alone checks only the app project. The intent, that e2e code is inside the gate typecheck and passes, is met.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `web/src/puzzle/valve-format.ts` | `MAX_OPEN_TICK` cap | ✓ VERIFIED | Has `MAX_OPEN_TICK = 255` and `Math.min(…)`. Used by `hud-format.ts` and `reduce-puzzle.ts`. |
| `web/src/puzzle/placements.ts` | `defaultOpenTick(kind)` seam default | ✓ VERIFIED | `readSeamOpenTick(entry, kind)` runs after `isToolKind`. |
| `web/src/styles/puzzle-editor.css` | Stacked portrait two-kind chip | ✓ VERIFIED | 4 added rules (grid `22px minmax(0, max-content)`, rows `20px 16px`). No removed lines. |
| `web/src/components/PuzzleSelectionBar.tsx` | Readout as the group description | ✓ VERIFIED | `VALVE_TIME_ID`, conditional `aria-describedby`, no `aria-hidden`, no `aria-live`. |
| `web/e2e/puzzle-levels.spec.ts` | Tray-fit, landscape and Next e2e | ✓ VERIFIED | Tests at :231, :267 (×2) and :304 ran and passed. |
| `web/e2e/puzzle-valve.spec.ts` | 1×/2×/4× and ARIA e2e | ✓ VERIFIED | Has `buildReference` (shared), `toHaveAccessibleDescription` and `readRunFingerprint`. |
| `web/tsconfig.e2e.json` | Strict e2e project | ✓ VERIFIED | Has `"extends": "./tsconfig.json"`. Covers 26 files. |
| `web/e2e/tesla-benchmark.d.ts` | Typed `window.teslaBenchmark` | ✓ VERIFIED | After IN-06 it is `teslaBenchmark?: TeslaBenchmarkApi`, identical to the declaration in `benchmark.ts`. The plan's `typeof compareWorkerDirect` pattern was replaced by the shared exported type, which is an intentional review fix. live-worker passes 8/8. |
| `web/package.json` | Two-project typecheck script | ✓ VERIFIED | Only the `typecheck` line differs from `9e6efcc23`. |
| `target/phase44/screens/level-grand-finale-390x844.png` | Evidence screenshot | ✓ VERIFIED | Regenerated at the final-tree gate run (21:18 local). Viewed. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `div.puzzle-valve-stepper` | `span#puzzle-valve-time` | `aria-describedby` while `valve().enabled` | ✓ WIRED | Proven in the browser by `toHaveAccessibleDescription` and its absence after Escape. |
| `valve-format.ts maxOpenTick` | `spec.rs max_open_tick` (read-only) | 255 saturation | ✓ WIRED | Same semantics. `crates/` is unchanged. |
| `scripts/web-build.ts:292` | `web/tsconfig.e2e.json` | `bun run typecheck` | ✓ WIRED | `package.json` script. |
| `e2e/tesla-benchmark.d.ts` | `benchmarks/tesla-valve/benchmark.ts` | `import type { TeslaBenchmarkApi }` | ✓ WIRED | After IN-06. The augmentations merge without TS2717. |
| Next test | `PUZZLE_TEST_PLACEMENTS_GLOBAL` | `Reflect.deleteProperty(window, …)` | ✓ WIRED | The deletion is asserted (`name in window` is false). |
| Fast-forward test | `timed-gate-solution.txt` | `maybeExpectation.outcomeStep` / `peakHeld` | ✓ WIRED | 429/582 from the fixture. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `PuzzleSelectionBar` readout | `valve().text` | `valveStepperModel(props.state)` → selected valve's `openTick` | Yes. The browser description equals `valveSecondsText(9)` for the placed reference valve. | ✓ FLOWING |
| Tray names | `trayItems` (unchanged DOM) | Level tool inventory | Yes. Grand-finale shows "Plank ×2" and "Valve ×1". | ✓ FLOWING |

### Behavioral Spot-Checks (final tree `de4253283`)

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Two-project typecheck | `cd web && bun run typecheck` | exit 0 | ✓ PASS |
| Unit tests | `cd web && npx vitest run` | 119 files, 1844 tests passed | ✓ PASS |
| Repo checks | `bun scripts/bright-builds-check.ts all` | `all findings=0` | ✓ PASS |
| Rust untouched | `git diff --quiet 9e6efcc23 -- crates/` plus porcelain check | exit 0 | ✓ PASS |
| Pinned files and dependencies | pinned-files gate (protected tests, catalog, `bun.lock`, `tsconfig.json`, `PuzzleToolTray.tsx`, `package.json` limited to `typecheck`) | exit 0 | ✓ PASS |
| No non-planning Markdown changed | markdown-untouched gate | exit 0 | ✓ PASS |
| App build | `just web-wasm && bun run build:app` | exit 0 | ✓ PASS |
| Puzzle e2e | `CI=true bun run test:puzzle` | 104 passed | ✓ PASS |
| Water e2e | `CI=true bun run test:water` | 13 passed | ✓ PASS |
| HUD safe frame | `CI=true bunx playwright test e2e/hud-safe-frame.spec.ts` | 64 passed | ✓ PASS |
| Review-fix-touched specs | `CI=true bunx playwright test e2e/canvas-stage.spec.ts e2e/live-worker.spec.ts` | 17 passed | ✓ PASS |
| e2e coverage of the typecheck | `npx tsc -p tsconfig.e2e.json --listFilesOnly` | 26 files (25 e2e plus the config) | ✓ PASS |

Results and log from the final-tree run are in the session scratchpad (`verify.results`, `verify.log`). The executor's `target/phase44/checks-03.*` files are preserved unchanged as the pre-review run.

### Requirements Coverage

Phase 44 declares no requirement IDs. All three plans have `requirements: []`, and ROADMAP.md says "None new (tech-debt cleanup; touches TOOL-01, TOOL-05, TOOL-07, PUZ-01, PROG-02 behavior without changing them)". REQUIREMENTS.md maps no ID to Phase 44, so there are no orphaned requirements. The behavior of the touched requirements is protected by the unchanged test:puzzle suite (104/104).

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| (none) | — | Tech-debt phase | N/A | No IDs declared or mapped |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `web/e2e/puzzle-levels.spec.ts` | `expectTrayFits` | D-01's "no `…`" text check was removed by review fix IN-01, because a CSS ellipsis never appears in `textContent` | ℹ️ Info | `scrollWidth <= clientWidth` is the real overflow guard. The pre-plan CSS failed it (42 > 37), and the screenshot confirms there is no ellipsis. |
| `web/src/styles/puzzle-editor.css` | 340 | The comment limits the fit guarantee to "360 px wide and up" | ℹ️ Info | Matches the tested widths. Narrower phones are untested but outside SC1. |

There are no TODOs, stubs, escape hatches or empty handlers in the phase files.

### Human Verification Required

None. Under D-12 every criterion is objectively checkable. The visual check of the screenshot was done by the verifier.

### Gaps Summary

There are no gaps. All four roadmap success criteria hold on the final tree after the code-review fixes. The 10 phase gates and the two extra specs pass, `crates/` is untouched since `9e6efcc23`, and the lifecycle provenance (`yolo`, `44-2026-10-14T01-34-05`) matches across CONTEXT, all three PLANs and all three SUMMARYs.

---

_Verified: 2026-10-14T02:20:46Z_
_Verifier: Claude (gsd-verifier)_
