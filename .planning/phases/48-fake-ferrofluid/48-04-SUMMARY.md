---
phase: 48-fake-ferrofluid
plan: "04"
subsystem: readme-gallery-and-phase-gates
tags: [scene, fake-ferrofluid, color-check, readme-svg, webp, fingerprint-gate, scene-spot, gates]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 48-2026-10-15T04-20-00
generated_at: "2026-10-15T07:07:19.000Z"
requires:
  - "48-01: scene core, particle_force helper, #8A84AD liquid color"
  - "48-02: magnet seam, marker, 3.0 m walls, README timing test (flat 393, re-rise 495)"
  - "48-03: catalog registration, README plan (magnet cues at samples 120 and 160), FERRO preview color"
provides:
  - "docs/assets/readme/fake-ferrofluid-10s.svg and .webp plus the README gallery entry"
  - "docs/adding-a-scene.md: R15 (shared particle_force helper) and W7 (spoken-unit branch in formatRangeValueText)"
  - "Four-mode color check: readable in Shaded blob, GPU Water, circle wireframe and the static preview at 1280 x 800 and 390 x 844; no color change"
  - "Phase 48 gate evidence: 16/16 local checks, ferrofluid scene-spot 990/990 particles at 0.314 ms/step, id-join gate PASS 27/27 with fake-ferrofluid e73ef71f0615b4eb added"
  - "Full-catalog capture target/dam-break-perf/2026-10-15T07-06-56Z/scene-spot.json at f798bff91 for Plan 05's baseline growth"
affects: [48-05]
tech-stack:
  added: []
  patterns:
    - "Temporary Playwright screenshot spec seeded through localStorage render mode, deleted before commit"
key-files:
  created:
    - docs/assets/readme/fake-ferrofluid-10s.svg
    - docs/assets/readme/fake-ferrofluid-10s.webp
  modified:
    - README.md
    - docs/adding-a-scene.md
key-decisions:
  - "48-04: #8A84AD stays. The puddle and every crest are readable in all four render paths at both viewports, so the lighten fallback was not used"
  - "48-04: README cues stay at samples 120 and 160. The reviewed clip shows spikes at 1 s and 5.83 s, a flat puddle at 7.83 s and spikes again at 9.83 s"
  - "48-04: ferrofluid scene-spot 2026-10-15T07-06-27Z gives start 990, end 990 and median 0.314 ms/step. The full-catalog gate passes 27/27 unchanged with fake-ferrofluid e73ef71f0615b4eb added (capture 2026-10-15T07-06-56Z at f798bff91)"
  - "48-04: Finding, not re-tuned: in the 3.0 m frame the crests rise about 0.3 m and fill only the bottom quarter of the dish height. They read as rounded crests more than sharp spikes"
requirements-completed: []
requirements-advanced: [FERRO-01, FERRO-02, FERRO-03]
duration: 27min
completed: 2026-10-15
---

# Phase 48 Plan 04: Color Check, README Preview and Phase Gates Summary

**The graphite-violet ferrofluid can be read in all four render paths, so its color is unchanged. The README gallery now has a reviewed 10 s clip with spikes, a collapse when the magnet switches off and a re-rise. Every local gate passes on the shipped tree, and the id-join fingerprint gate reports 27/27 protected ids unchanged with `fake-ferrofluid` added. The scene runs 990 particles at 0.314 ms/step in native release.**

## Performance

- **Duration:** 27 min
- **Started:** 2026-10-15T06:41:09Z
- **Completed:** 2026-10-15T07:07:19Z
- **Tasks:** 3 (Task 2 has a commit; Tasks 1 and 3 changed no tracked file)
- **Files modified:** 4 (2 created, 2 modified)

## Task 1: Four-mode liquid color check (D-16, UI-D73)

I ran a temporary spec, `web/e2e/zz-ferro-color-check.spec.ts`, against `just web-build`. It seeded `liquidfun.render-mode.v1`, waited until `data-step-index` was at least 120 (2 s), then took screenshots. The spec was deleted after the run, and `git status --short web/e2e` is empty. The canvas reported the drawn surface, so GPU Water actually ran in headless Chromium (`drawn=webgl-water`) and no fallback was involved.

