---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 41-2026-10-13T15-49-07
generated_at: 2026-10-13T15:52:27.012Z
---

# Phase 41: Level Select and Progress - Context

**Gathered:** 2026-10-13
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Players can open a Puzzles section at `#/puzzles` from site navigation and browse the six levels in a grid that shows each level's title, idea, stars and lock state. Stars, solved levels, unlocks and the last placements per level persist in this browser through localStorage. Levels unlock in order as earlier ones are solved, but a direct link always opens any level. When storage is unavailable, every level is unlocked, the game stays fully playable, and a note says progress won't be saved. Saved placements are discarded when the level has changed.

Requirements: PUZ-01, PROG-01, PROG-02, PROG-03, PROG-04.

Out of scope: sound (Phase 42), GPU water (Phase 43), new levels, a progress reset or export UI, accounts or cloud sync, and changes to the 25 scenes.

</domain>

<decisions>
## Implementation Decisions

### Route and navigation (PUZ-01)

- **D-01:** Add a `{kind:"puzzles"}` variant to `PlayableRoute` in `web/src/routing/playable-route.ts`. It matches exactly `#/puzzles`, and any extra segment is unknown. `maybeParseSceneRoute` and `normalizeSceneRoute` in `web/src/routing/hash.ts` stay unchanged (Phase 38 D-01). `normalizePlayableRoute` must never rewrite `#/puzzles`. An empty hash still lands on the first scene, so the default landing page does not change.
- **D-02:** The grid is a non-canvas page. `maybePlayableTarget` returns `undefined` for it, so the existing `leavePuzzle` and `abandonScene` paths tear down any canvas session. `titleForPlayableRoute` returns "Puzzles", and `playableRouteIdentity` gets a `puzzles` identity so the mobile drawer closes on navigation. `PlaygroundStage` gets a new `<Match>` that renders a `PuzzleSelectPanel` (name at Claude's discretion). The grid uses no `data-hud-obstacle`, because that attribute is only for canvas-overlay chrome. The site footer shows on this page, as it does on other non-canvas pages.
- **D-03:** Site navigation gets a text-only "Puzzles" link to `#/puzzles` inside `SidebarContent`, above the `DemoNavigation` "Demos" list in `web/src/components/PlaygroundShell.tsx`. It has no preview image and no `svg[aria-hidden]` icon, so `e2e/shell.spec.ts`'s count of 25 previews and icons in `.demo-sidebar` stays valid without edits. The link shows a current-page state on `#/puzzles` and on `#/puzzle/<id>`. The scene list, `SCENE_IDS` and README plans stay untouched.
- **D-04:** Ways back to the grid:
  - The "Puzzle options" drawer (Phase 38 D-24) gains an "All levels" link to `#/puzzles`.
  - The unknown-puzzle fallback (`PuzzleFallbackPanel`) makes "All levels" (`#/puzzles`) its primary link and keeps the link back to the scenes as secondary. Update its tests to match.
  - On the last level, the result card's Next slot becomes an enabled "All levels" link to `#/puzzles`. It replaces the disabled "No next level yet" button, sits in the same slot and keeps the same box size (D-12).

### Level grid content and layout (PUZ-01)

- **D-05:** The grid has one card per `PUZZLE_IDS` entry, in registry order. Each card shows:
  - the level number (1–6)
  - the title from `PUZZLE_TITLES`
  - the idea from `PUZZLE_CAPTIONS`
  - three star slots labelled Win, Brim and Thrifty, filled when earned. Each slot has an accessible label such as "Brim: earned" or "Brim: not yet".
  - the lock state

  The grid needs nothing from wasm. It renders from TS metadata and the persisted progress only, so it appears before wasm loads and never builds a physics session. Showing a small "Best fill N%" line from `bestPeakHeldPct` is at Claude's discretion. Par and tool budgets are not shown, because they would need a TS mirror of Rust data.
- **D-06:** An unlocked card is a link (`<a href="#/puzzle/<id>">`) covering the whole card, with a touch target of at least 44 px. A locked card shows a lock icon and the text "Solve {previous title} to unlock". It is not a link: render it as a non-interactive element whose accessible name includes "locked". Locking is soft. A typed or shared `#/puzzle/<id>` link opens a locked level normally (D-10).
- **D-07:** The layout is a responsive grid: one column at phone width (390 px) and two or three columns on wider viewports. It uses the existing dark theme tokens and shadcn-solid styling (see `standards-overrides.md`) and must not scroll horizontally at 390 px. A short page heading "Puzzles" sits above the grid, with a one-line intro such as "Get enough water into the cup before time runs out." The storage note from D-15 shows under the heading when it applies.

### Progress model and unlocks (PROG-01, PROG-02)

- **D-08:** Add a separate persistence module, `web/src/puzzle/progress.ts` (or `src/player/puzzle-progress.ts`; location at Claude's discretion). Keep it apart from `placements.ts`, which `tests/puzzle-placements.test.ts:239` keeps free of storage access. Split it into a pure core and a thin shell:
  - The pure core does parsing, validation, merging and the unlock rule, and is fully unit-tested.
  - The shell takes a storage provider (`() => window.localStorage`) and wraps every access in try/catch, following `first-hint-preference.ts` and `hud-sliders-preference.ts`.
- **D-09:** Stars are the Phase 39 `StarStoreSnapshot` shape, `{levelId → {win, brim, thrifty, bestPeakHeldPct}}`. The runtime `StarStore` is hydrated from the persisted progress at startup. Each `record` writes through to storage on the existing `recordDecision` path. `mergeLevelStars` is the only merge, so a saved star never goes down. A write first re-reads the stored record and merges with it, so two tabs cannot lower each other's stars. A level counts as **solved** when its `win` star is earned.
- **D-10:** The unlock rule is a pure function. Level index 0 is always unlocked. Level `i` is unlocked when level `i − 1` is solved. In degraded mode (D-14), every level is unlocked. The lock state appears only on the grid. A direct `#/puzzle/<id>` link always opens the level, with no locked banner and no redirect.
- **D-11:** Unlocks update live. When a run wins, the store records it before the result card renders, so the grid and the next level's lock state are already current when the player navigates.
- **D-12:** Next on the result card keeps linking to the next entry in `PUZZLE_IDS`, as in Phase 38 D-15. Because locks are soft, Next opens the next level even after a loss. On the last level, Next becomes "All levels" (D-04). `resultActionsModel` in `web/src/puzzle/format.ts` owns this, and its Vitest coverage is updated.

### Storage schema and degraded mode (PROG-03)

- **D-13:** One key, `liquidfun.puzzle-progress.v1`, holds a JSON document:
  ```
  { "version": 1,
    "levels": { "<levelId>": { "win": bool, "brim": bool, "thrifty": bool,
                               "bestPeakHeldPct": number,
                               "placements"?: { "levelHash": string,
                                                "tools": [{kind,x,y,angle,openTick}] } } } }
  ```
  - Entries for level ids this build does not know are preserved unchanged on write. They are not shown.
  - The Phase 40 first-hint key `liquidfun.puzzle-first-hint.v1` stays a separate key with unchanged behavior. Do not migrate it. Reuse its provider and try/catch pattern.
- **D-14:** **Degraded mode** means storage is unavailable. It turns on at startup when any of these happen:
  - the `localStorage` getter throws or returns null
  - a probe `setItem` or `removeItem` on a separate probe key throws
  - the stored value is not valid JSON
  - the JSON does not match the v1 schema (missing or wrong-typed fields, `bestPeakHeldPct` out of range)
  - `version` is greater than 1 (a future version)

  It also turns on mid-session when any later `setItem` throws, for example on quota.

  In degraded mode:
  - every level is unlocked
  - the game stays fully playable
  - stars and placements keep working in memory for the page session
  - **nothing is written to storage**, so corrupt or future-version data is never overwritten or destroyed
- **D-15:** The degraded-mode note reads: "Progress can't be saved in this browser, so every level is unlocked." It is plain text, with the exact wording at Claude's discretion. It shows under the grid heading and inside the "Puzzle options" drawer on puzzle routes. It is not added to the canvas HUD, so the HUD obstacle set and the safe frame stay unchanged. A level opened by a deep link stays fully playable without the note being visible on the canvas.

### Saved placements and level change detection (PROG-04)

- **D-16:** The last placements are saved per level during building, after each committed edit: place, drop at the end of a drag, rotate, nudge, remove, clear and valve-time changes. They are also saved at Play. Pointer-move frames during a drag are not saved. Clear saves an empty list. Saving stores the editor's placement order (`placementsInOrder`) and the level hash. Invalid placements (red tools) are saved too, and the validator marks them again on restore.
- **D-17:** **Level hash.** The bridge exposes a deterministic level fingerprint, `levelInfo().levelHash` (name at Claude's discretion). It is a fixed-width hex string: FNV-1a 64 over a canonical encoding of the level's `LevelSpec`. The preferred encoding is the `Debug` formatting that `LevelSpec` already derives, because it covers every field automatically and Rust float formatting is platform-independent. The planner may choose an explicit field encoding instead, if research finds Debug unsuitable.
  - Add a Rust unit test that pins the current hash of each of the six levels. Changing a level spec then fails the test, and the author must update the pin consciously. That update is what invalidates saved placements.
  - Add a second test showing that the hash is identical in native and wasm builds, or that it depends only on integer and Debug-string data.
  - This is a read-only getter. `crates/liquidfun` is not modified, and all 25 scene fingerprints must stay identical.
- **D-18:** Restore runs on level entry, after `levelInfo()` is available, and seeds the editor through `editorFromPlacements`. The precedence is:
  1. the test-only seam `maybeReadTestPlacements` (unchanged)
  1. saved placements, when their `levelHash` equals the live hash and every entry passes a strict parser (reuse `parseSeamPlacement` or its checks)
  1. empty placements

  When the hash differs or any entry fails to parse, the saved placements for that level are discarded silently, and stars are kept. Saved placements are removed on the next write. Retry, Edit and Stop keep their Phase 39 behavior within the session.

### Testing and done criteria

- **D-19:** Vitest for the progress core and shell covers:
  - a throwing getter, a null storage, a throwing probe `setItem`, and a mid-session throwing `setItem`
  - corrupt JSON, wrong-shape JSON and a future version. Each turns on degraded mode, and no write happens afterwards.
  - preservation of unknown level ids
  - a never-lowering merge, including a stored record from another tab that has more stars
  - the unlock rule, including degraded mode unlocking all levels
  - a placement round trip, discard on a hash mismatch, and discard on a malformed placement
  - `resultActionsModel` showing "All levels" on the last level
  - the route parser for `#/puzzles` and its extra-segment unknown case
- **D-20:** Playwright adds a `puzzle-progress.spec.ts` (name at Claude's discretion), run on desktop and on phone portrait. It covers:
  - Grid on fresh storage: six cards in order, with titles and captions. Level 1 is unlocked and levels 2–6 are locked. The sidebar link opens it.
  - Solve level 1 through the existing `installSolutionPlacements` seam, reload `#/puzzles`, and see level 1's stars and level 2 unlocked.
  - A worse later run, such as an empty losing run on a solved level, leaves the saved stars unchanged after a reload.
  - A deep link to a locked level opens it in building.
  - Blocked storage, using the `Object.defineProperty` init-script pattern from `e2e/puzzle-hint.spec.ts`: every level is unlocked, the note is visible, and a level is still playable through to its result card.
  - A future-version record is seeded. The note shows, and the stored value is byte-for-byte unchanged after play.
  - Placements are restored after a reload. A seeded record with a wrong `levelHash` opens with no tools.
  - Next on the last level goes to `#/puzzles`.
  - Existing shell, puzzle and hint specs pass, with only the intended fallback and last-level updates.
- **D-21:** The phase is done only if all of these hold:
  - unit tests pass (`bun run test:unit`, or the repository's equivalent)
  - the puzzle Playwright specs, including the new one, pass
  - `just web-player-smoke` passes
  - all 25 scene fingerprints are identical
  - `crates/liquidfun` is not modified
  - Rust checks for `liquidfun-wasm` pass: fmt, clippy with `-D warnings`, and tests, including the pinned level-hash test
  - `just markdown-check` passes if non-GSD Markdown changed

### Claude's Discretion

- Component and module names, file locations, and the exact copy for the intro and the note
- Showing "Best fill N%" on cards
- The exact level-hash encoding (Debug string or explicit fields), provided it is deterministic and pinned
- Whether the grid adds small progress text, such as "3 of 6 solved"
- The visual design of cards and locks within the existing theme. Plan-phase may produce a UI-SPEC because the UI hint is set.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements and roadmap

- `.planning/ROADMAP.md` § Phase 41: goal and the five success criteria
- `.planning/REQUIREMENTS.md`: PUZ-01, PROG-01..PROG-04
- `.planning/PROJECT.md` § Current Milestone v1.5 and the v1.5 owner decisions in `.planning/STATE.md` (soft linear locks, deep links always open, blocked storage unlocks every level and shows a note, stars keep the best per criterion)

### Prior phase decisions

- `.planning/phases/38-puzzle-route-and-player-shell/38-CONTEXT.md`: D-01 (wrapper route parser), D-03 (`#/puzzles` deferred here), D-05 (unknown-puzzle fallback), D-14/D-15 (result card slot and Next), D-18 (HUD obstacles), D-24 (Puzzle options drawer)
- `.planning/phases/39-place-then-play-tools-and-stars/39-CONTEXT.md`: D-14 (star-store shape meant for serialization), D-15 (recording on Stop after a win), D-16 (criteria display), D-25 (editor state)
- `.planning/phases/40-timed-valve-and-teaching-levels/40-CONTEXT.md`: D-02 (stride-4 pose lane and `open_tick`), D-20 (versioned storage key and try/catch pattern), D-21 (captions in TS metadata for this grid)

### Repository rules

- `AGENTS.md` / `CLAUDE.md`: repo-local guidance (portrait frames and README plans apply to scenes only, so they do not apply to this grid)
- `standards/core/frontend-ui.md`: dark default and source-link chrome
- `standards-overrides.md`: shadcn-solid shell override
- `standards/core/testing.md` and `standards/languages/typescript-javascript.md`

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `web/src/puzzle/star-store.ts` (`StarStore`, `StarStoreSnapshot`) and `web/src/puzzle/stars.ts` (`mergeLevelStars`, which never lowers): persist these as-is.
- `web/src/catalog/puzzle-levels.ts`: `PUZZLE_TITLES`, `PUZZLE_CAPTIONS`, `FIRST_PUZZLE_ID`, `puzzleHash(id)` and `maybeNextPuzzleId(id)`. Everything the grid needs before wasm loads.
- `web/src/player/first-hint-preference.ts` and `hud-sliders-preference.ts`: the storage provider and try/catch pattern.
- `web/src/puzzle/placements.ts`: `PuzzlePlacement`, `checkPlacement` and `parseSeamPlacement`, a strict parser for untrusted placement data.
- `web/src/puzzle/editor.ts`: `editorFromPlacements`, `placementsInOrder` and `canonicalPlacements`.
- `web/src/puzzle/frame-hash.ts`: an FNV-1a 64 implementation in TS, used as a reference only. The level hash itself is computed in Rust.
- `crates/liquidfun-wasm/src/puzzle/spec.rs`: `LevelSpec` and the types it uses derive `Debug`. `info.rs` holds the `levelInfo` getters.

### Established Patterns

- Routing: `web/src/routing/playable-route.ts` wraps the scene parser, and `web/src/player/playable-target.ts` maps routes to targets, titles and identities.
- View switch: the `<Switch>`/`<Match>` in `web/src/components/PlaygroundStage.tsx:167`.
- Hash changes: `web/src/player/scene-lifecycle.ts:256-296` tears down the puzzle or scene when the next route has no target.
- Star recording: `recordDecision` in `web/src/player/puzzle-run.ts:61-79`.
- Placement seed: `web/src/player/puzzle-lifecycle.ts:85-86` (`maybeReadTestPlacements(window, id) ?? EMPTY_PLACEMENTS`).

### Integration Points

- `PlaygroundShell.tsx:61-78`: the sidebar link.
- `PuzzleResultCard.tsx` and `format.ts` `resultActionsModel`: the last-level Next.
- `PuzzleFallbackPanel.tsx`: the All levels link.
- The Puzzle options drawer: the All levels link and the note.
- `puzzle-runtime.ts:30,64`: StarStore creation, where hydration from progress happens.
- `info.rs`: the new `levelHash` getter.

### Test Layout

- Vitest: `web/tests/*.test.ts`, run in the node environment.
- Playwright: `web/e2e/*.spec.ts`. Helpers are in `e2e/puzzle-helpers.ts` (`puzzlePath`, `installSolutionPlacements`, viewports). Add the new spec to `package.json` `test:puzzle`.
- `e2e/shell.spec.ts:330-335` counts 25 previews and icons in the sidebar. Do not add an icon to the Puzzles link.

</code_context>

<specifics>
## Specific Ideas

- Owner decisions carried from v1.5 planning: soft linear locks, deep links always open, blocked storage unlocks every level and shows a note, and stars keep the best result per criterion across runs.
- "Fail closed" is the house style for untrusted data: a malformed record turns on degraded mode, and a malformed placement is discarded. Neither crashes the page or silently overwrites data.

</specifics>

<deferred>
## Deferred Ideas

- A "Reset progress" or export/import control, as a separate small task if wanted
- Showing par or the tool mix on grid cards, which would need a TS mirror of Rust level data
- A README puzzle gallery section
- Migrating the first-hint key into the progress document
- Phase 40 deferred item: the tool tray truncates names when a level allows two tool kinds at 390×844. That is tray UI, not part of this phase.

</deferred>

---

*Phase: 41-level-select-and-progress*
*Context gathered: 2026-10-13*
