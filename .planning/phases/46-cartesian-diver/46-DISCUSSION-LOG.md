# Phase 46: Cartesian Diver - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-10-14
**Phase:** 46-cartesian-diver
**Mode:** Yolo (recommended answers auto-selected; no user prompts)
**Areas discussed:** Squeeze input model, Sink and rise mechanism, Squeeze visuals, Bottle geometry and scale, Phone long-press callout, Catalog copy and cues

---

## Squeeze input model

| Option | Description | Selected |
|--------|-------------|----------|
| One state, last input wins | Down on the bottle squeezes, Up/Cancel releases, the Squeeze action flips | ✓ |
| Toggle OR pointer | Squeezed while either the toggle or the press is active | |
| Press anywhere on canvas | No hit region | |

**User's choice:** [auto] One state, last input wins; hit region is the bottle plus about 0.3 m
**Notes:** The success criterion says "pressing and holding on the bottle". Its keyboard path is the existing panel action, so no new control kind is needed.

## Sink and rise mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| Fixture density switch + reset mass | Matches the caption wording; research-recommended | ✓ |
| Gravity scale switch | Does not wake the body; the caption would have to say "gravity scale" | |
| Scripted force on the diver | Not buoyancy-driven | |

**User's choice:** [auto] Density switch with wide margins, short ramp, damping, circle body with sleep disabled
**Notes:** PITFALLS 6: densities near 1.5–2.0 hover because water compresses with depth.

## Squeeze visuals

| Option | Description | Selected |
|--------|-------------|----------|
| Draw-only dent + drawn bubble circle | Stable, no water push, honest caption | ✓ |
| Kinematic walls that move inward | Real displacement, but pressurizes a closed bottle and risks instability at gravity 80 | |
| Bottle tint only | Does not satisfy "visibly dents" | |

**User's choice:** [auto] Draw-only dent and drawn bubble shrinking from about 0.6 to 0.3 of the diver radius

## Bottle geometry and scale

| Option | Description | Selected |
|--------|-------------|----------|
| Tall closed bottle at toy meter scale, small headspace | Portrait-friendly, ≤ 3,000 particles | ✓ |
| Real-size 0.3 m bottle | Violates toy meter scale guidance | |
| Open-topped bottle | The criterion says closed bottle | |

**User's choice:** [auto] About 2–2.5 m by 5–6 m interior, closed cap, diver at least 6 particle diameters wide

## Phone long-press callout

| Option | Description | Selected |
|--------|-------------|----------|
| Shared canvas CSS + contextmenu preventDefault, e2e-checked | Fixes all scenes, no fingerprint impact | ✓ |
| Diver-only suppression | Per-scene branching in shared chrome | |

**User's choice:** [auto] Shared suppression on `canvas.scene-canvas`; a real-iPhone check is optional manual evidence

## Catalog copy and cues

| Option | Description | Selected |
|--------|-------------|----------|
| New `curiosity` family files, Squeeze action cues at ~1 s and ~5.5 s, survey `Action("squeeze")` | No new cue kinds | ✓ |
| New pointer-down README/survey cue kinds | Forbidden by shared rules | |

**User's choice:** [auto] As selected. The baseline grows in a separate final commit.

## Claude's Discretion

- Exact densities, ramp, damping, geometry, particle radius and count; dent shape; preview style; plan split; the module file layout.

## Deferred Ideas

- DIVER-04 "no return" depth and multiple divers.
- Squeeze-strength slider.
