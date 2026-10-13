---
gsd_state_version: 1.0
milestone: v1.5
milestone_name: milestone
status: executing
stopped_at: Completed 39-07-PLAN.md
last_updated: "2026-10-13T00:59:26.079Z"
last_activity: 2026-10-13
progress:
  total_phases: 7
  completed_phases: 2
  total_plans: 31
  completed_plans: 24
  percent: 77
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-10-11)

**Core value:** Deliver a useful, independent Rust physics library for enjoyable experimentation, with honest limitations and a lightweight native development loop.
**Current focus:** Phase 39 — Place-then-Play Tools and Stars

## Current Position

Phase: 39 (Place-then-Play Tools and Stars) — EXECUTING
Plan: 8 of 14
Status: Ready to execute
Last activity: 2026-10-13

Progress: [░░░░░░░░░░] 0%

v1.4 is archived, and annotated tag `v1.4` marks the archive, authorized by the owner on 2026-10-11. v1.3 also remains archived. Package publication remains separately authorized.

## Accumulated Context

### Roadmap Evolution

- v1.5 (2026-10-11): Roadmap created with Phases 37–43 — puzzle core in the bridge, puzzle route and player shell, tools and stars, timed valve and teaching levels, level select and progress, opt-in sound, GPU water (closing).
- v1.4 (2026-10-06): Simplified to Phases 34–36 — timing survey, targeted fixes for the slowest scenes, re-survey. The original Phases 34–60 plan is in commit `011e80e`.
- Phase 30 added: Periodic hydraulic fountain
- Phase 31 added: Sinusoidal wave tank
- Phase 32 added: Liquid motion bubbler
- Phase 33 added: Stacked drip fidget

### Decisions

v1.3 decisions are in `.planning/milestones/v1.3-STATE.md` and PROJECT.md Key Decisions.

