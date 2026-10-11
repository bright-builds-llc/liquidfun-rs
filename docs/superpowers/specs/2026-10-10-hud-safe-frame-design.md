# HUD Safe Frame Design

**Date:** 2026-10-10
**Status:** Awaiting owner review

## Goal

On-canvas HUD sliders must not cover a scene's content on any viewport, from an
iPhone in portrait to a wide desktop. Add a persistent HUD button that shows
and hides those sliders. The fix must be sustainable: new scenes, new sliders,
and new chrome get correct framing without per-scene tuning.

## Problem

The default canvas stage floats its chrome over the canvas with absolute
positioning. Up to four HUD sliders (Wave tank) sit in a card at most 22rem
wide above the transport, scale legend, and zoom toolbar. `createCamera` in
`web/src/render/camera.ts` contain-fits the world rectangle with a uniform
16px `VIEWPORT_INSET` and knows nothing about the chrome. The only protection
today is hand-tuned empty space in Tesla valve's portrait frame
(`TESLA_VALVE_PORTRAIT` in `web/src/catalog/portrait-bounds.ts`).

## Decisions

1. The camera fits the world around measured chrome rather than moving the
   sliders off the canvas or accepting overlap.
1. Fitting uses best fit among obstacle rectangles, not top and bottom band
   insets, so wide viewports can place the scene beside the slider card at
   full height.
1. Sliders are shown by default. Hiding them is one global preference that
   persists across scenes and reloads.
1. The toggle sits at the leading edge of the bottom toolbar, under the card
   it controls, not in the top-right corner beside the Controls drawer
   trigger.
1. When the best obstacle-free fit is below 50% of the unobstructed scale,
   ignore obstacles and accept overlap; the toggle gives a clean view.

## Scope

In scope: the canvas-stage layout used by default on every form factor.

Out of scope and unchanged:

- The classic `?shell=form-factor` layout, which already places sliders below
  the canvas.
- SVG, README, and recording exports, which have no chrome.
- Gravity and other panel controls, which stay in the Controls drawer.

## Component 1: Show/hide toggle

- Render a `hud-button` toggle at the leading edge of `.canvas-hud-toolbar`
  in `CanvasHud` (`web/src/components/CanvasStage.tsx`).
- Render it only when the scene has at least one HUD control, as reported by
  `sceneControlsForSurface(controls, "hud")`. This includes the
  washing-machine spinner.
- Set `aria-expanded` to the card's visibility and `aria-controls` to the
  card's id. The accessible label is "Hide sliders" when shown and
  "Show sliders" when hidden. The icon is a slider glyph, struck through
  when hidden.
- When hidden, remove the card from the DOM rather than hiding it visually,
  so it stops being an obstacle and cannot receive taps.
- Persist the choice in `web/src/player/hud-sliders-preference.ts`, following
  `web/src/render/mode.ts`: key `liquidfun.hud-sliders.v1`, values `"shown"`
  and `"hidden"`, default `"shown"`. Wrap every storage access in try/catch;
  any failure or unknown value reads as `"shown"`.

## Component 2: Safe frame core

New pure module `web/src/render/safe-frame.ts`:

```ts
/** CSS-pixel rectangle in canvas coordinates. */
export interface ScreenRect {
  readonly left: number;
  readonly top: number;
  readonly right: number;
  readonly bottom: number;
}

/** Result of fitting a world aspect around HUD obstacles. */
export interface SafeFrame {
  readonly frame: ScreenRect;
  readonly usedFallback: boolean;
}

export function safeFrame(
  viewport: ViewportSize,
  obstacles: readonly ScreenRect[],
  worldAspect: number,
): SafeFrame;
```

Algorithm:

1. Inflate each obstacle by an 8px gap and clip it to the viewport.
1. Candidate x edges are the viewport's left and right plus every inflated
   obstacle's left and right; candidate y edges likewise.
1. For every pair of x edges and every pair of y edges, reject the rectangle
   if it intersects any inflated obstacle with positive area.
1. Score each remaining rectangle by the scale it allows:
   `min((w - 2 * VIEWPORT_INSET) / worldW, (h - 2 * VIEWPORT_INSET) / worldH)`,
   using `worldAspect` for the ratio. Non-positive scores are rejected.
1. Among rectangles within 0.5% of the best score, prefer the one whose
   centre is nearest the viewport centre, then the lowest `top`, then the
   lowest `left`, so results do not depend on obstacle order.
1. If no rectangle remains, or the best score is below 50% of the score of
   the full viewport, return the full viewport with `usedFallback: true`.

The function runs on layout changes only, not per frame. With about eight
obstacles there are roughly 18 edges per axis and about 20,000 candidates.

