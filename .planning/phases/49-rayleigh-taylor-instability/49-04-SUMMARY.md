---
phase: 49-rayleigh-taylor-instability
plan: "04"
subsystem: readme-gallery-and-phase-gates
tags: [scene, rayleigh-taylor, color-check, readme-svg, webp, fingerprint-gate, scene-spot, gates]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 49-2026-10-19T00-36-51
generated_at: "2026-10-19T03:12:05Z"
requires:
  - "49-01: scene core, TUNING (r 0.035, settle 36, ramp 24, 1,484 particles), #FB923C / #3B82F6 layer colors"
  - "49-02: release session tests (fingers at step 114, overturn at step 156)"
  - "49-03: catalog registration, watch-first README plan { id: rayleigh-taylor, cues: [] }, RT_HEAVY / RT_LIGHT preview colors"
provides:
  - "docs/assets/readme/rayleigh-taylor-10s.svg and .webp plus the README gallery entry"
  - "docs/adding-a-scene.md: R16 (recreating preset Rust arm: build reads and rejects, apply_control returns Recreated)"
  - "Four-mode color check: orange and blue are distinct from each other and from the canvas in Shaded blob, GPU Water, circle wireframe and the static preview, at 1280 x 800 and 390 x 844. No color change"
  - "Phase 49 gate evidence: 16/16 local checks, rayleigh-taylor scene-spot 1,484/1,484 particles at 0.363 ms/step, id-join gate PASS 28/28 with rayleigh-taylor ac59254e2231967c added"
  - "Full-catalog capture target/dam-break-perf/2026-10-19T03-11-46Z/scene-spot.json at e495c30f5 for Plan 05's baseline growth"
affects: [49-05]
tech-stack:
  added: []
  patterns:
    - "Temporary Playwright screenshot spec seeded through localStorage render mode and waiting on main[data-step-index], deleted before commit"
    - "Pixel-column trough profile (orange lowest / blue highest per 20 px band) to back the frame-61 visible-motion judgement with numbers"
key-files:
  created:
    - docs/assets/readme/rayleigh-taylor-10s.svg
    - docs/assets/readme/rayleigh-taylor-10s.webp
  modified:
    - README.md
    - docs/adding-a-scene.md
key-decisions:
  - "49-04: #FB923C and #3B82F6 stay. Both layers read as two distinct layers against #071018 in all four render paths at both viewports, so neither fallback (#4B8DF8, #F97316) was used"
  - "49-04: README clip reviewed at frames 1, 31, 61, 115, 157, 241, 571. The real timeline is earlier than the UI-SPEC nominals: fingers show at frame 115 (1.9 s) and the overturn is complete by frame 241 (4 s), so frame 115 is the finger frame and frame 241 is already overturned. No cue added"
  - "49-04: frame-61 rule passes. The interface amplitude grows from about 55 px (frame 31) to 80-100 px (frame 61); the interior trough near x 820 deepens by 16 px and the crest near x 720 rises by about 40 px. No tuning change"
  - "49-04: rayleigh-taylor scene-spot 2026-10-19T03-11-20Z gives start 1,484, end 1,484 and median 0.363 ms/step. The full-catalog gate passes 28/28 unchanged (fake-ferrofluid 9f1e59fce8af89e7 included) with rayleigh-taylor ac59254e2231967c added (capture 2026-10-19T03-11-46Z at e495c30f5)"
requirements-completed: []
requirements-advanced: [RT-01, SCENE-01, SCENE-02, SCENE-03]
duration: 24min
completed: 2026-10-19
---

# Phase 49 Plan 04: Color Check, README Preview and Phase Gates Summary

**The orange and blue layers read as two distinct layers in all four render paths, so the colors are unchanged. The README gallery now has a reviewed 10 s watch-first clip that shows the seeded ripple, five sinking orange fingers and the overturn. Every local gate passes on the shipped tree, and the id-join fingerprint gate reports 28/28 protected ids unchanged with `rayleigh-taylor` added. The scene runs 1,484 particles at 0.363 ms/step in native release.**

## Performance

- **Started:** 2026-10-19T02:48:27Z
- **Completed:** 2026-10-19T03:12:05Z
- **Tasks:** 3 (Task 2 has a commit; Tasks 1 and 3 changed no tracked file)
- **Files modified:** 4 (2 created, 2 modified)

## Task Commits

