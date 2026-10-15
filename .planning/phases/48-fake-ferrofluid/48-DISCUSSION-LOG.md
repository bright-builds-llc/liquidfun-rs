# Phase 48: Fake Ferrofluid - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-10-15
**Phase:** 48-fake-ferrofluid
**Mode:** Yolo
**Areas discussed:** Field model and spikes, Magnet state and pointer, Strength and on/off controls, Shared force helper, Geometry/color/marker, Catalog/caption/cues, Tests and evidence

---

## Field model and spikes

| Option | Description | Selected |
|--------|-------------|----------|
| Lift along the field axis + spatial modulation + weak lateral pull, capped and damped | Spikes by design; follows gravity; research recommendation | ✓ |
| Pure point attraction toward the magnet | Easiest; makes a ball or flings droplets (anti-feature) | |
| Real magnetostatics | O(n²) or a field solve; out of scope | |

**User's choice:** [auto] Lift + modulation + lateral pull (recommended default)
**Notes:** A Plan 1 spike picks between cos(k·s) modulation and fixed spike-tip targets against written pass rules.

---

## Magnet state and pointer

| Option | Description | Selected |
|--------|-------------|----------|
| Rest under the dish on load; press-drag moves it; Up/Cancel switches it off and parks it at rest | Matches the roadmap "release collapses"; watch-first on load | ✓ |
| Up/Cancel leaves the magnet where it was dropped | Spikes would not collapse on release | |
| Up/Cancel returns the magnet to rest, still on | Spikes would not collapse on release | |

**User's choice:** [auto] Switch off and park on release (recommended default)

---

## Strength and on/off controls

| Option | Description | Selected |
|--------|-------------|----------|
| Live HUD spinner 0–100 % (100 = tuned safe max) + panel action `magnet` toggle | Keyboard path and README cue without new cue kinds | ✓ |
| Strength only (0 = off) | No keyboard on/off; README collapse needs a strength change | |
| Recreating preset for strength | Restarts the scene on every change | |

**User's choice:** [auto] Spinner + toggle action (recommended default)

---

## Shared force helper

| Option | Description | Selected |
|--------|-------------|----------|
| New shared bridge module, mass- and iteration-calibrated, gravity-aware, reused by Phase 49 | Roadmap dependency for Phase 49 | ✓ |
| Inline the force code in the scene | Phase 49 would duplicate it | |

**User's choice:** [auto] Shared module (recommended default)

---

## Geometry, color and marker

| Option | Description | Selected |
|--------|-------------|----------|
| Toy meter scale, shallow wide dish, blue-gray/graphite liquid, drawn marker as world content | Research defaults; contrast with `#071018` | ✓ |
| Real centimeter scale | Tiny particles cost about 10× per step | |
| Black liquid | Invisible on the canvas | |

**User's choice:** [auto] Toy scale with blue-gray liquid (recommended default)

---

## Catalog, caption and cues

| Option | Description | Selected |
|--------|-------------|----------|
| Title "Fake Ferrofluid", "scripted magnetic pull" caption, README cues `magnet` off at ~6 s and on at ~8 s, survey Default | Honest caption; shows collapse and re-rise | ✓ |
| No README cues | The clip would not show a collapse | |

**User's choice:** [auto] Fake title with action cues (recommended default)

---

## Tests and evidence

| Option | Description | Selected |
|--------|-------------|----------|
| Native release tests per behavior, 60 s held max-strength stability at three positions, g2/g80 stability smokes, gate and separate baseline commit | Matches the roadmap criteria and prior phases | ✓ |
| Require spikes at gravity 80 too | Conflicts with the pressure cap lesson (Phase 46 D-07a) | |

**User's choice:** [auto] Per-behavior native tests (recommended default)

## Claude's Discretion

Field formula and tuning, dish geometry, particle radius and count, color hex values, marker style, helper module shape, plan breakdown, and file splits.

## Deferred Ideas

- Hysteresis (FERRO-04)
- A magnet position preset and a ferrofluid GPU Water tint
- Held-pointer README cue kinds
