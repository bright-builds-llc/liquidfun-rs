# Phase 33: Stacked drip fidget - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-27T23:41:57.653Z
**Phase:** 33-stacked-drip-fidget
**Mode:** Yolo
**Areas discussed:** Stack layout, Tray reaction, Repeat cycle, Catalog proof

---

## Stack layout

| Option | Description | Selected |
|--------|-------------|----------|
| Three tipping trays | A top reservoir pours onto three dynamic trays, stacked so each pour feeds the next. Distinct from Liquid Timer's static shelves and Liquid Bubbler's one wheel. | ✓ |
| A column of wheels | Several motor-off wheels in a vertical line. | |
| Static shelves with timed flaps | Fixed shelves whose flaps move on a clock. | |

**User's choice:** Three tipping trays
**Notes:** [auto] Stack layout — Q: "What is the stack?" → Selected: "Three tipping trays" (recommended default)

| Option | Description | Selected |
|--------|-------------|----------|
| New scene `stacked-drip` | Title Stacked Drip, appended after `liquid-bubbler`. Leave Liquid Timer, Liquid Bubbler, Water Wheel, Hydraulic Fountain, and Liquid Tumbler alone. | ✓ |
| Replace Liquid Timer | Retune the existing shelf drain into tipping trays. | |
| Add trays to Liquid Bubbler | Extend the waist-and-wheel scene instead of adding a catalog id. | |

**User's choice:** New scene `stacked-drip`
**Notes:** [auto] Stack layout — Q: "Where does this scene live?" → Selected: "New scene stacked-drip" (recommended default)

---

## Tray reaction

| Option | Description | Selected |
|--------|-------------|----------|
| Motor-off revolute | Each tray is dynamic. Liquid weight tips it. The motor stays off. | ✓ |
| Timed motor | A clock tips each tray whether or not liquid has arrived. | |
| SolidJS animation | The player draws the tip without a physics joint. | |

**User's choice:** Motor-off revolute
**Notes:** [auto] Tray reaction — Q: "What tips each tray?" → Selected: "Motor-off revolute" (recommended default)

| Option | Description | Selected |
|--------|-------------|----------|
| Rest stop or joint limit | Each tray waits until liquid arrives, then tips one way. The upper tray moves before the lower trays. | ✓ |
| Free spin | Trays can tumble before the drip, so the sequence is unreadable. | |
| Timer release | A clock releases the stop independent of the particles. | |

**User's choice:** Rest stop or joint limit
**Notes:** [auto] Tray reaction — Q: "What keeps the cascade in order?" → Selected: "Rest stop or joint limit" (recommended default)

---

## Repeat cycle

| Option | Description | Selected |
|--------|-------------|----------|
| Quiet physical return | Liquid lifts back to the top reservoir. The trays stay the spectacle. Particles are not destroyed to fake the loop. | ✓ |
| One drain until Reset | The stack empties once, like Liquid Timer, and stays empty. | |
| Respawn at the top | Delete particles below the stack and create new ones in the reservoir. | |

**User's choice:** Quiet physical return
**Notes:** [auto] Repeat cycle — Q: "Does the cascade repeat?" → Selected: "Quiet physical return" (recommended default)

| Option | Description | Selected |
|--------|-------------|----------|
| Restore the initial layout | Reset returns the reservoir, resting trays, and initial particles. | ✓ |
| Keep the last pose | Reset leaves trays where the drip left them. | |

**User's choice:** Restore the initial layout
**Notes:** [auto] Repeat cycle — Q: "What does Reset restore?" → Selected: "Restore the initial layout" (recommended default)

---

## Catalog proof

| Option | Description | Selected |
|--------|-------------|----------|
| One plain water group | One `ParticleColor`. No tensile, elastic, rigid, or mixing groups. `MAX_ADVANCE_STEPS` stays 4. | ✓ |
| Tensile viscous drain | Match Liquid Timer's material flags. | |
| Several colors | Mix colors as the liquid crosses the trays. | |

**User's choice:** One plain water group
**Notes:** [auto] Catalog proof — Q: "What liquid is in the stack?" → Selected: "One plain water group" (recommended default)

| Option | Description | Selected |
|--------|-------------|----------|
| Watch-first plus native cascade test | Play, pause, and Reset only. Chromium `just web-player-smoke` covers the new scene and the existing catalog. A native test shows particles pass below the bottom tray and each tray angle changes, upper tray first. | ✓ |
| Visual check only | No native angle or crossing test. | |
| Visitor tray controls | Sliders for tray count, color, or tip speed. | |

**User's choice:** Watch-first plus native cascade test
**Notes:** [auto] Catalog proof — Q: "How is the scene proved?" → Selected: "Watch-first plus native cascade test" (recommended default)

| Option | Description | Selected |
|--------|-------------|----------|
| Original playground scene | Phase 18 credit chrome. No pinned LiquidFun test. Experimental copy, not sealed parity. | ✓ |
| Cite Liquid Timer | Credit `testLiquidTimer.js` even though the trays are original. | |

**User's choice:** Original playground scene
**Notes:** [auto] Catalog proof — Q: "What do the credits cite?" → Selected: "Original playground scene" (recommended default)

---

## Claude's Discretion

- Exact reservoir size, tray length, pivot placement, rest angle, joint limits, particle radius, particle count, and the single particle color.
- How the return lift is built, as long as each tray tips only after particles reach it and the return stays quieter than the stack.
- Static SVG preview artwork captioned `Static preview`.
- File split inside `crates/liquidfun-wasm/src/scene/` when the scene module approaches the file-length trigger.

## Deferred Ideas

- Visitor tray, color, or speed controls.
- Replacing or retuning Liquid Timer, Liquid Bubbler, Water Wheel, Hydraulic Fountain, or Liquid Tumbler.
- A stack of wheels, or more than three reacting parts.
- Two-color mixing through the trays.
- Sealed per-scene differential evidence (PARITY-01).
- Firefox, Safari, Linux qualification, and a live Pages redeploy.