1. **Task 1: four-mode layer color check**: no commit (all modes passed; the temporary spec was deleted)
1. **Task 2: README preview and R16 checklist row**: `e495c30f5` (docs)
1. **Task 3: gates and evidence**: no commit (no tracked file changed)

## Task 1: Four-mode layer color check (D-12, UI-D86)

I ran a temporary spec, `web/e2e/zz-rt-color-check.spec.ts`, against `just web-build`. It seeded `liquidfun.render-mode.v1` with an init script, opened `/liquidfun-rs/#/scene/rayleigh-taylor`, and took viewport screenshots once `main[data-step-index]` reached 30 (0.5 s), 120 (2 s), 240 (4 s) and 600 (10 s). It recorded `canvas.scene-canvas[data-drawn-surface]` at each moment. For the static preview it opened the "Demos" drawer, because the canvas-stage layout hides the sidebar, and screenshotted the `a.demo-nav-link[href$="#/scene/rayleigh-taylor"]` card. The spec was deleted after the run, and `git status --short web/e2e` is empty. There are 31 PNGs in `target/phase49/color/`.

GPU Water really ran in headless Chromium (`drawn=webgl-water` at every moment), so no fallback was involved. Shaded blob drew on `webgl` and circle wireframe on `canvas`.

| Mode | Viewport | Moment (step) | Orange vs blue distinct | Both vs canvas | Note |
|---|---|---|---|---|---|
| Shaded blob | 1280 x 800 | 0.5 s (30) | yes | yes | Bright orange upper layer over saturated blue. The interface already shows five rounded troughs from the seed. |
| Shaded blob | 1280 x 800 | 2 s (120) | yes | yes | Orange fingers sink to the floor at five places. A blue plume rises through the middle and breaks the top surface. |
| Shaded blob | 1280 x 800 | 4 s (240) | yes | yes | Mixed: blue on top, orange concentrated in the bottom third, interleaved blobs in between. |
| Shaded blob | 1280 x 800 | 10 s (600) | yes | yes | Blue top half, orange packed on the floor, a speckled mixed band between them. |
| Shaded blob | 390 x 844 | all four | yes | yes | The tank fills the width. The same states as desktop, smaller but every finger is clear. |
| GPU Water | 1280 x 800 | all four | yes | yes | Slightly darker, more saturated orange and blue with white specular specks throughout both layers. The layers and every finger stay clear. |
| GPU Water | 390 x 844 | all four | yes | yes | Same as desktop, with the white specks more visible at that scale. |
| Circle wireframe | 1280 x 800 | all four | yes | yes | The faintest mode: thin strokes. A 3x zoom of the flat interface shows thin blue rings (`#1D427A`-`#204A8A` after anti-aliasing) clearly separate from the orange rings and from the canvas. Fingers and the overturned end read as orange and blue meshes. |
| Circle wireframe | 390 x 844 | all four | yes | yes | Faint but distinct, a fine orange mesh over a blue mesh. Fingers and the overturned end are readable. |
| Static preview (sidebar card) | 1280 x 800 | static | yes | yes | Orange top band, blue bottom band, three orange finger stems with caps and two blue bubbles inside the gray tank outline. It is small in the card, because the wide tank sets the width. |

Screenshots: `target/phase49/color/<mode>-<desktop|portrait>-<flat-0p5s|fingers-2s|fingers-4s|overturned-10s>.png`, `static-preview-desktop.png`, `static-preview-svg-desktop.png`, plus `zoom-wire-flat.png`, `zoom-static.png` and the portrait montages `montage-<mode>-portrait.png`.

`ParticleColor::new(251, 146, 60, 255)` / `RT_HEAVY = "#FB923C"` and `ParticleColor::new(59, 130, 246, 255)` / `RT_LIGHT = "#3B82F6"` still match, and neither file changed. No foam whitening at fast fingers was noticeable at ratio 2. The specks in Water are present at rest too, as in Phase 48.

## Task 2: README preview (S2, D-16, D-17)

I ran `LIQUIDFUN_README_FONT_FILE=/usr/local/texlive/2025/texmf-dist/fonts/truetype/public/dejavu/DejaVuSans.ttf just readme-svg`; the log is `target/phase49/readme-svg.log`. The run wrote the new SVG (3,913,636 bytes, 1,484 circles, 16 lines) and the WebP (33,759,958 bytes, 600 frames at 1280 x 960), and upserted the gallery entry.

