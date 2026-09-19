# Phase 19: Interaction Polish and Browser Verification - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-19T01:45:03.493Z
**Phase:** 19-interaction-polish-and-browser-verification
**Mode:** Yolo
**Areas discussed:** Canvas pointer gestures, Stuck-pointer and page-scroll isolation, Keyboard focus and instruction copy, Focused six-scene browser evidence

---

[auto-select] Selected all gray areas: Canvas pointer gestures, Stuck-pointer and page-scroll isolation, Keyboard focus and instruction copy, Focused six-scene browser evidence.
[auto-select] No pending todos matched Phase 19 (`todo_count: 0`).
[auto-select] Advisor mode off — no USER-PROFILE.md.

## Canvas pointer gestures

| Option | Description | Selected |
|--------|-------------|----------|
| Shared Pointer Events plus one gesture per scene, keep labeled controls | Canvas capture → camera world coords → WASM `pointer_action`; existing buttons remain the keyboard path | ✓ |
| Labeled buttons only | Treat Phase 18 actions as WEB-05 completion and skip canvas handlers | |
| Full canvas editor (pan/zoom/multi-touch) | Camera manipulation and multi-object picking like the desktop testbed | |

**User's choice:** Shared Pointer Events plus one gesture per scene, keep labeled controls (recommended default)
**Notes:** [auto] Matches FEATURES.md “at least one meaningful interaction per scene” and Phase 18 D-06 deferral of pointer aiming/stirring/poking. Buttons stay for WEB-07 keyboard. Locked mapping: Dam Break obstacle, Fountain aim, Float or Sink click-to-drop, Color Mixer stir, Jelly Drop poke, Water Wheel jet.

---

## Stuck-pointer and page-scroll isolation

| Option | Description | Selected |
|--------|-------------|----------|
| Capture on canvas down; clear on up/cancel/lost; preventDefault only while captured | `touch-action: none` on canvas only; page scroll stays native; resize updates CSS→world transform without rebuilding the world | ✓ |
| `touch-action: none` on the whole player/page | Prevents accidental scroll everywhere, including catalog and footer | |
| No pointer capture, rely on element mouseup | Simpler, but leaves stuck drags when the pointer leaves the canvas or a touch is cancelled | |

**User's choice:** Capture on canvas down; clear on up/cancel/lost; preventDefault only while captured (recommended default)
**Notes:** [auto] Follows PITFALLS.md resize/high-DPI guidance and WEB-05 “ordinary scrolling outside the player.” Pause/reset/scene-change/cleanup must drop in-flight gestures.

---

## Keyboard focus and instruction copy

| Option | Description | Selected |
|--------|-------------|----------|
| Native control keyboard plus per-scene instruction copy; keep D-09 CSS | No custom Space-to-pause; 44px targets and existing focus ring; instruction line beside canvas; polish wrap at ≤480px | ✓ |
| Add global keyboard shortcuts (Space pause, arrow aim) | Faster for power users; conflicts with 18-UI-SPEC native Enter/Space and risks traps | |
| Adopt MysticUI/Tailwind for the polish pass | Would replace the locked semantic HTML + scoped CSS exception before the 2026-12-17 review | |

**User's choice:** Native control keyboard plus per-scene instruction copy; keep D-09 CSS (recommended default)
**Notes:** [auto] WEB-07 asks for labeled keyboard-operable controls, visible focus, contrast, and concise text instructions. 18-UI-SPEC already forbids custom Enter/Space and a hamburger.

---

## Focused six-scene browser evidence

| Option | Description | Selected |
|--------|-------------|----------|
| Chromium Playwright on production-base dist plus targeted live Pages | All six scenes: stepping, playback/reset, one pointer + one labeled control, switch cleanup, hidden-tab, 375px a11y; record live URL/SHA | ✓ |
| Broad browser/native matrix | Firefox, Safari, WebKit, plus Linux native qualification | |
| Local unit tests only | Skip real-browser WASM smoke and hosted URL refresh | |

**User's choice:** Chromium Playwright on production-base dist plus targeted live Pages (recommended default)
**Notes:** [auto] WEBTEST-01 explicitly asks for focused smoke, not a broad matrix. Extend `web/e2e/player.spec.ts`. Independent AI review remains required.

---

## Claude's Discretion

- Exact pointer API naming, force magnitudes, Dam Break drag vs click-to-reposition, Water Wheel aim vs strength, instruction markup, and Playwright file split.

## Deferred Ideas

- MysticUI/Tailwind/shadcn adoption until 2026-12-17
- Camera pan/zoom, pinch, scene editor
- Firefox/Safari/WebKit matrices, screenshot-hash oracles, WASM workers/WebGPU, share-link seeds