- [v1.3]: Archive is a planning label. No crate publication. Annotated tag `v1.3` marks the archive.
- [v1.4]: Speed takes priority over compact memory; bounded measured increases are allowed. Preserve authored physics/settings, ordering, visuals and lifecycle; no promised percentage/FPS gain.
- [v1.4]: Simplified 2026-10-06. Survey all scenes, fix only the slowest few with profile-backed before/after gains; keep measurement out of production scene code. The Phase 34 harness (plans 1–9, ~150k lines, ~3.8 GB evidence) is shelved on `wip/v1.4-phase34-harness`; evidence moved to `~/Archives/liquidfun-rs-v1.4-phase34-evidence/`.
- The recorded ≤ 3× pair stays bound to git `89d34564`. A fresh unprofiled pair is required before claiming that ratio for later HEAD.
- [Phase 34]: Scene survey SCENE_WALL_TIMEOUT applies per run; start_particles read before the survey cue; even-run median averages middle values
- [Phase 34]: Recorded scene survey: stamp 2026-10-07T06-20-54Z on 96a3ac6a6; top 5 liquid-tumbler, tesla-valve, stacked-drip, washing-machine, particles
- [Phase 35]: Survey fingerprint is FNV-1a 64 over f32 bits (particles, color lane marker, bodies) after the first run; runs in one invocation must agree
- [Phase 35]: Survey scene filter (repeatable --scene) in bin and xtask; unknown or duplicate ids fail closed before any stamp
- [Phase 35]: Phase before binary target/phase35/bin/spot-before at 809582431; every keep decision uses ABBA runs of saved binaries plus keep_rule.py against before-full.jsonl fingerprints
- [Phase 35]: Target hot paths: damping (liquid-tumbler 22.9%, stacked-drip 17.1%, particles 22.8% self), visit_sorted_tag_indices_in_aabb 13.7% (tesla-valve), full-scan CCD push_fixture_particle_hit 15.2% self / 29.1% incl (washing-machine); D-03 browser check not run
- [Phase 35]: 35-03 A1 kept (91b27a6d6): retained sorted contact-proxy order + sort_unstable_by_key((tag,row)); liquid-tumbler ~24.2-24.4 -> 23.3 ms/step in ABBA, gains also for stacked-drip and particles, 25/25 fingerprints equal
- [Phase 35]: 35-03 A2 reverted: bounded insertion sort beat A1 on liquid-tumbler/stacked-drip but particles and washing-machine regressed in both pairs; diff in target/phase35/attempts/A2.patch
- [Phase 35]: 35-04 A3 reverted (0a2fe8cb8, revert 7f0b36b18): chain child edge hoist cut liquid-tumbler 9.4%/10.4% with 25/25 fingerprints, but soup-stirrer (no chain) exceeded its base max in both isolated pairs
- [Phase 35]: 35-04 A3b reverted (never committed, attempts/A3b.patch): no-branch Shape::Edge variant cut liquid-tumbler 7.6%/7.3% with 25/25 fingerprints, but fountain (no chain) exceeded its base max in both isolated pairs; neither regression reproduced in a later diagnostic pair
- [Phase 35]: 35-05 A4 reverted: per-row AABB tag query gained on no target and regressed liquid-tumbler plus four scenes in both pairs (never committed; attempts/A4.patch)
- [Phase 35]: 35-05 A5 kept (216de3929): bitset walk for body-contact candidate rows, liquid-tumbler -3.0%/-3.0%, tesla-valve -0.9%/-2.2%, 25/25 fingerprints
- [Phase 35]: 35-06 A6 kept (2256dd8cd): verified conservative query pad for moving fixtures at iteration 0 replaces the 0..n CCD scan; washing-machine -29.0%/-28.9% in ABBA, soup-stirrer/water-wheel/theo-jansen faster, 25/25 fingerprints; A7 not tried
- [Phase 35]: 35-07 A8 kept (5695f4b39): ungrouped particle creation reuses group_records with the rebuild's empty-record normalization instead of the O(n) group-lane clone and rebuild; tesla-valve -7.8%/-12.2%, fountain and water-wheel about -20%, 25/25 fingerprints
- [Phase 35]: 35-07 A9 kept (f7041fc75): eviction index uses an in-tree deterministic ParticleIdHasher, in-place resequence and bulk BTreeMap builds; tesla-valve -9.1%/-8.1%, fountain and water-wheel about -27%, 25/25 fingerprints
- [Phase 35]: 35-08 close-out: final HEAD 202075791 has 25/25 fingerprints equal to the phase before; cumulative spot-before vs spot-final ABBA puts all five targets below the before min in both pairs (liquid-tumbler -5.8%/-8.5%, tesla-valve -19.7%/-14.6%, stacked-drip -9.9%/-6.4%, washing-machine -31.8%/-31.7%, particles -5.1%/-7.4%)
- [Phase 35]: 35-08: A3/A3b stay reverted under D-11; possible host-noise evidence and a retry idea recorded in 35-PROFILES.md Notes for Phase 36
- [Phase 36]: 36-01 WR-01 fixed (c4252c01b): filtered scene-spot stamps record scene_filter (request order) and full_catalog false with a requested-scenes-only disclaimer; full-catalog stamps keep the original disclaimer and add scene_filter [] and full_catalog true
- [Phase 36]: 36-02 v1.4 before/after survey published (e5640ea88): spot-before 809582431 vs spot-v14-final (crates/ = edcafebc0), one gated ABBA set over 25 scenes at --runs 5: faster 24, within noise 1 (drawing-particles), slower 0; 25/25 fingerprints in all six legs; current ranking from v14-rank-b1.jsonl
- [v1.5]: Owner decisions: the valve is timed and set while building (no live tapping); the elastic level is a "jelly pad" deflector whose particles never count; stars keep the best per criterion across runs; soft linear locks with deep links always open; storage blocked means all levels unlocked plus a note.
- [v1.5]: Levels live in their own registry (`LevelId` / `PUZZLE_IDS`), never `SCENE_IDS`. Outcome, timer and stars are decided per fixed step in Rust. No engine change may alter the 25 scene fingerprints; phases touching the bridge or frame contract require 25/25 identical fingerprints.
- [Phase 36]: 36-03 v1.4 leftovers recorded in REQUIREMENTS.md Left over from v1.4 (presortedness test, A7, world backup clone, emission reallocation, grouped prepare_create, wasm32 bins, support.rs:98 warning, stale web-smoke spec); PERF-10 Complete after survey commit e5640ea88; no v1.4 tag or archive (D-10)
- [Phase 37]: 37-01 spike (300 steps, base 958d6fad7): release-native == wasm for dam-break and water-wheel, theo-jansen diverges @70; spike_contract exact (release-native vs wasm)
- [Phase 37]: 37-01: native debug diverges from release (dam-break @91, water-wheel @39, theo-jansen @2); default debug cargo test replay asserts the fallback form; static tools use tau/24 baked vertices, no crates/liquidfun change
- [Phase 37]: 37-02: TS parses D-17 puzzle scalars into optional RenderFrame.maybePuzzle, fail closed (all-or-nothing, outcome 0/1/2, outcomeStep iff latched, loss at stepLimit); worker path stays scene-shaped until Phase 38
- [Phase 37]: 37-03: pure puzzle::goal core (strict GoalRect containment, generic count_in_goal, clamped fill, div_ceil(20) margin) and step-latched GoalTracker (win checked before loss, latch never flips); LevelId/PUZZLE_IDS registry proven disjoint from 25 SCENE_IDS (1b6887a67)
- [Phase 37]: 37-04: SessionCore builds from SessionSource::Scene and stores precomputed EscapeLimits (wave-tank 80/80, others 12/48); scene-spot stamp 2026-10-11T23-48-25Z gives 25/25 fingerprints identical to base (feb4804ff)
- [Phase 37]: 37-05: plank vertices from TAU_24 literals and integer 0.05 m grid poses (no trig); Placements::parse fails closed with 8 typed PlacementErrors and sorts canonically; FIRST_POUR const spec builds 1989 particles, 32 outline segments (36 with one plank) through generic build_level; PuzzleHooks counts per fixed step with its own counter (7ef3f738b)
- [Phase 37]: 37-06: puzzle sessions build from SessionSource::Puzzle(LevelId, Placements) at step 0; apply_control/set_gravity reject puzzles with UnknownControl, restore is an Ok no-op; first-pour unchanged: empty loses (max 0), plank [8,64,23] wins at 201, raised-target tracker at 210, identical debug/release; 25/25 fingerprints (b85ed6ec0, 52feaaaee)
- [Phase 37]: 37-06 Option A: won margin = second GoalTracker at target + full.div_ceil(20), same settle_steps, wins by step_limit - 120 (goal::maybe_margin_win_step); window-minimum rule needs a one-step inflow > margin that no pour gives; 37-08-PLAN.md updated
- [Phase 37]: 37-07: newPuzzle/levelInfo/PuzzleLevelInfo and six optional ProofFrame goal getters on the wasm surface; set_puzzle_status fails closed when hooks observed_step != frame step index (D-14); LEVEL_IDS drives parse_level_id; test-only puzzle helpers #[cfg(test)]; no temporary dead_code allows remain (bb34c4c04)
- [Phase 37]: 37-08: first-pour fixtures replay identically in release, debug and wasm (won 201 / 689, lost 720 / 0); release and wasm exact, debug fallback with DEBUG_OUTCOME_STEP_TOLERANCE 5; 14/14 final gates incl. 25/25 fingerprints (410e2111b, 74b85a09e)
- [Phase 39]: 39-01: placement status wire codes 0 Valid..12 OverlapsNoPlace; inspector and newPuzzle share placement_statuses/first_overlap (SAT, gap 2 x particle_radius, NaN fails closed)
- [Phase 39]: 39-01: first-pour budget 2 planks, par_tools 1, brim_count 810, no_place empty; builder rejects specs unless target < brim <= full and par <= budget
- [Phase 39]: 39-01: D-30 baseline target/phase39/base.txt from scene-spot stamp 2026-10-12T23-50-30Z (Rust == 424735672), identical to Phase 38 after-final
- [Phase 39]: 39-02: TS PLACEMENT_STATUS mirrors Rust 0..12; BuildingEditor is pure with live (tint/hit-test) and committed (copy/Play) inspections; applyInspection drops stale revisions and length mismatches
- [Phase 39]: 39-02: pickTool uses 22 CSS px slop, ties to selected then highest id; trayItems generic over kind strings (D-06); statusLine reads committed snapshot only (UI-D14)
- [Phase 39]: 39-03: star store is a per-runtime Map allowlisted to PUZZLE_IDS; snapshot() is a fresh null-prototype JSON-safe copy; a loss leaves the best (and bestPeakHeldPct) unchanged; merge rejects peak held % outside 0..100
- [Phase 39]: 39-03: puzzleKeyCommand is pure over a PuzzleKeyInput record; Shift (not letter case) sets rotate direction; Tab cycles only with target canvas; form controls, Ctrl/Meta/Alt and Space/Enter on buttons yield undefined
- [Phase 39]: 39-03: speed helpers multiply the per-frame step request by 1/2/4 and restore unrun steps as (unrun/speed)*STEP_SECONDS; MAX_STEPS_PER_FRAME stays 4
- [Phase 39]: 39-04: TS frame parses D-11 peakHeldCount/brimCount/brimStep/decidedStep fail closed (target < brim <= full, won iff peak >= target, Brim iff peak >= brim within outcomeStep..min(step, limit), decided per D-10); peak never bounded by particle count
- [Phase 39]: 39-04: puzzle parsing split into frame-puzzle.ts (type-only import of frame.ts) with shared helpers in frame-scalars.ts; frame.ts keeps all exported types
- [Phase 39]: 39-05: PeakHeldTracker beside the unchanged GoalTracker (running max of the settle-window minimum, frozen after step_limit, first-step Brim latch); decided_step = loss latch, Brim step, or step_limit for a win without Brim; 200-sequence test proves first peak >= target equals the GoalTracker win step
- [Phase 39]: 39-05: frame/puzzle.rs status_is_consistent is the single Rust rule set mirroring frame-puzzle.ts; D-07 measured reference peak held 541@201, 811@262, 1331@600, 1361@720, Brim 810 at step 262 (decided 262); fixtures unchanged (won 201/689, lost 720/0)
- [Phase 39]: 39-06: PuzzleLevelInfo parses brimCount, parTools, toolBudget (aligned allowlisted unique kinds, counts 1..16) and buildAreaCells fail-closed; target < brim <= full, 0 <= par <= total budget
- [Phase 39]: 39-06: decodeInspection fails closed on lane lengths, unknown codes and non-finite geometric outlines; non-geometric codes decode to undefined; inspectPlacements frees in finally
- [Phase 39]: 39-06: puzzle-loader caches one init promise; buildPuzzleSessionSync and inspectPlacementLanes throw 'Puzzle wasm is not initialized' before the first load
- [Phase 39]: 39-07: stop event carries stepIndex with status (like loaded) so non-step-0 previews throw; Plan 09 dispatches stop {stepIndex, status}
- [Phase 39]: 39-07: edit events live in edit-event.ts and HUD transport/tray/star helpers in hud-format.ts; reduce-puzzle.ts 359 lines, format.ts 268
- [Phase 39]: 39-07: canPlay gate means seeded-placement runs need a committed inspection on building entry (Plan 09); solution-win e2e cannot start a run until then

