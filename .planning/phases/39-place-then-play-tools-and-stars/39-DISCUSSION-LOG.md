# Phase 39: Place-then-Play Tools and Stars - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-10-12
**Phase:** 39-place-then-play-tools-and-stars
**Mode:** Yolo (recommended answers auto-selected; no interactive questions)
**Areas discussed:** Validator parity, Brim and run decision, Star store, Editor gestures and rotation, Tool set and first level, Fast-forward, Keyboard, Tests

---

## Validator parity

| Option | Description | Selected |
|--------|-------------|----------|
| Stateless Rust inspector called from TS on each edit | One source of truth; also returns baked outlines for drawing and hit-testing | ✓ |
| TS geometry mirror plus a parity test | No wasm call while dragging, but two implementations to keep aligned | |
| Rust check only on Play | Simplest, but no live red feedback (fails TOOL-03) | |

**User's choice:** [auto] Stateless Rust inspector (recommended default)
**Notes:** Pure polygon math on a handful of shapes is cheap per animation frame. Research allowed a TS mirror only with a parity test.

---

## Brim and when a won run is decided

| Option | Description | Selected |
|--------|-------------|----------|
| Peak held fill in Rust; a won run is decided at the Brim latch or the step limit | One quantity behind Win, Brim and the card percentage; matches the research "run ends at Brim or timer" | ✓ |
| Judge Brim inside the Phase 38 120-step tail after the win | Keeps Phase 38 timing, but splits the rule between TS and Rust | |
| Brim is the fill at the win latch | Simple, but rewards early splashes and ignores "held" | |

**User's choice:** [auto] Peak held fill, decided at Brim or the step limit (recommended default)
**Notes:** This intentionally changes won-run end timing from Phase 38 (D-10). Fast-forward offsets the extra wait.

---

## Star store across runs

| Option | Description | Selected |
|--------|-------------|----------|
| Per-criterion best-of, kept in memory for the page session; Phase 41 persists it | Meets STAR-02 now with no storage scope creep | ✓ |
| localStorage now | Overlaps Phase 41 (PROG-01, PROG-03 storage-blocked rules) | |

**User's choice:** [auto] In-memory best-of (recommended default)

---

## Editor gestures and rotation

| Option | Description | Selected |
|--------|-------------|----------|
| Tap-to-arm then tap the canvas; drag to move; selection-bar rotate and remove buttons | Works the same for mouse and touch; 44 px fixed HUD buttons; keyboard maps directly | ✓ |
| On-canvas rotate handle | Small at phone scale; adds hit-testing near the tool | |
| Two-finger twist | Conflicts with gesture rules; research recommends against it for the MVP | |

**User's choice:** [auto] Tap-to-arm plus a selection bar (recommended default)
**Notes:** Invalid tools stay in place, tinted red, and Play is disabled. They do not snap back, which matches TOOL-03 wording.

---

## Tool set and first level

| Option | Description | Selected |
|--------|-------------|----------|
| Plank only; data-driven tray; first-pour budget 2 planks with par 1 | Makes Thrifty meaningful with no new physics; geometry and fixtures unchanged | ✓ |
| Add short plank and pin kinds now | Scope belongs with the Phase 40 levels that need them | |
| Keep budget 1 with par 1 | Thrifty would always equal Win | |

**User's choice:** [auto] Plank only, budget 2, par 1 (recommended default)

---

## Fast-forward

| Option | Description | Selected |
|--------|-------------|----------|
| More fixed steps per frame (1×/2×/4×), keeping `MAX_ADVANCE_STEPS` = 4 | Outcome is unchanged by construction | ✓ |
| Change the timestep | Would change physics outcomes | |

**User's choice:** [auto] More fixed steps per frame (recommended default)

---

## Keyboard

| Option | Description | Selected |
|--------|-------------|----------|
| Arrows nudge (Shift ×10), R/Shift+R rotate, Delete/Backspace remove, Space Play/Stop, Esc deselect, Enter on a tray item places | Covers TOOL-06 plus selection without a pointer | ✓ |

**User's choice:** [auto] Recommended key map

---

## Tests

| Option | Description | Selected |
|--------|-------------|----------|
| Edit-history fingerprint at reducer, Rust and e2e levels; synthetic-clock win e2e; a hasTouch project; a 1×/2×/4× equality e2e | Covers every Phase 39 success criterion | ✓ |

**User's choice:** [auto] Recommended test set

## Claude's Discretion

- Module and file split, inspector naming and lane layout, the exact Brim threshold (within D-07), tray and selection-bar placement and styling, drag throttling, and the selection-cycling key.

## Deferred Ideas

- Drag-from-tray, two-finger twist, hint ladder, trail ghost, extra tool kinds (Phase 40), persistence (Phase 41), timed valve (Phase 40).
