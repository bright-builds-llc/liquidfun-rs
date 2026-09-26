import type { RenderFrame } from "../physics/frame";
import {
  type Camera,
  type CameraView,
  type WorldBounds,
  IDENTITY_CAMERA_VIEW,
  WORLD_BOUNDS,
  createCamera,
  projectPoint,
  projectRadius,
} from "./camera";
import { paintContour } from "./contour";
import { paintMetaball } from "./metaball";
import {
  rigidRenderMode,
  usesParticleStride,
  type CircleRenderMode,
  type RenderMode,
} from "./mode";
import { eachProjectedParticle } from "./projected-particle";
import {
  DEFAULT_WIREFRAME_STROKE_WIDTH,
  maybeParseWireframeStrokeWidth,
} from "./stroke-width";

const CANVAS_COLOR = "#071018";
const BASIN_STROKE_COLOR = "#94A3B8";
const RIGID_FILL_COLOR = "#334155";
const RIGID_STROKE_COLOR = "#CBD5E1";
const RIGID_STROKE_WIDTH = 2;
const MAX_DEVICE_PIXEL_RATIO = 2;

/** Caps the backing-store scale shared by the 2D and shaded-blob canvases. */
export function cappedDevicePixelRatio(devicePixelRatio: number): number {
  return Math.min(requirePositiveFinite(devicePixelRatio), MAX_DEVICE_PIXEL_RATIO);
}

/** Particle surface painters. Tests replace these so node can skip real canvases. */
export type SurfacePainters = {
  readonly paintMetaball: typeof paintMetaball;
  readonly paintContour: typeof paintContour;
};

const DEFAULT_SURFACE_PAINTERS: SurfacePainters = {
  paintMetaball,
  paintContour,
};
const INVALID_CANVAS_MESSAGE = "Invalid Canvas dimensions";
const INVALID_RENDER_FRAME_MESSAGE = "Invalid renderer frame";
const TAU = Math.PI * 2;

function requirePositiveFinite(value: number): number {
  if (!Number.isFinite(value) || value <= 0) {
    throw new Error(INVALID_CANVAS_MESSAGE);
  }

  return value;
}

function valueAt(
  values: Float32Array | Uint8Array,
  index: number,
): number {
  const value = values[index];
  if (value === undefined) {
    throw new Error(INVALID_RENDER_FRAME_MESSAGE);
  }

  return value;
}

/**
 * Sizes a Canvas backing store for its CSS viewport and returns its camera.
 *
 * The drawing transform keeps all subsequent coordinates in CSS pixels.
 */
export function resizeCanvasBackingStore(
  canvas: HTMLCanvasElement,
  cssWidth: number,
  cssHeight: number,
  devicePixelRatio: number,
  view: CameraView = IDENTITY_CAMERA_VIEW,
  bounds: WorldBounds = WORLD_BOUNDS,
): Camera {
  const camera = createCamera(cssWidth, cssHeight, view, bounds);
  const pixelRatio = cappedDevicePixelRatio(devicePixelRatio);

  canvas.width = Math.round(cssWidth * pixelRatio);
  canvas.height = Math.round(cssHeight * pixelRatio);

  const maybeContext = canvas.getContext("2d");
  if (maybeContext === null) {
    throw new Error("Canvas 2D is unavailable");
  }
  maybeContext.setTransform(pixelRatio, 0, 0, pixelRatio, 0, 0);
  maybeContext.fillStyle = CANVAS_COLOR;
  maybeContext.fillRect(0, 0, cssWidth, cssHeight);

  return camera;
}

function resolvedWireframeStrokeWidth(width: number): number {
  return (
    maybeParseWireframeStrokeWidth(String(width)) ??
    DEFAULT_WIREFRAME_STROKE_WIDTH
  );
}

function outlineWidth(
  renderMode: CircleRenderMode,
  wireframeStrokeWidth: number,
): number {
  if (renderMode === "wireframe") {
    return resolvedWireframeStrokeWidth(wireframeStrokeWidth);
  }

  return RIGID_STROKE_WIDTH;
}

function drawDiscParticles(
  context: CanvasRenderingContext2D,
  frame: RenderFrame,
  camera: Camera,
  renderMode: CircleRenderMode,
  wireframeStrokeWidth: number,
  maxRenderedParticles: number,
): void {
  const strokeWidth = resolvedWireframeStrokeWidth(wireframeStrokeWidth);
  eachProjectedParticle(
    frame,
    camera,
    maxRenderedParticles,
    (x, y, radius, red, green, blue, alpha) => {
      const color = `rgba(${red}, ${green}, ${blue}, ${alpha})`;
      context.beginPath();
      context.arc(x, y, radius, 0, TAU);
      if (renderMode === "wireframe") {
        context.strokeStyle = color;
        context.lineWidth = strokeWidth;
        context.stroke();
        return;
      }

      context.fillStyle = color;
      context.fill();
    },
  );
}

