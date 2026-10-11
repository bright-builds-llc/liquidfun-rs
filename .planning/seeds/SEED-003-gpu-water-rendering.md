---
id: SEED-003
status: dormant
planted: 2026-10-11
planted_during: between milestones (v1.4 archived 2026-10-11)
trigger_when: The next milestone touches rendering, visual polish, or README and demo presentation
scope: Medium
---

# SEED-003: GPU "realistic water" render mode

## Why This Matters

The playground already draws liquid as a surface (`shaded-blob` is the default, with `soft-blob` and `contour` beside it), using canvas blur and thresholding. A WebGL shader mode could go further: refraction of what sits behind the water, specular highlights, thickness-based tint, and foam or spray where particles move fast. It only changes the renderer, so physics and its tests stay untouched, and every scene and README export improves at once.

## When to Surface

**Trigger:** The next milestone touches rendering, visual polish, or README and demo presentation

This seed should be presented during `/gsd-new-milestone` when the milestone
scope matches any of these conditions:
- Visual polish, new render modes, or GPU rendering in the playground
- Refreshing the README gallery or demo exports
- A closing phase for a player-facing milestone (for example SEED-001)

## Scope Estimate

**Medium** — a phase or two: a WebGL field-and-shade pass with a canvas fallback; foam/speed cues (needs per-particle velocity in the frame); export parity decisions for SVG and WebP.

## Breadcrumbs

- `web/src/render/mode.ts` — render mode list and controls-sheet groups
- `web/src/render/metaball.ts`, `web/src/render/density-shade.ts`, `web/src/render/contour.ts` — existing surface modes to build on
- `web/src/render/webgl-particles.ts`, `web/src/render/blob-uniforms.ts` — existing WebGL path
- `web/src/physics/frame.ts`, `crates/liquidfun-wasm/src/frame.rs` — what each frame carries to the renderer
- `web/scripts/readme-svg/` — README export pipeline

## Notes

Originally suggested as "make the water look like water"; checking the code showed smooth surfaces already exist, so this seed is the GPU and effects step beyond them.
