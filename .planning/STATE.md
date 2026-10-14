---
gsd_state_version: 1.0
milestone: v1.5
milestone_name: Polish and Test Gaps
status: complete
stopped_at: Phase 44 complete — verification passed (2026-10-14)
last_updated: "2026-10-14T02:21:53.135Z"
last_activity: 2026-10-14
progress:
  total_phases: 8
  completed_phases: 8
  total_plans: 64
  completed_plans: 64
  percent: 100
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-10-11)

**Core value:** Deliver a useful, independent Rust physics library for enjoyable experimentation, with honest limitations and a lightweight native development loop.
**Current focus:** Phase 44 — v1.5 Polish and Test Gaps

## Current Position

Phase: 44
Plan: 3 of 3
Status: Phase complete — verified; v1.5 phases 37–44 all complete, ready for milestone re-audit
Last activity: 2026-10-14

Progress: [██████████] 100%

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
- [Phase 39]: 39-08: replay fixtures record peak_held and brim_step at the decision step (solution 811/262, empty 0/none); release native == wasm; a recorded Brim must hold at brim+margin in both contracts; record mode parses only level/placement lines
- [Phase 39]: 39-09: host primitives in puzzle-host.ts; buildPuzzle, Play, Stop, Retry and Edit in puzzle-run.ts; lifecycle keeps route entry, Try again, leave and the controller (acyclic imports)
- [Phase 39]: 39-09: building shows an empty-lane preview; Play and Stop build sessions synchronously via buildPuzzleSessionSync; Retry still uses the async build with canonical run lanes
- [Phase 39]: 39-09: stars recorded in the loop dispatch closure on running->ended win, and by Stop before the preview rebuild (D-15); store lives in PuzzleRuntime.starStore for the page session
- [Phase 39]: 39-09: every state-changing committed edit runs a commit inspection; live edits coalesce to one rAF inspection; building entry and Stop commit before publishing (D-20, UI-D14)
- [Phase 39]: 39-09: Space togglePlay routed by applyPuzzleCommand in the lifecycle; applyKeyCommand handles only editor keys
- [Phase 39]: 39-10: puzzle loop requests stepCount x speed steps per frame via speedStepRequest; 12 ms budget and MAX_STEPS_PER_FRAME 4 unchanged; unrun steps restore as (unrun / speed) x STEP_SECONDS
- [Phase 39]: 39-11: Tool overlay reads the latest (live or commit) inspection; drawPuzzleTools returns before save() for no tools and draws between particles and the glass cup
- [Phase 39]: 39-11: Editor pointer is a separate PuzzleRuntime.maybeEditorInput handle: armed taps place at the press point without lift, press selects, drags past >4/>8 CSS px move live with grab offset and 48 px touch lift, one pointer at a time; leavePuzzle detaches it
- [Phase 39]: 39-11: main carries data-puzzle-placements/selected/armed/speed/play-enabled/deciding always, outcome-step/decided-step/peak-held when ended, held-hash when set
- [Phase 39]: 39-12: tray items and star rows render with Index (not For) so per-frame publishes never recreate the focused tray button; PuzzlePlayButton removed, form-factor uses PuzzleTransport
- [Phase 39]: 39-12: focus rescue moves to Play/Stop only when activeElement is body, detached or aria-disabled; ended focuses the primary action; panel keydown ignores keys while options are open
- [Phase 39]: 39-12: editor leaves measured fixed (desktop tray 8,236 136x44, selection bar bottom inset+36; portrait tray 218x44, selection 148x44, transport 236x44, result 374x160); deciding state lasts ~300 ms wall time under the 15 Hz synthetic clock, so Plan 13 should observe it in-page
- [Phase 39]: 39-13: won-run e2e literals derive from first-pour-solution.txt via solutionExpectation() (90% of 60%, 0:07.6, held 382); touch project is Chromium Pixel 7 at 390x844 with hasTouch, matching only puzzle-touch.spec.ts
- [Phase 39]: 39-13: run determinism in e2e is a four-attribute fingerprint (outcome, decided, peak held, held hash); Stop-replay, three-round edit history and 1x/2x/4x all equal a clean seam run
- [Phase 39]: 39-14 D-30 gates: 25/25 pass on attempt 1 at a15f04eee (scene-spot 2026-10-13T02-05-45Z 25/25 identical, engine untouched, fixture outcomes unchanged debug/release/wasm, smoke 45, HUD 64, puzzle e2e 29); added cargo-clippy-wasm and cargo-test-wasm gates because default-members is only liquidfun
- [Phase 39]: Phase 40 note: full_count 900 is below real cup capacity (peak held reaches ~1361), so almost any winning run also earns Brim 810
- [Phase 40]: 40-01: POSE_STRIDE 4 (x, y, angle, open_tick); ToolKind::Valve = 2 (1.2 x 0.25 m); open tick checked in parse_one after build area (valve 1..=step_limit/30, others 0) -> PlacementStatus 13 ValveTimeOutOfRange with outline
- [Phase 40]: 40-01: valves open in PuzzleHooks::on_advance (destroy_body before world.step of step open_tick x 30); frame S-1 has the gate's 4 segments, frame S does not; fixture plank 5 tokens / valve 6 tokens
- [Phase 40]: 40-01: D-24 baseline target/phase40/base.txt from scene-spot 2026-10-13T04-11-43Z at 65a37d35b, identical to Phase 39 after-final; after-01 25/25 identical; first-pour release won 201/689/811/262, lost 720/0/0/none
- [Phase 40]: 40-02: Bun fixture grammar mirrors Rust (plank 5 / valve 6 tokens, kind before arity); FIXTURE_FILES derives from PUZZLE_IDS plus EXTRA_FIXTURE_FILES; wasm replay exact won 201/689/811/262, lost 720/0/0/none
- [Phase 40]: 40-02: TS POSE_STRIDE 4 with PuzzlePlacement.openTick (seam absent -> 0), valve code 2 default tick 4, status 13 'Valve time out of range'; debug attribute keeps plank:x,y,angle and writes valve:x,y,angle,openTick; puzzle e2e 29/29
- [Phase 40]: 40-03: LiquidMaterial (Elastic = ELASTIC + SOLID), Option SystemTuning (None adds no builder call) and circle FloatBody floats; PuzzleHooks::new takes a LevelBodies bundle; first-pour fixtures exact
- [Phase 40]: 40-03: levels_tests harness iterates LEVEL_IDS (coverage, 3-star, empty, 3,000 budget); run_outcome skips the id allowlist so drafts can be nudge-checked; first-pour nudges 8/8 wins in release
- [Phase 40]: setValveTime is a PuzzleEvent handled in the reducer (needs stepLimit); applyPuzzleEdit/editAnnouncement take BuildingEditEvent
- [Phase 40]: Landscape selection bar is a 280x44 bottom-centre joined row; portrait bar is 148x96 with the result card raised to inset+156
- [Phase 40]: 40-05: landscape caption box is 48px (not 44) so the 44px Dismiss fits inside the chip border; portrait inset+124px x 52px; form-factor 480x52
- [Phase 40]: 40-05: first-level hint dismissal under liquidfun.puzzle-first-hint.v1 via firstHintMemory (lazy read, in-memory fallback); placing the first tool also dismisses
- [Phase 40]: 40-06 thick-syrup: viscous strength 1.5, reference planks -30/-45 deg win at 440 (Brim 501); the same planks at -15 deg peak 577 < target 610 while water wins at 207 (thick_syrup_needs_a_steep_path)
- [Phase 40]: 40-06 jelly-pad: fixed uncounted elastic pad fills a slotted cradle at the chimney foot; a raised roofed cup is reachable only along the channel past the pad; no-pad copy loses (peak 40), pad drift 0.138 m; no D-11 tool change
- [Phase 40]: 40-06: viscous and elastic levels replay exactly release-native vs wasm and debug (spread 0); per-level proofs live in levels_tests/level_proofs.rs (levels_tests.rs at 626/628 lines)
- [Phase 40]: 40-07 timed-gate ships the D-26 syrup fallback (strength 1.0) as a tent: an upright valve in a one-slot build area stops a water jet into the cup and seals a syrup tank; the preferred diverter spike was monotone in every variant
- [Phase 40]: 40-07 timed-gate window: ticks 1-2 lost, 3-21 won, 22-24 lost; reference tick 9 (only ticks 5-10 hold the raised Brim by step 600); target 502 / full 800 / Brim 582; wasm replay exact for all four fixtures
- [Phase 40]: 40-07 FIXTURES table moved to levels_tests/fixture_table.rs; later level plans add fixtures there
- [Phase 40]: 40-08 sand-chute dropped under D-25: six same-geometry powder-vs-water spike configs; only the r 0.08 slit drain passed 25% (-31% at 0.28 m), in a 0.28-0.32 m band that reverses at 0.20 m (+68% powder) and fades to -1% at 0.56 m; no placement wins only for powder
- [Phase 40]: 40-09 float-up ships (no D-09 drop): a density-0.4 circle (r 0.35) floats up its well and plugs the drain so a U-tube pipe fills to the cup window; float_up_needs_buoyancy (every lane) shows a density-2.0 ball stays down and the reference loses
- [Phase 40]: 40-09: early dense-control wins came from leaks (funnel jet through the window, roof/side-wall seam), fixed by a trap baffle and a 0.64 m thick roof edge; compressed LiquidFun water floats density 2.0 near 3 m depth, so the well column is kept low
- [Phase 40]: 40-10 grand-finale combines jelly pad + timed valve + syrup (Timed Gate tent fed by a pad nozzle); build area is a column above the valve slot so it holds no liquid; ref plank 10 172 1 + valve 4 61 6 14, par 2, limit 900, 1,037 particles
- [Phase 40]: 40-10 grand-finale window: ticks 1-9 and 28-30 lose, 10-27 win; nudges 16/16; exact wasm replay of all 14 fixtures; registry is 6 levels ending with grand-finale
- [Phase 40]: 40-11: D-24 holds on 7353eaded; 27/27 gates exit 0 on the first run, scene-spot 2026-10-13T09-27-14Z 25/25 fingerprints identical, 14/14 wasm fixtures exact, puzzle e2e 57/57; no fix commit
- [Phase 40]: 40-11: final registry 6 levels (first-pour, thick-syrup, jelly-pad, float-up, timed-gate, grand-finale); sand-chute dropped (D-25), float-up kept (D-09); release ms/step 0.25-0.85 (informational, D-23)
- [Phase 41]: 41-01: level hash = FNV-1a 64 of level-hash-v1|{LevelSpec:?}|grid={GRID_METERS:?}|tick={STEPS_PER_OPEN_TICK} (16 lowercase hex); pins in puzzle/level-hashes.txt checked by the native test and the wasm replay (LEVELHASH lines, native == wasm for all six levels)
- [Phase 41]: 41-01: baseline target/phase41/base.txt from scene-spot 2026-10-13T16-42-00Z at f277ecad9, identical to Phase 40 after-final; after-01 25/25 identical; TS levelHash parsed fail-closed against /^[0-9a-f]{16}$/
- [Phase 41]: 41-02: progress v1 core is pure: parseProgress fail-closed (json/shape/future-version), unknown level ids and top-level keys written back verbatim, unknown fields in known records dropped, brim/thrifty without win is shape; stars combine only via mergeLevelStars
- [Phase 41]: 41-02: not-won placements stored with zero stars; restoreChoice restores only for the live levelHash, <= 16 entries, each through strict maybeParsePlacement, else discard; mergeProgress prefers stored placements and stored unknown data
- [Phase 41]: 41-03: progress shell (player/puzzle-progress.ts) degrades once on any provider/probe/read/write failure or unusable value and never writes the progress key afterwards; probe uses liquidfun.puzzle-progress.probe; each write re-reads and merges
- [Phase 41]: 41-03: runtime hydrates StarStore from persisted wins; recordStars writes through before publish and folds back another tab's stars only when they differ (sameLevelStars); createPuzzleRuntime() defaults to UNAVAILABLE_PROGRESS_STORAGE; PuzzleController.progressDegraded
- [Phase 41]: 41-04: restore runs once per route entry in buildPuzzle after adopt and before loaded (seam disarms it; Retry/Edit/Stop/Try again never restore); reset re-dispatched, reducer unchanged
- [Phase 41]: 41-04: placements save in commitNow (building) and playPuzzle after guards, never on live drag frames; dedupe key id|levelHash|JSON(tools) reset on entry and leave; restored first-pour tools dismiss the first hint (accepted)
- [Phase 41]: 41-05: #/puzzles (and #/puzzles/) is {kind:"puzzles"}, title 'Puzzles · liquidfun-rs playground', identity 'puzzles'; #/puzzles/<x> stays an unknown scene; grid is a self-scrolling fixed page in the canvas shell with an inline SiteFooter
- [Phase 41]: 41-05: sidebar Puzzles link above Demos (aria-current page on grid, true on levels) adds one drawer focusable, so shell.spec focus-trap cycle is 27; puzzle-chrome splits HUD_STYLESHEETS from PUZZLE_STYLESHEETS
- [Phase 41]: 41-06: PuzzleOptionsSheet requires progressDegraded and renders PuzzleLevelLinks (All levels + degraded note) first, so the fallback sheet gets it too; form-factor row starts with it; HUD unchanged
- [Phase 41]: 41-06: last level's Next slot is an accented primary All levels link to #/puzzles after win or loss; the disabled No next level yet button is deleted; fallback primary link is All levels
- [Phase 41]: 41-07: progress e2e (24 tests, both viewports) reads the live first-pour hash via parseLevelHashes; a correct-hash seed must restore plank:8,64,23 so the wrong-hash discard is not vacuous; seam runs stay on their own page and seam-free checks use context().newPage()
- [Phase 41]: 41-08: D-21 holds on 5e10940e9; 27/27 gates exit 0 on the first run, scene-spot 2026-10-13T17-31-31Z 25/25 fingerprints identical, engine untouched, six level-hash pins native == wasm, 14/14 wasm fixtures, puzzle e2e 81/81; no fix commit
- [Phase 42]: 42-01: puzzle frames carry read-only splashContacts (new particle-fixture contacts), impactCount (non-sensor rigid Begin) and impactImpulse (max summed solved normal impulse) from the existing StepReport; tally-off stepping bit-identical; crates/liquidfun untouched (1a7f1ac99, 56b542f69)
- [Phase 42]: 42-01: A2 held (float-up Begin contacts solved same step, impulse max 0.062 N·s); only float-up has rigid impacts; splash per-step max 86-153, p90 4-40 (target/phase42/sound-probe.txt)
- [Phase 42]: 42-01: baseline target/phase42/base.txt from scene-spot 2026-10-13T19-08-15Z at bd685b08f, identical to Phase 41 after-final; after-01 25/25 identical; wasm replay 6 LEVELHASH + 14 REPLAY pass
- [Phase 42]: 42-02: splash thresholds from the probe: rest p95 3 -> SPLASH_MIN_CONTACTS_PER_STEP 4, p90 of steps >= 4 is 29 -> SPLASH_FULL 29, IMPACT_FULL_IMPULSE 0.062008 (float-up solution impulse p90); first-pour solution audible 302/321 steps
- [Phase 42]: 42-02: pure core: chime takes 3 pool entries and evicts oldest (startedAt, id); dropped requests keep lastStartedAt; burst onset and rigid impact merge into one impact at the larger strength; a non-increasing stepIndex resets memory then records the frame
- [Phase 42]: 42-03: pointerup unlocks only for non-mouse pointers (HTML activation list); mouse unlocks on pointerdown
- [Phase 42]: 42-03: engine suspend() clears the resume in-flight guard so a pending resume never blocks the next
- [Phase 42]: 42-03: construction failure logs once via glue logError and makes sound unavailable; runtime failure sets failed and suspends once
- [Phase 42]: 42-03: each chime note takes its own pool entry, oscillator and gain
- [Phase 42]: 42-04: slot gap 8px on the shared transport/options-slot flex rule; transport computed gap unchanged; puzzle.css 452 lines
- [Phase 42]: 42-04: unavailable toggle is a separate disabled button behind Show; on/off share one button so focus stays; probe boxes match the UI-SPEC fit table (8 px clearance at all 5 viewports)
- [Phase 42]: 42-05: sound on/off first-pour runs give identical RunFingerprint (outcome 201, decided 262, peak 811, hash 0cf8c2703dcd8fac); 16 source starts with sound on
- [Phase 42]: 42-05: 32/32 phase gates exit=0 on first full run; 25/25 scene fingerprints identical to base
- [Phase 43]: 43-01: speed lane is a stateless per-capture flag (captureFrame(true)); lane = length_squared().sqrt() from the same velocity read so max(lane) == maxSpeed bit for bit; TS parses RenderFrame.maybeParticleSpeeds fail closed (52eec199e, 62e62e041)
- [Phase 43]: 43-01: SessionCore::capture_frame() (no lane) is #[cfg(test)]; wasm uses capture_frame_with_speeds; on/off 120-step proofs bit-identical for dam-break and first-pour
- [Phase 43]: 43-01: baseline target/phase43/base.txt from scene-spot 2026-10-13T23-40-01Z at 2653fd987, identical to Phase 42 after-postfix; after-01 25/25 identical; wasm replay 6 LEVELHASH + 14 REPLAY pass
- [Phase 43]: 43-02: field-pass.ts owns one WebGL2 context per particle canvas (shared buffers + metadata cache, pass slots keyed by module symbols, per-key availability); shaded blob accepts EXT_color_buffer_float or EXT_color_buffer_half_float with checkFramebufferStatus deciding (D-13) (cb9a0ad07)
- [Phase 43]: 43-02: watchParticleSurface creates the surface before adding repaint listeners so the reset runs first; restore repaint builds fresh programs (D-14) (aabbf02e6)
- [Phase 43]: 43-03: D-07 refined: Water refracts a water-owned stable-walls raster (water-backdrop.ts) re-rasterized only when a frame repeats and differs from the last raster, reusing static-layer sameContent/sameCameraView; procedural backdrop otherwise; texture upload gated by revision (39fd0571a, 0c6bc8c8b)
- [Phase 43]: 43-03: drawWater (lazy WATER_PASS, 2-attachment RGBA16F, constant-0 speed attribute when the lane is absent) returns false on every unsupported state; cachedWaterSupport is a per-page probe on a detached canvas released with loseContext; only water-programs.ts imports .glsl?raw (53b89a36e)
- [Phase 43]: 43-04: presentSceneFrame returns DrawnSurface from one chain (Water, shaded blob, canvas soft blob); scene canvas carries data-drawn-surface and data-speed-lane after every draw; data-particle-surface stays the requested surface (6d0856d57, f3adfdbd8)
- [Phase 43]: 43-04: assignParticleSurface detaches the previous watchParticleSurface and repaints only while the canvas is current; Water availability read once in App.tsx via cachedWaterSupport() and shared through RenderModeOptions (857603bcd)
- [Phase 43]: 43-05: D-11 capture option is a stateless per-capture flag: direct scene, puzzle and Tesla Valve worker owners read wantsParticleSpeeds(appearance.renderMode()) at capture time; worker advance/capture carry a validated boolean speeds and transfer the lane (948a5bcc0, dc81856c6)
- [Phase 43]: 43-06: 13 Chromium Water proofs (no shader pixel diff); Water/Shaded blob RunFingerprint equal (201/262/811/0cf8c2703dcd8fac); 1 s Dam Break SVG sha256 ff92a2ad equal in both modes (d0987eae8)
- [Phase 43]: 43-06: tuned water look: foam 3.5/9.0 m/s, shininess 64, specular 0.4; normal 6.0, refraction 10 px, absorption 0.9, foam max 0.85 unchanged (4f68a6aab)
- [Phase 43]: 43-06: 27/27 phase gates exit=0 on first run at 4f68a6aab; scene-spot 2026-10-14T00-27-52Z 25/25 fingerprints identical; D-20 iPhone view stays non-blocking UAT
- [Phase 44]: 44-01: maxOpenTick saturates at MAX_OPEN_TICK 255 like Rust; seam valves without openTick read DEFAULT_VALVE_OPEN_TICK (kind checked first, so unknown kind + bad openTick now throws 'Invalid puzzle placement') (f57c59b9a)
- [Phase 44]: 44-01: portrait two-kind tray chip is a CSS-only 22px + max-content grid (name 14/20 over count 12/16); tray stays 218/188 x 44; names 37/37 and 36/36 at both phone widths (877aba559)
- [Phase 44]: 44-01: #puzzle-valve-time is no longer aria-hidden and describes the valve stepper group only while a valve is selected; no live region
- [Phase 44]: 44-02: web/tsconfig.e2e.json extends the strict base (overrides only types node+vite/client and include e2e/**/*.ts + playwright.config.ts); bun run typecheck runs both projects (42af8afef)
- [Phase 44]: 44-02: all 16 e2e type errors fixed at the root without any/ts-ignore; e2e/tesla-benchmark.d.ts types window.teslaBenchmark as typeof compareWorkerDirect; touched specs 30/30 (d15e553f5)
- [Phase 44]: 44-03: Next e2e deletes the seam global with Reflect.deleteProperty before the hash-link click; without it the route never reaches building (29b792625)
- [Phase 44]: 44-03: timed-gate 1x/2x/4x share one build and give identical fingerprints (429/519/582/e405de2194871266); stepper described only with a valve selected (813516547)
- [Phase 44]: 44-03: 10/10 phase gates exit=0 on first run at 813516547; crates/ untouched since 9e6efcc23; puzzle e2e 104, water 13, HUD 64; no fix commit

