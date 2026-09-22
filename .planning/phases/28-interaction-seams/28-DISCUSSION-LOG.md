# Phase 28: Interaction seams - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-22
**Phase:** 28-Interaction seams
**Mode:** Yolo
**Areas discussed:** Catalog placement, Per-scene controls, Soup Stirrer rail, Impulse shove, Wave Machine rocking, Theo Jansen reverse, Native seams, Shared soup setup, Credits and honesty, Proof gate

---

## Catalog placement

| Option | Description | Selected |
|--------|-------------|----------|
| Append after the current eleven ids | Keeps Dam Break through Rigid Particles first, then Soup, Soup Stirrer, Impulse, Wave Machine, and Theo Jansen. Matches Phases 26 and 27. | ✓ |
| Alphabetical among all scenes | Rewrites the existing order and fights the append-only catalog. | |
| Interactive scenes first | Puts new scenes ahead of the original six. | |

**User's choice:** Append after the current eleven ids (recommended default)
**Notes:** Hash routes stay `#/scene/{id}`. Each entry is ready once the native scene runs, with a static SVG captioned `Static preview`. The Kobalte shell stays; `.catalog-card` stays absent.

---

## Per-scene controls

| Option | Description | Selected |
|--------|-------------|----------|
| Only the required interactions | Soup and Wave Machine stay watch-first. Soup Stirrer, Impulse, and Theo Jansen get one labeled control each. | ✓ |
| Pointer drag on every scene | Adds gestures the success criteria do not ask for. | |
| Freeglut keys only | Hides free, shove, and reverse behind unlabeled keys. | |

**User's choice:** Only the required interactions (recommended default)
**Notes:** PRESET-01 particle-type pickers stay deferred.

---

## Soup Stirrer rail

| Option | Description | Selected |
|--------|-------------|----------|
| Labeled toggle plus canvas click | One non-recreating action, `Toggle paddle rail`. Canvas click or tap sends the same command. Reset puts the paddle back on the rail. | ✓ |
| Drag the paddle off the rail | Invents a gesture the pinned test does not use. | |
| Keyboard `t` only | No visible control for free or put back. | |

**User's choice:** Labeled toggle plus canvas click (recommended default)
**Notes:** Both paths call the same native joint create or destroy.

---

## Impulse shove

| Option | Description | Selected |
|--------|-------------|----------|
| Pointer-aimed native group shove | Click or tap inside the box applies group force from the blob center toward the pointer. A non-recreating `Push` preset switches that gesture to linear impulse. Outside clicks do nothing. Reset restores force. | ✓ |
| Fixed rightward impulse | Ignores where the visitor clicked. | |
| JavaScript position tween | Fakes the shove outside the engine. | |

**User's choice:** Pointer-aimed native group shove (recommended default)
**Notes:** Force is the JS default. Particle-type presets stay deferred.

---

## Wave Machine rocking

| Option | Description | Selected |
|--------|-------------|----------|
| Simulated-time motor | Watch-first. Each advance sets live revolute motor speed to `0.05 * cos(t) * π` from step count times `dt`. Pause freezes `t`. Reset zeroes `t`. | ✓ |
| Visitor drags the tank | Adds a pointer the success criterion does not require. | |
| Wall-clock JavaScript animation | Bypasses the session step budget and the 4-step cap. | |

**User's choice:** Simulated-time motor (recommended default)
**Notes:** Do not copy Water Wheel's motor-off revolute.

---

## Theo Jansen reverse

| Option | Description | Selected |
|--------|-------------|----------|
| Labeled motor direction | Non-recreating preset `forward` (default) and `reverse` flips the live revolute motor speed sign. Walker starts forward under a particle load. Soft distance joints and self-collision filtering stay in the port. | ✓ |
| Keyboard `a/s/d/m` only | Direction exists but is not labeled. | |
| Flip gravity | Does not reverse the motor. | |

**User's choice:** Labeled motor direction (recommended default)
**Notes:** The limit-toggle key is chrome. Do not weld the legs.

---

## Native seams

| Option | Description | Selected |
|--------|-------------|----------|
| Rust engine or WASM scene helpers | Destroy-in-shape, group force and impulse, and live motor updates run in Rust. Add a public API only when the scene cannot show the behavior otherwise. | ✓ |
| Approximate in SolidJS | Carves and shoves by editing rendered positions. | |
| Playground-only physics fork | Splits gallery behavior from `liquidfun`. | |

**User's choice:** Rust engine or WASM scene helpers (recommended default)
**Notes:** Focused tests cover a particle-free pocket, a group momentum change, motor speed updates inside the session step, and Soup Stirrer toggle twice plus Reset. Destruction-by-age stays off.

---

## Shared soup setup

| Option | Description | Selected |
|--------|-------------|----------|
| One private soup builder | Soup Stirrer composes that builder plus paddle, carve, prismatic joint, and stirring force. | ✓ |
| Duplicate the soup layout | The two scenes drift. | |
| Require opening Soup first | Catalog order would become load-bearing. | |

**User's choice:** One private soup builder (recommended default)
**Notes:** The builder stays private inside `liquidfun-wasm`.

---

## Credits and honesty

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 18 credit chrome at the existing pin | Implementation links stay on this repo. Inspiration cites the JS and C++ tests at `7f20402173fd143a3988c921bc384459c6a858f2`. Copy stays experimental. | ✓ |
| Link `google/liquidfun` as the running code | Misstates what the page executes. | |
| Claim sealed parity | The milestone is recognizable ports. | |

**User's choice:** Phase 18 credit chrome at the existing pin (recommended default)
**Notes:** Adapt upstream material only with `THIRD_PARTY_NOTICES.md` preserved.

---

## Proof gate

| Option | Description | Selected |
|--------|-------------|----------|
| Extend Chromium `just web-player-smoke` | Five new scenes play, pause, and reset. Eleven current scenes still open. One gesture each for the rail toggle, the impulse click, and the motor reverse. | ✓ |
| Firefox, Safari, and Linux too | Those are optional profiles, not this phase's gate. | |
| Live Pages redeploy | Publication stays separately authorized. | |

**User's choice:** Extend Chromium `just web-player-smoke` (recommended default)
**Notes:** Do not cut particle counts or raise `MAX_ADVANCE_STEPS` above 4. Independent AI review stays eligible; the implementing agent does not self-approve.

---

## Claude's Discretion

- Exact coordinates, radii, and counts when the pinned tests stay recognizable.
- Static SVG artwork captioned `Static preview`.
- Public destroy-in-shape helper versus a tested scene-local carve.
- Scene file splits near the file-length trigger.
- Exact control wording when free-or-restore, force-or-impulse, and forward-or-reverse stay obvious.
- An optional Theo Jansen speed magnitude beside direction.

## Deferred Ideas

- PRESET-01 particle-type presets for Impulse and Liquid Timer.
- Phase 29 Sparky, Drawing Particles, and the full twelve-scene catalog claim.
- Theo Jansen limit toggle and extra speed keys.
- C++ particle-parameter panels.
- Soup to Soup Stirrer teaching links.
- Firefox, Safari, Linux qualification, and a live Pages redeploy.
