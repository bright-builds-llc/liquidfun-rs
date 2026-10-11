import type { ScreenRect } from "./safe-frame";

/** Axis-aligned world rectangle fitted to the canvas. */
export type WorldBounds = {
  readonly minX: number;
  readonly minY: number;
  readonly maxX: number;
  readonly maxY: number;
};

/**
 * Default proof-scene bounds.
 *
 * Scenes that need their own frame, such as Liquid Tumbler and Theo Jansen,
 * pass a rectangle. Every other scene keeps this 12 m by 9 m frame.
 */
export const WORLD_BOUNDS: WorldBounds = {
  minX: -6,
  minY: -1,
  maxX: 6,
  maxY: 8,
};

/** Internal Canvas padding reserved around projected world geometry. */
export const VIEWPORT_INSET = 16;

const INVALID_VIEWPORT_MESSAGE = "Invalid Canvas viewport";

/** Positive finite CSS-pixel dimensions used by the projection. */
export interface ViewportSize {
  readonly width: number;
  readonly height: number;
}

/** A point in either world or CSS-pixel coordinates. */
export interface Point {
  readonly x: number;
  readonly y: number;
}

/** Aspect-preserving transform from one world rectangle into CSS pixels. */
export interface Camera {
  readonly viewport: ViewportSize;
  readonly bounds: WorldBounds;
  readonly scale: number;
  readonly offsetX: number;
  readonly offsetY: number;
}

/** User zoom and pan applied on top of the fitted world view. */
export interface CameraView {
  readonly zoom: number;
  readonly panX: number;
  readonly panY: number;
}

export const IDENTITY_CAMERA_VIEW: CameraView = {
  zoom: 1,
  panX: 0,
  panY: 0,
};

export const MIN_CAMERA_ZOOM = 0.25;
export const MAX_CAMERA_ZOOM = 8;
export const CAMERA_ZOOM_FACTOR = 1.25;

export function clampCameraZoom(zoom: number): number {
  if (!Number.isFinite(zoom)) {
    return IDENTITY_CAMERA_VIEW.zoom;
  }

  return Math.min(MAX_CAMERA_ZOOM, Math.max(MIN_CAMERA_ZOOM, zoom));
}

export function zoomCameraView(
  view: CameraView,
  direction: "in" | "out",
): CameraView {
  const factor = direction === "in" ? CAMERA_ZOOM_FACTOR : 1 / CAMERA_ZOOM_FACTOR;
  return {
    ...view,
    zoom: clampCameraZoom(view.zoom * factor),
  };
}

function parseViewportSize(width: number, height: number): ViewportSize {
  if (
    !Number.isFinite(width) ||
    !Number.isFinite(height) ||
    width <= 0 ||
    height <= 0
  ) {
    throw new Error(INVALID_VIEWPORT_MESSAGE);
  }

  const availableWidth = width - VIEWPORT_INSET * 2;
  const availableHeight = height - VIEWPORT_INSET * 2;
  if (availableWidth <= 0 || availableHeight <= 0) {
    throw new Error(INVALID_VIEWPORT_MESSAGE);
  }

  return { width, height };
}

function parseFrame(frame: ScreenRect): ScreenRect {
  const edges = [frame.left, frame.top, frame.right, frame.bottom];
  if (!edges.every(Number.isFinite)) {
    throw new Error(INVALID_VIEWPORT_MESSAGE);
  }

  const availableWidth = frame.right - frame.left - VIEWPORT_INSET * 2;
  const availableHeight = frame.bottom - frame.top - VIEWPORT_INSET * 2;
  if (availableWidth <= 0 || availableHeight <= 0) {
    throw new Error(INVALID_VIEWPORT_MESSAGE);
  }

  return frame;
}

/**
 * Creates the world projection for a positive finite CSS viewport.
 *
 * The optional `frame` is the canvas rectangle the world is fitted into; it
 * defaults to the full viewport. The inset applies inside the frame and the
 * world centres within it, while `camera.viewport` stays the full canvas.
 */
export function createCamera(
  width: number,
  height: number,
  view: CameraView = IDENTITY_CAMERA_VIEW,
  bounds: WorldBounds = WORLD_BOUNDS,
  frame?: ScreenRect,
): Camera {
  const viewport = parseViewportSize(width, height);
  const fitFrame = parseFrame(
    frame ?? { left: 0, top: 0, right: viewport.width, bottom: viewport.height },
  );
  const frameWidth = fitFrame.right - fitFrame.left;
  const frameHeight = fitFrame.bottom - fitFrame.top;
  const worldWidth = bounds.maxX - bounds.minX;
  const worldHeight = bounds.maxY - bounds.minY;
  const availableWidth = frameWidth - VIEWPORT_INSET * 2;
  const availableHeight = frameHeight - VIEWPORT_INSET * 2;
  const fittedScale = Math.min(
    availableWidth / worldWidth,
    availableHeight / worldHeight,
  );
  const scale = fittedScale * clampCameraZoom(view.zoom);

  return {
    viewport,
    bounds,
    scale,
    offsetX: fitFrame.left + (frameWidth - worldWidth * scale) / 2 + view.panX,
    offsetY: fitFrame.top + (frameHeight - worldHeight * scale) / 2 + view.panY,
  };
}

/** Projected world rectangle in canvas CSS pixels. */
export function projectedWorldRect(camera: Camera): ScreenRect {
  const { bounds, scale } = camera;
  return {
    left: camera.offsetX,
    top: camera.offsetY,
    right: camera.offsetX + (bounds.maxX - bounds.minX) * scale,
    bottom: camera.offsetY + (bounds.maxY - bounds.minY) * scale,
  };
}

/** Projects one world-space point, inverting only the world y-axis. */
export function projectPoint(camera: Camera, point: Point): Point {
  return {
    x: camera.offsetX + (point.x - camera.bounds.minX) * camera.scale,
    y: camera.offsetY + (camera.bounds.maxY - point.y) * camera.scale,
  };
}

/** Inverts projectPoint from CSS pixels back into the camera's world bounds. */
export function unprojectPoint(camera: Camera, point: Point): Point {
  return {
    x: camera.bounds.minX + (point.x - camera.offsetX) / camera.scale,
    y: camera.bounds.maxY - (point.y - camera.offsetY) / camera.scale,
  };
}

/** Projects one world-space radius into CSS pixels. */
export function projectRadius(camera: Camera, radius: number): number {
  return radius * camera.scale;
}
