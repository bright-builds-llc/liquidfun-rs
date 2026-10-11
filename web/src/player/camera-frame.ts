import {
  projectedWorldRect,
  type Camera,
  type WorldBounds,
} from "../render/camera";
import { safeFrame, type SafeFrame } from "../render/safe-frame";
import type { FrameClock } from "./frame-loop";

/** Clock fields that decide where the live camera fits the world. */
export type CameraFrameInputs = Pick<
  FrameClock,
  "viewportWidth" | "viewportHeight" | "hudObstacles" | "worldBounds"
>;

function aspectOf(bounds: WorldBounds): number {
  return (bounds.maxX - bounds.minX) / (bounds.maxY - bounds.minY);
}

/** Best obstacle-free canvas rectangle for the clock's world bounds. */
export function cameraFrame(clock: CameraFrameInputs): SafeFrame {
  return safeFrame(
    { width: clock.viewportWidth, height: clock.viewportHeight },
    clock.hudObstacles,
    aspectOf(clock.worldBounds),
  );
}

function roundedEdge(value: number): string {
  return String(Math.round(value * 100) / 100);
}

/**
 * Publishes the projected world rectangle on the canvas for browser checks.
 *
 * `data-world-frame` is `left,top,right,bottom` in canvas CSS pixels, and
 * `data-world-frame-fallback` is present only when obstacles were ignored.
 */
export function publishWorldFrame(
  canvas: HTMLCanvasElement,
  camera: Camera,
  usedFallback: boolean,
): void {
  const rect = projectedWorldRect(camera);
  canvas.dataset.worldFrame = [rect.left, rect.top, rect.right, rect.bottom]
    .map(roundedEdge)
    .join(",");
  if (usedFallback) {
    canvas.dataset.worldFrameFallback = "true";
    return;
  }

  delete canvas.dataset.worldFrameFallback;
}
