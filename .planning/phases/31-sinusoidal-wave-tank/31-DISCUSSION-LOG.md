# Phase 31: Sinusoidal wave tank - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-27
**Phase:** 31-sinusoidal-wave-tank
**Mode:** Yolo
**Areas discussed:** Pool and platform, Drive, Liquid, Catalog credits and proof

---

## Pool and platform

| Option | Description | Selected |
|--------|-------------|----------|
| New still pool with one rising end | Leaves Wave Machine as the rocking-container port. Waves travel toward a fixed far wall. | ✓ |
| Retune Wave Machine into a wave tank | Drops the existing rocking-tank scene. | |
| Whole floor tilts | That is Wave Machine again. | |

**User's choice:** New still pool with one rising end (recommended default)
**Notes:** [auto] Pool and platform — Q: "What holds the water?" → Selected: "New still pool with one rising end"

| Option | Description | Selected |
|--------|-------------|----------|
| Catalog id `wave-tank`, after `hydraulic-fountain` | Short id, distinct from `wave-machine`. | ✓ |
| Catalog id `sinusoidal-wave-tank` | Matches the phase slug and is longer than the other scene ids. | |
| Replace `wave-machine` in the catalog | Removes a shipped scene. | |

**User's choice:** Catalog id `wave-tank`, after `hydraulic-fountain` (recommended default)
**Notes:** [auto] Pool and platform — Q: "What is the catalog id?" → Selected: "`wave-tank` after hydraulic-fountain"

---

## Drive

| Option | Description | Selected |
|--------|-------------|----------|
| Prismatic motor following a sinusoid of simulation time | Same live-motor pattern as Hydraulic Fountain. The stroke is smooth and repeating. | ✓ |
| Kinematic body teleported each frame | Can tunnel through the liquid. | |
| Rock the whole tank on a revolute joint | Copies Wave Machine. | |

**User's choice:** Prismatic motor following a sinusoid of simulation time (recommended default)
**Notes:** [auto] Drive — Q: "What moves the end platform?" → Selected: "Prismatic motor on a sinusoid"

| Option | Description | Selected |
|--------|-------------|----------|
| Built-in period and stroke, watch-first | Play, pause, and reset only. Matches Hydraulic Fountain and Wave Machine. | ✓ |
| Labeled amplitude presets | Extra controls the phase does not need. | |
| Slider that recreates the world | Breaks the watch-first reset story. | |

**User's choice:** Built-in period and stroke, watch-first (recommended default)
**Notes:** [auto] Drive — Q: "Who sets the period and stroke?" → Selected: "Built-in period, watch-first"

---

## Liquid

| Option | Description | Selected |
|--------|-------------|----------|
| One plain water group | The spectacle is the traveling wave. | ✓ |
| Tensile colored surface | Adds a material the phase does not need. | |
| Several material flags | Belongs to later fidget phases if anywhere. | |

**User's choice:** One plain water group (recommended default)
**Notes:** [auto] Liquid — Q: "What liquid fills the pool?" → Selected: "One plain water group"

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
| Ready catalog entry, static preview, portrait frame, README plan | Same companions Phase 30 required for a new scene id. | ✓ |
| Interactive preview that runs the scene | Heavier than the catalog contract. | |
| Skip the portrait frame | Fails the phone-canvas check. | |

**User's choice:** Ready catalog entry, static preview, portrait frame, README plan (recommended default)
**Notes:** [auto] Catalog credits and proof — Q: "How does the scene appear in the catalog?" → Selected: "Ready entry plus static preview, portrait frame, and README plan"

| Option | Description | Selected |
|--------|-------------|----------|
| Original-scene credit, no pinned test | This phase is not a LiquidFun port. | ✓ |
| Credit Wave Machine's test | Would claim a port this scene is not. | |
| Omit credits | Breaks the Phase 18 chrome. | |

**User's choice:** Original-scene credit, no pinned test (recommended default)
**Notes:** [auto] Catalog credits and proof — Q: "What does the credit cite?" → Selected: "Original playground scene"

| Option | Description | Selected |
|--------|-------------|----------|
| Chromium smoke plus a native far-wall vertical-motion test | Opens, plays, pauses, resets, and shows the wave reached the far end. Existing scenes still open. | ✓ |
| Visual inspection only | No regression lock for the wave. | |
| Sealed C++ differential | PARITY-01 stays future work. | |

**User's choice:** Chromium smoke plus a native far-wall vertical-motion test (recommended default)
**Notes:** [auto] Catalog credits and proof — Q: "What proves the wave?" → Selected: "Chromium smoke and a native far-end motion test"

---

## Claude's Discretion

- Exact pool length, platform width, stroke, period, particle radius, and particle count.
- Short floor slab versus a vertical face that translates with that slab.
- Static SVG preview artwork.
- File split when the scene module approaches the file-length trigger.

## Deferred Ideas

- Phase 32 liquid motion bubbler.
- Phase 33 stacked drip fidget.
- Visitor amplitude, period, or stroke controls.
- Replacing or retuning Wave Machine.
- Sealed per-scene differential evidence.
- Firefox, Safari, Linux qualification, and a live Pages redeploy.
