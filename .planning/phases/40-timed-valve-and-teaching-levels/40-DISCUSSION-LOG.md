# Phase 40: Timed Valve and Teaching Levels - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-10-12
**Phase:** 40-Timed Valve and Teaching Levels
**Mode:** Yolo (recommended answers auto-accepted; advisor research skipped for a lightweight pass)
**Areas discussed:** Valve mechanism, Level set, Solvability proof, First-level hint, Frames and budgets

---

## Valve mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| Timed gate tool, open tick as 4th pose value, body destroyed at open step | Input-free, integer-exact, uses the reserved lane | ✓ |
| Level-authored valve with a player-set time | Fewer tool changes, but the player does not place it | |
| Passive one-way flap | No timing idea to teach | |

**User's choice:** Timed gate tool (recommended)
**Notes:** Half-second ticks; stride 4 for every kind with 0 required for non-valves; stepper in the selection bar.

## Level set

| Option | Description | Selected |
|--------|-------------|----------|
| 7 levels: water, viscous, powder, jelly pad, buoyancy, valve, finale | Covers every listed idea | ✓ |
| 6 levels without buoyancy | Lower risk | (fallback only) |
| 8 levels with two finales | More tuning cost | |

**User's choice:** 7 levels, with buoyancy droppable to 6 if it cannot be proven solvable.

## Jelly pad form

| Option | Description | Selected |
|--------|-------------|----------|
| Level-authored elastic group, never counted | Predictable, anchored by geometry | ✓ |
| Placeable jelly pad tool | More player agency, harder to anchor and validate | |

## Solvability proof

| Option | Description | Selected |
|--------|-------------|----------|
| Solution and empty fixtures per level; nudge sweep ±1/±2 cells, ≥70% win, release lane | Meets LEVEL-02 without slowing the debug lane | ✓ |
| Nudge sweep in debug lane too | Slow on macOS debug builds | |

## First-level hint

| Option | Description | Selected |
|--------|-------------|----------|
| Inside a fixed caption HUD leaf, dismissal in versioned localStorage with in-memory fallback | Keeps the obstacle set fixed | ✓ |
| Floating callout without `data-hud-obstacle` | Violates the HUD chrome rule | |

## Frames and budgets

| Option | Description | Selected |
|--------|-------------|----------|
| Portrait-friendly level geometry, ≤3,000 particles at step 0, timing recorded only | Fills phones; budget enforced by test | ✓ |
| Wide levels like first-pour | Poor portrait fill | |

## Claude's Discretion

Level ids, titles, captions, geometry and tuning; valve size, default and keys; valve module placement; material enum naming.

## Deferred Ideas

Live valve tapping, hint ladder, sponge absorption, more levels, moving cups.
