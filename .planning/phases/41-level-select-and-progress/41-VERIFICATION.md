---
phase: 41-level-select-and-progress
verified: 2026-10-13T17:54:45Z
status: passed
score: 5/5 roadmap success criteria verified (plus 41/41 plan must-have truths)
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 41-2026-10-13T15-49-07
generated_at: 2026-10-13T17:54:45.421Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 41: Level Select and Progress Verification Report

**Phase Goal:** Players can browse the levels from a Puzzles section and keep their stars, unlocks and last placements across visits in this browser.
**Verified:** 2026-10-13T17:54:45Z
**Status:** passed
**Re-verification:** No (initial verification)
**Verifier:** Claude (gsd-verifier), an AI verifier separate from the implementing agents. This is AI verification, not human approval.

## Goal Achievement

### Observable Truths (ROADMAP success criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | The Puzzles link in site navigation opens `#/puzzles`, a level grid showing each level's title, idea, stars and lock state | VERIFIED | `PlaygroundShell.tsx:74-82` renders a text-only `a.puzzles-nav-link` to `PUZZLES_HASH` (`#/puzzles`) above `DemoNavigation`, with `aria-current` from `puzzlesNavCurrent`. `playable-route.ts:33` parses exactly one `puzzles` segment to `{kind:"puzzles"}`, and `normalizePlayableRoute` never rewrites it. `PlaygroundStage.tsx:249` `<Match kind === "puzzles">` renders `PuzzleSelectPanel`, which builds cards from `levelCardModels` (TS metadata plus persisted progress, no wasm): level number, title, caption, Win/Brim/Thrifty `role="img"` slots, a Locked tag and footer. Locked cards are a non-link `role="group"`. e2e `puzzle-progress.spec.ts:207,277` (fresh grid, sidebar link) pass at 1280x800 and 390x844. I viewed screenshots `target/phase41/screens/grid-fresh-*.png`. |
| 2 | Stars and solved levels persist across reloads, and a later worse run never lowers a saved star | VERIFIED | `playground-session.ts:125` `createPuzzleRuntime(createPuzzleProgress(() => window.localStorage))`. `puzzle-run.ts:62-76` writes stars through `progress.recordStars` after `StarStore.record`, on the running→ended decision before the result card publishes. `puzzle-runtime.ts:53-62` hydrates the `StarStore` from persisted wins. All merges go through `mergeStarFields`→`mergeLevelStars` (`progress.ts`), `withStars` ignores losses, and `commit` re-reads and merges before every write (`puzzle-progress.ts:120-140`). e2e `:455` (reload in a seam-free page shows the stars and "1 of 6 solved") and `:489` (an empty losing run leaves the stored stars `{win,brim,thrifty:true, 90}` unchanged) pass. Unit tests cover the cross-tab never-lower merge. |
| 3 | Levels unlock in order as earlier ones are solved, a direct link opens any level even when locked, and Next on the result card goes to the next level | VERIFIED | `progress-view.ts:19-31` `unlockedLevelIds`: index 0 always unlocked, index i unlocked when level i−1 is solved (and, after WR-01, when level i is itself solved), every level unlocked when degraded. Deep links skip any lock check: `#/puzzle/<id>` goes straight to `beginPuzzleRoute`. e2e `:314` opens locked `timed-gate` in building. `format.ts:245-252` `resultActionsModel`: Next goes to `puzzleHash(nextId)`, and on the last level the same slot becomes `All levels` → `#/puzzles`. e2e `puzzle-levels.spec.ts:125`, `puzzle.spec.ts:91` (Next) and `puzzle-progress.spec.ts:579` (last level → grid, accented and focused) pass. |
| 4 | When storage is unavailable (a throwing getter or setItem, corrupt JSON, or a future version), every level is unlocked, the game stays fully playable, and a note says progress won't be saved | VERIFIED | `puzzle-progress.ts:57-81` `openStorage` degrades on a throwing or null provider, a failing probe `setItem`/`removeItem` on a separate key, a failing `getItem`, or a parse that is not `ok`. `parseProgress` returns `json`, `shape` or `future-version`. A mid-session `setItem` failure calls `degrade` (`tryWrite`). In degraded mode `maybeStorage` is undefined, so nothing is written. `PROGRESS_NOTE` shows under the grid heading (`PuzzleSelectPanel`) and in the options sheet and form-factor row (`PuzzleLevelLinks`), and not on the HUD. Unit tests in `puzzle-progress-shell.test.ts` cover a throwing provider, a null provider, a throwing probe set and remove, a throwing getItem, a mid-session quota error, and unusable seeds staying byte-for-byte. e2e `:361` (blocked `localStorage` getter: every level unlocked, note visible, plays to a won result card), `:408` (future version kept byte-for-byte after play) and `:433` (corrupt JSON kept byte-for-byte) pass. |
| 5 | Reopening a level restores the player's last placements, and saved placements are discarded when the level has changed | VERIFIED | Rust `level_hash.rs` FNV-1a 64 over `level-hash-v1\|{spec:?}\|grid\|tick`, exposed as `levelInfo().levelHash` (`info.rs:124`), pinned by `level-hashes.txt` and checked in both Rust tests and `just puzzle-replay-wasm`. TS parses it fail-closed (`puzzle-session.ts:222`). Saves happen in `commitNow` (`puzzle-editing.ts:83`) and at Play (`puzzle-run.ts:196`), deduped by key. `restorePendingPlacements` runs once per route entry in `buildPuzzle` (`puzzle-run.ts:165`), and the test seam takes precedence (`puzzle-lifecycle.ts:87-88`). `restoreChoice` discards on a hash mismatch, more than 16 entries or any failed strict `maybeParsePlacement`, and `maybeRestore` then removes the placements while keeping the stars. e2e `:520` (restore after reload, and after Play) and `:551` (a stale hash opens empty, drops `placements`, keeps the stars; the live hash restores) pass. |

