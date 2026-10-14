---
phase: 46-cartesian-diver
plan: "04"
subsystem: readme-gallery-and-phase-gates
tags: [scene, cartesian-diver, readme-svg, webp, fingerprint-gate, scene-spot, gates]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 46-2026-10-14T17-38-05
generated_at: "2026-10-14T19:33:00Z"
requires:
  - "46-01: diver scene and README cue samples 20/110"
  - "46-02: catalog registration, README plan entry, SURVEY_SCENES Action(squeeze)"
  - "46-03: scene canvas long-press suppression and canvas-callout e2e"
provides:
  - "docs/assets/readme/cartesian-diver-10s.svg and .webp plus the README gallery entry"
  - "docs/adding-a-scene.md rows R13 (release-only motion tests) and the W6 view-bounds re-export chain"
  - "Phase 46 gate evidence: 15/15 local checks, diver scene-spot numbers, id-join fingerprint gate PASS 25/25 with cartesian-diver added"
  - "Full-catalog capture target/dam-break-perf/2026-10-14T19-29-46Z/scene-spot.json at 89983a79b for Plan 05's baseline growth"
affects: [46-05]
tech-stack:
  added: []
  patterns:
    - "Animated WebP frame review with the installed webpmux + dwebp (ffmpeg 8.1.1 cannot decode animated WebP)"
    - "SVG keyframe extraction of the diver circle cy, calibrated by its 87 px radius (0.6 m -> 145 px/m)"
key-files:
  created:
    - docs/assets/readme/cartesian-diver-10s.svg
    - docs/assets/readme/cartesian-diver-10s.webp
  modified:
    - README.md
    - docs/adding-a-scene.md
key-decisions:
  - "46-04: README cues stay at samples 20 and 110; the generated clip reaches the bottom third 1.1 s after the first cue and the top third 2.2 s after the second, so plans.ts, readme-svg.test.ts and motion_tests.rs are untouched"
  - "46-04: other scenes' WebPs re-encoded with different bytes on this host (local ffmpeg 8.1.1 and TeXLive DejaVu Sans); all their SVGs were byte-identical, so the 25 WebPs were restored and not committed"
  - "46-04: diver scene-spot (stamp 2026-10-14T19-29-18Z) start 1936, end 1936, median 0.582 ms/step; full-catalog gate PASS 25/25 unchanged with cartesian-diver fa849f40b8c2a791 added (capture 2026-10-14T19-29-46Z at 89983a79b)"
requirements-completed: [SCENE-01, SCENE-02]
duration: about 27 min
completed: 2026-10-14
---

# Phase 46 Plan 04: README Preview and Phase Gates Summary

**The README gallery now shows a reviewed 10 s Cartesian Diver clip (SVG plus 60 fps WebP). The diver sinks to the bottom third and rises back to the top third, and the cues are unchanged. All 15 local gates pass on the shipped tree. The id-join fingerprint gate reports 25/25 protected scenes unchanged, with `cartesian-diver` added. The diver uses 1,936 particles at a median of 0.582 ms/step in native release.**

## Performance

- **Duration:** about 27 min
- **Started:** 2026-10-14T19:04:40Z
- **Completed:** 2026-10-14T19:33Z
- **Tasks:** 2 of 2
- **Files:** 2 created, 2 modified

## Accomplishments

### Task 1 (89983a79b): README preview, clip review and checklist

