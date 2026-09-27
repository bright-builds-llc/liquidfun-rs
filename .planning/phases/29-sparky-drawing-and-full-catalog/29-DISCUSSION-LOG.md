# Phase 29: Sparky, Drawing, and full catalog - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-27T04:15:12.770Z
**Phase:** 29-Sparky, Drawing, and full catalog
**Mode:** Yolo
**Areas discussed:** Sparky contact sparks, Drawing Particles paint, Catalog claim credits and proof

---

## Sparky contact sparks

| Option | Description | Selected |
| --- | --- | --- |
| Post-step begin contacts | Record sparkable body begins and spawn powder VFX after the step unlocks | ✓ |
| Mid-step listener mutation | Create particle groups inside the contact callback, matching the C++ listener shape | |
| Timed emitter | Spray sparks on a clock without reading contacts | |

**User's choice:** Post-step begin contacts (recommended default)
**Notes:** Other scenes keep `NoDecisionHook`. Persistent contacts must not keep sparking. No FFI expansion.

| Option | Description | Selected |
| --- | --- | --- |
| Bounded fade-and-destroy ring | Powder group, outward velocity, batch color fade, destroy on lifetime, overwrite destroys the previous slot | ✓ |
| Destruction-by-age on the whole system | Enable age destruction so sparks disappear with every other particle | |
| Permanent bursts | Leave spark groups alive | |

**User's choice:** Bounded fade-and-destroy ring (recommended default)
**Notes:** Exact pool length 50 is discretion below or at that cap. Reset clears live slots.

## Drawing Particles paint

| Option | Description | Selected |
| --- | --- | --- |
| Drag paint with water and elastic | Empty vessel; destroy-then-create on drag; labeled water/elastic preset; clear lastGroup on pointer-up, destroy, and reset | ✓ |
| Full keyboard matrix | Ship every freeglut material key in this phase | |
| Water-only stamps | Paint plain water with no contrasting material | |

**User's choice:** Drag paint with water and elastic (recommended default)
**Notes:** DRAW-02 keeps the rest of the material matrix. Powder may be added later only as discretion if it stays a small preset.

## Catalog claim credits and proof

| Option | Description | Selected |
| --- | --- | --- |
| Append two scenes and extend Chromium smoke | Keep current order including liquid-tumbler; static previews; portrait frames; README plan ids; host-locked credits; recognizable-port copy; 4-step cap unchanged | ✓ |
| New catalog shell | Card grid or filters for the twelve-scene claim | |
| Parity gate | Block the phase on sealed C++ differentials and a Pages redeploy | |

**User's choice:** Append two scenes and extend Chromium smoke (recommended default)
**Notes:** Independent AI review stays eligible. The implementing agent does not approve its own work.

## Claude's Discretion

- Chamber and stroke numbers that stay recognizable.
- Contact-capture mechanism, as long as spawn happens after unlock.
- Color-fade API shape.
- Preview art, portrait rectangles, file split, and preset wording.
- Optional powder as a third Drawing material.

## Deferred Ideas

- DRAW-02 full keyboard matrix.
- PRESET-01 Impulse and Liquid Timer presets.
- PARITY-01 and BOX2D-01.
- Sparky pointer poke.
- Phases 30–33.
- Firefox, Safari, Linux qualification, and live Pages redeploy.
