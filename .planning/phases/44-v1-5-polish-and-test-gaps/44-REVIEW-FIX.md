---
phase: 44-v1-5-polish-and-test-gaps
fixed_at: 2026-10-14T02:17:18Z
review_path: .planning/phases/44-v1-5-polish-and-test-gaps/44-REVIEW.md
iteration: 1
findings_in_scope: 8
fixed: 8
skipped: 0
status: all_fixed
---

# Phase 44: Code Review Fix Report

**Fixed at:** 2026-10-14T02:17:18Z
**Source review:** .planning/phases/44-v1-5-polish-and-test-gaps/44-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 8 (fix_scope: all)
- Fixed: 8
- Skipped: 0

No changes under `crates/`. The UI-SPEC layout (UI-D54..D56) is unchanged.

## Fixed Issues

### IN-01: Ellipsis assertion can never fail

**Files modified:** `web/e2e/puzzle-levels.spec.ts`
**Commit:** 3f895f666
**Applied fix:** Removed `nameText` from `TrayItemFit`, from the evaluate, and from the `not.toContain("…")` assertion. The `nameScroll <= nameClient` check is still the overflow guard. The grand-finale label oracle (`["Plank", "Valve"]`) was already present.

### IN-02: Missing `data-kind-count` silently skips the two-kind checks

**Files modified:** `web/e2e/puzzle-levels.spec.ts`
**Commit:** ccf6711f8
**Applied fix:** `expectTrayFits` now asserts `expect([1, 2]).toContain(fit.kindCount)` before it branches. The portrait loop counts two-kind levels and asserts that the count is greater than 0.

### IN-03: The "last level is grand-finale" assumption is only in a comment

**Files modified:** `web/e2e/puzzle-levels.spec.ts`
**Commit:** 1eba97c1e
**Applied fix:** Added `expect(PUZZLE_IDS.at(-1)).toBe("grand-finale")` before the final grand-finale tray assertions.

### IN-04: Tray border and padding are hardcoded

**Files modified:** `web/e2e/puzzle-levels.spec.ts`
**Commit:** c3e592a64
**Applied fix:** `innerRight` is now `itemRect.right - parseFloat(borderRightWidth) - parseFloat(paddingRight)`. Both values come from the item's computed style, read in the same evaluate.

### IN-05: The 2×/4× run test never checks the outcome is "won"

**Files modified:** `web/e2e/puzzle-valve.spec.ts`
**Commit:** 56bc5b703
**Applied fix:** Before each `runs.push` (1×, 2× and 4×), the test now asserts `data-puzzle-outcome` equals `maybeExpectation.outcome`, which is "won". I used the review's second option so the shared `RunFingerprint` stays unchanged for the four other specs that compare it.

### IN-06: Two divergent `Window.teslaBenchmark` declarations

**Files modified:** `web/benchmarks/tesla-valve/benchmark.ts`, `web/e2e/tesla-benchmark.d.ts`, `web/e2e/live-worker.spec.ts`, `web/scripts/bench-tesla/browser.ts`
**Commit:** bab49b59d
**Applied fix:**
- `benchmark.ts` exports `TeslaBenchmarkApi` (`runCase`, `compareWorkerDirect`).
- Both `Window` augmentations now declare the same optional `teslaBenchmark?: TeslaBenchmarkApi`, so they merge without TS2717.
- `live-worker.spec.ts` has a `compareWorkerDirectInPage` helper. It waits for the API, then calls it inside `page.evaluate` with a guard that throws if the API is missing. This replaced three copies of the same code.
- `bench-tesla/browser.ts` uses the same guard around `runCase`.

### IN-07: The "never ellipsize" comment only holds at the tested widths

**Files modified:** `web/src/styles/puzzle-editor.css`
**Commit:** d6ec84d5c
**Applied fix:** The comment now says names "fit in full at 360 px wide and up". Changed `display: inline-grid` to `display: grid`. The tray item is a flex item, so it was already blockified to grid and the computed layout does not change. The UI-D54 grid layout is unchanged.

### IN-08: `Viewport` type duplicated across e2e specs; boundary test misses the cap

**Files modified:** `web/e2e/puzzle-helpers.ts`, `web/e2e/puzzle-hint.spec.ts`, `web/e2e/puzzle-levels.spec.ts`, `web/e2e/water.spec.ts`, `web/e2e/puzzle-sound.spec.ts`, `web/e2e/puzzle-progress.spec.ts`, `web/e2e/puzzle.spec.ts`, `web/e2e/puzzle-editor.spec.ts`, `web/tests/puzzle-valve-format.test.ts`
**Commit:** fdff35469
**Applied fix:**
- `puzzle-helpers.ts` now exports a single `Viewport` type. The seven specs import it (one line each) and their local copies are deleted.
- Added `expect(maxOpenTick(7_680)).toBe(255)`. This is the first step limit whose floor (256) needs the `Math.min` cap.

## Verification

- `cd web && bun run typecheck` (`tsc --noEmit` and `tsc --noEmit -p tsconfig.e2e.json`): pass. Run after every fix and again at the end.
- `cd web && npx vitest run`: 119 files and 1844 tests passed.
- `bun scripts/bright-builds-check.ts all`: `SUMMARY all findings=0`. This covers file-lengths (1562 scanned, 0 findings) and lessons (0 findings).
- `cd web && bun run build:app`: built.
- `cd web && CI=true npx playwright test e2e/puzzle-levels.spec.ts e2e/puzzle-valve.spec.ts e2e/live-worker.spec.ts`: 32 passed (38.7s). live-worker also passed 8/8 by itself straight after the IN-06 change.
- The other six specs that IN-08 touched are type-only. `CI=true npx playwright test --list` on them loaded 92 tests in 6 files without errors. The strict e2e typecheck covers their types.
- `.planning/config.json` was never staged.

---

_Fixed: 2026-10-14T02:17:18Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
