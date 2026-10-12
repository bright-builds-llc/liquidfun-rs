# Phase 38: Puzzle Route and Player Shell - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md. This log preserves the alternatives considered.

**Date:** 2026-10-12
**Phase:** 38-puzzle-route-and-player-shell
**Mode:** Yolo. All gray areas were auto-selected, and the recommended answers were accepted without prompts. Advisor research agents were skipped because the yolo recommendation engine resolves every area in a single pass and the repo's lightweight-process preference applies.
**Areas discussed:** Routing and lifecycle, Puzzle phase machine, Timer, Fill meter and glass cup, Result card, Interactions/chrome/camera, Bounds registry, Win-path test seam

---

## Routing and lifecycle

| Option | Description | Selected |
|--------|-------------|----------|
| Wrapper parser around the unchanged scene parser | `maybeParsePlayableRoute` recognizes `puzzle/<id>` and delegates the rest; existing hash tests stay untouched | ✓ |
| Extend `maybeParseSceneRoute` | Fewer functions, but `tests/hash.test.ts` asserts other paths are unknown, so it would need edits (violates SC-5) | |
| Adopt `@solidjs/router` | Rejected by milestone research | |

**User's choice:** [auto] Wrapper parser plus a `PlayableTarget` union, with the scene arm unchanged and the direct backend for puzzles.
**Notes:** `#/puzzles` is deferred to Phase 41.

## Puzzle phase machine

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal `reducePuzzle`: loading/building/running/ended with Play, Retry, Edit | Enough for the timer-only-during-run and camera-across-states criteria; Phase 39 extends it | ✓ |
| Auto-run on load | No building state, so the camera check across build/run is impossible | |
| Full Stop/Reset/Clear now | Belongs to TOOL-04 in Phase 39 | |

**User's choice:** [auto] Minimal reducer, a 120-step settle tail after the latch, then hold.

## Timer

| Option | Description | Selected |
|--------|-------------|----------|
| Countdown of remaining simulation time `0:12.3` | Matches "Out of time" framing; step-based | ✓ |
| Count-up elapsed | Less clear sense of the limit | |
| Progress bar only | Loses a precise readout | |

**User's choice:** [auto] Countdown labeled as simulation time; percentages are floored in the loss message.

## Fill meter and glass cup

| Option | Description | Selected |
|--------|-------------|----------|
| HUD fill chip in a fixed slot, plus a canvas glass cup with an etched target line and ticks | Satisfies the HUD-obstacle-leaf criterion and PUZ-05 | ✓ |
| World-anchored meter beside the cup | Would move with the camera and is not a fixed HUD slot | |
| Canvas-only meter | Not a `data-hud-obstacle` leaf | |

**User's choice:** [auto] Split readout. The tick list is data-driven (Win target plus brim for now; ★ Brim comes in Phase 39).

## Result card

| Option | Description | Selected |
|--------|-------------|----------|
| Fixed slot always mounted; content swaps | No obstacle set change, so no camera refit | ✓ |
| Modal overlay | Blocks the view and changes obstacles | |
| Card mounts only at the end | Triggers a safe-frame refit (pitfall) | |

**User's choice:** [auto] Fixed slot. Next is disabled with "No next level yet" while only one level exists.

## Interactions, chrome and camera

| Option | Description | Selected |
|--------|-------------|----------|
| Do not attach or send pointer, tilt, gravity, zoom or pan, and hide scene-only chrome; keep the legend and render mode | Matches PUZ-03 and the research pitfalls | ✓ |
| Attach but ignore in Rust | Rust already rejects these, but TS would still re-apply tilt and show controls | |

**User's choice:** [auto] TS-side off, plus an e2e check of frame and scale equality across building, running and ended on desktop and phone.

## Bounds registry

| Option | Description | Selected |
|--------|-------------|----------|
| Separate `puzzle-bounds.ts` (landscape and portrait) plus its own fill test | Keeps scene tests unedited (SC-4 and SC-5) | ✓ |
| Add levels to `PORTRAIT_VIEW_BOUNDS` | Couples levels to the scene catalog | |
| Derive bounds at runtime from `levelInfo()` | No frame before the session loads; harder to test | |

**User's choice:** [auto] Separate registry with `worldBoundsForTarget` dispatch.

## Win-path test seam

| Option | Description | Selected |
|--------|-------------|----------|
| Test-only window global with recorded fixture placements, plus Vitest fakes | Exercises the real win card without user-facing URL params | ✓ |
| URL query param for placements | User-reachable; overlaps SEED-002 share links | |
| Vitest only | No browser proof of the win card | |

**User's choice:** [auto] Window-global seam plus Vitest fakes; the natural run asserts the loss text.

## Claude's Discretion

- Component and file split, HUD slot placement per layout, glass styling, single versus double session for the build frame, the exact `first-pour` rectangles, and Play button styling.

## Deferred Ideas

- `#/puzzles` grid (Phase 41); Stop/Reset/Clear, tools and stars (Phase 39); fast-forward (Phase 39); worker backend for puzzles (only if needed); the `EXT_color_buffer_half_float` fix (separate quick task).