### Pending Todos

- Optional quick task (research recommendation, before or alongside Phase 38): accept `EXT_color_buffer_half_float` as well as `EXT_color_buffer_float` in `web/src/render/webgl-particles.ts`, relying on `checkFramebufferStatus`; verify on an iPhone. Otherwise fold into Phase 43.

### Blockers/Concerns

- Phase 37 opens with the native-vs-wasm determinism spike; its result sets the angle-snapping and fill-margin policy for every level.
- A fresh unprofiled pair is still required before claiming the ≤ 3× Dam Break ratio for any HEAD after `89d34564`.

## Retained Context

- Local checks plus one macOS CI job; expensive qualification stays optional/manual.
- No package publication or release tag.
- PLAT-01, PLAT-05 and DOCS-09 remain deferred in archived requirements.
- Historical decisions: `.planning/milestones/v1.3-STATE.md`, `.planning/milestones/v1.2-STATE.md`, `.planning/milestones/v1.1-STATE.md`, `.planning/milestones/v1.0-STATE.md`, `.planning/MILESTONES.md`.

### Quick Tasks Completed

| ID | Description | Date | Status | Directory |
| --- | --- | --- | --- | --- |
| 260921-tbx | Global wireframe stroke width slider from 0.1 to 1.5, default 0.3 | 2026-09-22 | complete | [260921-tbx](./quick/260921-tbx-global-wireframe-stroke-width-slider-fro/) |
| 261001-lhi | Resolve main pull conflicts and publish verified scene changes | 2026-10-01 | complete | [261001-lhi](./quick/261001-lhi-resolve-main-pull-conflicts-and-commit-s/) |
| 261001-tvf | Match Tesla Valve reference with alternating curved lobes and closed splitters | 2026-10-01 | complete | [261001-tvf](./quick/261001-tvf-reshape-tesla-valve-with-alternating-cur/) |
| 261001-uk9 | Widen Tesla Valve necks and measure higher flow | 2026-10-01 | complete | [261001-uk9](./quick/261001-uk9-widen-tesla-valve-stage-necks-and-verify/) |
| 261001-w0a | Smooth Tesla Valve curvature and refine particle flow | 2026-10-01 | complete | [261001-w0a](./quick/261001-w0a-smooth-tesla-valve-curvature-with-dense-/) |
| 261002-1yc | Circular Tesla Valve bends and uniform channel width | 2026-10-02 | complete | [261002-1yc](./quick/261002-1yc-make-tesla-valve-bends-circular-and-flow/) |
| 261002-ejw | Set Tesla Valve clear pipe width to six centimeters | 2026-10-02 | complete | [261002-ejw](./quick/261002-ejw-set-tesla-valve-clear-pipe-width-to-six-/) |
| 261002-f8p | Continue Tesla Valve loop stems into the main pipe | 2026-10-02 | complete | [261002-f8p](./quick/261002-f8p-continue-tesla-valve-loop-stems-into-the/) |
| 261002-irz | Five Tesla performance changes with immutable serial benchmark history | 2026-10-02 | complete | [261002-irz](./quick/261002-irz-optimize-tesla-valve-physics-and-worker-/) |
| 261010-ibo | Retry chain-edge hoist on final Phase 35 engine; both variants reverted under D-11 | 2026-10-10 | complete (reverted) | [261010-ibo](./quick/261010-ibo-retry-phase-35-chain-edge-hoist-a3-and-a/) |
| 261010-mrp | Calibrate D-11 keep rule with A/A runs; 1 of 3 no-op rounds falsely rejected; proposed floor F = 4% | 2026-10-10 | complete | [261010-mrp](./quick/261010-mrp-calibrate-d-11-keep-rule-with-a-a-runs-o/) |
| 261010-npx | Amend D-11 with 4% noise floor; fresh A3r2 chain-edge hoist kept (liquid-tumbler -9 to -11%) | 2026-10-11 | complete (kept, 154ace4bf) | [261010-npx](./quick/261010-npx-amend-d-11-with-a-4-percent-noise-floor-/) |
| 261010-rif | Re-judge A2 bounded insertion sort under amended D-11; kept (liquid-tumbler -5 to -6%, stacked-drip -6 to -11%; particles +7.4%/+2.6% within floor) | 2026-10-11 | complete (kept, edcafebc0) | [261010-rif](./quick/261010-rif-re-judge-a2-bounded-insertion-sort-under/) |
| 261010-vzz | HUD safe frame: slider show/hide toggle and chrome-aware camera fit (no fallback cases; Tesla valve phone frame smaller, follow-up noted) | 2026-10-11 | complete (627fe13e3) | [261010-vzz](./quick/261010-vzz-hud-safe-frame-slider-toggle-and-chrome-/) |
| 261010-wkd | Distinct HUD slider toggle icon (panel chevron) and Tesla valve portrait frame fitted to its walls (+19.9% px/m at 390x844) | 2026-10-11 | complete (6bc85b040) | [261010-wkd](./quick/261010-wkd-distinct-hud-slider-toggle-icon-and-tigh/) |
| 261011-n6v | Fix flaky Pages CI timeout: static-layer pixel comparisons use a fast mismatch helper instead of deep equality (about 1 s to under 10 ms per test) | 2026-10-11 | complete (4e23b88cb) | [261011-n6v](./quick/261011-n6v-fix-flaky-static-layer-test-timeout-in-p/) |
| 261012-eaw | Fix flaky README WebP missing-encoder test: encoder start waits for spawn/error, so a missing ffmpeg rejects start instead of racing a frame write | 2026-10-12 | complete (7702cc6b3) | [261012-eaw](./quick/261012-eaw-make-readme-svg-webp-missing-encoder-tes/) |
| fast-261007 | Fix wasm32 particle join bound overflow that capped Drawing near ~1,600 particles (commit 04216676b) | 2026-10-07 | complete | - |

