---
gsd_state_version: 1.0
milestone: v1.3
milestone_name: Reference Testbed Scenes
status: executing
stopped_at: Completed 28-04-PLAN.md
last_updated: "2026-09-22T14:17:01.835Z"
last_activity: 2026-09-22
progress:
  total_phases: 8
  completed_phases: 2
  total_plans: 19
  completed_plans: 15
  percent: 79
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-09-21)

**Core value:** Deliver a useful, independent Rust physics library for enjoyable experimentation, with honest limitations and a lightweight native development loop.
**Current focus:** Phase 28 — Interaction seams

## Current Position

Phase: 28 (Interaction seams) — EXECUTING
Plan: 5 of 8
Status: Ready to execute
Last activity: 2026-09-22

Progress: [██░░░░░░░░] 25%

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

## Session Continuity

Last session: 2026-09-22T14:17:01.832Z
Stopped at: Completed 28-04-PLAN.md
Resume file: None
