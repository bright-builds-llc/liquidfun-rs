---
id: SEED-004
status: promoted
promoted_to: v1.6 Physics Curiosity Scenes (2026-10-14)
planted: 2026-10-11
planted_during: between milestones (v1.4 archived 2026-10-11)
trigger_when: The next milestone adds playground scenes or wants small self-contained wins
scope: Medium
---

# SEED-004: Physics-curiosity scenes

## Why This Matters

Small, self-contained scenes of physics people tend to share and wonder about. Each is about one phase, and the catalog, portrait framing, and README gallery already make adding a scene routine. Candidates:
- Siphon
- Archimedes screw
- Sand hourglass (powder particles)
- Cartesian diver
- Rayleigh–Taylor instability (heavy liquid fingering down through light liquid)
- Fake ferrofluid that spikes toward the pointer

## When to Surface

**Trigger:** The next milestone adds playground scenes or wants small self-contained wins

This seed should be presented during `/gsd-new-milestone` when the milestone
scope matches any of these conditions:
- Adding new scenes to the playground catalog
- A filler or warm-up phase is wanted between larger features
- Showcasing particle materials (powder, viscous, tensile, elastic) or multiple densities

## Scope Estimate

**Medium** — roughly one phase per scene; pick two to four per milestone. Rayleigh–Taylor may need two particle systems or per-group density and is the riskiest; the ferrofluid needs a pointer-driven force field.

## Breadcrumbs

- `web/src/catalog/scenes.ts`, `web/src/catalog/scene-records.ts` — catalog registration
- `web/src/catalog/portrait-bounds.ts` — required portrait frame per scene
- `web/scripts/readme-svg/plans.ts` — required README SVG plan per scene
- `crates/liquidfun-wasm/src/scene/` — native scene builders
- `crates/liquidfun/src/particle/force.rs` — particle forces for a ferrofluid-style pull
- Repo-local `CLAUDE.md` sections: README scene gallery, Playground canvas framing, Playground scale legend
