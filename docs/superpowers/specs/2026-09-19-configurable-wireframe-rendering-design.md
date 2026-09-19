# Configurable Wireframe Rendering Design

**Date:** 2026-09-19
**Status:** Approved for implementation

## Goal

Make wireframe rendering the default presentation for every web playground
scene while allowing visitors to switch the complete renderer between
wireframe and solid modes. The selection applies globally, survives scene
switches and browser reloads, and never changes simulation state.

## Rendering model

Introduce a two-value `RenderMode` domain type:

- `wireframe`
- `solid`

`drawRenderFrame` receives the mode explicitly with the frame and camera.
Rendering policy does not enter the Rust/WASM boundary, scene definitions, or
physics session.

Wireframe mode:

- clears the canvas with the existing background color
- strokes each particle circle using that particle's existing RGBA color
- does not fill particle circles
- preserves the existing stroked rigid segments
- strokes rigid circles with the existing rigid-body stroke color
- does not fill rigid circles

Solid mode preserves the current renderer behavior exactly: particles use
their RGBA fills, rigid segments remain stroked, and rigid circles use the
existing fill and stroke colors.

Stroke widths must remain visible at supported canvas scales without changing
the captured physical radius. Particle alpha remains the simulation-provided
alpha; the renderer does not force a minimum opacity.

## Preference state and persistence

The application owns one global render-mode signal. Its initial value is read
from a versioned local-storage key. Missing, invalid, inaccessible, or
exception-throwing storage falls back to `wireframe`.

Changing the preference:

1. updates in-memory state immediately
1. attempts to persist the new mode
1. redraws the latest captured frame with the existing camera
1. does not call `nextFrame`, recreate the session, reset observations, or
   change the step index

Storage write failures do not poison or dispose the active scene. The in-memory
selection remains active for the current page.

## User interface

Add one labeled `Rendering` select to the shared player controls:

- `Wireframe`
- `Solid`

The control is global rather than scene-specific and appears beside the
existing Play, Pause, and Reset controls. It remains usable whenever the player
is mounted because it does not depend on WASM session readiness. Keyboard and
focus behavior follow the existing select and focus-visible styles.

Expose the active value as `data-render-mode` on the application root for
browser verification and support diagnostics.

## Architecture

Keep the mode parser and persistence adapter separate from Canvas effects:

- a pure parser accepts only the two allowlisted values
- a storage adapter handles read/write failures and the versioned key
- the Canvas renderer accepts a parsed `RenderMode`
- the SolidJS application coordinates the signal, control, redraw, and storage

This preserves a functional core around mode parsing and draw-policy decisions
with thin local-storage and DOM shells.

## Tests and verification

Unit coverage must prove:

1. missing and invalid persisted values default to wireframe
1. both allowlisted values parse correctly
1. storage read/write exceptions do not escape
1. wireframe particles and rigid circles stroke without filling
1. particle stroke colors preserve frame RGBA values
1. solid mode retains the existing fill/stroke behavior
1. segments render identically in both modes

Chromium coverage must prove:

1. a fresh browser context starts in wireframe mode
1. switching to solid changes Canvas pixels without advancing the step index
1. the selected mode survives a scene switch
1. the selected mode survives a reload
1. switching back to wireframe restores wireframe pixels without recreating the
   session

Run the complete web player smoke, Markdown check, managed Bright Builds check,
and relevant Rust/WASM tests. The current `main` baseline has no failing CI
jobs, so the implementation must preserve that state.

## Deterministic demo media

The committed README recordings were generated from the old solid default.
After implementation:

1. regenerate all six MP4 recordings, animated WebP previews, and the manifest
   with `just demo-media`
1. prove a second generation is byte-identical
1. run `just demo-media-check`
1. visually inspect representative wireframe frames from every scene
1. retain the README links and gallery structure

The manifest's capture-input digest, media hashes, and font/runtime profile must
match the final committed source. Capture defaults to wireframe in a fresh
browser context.

## Out of scope

- Independent particle and rigid-object mode controls
- Per-scene rendering preferences
- Additional rendering styles such as points, heat maps, or mixed mode
- Changes to Rust physics, WASM frame lanes, scene recipes, or public crate APIs
- Forced minimum particle opacity