### Pending Todos

- Half-float render targets: implemented in 43-02 (`field-pass.ts` `supportsHalfFloatTargets`, cb9a0ad07). The iPhone check remains a non-blocking D-20 UAT item.

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
| Phase 39 P08 | 14min | 2 tasks | 10 files |
| Phase 39 P09 | 9min | 2 tasks | 14 files |
| Phase 39 P10 | 2min | 1 tasks | 2 files |
| Phase 39 P11 | 7min | 3 tasks | 12 files |
| Phase 39 P12 | 13min | 2 tasks | 11 files |
| Phase 39 P13 | 9min | 3 tasks | 7 files |
| Phase 39 P14 | 10min | 1 tasks | 0 files |
| Phase 40 P01 | 21min | 3 tasks | 20 files |
| Phase 40 P02 | 5min | 2 tasks | 26 files |
| Phase 40 P03 | 26min | 2 tasks | 12 files |
| Phase 40 P04 | 10min | 2 tasks | 25 files |
| Phase 40 P05 | 7min | 3 tasks | 14 files |
| Phase 40 P06 | 57min | 3 tasks | 16 files |
| Phase 40 P07 | 66min | 3 tasks | 16 files |
| Phase 40 P08 | 12min | 2 tasks | 2 files |
| Phase 40 P09 | 49min | 2 tasks | 10 files |
| Phase 40 P10 | 43min | 3 tasks | 13 files |
| Phase 40 P11 | 11min | 2 tasks | 0 files |
| Phase 41 P01 | 11min | 3 tasks | 12 files |
| Phase 41 P02 | 6min | 2 tasks | 6 files |
| Phase 41 P03 | 6min | 2 tasks | 12 files |
| Phase 41 P04 | 5min | 2 tasks | 8 files |
| Phase 41 P05 | 7min | 3 tasks | 15 files |
| Phase 41 P06 | 3min | 2 tasks | 13 files |
| Phase 41 P07 | 4min | 2 tasks | 2 files |
| Phase 41 P08 | 9min | 2 tasks | 0 files |
| Phase 42 P01 | 15min | 3 tasks | 12 files |
| Phase 42 P02 | 6min | 3 tasks | 10 files |
| Phase 42 P03 | 8min | 3 tasks | 15 files |
| Phase 42 P04 | 5min | 2 tasks | 7 files |
| Phase 42 P05 | 20min | 2 tasks | 3 files |
| Phase 43 P01 | 17min | 3 tasks | 9 files |
| Phase 43 P02 | 5min | 2 tasks | 5 files |
| Phase 43 P03 | 9min | 3 tasks | 15 files |
| Phase 43 P04 | 4min | 3 tasks | 14 files |
| Phase 43 P05 | 5min | 2 tasks | 12 files |
| Phase 43 P06 | 19min | 3 tasks | 4 files |
| Phase 44 P01 | 3min | 2 tasks | 8 files |
| Phase 44 P02 | 3min | 2 tasks | 9 files |
| Phase 44 P03 | 5min | 3 tasks | 2 files |

## Session Continuity

Last session: 2026-10-14T02:09:40.270Z
Stopped at: Completed 44-03-PLAN.md
Resume file: None
