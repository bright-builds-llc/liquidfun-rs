---
phase: 47-archimedes-screw
plan: "04"
subsystem: readme-gallery-and-phase-gates
tags: [scene, archimedes-screw, readme-svg, webp, fingerprint-gate, scene-spot, gates]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 47-2026-10-14T20-01-42
generated_at: "2026-10-14T22:42:49.207Z"
requires:
  - "47-01: screw scene, geometry and release spike tests"
  - "47-02: live screw-speed / screw-direction controls, DEFAULT 30 / MAX 60 rpm, README timing test (41 scooped particles by step 597)"
  - "47-03: catalog registration, README plan { id: archimedes-screw, cues: [] }, SURVEY_SCENES Default"
provides:
  - "docs/assets/readme/archimedes-screw-10s.svg and .webp plus the README gallery entry"
  - "docs/adding-a-scene.md: R13 widened to release-only spike/flow tests, new R14 (include_str! web/Rust constant sync tests), W3/W4 names the panel runtimePreset second live control"
  - "Phase 47 gate evidence: 15/15 local checks, screw scene-spot numbers, id-join fingerprint gate PASS 26/26 with archimedes-screw added"
  - "Full-catalog capture target/dam-break-perf/2026-10-14T22-42-42Z/scene-spot.json at 23541975c for Plan 05's baseline growth"
affects: [47-05]
tech-stack:
  added: []
  patterns:
    - "Numeric SVG keyframe scan for per-sample line jumps, to separate real geometry jumps from recorder index reassignment"
key-files:
  created:
    - docs/assets/readme/archimedes-screw-10s.svg
    - docs/assets/readme/archimedes-screw-10s.webp
  modified:
    - README.md
    - docs/adding-a-scene.md
key-decisions:
  - "47-04: default rpm stays 30 (max 60); the reviewed clip shows pockets climbing the full tube, filling the basin and overflowing down the chute before 10 s, so the rpm files are untouched"
  - "47-04: 25 other scenes' WebPs re-encoded with different bytes on this host and were restored (cartesian-diver re-encoded byte-identical); every other SVG was byte-identical"
  - "47-04: screw scene-spot (stamp 2026-10-14T22-42-14Z) start 1729, end 1729, median 0.688 ms/step; full-catalog gate PASS 26/26 unchanged with archimedes-screw 86a28d366447bbeb added (capture 2026-10-14T22-42-42Z at 23541975c)"
requirements-completed: []
duration: about 24 min
completed: 2026-10-14
---

# Phase 47 Plan 04: README Preview and Phase Gates Summary

**The README gallery now shows a reviewed 10 s Archimedes Screw clip (SVG plus 60 fps WebP) at the default 30 rpm with no cues. Pockets climb the tube between slanted vanes, spill into the basin and overflow down the chute. All 15 local gates pass on the shipped tree. The id-join fingerprint gate reports 26/26 protected scenes unchanged, with `archimedes-screw` added. The screw uses 1,729 particles at a median of 0.688 ms/step in native release.**

## Performance

- **Duration:** about 24 min
- **Started:** 2026-10-14T22:19:13Z
- **Completed:** 2026-10-14T22:43Z
- **Tasks:** 2 of 2
- **Files:** 2 created, 2 modified

## Accomplishments

### Task 1 (23541975c): README preview, clip review and checklist

- `LIQUIDFUN_README_FONT_FILE=/usr/local/texlive/2025/texmf-dist/fonts/truetype/public/dejavu/DejaVuSans.ttf just readme-svg` (log `target/phase47/readme-svg.log`) finished on the first attempt with `updated 1 SVG file and 26 WebP files and the README gallery`.
- Only the screw's SVG and WebP were new; every other scene's SVG was byte-identical. The 25 other changed WebPs (host re-encode) were restored with `git checkout --`, listed in `target/phase47/restored-assets.txt`. The cartesian-diver WebP re-encoded byte-identically, because Phase 46 committed it from this host.
- README gallery entry: README lines 200-204, between the markers (begin line 35, end line 206 after the insert), with the heading link `#/scene/archimedes-screw` and alt text `Archimedes Screw simulation preview`.
- `docs/adding-a-scene.md`:
  - **R13** now covers release-only `spike_tests.rs` and `flow_tests.rs` as well as `motion_tests.rs`.
  - New **R14** (optional) covers a Rust test that reads a web file with `include_str!` and compares copied constants as parsed `f32` bits.
  - **W3/W4** now names the `<id>-speed.ts` file, a panel `runtimePreset` as a second live control, and the rule that `DEFAULT_PRESET_VALUES` is needed only when the default is not the first value.
  - The file was formatted with the pipx mdformat 1.0.0 (gfm and frontmatter plugins), and `just markdown-check` exits 0.

### Clip review (D-17, D-18)

