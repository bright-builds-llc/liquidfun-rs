---
phase: quick-261010-wkd
plan: "01"
subsystem: web-playground-hud
tags: [hud, icon, portrait-frame, tesla-valve, safe-frame]
generated_by: gsd-executor
lifecycle_mode: quick
phase_lifecycle_id: quick-261010-wkd
generated_at: "2026-10-11T04:36:46Z"
requires: [quick-261010-vzz]
provides:
  - SliderPanelIcon (panel + visibility-aware chevron) on the HUD sliders toggle
  - TESLA_VALVE_PORTRAIT narrowed to the valve's wall extent
affects:
  - web/src/components/CanvasStage.tsx
  - web/src/catalog/portrait-bounds.ts
tech-stack:
  added: []
  patterns: [source-text guard tests for SVG glyph reuse]
key-files:
  created: []
  modified:
    - web/src/components/CanvasStage.tsx
    - web/tests/hud-sliders-toggle.test.ts
    - docs/superpowers/specs/2026-10-10-hud-safe-frame-design.md
    - web/src/catalog/portrait-bounds.ts
    - web/tests/portrait-bounds.test.ts
decisions:
  - "Tesla valve portrait x extent is ±0.54 m: outer bypass wall faces at ±0.45 m (centerline ±0.4199 + 0.03 half-thickness) plus a 0.09 m margin; Y left at -0.2..4.6 because X-only narrowing already met the target"
  - "Wall extent derived from a Python port of geometry.rs (no cargo run), per the executor override"
  - "No CSS change: .hud-button svg already sets fill: none and stroke on the svg, which rect inherits"
metrics:
  duration: 5min
  completed: 2026-10-11
  tasks: 2
  files: 5
---

# Quick 261010-wkd: Distinct HUD slider toggle icon and tighter Tesla valve frame Summary

The HUD sliders toggle now uses a Lucide-style panel-with-chevron glyph that shares no path data with the Controls drawer glyph. The Tesla valve portrait frame is narrowed from ±1.35 m to ±0.54 m, measured from the wall geometry. At 390x844 the valve grows from 98.5 to 118.1 px/m (+19.9%), and its frame centre moves from x=241 to x=211.

## Tasks

| Task | Name | Commits | Files |
| ---- | ---- | ------- | ----- |
| 1 | Panel-with-chevron toggle icon | 05e1ca545 (RED), 1e29da273 (GREEN) | CanvasStage.tsx, hud-sliders-toggle.test.ts, hud-safe-frame-design.md |
| 2 | Fit Tesla valve portrait frame to its walls | 6bc85b040 | portrait-bounds.ts, portrait-bounds.test.ts |

### Task 1

- `SlidersIcon` was deleted. The new `SliderPanelIcon({ expanded })` draws `rect 3,3 18x18 rx2` and a `M3 15h18` strip. Inside `<Show when={props.expanded}>` it draws the down chevron `m9 9 3 3 3-3` (Hide sliders); the fallback is the up chevron `m9 12 3-3 3 3` (Show sliders).
- `HudSlidersToggle` renders `<SliderPanelIcon expanded={props.visibility === "shown"} />`. The aria-label, aria-expanded, aria-controls, data-hud-obstacle, and class are unchanged.
- The 4 new guard tests failed before the change (4 failed, 16 passed) and passed after it (20/20).
- The spec sentence was updated. `just markdown-check` passes.

### Task 2: measurement method and extents

**Method:** I did not use a Rust probe and ran no cargo build or test, per the executor override. I ported `valve_geometry`/`outline_segments` from `crates/liquidfun-wasm/src/scene/tesla_valve/geometry.rs` line for line into a scratchpad Python script (`scratchpad/visual-wkd/tesla_extent.py`) and evaluated it for both flow directions.
- To validate the port, I checked it against points already in the test. The inlet wall faces come out at x = -0.0331 ± (0.12+0.03) → -0.183 / -0.003, and the end y is 4.272 + 0.01 end overlap = 4.282. Both match the test's existing (-0.183/-0.003, 4.283) points.

| Quantity | Forward | Reverse |
| --- | --- | --- |
| Centerline x | -0.4199 .. 0.4199 | -0.4199 .. 0.4199 |
| Centerline y | 0.0280 .. 4.2720 | 0.0280 .. 4.2720 |
| Extreme -x point | (-0.4199, 2.7409) | (-0.4199, 1.5591) |
| Extreme +x point | (0.4199, 3.5909) | (0.4199, 0.7091) |

- Wall faces: x ±0.4499 m. The planner estimated ±0.49; the difference is 0.04, within the 0.05 tolerance.
- Chosen bounds: `minX -0.54, maxX 0.54` (face + 0.09 margin). `minY -0.2` and `maxY 4.6` are unchanged.
- No Y tightening was needed: narrowing X alone raised px/m by 19.9%.
- The drain sensor (x ±0.7) only destroys particles and is never drawn, so it is not framed.

### Unit-test fill fractions (390x844, full viewport camera)

