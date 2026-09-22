# Phase 26: Catalog shell and basin scenes - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-22T00:02:54.967Z
**Phase:** 26-Catalog shell and basin scenes
**Mode:** Yolo
**Areas discussed:** Catalog shell, Watch-first controls, Pinned-test credits, Recognizable basin layouts, Shared session and proof

---

## Catalog shell

| Option | Description | Selected |
|--------|-------------|----------|
| Append two ready entries to the existing sidebar and drawer | Keep the six scenes first; hash routes `#/scene/particles` and `#/scene/liquid-timer`; static SVG previews captioned `Static preview` | ✓ |
| New "Testbed ports" catalog page | Splits the player from the Phase 20 shell | |
| Replace or reorder the original six | Drops scenes this milestone must keep | |

**User's choice:** Yolo recommended default — append Particles and Liquid Timer to the existing `DemoNavigation` shell.
**Notes:** No `.catalog-card` grid, search, or filters. Carries Phase 18 D-01/D-02 and Phase 20 D-01 through D-06.

---

## Watch-first controls

| Option | Description | Selected |
|--------|-------------|----------|
| Play, pause, and Reset only | Matches BASIN success criteria and leaves PRESET-01 for later | ✓ |
| Dam Break-style water and gravity presets | Extra chrome the pinned tests do not require for recognition | |
| Full particle-type picker | PRESET-01 / C++ parameter bar; out of this phase | |

**User's choice:** Yolo recommended default — watch-first play, pause, and Reset.
**Notes:** Reset recreates the documented initial layout. No pointer or keyboard material modes.

---

## Pinned-test credits

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 18 chrome plus pinned JS and C++ test links | Implementation stays in this repo; inspiration cites commit `7f20402173fd143a3988c921bc384459c6a858f2` | ✓ |
| JS file only | Omits the C++ cross-check the research uses | |
| Showcase homepage only | Too vague for PLAY-03 | |

**User's choice:** Yolo recommended default — credit `testParticles.js` / `Particles.h` and `testLiquidTimer.js` / `LiquidTimer.h` at the pinned commit.
**Notes:** Copy stays an experimental recognizable port. No sealed-parity claim.

---

## Recognizable basin layouts

| Option | Description | Selected |
|--------|-------------|----------|
| Close recognizable ports of the pinned tests | Open basin plus falling water and ball; tensile/viscous drain through shelves into four columns | ✓ |
| Smaller stylized toys | Cuts particle counts to look smoother; forbidden by requirements | |
| Sealed C++ differential gate | PARITY-01; deferred | |

**User's choice:** Yolo recommended default — recognizable layouts from `.planning/research/FEATURES.md`, native `liquidfun` only.
**Notes:** Engine work only if a scene cannot run without it. Catch-up cap stays 4.

---

## Shared session and proof

| Option | Description | Selected |
|--------|-------------|----------|
| Extend the existing scene factory and Chromium smoke | One WASM world; six scenes remain; smoke covers open/play/pause/reset for the two new scenes | ✓ |
| Separate player route | Forks the shared player this phase is meant to grow | |
| Desktop testbed only | Does not satisfy the visitor-facing playground goal | |

**User's choice:** Yolo recommended default — grow `SceneId` and `just web-player-smoke`.
**Notes:** Independent AI review required. Implementing agent does not self-approve.

---

## Claude's Discretion

- Exact basin coordinates, radii, and particle counts when the result stays recognizable.
- Static SVG artwork.
- Whether tensile/viscous drain needs a narrow engine addition.
- Scene-module file splits near the file-length trigger.

## Deferred Ideas

- Phases 27–29 scenes and PLAY-01 full catalog.
- PRESET-01, DRAW-02, PARITY-01, BOX2D-01.
- Optional pointer nudge and cross-links between related scenes.