| Mode | Viewport | Drawn surface | Readable | Note | Screenshot |
|---|---|---|---|---|---|
| Shaded blob (default) | 1280 x 800 | `webgl` | yes | Solid muted lavender-gray puddle. 7 crests (5 tall center crests and 2 shoulders) stand out sharply against `#071018`. | `target/phase48/color/shaded-blob-desktop.png` |
| GPU Water | 1280 x 800 | `webgl-water` | yes | Slightly denser violet-gray with small scattered white specks inside the body. The outline and every crest stay clear. | `target/phase48/color/water-desktop.png` |
| Circle wireframe | 1280 x 800 | `canvas` | yes | The dimmest mode, with thin 0.3 strokes. The circle lattice and crest outlines still separate clearly from the canvas. | `target/phase48/color/circle-wireframe-desktop.png` |
| Shaded blob | 390 x 844 | `webgl` | yes | 7 small crests that can be counted across the dish. | `target/phase48/color/shaded-blob-portrait.png` |
| GPU Water | 390 x 844 | `webgl-water` | yes | Same as desktop, with visible white specks. | `target/phase48/color/water-portrait.png` |
| Circle wireframe | 390 x 844 | `canvas` | yes | Reads as a fine violet mesh with crests on top. Faint but distinct. | `target/phase48/color/circle-wireframe-portrait.png` |
| Static preview (sidebar) | 1280 x 800 | SVG | yes | The `#8A84AD` spike row and puddle are clear in the dish, above the magnet bar. | `target/phase48/color/static-preview-desktop.png`, `static-preview-svg-desktop.png` |

`ParticleColor::new(138, 132, 173, 255)` and `FERRO = "#8A84AD"` still match, and neither file changed.

### Does the liquid read as "dark" next to the shared water blue? (UI checker recommendation)

- It reads as clearly **not water**. `#8A84AD` is a desaturated gray-violet, and the shared `#4DA3FF` is a saturated sky blue. Nobody would mistake one for the other.
- On the near-black canvas it reads as a **mid-tone lavender-gray, not a dark liquid**. Real ferrofluid looks black, and this puddle is visibly lighter than a "dark liquid" would be.
- This follows from the UI-SPEC choice: `#8A84AD` is the darkest candidate that keeps 3:1 contrast at the 0.73 density-shade floor. A darker value would trade away D-16 readability, especially in circle wireframe, which is already the faintest mode.
- Recorded as a finding for the owner. I did not change the color, because the plan only allows lightening and only when a mode fails.

### Observation: white specks in GPU Water

In Water mode the puddle shows small scattered white specks at both viewports. The UI-SPEC expected no foam whitening, because the probe measured at most 0.6 m/s at the default strength. I did not diagnose the cause (foam or specular highlights). The specks do not hurt readability.

## Task 2: README preview (S2, D-21, D-22)

I ran `LIQUIDFUN_README_FONT_FILE=/usr/local/texlive/2025/texmf-dist/fonts/truetype/public/dejavu/DejaVuSans.ttf just readme-svg`; the log is `target/phase48/readme-svg.log`. The run wrote the new SVG (2,624,682 bytes, 990 circles, 20 lines) and the WebP (12,438,516 bytes, 600 frames at 1280 x 960), and upserted the gallery entry.

- 27 other scenes' WebPs were re-encoded with different bytes. All of them were restored with `git checkout --` and are listed in `target/phase48/restored-assets.txt`. No other scene's SVG changed bytes.
- The gallery entry sits between the `readme-svg-gallery` markers, after Archimedes Screw. Its alt text is "Fake Ferrofluid simulation preview" and it links to `#/scene/fake-ferrofluid`. It contains none of the UI-D77 forbidden phrases.

### Clip review (1-based WebP frames, frame N at (N - 1) / 60 s)

| Frame | Time | What it shows | Check |
|---|---|---|---|
| 61 | 1.00 s | The whole dish, both 3.0 m walls and the magnet in frame. The marker has its on look (outline, pole divider and 3 ticks). The puddle shows 7 crests: 5 tall center spikes and 2 lower shoulders. | >= 4 separate spikes: pass |
| 351 | 5.83 s | Same layout just before the off cue. 7 crests (5 tall), centered over the on marker. | >= 4 spikes: pass |
| 471 | 7.83 s | A flat puddle with only a slight ripple. The marker is a plain outline, with no divider or ticks. | flat and off: pass |
| 591 | 9.83 s | 7 crests again (5 tall). The marker has its on look. | re-rise: pass |

PNGs: `target/phase48/readme-frame-{61,351,471,591}.png`. No liquid leaves the dish in any frame. The puddle can be read in the circle-wireframe clip, but it is faint: thin violet strokes on `#071018`.

**Result:** the clip shows spikes, a collapse and a re-rise. The cue samples stay at 120 and 160, and `plans.ts` is unchanged.

**SVG recorder line indices (Phase 47 deferred item):** I saw no sweep. The marker's 4 outline lines are static. The divider and the 3 ticks keep fixed indices 17-20 and only animate `opacity`: 1 to 0 at sample 120 and back to 1 at sample 160. With `calcMode="linear"` that is a one-sample fade, not a geometry sweep.

### Finding: spike size in the taller 3.0 m frame (48-02 frame change)