- 27 other scenes' WebPs were re-encoded with different bytes. All of them were restored with `git checkout --` and are listed in `target/phase49/restored-assets.txt`. The `fake-ferrofluid` WebP came out byte-identical. No other scene's SVG changed bytes.
- The gallery entry sits between the `readme-svg-gallery` markers, after Fake Ferrofluid. Its title is "Rayleigh–Taylor Instability" (en dash), its alt text is "Rayleigh–Taylor Instability simulation preview", and it links to `#/scene/rayleigh-taylor`. It contains none of the UI-D90 forbidden phrases (grep for `density|denser|atwood|growth rate|measured|simulated|real fluid|two fluids` on the entry finds nothing).
- `cd web && bunx vitest run tests/readme-svg.test.ts`: 8 passed.

### Clip review (circle wireframe, 1-based WebP frames, frame N at (N - 1) / 60 s, one step per frame)

The orchestrator noted that the real timeline runs earlier than the UI-SPEC nominals: fingers at step 114 and overturn at step 156. I reviewed the plan's four frames plus frame 1, frame 115 (fingers) and frame 157 (overturn), and opened each PNG myself.

| Frame | Time | What it shows | Check |
|---|---|---|---|
| 1 | 0.00 s | The seeded lattice: an orange block over a blue block. The interface carries five shallow troughs from the polynomial seed. The whole tank, both thick walls, the floor and the lid are in frame. | start state: pass |
| 31 | 0.50 s | Orange over blue after part of the settle. The lattice has loosened, and the seeded ripple shows as five rounded orange troughs, at x of about 270, 450, 630, 810 and 990 px. | flat start with the ripple: pass |
| 61 | 1.00 s | The troughs are narrower and deeper, and the blue crests between them rise. The orange top surface dips over the middle crest. | troughs deepening (SCENE-02): pass, numbers below |
| 115 | 1.90 s | Five separate orange fingers sink away from the walls, at about x 300 (reaching the floor), 450, 610, 770 and 970 px. A wide blue plume rises at x of about 530 and breaks the top surface. | at least 3 fingers by about 4 s: pass (5 at 1.9 s) |
| 157 | 2.60 s | Overturn in progress. Orange columns reach the floor on both sides, and blue fills the top and the middle. | overturn under way |
| 241 | 4.00 s | Already overturned: blue mesh across the top half, orange concentrated in the bottom third, interleaved patches in between. The fingers have merged, so separate fingers are no longer visible at this frame. | finger count is met at frame 115 instead |
| 571 | 9.50 s | Settled: blue on top, orange packed on the floor about a third of the fluid deep, with a mixed band between. | orange mostly below blue: pass |

PNGs: `target/phase49/readme-frame-{1,31,61,91,115,131,157,241,571,600}.png`. No liquid leaves the tank in any frame. The highest fluid, at frame 157, stays below the lid. The blue rings can be read in the circle-wireframe clip, but the whole clip is dim (thin strokes on `#071018`), as in Phase 48. After 4 s the clip shows a long, nearly static settled tail, which the orchestrator accepted.

**Frame-61 rule (SCENE-02).** Beyond the visual comparison, `target/phase49/trough-depths.txt` gives each 20 px column band's lowest orange row and highest blue row, measured at 312 px/m:

- Interface amplitude, from blue crest to orange trough: about 55 px at frame 31, and 80-100 px at frame 61.
- The interior trough near x 810-830 deepens from 583 to 599 px (+16 px, about 5 cm). The trough near x 970-990 deepens from 575 to 590 px (+15 px).
- The crest near x 690-730 rises from about 537 to 495 px (about 42 px).

At least one interior trough is visibly deeper at frame 61, so the failure path was not needed and `TUNING` is unchanged. This agrees with the S6 guard (ripple depth 0.0525 m at step 0 and 0.1329 m at step 60).

**Result:** the clip shows the ripple, the fingers and the overturn, and `plans.ts` is unchanged (no cue).

### Checklist upkeep (D-17)

`docs/adding-a-scene.md` gains row **R16** after R15, `optional`: `scene/<module>.rs` `build` and `apply_control`. It reads: "A recreating preset (Dam Break `water-amount`, Rayleigh–Taylor `weight-ratio`): `build` reads the stored preset and rejects unknown values, and `apply_control` returns `ControlEffect::Recreated` for each valid value so the session stores it and rebuilds. Without that arm the web cannot apply the preset. Pair it with W3/W4 and remember that Reset scene clears construction values." The value column uses plain `optional`, like rows R10-R15. I formatted the file with the pipx mdformat 1.0.0, which reflowed the Rust table's column widths. `just markdown-check` passes.

