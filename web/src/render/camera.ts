/** Fixed proof-scene bounds supplied by the Rust session. */
export const WORLD_BOUNDS = {
  minX: -6,
  minY: -1,
  maxX: 6,
  maxY: 8,
} as const;

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

/** Aspect-preserving transform from the fixed world into CSS pixels. */
export interface Camera {
  readonly viewport: ViewportSize;
  readonly scale: number;
  readonly offsetX: number;
  readonly offsetY: number;
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

/** Creates the fixed-world projection for a positive finite CSS viewport. */
export function createCamera(width: number, height: number): Camera {
  const viewport = parseViewportSize(width, height);
  const worldWidth = WORLD_BOUNDS.maxX - WORLD_BOUNDS.minX;
  const worldHeight = WORLD_BOUNDS.maxY - WORLD_BOUNDS.minY;
  const availableWidth = viewport.width - VIEWPORT_INSET * 2;
  const availableHeight = viewport.height - VIEWPORT_INSET * 2;
  const scale = Math.min(
    availableWidth / worldWidth,
    availableHeight / worldHeight,
  );

  return {
    viewport,
    scale,
    offsetX: (viewport.width - worldWidth * scale) / 2,
    offsetY: (viewport.height - worldHeight * scale) / 2,
  };
}

/** Projects one world-space point, inverting only the world y-axis. */
export function projectPoint(camera: Camera, point: Point): Point {
  return {
    x: camera.offsetX + (point.x - WORLD_BOUNDS.minX) * camera.scale,
    y: camera.offsetY + (WORLD_BOUNDS.maxY - point.y) * camera.scale,
  };
}

/** Inverts projectPoint from CSS pixels back into the shared world bounds. */
export function unprojectPoint(camera: Camera, point: Point): Point {
  return {
    x: WORLD_BOUNDS.minX + (point.x - camera.offsetX) / camera.scale,
    y: WORLD_BOUNDS.maxY - (point.y - camera.offsetY) / camera.scale,
  };
}

/** Projects one world-space radius into CSS pixels. */
export function projectRadius(camera: Camera, radius: number): number {
  return radius * camera.scale;
}
