import type { RenderFrame } from "../physics/frame";
import {
  type Camera,
  createCamera,
  projectPoint,
  projectRadius,
} from "./camera";

const CANVAS_COLOR = "#071018";
const BASIN_STROKE_COLOR = "#94A3B8";
const RIGID_FILL_COLOR = "#334155";
const RIGID_STROKE_COLOR = "#CBD5E1";
const RIGID_STROKE_WIDTH = 2;
const MAX_DEVICE_PIXEL_RATIO = 2;
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
): Camera {
  const camera = createCamera(cssWidth, cssHeight);
  const pixelRatio = Math.min(
    requirePositiveFinite(devicePixelRatio),
    MAX_DEVICE_PIXEL_RATIO,
  );

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

function drawParticles(
  context: CanvasRenderingContext2D,
  frame: RenderFrame,
  camera: Camera,
): void {
  for (
    let particleIndex = 0;
    particleIndex < frame.particleCount;
    particleIndex += 1
  ) {
    const positionIndex = particleIndex * 2;
    const colorIndex = particleIndex * 4;
    const center = projectPoint(camera, {
      x: valueAt(frame.particlePositions, positionIndex),
      y: valueAt(frame.particlePositions, positionIndex + 1),
    });
    const radius = projectRadius(
      camera,
      valueAt(frame.particleRadii, particleIndex),
    );
    const red = valueAt(frame.particleColors, colorIndex);
    const green = valueAt(frame.particleColors, colorIndex + 1);
    const blue = valueAt(frame.particleColors, colorIndex + 2);
    const alpha = valueAt(frame.particleColors, colorIndex + 3) / 255;

    context.beginPath();
    context.arc(center.x, center.y, radius, 0, TAU);
    context.fillStyle = `rgba(${red}, ${green}, ${blue}, ${alpha})`;
    context.fill();
  }
}

function drawSegments(
  context: CanvasRenderingContext2D,
  frame: RenderFrame,
  camera: Camera,
): void {
  context.strokeStyle = BASIN_STROKE_COLOR;
  context.lineWidth = RIGID_STROKE_WIDTH;

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
): void {
  context.fillStyle = RIGID_FILL_COLOR;
  context.strokeStyle = RIGID_STROKE_COLOR;
  context.lineWidth = RIGID_STROKE_WIDTH;

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
    context.fill();
    context.stroke();
  }
}

/** Draws one validated bulk Rust frame in the approved presentation order. */
export function drawRenderFrame(
  context: CanvasRenderingContext2D,
  frame: RenderFrame,
  camera: Camera,
): void {
  context.fillStyle = CANVAS_COLOR;
  context.fillRect(
    0,
    0,
    camera.viewport.width,
    camera.viewport.height,
  );

  drawParticles(context, frame, camera);
  drawSegments(context, frame, camera);
  drawCircles(context, frame, camera);
}