Frames were extracted with `webpmux -get frame N` and `dwebp`; the PNGs are in `target/phase47/readme-frame-*.png`. Every frame is 1280 x 960, full-canvas, not blended, 600 frames.

| Frame (time) | Observation |
|---|---|
| 61 (1.0 s) | The whole loop is in frame: reservoir at lower left, the 30 degree tube, the basin and weir at upper right, the chute down the right side, and the return ramp. Separate pockets sit between the slanted vanes. The pre-filled top pocket is tipping out of the outlet toward the empty basin. |
| 241 (4.0 s) | A thin layer of water covers the basin floor. Five pockets climb the tube, each behind a vane, and the reservoir still covers the inlet. |
| 421 (7.0 s) | The basin is about half full. A pocket is pouring over the outlet into the basin, and two drops are falling in the chute below it. |
| 591 (9.83 s) | The basin is full to the weir lip and overflowing: drops fall down the chute toward the ramp. Pockets keep climbing. Water from the reservoir has climbed the full tube and spilled into the basin before the clip ends. |

- No particle appears inside a wall, housing or vane in any reviewed frame. The 47-02 release flow tests assert zero inside walls and vanes.
- Vane continuity: consecutive 20 Hz samples across one 2 s pitch (frames 241 to 361 every 6 frames, `target/phase47/seq/montage.png`) and frames 7-10 around a snap (`target/phase47/sweep/m.png`) show the vanes sliding into and out of the housings with no visible jump in the WebP.
- The 47-02 release test `README_SCOOPED_MIN` backs the spill claim numerically: 41 reservoir particles reach the basin by step 597.
- **Final rpm:** DEFAULT 30, MAX 60, unchanged. `crates/liquidfun-wasm/src/scene/archimedes_screw.rs`, `web/src/catalog/archimedes-screw-speed.ts` and `web/tests/controls.test.ts` are not modified.

### Task 2: phase gates and recorded numbers (no tracked changes, no commit)

`target/phase47/checks-04.sh` ran once on the final tree, at HEAD 23541975c with only `.planning/config.json` dirty. All 15 steps exited 0 on the first run (`target/phase47/checks-04-run1.log`, with per-step logs in `target/phase47/checks-04-logs/`):

| # | Gate | Result |
|---|---|---|
| 1 | `cargo fmt --all --check` | pass |
| 2 | `cargo clippy --all-targets --all-features -- -D warnings` | pass |
| 3 | `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings` | pass |
| 4 | `cargo clippy -p xtask --all-targets --all-features -- -D warnings` | pass |
| 5 | `cargo build --all-targets --all-features` | pass |
| 6 | `cargo test --all-features` | 1084 passed, 0 failed |
| 7 | `cargo test -p liquidfun-wasm --all-features` | 681 passed, 52 ignored (release-only) |
| 8 | `cargo test -p liquidfun-wasm --release --lib archimedes_screw` | 62 passed, 0 ignored; all 16 spike and flow tests ran |
| 9 | `cargo test -p xtask --bin xtask playground::` | 104 passed |
| 10 | `cd web && bun run typecheck && bun run test:unit` | 120 files, 1852 tests passed |
| 11 | `bun scripts/bright-builds-check.ts all` | findings=0 |
| 12 | `just web-build` | pass |
| 13 | `CI=true bunx playwright test e2e/player.spec.ts e2e/shell.spec.ts e2e/hud-safe-frame.spec.ts --project chromium` | 98 passed |
| 14 | `just markdown-check` (pipx mdformat first on PATH) | pass |
| 15 | `git diff --quiet -- docs/benchmarks/scene-fingerprints.json crates/liquidfun` | pass |

**Screw scene-spot (D-21, SCENE-02):** `cargo xtask playground scene-spot --scene archimedes-screw` (log `target/phase47/scene-spot-screw.log`), stamp `target/dam-break-perf/2026-10-14T22-42-14Z/scene-spot.json` (scene_filter `["archimedes-screw"]`, full_catalog false):

| Scene | start_particles | end_particles | median_ms_per_step | min / max ms/step | fingerprint |
|---|---:|---:|---:|---|---|
| archimedes-screw | 1729 | 1729 | 0.688098 | 0.687 / 0.697 | 86a28d366447bbeb |

The count of 1,729 is within the 3,000 budget, and start equals end, so nothing was evicted. In the full-catalog capture below, the same scene reads 1729/1729 and a median of 0.682380 ms/step, with the same fingerprint.

**Full-catalog capture and id-join gate (D-21):** `just playground-scene-spot` (log `target/phase47/scene-spot-full.log`) wrote `target/dam-break-perf/2026-10-14T22-42-42Z/scene-spot.json` (27 scenes, full_catalog true). The capture commit was 23541975cd276880934f0044c732915d8c88d287 (`target/phase47/full-capture-commit.txt`). Plan 05 may reuse this capture only while `crates`, `tools` and `web` are unchanged since that commit; the commits after it change only `.planning/**`.