- `just readme-svg` failed on the first attempt (`target/phase46/readme-svg.log`) because the canonical font `/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf` exists only on the Linux CI image. Attempt 2 (`target/phase46/readme-svg-attempt2.log`) used the documented override `LIQUIDFUN_README_FONT_FILE=/usr/local/texlive/2025/texmf-dist/fonts/truetype/public/dejavu/DejaVuSans.ttf` and finished with `updated 1 SVG file and 26 WebP files and the README gallery`.
- Only the diver's SVG was new, so every other scene's SVG export was byte-identical. The 25 other WebPs had different bytes because they were re-encoded on this host with ffmpeg 8.1.1 and the TeXLive font. That change is unrelated to this phase, so they were restored with `git checkout --` (list in `target/phase46/restored-webps.txt`). The post-merge Readme scene preview workflow re-encodes every WebP on the canonical image.
- README gallery entry: between the gallery markers (begin line 35, entry line 196, end line 200), with the heading link `#/scene/cartesian-diver` and the alt text `Cartesian Diver simulation preview`.
- `docs/adding-a-scene.md` gains row **R13**, which records the release-only `motion_tests.rs` pattern and its `cfg_attr(debug_assertions, ignore = ...)` gating. Row **W6** now names the view-bounds re-export chain (`scene-view-bounds.ts` -> `scene-records.ts` -> `scenes.ts`) that the portrait frame needs. Formatting used the pipx mdformat 1.0.0 with gfm and frontmatter plugins. The Homebrew `mdformat` on PATH has no plugins and mangles tables. Running `just markdown-check` with `~/.local/bin` first on PATH exits 0.

### Clip review (D-17, D-18)

ffmpeg 8.1.1 cannot decode the animated WebP (`dec:webp` error). Every frame is full-canvas, not blended and disposed to background (`webpmux -info`), so the review extracted exact frames with the installed `webpmux -get frame N` and `dwebp`, with no new dependency. The plan's 0-based frames 10, 200, 320 and 590 are 1-based frames 11, 201, 321 and 591. PNGs are in `target/phase46/readme-frame-*.png`.

| Frame (time) | Observation |
|---|---|
| 11 (0.17 s, rest) | Straight bottle walls fully in frame. Diver at the top, touching the water surface, with a large bubble. Water fills the bottle to just below the cap. |
| 201 (3.3 s, squeezed) | Side walls bowed inward at mid-height. Diver resting on the floor with a small bubble. The water is still churning from the sink. |
| 321 (5.3 s, still squeezed) | Bowed walls. Diver still on the floor with a small bubble. The water has settled lower, with a sloped surface. |
| 591 (9.8 s, risen) | Straight walls again. Diver back at the top, floating on the surface with a large bubble. |

SVG keyframes give the diver circle's center y per 20 Hz sample. The scale is calibrated from its 87 px radius = 0.6 m (145 px/m), anchored at sample 0 = `DIVER_START` y 4.6. The floor rest value of 0.605 m matches the radius.

| Sample | 0 | 20 (cue 1) | 40 | 42 | 60-110 | 110 (cue 2) | 130 | 150 | 154 | 160 | 180 | 190 | 200 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| y (m) | 4.60 | 3.44 | 2.29 | < 1.867 | 0.605 (floor) | 0.605 | 1.99 | 3.23 | > 3.733 | 4.31 | 3.41 | 4.11 | 3.85 |

- Bottom third (y < 1.867) first reached at sample 42 (2.10 s), 1.10 s after the first cue.
- Top third (y > 3.733) first reached at sample 154 (7.70 s), 2.20 s after the second cue. Samples 190-200 stay at or above 3.85 m, so the clip ends with the diver risen. The 3.4 m dips at samples 20 and 180 are the surface bob, which 46-01 also measured at rest (3.443).
- **Final cues:** squeeze at samples 20 and 110 (steps 60 and 330), unchanged. `web/scripts/readme-svg/plans.ts`, `web/tests/readme-svg.test.ts` and `crates/liquidfun-wasm/src/scene/cartesian_diver/motion_tests.rs` are not modified.

### Task 2: phase gates and recorded numbers (no tracked changes, no commit)

`target/phase46/checks-04.sh` ran once on the final tree, at HEAD 89983a79b with only `.planning/config.json` dirty. All 15 steps exited 0 on the first run (`target/phase46/checks-04-run1.log`, with per-step logs in `target/phase46/checks-04-logs/`):

