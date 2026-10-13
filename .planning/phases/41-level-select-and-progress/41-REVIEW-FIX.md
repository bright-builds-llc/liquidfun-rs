---
phase: 41-level-select-and-progress
fixed_at: 2026-10-13T17:50:36Z
review_path: .planning/phases/41-level-select-and-progress/41-REVIEW.md
iteration: 1
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 41: Code Review Fix Report

**Fixed at:** 2026-10-13T17:50:36Z
**Source review:** .planning/phases/41-level-select-and-progress/41-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 3 (WR-01, plus IN-03 and IN-04 by orchestrator request)
- Fixed: 3
- Skipped: 0 in scope (IN-01, IN-02, IN-05, IN-06, IN-07 were out of scope and left unfixed on purpose, listed below)

## Fixed Issues

### WR-01: A level the player has solved can still show as locked and is not reachable from the grid

**Files modified:** `web/src/puzzle/progress-view.ts`, `web/tests/puzzle-progress-view.test.ts`
**Commit:** 762fdaa67
**Applied fix:** `unlockedLevelIds` now also unlocks a level that is itself solved: `previous === undefined || isSolved(id) || isSolved(previous)`. Unsolved levels still follow the D-10 predecessor rule, so first-run progression does not change. The doc comment explains the extra case. Tests:
- New test: with jelly-pad (level 3) solved and thick-syrup (level 2) unsolved, level 3 is unlocked and level 2 stays locked.
- New test: the predecessor card stays locked with "Solve First Pour to unlock".
- Updated the out-of-order `unlockedLevelIds` expectation to `["first-pour", "jelly-pad", "float-up"]`.
- The old test "keeps an out-of-order solved card locked" now checks that the card is unlocked, solved, not next up, keeps its stars and shows "Best fill 61%".

E2E: no assertion needed updating. The deep-link test (`puzzle-progress.spec.ts:314`) never wins timed-gate, so it correctly stays `data-locked="true"`. No spec seeds an out-of-order solve.
**Status:** fixed: requires human verification (this changes the D-10 unlock rule, so confirm the rule change is intended)

### IN-03: Leftover fields from the old disabled-Next design in `ResultActionsModel`

**Files modified:** `web/src/puzzle/format.ts`, `web/src/components/PuzzleResultCard.tsx`, `web/tests/puzzle-format.test.ts`
**Commit:** 93d816f37
**Applied fix:**
- `ResultActionsModel` is now `{ next: { label, ariaLabel, href: string } }`. `enabled`, `maybeTitle` and `primary` are removed.
- In the result card, the `Show when={maybeHref}` guard, `assignPrimary` and the Retry accent/primary wiring are removed. The Next/All levels link always carries `product-control--accent` and receives `primaryRef`. Retry is a plain button.
- Format tests updated to the new shape.

### IN-04: `LevelCardModel.maybeBestFillPct` is computed but never rendered

**Files modified:** `web/src/puzzle/progress-view.ts`, `web/tests/puzzle-progress-view.test.ts`
**Commit:** 349824eb3
**Applied fix:** Removed the field from the type and from `cardModel`, and removed its two test expectations. The footer ("Best fill N%") is still the only place best fill is rendered, and the tests already cover it.

## Skipped Issues

### IN-01: Restore, grid and star store do not pick up other tabs' changes until this tab commits

**File:** `web/src/player/puzzle-progress.ts:156-157`
**Reason:** Out of scope for this iteration (orchestrator asked to leave it unfixed). The review marks the fix as optional, and it would add a `storage` listener and re-read paths, which is new cross-tab behavior rather than a narrow fix.
**Original issue:** `maybeRestore`, the grid and the `StarStore` use this tab's snapshot, so tabs can briefly disagree. This never lowers stars or corrupts data.

### IN-02: Unknown fields inside known level records are dropped on write

**File:** `web/src/puzzle/progress.ts:92-116`
**Reason:** Out of scope for this iteration (orchestrator asked to leave it unfixed). The current behavior is a documented design choice, and changing it is a schema-policy decision.
**Original issue:** `parseLevel` keeps only known per-level fields, unlike the preserve-unknown policy used everywhere else.

### IN-05: The level-hash pattern is defined in three places

**File:** `web/src/physics/puzzle-session.ts:79`
**Reason:** Out of scope for this iteration (orchestrator asked to leave it unfixed). It is a duplication cleanup with no behavior impact.
**Original issue:** `puzzle-session.ts` and the replay script redeclare the `LEVEL_HASH_PATTERN` regex.

### IN-06: The level hash also changes when cosmetic fields or upstream `Debug` formats change

**File:** `crates/liquidfun-wasm/src/puzzle/level_hash.rs:29-31`
**Reason:** Out of scope for this iteration (orchestrator asked to leave it unfixed). It needs documentation only, and the behavior is intended (D-17) and pinned by tests.
**Original issue:** The module doc does not mention that dependency `Debug` formats and cosmetic fields affect the hash.

### IN-07: An unlocked card's link name is the whole card text

**File:** `web/src/components/PuzzleSelectPanel.tsx:93-102`
**Reason:** Out of scope for this iteration (orchestrator asked to leave it unfixed). It is accessibility polish. The current name is correct, just verbose.
**Original issue:** The card `<a>` takes its accessible name from all of its content, not from `aria-labelledby`/`aria-describedby`.

## Verification

- Before each commit: `cd web && bun run typecheck` passed, `bun run test:unit` passed (1543 tests), and `bun scripts/bright-builds-check.ts all` reported 0 findings.
- After all fixes: `bun run build:app` succeeded. `CI=true bunx playwright test e2e/puzzle-progress.spec.ts e2e/puzzle-levels.spec.ts` gave 42 passed. `e2e/puzzle.spec.ts` (result card coverage) gave 16 passed.

---

_Fixed: 2026-10-13T17:50:36Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
