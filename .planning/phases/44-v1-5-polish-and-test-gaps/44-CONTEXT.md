---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 44-2026-10-14T01-34-05
generated_at: 2026-10-14T01:34:05.642Z
---

# Phase 44: v1.5 Polish and Test Gaps - Context

**Gathered:** 2026-10-14
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

This phase cleans up three areas of tech debt that the owner chose from the v1.5 milestone audit (`.planning/v1.5-MILESTONE-AUDIT.md`):

1. **Phone tray and valve accessibility:**
   - Tool names must no longer truncate in the puzzle tool tray on phones.
   - The valve opening-time readout must stop being hidden from assistive technology.
   - The TypeScript valve tick must be capped at 255, matching the Rust `u8` lane.
2. **E2E coverage:**
   - Add a test that clicks Next from a won level to the next level.
   - Add a test that runs the timed-gate valve level at 2× and 4× and compares it with 1×.
3. **E2E typecheck:**
   - Bring `web/playwright.config.ts` and `web/e2e/**` into the TypeScript typecheck.
   - Fix every error that this exposes.

Out of scope:

- new features, levels or tools;
- Rust or engine changes;
- the wider sweep of info-level review notes;
- re-authoring the first-pour frame;
- the Water probe retry, which would conflict with UI-D50.

</domain>

<decisions>
## Implementation Decisions

### Phone tool tray (success criterion 1)

- **D-01:** The tray never shows an ellipsized tool name at portrait phone widths. At 390×844 and at 360×780, every visible `.puzzle-tray-name` must be fully shown. In practice that means `scrollWidth <= clientWidth` for each name and no `…`.
- **D-02:** Keep the tray's HUD contract:
  - the fixed `data-hud-obstacle` box, whose portrait width is `calc(100% - insets - 156px)`;
  - the 44 px chip height;
  - chips of equal width;
  - 44 px minimum touch targets.

  Gain the room inside each chip instead, by changing the chip's layout when two kinds are shown in portrait (Claude's discretion). Options include stacking the count under the name, tightening the icon and text gaps, or using a slightly smaller label size. A label must not go below 12 px, matching the existing smallest UI size. The UI-SPEC portrait fit check (`39-UI-SPEC.md` line 209) must still hold.
- **D-03:** Landscape and non-stage layouts stay as they are. Landscape shows icon plus count when two kinds are present, and the full name stays in the accessible name and title. No new copy is added. The accessible names "Plank, 2 left" and "Valve, 1 left" are unchanged.

### Valve accessibility and tick parity (success criterion 2)

- **D-04:** Remove `aria-hidden="true"` from the valve time readout (`PuzzleSelectionBar.tsx` lines 72-78). Make it the stepper group's accessible description through `aria-describedby`, so screen readers read the current opening time with the group ("Valve opening time", "Opens at 2.5 s").
  - Do not add an `aria-live` region. Changes are already announced through `editor-announce.ts` ("Valve opens at 2.5 s"), so a live region would announce them twice.
- **D-05:** Cap the TypeScript maximum tick (`valve-format.ts` lines 13-15) with `Math.min(Math.floor(stepLimit / 30), 255)`, matching Rust's `u8::try_from(...).unwrap_or(u8::MAX)` in `spec.rs` lines 148-149. Add a unit test with a step limit above 7,650 steps.
- **D-06 (folded, same valve-lane parity):** Fix review finding 40-REVIEW IN-06. When the test seam's `readSeamOpenTick` (`placements.ts` lines 95-98) gets a valve with no `openTick`, it must default with `defaultOpenTick` for that kind instead of 0. Planks keep 0. Add a unit test.

### E2E coverage (success criterion 3)