In the README frame, at about 173 px/m:

- The puddle is about 0.43 m deep (74 px), and the crests rise about 0.28 m above it (about 48 px).
- The crest tips reach about 0.7 m of the 3.0 m wall height, so the liquid and its crests fill only the bottom quarter of the dish interior. The top 60 % of the frame is empty wall.
- The crests look like a row of rounded, evenly spaced humps more than needle-sharp ferrofluid spikes.

Desktop (1280 x 800): the crests are clear and easy to count.

Portrait (390 x 844, about 50 px/m): the dish is about 330 px wide, and each crest is roughly 15 px tall. They can still be counted, but they are small.

This is recorded as a finding with the frame and screenshot paths above. I did not re-tune physics in this plan. Possible follow-ups for the owner:

- Lower the walls again after a re-check of the tallest held-magnet column (2.67 m, which set the 3.0 m walls).
- Raise the default strength within D-04's 5-9 peaks.

### Checklist upkeep (D-22)

`docs/adding-a-scene.md` gains two `optional` rows:

- **R15** `scene/particle_force.rs`: "Continuous per-particle forces: convert accelerations with `ForceScale` and `GravityFrame` from the shared helper and apply them once per fixed step in `on_advance`; never hard-code -y."
- **W7** `web/src/components/range-control.ts` `formatRangeValueText`: a spoken-unit branch only when a live control introduces a new unit. This phase needed it for `%` (48-03), and the checklist did not list it.

`just markdown-check` passes.

## Task 3: Gates and evidence (D-15, D-24, D-26)

### Local checks (`bash target/phase48/checks-04.sh`, exit 0 on f798bff91)

Logs are in `target/phase48/checks-04/`, and the run transcript is in `target/phase48/checks-04.log`.

