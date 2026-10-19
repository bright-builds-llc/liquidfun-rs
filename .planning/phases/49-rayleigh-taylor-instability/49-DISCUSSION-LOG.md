# Phase 49: Rayleigh–Taylor Instability - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-10-18
**Phase:** 49-rayleigh-taylor-instability
**Mode:** Yolo (recommended answers auto-selected)
**Areas discussed:** Physics model, Finger seeding and flags, Weight ratio and restart, Look and frame, Caption and cues

---

## Physics model

| Option | Description | Selected |
|--------|-------------|----------|
| One system, extra weight on heavy group via `apply_group_acceleration` | Bridge force along `world.gravity()`, no engine change | ✓ |
| Two particle systems with different densities | Systems do not interact, so the layers pass through each other | |
| Engine per-group density | Deep solver change that risks the protected fingerprints | |

**User's choice:** Recommended default (yolo).
**Notes:** Settle about 0.3 to 0.6 s, then ramp the weight on.

## Finger seeding and flags

| Option | Description | Selected |
|--------|-------------|----------|
| Seeded sinusoidal interface plus a flag sweep in a spike | Deterministic fingers, measured against written pass rules | ✓ |
| Lattice noise only | Fingers look random and may rain through | |
| Unseeded random perturbation | Breaks determinism and fingerprints | |

**User's choice:** Recommended default (yolo).
**Notes:** Flag candidates in order: plain, `VISCOUS`, `VISCOUS` plus `REPULSIVE`. No `COLOR_MIXING`, no bulk `TENSILE`.

## Weight ratio and restart

| Option | Description | Selected |
|--------|-------------|----------|
| Recreating preset 1.2×, 1.5×, 2× (default), 3×, plus the shared Reset scene | Ratio change restarts the run; no new action | ✓ |
| Live ratio spinner | Changes weight mid-run and muddles the replay | |
| Dedicated Flip action | RT-03, future | |

**User's choice:** Recommended default (yolo).

## Look and frame

| Option | Description | Selected |
|--------|-------------|----------|
| Warm orange heavy over deep blue light, tall closed tank | High contrast in every render mode; fills a phone | ✓ |
| Red over cyan | Also contrasting, less readable after density shading | |

**User's choice:** Recommended default (yolo).

## Caption and cues

| Option | Description | Selected |
|--------|-------------|----------|
| "The orange layer is given extra weight"; watch-first README with no cue | Honest caption; whole story fits the 10 s clip | ✓ |
| Caption claiming 2× density | Not allowed by SCENE-03 | |

**User's choice:** Recommended default (yolo).

## Claude's Discretion

Tank size, particle radius and count, layer depths, settle and ramp timings, perturbation modes, flag candidate, exact colors, preview drawing, plan breakdown and file splits.

## Deferred Ideas

- Perturbation presets and a flip-tank action (RT-03).
- Interface tension or viscosity toggle.
- Atwood number readout.
