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
 * Scenes that model a smaller world, such as Liquid Tumbler, pass their own
 * rectangle. Every other scene keeps this 12 m by 9 m frame.
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

/** Creates the world projection for a positive finite CSS viewport. */
export function createCamera(
  width: number,
  height: number,
  view: CameraView = IDENTITY_CAMERA_VIEW,
  bounds: WorldBounds = WORLD_BOUNDS,
): Camera {
  const viewport = parseViewportSize(width, height);
  const worldWidth = bounds.maxX - bounds.minX;
  const worldHeight = bounds.maxY - bounds.minY;
  const availableWidth = viewport.width - VIEWPORT_INSET * 2;
  const availableHeight = viewport.height - VIEWPORT_INSET * 2;
  const fittedScale = Math.min(
    availableWidth / worldWidth,
    availableHeight / worldHeight,
  );
  const scale = fittedScale * clampCameraZoom(view.zoom);

  return {
    viewport,
    bounds,
    scale,
    offsetX: (viewport.width - worldWidth * scale) / 2 + view.panX,
    offsetY: (viewport.height - worldHeight * scale) / 2 + view.panY,
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
