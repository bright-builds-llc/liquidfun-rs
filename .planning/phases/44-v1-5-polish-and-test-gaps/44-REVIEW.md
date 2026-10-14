---
phase: 44-v1-5-polish-and-test-gaps
reviewed: 2026-10-14T02:12:19Z
depth: standard
files_reviewed: 18
files_reviewed_list:
  - web/e2e/canvas-stage.spec.ts
  - web/e2e/player-helpers.ts
  - web/e2e/puzzle-editor-helpers.ts
  - web/e2e/puzzle-levels.spec.ts
  - web/e2e/puzzle-valve.spec.ts
  - web/e2e/tesla-benchmark.d.ts
  - web/e2e/water-helpers.ts
  - web/e2e/water.spec.ts
  - web/e2e/worker-quality-fixture.ts
  - web/package.json
  - web/playwright.config.ts
  - web/src/components/PuzzleSelectionBar.tsx
  - web/src/puzzle/placements.ts
  - web/src/puzzle/valve-format.ts
  - web/src/styles/puzzle-editor.css
  - web/tests/puzzle-placements.test.ts
  - web/tests/puzzle-valve-format.test.ts
  - web/tsconfig.e2e.json
findings:
  critical: 0
  warning: 0
  info: 8
  total: 8
status: issues_found
---

# Phase 44: Code Review Report

**Reviewed:** 2026-10-14T02:12:19Z
**Depth:** standard
**Files Reviewed:** 18
**Status:** issues_found

## Summary

Reviewed the Phase 44 diff (`9e6efcc23..HEAD`) for the 18 listed files. The production changes are small and correct:

- The seam's `openTick` now defaults by kind. Previously, a valve without `openTick` got 0, which Rust's `checked_open_tick` rejects (`1..=max_open_tick`).
- `maxOpenTick` now saturates at 255, matching `LevelSpec::max_open_tick` in `crates/liquidfun-wasm/src/puzzle/spec.rs:148`.
- The valve readout is now exposed and linked to the stepper group through `aria-describedby`. Step changes are still announced through `#puzzle-editor-status` (`editor-announce.ts:125`).
- The portrait two-kind tray grid is scoped tightly enough that it doesn't affect landscape or the form-factor layout.

The e2e typecheck work removes the escape hatches cleanly. `tsc --noEmit -p tsconfig.e2e.json` passes. `vitest run tests/puzzle-placements.test.ts tests/puzzle-valve-format.test.ts` passes (60 tests).

I checked for a stale-state race in the hash-only `goto` loop of the portrait tray test and found none. `handleHashChange` calls `setRoute` and the synchronous prefix of `beginPuzzleRoute` (which dispatches `reset` and sets the view to loading) in the same task. So `data-puzzle` never shows the new id while the previous level's `building` phase is still visible.

There are no critical or warning findings. The eight info items are mostly e2e assertions that are weaker or more brittle than they look, plus some small type-duplication and documentation points.

## Info

### IN-01: Ellipsis assertion can never fail

**File:** `web/e2e/puzzle-levels.spec.ts:99,122`
**Issue:** `nameText` is read from `textContent`, and the check is `not.toContain("…")`. CSS `text-overflow: ellipsis` (from `.puzzle-tray-name` in `puzzle-editor.css`) only changes rendering, not `textContent`, and no component code adds an ellipsis character to tray labels. So this assertion always passes. The `nameScroll <= nameClient` check on the line above is the real overflow guard.
**Fix:** Remove `nameText` and its assertion. Alternatively, replace it with a check that has a real oracle, such as comparing `nameText` to the expected label (`"Plank"` / `"Valve"`).

### IN-02: Missing `data-kind-count` silently skips the two-kind checks

**File:** `web/e2e/puzzle-levels.spec.ts:108,129-131`
**Issue:** `Number(tray.getAttribute("data-kind-count"))` returns `0` when the attribute is missing. `expectTrayFits` then returns early and skips the D-01 checks (visible name, 14 px name, 12 px count). If the attribute were renamed, the grand-finale checks at the end would still pass: `toHaveText` reads text content, so it also matches names that are `display: none`.
**Fix:** Assert the attribute is valid before branching, and require at least one two-kind level in the portrait loop:

```ts
expect([1, 2]).toContain(fit.kindCount);
```

### IN-03: The "last level is grand-finale" assumption is only in a comment

