# Demo Recordings and README Gallery Design

**Date:** 2026-09-19
**Status:** Approved for implementation

## Goal

Show each of the six web playground demos directly in the README while retaining
a full video recording that readers can open. Each recording demonstrates the
default simulation plus one representative interaction or control change.

## Media presentation

Each scene has two repository-owned media files under `docs/assets/demos/`:

- an H.264 MP4 recording for full playback
- a compact animated WebP preview for inline README playback

The README adds a `Demo gallery` section beneath `Web playground`. Each gallery
entry includes the scene title linked to its hosted route and an animated WebP
linked to the corresponding MP4. This avoids relying on GitHub to render
repository-hosted MP4 files inline.

## Capture workflow

A rerunnable TypeScript script uses the repository's existing Playwright
installation against the production-base preview. It records the six canonical
scene routes at a fixed desktop viewport and performs one scene-specific action:

1. Dam Break: drag the obstacle.
1. Fountain: change or aim the jet.
1. Float or Sink: drop the selected body.
1. Color Mixer: stir the particle groups.
1. Jelly Drop: poke the jelly.
1. Water Wheel: aim or strengthen the jet.

Each clip is silent and approximately 8–10 seconds. The capture includes the
playground chrome, canvas, and controls so viewers can connect the simulation
with its available interaction. The script waits for the scene to report
`Playing` and for its step index to advance before recording the representative
action.

Playwright's captured video is converted with `ffmpeg` into:

- an H.264, browser-compatible MP4
- a reduced-size animated WebP suitable for README display

The script fails with an actionable message when `ffmpeg`, Chromium, the
production preview, a scene route, or expected player state is unavailable.

## Repository integration

The capture script and committed media are repository-owned. Transient raw
recordings and conversion intermediates remain under ignored output paths.
Final filenames use scene IDs:

```text
docs/assets/demos/dam-break.mp4
docs/assets/demos/dam-break.webp
...
docs/assets/demos/water-wheel.mp4
docs/assets/demos/water-wheel.webp
```

The existing README maturity and compatibility statements remain unchanged.
The gallery describes the recordings as demonstrations, not parity evidence.

## Verification

Implementation is complete when:

1. All six MP4 and six animated WebP files exist and are non-empty.
1. `ffprobe` can read every MP4 and WebP, and durations remain within the
   intended short-clip range.
1. Every README preview path, video link, and hosted scene link resolves.
1. The capture script is rerunnable from a clean generated web build.
1. `just web-player-smoke` passes.
1. `just markdown-check` passes for the README and design documentation.
1. The managed Bright Builds check is run and any unrelated existing findings
   are reported without being misrepresented as introduced by this work.

## Out of scope

- Audio, narration, captions, and background music
- Mobile and browser-matrix recordings
- Replacing the live playground with video
- Treating recordings as physics parity or release evidence