Command: `just scene-fingerprint-gate --spot target/dam-break-perf/2026-10-14T22-42-42Z/scene-spot.json` (log `target/phase47/fingerprint-gate.log`, exit 0):

```
Scene fingerprint gate: PASS
- protected: 26/26 unchanged
- changed: none
- missing: none
- added (accepted): archimedes-screw 86a28d366447bbeb
- baseline: docs/benchmarks/scene-fingerprints.json (captured at 89983a79b546153cb61b8bbd5a5dadafe024fec1, 2026-10-14T19-29-46Z)
- observed: target/dam-break-perf/2026-10-14T22-42-42Z/scene-spot.json (git 23541975cd276880934f0044c732915d8c88d287, full_catalog true)
```

`docs/benchmarks/scene-fingerprints.json` is unchanged. `git log -1` on it still shows 18c8a5e92 (`chore(46-05): protect the cartesian-diver scene fingerprint`).

## Phase decision coverage (D-01..D-23)

| Decision | Plan / task |
|---|---|
| D-01, D-02, D-03 (kinematic vane train, candidate A, no in-water appear/teleport) | 47-01 T1/T2 (spike P1-P5); release spike tests in gate step 8 here |
| D-04 (clearance and tunneling guards) | 47-01 T2, 47-02 (flow tests, 0 inside walls/vanes) |
| D-05 (slanted vanes) | 47-01; visible in the clip review here |
| D-06, D-07, D-08 (closed tube, weir basin, water sizing) | 47-01 geometry tests; basin overflow visible at frame 591 here |
| D-09, D-10 (screw-speed HUD spinner, screw-direction preset) | 47-02 (Rust controls), 47-03 (web controls and tests) |
| D-11 (no scene pointer behavior) | 47-03 (POINTER_CONTROL clicks the Screw speed control) |
| D-12, D-13 (toy scale, wide frame) | 47-01, 47-03 (view bounds, portrait frame); 1,729 particles recorded here |
| D-14 (optional shaft decoration) | 47-01 (discretion) |
| D-15, D-16 (id, family, honest caption) | 47-03 T1/T2 |
| D-17 (watch-first README and survey, no cues) | 47-02 (README timing test), 47-03 (plans), 47-04 T1 (clip review, rpm kept) |
| D-18 (every checklist site, README generated and reviewed) | 47-03, 47-04 T1 (assets, review, checklist rows R13/R14/W3-W4) |
| D-19 (release flow tests) | 47-02; gate step 8 here (62/62, none ignored) |
| D-20 (gravity 2 and 80 smoke) | 47-02; gate step 8 here |
| D-21 (gate 26/26 plus screw numbers) | 47-04 T2 |
| D-22 (baseline growth in its own commit) | 47-05 (pending) |
| D-23 (normal local checks) | Every plan; full set in 47-04 T2 |

## Decisions Made

- The default rpm stays 30, because the clip shows pockets reaching the basin and the basin overflowing before 10 s.
- Unrelated host re-encodes of other scenes' WebPs are not committed. Their SVGs are byte-identical, and the canonical encode happens in the post-merge workflow.

## Deviations from Plan

None in the plan's actions. Notes:

- Beyond the four required frames, the review also checked vane continuity across one full pitch, plus a numeric scan of the SVG keyframes. The scan found that the README SVG recorder reassigns `<line>` indices when the clipped vane outlines change, so the standalone animated SVG sweeps a line for 50 ms about 4 times per pitch. The same happens in washing-machine (2,936 jumps), hydraulic-fountain and stacked-drip, so it predates this phase and is out of scope. It is logged in `deferred-items.md`. The gallery WebP renders every step and is clean.
- Task 2 changed no tracked files, so it has no task commit, as the plan expected.

## Known Stubs

None.

## Next Phase Readiness

Plan 05 can grow `docs/benchmarks/scene-fingerprints.json` with `archimedes-screw` `86a28d366447bbeb` in its own commit, from the capture `target/dam-break-perf/2026-10-14T22-42-42Z/scene-spot.json` (git 23541975c). It should do so only if no code has changed since that commit; otherwise it should take a fresh capture. It should then rerun the gate and expect 27/27 protected unchanged.

## Self-Check: PASSED

- FOUND: docs/assets/readme/archimedes-screw-10s.svg, docs/assets/readme/archimedes-screw-10s.webp (both tracked)
- FOUND: commit 23541975c
- FOUND: target/phase47/fingerprint-gate.log with PASS, 26/26 unchanged, archimedes-screw added
- FOUND: target/dam-break-perf/2026-10-14T22-42-14Z/scene-spot.json and target/dam-break-perf/2026-10-14T22-42-42Z/scene-spot.json
