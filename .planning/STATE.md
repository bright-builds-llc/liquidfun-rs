---
gsd_state_version: 1.0
milestone: v1.3
milestone_name: Reference Testbed Scenes
status: verifying
stopped_at: Completed 32-03-PLAN.md
last_updated: "2026-09-27T20:06:54.558Z"
last_activity: 2026-09-27
progress:
  total_phases: 8
  completed_phases: 6
  total_plans: 33
  completed_plans: 33
  percent: 100
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-09-21)

**Core value:** Deliver a useful, independent Rust physics library for enjoyable experimentation, with honest limitations and a lightweight native development loop.
**Current focus:** Phase 32 — liquid-motion-bubbler

## Current Position

Phase: 32 (liquid-motion-bubbler) — EXECUTING
Plan: 3 of 3
Status: Ready for verification
Last activity: 2026-09-27

Progress: [██████████] 100%

v1.3 Reference Testbed Scenes. Add the twelve JavaScript testbed scenes the playground does not already have. Phase numbering continues after 25. No package publication or release tag.

## Accumulated Context

### Roadmap Evolution

- Phase 30 added: Periodic hydraulic fountain
- Phase 31 added: Sinusoidal wave tank
- Phase 32 added: Liquid motion bubbler
- Phase 33 added: Stacked drip fidget

### Decisions

v1.2 decisions are in `.planning/milestones/v1.2-STATE.md` and PROJECT.md Key Decisions.

