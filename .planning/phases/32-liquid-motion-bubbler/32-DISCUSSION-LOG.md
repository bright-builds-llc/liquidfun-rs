# Phase 32: Liquid motion bubbler - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `32-CONTEXT.md`; this log preserves alternatives considered by the yolo advisor pass.

**Date:** 2026-09-27
**Phase:** 32-liquid-motion-bubbler
**Mode:** Yolo
**Areas discussed:** Vessel, Wheel and flow, Liquid color, Catalog credits and proof

---

## Vessel

| Option | Description | Selected |
|--------|-------------|----------|
| Upper reservoir, one static waist, wheel in the lower chamber | Matches the roadmap line and stays distinct from Water Wheel and Liquid Timer. | ✓ |
| Open basin with an aimed jet | That is Water Wheel. | |
| Shelves or a stack of reacting parts | Liquid Timer and Phase 33. | |

**User's choice:** Upper reservoir, one static waist, wheel in the lower chamber (recommended default)
**Notes:** [auto] Vessel — Q: "What holds the liquid?" → Selected: "Upper reservoir, one static waist, wheel below"

| Option | Description | Selected |
|--------|-------------|----------|
| Catalog id `liquid-bubbler`, after `wave-tank` | Short id, distinct from `water-wheel`. | ✓ |
| Catalog id `liquid-motion-bubbler` | Matches the phase slug and is longer than the other scene ids. | |
| Replace `water-wheel` in the catalog | Removes a shipped scene. | |

**User's choice:** Catalog id `liquid-bubbler`, after `wave-tank` (recommended default)
**Notes:** [auto] Vessel — Q: "What is the catalog id?" → Selected: "`liquid-bubbler` after wave-tank"

---

## Wheel and flow

| Option | Description | Selected |
|--------|-------------|----------|
| Motor-off revolute paddle wheel turned by the drip | The liquid is the cause of the turn. | ✓ |
| Motor-driven wheel | The wheel turns whether or not liquid hits it. | |
| Reuse Water Wheel's jet | Copies the existing interactive scene. | |

**User's choice:** Motor-off revolute paddle wheel turned by the drip (recommended default)
**Notes:** [auto] Wheel and flow — Q: "What turns the wheel?" → Selected: "Particles hitting a motor-off paddle wheel"

| Option | Description | Selected |
|--------|-------------|----------|
| Quiet return to the upper reservoir | The drip continues for the whole play session without becoming a second fountain. | ✓ |
| One-shot drain until empty | The watch scene goes still after one pass. | |
| Flip the whole vessel on a timer | A different fidget, and easy to confuse with a tumbler. | |

**User's choice:** Quiet return to the upper reservoir (recommended default)
**Notes:** [auto] Wheel and flow — Q: "Does the drip continue?" → Selected: "Quiet return lift; waist and wheel stay the spectacle"

---

## Liquid color

| Option | Description | Selected |
|--------|-------------|----------|
| One water group with one distinct particle color | Reads as colored liquid without a new material flag. | ✓ |
| Two color-mixing groups | Closer to Color Mixer and Surface Tension. | |
| Uncolored water | Drops the "colored" part of the roadmap line. | |

**User's choice:** One water group with one distinct particle color (recommended default)
**Notes:** [auto] Liquid color — Q: "How is the liquid colored?" → Selected: "One tinted water group"

---

## Catalog credits and proof

| Option | Description | Selected |
|--------|-------------|----------|
| Watch-first, static preview, portrait frame, README plan, Chromium smoke, native waist-and-wheel test | Same gate as Wave Tank. Original-scene credits. Implementer does not self-approve. | ✓ |
| Cite a pinned LiquidFun test | This fidget is original. | |
| Add Firefox, Safari, Linux, or a Pages redeploy as the phase gate | Outside hobby-scope local proof. | |

**User's choice:** Watch-first catalog scene with Chromium smoke and a native crossing-and-rotation test (recommended default)
**Notes:** [auto] Catalog credits and proof — Q: "How is the scene proved?" → Selected: "Chromium smoke plus native waist crossing and wheel rotation"

## Claude's Discretion

- Chamber sizes, waist gap, wheel radius, paddle count, particle radius, particle count, and the single color.
- Return-lift mechanism, as long as the drip crosses the waist, turns the wheel, and stays the spectacle.
- Static SVG preview artwork.
- Scene module file split.

## Deferred Ideas

- Phase 33 stacked drip fidget.
- Visitor waist, color, or wheel controls.
- Replacing or retuning Water Wheel, Color Mixer, Liquid Timer, or Hydraulic Fountain.
- Two-color mixing through the waist.
- Sealed per-scene differential evidence.
- Firefox, Safari, Linux qualification, and a live Pages redeploy.