**File:** `web/e2e/puzzle-levels.spec.ts:284`
**Issue:** The final assertions depend on `PUZZLE_IDS` ending with `grand-finale`. That is true today (`src/catalog/puzzles.ts:5-12`). If a level is appended, the failure will be a confusing tray-name mismatch.
**Fix:** Add `expect(PUZZLE_IDS.at(-1)).toBe("grand-finale");` before the final assertions, or navigate to `grand-finale` explicitly.

### IN-04: Tray border and padding are hardcoded

**File:** `web/e2e/puzzle-levels.spec.ts:104`
**Issue:** `itemRect.right - 1 - 8` hardcodes the item border and `padding: 0 8px` from `.puzzle-tray .puzzle-tray-item`. If the CSS padding changes, the count-inside-item check drifts without any signal.
**Fix:** Read the values in the same evaluate: `parseFloat(getComputedStyle(item).paddingRight)` and `borderRightWidth`.

### IN-05: The 2×/4× run test never checks the outcome is "won"

**File:** `web/e2e/puzzle-valve.spec.ts:253-277`
**Issue:** `RunFingerprint` has `outcomeStep`, `decidedStep`, `peakHeld` and `heldHash`, but not `data-puzzle-outcome`. This test checks `maybeExpectation.outcome === "won"` but never checks the outcome attribute in any of the three runs. In practice, matching `outcomeStep`, `peakHeld` and `heldHash` makes a lost run very unlikely to pass, but the test name says each run has "the 1× outcome".
**Fix:** Add `outcome: await read("data-puzzle-outcome")` to `readRunFingerprint`, or assert `await expect(main(page)).toHaveAttribute("data-puzzle-outcome", "won")` before each `runs.push`.

### IN-06: Two divergent `Window.teslaBenchmark` declarations

**File:** `web/e2e/tesla-benchmark.d.ts:4-7` (and `web/benchmarks/tesla-valve/benchmark.ts:242-249`)
**Issue:** The e2e declaration is `{ readonly compareWorkerDirect }`, while the benchmark declares `{ runCase; compareWorkerDirect }`. Each tsconfig program sees only one of them today, so typecheck passes. If an e2e file ever imports `benchmark.ts`, TS2717 ("Subsequent property declarations must have the same type") will break the e2e typecheck. Also, the property is non-optional, but `live-worker.spec.ts` checks `window.teslaBenchmark !== undefined` before it is installed.
**Fix:** Export a `TeslaBenchmarkApi` type from `benchmark.ts`, reference it with `import type` in the e2e declaration, and make the property optional (`teslaBenchmark?: TeslaBenchmarkApi`). Spec bodies that run after the `waitForFunction` would then use a non-null guard.

### IN-07: The "never ellipsize" comment only holds at the tested widths

**File:** `web/src/styles/puzzle-editor.css:340-352`
**Issue:** The comment says names "never ellipsize". But the name column is `minmax(0, max-content)`, and `.puzzle-tray-name` still has `overflow: hidden; text-overflow: ellipsis`. At a 320 px wide phone, each item's content box is roughly 50 px. After the 22 px icon and 4 px gap, "Plank" at 14 px won't fit, so it will ellipsize. Also, `display: inline-grid` on a flex item is blockified to `grid` anyway, so `inline-` has no effect.
**Fix:** Change the comment to name the guaranteed widths ("at 360 px and wider"), or add a narrower e2e case if 320 px matters. Use `display: grid` for clarity.

### IN-08: `Viewport` type duplicated across e2e specs; boundary test misses the cap

**File:** `web/e2e/water.spec.ts:75`, `web/tests/puzzle-valve-format.test.ts:66-68`
**Issue:**
- This phase adds a seventh local copy of `type Viewport = { readonly width: number; readonly height: number }` (also in `puzzle-hint`, `puzzle-levels`, `puzzle`, `puzzle-sound`, `puzzle-progress` and `puzzle-editor` specs).
- In `puzzle-valve-format.test.ts`, the test titled "keeps 255 at exactly 7,650 steps" passes with or without `Math.min`, because `floor(7650 / 30)` is already 255. 7,680 steps is the first value that actually needs the cap.

**Fix:**
- Export one `Viewport` type from `e2e/puzzle-helpers.ts`, or use Playwright's `ViewportSize`.
- Add `expect(maxOpenTick(7_680)).toBe(255)` (floor gives 256) as the boundary case.

---

_Reviewed: 2026-10-14T02:12:19Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