| # | Step | Result |
|---|---|---|
| 1 | `cargo fmt --all --check` | pass |
| 2 | `cargo clippy --all-targets --all-features -- -D warnings` | pass |
| 3 | `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings` | pass |
| 4 | `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | pass |
| 5 | `cargo build --all-targets --all-features` | pass |
| 6 | `cargo test --all-features` | pass (1,084 passed, 0 ignored) |
| 7 | `cargo test -p liquidfun-wasm --all-features` | pass (744 passed, 81 ignored release-only) |
| 8 | `cargo test -p liquidfun-wasm --release --lib fake_ferrofluid` | pass (79 passed, 0 ignored; spike, flow and sync tests ran) |
| 9 | `cargo test -p liquidfun-wasm --release --lib particle_force` | pass (9 passed) |
| 10 | `cargo test -p xtask --bin xtask playground::` | pass (104 passed) |
| 11 | `cd web && bun run typecheck && bun run test:unit` | pass (121 files, 1,856 tests) |
| 12 | `bun scripts/bright-builds-check.ts all` | pass (0 findings, 1,603 files scanned for length) |
| 13 | `just web-build` | pass |
| 14 | Playwright `player.spec.ts`, `shell.spec.ts`, `hud-safe-frame.spec.ts` (chromium) | pass (102 passed) |
| 15 | `just markdown-check` | pass |
| 16 | `git diff --quiet -- docs/benchmarks/scene-fingerprints.json crates/liquidfun` | pass |

### Ferrofluid scene-spot (D-15, D-24, SCENE-02)

`cargo xtask playground scene-spot --scene fake-ferrofluid` gave stamp `2026-10-15T07-06-27Z` (`target/dam-break-perf/2026-10-15T07-06-27Z/scene-spot.json`, filtered and not full-catalog). The log is `target/phase48/scene-spot-ferro.log`.

| Field | Value | Rule |
|---|---|---|
| `start_particles` | **990** | <= 3,000: pass |
| `end_particles` | **990** | equals start (no evictions): pass |
| `median_ms_per_step` | **0.313545** | <= about 1.5: pass |
| `fingerprint` | `e73ef71f0615b4eb` | same as the full capture |

The full-catalog capture gives the same numbers: 990 / 990 at 0.31299 ms/step.

### Id-join fingerprint gate (D-24)

The full-catalog capture `just playground-scene-spot` gave stamp `2026-10-15T07-06-56Z` (28 scenes). The capture commit was `f798bff915e346a0b901454bf15eeb590d03fd77`, recorded in `target/phase48/full-capture-commit.txt`, and `crates`, `tools` and `web` were clean when it ran. Plan 05 may reuse this capture only if `crates tools web` are unchanged since f798bff91.

`just scene-fingerprint-gate --spot target/dam-break-perf/2026-10-15T07-06-56Z/scene-spot.json` (`target/phase48/fingerprint-gate.log`):

```
Scene fingerprint gate: PASS
- protected: 27/27 unchanged
- changed: none
- missing: none
- added (accepted): fake-ferrofluid e73ef71f0615b4eb
- baseline: docs/benchmarks/scene-fingerprints.json (captured at 23541975cd276880934f0044c732915d8c88d287, 2026-10-14T22-42-42Z)
- observed: target/dam-break-perf/2026-10-15T07-06-56Z/scene-spot.json (git f798bff915e346a0b901454bf15eeb590d03fd77, full_catalog true)
```

`archimedes-screw` is among the 27 unchanged ids. The baseline is untouched: `git log --oneline -1 -- docs/benchmarks/scene-fingerprints.json` is still `9989d66ff chore(47-05): protect the archimedes-screw scene fingerprint`.

## Decision Coverage (D-01..D-26)

| Decision | Where |
|---|---|
| D-01 three-part fake field | 48-01 Task 2-3; 48-02 Task 2 (`|E(h)|` sideways term) |
| D-02 bounded profile, cap, drag | 48-01 Task 3 (cap 1.5) |
| D-03 TENSILE \| VISCOUS, viscous <= 1.5 | 48-01 Task 2 |
| D-04 5-9 spikes, appear/collapse timing | 48-01 Task 3, 48-02 Task 2; 48-04 Task 2 clip (7 crests, 5 tall) |
| D-05 spike first, candidate (a) | 48-01 Task 3 |
| D-06 magnet on at rest | 48-01 Task 2 |
| D-07 pointer down/move/up/cancel | 48-02 Task 1 |
| D-08 intent-only hooks | 48-01 Task 2, 48-02 Task 1 |
| D-09 magnet-strength live spinner | 48-02 Task 1 (Rust), 48-03 Task 1-2 (web) |
| D-10 Magnet action | 48-02 Task 1, 48-03 Task 1 |
| D-11 strength and on/off independent | 48-01 Task 2, 48-02 Task 2 |
| D-12 shared particle_force helper | 48-01 Task 1; 48-04 Task 2 checklist R15 |
| D-13 calibration and tilt tests | 48-01 Task 1 |
| D-14 gravity-relative terms | 48-01 Task 1-2 |
| D-15 toy scale, <= 3,000 particles, <= 1.5 ms/step | 48-01 Task 2; 48-04 Task 3 (990/990, 0.314 ms/step) |
| D-16 dark but readable color, four-mode check | 48-01 Task 2 (`#8A84AD`), 48-03 (FERRO); 48-04 Task 1 |
| D-17 magnet marker via segments | 48-02 Task 1; 48-04 Task 2 (on/off looks in the clip) |
| D-18 view and portrait frames | 48-03 Task 1-2 |
| D-19 id, module, family, credits | 48-01 Task 2, 48-03 Task 1 |
| D-20 honest caption and hint | 48-03 Task 1-2 (UI-D77 guard); 48-04 Task 2 (gallery alt text) |
| D-21 README cues, survey Default, demo-media, POINTER_CONTROL | 48-02 Task 2 (timing test), 48-03 Task 1-2; 48-04 Task 2 (clip review) |
| D-22 checklist sites and README assets | 48-03; 48-04 Task 2 |
| D-23 release outcome tests | 48-02 Task 2 |
| D-24 gate 27/27 plus scene-spot numbers | 48-04 Task 3 |
| D-25 baseline growth in its own commit | 48-05 (pending) |
| D-26 normal local checks | every plan; 48-04 Task 3 (16/16) |

## Task Commits

1. **Task 1: four-mode color check**: no commit. Every mode was readable, so neither color file changed.
1. **Task 2: README preview and checklist rows**: `f798bff91` (docs)
1. **Task 3: gates and evidence**: no commit. Every gate passed on the first run, so no fix was needed and the evidence stays in gitignored `target/phase48/`.

## Deviations from Plan

None in what was built: the plan was executed as written. Notes:

- Task 2 is a docs and asset commit, so before it I ran only `just markdown-check` and `tests/readme-svg.test.ts`. The full Rust and web suite (Task 3) then ran on the tree that includes that commit, and it passed.
- The checklist also gained W7 (the spoken-unit site), as the `<facts>` "any other site this phase needed" clause allows.

## Issues Encountered

None. The findings above (crest size in the 3.0 m frame, a mid-tone rather than dark liquid, and white specks in Water) are recorded for the owner and do not block anything.

## Known Stubs

None.

## Next Phase Readiness

Plan 05 can grow `docs/benchmarks/scene-fingerprints.json` in its own single-file commit. It appends `fake-ferrofluid` `e73ef71f0615b4eb` and can reuse capture `2026-10-15T07-06-56Z` at f798bff91 if `crates tools web` are unchanged. It should then rerun the gate and expect 28/28 unchanged.

## Self-Check: PASSED
