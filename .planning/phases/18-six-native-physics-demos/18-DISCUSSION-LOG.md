# Phase 18: Six Native Physics Demos - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-18T03:37:51.969Z
**Phase:** 18-Six Native Physics Demos
**Mode:** Yolo
**Areas discussed:** Catalog cards and previews, Per-scene control surface, Scene authorship and physics honesty, Source credits and notices, Uncertain-scene order, Design system and verification

[auto-select] Selected all gray areas: Catalog cards and previews, Per-scene control surface, Scene authorship and physics honesty, Source credits and notices, Uncertain-scene order, Design system and verification.

---

## Catalog cards and previews

| Option | Description | Selected |
|--------|-------------|----------|
| Six-card catalog with static previews | Title, short description, static SVG/CSS preview, one Open action per approved scene | ✓ |
| Keep Phase 17 thin name list | Add descriptions later; cards wait for Phase 19 | |
| Live WASM thumbnails | Start a world per card or capture Canvas snapshots | |

**User's choice:** Six-card catalog with static previews (recommended default)
**Notes:** [auto] Catalog cards and previews — Q: "How should visitors browse the six scenes?" → Selected: "Six-card catalog with static previews" (recommended default). WEB-01 requires names, descriptions, and previews; FEATURES.md and ARCHITECTURE.md reject live worlds per card.

[auto] Catalog cards and previews — Q: "When do scenes become ready?" → Selected: "Mark each approved scene ready once it has a working native implementation" (recommended default).

---

## Per-scene control surface

| Option | Description | Selected |
|--------|-------------|----------|
| Two or three labeled bounded controls | Discrete presets and constrained ranges; construction changes reset with explicit copy | ✓ |
| Shared play/pause/reset only | Defer all scene parameters | |
| Unbounded sliders and live editors | Continuous unbound values and a mini scene editor | |

**User's choice:** Two or three labeled bounded controls (recommended default)
**Notes:** [auto] Per-scene control surface — Q: "How should each demo be tunable?" → Selected: "Two or three labeled bounded controls" (recommended default). Control set locked from FEATURES.md. Pointer aiming/stirring/poking polish deferred to Phase 19.

[auto] Per-scene control surface — Q: "When does a control reset the scene?" → Selected: "Construction presets reset; live-safe runtime controls may update in place and must say so if they cannot" (recommended default).

---

## Scene authorship and physics honesty

| Option | Description | Selected |
|--------|-------------|----------|
| Native Rust scenes in liquidfun-wasm | Public engine API, one-session factory, no fake physics | ✓ |
| Replay diagnostic catalog recipes | Drive the playground from test-protocol scenarios | |
| JavaScript-authored motion | Animate positions/colors/wheel rotation in the frontend | |

**User's choice:** Native Rust scenes in liquidfun-wasm (recommended default)
**Notes:** [auto] Scene authorship and physics honesty — Q: "Where should the six demos be implemented?" → Selected: "Native Rust scenes in liquidfun-wasm" (recommended default).

[auto] Scene authorship and physics honesty — Q: "What if Float or Sink, Jelly Drop, or Water Wheel fail visually?" → Selected: "Investigate or record an explicit scope decision; never fake physics or silent substitution" (recommended default).

---

## Source credits and notices

| Option | Description | Selected |
|--------|-------------|----------|
| Per-scene implementation, inspiration, and notices | Player chrome links this repo's scene source plus inspiration; footer stays site-level | ✓ |
| Footer-only credits | Rely on Phase 17 repository chrome | |
| Present upstream C++ as the running implementation | Link Google LiquidFun sources as if they execute in the browser | |

**User's choice:** Per-scene implementation, inspiration, and notices (recommended default)
**Notes:** [auto] Source credits and notices — Q: "How should WEB-08 credits appear?" → Selected: "Per-scene implementation, inspiration, and notices" (recommended default). Inspiration links do not replace notices; experimental Rust claims stay honest.

---

## Uncertain-scene order

| Option | Description | Selected |
|--------|-------------|----------|
| Spike Float or Sink, Jelly Drop, and Water Wheel early | Then complete Fountain and Color Mixer on the same contract; evolve Dam Break | ✓ |
| Finish easy scenes first | Dam Break polish, Fountain, Color Mixer, then uncertain compositions | |
| Ship five and leave Water Wheel labeled not ready | Honest incomplete catalog continues | |

**User's choice:** Spike Float or Sink, Jelly Drop, and Water Wheel early (recommended default)
**Notes:** [auto] Uncertain-scene order — Q: "Which demos should be proven first?" → Selected: "Spike Float or Sink, Jelly Drop, and Water Wheel early" (recommended default). Matches ROADMAP.md.

---

## Design system and verification

| Option | Description | Selected |
|--------|-------------|----------|
| Keep semantic HTML and scoped dark CSS | No MysticUI/Tailwind/shadcn; local native-behavior proofs; independent AI review | ✓ |
| Adopt MysticUI or Tailwind now | Six-card catalog as the design-system trigger | |
| Require Phase 19 smoke and live Pages matrix to close Phase 18 | Pointer/a11y/six-scene hosted suite as this phase's gate | |

**User's choice:** Keep semantic HTML and scoped dark CSS (recommended default)
**Notes:** [auto] Design system and verification — Q: "Should Phase 18 adopt a component library?" → Selected: "Keep semantic HTML and scoped dark CSS" (recommended default). Continues Phase 17 D-09 and standards-overrides.md.

[auto] Design system and verification — Q: "What verification closes this phase?" → Selected: "Local native-behavior proofs per scene plus independent AI review" (recommended default).

---

## Claude's Discretion

- Exact particle counts, geometry, colors, camera bounds, and control enumerations.
- Exact card layout, SVG artwork, credit-panel markup, and control labels.
- WASM factory naming and module split.
- Whether Dam Break keeps the existing dynamic circle or an equivalent documented obstacle.

## Deferred Ideas

- Pointer/touch polish, accessibility/responsive acceptance, and six-scene browser smoke — Phase 19.
- MysticUI/Tailwind/shadcn — not justified now.
- Share-link control serialization, scene editor, diagnostic gallery, workers/threads/zero-copy, WebGPU, package publication — outside v1.1.
