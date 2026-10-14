import type { RenderFrame } from "../physics/frame";
import { drawRenderFrame } from "./canvas";
import type { Camera } from "./camera";
import { particleSurface, type RenderMode } from "./mode";
import { drawShadedBlob } from "./webgl-particles";
import { drawWater } from "./webgl-water";

/** The particle surface that actually drew a frame (D-15). */
export type DrawnSurface = "webgl-water" | "webgl" | "canvas";

/**
 * Draws one frame and reports which surface drew its particles.
 *
 * One fallback chain per frame: Water, then shaded blob, then the canvas soft
 * blob (D-12). Never presents a blank frame.
 */
export function presentSceneFrame(
  context: CanvasRenderingContext2D,
  frame: RenderFrame,
  camera: Camera,
  mode: RenderMode,
  wireframeStrokeWidth: number,
  maxRenderedParticles: number,
  maybeParticleCanvas: HTMLCanvasElement | undefined,
  devicePixelRatio: number,
  densityShading = true,
): DrawnSurface {
  const surface = particleSurface(mode);
  const water =
    surface === "webgl-water" &&
    drawWater(maybeParticleCanvas, frame, camera, devicePixelRatio, densityShading);
  const blob =
    !water &&
    (surface === "webgl" || surface === "webgl-water") &&
    drawShadedBlob(
      maybeParticleCanvas,
      frame,
      camera,
      devicePixelRatio,
      densityShading,
    );
  drawRenderFrame(
    context,
    frame,
    camera,
    mode,
    wireframeStrokeWidth,
    maxRenderedParticles,
    water || blob,
    densityShading,
    undefined,
    devicePixelRatio,
  );
  if (water) {
    return "webgl-water";
  }

  return blob ? "webgl" : "canvas";
}