## Performance Metrics

| Plan | Duration | Tasks | Files |
| --- | --- | --- | --- |
| Phase 30 P01 | 41 min | 2 tasks | 5 files |
| Phase 30 P02 | 7 min | 2 tasks | 13 files |
| Phase 30 P03 | 5 min | 2 tasks | 1 files |
| Phase 31 P01 | 13 min | 2 tasks | 5 files |
| Phase 31 P02 | 5 min | 2 tasks | 13 files |
| Phase 31 P03 | 3 min | 2 tasks | 1 files |
| Phase 32 P01 | 20 min | 2 tasks | 5 files |
| Phase 32 P02 | 7 min | 2 tasks | 13 files |
| Phase 32 P03 | 4 min | 2 tasks | 1 files |
| Phase 32 P04 | 24 min | 2 tasks | 3 files |
| Phase 33 P01 | 21 min | 2 tasks | 6 files |
| Phase 33 P02 | 7 min | 2 tasks | 13 files |
| Phase 33 P03 | 3 min | 2 tasks | 1 files |
| Phase 33 P04 | 25 min | 2 tasks | 4 files |
| Phase 34 P01 | 26min | 2 tasks | 8 files |
| Phase 34 P02 | 5min | 2 tasks | 2 files |
| Phase 35 P01 | 445min | 2 tasks | 11 files |
| Phase 35 P02 | 40min | 2 tasks | 1 files |
| Phase 35 P03 | 732min | 2 tasks | 6 files |
| Phase 35 P04 | 659min | 2 tasks | 7 files |
| Phase 35 P05 | 422min | 3 tasks | 3 files |
| Phase 35 P06 | 478min | 2 tasks | 4 files |
| Phase 35 P07 | 411min | 2 tasks | 5 files |
| Phase 35 P08 | 84min | 2 tasks | 3 files |
| Phase 36 P01 | 39min | 2 tasks | 2 files |
| Phase 36 P02 | 15min | 3 tasks | 4 files |
| Phase 36 P03 | 1min | 1 tasks | 1 files |
| Phase 37 P01 | 26min | 2 tasks | 9 files |
| Phase 37 P02 | 2min | 2 tasks | 2 files |
| Phase 37 P03 | 14min | 3 tasks | 6 files |
| Phase 37 P04 | 11min | 2 tasks | 3 files |
| Phase 37 P05 | 12min | 3 tasks | 10 files |
| Phase 37 P06 | 22min | 3 tasks | 5 files |
| Phase 37 P07 | 11min | 3 tasks | 10 files |
| Phase 37 P08 | 16min | 3 tasks | 10 files |
| Phase 39 P01 | 15min | 3 tasks | 14 files |
| Phase 39 P02 | 3min | 2 tasks | 8 files |
| Phase 39 P03 | 3min | 2 tasks | 9 files |
| Phase 39 P04 | 5min | 2 tasks | 9 files |
| Phase 39 P05 | 22min | 2 tasks | 12 files |
| Phase 39 P06 | 5min | 2 tasks | 8 files |
| Phase 39 P07 | 9min | 2 tasks | 14 files |

## Session Continuity

Last session: 2026-10-13T00:59:26.050Z
Stopped at: Completed 39-07-PLAN.md
Resume file: None
