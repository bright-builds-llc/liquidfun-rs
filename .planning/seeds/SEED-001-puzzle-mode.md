---
id: SEED-001
status: promoted
promoted_to: v1.5 Puzzle Mode (2026-10-11)
planted: 2026-10-11
planted_during: between milestones (v1.4 archived 2026-10-11)
trigger_when: The next milestone is chosen for player-facing fun, a game, or goals and challenges in the playground
scope: Large
---

# SEED-001: Puzzle mode — "get the water into the cup"

## Why This Matters

The playground has 25 scenes to watch and poke at, but nothing to accomplish. Puzzle levels give people a reason to stay: a goal zone that counts particles, a limited toolbox (ramps, pins, valves, a sponge made of elastic particles), and a target such as "fill to 80% within 10 seconds". Fixed-step determinism means a solution that works once always works, so a level never fails for reasons the player cannot see.

## When to Surface

**Trigger:** The next milestone is chosen for player-facing fun, a game, or goals and challenges in the playground

This seed should be presented during `/gsd-new-milestone` when the milestone
scope matches any of these conditions:
- Adding goals, scoring, levels, or a game mode to the web playground
- Adding placeable tools or interactive objects players can position before pressing play

## Scope Estimate

**Large** — a full milestone. Likely phases: a goal-zone particle counter and win/lose state; a placeable-tool layer on the canvas; 5 to 8 levels that each teach one mechanic (tension, viscosity, powder, buoyancy); level select and progress.

## Breadcrumbs

- `web/src/catalog/scene-records.ts` — existing scene definitions a level could extend
- `web/src/input/canvas-pointer.ts`, `web/src/input/tilt-gravity.ts` — current interaction seams
- `crates/liquidfun/src/particle/query.rs` — particle region queries for a goal-zone counter
- `crates/liquidfun-wasm/src/scene.rs`, `crates/liquidfun-wasm/src/session.rs` — the browser bridge any per-step goal check passes through
- Repo-local rules in `CLAUDE.md`: new catalog scenes need a portrait frame, README SVG plan entry, and the scale legend

## Notes

Ranked first in the 2026-10-11 brainstorm. A good pairing is SEED-002 (the sandbox editor doubles as a level-authoring tool) and SEED-003 (prettier water as a closing phase).
