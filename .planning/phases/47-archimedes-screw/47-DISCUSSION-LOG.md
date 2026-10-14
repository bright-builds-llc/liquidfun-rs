# Phase 47: Archimedes Screw - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-10-14
**Phase:** 47-archimedes-screw
**Mode:** Yolo (recommended answers auto-selected; no user prompts)
**Areas discussed:** Vane mechanism and wrap, Water loop and reverse, Speed and direction controls, Geometry and scale, Catalog copy and cues, Tests and gates

---

## Vane mechanism and wrap

| Option | Description | Selected |
|--------|-------------|----------|
| Kinematic vane train, snap back one pitch; spike first, closed-loop paddles as fallback | Research synthesis recommendation; geometrically invisible snap | ✓ |
| Closed-loop kinematic paddles only | Never teleports, but turning ends use angles and diverge | |
| Dynamic screw on a revolute motor | Stalls under load; a rotating 2D shape does not convey along an axis | |

**User's choice:** [auto] Kinematic vane train with a short wrap and leak spike in Plan 1
**Notes:** PITFALLS 7 clearance, thickness and speed-assert guards apply to whichever candidate passes.

## Water loop and reverse

| Option | Description | Selected |
|--------|-------------|----------|
| Upper tube end submerged in a weir basin; overflow chute back to the reservoir | Symmetric screw, so reverse scoops basin water back down; closed loop | ✓ |
| Tube spills over the basin edge; basin drains through a bottom hole | Reverse would only move water already in the tube | |
| Emitter refills the reservoir | Teleports or creates particles; research prefers a chute | |

**User's choice:** [auto] Submerged upper end, overflow weir and return chute

## Speed and direction controls

| Option | Description | Selected |
|--------|-------------|----------|
| rpm speed spinner (0 to max) plus a Forward/Reverse Direction preset | Washing Machine and Theo Jansen precedents; keyboard accessible | ✓ |
| One signed spinner from −max to +max | Single control, but reverse is less discoverable | |
| Reverse as an action button | Toggle state is hidden | |

**User's choice:** [auto] `screw-speed` spinner in the HUD and `screw-direction` live preset

## Geometry and scale

| Option | Description | Selected |
|--------|-------------|----------|
| Toy meter scale like the diver, closed tube tilted about 30°, at most 3,000 particles | Matches real practice and the shared budget | ✓ |
| Realistic 1 m screw at fine resolution | Exceeds the particle budget | |

**User's choice:** [auto] Toy meter scale, 25 to 35° tilt, exact numbers at Claude's discretion

## Catalog copy and cues

| Option | Description | Selected |
|--------|-------------|----------|
| Watch-first README and survey with no cues, written hint, honest helix caption | The default shows lifting; reverse is a player choice | ✓ |
| Cue reverse partway through the README clip | The 10 s clip needs the full lift and spill | |

**User's choice:** [auto] Watch-first, caption names flat vanes standing in for a helix

## Tests and gates

| Option | Description | Selected |
|--------|-------------|----------|
| 60 s default and max runs, a reverse test, a g2/g80 stability smoke, the fingerprint gate, then a separate baseline commit | Mirrors Phase 46 and the roadmap criteria | ✓ |
| Lift tests at every gravity | The g80 pressure cap makes lift at g80 unreliable (Phase 46 D-07a) | |

**User's choice:** [auto] As selected

## Claude's Discretion

- Pitch, vane slant and thickness, tube size, tilt within 25 to 35°, particle radius and count, and default and maximum rpm
- Which wrap candidate ships, based on spike evidence
- Preview style, an optional shaft line, plan breakdown and file splits

## Deferred Ideas

- Sand material and tilt control (SCREW-03)
- Helix overlay
