# Phase 41: Level Select and Progress - Research

**Researched:** 2026-10-13
**Domain:** SolidJS hash routing, localStorage persistence with fail-closed parsing, and a read-only Rust/wasm level fingerprint
**Confidence:** HIGH (every integration point was read in the codebase; the f32 Debug portability between native and wasm was checked by experiment)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Route and navigation (PUZ-01)

- **D-01:** Add a `{kind:"puzzles"}` variant to `PlayableRoute` in `web/src/routing/playable-route.ts`. It matches exactly `#/puzzles`, and any extra segment is unknown. `maybeParseSceneRoute` and `normalizeSceneRoute` in `web/src/routing/hash.ts` stay unchanged (Phase 38 D-01). `normalizePlayableRoute` must never rewrite `#/puzzles`. An empty hash still lands on the first scene, so the default landing page does not change.
- **D-02:** The grid is a non-canvas page. `maybePlayableTarget` returns `undefined` for it, so the existing `leavePuzzle` and `abandonScene` paths tear down any canvas session. `titleForPlayableRoute` returns "Puzzles", and `playableRouteIdentity` gets a `puzzles` identity so the mobile drawer closes on navigation. `PlaygroundStage` gets a new `<Match>` that renders a `PuzzleSelectPanel` (name at Claude's discretion). The grid uses no `data-hud-obstacle`, because that attribute is only for canvas-overlay chrome. The site footer shows on this page, as it does on other non-canvas pages.
- **D-03:** Site navigation gets a text-only "Puzzles" link to `#/puzzles` inside `SidebarContent`, above the `DemoNavigation` "Demos" list in `web/src/components/PlaygroundShell.tsx`. It has no preview image and no `svg[aria-hidden]` icon, so `e2e/shell.spec.ts`'s count of 25 previews and icons in `.demo-sidebar` stays valid without edits. The link shows a current-page state on `#/puzzles` and on `#/puzzle/<id>`. The scene list, `SCENE_IDS` and README plans stay untouched.
- **D-04:** Ways back to the grid:
  - The "Puzzle options" drawer (Phase 38 D-24) gains an "All levels" link to `#/puzzles`.
  - The unknown-puzzle fallback (`PuzzleFallbackPanel`) makes "All levels" (`#/puzzles`) its primary link and keeps the link back to the scenes as secondary. Update its tests to match.
  - On the last level, the result card's Next slot becomes an enabled "All levels" link to `#/puzzles`. It replaces the disabled "No next level yet" button, sits in the same slot and keeps the same box size (D-12).

#### Level grid content and layout (PUZ-01)

- **D-05:** The grid has one card per `PUZZLE_IDS` entry, in registry order. Each card shows:
  - the level number (1–6)
  - the title from `PUZZLE_TITLES`
  - the idea from `PUZZLE_CAPTIONS`
  - three star slots labelled Win, Brim and Thrifty, filled when earned. Each slot has an accessible label such as "Brim: earned" or "Brim: not yet".
  - the lock state

  The grid needs nothing from wasm. It renders from TS metadata and the persisted progress only, so it appears before wasm loads and never builds a physics session. Showing a small "Best fill N%" line from `bestPeakHeldPct` is at Claude's discretion. Par and tool budgets are not shown, because they would need a TS mirror of Rust data.
- **D-06:** An unlocked card is a link (`<a href="#/puzzle/<id>">`) covering the whole card, with a touch target of at least 44 px. A locked card shows a lock icon and the text "Solve {previous title} to unlock". It is not a link: render it as a non-interactive element whose accessible name includes "locked". Locking is soft. A typed or shared `#/puzzle/<id>` link opens a locked level normally (D-10).
- **D-07:** The layout is a responsive grid: one column at phone width (390 px) and two or three columns on wider viewports. It uses the existing dark theme tokens and shadcn-solid styling (see `standards-overrides.md`) and must not scroll horizontally at 390 px. A short page heading "Puzzles" sits above the grid, with a one-line intro such as "Get enough water into the cup before time runs out." The storage note from D-15 shows under the heading when it applies.

#### Progress model and unlocks (PROG-01, PROG-02)

- **D-08:** Add a separate persistence module, `web/src/puzzle/progress.ts` (or `src/player/puzzle-progress.ts`; location at Claude's discretion). Keep it apart from `placements.ts`, which `tests/puzzle-placements.test.ts:239` keeps free of storage access. Split it into a pure core and a thin shell:
  - The pure core does parsing, validation, merging and the unlock rule, and is fully unit-tested.
  - The shell takes a storage provider (`() => window.localStorage`) and wraps every access in try/catch, following `first-hint-preference.ts` and `hud-sliders-preference.ts`.
- **D-09:** Stars are the Phase 39 `StarStoreSnapshot` shape, `{levelId → {win, brim, thrifty, bestPeakHeldPct}}`. The runtime `StarStore` is hydrated from the persisted progress at startup. Each `record` writes through to storage on the existing `recordDecision` path. `mergeLevelStars` is the only merge, so a saved star never goes down. A write first re-reads the stored record and merges with it, so two tabs cannot lower each other's stars. A level counts as **solved** when its `win` star is earned.
- **D-10:** The unlock rule is a pure function. Level index 0 is always unlocked. Level `i` is unlocked when level `i − 1` is solved. In degraded mode (D-14), every level is unlocked. The lock state appears only on the grid. A direct `#/puzzle/<id>` link always opens the level, with no locked banner and no redirect.
- **D-11:** Unlocks update live. When a run wins, the store records it before the result card renders, so the grid and the next level's lock state are already current when the player navigates.
- **D-12:** Next on the result card keeps linking to the next entry in `PUZZLE_IDS`, as in Phase 38 D-15. Because locks are soft, Next opens the next level even after a loss. On the last level, Next becomes "All levels" (D-04). `resultActionsModel` in `web/src/puzzle/format.ts` owns this, and its Vitest coverage is updated.

#### Storage schema and degraded mode (PROG-03)

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

#### Saved placements and level change detection (PROG-04)

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

#### Testing and done criteria

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

### Deferred Ideas (OUT OF SCOPE)

- A "Reset progress" or export/import control, as a separate small task if wanted
- Showing par or the tool mix on grid cards, which would need a TS mirror of Rust level data
- A README puzzle gallery section
- Migrating the first-hint key into the progress document
- Phase 40 deferred item: the tool tray truncates names when a level allows two tool kinds at 390×844. That is tray UI, not part of this phase.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PUZ-01 | Player can open a Puzzles section (`#/puzzles`) from site navigation and see a level grid with title, idea, stars and lock state. | Route variant and the five touch points under "Pattern 1". Sidebar link placement under "Pattern 6". Canvas-stage scrolling and footer under Pitfall 1. Grid model is pure and built from `PUZZLE_TITLES`/`PUZZLE_CAPTIONS` plus the progress signal (Pattern 3). |
| PROG-01 | Stars and solved levels persist in this browser across visits. | Progress document, fail-closed parser and write-through on `recordStars` (Patterns 2–4). Merging with a re-read of storage uses `mergeLevelStars` (Code Example 2). |
| PROG-02 | Levels unlock in order; direct links always open any level. | Pure `unlockedLevelIds(doc, degraded)` (Code Example 2). Deep links need no change, since `handleHashChange` already opens any allowlisted level. Next/All levels via `resultActionsModel` (Pattern 7). |
| PROG-03 | Storage unavailable: all unlocked, fully playable, note shown. | Probe design (Pattern 4 and the Safari/Firefox/Chrome notes). Degraded flag as a signal. The note goes on the grid and in the options sheet, plus the form-factor control row (Pitfall 6). |
| PROG-04 | Last placements restored; discarded when the level changed. | Rust `levelHash` getter (Pattern 5, with the native/wasm equality verified). Restore point inside `buildPuzzle` before the `loaded` dispatch (Pattern 8). Save hook in `commitNow` and `playPuzzle` (Pattern 9). Cap of 16 entries (Pitfall 4). |
</phase_requirements>

## Project Constraints (from CLAUDE.md / AGENTS.md)

- `crates/liquidfun` must not change. A puzzle-bridge change requires 25/25 identical scene fingerprints (`[v1.5]` STATE decision).
- Levels live in `PUZZLE_IDS` / `LevelId` only and must never enter `SCENE_IDS`. The README gallery, portrait frames and scene-legend rules apply to **scenes only**, so a new route needs no README plan and no portrait frame.
- `data-hud-obstacle` is for on-canvas chrome leaves only. The grid page and the options-sheet content get none.
- Rust: no `unwrap()`. Use `?` and `let...else`, `maybe_` names for `Option`, docs on public items (`missing_docs` warns), and `clippy::pedantic` under `-D warnings`. Use `#[must_use]` on getters, as `info.rs` does. Formatting goes through `cargo fmt --all`.
- TS: `maybe` prefix for nullable names, early returns, data-in/data-out core with a thin shell, and parse untrusted input into strong types. Tests use explicit `// Arrange`, `// Act` and `// Assert`, one concern per test.
- File-length trigger: 628 lines (`bun scripts/bright-builds-check.ts file-lengths`). `crates/liquidfun-wasm/src/puzzle/levels_tests.rs` is already at **620** lines, so put the hash pin test in a new file. `tests/puzzle-lifecycle.test.ts` (500), `tests/puzzle-run.test.ts` (486) and `src/styles/puzzle.css` (450) also call for new files rather than growth.
- UI stack: shadcn-solid (Tailwind v4, Kobalte and Corvu drawer), per `standards-overrides.md`. Dark default, with tokens in `src/styles/tailwind.css` (`--background #0b0f14`, `--secondary #151d28`, `--border #2a3441`, `--primary #4da3ff`). Panels use `#151D28` and `#2A3441` (`app.css` `.fallback-panel`).
- The source link must stay reachable in product chrome (`standards/core/frontend-ui.md`, level `must`). `SiteFooter` contains the GitHub link.
- `.planning/**` must never be run through mdformat. Non-GSD Markdown changes need `just markdown-check` (mdformat 1.0.0, Python 3.13).
- Execute on the main tree, without worktrees (user memory). GSD summaries need lifecycle provenance frontmatter (user memory). `gsd-tools commit` drops the Co-Authored-By trailer (user memory), so add the trailer manually.
- macOS: a new test binary waits about 2 min on a security scan (user memory). Budget for it in the Rust test steps.

## Summary

The phase is mostly TypeScript wiring around one small Rust getter. Every integration point named in CONTEXT.md exists as described, with one addition: the restore cannot happen in `beginPuzzleRoute`. That function dispatches `reset` with the seed placements **before** wasm loads (`puzzle-lifecycle.ts:85-86`). `levelInfo()` only exists after `loadPuzzleSession` resolves inside `buildPuzzle` (`puzzle-run.ts:150-160`). The preview session is always built with empty lanes (`lanesFor(current, false)`), and the state is still `loading` at that point. So the clean restore point is in `buildPuzzle`, after `adoptPuzzleSession` and before `dispatch({kind:"loaded"})`: re-dispatch `reset` with the restored placements and the current speed. That point is guarded by a per-entry "restore pending" flag, so Retry, Edit and Try again never restore. The reducer stays unchanged and pure.

Save hooks belong in the imperative shell. `commitNow` in `player/puzzle-editing.ts` is the single funnel for every committed edit (place, select, rotate, nudge, remove, clear, valve time) and for drag end (`inspectNow`, called by the editor input's `settle`). Live drag frames go through the separate `mode === "live"` branch and never reach `commitNow`. Adding a deduplicated `savePlacements` call after `commitNow` and at the top of `playPuzzle` covers D-16 exactly. Star write-through has one funnel too: `recordStars` in `puzzle-run.ts`, used by both `recordDecision` and `recordStop`.

On the Rust side, `LevelSpec` and every nested type (`Vec2`, `ParticleColor`, `GoalRect`, `GridRect`, `ToolAllowance`, `LiquidMaterial`, `SystemTuning`, `FloatBody`, `LiquidBlock` and `LevelId`) derive `Debug`. No field is a bitflags value. I compiled the same 1.97.0 toolchain for `aarch64-apple-darwin` and `wasm32-unknown-unknown` and formatted about 2,000 f32 values plus edge cases (`1e-7`, `1e16`, `-0.0`, subnormal minimum, `usize`, an escaped `&str`) with `{:?}`. FNV-1a 64 of the strings matched exactly (`cbd31c8cdee2e581`). The real caveat is that derived Debug output is **not stable across Rust versions**. The toolchain is pinned to 1.97.0, and the pin test catches a toolchain bump, so the Debug encoding is acceptable with an encoding-version prefix.

**Primary recommendation:** Use one pure `puzzle/progress.ts` (parse, merge, unlock and placement restore) and one shell `player/puzzle-progress.ts`, which exposes a `degraded()` signal and a `document()` signal and is injected into `createPuzzleRuntime`. Hydrate `StarStore` by calling `record()`, so `StarStore` itself needs no change. Write through in `recordStars`. Restore in `buildPuzzle` before `loaded`, and save in `commitNow` and `playPuzzle`. Add a Rust `levelHash` getter, pinned through a shared `level-hashes.txt` that both the native test and the wasm replay check.

## Standard Stack

No new dependencies. Everything uses what is already pinned.

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| solid-js | 1.9.15 [VERIFIED: web/package.json] | Signals for the `degraded` and `document` accessors, and the `<Match>` route view | Existing UI framework |
| Web Storage API (`window.localStorage`) | browser | Persistence | Existing pattern (`first-hint-preference.ts`, `hud-sliders-preference.ts`, `appearance`) |
| wasm-bindgen | =0.2.128 [VERIFIED: crates/liquidfun-wasm/Cargo.toml] | `#[wasm_bindgen(js_name = levelHash)]` getter | Existing bridge |
| vitest | 5.0.1 [VERIFIED: web/package.json] | Pure core and shell unit tests (node env) | Existing |
| @playwright/test | 1.63.0 [VERIFIED: web/package.json] | `puzzle-progress.spec.ts` | Existing |

### Supporting
| Asset | Location | Use |
|-------|----------|-----|
| `mergeLevelStars` | `web/src/puzzle/stars.ts` | The only star merge (it never lowers). Merge two `LevelStars` with `mergeLevelStars(a, b, b.bestPeakHeldPct)` when `b.win`. |
| `StarStore.record` | `web/src/puzzle/star-store.ts` | Hydration and absorbing other-tab stars. `record()` already merges, so no new method is needed. |
| `parseSeamPlacement` checks | `web/src/puzzle/placements.ts` (not exported today) | Export a non-throwing `maybeParsePlacement(entry: unknown)`, or export `parseSeamPlacement` and wrap it. Do **not** import storage into `placements.ts`: `tests/puzzle-placements.test.ts:236-246` greps it for `localStorage`, `sessionStorage`, `location` and `document.`. |
| `editorFromPlacements`, `placementsInOrder` | `web/src/puzzle/editor.ts` | Seeding and save order |
| `StarIcon({filled})` | `web/src/components/PuzzleIcons.tsx:121` | Grid star slots. A lock icon must be added. |
| `PuzzleOptionsSheet` (accepts children) | `web/src/components/PuzzleOptionsDrawer.tsx` | All levels link and note. It is also used by `PuzzleFallbackPanel` in canvas stage. |
| Rust FNV | `scene_spot/fingerprint.rs` is `#[cfg(not(target_arch = "wasm32"))]`, native only | **Do not reuse.** Write a 10-line FNV-1a 64 in `puzzle/level_hash.rs`. |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `Debug` encoding of `LevelSpec` | Explicit field-by-field `to_bits` encoding | Explicit encoding is stable across Rust versions, but every new spec field must be added by hand, and a forgotten field silently keeps stale placements. Debug covers new fields automatically. The pin test catches toolchain-driven changes. Choose Debug (D-17 preference). |
| Re-dispatching `reset` while loading | A new `restore` reducer event | A new event is more explicit but touches the reducer, its exhaustive switch and its tests. Re-dispatching `reset` while `loading && !pendingRun` keeps the reducer unchanged. Either is pure. Choose re-dispatch, unless the planner wants the named event for readability. |
| Separate progress signal | A `progress` field on `PuzzleController` | The grid lives outside `PuzzlePanel` and needs progress without a level. Pass it as its own `PlaygroundStage` prop and give the controller only `progressDegraded()` for the sheet note. |

**Installation:** none.

## Architecture Patterns

### Recommended file layout
```
web/src/puzzle/progress.ts            # pure core: types, parse, serialize, merge, unlock, restore choice
web/src/player/puzzle-progress.ts     # shell: provider, probe, re-read+merge+write, degraded/document signals
web/src/components/PuzzleSelectPanel.tsx  # grid page (no wasm, no data-hud-obstacle)
web/src/styles/puzzle-select.css      # new stylesheet (puzzle.css is 450 lines)
crates/liquidfun-wasm/src/puzzle/level_hash.rs     # fnv1a64 + level_hash_hex(spec) + #[cfg(test)] pins
crates/liquidfun-wasm/src/puzzle/level-hashes.txt  # "first-pour 0123…" × 6; shared by Rust test and wasm replay
web/tests/puzzle-progress.test.ts     # core (parse/merge/unlock/restore)
web/tests/puzzle-progress-shell.test.ts   # shell (providers, probe, degraded, no-write)
web/tests/puzzle-restore.test.ts      # buildPuzzle restore + save hooks via puzzle-host-fixture
web/e2e/puzzle-progress.spec.ts       # added to package.json "test:puzzle"
```

### Pattern 1: Route variant, with every place that must learn it [VERIFIED: codebase]

1. `routing/playable-route.ts`: add `| { readonly kind: "puzzles" }`. In `maybeParsePlayableRoute`, when `parts[0] === "puzzles"` and `parts.length === 1`, return `{kind:"puzzles"}`. Any other `puzzles/...` falls through to `maybeParseSceneRoute`, which yields `{kind:"unknown", maybeRaw:"puzzles/x"}` ("Scene not found"). `normalizePlayableRoute` must early-return `puzzles` along with `puzzle` and `unknown-puzzle`, because otherwise it delegates to `normalizeSceneRoute`. A trailing slash (`#/puzzles/`) collapses to one part because of `.filter(part => part.length > 0)`, so it is the grid.
1. `player/playable-target.ts`: `maybePlayableTarget` returns `undefined` for `puzzles`, which the existing `if (route.kind === "unknown-puzzle")` branch already does. `titleForPlayableRoute` handles `puzzles`. `playableRouteIdentity` gets `case "puzzles": return "puzzles";`, and its `assertNever` makes TypeScript flag a missing case.
1. `player/runtime.ts` `maybeReadySceneId` and `titleForRoute` already return `undefined` or the default for non-scene kinds, so they need no change.
1. `player/scene-lifecycle.ts` `handleHashChange` (lines 256-296): no change needed. Puzzle → puzzles calls `leavePuzzle()`, then `maybeReadySceneId` is undefined, then `abandonScene()`, which sets `view: fallback`. Puzzles → puzzle has no canvas, so `setView({kind:"loading"})`, then `PuzzlePanel` mounts and the canvas assignment starts the level. The same path already serves unknown-puzzle → puzzle.
1. `player/playground-session.ts` `bindNavigation` effect: no target, so it calls `abandonScene()`. No change.
1. `components/PlaygroundStage.tsx` `<Switch>` (line 167): add `<Match when={props.route().kind === "puzzles"}>` rendering `PuzzleSelectPanel`. Pass `canvasStage`, a `progress` view and a `hasTarget`-free props set. The `main` keeps `data-playback="fallback"`.
1. Tests to update: `tests/playable-route.test.ts:69-77` currently asserts `#/puzzles` → `{kind:"unknown", maybeRaw:"puzzles"}`. Invert it, and add an extra-segment case. `tests/playable-target.test.ts:154` ("keys every route kind distinctly") and the title tests need the new kind. `e2e/puzzle.spec.ts:413-421` asserts "Scene not found" on `#/puzzles`, so change it to the grid heading. Its other assertions still hold: `a[href*="#/puzzle/"]` does not match `#/puzzles`, and no "First Pour" link appears on a scene page.

### Pattern 2: Progress document (pure core)

```typescript
// web/src/puzzle/progress.ts — shape only; names at discretion
export const PROGRESS_STORAGE_KEY = "liquidfun.puzzle-progress.v1";
export const PROGRESS_VERSION = 1;
export const MAX_SAVED_PLACEMENTS = 16;          // mirrors Rust MAX_PLACEMENTS (tools.rs:12)
const LEVEL_HASH_PATTERN = /^[0-9a-f]{16}$/;

export type SavedPlacements = { readonly levelHash: string; readonly tools: readonly unknown[] }; // tools parsed lazily at restore
export type LevelProgress = LevelStars & { readonly maybePlacements: SavedPlacements | undefined };
export type ProgressDocument = {
  /** Null-prototype, known ids only. */
  readonly levels: Readonly<Record<string, LevelProgress>>;
  /** Null-prototype, raw JSON values of ids this build does not know (D-13). */
  readonly unknownLevels: Readonly<Record<string, unknown>>;
};
export type ParsedProgress =
  | { readonly kind: "ok"; readonly document: ProgressDocument }
  | { readonly kind: "unusable"; readonly reason: "json" | "shape" | "future-version" };

export function parseProgress(maybeRaw: string | null): ParsedProgress;   // null → ok + empty document
export function serializeProgress(document: ProgressDocument): string;
export function mergeProgress(stored: ProgressDocument, local: ProgressDocument): ProgressDocument; // stars never lower
export function withStars(document, id: PuzzleId, stars: LevelStars): ProgressDocument;
export function withPlacements(document, id, maybeSaved: SavedPlacements | undefined): ProgressDocument;
export function unlockedLevelIds(document, degraded: boolean): ReadonlySet<PuzzleId>;
export function maybeRestorablePlacements(document, id, liveHash: string): readonly PuzzlePlacement[] | undefined;
export function levelCardModels(document, degraded): readonly LevelCardModel[]; // grid view model
```

Rules to encode:
- **Strict at startup on star fields:** `win`, `brim` and `thrifty` must be booleans. `bestPeakHeldPct` must be finite and in `[0, 100]`. `version` must be exactly the integer 1. An integer `version > 1` gives `future-version`, and anything else gives `shape`. `levels` must be a non-array object. A known-level record with `brim || thrifty` but not `win` contradicts `mergeLevelStars` (every star requires a win), so treat it as `shape`.
- **Lenient on `placements`:** D-18 says malformed placements are discarded silently while stars are kept, so a bad `placements` subtree must never trigger degraded mode. Keep the raw `tools` value and validate it only at restore (or drop it during parse). Record this split in a code comment, since it differs from the D-14 "wrong shape → degraded" rule.
- **Not-yet-won levels can exist:** a level with saved placements but no win is stored as `{win:false, brim:false, thrifty:false, bestPeakHeldPct:0, placements:{...}}`. Hydration must skip `win === false` records, or `StarStore.get` would return a "best" before the first win (`star-store.ts` documents undefined before the first win).
- **Unknown ids:** copy the raw JSON value into a null-prototype `unknownLevels` and write it back verbatim.

### Pattern 3: Grid view model (pure)

`levelCardModels(document, degraded)` maps `PUZZLE_IDS` in order to `{id, number, title: PUZZLE_TITLES[id], caption: PUZZLE_CAPTIONS[id], stars: {win, brim, thrifty}, maybeBestFillPct, locked, maybeLockText: "Solve {PUZZLE_TITLES[prev]} to unlock", href: puzzleHash(id)}`. The component only renders this model, so it is cheap to unit-test and contains no `innerHTML`. Add the new component to `PUZZLE_COMPONENTS` in `tests/puzzle-chrome.test.ts` so the wall-clock and `innerHTML` scan covers it.

### Pattern 4: Shell, provider and probe (D-14)

```typescript
// web/src/player/puzzle-progress.ts
export const PROGRESS_PROBE_KEY = "liquidfun.puzzle-progress.probe";
type ProgressStorage = Pick<Storage, "getItem" | "setItem" | "removeItem">;
export type ProgressStorageProvider = () => ProgressStorage | null;

export type PuzzleProgress = {
  readonly degraded: Accessor<boolean>;
  readonly document: Accessor<ProgressDocument>;
  /** Merges a best into memory and, unless degraded, re-reads + merges + writes; returns the merged best. */
  recordStars(id: PuzzleId, best: LevelStars): LevelStars;
  savePlacements(id: PuzzleId, levelHash: string, placements: readonly PuzzlePlacement[]): void;
  discardPlacements(id: PuzzleId): void;
};
export function createPuzzleProgress(provider: ProgressStorageProvider): PuzzleProgress;
```

Startup sequence (once, in `createPlaygroundSession`, inside the Solid root):
1. `try { maybeStorage = provider() } catch → degraded`. If `maybeStorage === null` (or not an object), degraded.
1. Probe: `setItem(PROBE, "1")` then `removeItem(PROBE)`. A throw means degraded. This is deliberately stricter than MDN's `storageAvailable`, which treats `QuotaExceededError` with existing data as "available" [CITED: developer.mozilla.org/en-US/docs/Web/API/Web_Storage_API/Using_the_Web_Storage_API]. D-14 says any throw, so a quota-full origin also goes degraded. That is conservative and fine.
1. `parseProgress(getItem(KEY))`. A result other than `ok`, or a throwing `getItem`, means degraded, with an empty in-memory document and **no write ever**.
1. Write path when not degraded: `getItem` → `parseProgress`. If the re-read is unusable (for example another tab wrote a future version), go degraded and do not write. Otherwise `mergeProgress(stored, local)`, apply the change, `setItem`. On a throw, go degraded. Then `setDocument(merged)`, so stars from another tab appear.

Browser behaviours this design covers:
- Accessing `window.localStorage` throws `SecurityError` when the origin is opaque or a policy blocks persistence, such as blocked cookies or site data [CITED: developer.mozilla.org/en-US/docs/Web/API/Window/localStorage].
- `setItem` throws `QuotaExceededError` when quota runs out [CITED: developer.mozilla.org/en-US/docs/Web/API/Storage/setItem]. Some browsers' private modes have historically offered a zero-quota storage object [CITED: MDN Using the Web Storage API, "Testing for availability"]. Old iOS Safari (before 11) private mode threw on every `setItem` [ASSUMED].
- Firefox with `dom.storage.enabled=false` returns `null` from `window.localStorage` [ASSUMED]. That is why the null check exists.

Inject `PuzzleProgress` into `createPuzzleRuntime(progress)`. Give it a default in-memory, unavailable provider (`() => null`), so the existing node-env tests that call `createPuzzleRuntime()` (`tests/puzzle-host-fixture.ts:191`, `puzzle-shell-wiring.test.ts:128` and `worker-loop-fixture.ts:128`) keep working without a `window`.

### Pattern 5: Rust `levelHash` getter (D-17) [VERIFIED: experiment + codebase]

```rust
// crates/liquidfun-wasm/src/puzzle/level_hash.rs
//! Deterministic level fingerprint for saved-placement invalidation (Phase 41 D-17).
use super::spec::LevelSpec;
use super::tools::GRID_METERS;
use super::valve::STEPS_PER_OPEN_TICK;

/// Bump when the encoding below changes; every saved placement is then discarded.
const ENCODING: &str = "level-hash-v1";
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

fn fnv1a64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(FNV_OFFSET, |hash, &byte| (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME))
}

/// 16 lowercase hex digits of FNV-1a 64 over the spec's `Debug` text plus grid and valve-tick units.
pub(crate) fn level_hash_hex(spec: &LevelSpec) -> String {
    let text = format!("{ENCODING}|{spec:?}|grid={GRID_METERS:?}|tick={STEPS_PER_OPEN_TICK}");
    format!("{:016x}", fnv1a64(text.as_bytes()))
}
```

`info.rs` adds this getter:

```rust
/// Returns the 16-hex-digit level fingerprint that keys saved placements (D-17).
#[must_use]
#[wasm_bindgen(js_name = levelHash)]
pub fn level_hash(&self) -> String { super::level_hash::level_hash_hex(self.spec) }
```

- Include `GRID_METERS` and `STEPS_PER_OPEN_TICK`, because saved poses are grid cells and half-second ticks, and those units live outside `LevelSpec`. Tool geometry such as `PLANK_HALF_LENGTH` is optional, since changing it does not change what a pose means.
- **Pin test:** `include_str!("level-hashes.txt")`, parse `"<id> <hex>"` lines, and assert one line per `LEVEL_IDS` entry, in order, each equal to `level_hash_hex(level_spec(id))`. Also assert 16 lowercase hex digits and that all six hashes are distinct. Generate the pins once by running the test and printing the values.
- **Native == wasm:** extend `web/scripts/puzzle-replay.ts replay` (run by `just puzzle-replay-wasm`) to read `level-hashes.txt` and compare `session.levelInfo().levelHash()` for each level (`readLevel` at line 105 already opens `levelInfo()`). Both targets then check the same committed pins. This is the strongest form of the D-17 second test.
- **Evidence:** a scratch crate on Rust 1.97.0 formatted `{:?}` of a struct with `f32`, `[f32;4]`, `usize`, an escaped `&str`, `Option<f32>`, an enum, and 2,000 f32 bit patterns. `aarch64-apple-darwin`: `cbd31c8cdee2e581`. `wasm32-unknown-unknown` (run in node): `cbd31c8cdee2e581`. [VERIFIED: local experiment] Core float formatting (`flt2dec`) is integer-only, so it does not depend on the platform [ASSUMED, consistent with the experiment].
- **Caveat:** "Derived `Debug` formats are not stable, and so may change with future Rust versions" [CITED: doc.rust-lang.org/std/fmt/trait.Debug.html]. The toolchain is pinned to 1.97.0 (`rust-toolchain.toml`). A toolchain bump that changes Debug fails the pin test, which forces a conscious update. Stars are unaffected, and only placements are discarded.
- All specs are `const`/`static` data, with no runtime trig, so the Debug text is fixed at compile time.
- **TS side:** add `levelHash(): string` to `RawPuzzleLevelInfo` (`physics/puzzle-session.ts:20`). Add `readonly levelHash: string` to `PuzzleLevelInfo`, validated against `/^[0-9a-f]{16}$/` in `parsePuzzleLevelInfo`, which fails closed with `Invalid puzzle level info`. Update the fakes in `tests/puzzle-host-fixture.ts:~80`, `tests/puzzle-level-info.test.ts:~33` and `tests/puzzle-session.test.ts`, and add a rejection case for a bad hash.
- **Cost:** `levelInfo()` is parsed on every session build, which means every Play. Formatting a few KB and hashing it takes microseconds, so it needs no cache.

### Pattern 6: Sidebar link and current state

In `PlaygroundShell.tsx`, inside `<SidebarContent class="demo-sidebar-content">` and before `<DemoNavigation>`, add `<nav aria-label="Puzzles"><a class="puzzles-nav-link" href="#/puzzles" aria-current={props.puzzlesCurrent ? "page" : undefined} onClick={() => sidebar.setOpenMobile(false)}>Puzzles</a></nav>`. Add the `puzzlesCurrent: boolean` prop, computed in `PlaygroundStage` as `route.kind === "puzzles" || route.kind === "puzzle"`. Keep it free of any `svg` and of "Static preview" text, because `e2e/shell.spec.ts:323-336` counts 25 of each. Do not reuse the class `demo-nav-link`. `player-helpers.ts:232` selects `a.demo-nav-link[href="#/scene/…"]`, which would still work, but a distinct class keeps the scene list's semantics clean. In the canvas shell the sidebar is the "Demos" dialog (`canvas-stage.spec.ts:416-420`), so the e2e path is: click the "Demos" button, then `dialog "Demos"`, then the link "Puzzles".

### Pattern 7: Result card on the last level (D-04, D-12)

`resultActionsModel(undefined)` returns `{ next: { enabled: true, label: "All levels", ariaLabel: "All levels", maybeTitle: undefined, maybeHref: "#/puzzles" }, primary: ??? }`. `PuzzleResultCard.tsx` already renders an `<a>` whenever `maybeHref` is set, so the disabled-button fallback becomes dead code and can go. Choosing `primary` is a decision: today it is `"retry"` with no next level. Recommend `"next"`, so the enabled link stays the accented primary in every case. Update `tests/puzzle-format.test.ts:~280-290` and `e2e/puzzle-levels.spec.ts:33-35,109-113`, which currently expect `"Next: No next level yet"`, to expect link "All levels" with `href="#/puzzles"`. The landscape card is 136 px wide, and its Next spans both columns (`puzzle.css:304-328`). "All levels" fits, but take a 390×844 and a landscape screenshot, because the slot box must not change (the fixed-size CSS test in `puzzle-chrome.test.ts:289`).

### Pattern 8: Restore point (D-18), the critical timing [VERIFIED: codebase]

The current order is:
1. `beginPuzzleRoute` (`puzzle-lifecycle.ts:56-97`) calls `maybeReadTestPlacements(window, id) ?? EMPTY_PLACEMENTS`, then `dispatch reset` (state `loading`, `pendingRun: false`), then `buildPuzzle`.
1. `buildPuzzle` (`puzzle-run.ts:136-178`) calls `await loadPuzzleSession(id, lanesFor(current, run))`, then `adoptPuzzleSession` (where `owned.levelInfo` is now available), then `captureStepZero`, then `dispatch loaded` (which goes to `building` with `withBudget`), then `setMaybeLevelInfo`, then `commitInspection` and `presentStepZero`.

Recommended insertion:
- In `beginPuzzleRoute`: `const maybeSeam = maybeReadTestPlacements(window, id);` and set `host.puzzle.maybePendingRestoreId = maybeSeam === undefined ? id : undefined`. This is a new mutable `PuzzleRuntime` field.
- In `buildPuzzle`, between `adoptPuzzleSession(...)` and `dispatch(host, {kind:"loaded", ...})`:

```typescript
const current = host.puzzle.current;
if (host.puzzle.maybePendingRestoreId === id && current.kind === "loading" && !current.pendingRun) {
  host.puzzle.maybePendingRestoreId = undefined;
  const maybeRestored = host.puzzle.progress.maybeRestore(id, info.levelHash); // discards on mismatch/malformed
  if (maybeRestored !== undefined) {
    dispatch(host, { kind: "reset", placements: maybeRestored, speed: current.speed });
  }
}
```

- The preview session is built from empty lanes either way (`lanesFor(current, false)`), so restoring after the build changes nothing in the step-0 frame. `commitInspection` after `loaded` then inspects the restored editor, marking invalid tools red and gating Play (D-16).
- Clear the flag in `leavePuzzle`. If the first build fails before this point, the flag survives, and Try again restores, which is acceptable. `retryPuzzle`, `editPuzzle` and `tryAgainPuzzle` never set the flag.
- Discard on a hash mismatch or a malformed entry: call `progress.discardPlacements(id)`. That is itself a normal re-read, merge and write that drops the level's `placements` and keeps its stars, and it is skipped in degraded mode. This satisfies "removed on the next write".
- First-pour hint interplay: `PuzzlePanel.tsx:144-152` dismisses the first-level hint for good when `state.editor.tools.length > 0` in building. Restored tools on first-pour therefore dismiss the hint. That matches D-20 of Phase 40, since a saved tool implies the player already placed one, but call it out in the plan.

### Pattern 9: Save hook (D-16)

- `player/puzzle-editing.ts` `commitNow(host)` is reached by every `applyPuzzleEdit(..., "commit")` and by `inspectNow` (drag end, through `scene-surface.ts:213` `settle`). After `publishState`, call `saveEditorPlacements(host)`.
- `player/puzzle-run.ts` `playPuzzle`: call `saveEditorPlacements(host)` after the `canPlay` guard and before the build.
- `saveEditorPlacements` reads `host.puzzle.maybeLevelId()`, `host.puzzle.maybeLevelInfo()?.levelHash` and `placementsInOrder(host.puzzle.current.editor)`. It **deduplicates** against the last saved serialization for that level, because `select`, `arm`, `disarm` and `escape` also commit without changing placements. It never throws into the edit path: storage errors are already caught in the shell. The `mode === "live"` drag branch in `applyPuzzleEdit` does not save.
- Star write-through: in `recordStars` (`puzzle-run.ts:61-69`), after `starStore.record(...)`, do `const merged = host.puzzle.progress.recordStars(id, best)` and then `starStore.record(id, merged, merged.bestPeakHeldPct)`, which absorbs higher stars from another tab. Then call `setMaybeBestStars`. Both `recordDecision` (the running → ended win) and `recordStop` (Stop after a win) flow through here. This happens inside `dispatchRecording` before `publish`, which satisfies D-11.
- Hydration: in `createPuzzleRuntime(progress)`, for each `PUZZLE_IDS` id whose document record has `win === true`, call `starStore.record(id, {win, brim, thrifty}, bestPeakHeldPct)`.

### Pattern 10: Grid page layout in both shells

The canvas shell is the default on every screen (`screen-shell.ts`: `?shell=form-factor` is the only opt-out), and `html[data-canvas-stage]` sets `overflow: hidden` on `html`, `body`, `#root`, `.app-shell`, `.app-body` and `.playground-main` (`canvas-stage.css:1-22`). The grid page must therefore be its own scroll container. Mirror `.fallback-panel--canvas-stage` (`canvas-stage.css:402-416`): `position: fixed`, the visual-viewport top and height, a top padding of `max(8px, safe-area) + 60px` to clear the fixed "Demos" trigger, plus `align-content: start` and `overflow-y: auto; overscroll-behavior: contain`. In canvas stage `PlaygroundShell` hides `SiteFooter` (`Show when={!props.canvasStage}`), so render `<SiteFooter />` inline at the end of the scrolling grid in canvas stage. The footer holds the GitHub link that the `must` source-link rule requires. In the form-factor shell, the page scrolls normally and the shell footer shows.

### Anti-Patterns to Avoid
- **Restoring in `beginPuzzleRoute`:** no `levelInfo` and no hash exist yet. Restoring there would need a TS mirror of the hash, which violates D-17.
- **Saving from the reducer or `dispatch`:** that makes the pure core impure and saves on every `inspected` or `stepped` event.
- **Copying parsed JSON into `{}` with `obj[key] = value`:** a `"__proto__"` key would re-prototype the object. Use `Object.create(null)` (as `StarStore.snapshot` does) or a `Map`.
- **Writing on read failure:** a "repair" write would destroy corrupt or future data, which D-14 forbids.
- **Adding `data-hud-obstacle` to the grid, the sheet link or the note:** it is reserved for canvas leaves.
- **Reusing `scene_spot::Fnv1a`:** that module is compiled out on wasm32.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Star merge | A new OR/max routine | `mergeLevelStars` | It is the D-09 single merge and already validates the 0–100 range. |
| Placement validation | New per-field checks | Exported `parseSeamPlacement` / `checkPlacement` logic | It is the same untrusted-data contract (safe integers, i32 range, kind allowlist). |
| Spec canonicalization | A hand-written field encoder | `#[derive(Debug)]` text | It covers every field automatically. Verified identical on native and wasm. |
| Storage availability | Feature sniffing (`'localStorage' in window`) | Getter in try/catch, plus the probe `setItem`/`removeItem` | Browsers expose the object yet block or zero-quota it [CITED: MDN]. |
| Unlock and lock display | Ad hoc logic in TSX | Pure `unlockedLevelIds` and `levelCardModels` | Unit-testable, and D-10 requires it to be pure. |

**Key insight:** every hard part (merging, parsing untrusted placements, validation of restored tools) already exists. The phase connects them at two well-defined shell funnels, `commitNow` and `recordStars`, plus one restore point.

## Runtime State Inventory

This is not a rename or refactor phase. It introduces new state, listed here for planning:

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | New `localStorage` key `liquidfun.puzzle-progress.v1` and probe key `liquidfun.puzzle-progress.probe`. Existing keys `liquidfun.puzzle-first-hint.v1`, `liquidfun.hud-sliders.v1` and the appearance keys are untouched. | Code only. No migration (D-13). |
| Live service config | None. Static GitHub Pages site, no backend. | None |
| OS-registered state | None | None |
| Secrets/env vars | None | None |
| Build artifacts | `web/src/generated/liquidfun-wasm/` (gitignored) must be regenerated for the new `levelHash` getter. | Run `just web-wasm` before `build:app` and Playwright. |

## Common Pitfalls

### Pitfall 1: The grid cannot scroll in the default canvas shell
**What goes wrong:** six cards in one column at 390×844 overflow, and `overflow: hidden` on `html`, `body` and `main` clips them. The footer is also hidden.
**How to avoid:** use a fixed, self-scrolling page container with a top inset for the "Demos" trigger (Pattern 10). Show `SiteFooter` inline in canvas stage.
**Warning signs:** the e2e cannot `scrollIntoViewIfNeeded` the sixth card, or `document.documentElement.scrollWidth > innerWidth` (D-07 requires no horizontal scroll).

### Pitfall 2: The test seam is sticky across navigations
**What goes wrong:** `installSolutionPlacements` uses `page.addInitScript`, which runs again on every reload, and it defines a window global that persists across hash navigations. `maybeReadTestPlacements` **throws** "Test placements name another level" when you enter another level in the same page. That throw lands in `failScene`.
**How to avoid:** after solving level 1 with the seam, open level 2 and the grid in `await page.context().newPage()`. Page init scripts are per page, while localStorage is shared per context. Or delete the global with `page.evaluate` before navigating (it is `configurable: true`), but a reload re-adds it. The seam also outranks saved placements (D-18), so restore tests must run on a page **without** the seam.
**Warning signs:** a `failure` view right after Next on a page that used the seam.

### Pitfall 3: Saved placements on a not-yet-solved level create a fake "best"
**What goes wrong:** a record with `win:false` plus placements is hydrated into `StarStore`, and the result card shows a "best" before the first win.
**How to avoid:** hydrate only `win === true` records, and reject `brim || thrifty` without `win` as `shape`.

### Pitfall 4: More than 16 restored placements fail the whole inspection
**What goes wrong:** Rust `check_lanes` rejects more than `MAX_PLACEMENTS = 16` with `TooManyPlacements`. That is a lane-level error, not a per-tool status (`validate.rs:66-70`), so `inspectPlacements` throws and the level fails to load.
**How to avoid:** the restore parser discards saved lists with more than 16 entries. Every other bad value (angle, position, kind not allowed, over budget, valve tick) comes back as a per-tool status and is safe to restore red.

### Pitfall 5: The restore flag leaks into Retry or Try again
**What goes wrong:** Retry (`pendingRun`) or Try again rebuilds and restores stale saved placements over the editor.
**How to avoid:** set the flag only in `beginPuzzleRoute` when the seam is absent. Consume it once inside `buildPuzzle`, guarded by `loading && !pendingRun`. Clear it in `leavePuzzle`. Unit-test with `puzzle-host-fixture.ts`.

### Pitfall 6: The note is missing from the form-factor puzzle layout
**What goes wrong:** `PuzzleOptionsSheet` renders only when `canvasStage` (`PuzzlePanel.tsx:381-395`). The form-factor layout puts the render-mode select in a `.control-row`, so "All levels" and the note would be absent there.
**How to avoid:** render a small `PuzzleLevelLinks` fragment (the All levels link plus an optional note) in both the sheet and the form-factor control row. Putting it inside `PuzzleOptionsSheet` itself also covers `PuzzleFallbackPanel`'s canvas-stage sheet.

### Pitfall 7: The seeded future-version value changes "byte-for-byte"
**What goes wrong:** any write path, such as the probe hitting the main key, a discard write or a star write, touches the progress key.
**How to avoid:** the probe uses a separate key. In degraded mode every write method returns early before `setItem`. In the e2e, read the raw string before and after play and compare with `toBe`.

### Pitfall 8: A route-kind addition breaks exhaustive code
**What goes wrong:** `playableRouteIdentity`'s `assertNever` causes a compile error, which is good. Silent spots are code that branches on `route.kind !== "scene"` to mean "puzzle or unknown".
**How to avoid:** run `bun run typecheck`. Grep `route.kind ===` and `kind === "unknown-puzzle"` in `src/`. Today these appear in `playable-target.ts`, `playable-route.ts`, `PlaygroundStage.tsx` and `runtime.ts`, and only `PlaygroundStage` needs a new branch.

### Pitfall 9: Dedup state goes stale across levels
**What goes wrong:** the "last saved" cache is keyed by serialization only, so entering another level with identical placements (for example both empty) skips its save.
**How to avoid:** key the dedupe cache by level id (or `id + hash`), and reset it on level entry.

## Code Examples

### Example 1: Route parser (D-01)
```typescript
// web/src/routing/playable-route.ts
if (parts[0] === "puzzles" && parts.length === 1) {
  return { kind: "puzzles" };
}
if (parts[0] !== "puzzle") {
  return maybeParseSceneRoute(hash);   // "puzzles/x" → { kind: "unknown", maybeRaw: "puzzles/x" }
}
// normalizePlayableRoute:
if (route.kind === "puzzle" || route.kind === "unknown-puzzle" || route.kind === "puzzles") {
  return { route, maybeReplacementHash: undefined };
}
```

### Example 2: Never-lowering merge and unlock (pure)
```typescript
function mergeStars(maybeA: LevelStars | undefined, maybeB: LevelStars | undefined): LevelStars | undefined {
  if (maybeA === undefined || !maybeA.win) return maybeB?.win ? maybeB : maybeA ?? maybeB;
  if (maybeB === undefined || !maybeB.win) return maybeA;
  return mergeLevelStars(maybeA, maybeB, maybeB.bestPeakHeldPct); // OR per criterion, max %
}

export function unlockedLevelIds(document: ProgressDocument, degraded: boolean): ReadonlySet<PuzzleId> {
  if (degraded) return new Set(PUZZLE_IDS);
  return new Set(PUZZLE_IDS.filter((_, index) => {
    const maybePrevious = index === 0 ? undefined : PUZZLE_IDS[index - 1];
    return maybePrevious === undefined || document.levels[maybePrevious]?.win === true;
  }));
}
```

### Example 3: Fail-closed parse of the top level
```typescript
export function parseProgress(maybeRaw: string | null): ParsedProgress {
  if (maybeRaw === null) return { kind: "ok", document: EMPTY_PROGRESS };
  let value: unknown;
  try { value = JSON.parse(maybeRaw); } catch { return { kind: "unusable", reason: "json" }; }
  if (!isPlainRecord(value)) return { kind: "unusable", reason: "shape" };
  const version = value["version"];
  if (typeof version === "number" && Number.isInteger(version) && version > PROGRESS_VERSION) {
    return { kind: "unusable", reason: "future-version" };
  }
  if (version !== PROGRESS_VERSION || !isPlainRecord(value["levels"])) return { kind: "unusable", reason: "shape" };
  // Object.entries reads own enumerable keys, including a JSON "__proto__" key, safely.
  // Build `levels` and `unknownLevels` with Object.create(null).
  ...
}
```

### Example 4: Playwright seeding, blocked storage and a byte-for-byte check
```typescript
// Blocked storage (reuse e2e/puzzle-hint.spec.ts:145-152)
await page.addInitScript(() => {
  Object.defineProperty(window, "localStorage", {
    configurable: true,
    get() { throw new DOMException("storage blocked", "SecurityError"); },
  });
});

// Seed once, then load the app fresh (avoid addInitScript re-seeding on reload)
await page.goto("/liquidfun-rs/#/puzzles");
await page.evaluate(([key, raw]) => window.localStorage.setItem(key, raw), [PROGRESS_KEY, FUTURE_RAW]);
await page.reload();
// ... play a level to its result card ...
expect(await page.evaluate((key) => window.localStorage.getItem(key), PROGRESS_KEY)).toBe(FUTURE_RAW);

// Mid-session quota failure
await page.evaluate(() => {
  Storage.prototype.setItem = () => { throw new DOMException("full", "QuotaExceededError"); };
});
```

Restore check: read `main[data-puzzle-placements]` (from `puzzle/debug-attributes.ts`) and wait for `waitForPuzzlePhase(page, "building")`. A good positive test solves level 1 with the seam (Play saves the seam placements), then opens `first-pour` in a **new page** without the seam and expects the solution placements.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Old iOS Safari private mode: `setItem` always throws | Safari 11+ private mode: working, session-scoped storage | 2017 [ASSUMED] | The probe still covers locked-down or embedded contexts. |
| Feature detection by property presence | Probe write and remove in try/catch | Long-standing MDN guidance [CITED] | Implemented by D-14. |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Firefox with storage disabled returns `null` from `window.localStorage` | Pattern 4 | Low. The null check and try/catch both lead to degraded mode anyway. |
| A2 | iOS Safari before 11 threw on `setItem` in private mode, and 11+ works | Pattern 4, State of the Art | Low. Covered by the probe. |
| A3 | Rust core float formatting is integer-only, so it is platform-independent | Pattern 5 | Low. The experiment confirmed equality for 2,000+ values, and the wasm replay pin check re-verifies the real specs. |
| A4 | Choosing `primary: "next"` for the last-level All levels link | Pattern 7 | UX choice only. The planner or UI-SPEC may pick `"retry"` instead. |

## Open Questions

1. **Exact document title for `#/puzzles`**
   - What we know: D-02 says `titleForPlayableRoute` "returns 'Puzzles'". Other titles use the form `"<X> · puzzle · liquidfun-rs playground"` or `"<Scene> · liquidfun-rs playground"`.
   - Recommendation: use `"Puzzles · liquidfun-rs playground"` for consistency, unless the bare word was intended. This is a minor point for the planner.
1. **Extra top-level JSON keys in a v1 document**
   - What we know: D-13 preserves unknown **level ids** but says nothing about other top-level keys.
   - Recommendation: preserve them verbatim in a null-prototype `extra` record. Writing them back costs little and keeps the "never destroy data" spirit. Alternatively treat them as `shape`, which would push a v1 document with extra keys into degraded mode. That is too strict.
1. **Immediate versus lazy removal of mismatched placements**
   - Recommendation: remove them immediately through `discardPlacements`, which is a normal merged write. Edge case: an older build open in another tab could delete the newer build's placements. Accept this, because stars are never affected.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| bun | scripts, vitest, build | ✓ | 1.4.2 | — |
| cargo / rustc (pinned) | Rust getter and tests | ✓ | 1.97.0 | — |
| wasm32-unknown-unknown target | `just web-wasm` | ✓ | installed | — |
| wasm-bindgen / wasm-pack CLI | wasm build | ✓ | present in ~/.cargo/bin | — |
| node | ad hoc wasm check | ✓ | 24.13.0 | — |
| Playwright Chromium | e2e | ✓ | several revisions cached; @playwright/test 1.63.0 | `bun run browser:install` |
| just | recipes | ✓ | 1.48.0 | — |
| jq | fingerprint diff | ✓ | 1.7.1 | — |
| Python 3.13 (mdformat) | `just markdown-check` | ✓ | 3.13.12 | Only needed if non-GSD Markdown changes |

**Missing dependencies:** none.

## Validation Architecture

`workflow.nyquist_validation` is `false`. The orchestrator asked for concrete commands anyway, so they follow.

| Check | Command (from repo root unless noted) | Notes |
|-------|----------------------------------------|-------|
| Type check | `cd web && bun run typecheck` | Catches the new route kind and the `levelHash` interface changes |
| Unit (quick) | `cd web && bunx vitest run tests/puzzle-progress.test.ts tests/puzzle-progress-shell.test.ts tests/puzzle-restore.test.ts tests/playable-route.test.ts tests/playable-target.test.ts tests/puzzle-format.test.ts tests/puzzle-level-info.test.ts` | Under 30 s |
| Unit (full) | `cd web && bun run test:unit` | D-21 |
| Rust fmt / lint | `cargo fmt --all --check` and `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings` | D-21 (`just clippy` covers the whole workspace) |
| Rust tests | `cargo test -p liquidfun-wasm level_hash`, then `cargo test -p liquidfun-wasm` | Expect the macOS first-launch scan delay (~2 min) |
| Native == wasm hash | `just puzzle-replay-wasm` (extended to check `level-hashes.txt`) | Also rebuilds wasm |
| wasm + app build | `just web-wasm && (cd web && bun run build:app)` | Playwright serves `vite preview` of `dist` |
| Puzzle e2e | `cd web && bun run test:puzzle` (add `e2e/puzzle-progress.spec.ts` to the script) | D-21 |
| Player smoke | `just web-player-smoke` | Runs wasm, typecheck, unit, build and `test:player` (includes `shell.spec.ts`) |
| Scene fingerprints | Capture the base first (`just playground-scene-spot; S=$(ls -td target/dam-break-perf/*/ \| head -1); jq -r '.scenes[] \| "\(.scene) \(.fingerprint)"' "$S/scene-spot.json" \| sort > target/phase41/base.txt`), then repeat into `after-final.txt` and run `diff` with `wc -l` = 25 | The Phase 40 pattern (40-11-PLAN). `target/phase40/after-final.txt` is valid as a base only if `git log -- crates/` shows no change since `31dec5b1e`, and it currently shows none. |
| Engine untouched | `git diff --stat <phase-base> -- crates/liquidfun` is empty | D-21 |
| File lengths | `bun scripts/bright-builds-check.ts file-lengths` | `levels_tests.rs` is at 620 of 628 |
| Markdown | `just markdown-check` | Only if non-GSD Markdown changed |

### Phase Requirements → Test Map
| Req ID | Behavior | Type | Command / File |
|--------|----------|------|----------------|
| PUZ-01 | `#/puzzles` parses, is not normalized, has an identity and a title | unit | `tests/playable-route.test.ts`, `tests/playable-target.test.ts` |
| PUZ-01 | Grid model: 6 cards, titles, captions, star labels, lock text | unit | `tests/puzzle-progress.test.ts` |
| PUZ-01 | Sidebar link opens the grid; 6 cards; no horizontal scroll at 390 | e2e | `e2e/puzzle-progress.spec.ts` |
| PROG-01 | Parse, serialize round trip; never-lowering merge (including another tab) | unit | `tests/puzzle-progress.test.ts` |
| PROG-01 | Solve, reload, stars shown; a worse run does not lower them | e2e | `e2e/puzzle-progress.spec.ts` |
| PROG-02 | Unlock rule; last-level All levels model | unit | `tests/puzzle-progress.test.ts`, `tests/puzzle-format.test.ts` |
| PROG-02 | Deep link to a locked level opens in building; last-level Next goes to `#/puzzles` | e2e | `e2e/puzzle-progress.spec.ts`, `e2e/puzzle-levels.spec.ts` |
| PROG-03 | Throwing getter, null storage, probe throw, mid-session throw; corrupt, shape and future values mean no write | unit | `tests/puzzle-progress-shell.test.ts` |
| PROG-03 | Blocked storage: all unlocked, note shown, playable; future value byte-identical | e2e | `e2e/puzzle-progress.spec.ts` |
| PROG-04 | Restore precedence, hash mismatch discard, malformed or >16 discard, save dedupe, flag not used by Retry | unit | `tests/puzzle-restore.test.ts` (via `puzzle-host-fixture.ts`) |
| PROG-04 | Level hash pinned; 16 hex digits; distinct; native == wasm | Rust unit + wasm replay | `cargo test -p liquidfun-wasm level_hash`, `just puzzle-replay-wasm` |
| PROG-04 | Reload restores placements; a wrong hash opens empty | e2e | `e2e/puzzle-progress.spec.ts` |

### Wave 0 Gaps
- [ ] `web/tests/puzzle-progress.test.ts`, `puzzle-progress-shell.test.ts` and `puzzle-restore.test.ts` (new)
- [ ] `web/e2e/puzzle-progress.spec.ts` (new), and the `test:puzzle` script updated
- [ ] `crates/liquidfun-wasm/src/puzzle/level-hashes.txt` and the `level_hash.rs` tests (new)
- [ ] Fakes updated with `levelHash`: `tests/puzzle-host-fixture.ts`, `tests/puzzle-level-info.test.ts` and `tests/puzzle-session.test.ts`

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no (locks are soft by design) | — |
| V5 Input Validation | yes | Hand-written fail-closed parsers (`parseProgress`, exported `parseSeamPlacement` checks), kind allowlist, safe-integer and i32 checks, cap of 16, `/^[0-9a-f]{16}$/` for hashes |
| V6 Cryptography | no | FNV-1a is a change detector, not a security hash. Document that. |
| V12/V14 Data protection and config | yes (client storage) | No secrets stored. Unknown data preserved, never executed. |

### Known Threat Patterns (threat_model inputs)

| Pattern | STRIDE | Mitigation |
|---------|--------|------------|
| Tampered or corrupt `localStorage` value (another script, the user, an extension) | Tampering | `JSON.parse` in try/catch. Strict schema at startup leads to degraded mode with no writes. Per-placement checks happen at restore, and lists over 16 are discarded. |
| Prototype pollution via `"__proto__"` / `"constructor"` keys in `levels` | Tampering / EoP | `JSON.parse` creates `__proto__` as an own data property, which is harmless. **Never** assign parsed keys into `{}`. Build `levels` and `unknownLevels` with `Object.create(null)`, read with `Object.entries` or `Object.hasOwn`, and look up known ids only through the `PUZZLE_IDS` allowlist (as `StarStore` does with `KNOWN_LEVEL_IDS`). |
| XSS through stored strings | Tampering | No stored string is rendered. Titles and captions come from TS constants, and the `levelHash` is compared, never displayed. Placements reach the DOM only as the numeric `data-puzzle-placements` attribute through a Solid text binding. Keep `innerHTML` banned (`tests/puzzle-chrome.test.ts:235`) and add the new component to its scan list. |
| A crafted placement list that crashes or hangs the page | DoS | Cap the length at 16 (`MAX_PLACEMENTS`). Integer and range checks run before `encodePlacementLanes`. The Rust inspector returns per-tool statuses for everything else. |
| A future or corrupt document overwritten by this build | Tampering (integrity of user data) | Degraded mode never writes, and the probe uses a separate key (D-14). The e2e byte-for-byte check covers it. |
| Quota exhaustion | DoS | The document is small (6 levels × ≤16 tools). `setItem` throwing leads to degraded mode, and the session continues in memory. |
| Hash collision lets stale placements restore | Tampering (benign) | Probability is negligible at 64 bits. Even so, restored tools are re-validated by the Rust inspector, and Play is gated. |

## Sources

### Primary (HIGH confidence)
- Codebase, read in this session: `web/src/routing/{hash,playable-route}.ts`, `web/src/player/{playable-target,scene-lifecycle,playground-session,puzzle-runtime,puzzle-run,puzzle-lifecycle,puzzle-editing,puzzle-host,first-hint-preference,hud-sliders-preference,fullscreen-capability,screen-shell}.ts`, `web/src/components/{PlaygroundStage,PlaygroundShell,PuzzlePanel,PuzzleOptionsDrawer,PuzzleFallbackPanel,PuzzleResultCard,FallbackPanel,SiteHeader,DemoNavigation}.tsx`, `web/src/puzzle/{reduce-puzzle,editor,edit-event,placements,star-store,stars,format}.ts`, `web/src/physics/puzzle-session.ts`, `web/src/styles/canvas-stage.css`, `crates/liquidfun-wasm/src/puzzle{.rs,/spec.rs,/info.rs,/tools.rs,/validate.rs}`, `crates/liquidfun-wasm/src/scene_spot/fingerprint.rs`, `web/e2e/{puzzle-helpers,puzzle-hint.spec,puzzle.spec,puzzle-levels.spec,shell.spec,canvas-stage.spec,puzzle-editor.spec}.ts`, `web/tests/{playable-route,puzzle-placements,puzzle-chrome,puzzle-first-hint}.test.ts`, `web/package.json`, `web/playwright.config.ts`, `justfile`, `scripts/web-build.ts`, and the Phase 40 11-PLAN fingerprint gate.
- Local experiment (scratchpad crate `f32dbg`, Rust 1.97.0): the `{:?}` FNV hash was identical on aarch64-apple-darwin and on wasm32-unknown-unknown run in node.
- [Rust std `Debug` docs](https://doc.rust-lang.org/std/fmt/trait.Debug.html): derived and std `Debug` formats are not stable across Rust versions.

### Secondary (MEDIUM confidence)
- [MDN Window.localStorage](https://developer.mozilla.org/en-US/docs/Web/API/Window/localStorage): `SecurityError` cases.
- [MDN Storage.setItem](https://developer.mozilla.org/en-US/docs/Web/API/Storage/setItem): `QuotaExceededError`.
- [MDN Using the Web Storage API, "Testing for availability"](https://developer.mozilla.org/en-US/docs/Web/API/Web_Storage_API/Using_the_Web_Storage_API): probe pattern and zero-quota private modes.

### Tertiary (LOW confidence)
- Browser-specific details: Firefox null storage, and iOS Safari private mode before and after version 11. Both are [ASSUMED], and the design does not depend on them.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH. No new dependencies; versions read from manifests.
- Architecture and touch points: HIGH. Every file and line was read, and the restore timing was traced through `beginPuzzleRoute` and `buildPuzzle`.
- Level hash portability: HIGH for the same toolchain (experiment). The cross-version instability is documented and covered by the pin test.
- Pitfalls: HIGH for the codebase-derived ones (canvas-shell overflow, sticky seam, the 16-placement cap, the form-factor sheet). MEDIUM for browser storage specifics.

**Research date:** 2026-10-13
**Valid until:** 2026-11-12 (stable codebase; re-check if Phase 42 changes `PuzzlePanel` or the options sheet first)