- [v1.2]: Unprofiled Dam Break Medium closed at stamp `2026-09-21T20-38-50Z`, ratio `2.956857456935513`, git `89d34564`. Later solver fixes are not that measurement.
- [v1.2]: `reviewed_reports` stays empty. Planning archive is not a git release tag.
- [v1.3]: Port the twelve JavaScript testbed scenes besides Dam Break. Keep the six existing playground scenes. Recognizable behavior in the current player, not a sealed parity claim.
- [v1.3]: Roadmap clusters Phases 26–29 as basin/catalog → material flags → interaction seams → Sparky/Drawing/full catalog. Future DRAW-02, PRESET-01, PARITY-01, BOX2D-01 stay unmapped.
- [Phase 26]: Particles matches testParticles.js geometry; watch-first hooks; MAX_ADVANCE_STEPS stays 4
- [Phase 26]: Liquid Timer matches testLiquidTimer.js bowl, radius 0.025, TENSILE|VISCOUS slab, ten shelf edges; MAX_ADVANCE_STEPS stays 4
- [Phase 26]: Catalog appends particles then liquid-timer after water-wheel with ready:true and empty controls
- [Phase 26]: Inspiration cites pinned Particles/LiquidTimer JS and C++ tests at 7f204021; implementation stays host-locked
- [Phase 26]: SceneControls returns null when controls.length === 0 rather than empty chrome
- [Phase 26]: Kept Plan 03 eight-scene Vitest tables; Task 1 only strengthened credit label asserts
- [Phase 26]: Watch-first capture stubs use center click SceneAction so capture scripts keep a required action
- [Phase 26]: Watch-first scenes selected by controls.length === 0 so Phase 27 can reuse the rule
- [Phase 26]: POINTER_CONTROL is Partial so Particles and Liquid Timer never require gestures
- [Phase 26]: Drawer reverse-tab last link is Liquid Timer; wrap count raised to 10 for nine focusables
- [Phase 27]: Minimal SceneId wiring in Task 1 RED so failing construction tests compile under hooks — Sequential TDD commits need a compiling stub rather than compile-fail-only RED
- [Phase 27]: Private basin_family helper with vertical walls ending at y=2 for Surface Tension reuse by Elastic/Rigid — Claude Discretion and D-07 pinned geometry; prevents basin drift across material scenes
- [Phase 27]: Reuse basin_family for Elastic Particles; keep SPRING and ELASTIC on separate SOLID groups — Preserves spring vs elastic contrast and shared vertical-wall basin for Rigid next
- [Phase 27]: Reuse basin_family and Elastic poses; RIGID|SOLID group flags only, no elastic particle flags — Preserves solid-clump contrast for MAT-03 without spoofing elastic softness
- [Phase 27]: Extended scenes.test.ts eleven-scene Record tables in Task 2 so typecheck passes after SceneId grew — Plan verification requires bun run typecheck; incomplete Record tables blocked compile
- [Phase 27]: Rigid preview uses stroke outlines; Elastic uses soft ellipses with tilted blue box to contrast solidity — UI-SPEC preview art direction for MAT-02 vs MAT-03 contrast
- [Phase 27]: Task 1 eleven-scene Vitest contract was already locked in 27-04 for typecheck; 27-05 verified and did not rewrite it — Plan 27-04 extended Record tables so typecheck passed after SceneId grew; 27-05 acceptance criteria were already green
- [Phase 27]: Watch-first material capture stubs reuse center click no-op SceneAction like Particles and Liquid Timer — Capture script still needs a SceneAction; WASM pointer is a no-op for watch-first scenes
- [Phase 27]: Raised ALL_SCENE_TIMEOUT_MS to 220_000 for eleven-scene open/reset loops
- [Phase 27]: Forced CI=true for player-smoke so production preview is the gate, not a reused Vite dev server
- [Phase 27]: Scoped session status to .session-status after tilt/wireframe also expose status roles
- [Phase 28]: Minimal SceneId::Soup wiring in Task 1 RED so construction tests compile under hooks
- [Phase 28]: destroy_particles_in_shape marks then compact_pending_particles so the pocket is empty before first frame
- [Phase 28]: Private soup_family returns ground BodyId for Plan 02 Soup Stirrer prismatic rail
- [Phase 28]: Minimal SceneId::SoupStirrer wiring in Task 1 RED so construction tests compile under hooks
- [Phase 28]: Reuse ParticleSystemDef default damping 1.0 as pinned SetDamping(1.0); document PARTICLE_DAMPING explicitly
- [Phase 28]: Map destroy_joint failures to SceneConstruction rather than UnknownControl
- [Phase 28]: Minimal SceneId::Impulse wiring in Task 1 RED so construction tests compile under hooks
- [Phase 28]: Reject non-empty presets on Impulse build because push-mode is Live-only
- [Phase 28]: Minimal SceneId::WaveMachine wiring in Task 1 RED so construction tests compile under hooks
- [Phase 28]: Motor formula uses PI to match pinned testWaveMachine.js — not Water Wheel motor-off
- [Phase 28]: Minimal SceneId::TheoJansen wiring in Task 1 RED so construction tests compile under hooks — Same sequential-TDD pattern as Plans 01-04
- [Phase 28]: Direction-only motor-direction forward/reverse; no speed-magnitude or limit-toggle — Plan 05 discretion lock and D-07
- [Phase 28]: Split tests under theo_jansen/ to stay under Bright Builds file-length limit — Implementation plus inline tests exceeded 628 physical lines
- [Phase 28]: Left PlaygroundShell and THIRD_PARTY_NOTICES unchanged — no eleven-demo hardcoding and no new adapted notices
- [Phase 28]: Extended scenes.test.ts Record tables in Task 2 so typecheck passes after SceneId grew
- [Phase 28]: navigation.test.ts needed no edits — it already maps SCENE_IDS dynamically
- [Phase 28]: Used bun run test:unit because package.json has no test script
- [Phase 28]: Raised MAX_RIGID_SEGMENTS 16→64 and MAX_RIGID_CIRCLES 8→48 so Theo Jansen frame capture fits walker+balls
- [Phase 28]: Routed Soup Stirrer toggle-paddle-rail through apply_action to match catalog action kind
- [Phase 28]: Independent AI review remains eligible under D-26; implementing agent did not self-approve
- [Phase 30]: Kept the plan starting numbers for the hydraulic piston. — The crossing test passed at speed 0.6, stroke 0.35, period 2.0, max force 1.0e6, and a floor throat below y = 0.12, so those values were not retuned.
- [Phase 30]: Scene tests live beside the hydraulic fountain module. — The implementation plus inline tests would pass 500 physical lines, so tests moved to hydraulic_fountain/tests.rs and the credit path stayed a single scene file.
- [Phase 30]: Copied the Plan 01 wall and piston endpoints unchanged into the portrait frame — The crossing test did not retune stroke, walls, or piston x, so the catalog frame uses those finished constants.
- [Phase 30]: Raised the shared view rectangle maxY from 1.55 to 1.6 — At maxY 1.55 the fitted phone height was 0.277 after the 16px camera inset, under the required 0.28. maxY 1.6 clears that minimum and still contains the walls.
- [Phase 30]: Matched the original Fountain sidebar link as Static preview Fountain so Hydraulic Fountain does not share the click. — A role name of /Fountain/ also matched Hydraulic Fountain and failed Playwright strict mode.
- [Phase 30]: Left ALL_SCENE_TIMEOUT_MS at 380000 and left both step caps at 4. — The smoke failure was a locator collision, and the passing Chromium run finished in 42.6 seconds.
- [Phase 30]: Passing Chromium smoke is browser-gate evidence only. This implementing agent did not approve the work. — The 2026-09-16 owner policy keeps independent AI review eligible, and a passing smoke command is not a review acknowledgment.
- [Phase 31]: Kept the plan starting numbers for the wave tank. — The far-wall test passed at stroke 0.16, period 2.0, channel 0.90, depth 0.32, damping 0.2, radius 0.025, and max force 1.0e6, so those values were not retuned.
- [Phase 31]: Passing native wave-tank tests are evidence only. This implementing agent did not approve the work. — The 2026-09-16 owner policy keeps independent AI review eligible, and D-11 says the implementing agent must not approve its own work.
- [Phase 31]: Used the plan pool rectangle because Plan 01 kept the starting wall and stroke constants — The far-wall test did not retune stroke, walls, or platform, so viewBounds and the phone frame stay at minX -0.12, minY -0.28, maxX 1.52, maxY 1.02.
- [Phase 31]: Passing catalog unit tests are evidence only. This implementing agent did not approve the work. — The 2026-09-16 owner policy keeps independent AI review eligible, and D-11 says the implementing agent must not approve its own work.
- [Phase 31]: Left ALL_SCENE_TIMEOUT_MS at 380000 and left both step caps at 4. — The Chromium run finished in 36.1 seconds, so the suite timeout did not need to grow.
- [Phase 31]: Passing Chromium smoke is browser-gate evidence only. This implementing agent did not approve the work. — The 2026-09-16 owner policy keeps independent AI review eligible, and a passing smoke command is not a review acknowledgment.
- [Phase 32]: Wheel density is 0.03 because 0.20 only turned the wheel about 0.007 rad in 2 seconds — Particle hits crossed the waist but stayed under the 0.05 angle floor at the starting density. The revolute motor stayed off.
- [Phase 32]: The plate rests 0.02 m above the floor so polygon skin does not lift translation off 0 during the dwell — A flush contact separated to translation 0.015. Matching the joint frame to the raised rest pose keeps the dwell at speed 0.
- [Phase 32]: Passing native bubbler tests are evidence only. This implementing agent did not approve the work. — The 2026-09-16 owner policy and D-11 keep independent AI review eligible, and a passing cargo test is not a review acknowledgment.
- [Phase 32]: Kept the plan view rectangle because gap, stroke, and walls were unchanged; the raised plate still sits inside it — Plan 01 raised the plate 0.02 m and lowered wheel density. The waist gap, stroke, and walls stayed the same, so viewBounds and the phone frame stay at minX -0.71, minY -0.16, maxX 1.12, maxY 1.98.
- [Phase 32]: Passing catalog unit tests are evidence only. This implementing agent did not approve the work. — The 2026-09-16 owner policy and D-11 keep independent AI review eligible, and a passing Vitest run is not a review acknowledgment.
- [Phase 32]: Left ALL_SCENE_TIMEOUT_MS at 380000 and left both step caps at 4. — The Chromium run finished in 38.5 seconds, so the suite timeout did not need to grow.
- [Phase 32]: Passing Chromium smoke is browser-gate evidence only. This implementing agent did not approve the work. — The 2026-09-16 owner policy and D-11 keep independent AI review eligible, and a passing smoke command is not a review acknowledgment.

### Pending Todos

None.

### Blockers/Concerns

None for roadmap. A fresh unprofiled pair is required before claiming the ≤ 3× ratio for any HEAD after `89d34564`.

## Retained Context

- Local checks plus one macOS CI job; expensive qualification stays optional/manual.
- No package publication or release tag.
- PLAT-01, PLAT-05 and DOCS-09 remain deferred in archived requirements.
- Historical decisions: `.planning/milestones/v1.2-STATE.md`, `.planning/milestones/v1.1-STATE.md`, `.planning/milestones/v1.0-STATE.md`, `.planning/MILESTONES.md`.

### Quick Tasks Completed

| ID | Description | Date | Status | Directory |
| --- | --- | --- | --- | --- |
| 260921-tbx | Global wireframe stroke width slider from 0.1 to 1.5, default 0.3 | 2026-09-22 | complete | [260921-tbx](./quick/260921-tbx-global-wireframe-stroke-width-slider-fro/) |

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

## Session Continuity

Last session: 2026-09-27T20:05:53.785Z
Stopped at: Completed 32-03-PLAN.md
Resume file: None
