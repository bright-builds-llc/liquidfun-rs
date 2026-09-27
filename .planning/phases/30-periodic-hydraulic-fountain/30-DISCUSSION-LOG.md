# Phase 30: Periodic hydraulic fountain - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-27
**Phase:** 30-periodic-hydraulic-fountain
**Mode:** Yolo
**Areas discussed:** Travel layout, Drive, Liquid, Catalog credits and proof

---

## Travel layout

| Option | Description | Selected |
|--------|-------------|----------|
| Two chambers joined by a throat | Piston squeezes one side; liquid arrives on the other. Leaves the existing Fountain alone. | ✓ |
| One basin, piston only sloshes in place | Does not meet "travels elsewhere". | |
| Replace the existing Fountain emitter | Drops an original scene this milestone keeps. | |

**User's choice:** Two chambers joined by a throat (recommended default)
**Notes:** [auto] Travel layout — Q: "Where does the squeezed liquid go?" → Selected: "Two chambers joined by a throat"

| Option | Description | Selected |
|--------|-------------|----------|
| Advance, then retract, and repeat | Periodic stroke. Liquid is moved, not spawned or deleted to fake a jet. | ✓ |
| One-shot squeeze | Not periodic. | |
| Hidden emitter inside a piston shape | Duplicates Fountain. | |

**User's choice:** Advance, then retract, and repeat (recommended default)
**Notes:** [auto] Travel layout — Q: "What does a cycle look like?" → Selected: "Advance, then retract, and repeat"

---

## Drive

| Option | Description | Selected |
|--------|-------------|----------|
| Prismatic motor rewritten from simulation time | Same live-motor pattern as Wave Machine. Uses the existing joint API. | ✓ |
| Kinematic body teleported each frame | Can tunnel through the liquid. | |
| Visitor drags the piston | Not the timed behavior this phase names. | |

**User's choice:** Prismatic motor rewritten from simulation time (recommended default)
**Notes:** [auto] Drive — Q: "What moves the piston?" → Selected: "Prismatic motor from simulation time"

| Option | Description | Selected |
|--------|-------------|----------|
| Built-in period, watch-first | Play, pause, and reset only. Matches Wave Machine and Particles. | ✓ |
| Labeled period presets | Extra controls the phase does not need. | |
| Slider that recreates the world | Breaks the watch-first reset story. | |

**User's choice:** Built-in period, watch-first (recommended default)
**Notes:** [auto] Drive — Q: "Who sets the period?" → Selected: "Built-in period, watch-first"

---

## Liquid

| Option | Description | Selected |
|--------|-------------|----------|
| One plain water group | The spectacle is the squeeze and the crossing. | ✓ |
| Tensile colored jet | Adds a material the phase does not need. | |
| Several material flags | Belongs to later fidget phases if anywhere. | |

**User's choice:** One plain water group (recommended default)
**Notes:** [auto] Liquid — Q: "What liquid is squeezed?" → Selected: "One plain water group"

| Option | Description | Selected |
|--------|-------------|----------|
| Keep the 4-step cap and particle budget | Record any radius or count shrink as a playground adaptation. | ✓ |
| Cut particle count to look smoother | Forbidden by the milestone out-of-scope list. | |
| Raise MAX_ADVANCE_STEPS | Forbidden by the same list. | |

**User's choice:** Keep the 4-step cap and particle budget (recommended default)
**Notes:** [auto] Liquid — Q: "How should cost be handled?" → Selected: "Keep the 4-step cap"

---

## Catalog credits and proof

| Option | Description | Selected |
|--------|-------------|----------|
| Append `hydraulic-fountain` after `sparky` | Ready entry, static preview, portrait frame, README plan. | ✓ |
| Replace `fountain` | Removes the aimed-stream scene. | |
| Hide it until phases 31–33 land | The phase is this scene, not a batch. | |

**User's choice:** Append `hydraulic-fountain` (recommended default)
**Notes:** [auto] Catalog — Q: "How does the scene enter the catalog?" → Selected: "Append hydraulic-fountain"

| Option | Description | Selected |
|--------|-------------|----------|
| Original-scene credit, no pinned test | Honest. PLAY-03 is for ports. | ✓ |
| Cite the nearest upstream test | Would claim a port this phase is not. | |

**User's choice:** Original-scene credit (recommended default)
**Notes:** [auto] Catalog — Q: "What does the credit cite?" → Selected: "Original playground scene"

| Option | Description | Selected |
|--------|-------------|----------|
| Chromium smoke plus a native travel test | Open, play, pause, reset, and particles cross the throat within one period. | ✓ |
| Visual check only | No regression lock for the travel claim. | |
| Sealed C++ differential | Out of scope for this original scene. | |

**User's choice:** Chromium smoke plus a native travel test (recommended default)
**Notes:** [auto] Proof — Q: "What proves the scene?" → Selected: "Smoke plus native travel test"

---

## Claude's Discretion

- Chamber sizes, throat width, stroke, period, radius, and count, as long as one cycle moves liquid through the throat.
- Return path versus falling back on retract.
- Static SVG artwork.
- Scene-module file split.

## Deferred Ideas

- Sinusoidal wave tank (Phase 31).
- Liquid motion bubbler (Phase 32).
- Stacked drip fidget (Phase 33).
- Fountain aim and emission controls.
- Elaborate return plumbing beyond a repeating cycle.
- Sealed parity, Firefox, Safari, Linux qualification, and a live Pages redeploy.