## Component 3: Obstacle measurement shell

New module `web/src/player/hud-obstacles.ts`:

- Chrome leaf elements carry a `data-hud-obstacle` attribute: the title
  heading plus status, the stats column (FPS, debug toggle, and the debug
  readout when open), the corner button group, the slider card, the
  transport cluster, the trailing legend and zoom cluster, and the new
  toggle. Full-width containers such as `.canvas-hud-title` and
  `.canvas-hud-bottom` are not obstacles.
- One `ResizeObserver` watches the marked elements and the canvas. A
  `MutationObserver` on the HUD root picks up elements that appear or
  disappear.
- On each change, convert each element's `getBoundingClientRect()` into
  canvas coordinates and publish a Solid signal of `readonly ScreenRect[]`.
- Future chrome joins the fit by adding the attribute; no per-scene numbers
  are involved.

## Component 4: Camera integration

- `createCamera` gains an optional final `frame: ScreenRect` parameter that
  defaults to the full viewport. The inset applies inside `frame`, and the
  world centres within `frame`. Without the argument, output is identical to
  today, which keeps exports and the classic layout unchanged.
- `resizeCanvasBackingStore` forwards the frame.
- The scene clock gains `hudObstacles: readonly ScreenRect[]`, empty in the
  classic layout.
- A helper `cameraFrame(clock)` calls
  `safeFrame(viewport, clock.hudObstacles, aspectOf(clock.worldBounds))`. Both
  live camera call sites use it: the canvas `ResizeObserver` in
  `web/src/player/frame-loop.ts` and `refreshCamera` in
  `web/src/player/scene-surface.ts`.
- When the obstacle signal changes, call `session.refreshCamera()` and repaint
  the held frame if paused.
- The canvas exposes `data-world-frame="left,top,right,bottom"` with the
  projected world rectangle and `data-world-frame-fallback="true"` when the
  fallback applied.
- `worldBoundsForViewport` still chooses portrait or landscape bounds from the
  canvas aspect; the safe frame only places those bounds.
- User zoom multiplies the fitted scale and pan adds to the offset, as today.
  Reset zoom returns to the safe framing.
- The refit is instant. Add easing only if it feels jarring in review.

Both observers fire on a viewport resize. Each recomputes from current
viewport and obstacles, so the camera converges within one frame.

## Cleanup

- Remove the built-in empty space below the drain from
  `TESLA_VALVE_PORTRAIT`, keeping `portrait-bounds.test.ts` fill and wall
  checks passing.
- Update the "Playground canvas framing" repo-local guidance in `AGENTS.md`
  and `CLAUDE.md`: portrait frames still fit the subject tightly, but keeping
  the subject clear of chrome is automatic, and new chrome must carry
  `data-hud-obstacle`.

## Testing

Unit tests (vitest, Arrange/Act/Assert):

- `safe-frame.test.ts`: no obstacles returns the full viewport; a bottom band
  returns the region above it; a narrow bottom-left card on a wide viewport
  returns the full-height region to its right; equal side regions choose the
  one nearer the centre; shuffled obstacles return the same frame; a
  full-canvas obstacle triggers the fallback; the 8px gap is respected.
- `camera.test.ts`: no frame argument matches the current output; a frame
  centres the world inside it with the inset applied inside it; zoom and pan
  compose as before.
- `hud-sliders-preference.test.ts`: default is shown; a choice round-trips;
  throwing storage or an unknown value reads as shown.
- Component test with the same DOM setup as `scale-legend.test.ts`: the toggle
  renders only for scenes with HUD controls; a click flips `aria-expanded`,
  removes the card, and persists the choice.

Playwright (`web/e2e/hud-safe-frame.spec.ts`), with scenes enumerated from the
catalog so new scenes are covered automatically:

- Every scene at 390x844 and 1280x800, and HUD-control scenes also at 844x390
  and 768x1024: the `data-world-frame` rectangle intersects no
  `[data-hud-obstacle]` box beyond a 1px tolerance.
- When `data-world-frame-fallback` is set, assert instead that hiding the
  sliders clears the overlap and the fallback flag, and annotate the case in
  the report.
- Toggle round trip: hiding grows the frame, the hidden state survives a
  reload, and showing restores the original frame.

## Done criteria

- `bun run test:unit`, typecheck, lint, and build pass in `web/`.
- The new Playwright spec and the existing `canvas-stage`, `shell`, and
  `player` specs pass.
- `bun scripts/bright-builds-check.ts all` passes.
- `just markdown-check` passes for documentation changes.
- Visual check in the browser at 390x844 and 1280x800 on Wave tank and Tesla
  valve, before and after.