- Before: height 0.754, width about 1.0.
- After: width 0.468, height 0.962.
- Overrides: `MIN_WIDTH_FRACTION["tesla-valve"] = 0.42` (measured minus about 0.05) and `MIN_HEIGHT_FRACTION["tesla-valve"] = 0.9`, up from 0.75.
- Before the frame change, the new 0.9 height threshold failed at 0.754 (RED). It passes after the change.
- Wall list: I added the four extreme centerline points (±0.42 at y 2.741/3.591/1.559/0.709) and two wall-face points (±0.45). All are inside the frame.

### Browser measurement (vite preview of fresh frontend build, data-world-frame)

| Viewport / sliders | Before (vzz) frame | Before px/m | After frame | After px/m | Centre x before → after | Legend before → after |
| --- | --- | --- | --- | --- | --- | --- |
| 390x844 shown | 108,138.3 → 374,611.2 (266x472.9) | 98.52 | 147.2,54.5 → 274.8,621.5 (127.6x567) | 118.11 | 241 → 211 | 1 m → 50 cm |
| 390x844 hidden | 33.1,164 → 356.9,739.6 (323.8x575.6) | 119.92 | 133.9,54.5 → 288.1,739.6 (154.1x685.1) | 142.72 | 195 → 211 | 50 cm → 50 cm |
| 1280x800 shown | 613.3,16 → 834.7,784 | n/a (landscape bounds) | identical | n/a | 724 → 724 | 50 cm → 50 cm |
| 1280x800 hidden | 551.3,16 → 772.7,784 | n/a | identical | n/a | 662 → 662 | 50 cm → 50 cm |

- At 390x844 the valve is +19.9% larger, against a target of at least 5%. Its frame centre is 16 px right of x=195, within the 30 px limit; the HUD stats column ends at x=84.
- Hiding the sliders still grows the frame: area 72,349 px² → 105,566 px².
- 1280x800 is unchanged to the pixel.

**Screenshots (scratchpad only, not committed):** `/private/tmp/claude-501/-Users-peterryszkiewicz-Repos-liquidfun-rs/aa287ada-55ea-4cdd-97f9-4f78f778998c/scratchpad/visual-wkd/`
- `baseline/` holds the vzz `before-`/`after-tesla-valve-{390x844,1280x800}.png`, copied from `web/test-results/hud-safe-frame-visual/`.
- `before/before-tesla-valve-{390x844,1280x800}.png`, re-shot from the vzz build.
- `after/after-tesla-valve-{390x844,1280x800}.png`.
- Visual check: at 390x844 the valve fills the column between the title and the slider card, and the new panel-chevron icon is visible bottom-left.

## Verification

| Check | Result |
| --- | --- |
| `bunx vitest run tests/hud-sliders-toggle.test.ts` | 20 passed |
| `bunx vitest run tests/portrait-bounds.test.ts` | 5 passed |
| `bun run test:unit` | 60 files, 525 tests passed |
| `bun run typecheck` | clean |
| Frontend build (`bun run build:app` with VITE_* provenance env) | built |
| `bunx playwright test e2e/hud-safe-frame.spec.ts e2e/canvas-stage.spec.ts` | 73 passed |
| `bun scripts/bright-builds-check.ts all` | findings=0 |
| `just markdown-check` | pass |
| `cargo fmt --all --check` | pass |
| `git diff --quiet -- crates/` | clean |

## Deviations from Plan

1. **[Override - executor constraint] No Rust probe.** Task 2 Step 1's temporary `cargo test` probe was replaced with a Python port of the geometry, as described above. No Rust files were touched.
1. **[Override - executor constraint] `just web-build` not run.** It runs `wasm-pack` (a cargo build) before the frontend build. I ran the frontend half that `scripts/web-build.ts` performs (`typecheck`, `test:unit`, `build:app` with `VITE_APP_VERSION`/`VITE_GIT_SHA`/`VITE_BUILD_ID`/`VITE_BUILT_AT`) against the existing generated wasm. That wasm (2026-10-09 20:36 local) predates two Phase 35 perf-only crate commits, 154ace4bf and edcafebc0. Neither changes Tesla valve geometry, so the layout measurements are unaffected, but the browser runs did not exercise those perf changes.
1. **[Rule 1 - Bug] Typed the guard test's path helper.** `matchAll` captures are `string | undefined`, which failed `tsc`. The fix was folded into the GREEN commit 1e29da273.
1. **Wall-face points added.** Beyond the plan's centerline extremes, I added ±0.45 wall-face points so the test also guards wall thickness.
1. **Height threshold set to 0.9, not 0.80.** The plan allowed a higher value when measurement supports it; the measured height is 0.962.

No e2e premise change was needed: the toggle round trip still uses tesla-valve, and its frame grows.

## Known Stubs

None.

## Self-Check: PASSED

- Files: CanvasStage.tsx, hud-sliders-toggle.test.ts, hud-safe-frame-design.md, portrait-bounds.ts, portrait-bounds.test.ts are present and modified.
- Commits found: 05e1ca545, 1e29da273, 6bc85b040.
