# Phase 43: GPU Water Render Mode - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md. This log keeps the alternatives that were considered.

**Date:** 2026-10-13
**Phase:** 43-gpu-water-render-mode
**Mode:** Yolo (recommended answers chosen automatically from the v1.5 research)
**Areas discussed:** Mode wiring and picker, Look, Speed lane, Fallback and context loss, Overlays and exports, Verification

---

## Mode wiring and picker

| Option | Description | Selected |
| --- | --- | --- |
| Option in the "Surface" group | Reuses the existing group beside shaded blob | ✓ |
| New "GPU" group | A separate heading for one option | |
| Water as the default | Makes an unverified GPU path the first impression | |

**Choice:** Surface group, opt-in. The default stays shaded blob. When unavailable, the option is shown disabled with the name "Water (unavailable)".

## Look

| Option | Description | Selected |
| --- | --- | --- |
| Static layer as the refraction background, uploaded only when it changes | Real refraction without a per-frame upload | ✓ |
| Procedural backdrop only | Simpler, but it refracts nothing real | fallback only |
| Per-frame upload of the 2D canvas | Pitfall 13: too slow | |

**Choice:** Static-layer refraction, Blinn-Phong highlights, a Beer-Lambert tint that keeps particle colors, speed-ramped foam and `highp`.

## Speed lane

| Option | Description | Selected |
| --- | --- | --- |
| Opt-in Rust lane from `view.velocities()` | Exact, and produced only while Water is active | ✓ |
| Always-on lane | Wastes about 64 KB per frame in other modes | |
| TypeScript position deltas | Breaks when particles are destroyed and reordered | |

## Fallback and context loss

| Option | Description | Selected |
| --- | --- | --- |
| water → shaded blob → canvas per frame, plus restore repaint | Never shows a blank frame and recovers | ✓ |
| Switch the stored mode on failure | A silent preference change | |

**Choice:** Also accept half-float targets (folded todo). A `data-particle-surface` attribute is used for tests.

## Overlays and exports

| Option | Description | Selected |
| --- | --- | --- |
| Overlays stay on the 2D canvas; `svgRenderMode("water")` maps to `"solid"`; README and demo pins unchanged | No export drift | ✓ |
| Water-styled exports | Out of scope (deferred) | |

## Verification

| Option | Description | Selected |
| --- | --- | --- |
| Mock GL unit tests, plus SwiftShader e2e with forced context loss, plus iPhone UAT (non-blocking) | Matches the research testing guidance and hobby scope | ✓ |
| Screenshot diffs of shader output | Flaky across GPUs | |

## Claude's Discretion

- Shader constants, the file split, the worker option transport and probe caching.

## Deferred Ideas

- Water-styled exports
- Water as the default mode
- Reflections and caustics, and refraction of dynamic bodies
