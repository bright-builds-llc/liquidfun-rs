---
gsd_state_version: 1.0
milestone: v1.3
milestone_name: Reference Testbed Scenes
status: phase-complete
stopped_at: Phase 26 verified and complete
last_updated: "2026-09-22T01:56:43.248Z"
last_activity: 2026-09-22
progress:
  total_phases: 4
  completed_phases: 1
  total_plans: 5
  completed_plans: 5
  percent: 100
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-09-21)

**Core value:** Deliver a useful, independent Rust physics library for enjoyable experimentation, with honest limitations and a lightweight native development loop.
**Current focus:** Phase 27 — Material flag groups

## Current Position

Phase: 27
Plan: Not started
Status: Phase 26 complete — ready for Phase 27
Last activity: 2026-09-22

Progress: [██████████] 100%

v1.3 Reference Testbed Scenes. Add the twelve JavaScript testbed scenes the playground does not already have. Phase numbering continues after 25. No package publication or release tag.

## Accumulated Context

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
| 260921-tbx | Global wireframe stroke width slider from 0.25 to 1.5, default 1.0 | 2026-09-22 | complete | [260921-tbx](./quick/260921-tbx-global-wireframe-stroke-width-slider-fro/) |

## Session Continuity

Last session: 2026-09-22T01:56:43.248Z
Stopped at: Phase 26 verified and complete
Resume file: .planning/phases/26-catalog-shell-and-basin-scenes/26-VERIFICATION.md
