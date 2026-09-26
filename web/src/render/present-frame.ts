import type { RenderFrame } from "../physics/frame";
import { drawRenderFrame } from "./canvas";
import type { Camera } from "./camera";
import { needsWebglSurface, type RenderMode } from "./mode";
import { drawShadedBlob } from "./webgl-particles";

/** Draws one frame, using the WebGL canvas when the shaded blob is available. */
export function presentSceneFrame(
  context: CanvasRenderingContext2D,
  frame: RenderFrame,
  camera: Camera,
  mode: RenderMode,
  wireframeStrokeWidth: number,
  maxRenderedParticles: number,
  maybeParticleCanvas: HTMLCanvasElement | undefined,
  devicePixelRatio: number,
): void {
  const webglCovered =
    needsWebglSurface(mode) &&
    drawShadedBlob(maybeParticleCanvas, frame, camera, devicePixelRatio);
  drawRenderFrame(
    context,
    frame,
    camera,
    mode,
    wireframeStrokeWidth,
    maxRenderedParticles,
    webglCovered,
  );
}