## Task 3: Gates and evidence (D-11, D-19, D-21)

### Local checks (`bash target/phase49/checks-04.sh`, exit 0 on e495c30f5)

Per-step logs are in `target/phase49/check-<NN>-<label>.log`, and the run transcript is in `target/phase49/checks-04.log`.

| # | Step | Result |
|---|---|---|
| 1 | `cargo fmt --all --check` | pass |
| 2 | `cargo clippy --all-targets --all-features -- -D warnings` | pass |
| 3 | `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings` | pass |
| 4 | `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | pass |
| 5 | `cargo build --all-targets --all-features` | pass |
| 6 | `cargo test --all-features` | pass (1,084 passed, 0 ignored) |
| 7 | `cargo test -p liquidfun-wasm --all-features` | pass (807 passed, 102 ignored release-only) |
| 8 | `cargo test -p liquidfun-wasm --release --lib rayleigh_taylor` | pass (80 passed, 0 ignored; spike, hooks, flow, sync and unit tests ran) |
| 9 | `cargo test -p liquidfun-wasm --release --lib particle_force` | pass (9 passed) |
| 10 | `cargo test -p xtask --bin xtask playground::` | pass (104 passed) |
| 11 | `cd web && bun run typecheck && bun run test:unit` | pass (122 files, 1,859 tests) |
| 12 | `bun scripts/bright-builds-check.ts all` | pass (0 findings, 1,614 files scanned for length) |
| 13 | `just web-build` | pass |
| 14 | Playwright `player.spec.ts`, `shell.spec.ts`, `hud-safe-frame.spec.ts` (chromium) | pass (104 passed) |
| 15 | `just markdown-check` | pass |
| 16 | `git diff --quiet -- docs/benchmarks/scene-fingerprints.json crates/liquidfun` | pass |

### Rayleigh–Taylor scene-spot (D-11, D-19, SCENE-02)

`cargo xtask playground scene-spot --scene rayleigh-taylor` gave stamp `2026-10-19T03-11-20Z` (`target/dam-break-perf/2026-10-19T03-11-20Z/scene-spot.json`, filtered and not full-catalog; 60 warmup steps, 120 measured steps, 3 runs). The log is `target/phase49/scene-spot-rt.log`.

| Field | Value | Rule |
|---|---|---|
| `start_particles` | **1,484** | <= 3,000: pass |
| `end_particles` | **1,484** | equals start (no evictions): pass |
| `median_ms_per_step` | **0.363068** | <= about 1.5: pass (research estimate 0.7-1.1) |
| `fingerprint` | `ac59254e2231967c` | same as the full capture |

The full-catalog capture gives the same counts and fingerprint: 1,484 / 1,484 at 0.360143 ms/step.

### Id-join fingerprint gate (D-19)

The full-catalog capture `just playground-scene-spot` gave stamp `2026-10-19T03-11-46Z` (29 scenes). The capture commit was `e495c30f55f5e9bd667a5aeb6cef561c69e5399e`, recorded in `target/phase49/full-capture-commit.txt`, and the tree was clean apart from `.planning/config.json` when it ran. Plan 05 may reuse this capture only if `crates tools web` are unchanged since e495c30f5.

`just scene-fingerprint-gate --spot target/dam-break-perf/2026-10-19T03-11-46Z/scene-spot.json` (`target/phase49/fingerprint-gate.log`):

```
Scene fingerprint gate: PASS
- protected: 28/28 unchanged
- changed: none
- missing: none
- added (accepted): rayleigh-taylor ac59254e2231967c
- baseline: docs/benchmarks/scene-fingerprints.json (captured at 43f84a6058ed8691a40557cf5c5c3171e19fceaf, 2026-10-19T00-30-27Z)
- observed: target/dam-break-perf/2026-10-19T03-11-46Z/scene-spot.json (git e495c30f55f5e9bd667a5aeb6cef561c69e5399e, full_catalog true)
```

`fake-ferrofluid` is among the 28 unchanged ids (observed `9f1e59fce8af89e7`, the re-pinned value). The baseline is untouched: `git diff --quiet HEAD -- docs/benchmarks/scene-fingerprints.json` is clean, and `git log --oneline -1 -- docs/benchmarks/scene-fingerprints.json` is still `4d8bff99a chore(quick-261016-u0s): re-pin the fake-ferrofluid scene fingerprint`.

## Decision Coverage (D-01..D-21)

| Decision | Where |
|---|---|
| D-01 one system, two groups, no `COLOR_MIXING` | 49-01 Task 1-2; 49-04 Task 1 (layers stay two colors in every mode) |
| D-02 extra weight along -up on heavy group | 49-01 Task 2; 49-02 Task 1 (force-lane and tilted-gravity tests) |
| D-03 settle then ramp | 49-01 (settle 36, ramp 24); 49-02 Task 1 (no weight in settle); 49-04 Task 2 (frame-61 rule passes without retuning) |
| D-04 seeded deterministic perturbation | 49-01 (polynomial seed C); 49-04 Task 2 (ripple visible at frames 1 and 31) |
| D-05 flag candidates | 49-01 spike (candidate 1, WATER) |
| D-06 / D-06a spike rules and amendment | 49-01 Task 3 (S0-S6, rain veto, settle frame) |
| D-07 finger measurement | 49-01 test_support; 49-02 Task 2 (fingers at step 114) |
| D-08 `weight-ratio` recreating preset | 49-01 (Rust parse); 49-03 (web control); 49-04 Task 2 (R16 checklist row) |
| D-09 restart through preset and Reset scene | 49-03 Task 1-2 |
| D-10 no pointer, `Recreated` arm | 49-01 / 49-02 (`apply_control`); 49-03 (no `POINTER_CONTROL`) |
| D-11 tank, <= 3,000 particles, ms/step | 49-01 (2.8 x 1.6 m tank); 49-04 Task 3 (1,484 particles, 0.363 ms/step) |
| D-12 orange/blue contrast, four-mode check | 49-01 (Rust colors); 49-03 (RT_*); 49-04 Task 1 |
| D-13 landscape and portrait frames | 49-03 Task 1-2; 49-04 Task 1 (portrait screenshots fill the width) |
| D-14 id, module, title, family | 49-01, 49-03 |
| D-15 honest caption | 49-03 (honesty guard); 49-04 Task 2 (gallery alt-text check) |
| D-16 watch-first README, survey, demo media | 49-03 (plans); 49-04 Task 2 (clip generated and reviewed, no cue) |
| D-17 every checklist site filled, README assets | 49-03; 49-04 Task 2 (assets, R16) |
| D-18 release tests | 49-01, 49-02; 49-04 Task 3 step 8 (80 passed, 0 ignored) |
| D-19 fingerprint gate and numbers | 49-04 Task 3 |
| D-20 baseline growth in its own commit | 49-05 (not touched here) |
| D-21 normal local checks | 49-04 Task 3 (16/16) |

## Deviations from Plan

### Execution adjustments (no rule changes)

**1. Review frames extended to match the real timeline**
- **Found during:** Task 2
- **Issue:** The UI-SPEC finger frame (241, 4 s) falls after the overturn, which completes at step 156, so separate fingers are not visible at frame 241.
- **Fix:** As the orchestrator instructed, I also reviewed frames 1, 115 (fingers, 1.9 s) and 157 (overturn, 2.6 s). The "at least 3 fingers by about 4 s" check is met at frame 115 with five fingers. Every rule and deadline is unchanged, and no cue was added.

**2. Color-check spec opens the Demos drawer for the static preview**
- **Found during:** Task 1
- **Issue:** In the default canvas-stage layout the sidebar link is hidden behind the "Demos" button, so the first static-preview capture timed out.
- **Fix:** The temporary spec clicks "Demos" first. The spec is deleted and was never committed.

**3. Extra 2 s moment in the color check**
- The spec captured 0.5 s, 2 s, 4 s and 10 s. I added 2 s because the fingers exist around 1.9 s, and by 4 s the scene has already overturned.

No tracked-file deviations. `rayleigh_taylor.rs` and `preview-kit.tsx` are unchanged.

## Known Stubs

None.

## Next Phase Readiness

Plan 05 can grow the protected baseline from capture `2026-10-19T03-11-46Z` at e495c30f5, provided `crates tools web` are unchanged since then (only `.planning/` changes after this plan).

## Self-Check: PASSED

- FOUND: docs/assets/readme/rayleigh-taylor-10s.svg
- FOUND: docs/assets/readme/rayleigh-taylor-10s.webp
- FOUND: commit e495c30f5
- FOUND: target/phase49/fingerprint-gate.log (PASS, 28/28, rayleigh-taylor added)
- FOUND: target/phase49/checks-04.log (ALL CHECKS PASSED, exit 0)
