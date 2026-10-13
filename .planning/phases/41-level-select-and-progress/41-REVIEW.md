---
phase: 41-level-select-and-progress
reviewed: 2026-10-13T17:46:59Z
depth: standard
files_reviewed: 60
files_reviewed_list:
  - crates/liquidfun-wasm/src/puzzle.rs
  - crates/liquidfun-wasm/src/puzzle/info.rs
  - crates/liquidfun-wasm/src/puzzle/level_hash.rs
  - crates/liquidfun-wasm/src/puzzle/level_hash/tests.rs
  - crates/liquidfun-wasm/src/puzzle/level-hashes.txt
  - web/scripts/puzzle-replay.ts
  - web/scripts/puzzle-replay/level-hashes.ts
  - web/src/App.tsx
  - web/src/app.css
  - web/src/catalog/puzzle-levels.ts
  - web/src/components/PlaygroundShell.tsx
  - web/src/components/PlaygroundStage.tsx
  - web/src/components/PuzzleFallbackPanel.tsx
  - web/src/components/PuzzleIcons.tsx
  - web/src/components/PuzzleLevelLinks.tsx
  - web/src/components/PuzzleOptionsDrawer.tsx
  - web/src/components/PuzzlePanel.tsx
  - web/src/components/PuzzleResultCard.tsx
  - web/src/components/PuzzleSelectPanel.tsx
  - web/src/physics/puzzle-session.ts
  - web/src/player/playable-target.ts
  - web/src/player/playground-session.ts
  - web/src/player/puzzle-editing.ts
  - web/src/player/puzzle-lifecycle.ts
  - web/src/player/puzzle-progress.ts
  - web/src/player/puzzle-run.ts
  - web/src/player/puzzle-runtime.ts
  - web/src/player/puzzle-saved-placements.ts
  - web/src/puzzle/controller.ts
  - web/src/puzzle/format.ts
  - web/src/puzzle/placements.ts
  - web/src/puzzle/progress-view.ts
  - web/src/puzzle/progress.ts
  - web/src/puzzle/stars.ts
  - web/src/routing/playable-route.ts
  - web/src/styles/puzzle-select.css
  - web/src/styles/puzzle.css
  - web/e2e/puzzle-progress.spec.ts
  - web/e2e/puzzle-levels.spec.ts
  - web/e2e/puzzle.spec.ts
  - web/e2e/shell.spec.ts
  - web/package.json
  - web/tests/progress-storage-fixture.ts
  - web/tests/puzzle-progress.test.ts
  - web/tests/puzzle-progress-shell.test.ts
  - web/tests/puzzle-progress-stars.test.ts
  - web/tests/puzzle-progress-view.test.ts
  - web/tests/puzzle-restore.test.ts
  - web/tests/puzzle-save-placements.test.ts
  - web/tests/playable-route.test.ts
  - web/tests/playable-target.test.ts
  - web/tests/puzzle-format.test.ts
  - web/tests/puzzle-chrome.test.ts
  - web/tests/puzzle-level-hashes.test.ts
  - web/tests/puzzle-level-info.test.ts
  - web/tests/puzzle-placements.test.ts
  - web/tests/puzzle-host-fixture.ts
  - web/tests/puzzle-session.test.ts
  - web/tests/puzzle-stars.test.ts
  - web/src/puzzle/star-store.ts
findings:
  critical: 0
  warning: 1
  info: 7
  total: 8
status: issues_found
---

# Phase 41: Code Review Report

**Reviewed:** 2026-10-13T17:46:59Z
**Depth:** standard
**Files Reviewed:** 60
**Status:** issues_found

## Summary

Reviewed the Phase 41 diff (`f277ecad9..HEAD`): the progress core and shell, restore and save wiring, the route and target changes, the grid and level-link components, the result-card and fallback changes, the Rust level hash and its pins, and the new and updated tests. `web/src/puzzle/star-store.ts` was also read because the new code depends on its merge behavior.

Overall the work is careful and well tested. The focus areas mostly check out:

- **Fail-closed storage and no writes in degraded mode:** these hold. `openStorage` degrades on any of these: a throwing or null provider, a failed probe `setItem` or `removeItem`, a failed `getItem`, corrupt JSON, a wrong shape, or a future version. `commit` re-reads before every write and degrades without writing when the re-read is unusable or `setItem` throws. After that, `maybeStorage` stays `undefined`, so later commits update memory only. The only other write is the probe, which uses its own key. Unit tests check that unusable seeds stay byte-for-byte unchanged and that the progress key is never written.
- **Star merges never lower:** this holds, across tabs too. Every combine goes through `mergeLevelStars`. A loss never reaches `withStars`. The write path re-reads, merges, then re-applies the change. `puzzle-run.ts` feeds the merged result back into the page `StarStore`.
- **Restore and save timing:** the one-shot `maybePendingRestoreId` is checked against the level id and requires a non-run `loading` state. `leavePuzzle` and `beginPuzzleRoute` clear it, so Retry, Edit, Stop and Try again never restore, and a stale build cannot restore into another level. Saves need `maybeLevelInfo()`, which `clearLevelSignals` clears on entry. So a placement is never saved under the previous level's hash.
- **Prototype pollution and text-only rendering:** handled well. Parsed records are null-prototype, keys are read through `Object.entries`, a `__proto__` key stays plain data, and placements are rebuilt field by field by `maybeParsePlacement`. The grid renders only TS catalog strings, booleans, and `Math.floor` of a range-checked number. There is no `innerHTML`.
- **Route parsing:** `#/puzzles` and `#/puzzles/` give the grid, and `#/puzzles/<x>` falls through to the scene parser as unknown. Both are tested, and `normalizePlayableRoute` never rewrites them.
- **Rust:** no `unwrap` in production code. The hash depends only on `Debug` text, which is platform-independent, and the wasm replay checks it against the same pin file.

Verification run during review:

- `npx vitest run` on the 7 progress, restore, save and route test files: 137 passed.
- `cargo test -p liquidfun-wasm --lib level_hash`: 9 passed, 1 ignored (the print helper).

The one Warning is a grid edge case. A level the player has already solved, through a deep link, can still show as Locked, with the "Solve X to unlock" footer and no link. The Info items cover cross-tab staleness, a gap in forward-compatible data preservation, dead fields left over from the old disabled-Next design, and small duplication or accessibility polish.

## Warnings

### WR-01: A level the player has solved can still show as locked and is not reachable from the grid

**File:** `web/src/puzzle/progress-view.ts:16-27`, `web/src/puzzle/progress-view.ts:52-67`, `web/src/puzzle/progress-view.ts:78`

**Issue:** `unlockedLevelIds` looks only at whether the previous level is solved. Deep links always open any level (D-10, and the e2e spec checks a deep link into a locked level). So a player can deep-link to level 3, win it, and come back to the grid while level 2 is still unsolved. Level 3's card then:

- shows the Locked tag and the dashed locked styling
- shows its earned Win, Brim and Thrifty stars
- keeps the footer "Solve Thick Syrup to unlock"
- is a non-interactive `role="group"`

Two problems follow. The player cannot reopen their own solved level from the grid. And the same card says both "Locked" and "Win: earned". Level 4 also shows as unlocked while level 3, which unlocked it, shows as locked. The literal D-10 rule allows this, but it is an unhandled edge case that contradicts itself on screen.

**Fix:** Treat a solved level as unlocked. The rule stays the same for unsolved levels, so first-run progression does not change.

```ts
export function unlockedLevelIds(progress: ProgressDocument, degraded: boolean): ReadonlySet<PuzzleId> {
  if (degraded) {
    return new Set(PUZZLE_IDS);
  }

  return new Set(
    PUZZLE_IDS.filter((id, index) => {
      const maybePreviousId = PUZZLE_IDS[index - 1];
      return maybePreviousId === undefined || isSolved(progress, id) || isSolved(progress, maybePreviousId);
    }),
  );
}
```

Add a unit test in `web/tests/puzzle-progress-view.test.ts` where level 3 is solved and level 2 is not, and assert that level 3 is unlocked and level 2 is still locked.

## Info

### IN-01: Restore, grid and star store do not pick up other tabs' changes until this tab commits

**File:** `web/src/player/puzzle-progress.ts:156-157`, `web/src/player/puzzle-runtime.ts:53-62`, `web/src/player/puzzle-saved-placements.ts:43`

**Issue:** The star invariant holds across tabs, but three read paths still use this tab's snapshot:

- `maybeRestore` decides from `current()`, not a fresh read. Entering a level restores this tab's older placements, even when another tab has saved newer ones for that level.
- There is no `storage` event listener. The grid's lock state and stars only catch up after this tab's next `commit`.
- The `StarStore` is hydrated only at startup. A commit can pull another tab's stars into `current()`, and the grid then shows them, but the result card's "best" stays behind until this tab records a win. Similarly, the `maybeLastSavedKey` dedupe can skip a save at Play when another tab overwrote the record after this tab's last save.