function drawSegments(
  context: CanvasRenderingContext2D,
  frame: RenderFrame,
  camera: Camera,
  renderMode: CircleRenderMode,
  wireframeStrokeWidth: number,
): void {
  context.strokeStyle = BASIN_STROKE_COLOR;
  context.lineWidth = outlineWidth(renderMode, wireframeStrokeWidth);

  for (
    let segmentIndex = 0;
    segmentIndex < frame.rigidSegments.length;
    segmentIndex += 4
  ) {
    const start = projectPoint(camera, {
      x: valueAt(frame.rigidSegments, segmentIndex),
      y: valueAt(frame.rigidSegments, segmentIndex + 1),
    });
    const end = projectPoint(camera, {
      x: valueAt(frame.rigidSegments, segmentIndex + 2),
      y: valueAt(frame.rigidSegments, segmentIndex + 3),
    });

    context.beginPath();
    context.moveTo(start.x, start.y);
    context.lineTo(end.x, end.y);
    context.stroke();
  }
}

function drawCircles(
  context: CanvasRenderingContext2D,
  frame: RenderFrame,
  camera: Camera,
  renderMode: CircleRenderMode,
  wireframeStrokeWidth: number,
): void {
  context.fillStyle = RIGID_FILL_COLOR;
  context.strokeStyle = RIGID_STROKE_COLOR;
  context.lineWidth = outlineWidth(renderMode, wireframeStrokeWidth);

  for (
    let circleIndex = 0;
    circleIndex < frame.rigidCircles.length;
    circleIndex += 3
  ) {
    const center = projectPoint(camera, {
      x: valueAt(frame.rigidCircles, circleIndex),
      y: valueAt(frame.rigidCircles, circleIndex + 1),
    });
    const radius = projectRadius(
      camera,
      valueAt(frame.rigidCircles, circleIndex + 2),
    );

    context.beginPath();
    context.arc(center.x, center.y, radius, 0, TAU);
    if (renderMode !== "wireframe") {
      context.fill();
    }
    context.stroke();
    drawCircleLabel(context, frame.circleLabels[circleIndex / 3], center, radius);
  }
}

function drawCircleLabel(
  context: CanvasRenderingContext2D,
  label: string | undefined,
  center: { readonly x: number; readonly y: number },
  radius: number,
): void {
  if (label === undefined || label.length === 0) {
    return;
  }

  const fontSize = Math.min(radius * 0.55, 28);
  if (fontSize < 8) {
    return;
  }

  context.save();
  context.fillStyle = "#F8FAFC";
  context.font = `600 ${fontSize}px sans-serif`;
  context.textAlign = "center";
  context.textBaseline = "middle";
  context.fillText(label, center.x, center.y);
  context.restore();
}

function clearFrame(
  context: CanvasRenderingContext2D,
  camera: Camera,
  webglCovered: boolean,
): void {
  if (webglCovered) {
    context.clearRect(0, 0, camera.viewport.width, camera.viewport.height);
    return;
  }

  context.fillStyle = CANVAS_COLOR;
  context.fillRect(0, 0, camera.viewport.width, camera.viewport.height);
}

function drawParticleSurface(
  context: CanvasRenderingContext2D,
  frame: RenderFrame,
  camera: Camera,
  renderMode: RenderMode,
  wireframeStrokeWidth: number,
  maxRenderedParticles: number,
  webglCovered: boolean,
  painters: SurfacePainters,
): void {
  const particleLimit = usesParticleStride(renderMode)
    ? maxRenderedParticles
    : Number.POSITIVE_INFINITY;
  if (renderMode === "wireframe" || renderMode === "solid") {
    drawDiscParticles(
      context,
      frame,
      camera,
      renderMode,
      wireframeStrokeWidth,
      particleLimit,
    );
    return;
  }
  if (renderMode === "contour") {
    painters.paintContour(context, frame, camera, particleLimit);
    return;
  }
  if (renderMode === "shaded-blob" && webglCovered) {
    return;
  }

  painters.paintMetaball(context, frame, camera, particleLimit);
}

/** Draws one validated bulk Rust frame in the approved presentation order. */
export function drawRenderFrame(
  context: CanvasRenderingContext2D,
  frame: RenderFrame,
  camera: Camera,
  renderMode: RenderMode,
  wireframeStrokeWidth = DEFAULT_WIREFRAME_STROKE_WIDTH,
  maxRenderedParticles = Number.POSITIVE_INFINITY,
  webglCovered = false,
  painters: SurfacePainters = DEFAULT_SURFACE_PAINTERS,
): void {
  clearFrame(context, camera, webglCovered);
  drawParticleSurface(
    context,
    frame,
    camera,
    renderMode,
    wireframeStrokeWidth,
    maxRenderedParticles,
    webglCovered,
    painters,
  );
  const rigidMode = rigidRenderMode(renderMode);
  drawSegments(context, frame, camera, rigidMode, wireframeStrokeWidth);
  drawCircles(context, frame, camera, rigidMode, wireframeStrokeWidth);
}
