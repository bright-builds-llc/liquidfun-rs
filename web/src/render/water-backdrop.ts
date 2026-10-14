/**
 * The refraction backdrop of the Water pass: a raster of the stable walls.
 *
 * D-07 refinement: `static-layer.ts` caches `Path2D` objects per 2D context
 * and holds no image, and it skips scenes under 32 segments. The refraction
 * background is therefore a water-owned backdrop canvas that draws the static
 * walls and is re-uploaded only when that geometry or view changes, reusing
 * static-layer's equality helpers so "static" has one definition. When no
 * stable raster exists the shader uses a procedural backdrop. Never upload a
 * texture every frame.
 *
 * The raster follows static-layer's stability rule: it is drawn only when a
 * frame equals the previous frame (same segment bits, camera and field size)
 * and differs from what was last drawn. Moving geometry never re-rasterizes.
 */
import type { RenderFrame } from "../physics/frame";
import { projectPoint, type Camera } from "./camera";
import { BASIN_STROKE_COLOR, CANVAS_COLOR, outlineWidth } from "./canvas";
import type { FieldSize } from "./field-pass";
import { sameCameraView, sameContent } from "./static-layer";
import { DEFAULT_WIREFRAME_STROKE_WIDTH } from "./stroke-width";

/** The part of a canvas the backdrop needs; tests inject a fake. */
export type BackdropCanvas = {
  width: number;
  height: number;
  getContext(contextId: "2d"): CanvasRenderingContext2D | null;
};

/** A walls-only raster that changes only when the static state settles. */
export type WaterBackdrop = {
  /** Compares the frame with the last one and rasterizes once it is stable. */
  update(frame: RenderFrame, camera: Camera, size: FieldSize): void;
  /** Increments on every raster, so the GL texture uploads only on change. */
  readonly revision: number;
  /** True when the raster shows exactly the current frame's walls and view. */
  readonly matchesView: boolean;
  /** The raster canvas, once one has been drawn. */
  readonly maybeRaster: BackdropCanvas | undefined;
  /** Forgets every key so the next stable frame rasterizes again. */
  reset(): void;
};

type FrameKey = {
  readonly content: Uint32Array;
  readonly camera: Camera;
  readonly width: number;
  readonly height: number;
  readonly pixelRatio: number;
};

function segmentBits(frame: RenderFrame): Uint32Array {
  return new Uint32Array(
    frame.rigidSegments.buffer,
    frame.rigidSegments.byteOffset,
    frame.rigidSegments.length,
  );
}

function sameKey(
  key: FrameKey,
  content: Uint32Array,
  camera: Camera,
  size: FieldSize,
): boolean {
  return (
    key.width === size.width &&
    key.height === size.height &&
    key.pixelRatio === size.pixelRatio &&
    sameCameraView(key.camera, camera) &&
    sameContent(key.content, content)
  );
}

function snapshot(
  content: Uint32Array,
  camera: Camera,
  size: FieldSize,
): FrameKey {
  return {
    content: content.slice(),
    camera: {
      ...camera,
      viewport: { ...camera.viewport },
      bounds: { ...camera.bounds },
    },
    width: size.width,
    height: size.height,
    pixelRatio: size.pixelRatio,
  };
}

function paintWalls(
  context: CanvasRenderingContext2D,
  segments: Float32Array,
  camera: Camera,
  size: FieldSize,
): void {
  context.setTransform(1, 0, 0, 1, 0, 0);
  context.fillStyle = CANVAS_COLOR;
  context.fillRect(0, 0, size.width, size.height);
  context.setTransform(size.pixelRatio, 0, 0, size.pixelRatio, 0, 0);
  context.strokeStyle = BASIN_STROKE_COLOR;
  context.lineWidth = outlineWidth("solid", DEFAULT_WIREFRAME_STROKE_WIDTH);
  // One path is enough here: the raster is only ever seen refracted.
  context.beginPath();
  for (let index = 0; index + 3 < segments.length; index += 4) {
    const start = projectPoint(camera, {
      x: segments[index] ?? 0,
      y: segments[index + 1] ?? 0,
    });
    const end = projectPoint(camera, {
      x: segments[index + 2] ?? 0,
      y: segments[index + 3] ?? 0,
    });
    context.moveTo(start.x, start.y);
    context.lineTo(end.x, end.y);
  }
  context.stroke();
}

/**
 * Creates the backdrop. The canvas is created lazily on the first stable
 * frame; a factory returning undefined (no DOM) leaves the procedural
 * backdrop in place without an error.
 */
export function createWaterBackdrop(
  createCanvas: () => BackdropCanvas | undefined,
): WaterBackdrop {
  let maybeCanvas: BackdropCanvas | undefined;
  let maybeContext: CanvasRenderingContext2D | undefined;
  let canvasUnavailable = false;
  let maybePrevious: FrameKey | undefined;
  let maybeRasterized: FrameKey | undefined;
  let revision = 0;
  let matchesView = false;

  const context = (): CanvasRenderingContext2D | undefined => {
    if (maybeContext !== undefined || canvasUnavailable) {
      return maybeContext;
    }
    maybeCanvas = createCanvas();
    maybeContext = maybeCanvas?.getContext("2d") ?? undefined;
    canvasUnavailable = maybeContext === undefined;
    return maybeContext;
  };

  const rasterize = (
    frame: RenderFrame,
    camera: Camera,
    size: FieldSize,
    key: FrameKey,
  ): boolean => {
    const maybeTarget = context();
    if (maybeTarget === undefined || maybeCanvas === undefined) {
      return false;
    }
    maybeCanvas.width = size.width;
    maybeCanvas.height = size.height;
    paintWalls(maybeTarget, frame.rigidSegments, camera, size);
    maybeRasterized = key;
    revision += 1;
    return true;
  };

  return {
    update(frame, camera, size) {
      if (frame.rigidSegments.length === 0) {
        maybePrevious = undefined;
        matchesView = false;
        return;
      }
      const content = segmentBits(frame);
      const previous = maybePrevious;
      if (previous === undefined || !sameKey(previous, content, camera, size)) {
        if (
          maybeRasterized !== undefined &&
          sameKey(maybeRasterized, content, camera, size)
        ) {
          // Back on the drawn state: reuse its key so nothing redraws.
          maybePrevious = maybeRasterized;
          matchesView = true;
          return;
        }
        // Copy only on change, so a static scene allocates nothing per frame.
        maybePrevious = snapshot(content, camera, size);
        matchesView = false;
        return;
      }
      if (previous === maybeRasterized) {
        matchesView = true;
        return;
      }
      matchesView = rasterize(frame, camera, size, previous);
    },
    get revision() {
      return revision;
    },
    get matchesView() {
      return matchesView;
    },
    get maybeRaster() {
      return maybeRasterized === undefined ? undefined : maybeCanvas;
    },
    reset() {
      maybePrevious = undefined;
      maybeRasterized = undefined;
      matchesView = false;
    },
  };
}
