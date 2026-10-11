---
id: SEED-005
status: promoted
promoted_to: v1.5 Puzzle Mode (2026-10-11)
planted: 2026-10-11
planted_during: between milestones (v1.4 archived 2026-10-11)
trigger_when: The next milestone is about feel, juice, or atmosphere in the playground, or wants a quick delightful phase
scope: Small
---

# SEED-005: Sonification with WebAudio

## Why This Matters

Map impacts, splashes, and particle speed to synthesized sound. It is cheap to build and makes scenes feel alive — Liquid Bubbler, Stacked Drip, Dam Break, and Sparky would benefit most. Sound must stay opt-in (muted by default, browser autoplay rules) and must never feed back into the simulation.

## When to Surface

**Trigger:** The next milestone is about feel, juice, or atmosphere in the playground, or wants a quick delightful phase

This seed should be presented during `/gsd-new-milestone` when the milestone
scope matches any of these conditions:
- Playground polish, feel, or accessibility work
- Puzzle mode (SEED-001) wants audio feedback for goals and tools
- A small standalone phase is wanted

## Scope Estimate

**Small** — likely one phase: per-frame aggregate signals from the worker (contact counts, kinetic energy, impact events), a WebAudio synth layer, and a mute/volume control.

## Breadcrumbs

- `crates/liquidfun-wasm/src/frame.rs` — frame diagnostics already include `body_contact_count`
- `web/src/physics/simulation-worker.ts`, `web/src/physics/worker-messages.ts` — where per-frame signals cross to the main thread
- `web/src/components/SceneControls.tsx` — where a mute control would live
