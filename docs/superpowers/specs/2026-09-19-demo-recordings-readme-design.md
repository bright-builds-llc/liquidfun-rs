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

## Deterministic capture workflow

A rerunnable TypeScript script uses the repository's pinned Playwright and
Chromium installation against the production-base preview. It installs a
synthetic `requestAnimationFrame` queue before application code loads, then
advances that queue with a fixed one-ULP-safe 60 Hz timestamp interval instead
of waiting on wall-clock time. The interval is defined once by applying
`nextUp()` to `1_000 / CAPTURE_PROFILE.simulationHz`, each callback timestamp
is computed as `callbackCount * interval`, and capture fails fast if that
single fixed interval ever stops producing exactly one engine step per callback
across the full 480-step run.
The script captures the six canonical scene routes at a fixed desktop viewport
and device-pixel ratio and performs one scene-specific action:

1. Dam Break: drag the obstacle.
1. Fountain: change or aim the jet.
1. Float or Sink: drop the selected body.
1. Color Mixer: stir the particle groups.
1. Jelly Drop: poke the jelly.
1. Water Wheel: aim or strengthen the jet.

Each silent clip represents exactly eight seconds of simulation at 60 engine
steps per second and 30 output frames per second. The interaction occurs at a
fixed engine step using semantic controls or fixed canvas coordinates. Every
output frame is a numbered PNG captured from the stable `.player-panel` region
after exactly two engine steps. The player region includes status, canvas,
interaction hint, playback controls, scene controls, and credits while
excluding unrelated page chrome and volatile build-provenance text.

The script waits for the scene to report `Playing`, verifies the expected scene
ID and starting step, and drives the same step and interaction schedule on every
run. `ffmpeg` converts the numbered PNG sequence into:

- an H.264, browser-compatible MP4
- a reduced-size animated WebP suitable for README display

Encoding uses fixed frame rate, dimensions, codec settings, metadata, timebase,
and output ordering. Creation-time and host-specific metadata are omitted. The
script fails with an actionable message when the pinned Chromium, `ffmpeg`,
`ffprobe`, production preview, scene route, expected player state, or capture
region is unavailable.

## Determinism and idempotence contract

The workflow is deterministic within one recorded capture profile: a digest of
the web and WASM sources, capture script, lockfiles, and capture configuration;
the operating system; pinned Playwright Chromium; `ffmpeg` version; fonts;
viewport; device-pixel ratio; and encoding settings. Generated media and the
README are excluded from the input digest so the output cannot invalidate its
own profile. Cross-platform byte-for-byte identity is not claimed because
browser rasterization and codec implementations can differ between operating
systems or tool versions.

The script records the capture-profile identities and SHA-256 hashes in
`docs/assets/demos/manifest.json`. Generation writes all frames and encoded
media to a temporary directory, validates the complete set, and swaps the
complete output directory into place only after every scene succeeds. A failed
swap restores the previous directory. Existing committed media remains
untouched after any earlier failure.

Two modes are exposed through thin `just` recipes:

- `just demo-media` regenerates all scenes and replaces only files whose bytes
  changed.
- `just demo-media-check` regenerates into a temporary directory and fails when
  any committed media file or manifest entry is missing or stale.

Running generation twice with the same capture profile must produce identical
hashes and leave the worktree unchanged on the second run.

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
docs/assets/demos/manifest.json
```

The existing README maturity and compatibility statements remain unchanged.
The gallery describes the recordings as demonstrations, not parity evidence.

## Verification

Implementation is complete when:

1. All six MP4 files, six animated WebP files, and the manifest exist and are
   non-empty.
1. `ffprobe` can read every MP4 and WebP, each clip represents exactly eight
   seconds, and every output has the configured dimensions and 30 fps frame
   rate.
1. Every manifest hash matches its committed media file and records the capture
   profile used to produce it.
1. Two consecutive generation runs under the same capture profile produce
   identical hashes; the second run leaves no git diff.
1. `just demo-media-check` detects a deliberately changed or missing output.
1. Every README preview path, video link, and hosted scene link resolves.
1. The capture script is rerunnable from a clean generated web build, and an
   injected scene failure leaves all previously committed media untouched.
1. `just web-player-smoke` passes.
1. `just markdown-check` passes for the README and design documentation.
1. The managed Bright Builds check is run and any unrelated existing findings
   are reported without being misrepresented as introduced by this work.

## Out of scope

- Audio, narration, captions, and background music
- Mobile and browser-matrix recordings
- Replacing the live playground with video
- Treating recordings as physics parity or release evidence