**Score:** 5/5 truths verified

All plan frontmatter truths for 41-01 through 41-08 were checked against the code above and the gate evidence. None failed.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/liquidfun-wasm/src/puzzle/level_hash.rs` | fnv1a64, level_hash_hex | VERIFIED | Contains `const ENCODING: &str = "level-hash-v1";`. The test module has FNV vectors, pin-file, distinctness and field-change tests. |
| `crates/liquidfun-wasm/src/puzzle/level-hashes.txt` | six pinned hashes | VERIFIED | Six lines, one per level, in registry order |
| `web/scripts/puzzle-replay/level-hashes.ts` | parseLevelHashes | VERIFIED | Used by the replay script and the e2e spec |
| `web/src/puzzle/progress.ts` | v1 document parse/serialize/merge | VERIFIED | Pure, null-prototype records, preserves unknown ids and keys |
| `web/src/puzzle/progress-view.ts` | unlock rule, card models, restoreChoice | VERIFIED | Wired into `PuzzleSelectPanel` and `puzzle-progress.ts` |
| `web/src/puzzle/placements.ts` | maybeParsePlacement | VERIFIED | Imported by `progress-view.ts` |
| `web/src/player/puzzle-progress.ts` | createPuzzleProgress shell | VERIFIED | Wired from `playground-session.ts:125` |
| `web/src/player/puzzle-saved-placements.ts` | save and restore | VERIFIED | Called from `puzzle-editing.ts` and `puzzle-run.ts` |
| `web/src/components/PuzzleSelectPanel.tsx` | grid page | VERIFIED | 145 lines, rendered by `PlaygroundStage` |
| `web/src/styles/puzzle-select.css` | grid styles | VERIFIED | Imported by `PuzzleSelectPanel` |
| `web/src/components/PuzzleLevelLinks.tsx` | All levels link plus note | VERIFIED | Used in the options sheet and the form-factor row |
| `web/e2e/puzzle-progress.spec.ts` | D-20 browser coverage | VERIFIED | 605 lines, 12 tests × 2 viewports, listed in `test:puzzle` |
| `target/phase41/base.txt`, `after-final.txt` | 25 fingerprints | VERIFIED | `diff` is empty, 25 lines |

### Key Link Verification

The `gsd-tools verify key-links` helper reported several false negatives. It could not resolve `from` fields that carry a symbol suffix (for example `puzzle-run.ts recordStars`), and some escaped patterns did not match. I checked every one of those links by hand:

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `info.rs` | `level_hash.rs` | `js_name = levelHash` | WIRED | `info.rs:124-127` |
| `puzzle-replay.ts` | `level-hashes.txt` | `parseLevelHashes` | WIRED | `puzzle-replay.ts:28,276,285`; gate `puzzle-replay-wasm exit=0` |
| `puzzle-session.ts` | `raw.levelHash()` | fail-closed parse | WIRED | `puzzle-session.ts:222` |
| `progress.ts` | `mergeLevelStars` | the only merge | WIRED | `mergeStarFields` |
| `progress-view.ts restoreChoice` | `maybeParsePlacement` | per-entry parse | WIRED | in the loop in `restoreChoice` |
| `puzzle-run.ts recordStars` | `progress.recordStars` | write-through | WIRED | `puzzle-run.ts:71` |
| `playground-session.ts` | `createPuzzleProgress(() => window.localStorage)` | browser provider | WIRED | `:125` |
| `puzzle-runtime.ts` | `StarStore.record` | hydration | WIRED | `hydratedStarStore` |
| `puzzle-run.ts buildPuzzle` | `restorePendingPlacements` | after adopt, before loaded | WIRED | `:165` |
| `puzzle-editing.ts commitNow` | `saveEditorPlacements` | after publishState | WIRED | `:83` |
| `puzzle-run.ts playPuzzle` | `saveEditorPlacements` | after the canPlay guard | WIRED | `:196` |
| `PlaygroundStage.tsx` | `PuzzleSelectPanel` | `<Match kind === "puzzles">` | WIRED | import `:30`, match `:249` |
| `App.tsx` | `session.puzzle.progress` | `puzzleProgress` prop | WIRED | `App.tsx:49` |
| `PlaygroundShell.tsx` | `#/puzzles` | `a.puzzles-nav-link` | WIRED | `:76-77` |
| `PuzzleOptionsDrawer.tsx` | `PuzzleLevelLinks` | first child of the sheet | WIRED | `:44` |
| `PuzzlePanel.tsx` | `controller.progressDegraded()` | sheet and form-factor row | WIRED | `:382,391` |
| `PuzzleResultCard.tsx` | `resultActionsModel` | Next slot | WIRED | `:33` |
| `after-final.txt` | `base.txt` | empty diff | WIRED | rerun by verifier: identical |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `PuzzleSelectPanel` | `progress`, `degraded` | `puzzleProgress.current()` / `.degraded()` signals from `createPuzzleProgress(() => window.localStorage)` | Yes: parsed from localStorage and updated on each commit (e2e reload in a second page shows the stars) | FLOWING |
| `PuzzleResultCard` best stars | `starStore.get(id)` | hydrated from persisted progress plus recordStars merge | Yes | FLOWING |
| Editor seed on entry | `maybeRestore` result | stored `placements` under the live Rust `levelHash` | Yes (e2e restore after reload) | FLOWING |
| Degraded note | `degraded()` | `openStorage` / `degrade` | Yes (e2e blocked, future-version and corrupt cases) | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Phase 41 unit suites (progress core/view/shell/stars, restore, save, route, target, format, chrome, level hashes/info) | `npx vitest run <12 files>` | 12 files, 252 tests passed | PASS |
| TypeScript typecheck | `bun run typecheck` | clean | PASS |
| Progress e2e on desktop and phone, against `dist` built after the last fix commit | `CI=true bunx playwright test e2e/puzzle-progress.spec.ts` | 24 passed (13.1s) | PASS |
| Engine untouched | `git diff --stat f277ecad9 HEAD -- crates/liquidfun` | empty | PASS |
| No Rust change since the gate | `git diff --stat 5e10940e9 HEAD -- crates` | empty | PASS |
| Scene fingerprints | `diff target/phase41/base.txt target/phase41/after-final.txt` | identical (25 lines) | PASS |
| Gate record | `target/phase41/checks-08.results` | 27/27 `exit=0` (fmt, clippy, wasm build/tests, level-hash pin, puzzle-replay-wasm, vitest, player-smoke, hud-safe-frame, puzzle e2e, markdown, grep gates) | PASS |
| Post-fix regression | `target/phase41/regression-post-fix.log` | 1543 unit tests, 81 puzzle e2e, 64 hud-safe-frame passed | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| PUZ-01 | 41-02, 05, 06, 07, 08 | Puzzles section at `#/puzzles` from site navigation, grid with title, idea, stars, lock state | SATISFIED | Truth 1 |
| PROG-01 | 41-02, 03, 07, 08 | Stars and solved levels persist across visits | SATISFIED | Truth 2 |
| PROG-02 | 41-02, 03, 05, 06, 07, 08 | Ordered unlocks; direct links always open any level | SATISFIED | Truth 3 |
| PROG-03 | 41-02, 03, 05, 06, 07, 08 | Storage unavailable: all levels unlocked, playable, note shown | SATISFIED | Truth 4 |
| PROG-04 | 41-01, 02, 04, 07, 08 | Last placements restored; discarded when the level changed | SATISFIED | Truth 5 |