None of this lowers stars or corrupts data, but tabs can briefly disagree.

**Fix:** Optional. Have `maybeRestore` re-read storage when it is not degraded, and use the merged document. Add a `storage` listener for `PROGRESS_STORAGE_KEY` that re-parses the document, sets `current`, and re-records any won levels into the `StarStore`. Ignore unusable values there, or degrade on them.

### IN-02: Unknown fields inside known level records are dropped on write

**File:** `web/src/puzzle/progress.ts:92-116`, `web/src/puzzle/progress.ts:196-204`

**Issue:** The document keeps unknown level ids and unknown top-level keys verbatim. But `parseLevel` keeps only the four star fields and `placements`, so any other per-level field is lost on the next write. Suppose a later build adds a per-level field without bumping `version`. An older tab left open would then erase it. The comment on line 92 documents this choice, but it differs from the preserve-unknown policy used everywhere else in the document.

**Fix:** Either keep a raw `extra` record per level and spread it back in `serializeLevel`, or note next to `PROGRESS_VERSION` that adding any per-level field requires a version bump.

### IN-03: Leftover fields from the old disabled-Next design in `ResultActionsModel`

**File:** `web/src/puzzle/format.ts:231-271`, `web/src/components/PuzzleResultCard.tsx:62-66`, `web/src/components/PuzzleResultCard.tsx:123-126`

**Issue:** `resultActionsModel` now always returns `enabled: true`, `maybeTitle: undefined`, a defined `maybeHref`, and `primary: "next"`. No component reads `enabled` or `maybeTitle` any more. The `"retry"` member of `primary` can no longer happen, so `assignPrimary("retry")` and the Retry accent class are dead code.

**Fix:** Remove `enabled` and `maybeTitle`, make `maybeHref` a plain `href: string`, and either drop `primary` or narrow it to `"next"`. Then remove the `Show when={actions().next.maybeHref}` guard and the Retry primary wiring. Update `web/tests/puzzle-format.test.ts` to match.

### IN-04: `LevelCardModel.maybeBestFillPct` is computed but never rendered

**File:** `web/src/puzzle/progress-view.ts:41`, `web/src/puzzle/progress-view.ts:87`

**Issue:** The grid shows "Best fill N%" only through `footer`. `maybeBestFillPct` is used only by unit tests.

**Fix:** Remove the field, or render it, for example as a `data-best-fill` attribute that the tests can assert.

### IN-05: The level-hash pattern is defined in three places

**File:** `web/src/physics/puzzle-session.ts:79`, `web/src/puzzle/progress.ts:11`, `web/scripts/puzzle-replay/level-hashes.ts:10`

**Issue:** `progress.ts` already exports `LEVEL_HASH_PATTERN`, but `puzzle-session.ts` and the replay script each declare their own copy of the same regex. If the hash width ever changes, one copy could be missed.

**Fix:** Import `LEVEL_HASH_PATTERN` from `../puzzle/progress` in `puzzle-session.ts`, or move it into a shared module that both import. The replay script may keep its own copy if it should stay independent of the app.

### IN-06: The level hash also changes when cosmetic fields or upstream `Debug` formats change

**File:** `crates/liquidfun-wasm/src/puzzle/level_hash.rs:29-31`

**Issue:** Hashing the full `LevelSpec` `Debug` text means three kinds of change discard players' saved layouts even though the level plays the same:

- a title edit
- a particle `color` tweak
- a change to `liquidfun::math::Vec2`, `ParticleColor` or `bitflags` `Debug` output (an upstream crate or dependency update)

D-17 chose this on purpose, and the pin test will catch any such change. But the module doc names only the Rust toolchain as an outside dependency of the hash.

**Fix:** Mention the dependency `Debug` formats, and the fact that cosmetic fields count, in the module doc and in the pin-file header, so that whoever updates a pin knows saved placements will be discarded.

### IN-07: An unlocked card's link name is the whole card text

**File:** `web/src/components/PuzzleSelectPanel.tsx:93-102`

**Issue:** The `<a>` takes its accessible name from all of its content. A screen reader reads the level number, the "Up next" tag, the title, the caption, all three star labels and the footer as one long link name. It is correct, but verbose when moving through links.

**Fix:** Give the title an id. Use `aria-labelledby` pointing at the eyebrow and title ids for the link name, and `aria-describedby` pointing at the caption, stars and footer, so the name stays short ("Level 2, Thick Syrup").

---

_Reviewed: 2026-10-13T17:46:59Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
