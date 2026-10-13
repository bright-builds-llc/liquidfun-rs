# Phase 42: Opt-in Sound - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md. This log keeps the alternatives that were considered.

**Date:** 2026-10-13
**Phase:** 42-opt-in-sound
**Mode:** Yolo
**Areas discussed:** Toggle placement and persistence, sounds and signals, pausing and the silent-switch hint, determinism and verification

---

## Toggle placement and persistence

| Option | Description | Selected |
|--------|-------------|----------|
| HUD speaker toggle plus a drawer mirror | Visible 44 px button in the existing options slot, with a mirror switch and the hint in the options drawer | ✓ |
| Drawer-only switch | No HUD change, but the toggle is hidden behind a menu | |
| Global site-wide toggle | Applies to scenes too, which is out of phase scope | |

**User's choice:** Recommended default (yolo): HUD toggle plus drawer mirror, with a separate versioned key `liquidfun.puzzle-sound.v1`. The context is created only on an opted-in user gesture.
**Notes:** A stored "on" still waits for the first gesture before any audio, which follows the autoplay rules.

## Sounds and signals

| Option | Description | Selected |
|--------|-------------|----------|
| Hand-written Web Audio with read-only bridge aggregates | Noise and bandpass splash, pitch-drop impact, pentatonic plink from `goalCount` delta, arpeggio chime | ✓ |
| Vendored zzfx | Smaller tuning effort, but adds vendored code | |
| Sample assets with Howler | Needs asset licensing and a stale dependency | |

**User's choice:** Recommended default (yolo): hand-written Web Audio. Plinks come from the existing `goalCount`. Splash and impact come from new read-only puzzle frame scalars. Cap of 6 voices, per-voice rate limits, and the chime may steal voices.
**Notes:** zzfx stays the fallback only if tuning stalls.

## Pausing and the silent-switch hint

| Option | Description | Selected |
|--------|-------------|----------|
| Suspend on hidden and on leave, hint in the drawer and toggle title | Context kept for reuse, voices stopped on Stop, Edit and Retry | ✓ |
| Close the context on leave | Needs another gesture to recreate | |
| Force `audioSession` "playback" | Pauses the user's other audio and is not Baseline | |

**User's choice:** Recommended default (yolo).

## Determinism and verification

| Option | Description | Selected |
|--------|-------------|----------|
| Rust replay pins, 25/25 scene spot, Playwright fake AudioContext, Vitest pure core | Covers all four success criteria | ✓ |
| Manual listening only | Not verifiable | |

**User's choice:** Recommended default (yolo).

## Claude's Discretion

- Synthesis parameters, numeric limits within D-09, module and component names, bridge scalar names and aggregate definitions, icon artwork.

## Deferred Ideas

- Sound in catalog scenes, UI ticks, a volume slider, a loss sound, and an opt-in `audioSession` playback setting.