REQUIREMENTS.md maps exactly these five IDs to Phase 41, so no requirement is orphaned. Their "Pending" status is expected; the orchestrator ticks them.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (phase-changed `web/src`, `crates` files) | - | TODO/FIXME/placeholder/console.log | none found | - |
| `web/e2e/shell.spec.ts` | 235, 252 | Plan 41-05 said "passes unedited"; the focus-trap Tab counts went from 26 to 27 for the new Puzzles link | Info | A deliberate, minimal edit, sanctioned in `a75f21c51`. The 25-preview/icon counts are untouched. |
| `web/src/player/puzzle-progress.ts` | 156 | REVIEW IN-01: no cross-tab `storage` listener; restore reads this tab's snapshot | Info | Never lowers stars or corrupts data. Deliberately left for later. |
| `web/src/puzzle/progress.ts` | 92 | REVIEW IN-02: unknown per-level fields are dropped on write | Info | Documented schema choice. A per-level field addition needs a version bump. |
| IN-05 / IN-06 / IN-07 | - | duplicated hash regex, hash doc wording, verbose card link name | Info | No effect on the goal |

### Note on WR-01 (unlock-rule refinement)

After review, `unlockedLevelIds` also treats a level that is itself solved as unlocked (`762fdaa67`). This only unlocks more. An unsolved level still follows the D-10 predecessor rule, so the roadmap contract "levels unlock in order as earlier ones are solved" still holds. The change removes a card that read both "Locked" and "Win: earned" after a deep-link solve. The orchestrator accepted it under the repository's standing autonomous-iteration authorization, and no override entry is needed because no success criterion fails. Unit tests in `puzzle-progress-view.test.ts` cover it, and the e2e deep-link test still sees an unsolved locked level as `data-locked="true"`.

### Human Verification Required

None. I checked the visual result against the screenshots `target/phase41/screens/grid-fresh-1280x800.png`, `grid-fresh-390x844.png` and `last-level-ended-390x844.png`. They show the heading, intro, "0 of 6 solved", three columns on desktop and one on phone, Up next and Locked tags, star slots, lock footers, the site footer, and the accented All levels button in the Next slot. The e2e suite asserts the columns and no horizontal scrolling at 390 px. This was an agent check; the owner may still look at the page.

### Gaps Summary

No gaps. Every ROADMAP success criterion and every plan must-have truth is backed by wired, substantive code with real data flow. I reran the unit and e2e tests myself and they pass. The engine and the 25 scene fingerprints are unchanged.

---

_Verified: 2026-10-13T17:54:45Z_
_Verifier: Claude (gsd-verifier), AI verification_