| # | Gate | Result |
|---|---|---|
| 1 | `cargo fmt --all --check` | pass |
| 2 | `cargo clippy --all-targets --all-features -- -D warnings` | pass |
| 3 | `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings` | pass |
| 4 | `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | pass |
| 5 | `cargo build --all-targets --all-features` | pass |
| 6 | `cargo test --all-features` | 1084 passed, 0 failed |
| 7 | `cargo test -p liquidfun-wasm --all-features` | 634 passed, 36 ignored (release-only) |
| 8 | `cargo test -p liquidfun-wasm --release --lib cartesian_diver` | 44 passed, 0 ignored (motion tests ran) |
| 9 | `cargo test -p xtask --bin xtask playground::` | 104 passed |
| 10 | `cd web && bun run typecheck && bun run test:unit` | 120 files, 1849 tests passed |
| 11 | `bun scripts/bright-builds-check.ts all` | findings=0 (file-lengths 0) |
| 12 | `just web-build` | pass |
| 13 | `CI=true bunx playwright test e2e/player.spec.ts e2e/shell.spec.ts e2e/hud-safe-frame.spec.ts e2e/canvas-callout.spec.ts --project chromium` | 97 passed |
| 14 | `just markdown-check` | pass |
| 15 | `git diff --quiet -- docs/benchmarks/scene-fingerprints.json crates/liquidfun` | pass |

**Diver scene-spot (D-19, SCENE-02):** `cargo xtask playground scene-spot --scene cartesian-diver` (log `target/phase46/scene-spot-diver.log`), stamp `target/dam-break-perf/2026-10-14T19-29-18Z/scene-spot.json` (scene_filter `["cartesian-diver"]`, full_catalog false):

| Scene | start_particles | end_particles | median_ms_per_step | min / max ms/step | fingerprint |
|---|---:|---:|---:|---|---|
| cartesian-diver | 1936 | 1936 | 0.582406 | 0.576 / 0.587 | fa849f40b8c2a791 |

The count of 1,936 is within the 3,000 budget. In the full-catalog capture below, the same scene reads 1936/1936 and a median of 0.575207 ms/step with the same fingerprint.

**Full-catalog capture and id-join gate (D-19):** `just playground-scene-spot` (log `target/phase46/scene-spot-full.log`) wrote `target/dam-break-perf/2026-10-14T19-29-46Z/scene-spot.json` (26 scenes, full_catalog true). The capture commit was 89983a79b546153cb61b8bbd5a5dadafe024fec1 (`target/phase46/full-capture-commit.txt`). Plan 05 may reuse this capture only while HEAD code is unchanged since that commit; this SUMMARY commit changes only `.planning/**`.

Command: `just scene-fingerprint-gate --spot target/dam-break-perf/2026-10-14T19-29-46Z/scene-spot.json` (log `target/phase46/fingerprint-gate.log`, exit 0):

```
Scene fingerprint gate: PASS
- protected: 25/25 unchanged
- changed: none
- missing: none
- added (accepted): cartesian-diver fa849f40b8c2a791
- baseline: docs/benchmarks/scene-fingerprints.json (captured at 04650ff5ade210a0cbaf9770f38ef6fbfebad54a, 2026-10-14T15-53-11Z)
- observed: target/dam-break-perf/2026-10-14T19-29-46Z/scene-spot.json (git 89983a79b546153cb61b8bbd5a5dadafe024fec1, full_catalog true)
```

`docs/benchmarks/scene-fingerprints.json` is unchanged. `git log -1` on it still shows f31ac087a (`docs(45-01): pin the 25 scene fingerprint baseline`).

## Phase decision coverage (D-01..D-21)

| Decision | Plan / task |
|---|---|
| D-01, D-02, D-03 (squeeze intent, hit region, panel action) | 46-01 T1; panel action wired in 46-02 T1 |
| D-04, D-05, D-06 (circle diver, density switch, ramp and damping) | 46-01 T1 (mechanism), T2 (tuned values) |
| D-07, D-07a (timing at g 2/10/80, water-column rise at g 80) | 46-01 T2 (11 release motion tests; gate step 8 here: 44/44, none ignored) |
| D-08 (engine forces only) | 46-01 T1 |
| D-09, D-10 (draw-only dent and bubble) | 46-01 T1; visible in the clip review here |
| D-11 (toy scale, closed bottle, budget) | 46-01 T1/T2; 1,936 particles recorded here |
| D-12 (portrait and landscape frames) | 46-02 T1/T2 |
| D-13 (no R6/R12 arms unless needed) | 46-01 T2 (none added) |
| D-14 (long-press callout suppression) | 46-03 T1/T2; canvas-callout e2e in gate step 13 here |
| D-15, D-16 (id, family, honest caption and hint) | 46-02 T1/T2 |
| D-17 (README cues, survey, demo-media, POINTER_CONTROL) | 46-01 T2 (cue samples), 46-02 T1/T2, 46-04 T1 (clip check, cues kept) |
| D-18 (every checklist site, README generated and reviewed) | 46-02, 46-04 T1 (assets, review, checklist rows R13/W6) |
| D-19 (gate 25/25 plus diver numbers) | 46-04 T2 |
| D-20 (baseline growth in its own commit) | 46-05 (pending) |
| D-21 (normal local checks) | Every plan; full set in 46-04 T2 |

## Decisions Made

- The cues stay at samples 20 and 110, because the clip shows a full sink and a full rise before 10 s.
- Unrelated host re-encodes of other scenes' WebPs are not committed. Their SVGs are byte-identical, and the canonical encode happens in the post-merge workflow.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] README WebP font missing on macOS**
- **Found during:** Task 1, step 1
- **Issue:** `just readme-svg` exited 1 with `README WebP font is missing at /usr/share/fonts/truetype/dejavu/DejaVuSans.ttf`.
- **Fix:** Reran with the documented `LIQUIDFUN_README_FONT_FILE` override pointing at an installed DejaVu Sans (TeXLive copy). No code change. The failed log is kept as `target/phase46/readme-svg.log`.
- **Commit:** none (environment only)

**2. [Rule 3 - Blocking] ffmpeg cannot decode the animated WebP**
- **Found during:** Task 1, clip review
- **Issue:** The plan's ffmpeg frame extraction failed in the `dec:webp` decoder.
- **Fix:** Used the already-installed libwebp tools (`webpmux -get frame`, `dwebp`). No dependency was added, and the method is exact because every frame is full-canvas and not blended. The SVG keyframes were also checked numerically.
- **Commit:** none (review method only)

**3. [Rule 3 - Blocking] Plugin-less Homebrew mdformat shadows the gfm install**
- **Found during:** Task 1, checklist edit
- **Issue:** `/opt/homebrew/bin/mdformat` comes first on PATH and has no gfm or frontmatter plugins, so `just markdown-check` flags 19 unrelated files and would reformat tables wrongly.
- **Fix:** The checklist edit was reverted and redone, then formatted with `~/.local/bin/mdformat` (mdformat 1.0.0 with mdformat-gfm 1.0.0 and mdformat_frontmatter 2.1.2). `checks-04.sh` puts `~/.local/bin` first on PATH. No repository change.
- **Commit:** 89983a79b (checklist content only)

### Other notes

- The plan names frames 10, 200, 320 and 590 (0-based `n`). The webpmux 1-based equivalents 11, 201, 321 and 591 were reviewed.
- Task 2 changed no tracked files, so it has no task commit, as the plan expected.

## Known Stubs

None.

## Next Phase Readiness

Plan 05 can grow `docs/benchmarks/scene-fingerprints.json` with `cartesian-diver` `fa849f40b8c2a791` in its own commit, from the capture `target/dam-break-perf/2026-10-14T19-29-46Z/scene-spot.json` (git 89983a79b). It should do so only if no code has changed since that commit; otherwise it should take a fresh capture. It should then rerun the gate and expect 26/26 protected unchanged.

## Self-Check: PASSED

- FOUND: docs/assets/readme/cartesian-diver-10s.svg, docs/assets/readme/cartesian-diver-10s.webp (both tracked)
- FOUND: commit 89983a79b
- FOUND: target/phase46/fingerprint-gate.log with PASS, 25/25 unchanged, cartesian-diver added
- FOUND: target/dam-break-perf/2026-10-14T19-29-18Z/scene-spot.json and target/dam-break-perf/2026-10-14T19-29-46Z/scene-spot.json