- **D-07 (Next navigation):**
  1. Win first-pour quickly with the existing seam pattern (`solveFirstPour`: 15 Hz clock, `installSolutionPlacements`, 4×).
  2. **Delete the test-placements seam global** before clicking Next. Otherwise `maybeReadTestPlacements` throws "Test placements name another level" when the route enters the next level.
  3. Click the result card's "Next level" link.
  4. Assert all of the following:
     - the URL hash is `#/puzzle/thick-syrup`;
     - `main[data-puzzle="thick-syrup"]` and `data-puzzle-phase="building"` are set;
     - `data-puzzle-placements` is empty;
     - `#player-title` matches that level's title;
     - the canvas is not on the fallback.

  Put the test next to the existing progress and level specs (Claude's discretion on the file).
- **D-08 (valve fast-forward):** Reuse the `puzzle-valve.spec.ts` build of the timed-gate reference through the tray, canvas and stepper.
  1. Play at 1× and record the run fingerprint (`readRunFingerprint`).
  2. Retry at 2× and then at 4×, following the `puzzle-editor.spec.ts` lines 432-459 pattern.
  3. Assert that every run gives `outcome-step` 429 and `peak-held` 582, and that the fingerprints are equal.

  Keep the test under the existing valve spec's runtime budget, for example by sharing the build across the three runs.

### E2E typecheck (success criterion 4)

- **D-09:** Add `web/tsconfig.e2e.json`. It extends the strict base, including `exactOptionalPropertyTypes` and `noUncheckedIndexedAccess`, includes `e2e/**/*.ts` and `playwright.config.ts`, and adds Node and `@playwright/test` types as needed. Change the `typecheck` script to run both projects, so the existing gate (`bun run typecheck`, `scripts/web-build.ts:292`) covers e2e:

  ```
  tsc --noEmit && tsc --noEmit -p tsconfig.e2e.json
  ```

- **D-10:** Fix all 16 errors at the root. Do not loosen compiler flags and do not use `any` or `@ts-ignore`.

  | File | Line(s) | Error |
  | --- | --- | --- |
  | `playwright.config.ts` | 26 | the reporter tuple widens |
  | `player-helpers.ts` | 160, 168, 169 | needs canvas narrowing |
  | `canvas-stage.spec.ts` | 237 | implicit `any` |
  | `live-worker.spec.ts` | 38, 41, 55, 57, 73, 76 | `window.teslaBenchmark` is untyped |
  | `water-helpers.ts` | 62 | `getExtension` argument |
  | `water.spec.ts` | 451 | viewport literal type |
  | `worker-quality-fixture.ts` | 47 | Http2 server union |

  For `teslaBenchmark`, add a typed global declaration in an e2e-only `.d.ts`.

### Verification

- **D-11:** Behavior must not change:
  - `crates/` is untouched, so there is no Rust diff. Check this with `git diff --quiet <phase-base> -- crates/`, so no scene-spot rerun is needed.
  - Puzzle replays stay valid.
  - `bun run typecheck` passes with both projects.
  - `npx vitest run` passes.
  - The full `test:puzzle`, `test:water` and `hud-safe-frame` e2e suites pass after a fresh `build:app`.
  - `bun scripts/bright-builds-check.ts all` passes.
- **D-12:** No human UAT is needed. Every criterion is objectively checkable, and a 390×844 screenshot of grand-finale is saved as evidence under `target/phase44/`.

### Claude's Discretion

- The exact chip layout technique (D-02), the e2e file placement (D-07), and how the valve fast-forward test shares its build (D-08).
- The plan split. Keep it to about 2 or 3 small plans, in line with the owner's preference for a light process.

### Folded Todos

- 40-REVIEW IN-06 (seam `openTick` default), as D-06.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Scope and audit

- `.planning/ROADMAP.md` § Phase 44: goal and four success criteria.
- `.planning/v1.5-MILESTONE-AUDIT.md`: tech-debt items and integration findings F2 and F3.
- `.planning/phases/40-timed-valve-and-teaching-levels/deferred-items.md` line 7: tray truncation evidence.
- `.planning/phases/40-timed-valve-and-teaching-levels/40-REVIEW.md`: IN-02 (TS tick cap), IN-05 (`aria-hidden` valve time) and IN-06 (seam `openTick`).

### UI contracts

- `.planning/phases/39-place-then-play-tools-and-stars/39-UI-SPEC.md`: tray box sizes (lines 78-79, 184, 204), the portrait fit check (209), the equal-width item rule (222) and tray copy (338).
- `.planning/phases/41-level-select-and-progress/41-UI-SPEC.md` line 356: earlier deferral of the tray truncation.
- `AGENTS.md`: the playground canvas framing and HUD obstacle rules.

### Code

- `web/src/components/PuzzleToolTray.tsx`, `web/src/puzzle/editor-format.ts` (`trayItems`), `web/src/styles/puzzle-editor.css` (lines 62-73, 123-150, 247-265, 333-365).
- `web/src/components/PuzzleSelectionBar.tsx` (lines 64-86), `web/src/puzzle/hud-format.ts` (`valveStepperModel`, lines 115-131), `web/src/puzzle/editor-announce.ts` (lines 113-125).
- `web/src/puzzle/valve-format.ts`, `web/src/puzzle/placements.ts` (lines 95-98 and 163-165), `crates/liquidfun-wasm/src/puzzle/spec.rs` (lines 148-149, read-only).
- `web/e2e/puzzle-valve.spec.ts`, `web/e2e/puzzle-progress.spec.ts` (`solveFirstPour`, lines 189-195), `web/e2e/puzzle-levels.spec.ts`, `web/e2e/puzzle-editor.spec.ts` (lines 432-459), `web/e2e/puzzle-helpers.ts`, `web/e2e/puzzle-editor-helpers.ts`.
- `web/tsconfig.json`, `web/package.json` (`typecheck`), `web/scripts/web-build.ts` (line 292).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `solveFirstPour` and `installSolutionPlacements` win quickly. The seam global is `configurable: true`, so it can be deleted.
- `readRunFingerprint` and `waitForHeld` compare runs. A speed-cycling click pattern already exists in `puzzle-levels.spec.ts` lines 51-57.
- `PHONE_PORTRAIT` and `DESKTOP` viewports are defined, and `puzzle-levels.spec.ts` lines 66-90 already open every level at 390×844.

### Established Patterns

- Tests check DOM data attributes (`data-puzzle-*`), not pixels.
- HUD leaves keep fixed boxes that do not depend on the camera.
- Plans in this repo use explicit `git add` and plain `git commit` with the Co-Authored-By trailer, and keep `.planning/config.json` unstaged.

### Integration Points

- `bun run typecheck` is the single typecheck entry point used by the gate scripts.

</code_context>

<specifics>
## Specific Ideas

- The grand-finale screenshot at 390×844 (`target/phase40/screens/level-grand-finale-390x844.png`) shows "Pl… ×2" and "Va… ×1". After this phase it should read "Plank ×2" and "Valve ×1", or show the counts stacked under the full names.

</specifics>

<deferred>
## Deferred Ideas

- The broader sweep of the remaining info-level review notes from phases 37–41, such as parser message parity, lost-step validation and the cross-tab storage listener.
- Re-authoring first-pour's portrait frame, which currently fills about 34% of the height.
- Retrying a failed Water probe once, which conflicts with UI-D50.

### Reviewed Todos (not folded)

- None pending in STATE.md.

</deferred>

---

*Phase: 44-v1-5-polish-and-test-gaps*
*Context gathered: 2026-10-14*
