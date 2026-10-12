import { createSignal, onCleanup, type Accessor } from "solid-js";

import {
  sameScreenRects,
  screenRectInCanvas,
  type ScreenRect,
} from "../render/safe-frame";

/** Chrome leaves the camera fits the world around. */
export const HUD_OBSTACLE_SELECTOR = "[data-hud-obstacle]";

function markedElements(root: HTMLElement): HTMLElement[] {
  return Array.from(root.querySelectorAll<HTMLElement>(HUD_OBSTACLE_SELECTOR));
}

function measureObstacles(
  root: HTMLElement,
  canvas: HTMLElement,
): readonly ScreenRect[] {
  const canvasRect = canvas.getBoundingClientRect();
  return markedElements(root)
    .map((element) => element.getBoundingClientRect())
    .filter((rect) => rect.width > 0 && rect.height > 0)
    .map((rect) => screenRectInCanvas(rect, canvasRect.left, canvasRect.top));
}

/**
 * Publishes canvas-space boxes of every `data-hud-obstacle` element in `root`.
 *
 * A ResizeObserver watches the canvas and each marked element, and a
 * MutationObserver re-syncs that set when chrome appears or disappears.
 * The first measurement is synchronous, so callers must create this after the
 * chrome is attached (for example in `onMount`). Later measurements coalesce
 * into one animation frame, and the signal ignores sub-pixel churn, so text
 * updates such as the FPS counter do not refit the camera. Must be called
 * inside a Solid owner; observers stop on cleanup.
 */
export function createHudObstacles(
  root: HTMLElement,
  canvas: HTMLElement,
): Accessor<readonly ScreenRect[]> {
  const [obstacles, setObstacles] = createSignal<readonly ScreenRect[]>([], {
    equals: sameScreenRects,
  });
  let maybePendingFrameId: number | undefined;

  const scheduleMeasure = () => {
    if (maybePendingFrameId !== undefined) {
      return;
    }
    maybePendingFrameId = requestAnimationFrame(() => {
      maybePendingFrameId = undefined;
      setObstacles(measureObstacles(root, canvas));
    });
  };

  const resizeObserver = new ResizeObserver(scheduleMeasure);
  const observeMarked = () => {
    resizeObserver.disconnect();
    resizeObserver.observe(canvas);
    for (const element of markedElements(root)) {
      resizeObserver.observe(element);
    }
  };
  const mutationObserver = new MutationObserver(() => {
    observeMarked();
    scheduleMeasure();
  });

  observeMarked();
  mutationObserver.observe(root, { childList: true, subtree: true });
  // Measure now, not on the next frame: a route that builds within the
  // current task must already fit its first frame around the mounted chrome.
  setObstacles(measureObstacles(root, canvas));

  onCleanup(() => {
    mutationObserver.disconnect();
    resizeObserver.disconnect();
    if (maybePendingFrameId !== undefined) {
      cancelAnimationFrame(maybePendingFrameId);
    }
  });

  return obstacles;
}
