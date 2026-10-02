import type { RenderFrame } from "../physics/frame";
import { projectPoint, type Camera } from "./camera";
import {
  rigidRenderMode,
  type CircleRenderMode,
  type RenderMode,
} from "./mode";

type PaintSegments = (
  context: CanvasRenderingContext2D,
  frame: RenderFrame,
  camera: Camera,
  mode: CircleRenderMode,
  strokeWidth: number,
) => void;
type PrepareStroke = (
  context: CanvasRenderingContext2D,
  mode: CircleRenderMode,
  strokeWidth: number,
) => void;
type ViewKey = {
  readonly camera: Camera;
  readonly mode: RenderMode;
  readonly strokeWidth: number;
  readonly pixelRatio: number;
  readonly backingWidth: number;
  readonly backingHeight: number;
};
type Layer = {
  key: ViewKey;
  segments: Uint32Array;
  stepIndex: number;
  maybePaths?: readonly Path2D[];
};
const layers = new WeakMap<CanvasRenderingContext2D, Layer>();
const watchedContexts = new WeakSet<CanvasRenderingContext2D>();

function sameContent(first: Uint32Array, second: Uint32Array): boolean {
  if (first.length !== second.length) return false;
  for (let index = 0; index < first.length; index++)
    if (first[index] !== second[index]) return false;
  return true;
}

function sameView(
  key: ViewKey,
  context: CanvasRenderingContext2D,
  camera: Camera,
  mode: RenderMode,
  strokeWidth: number,
  pixelRatio: number,
): boolean {
  const before = key.camera;
  // Native paths apply current Canvas transforms/dashes when stroked, not when cached.
  return (
    key.mode === mode &&
    key.strokeWidth === strokeWidth &&
    key.pixelRatio === pixelRatio &&
    key.backingWidth === context.canvas.width &&
    key.backingHeight === context.canvas.height &&
    before.viewport.width === camera.viewport.width &&
    before.viewport.height === camera.viewport.height &&
    before.bounds.minX === camera.bounds.minX &&
    before.bounds.minY === camera.bounds.minY &&
    before.bounds.maxX === camera.bounds.maxX &&
    before.bounds.maxY === camera.bounds.maxY &&
    before.scale === camera.scale &&
    before.offsetX === camera.offsetX &&
    before.offsetY === camera.offsetY
  );
}

function segmentPaths(
  segments: Float32Array,
  camera: Camera,
): readonly Path2D[] {
  const paths: Path2D[] = [];
  for (let index = 0; index < segments.length; index += 4) {
    const x1 = segments[index],
      y1 = segments[index + 1],
      x2 = segments[index + 2],
      y2 = segments[index + 3];
    if (
      x1 === undefined ||
      y1 === undefined ||
      x2 === undefined ||
      y2 === undefined
    )
      throw new Error("Invalid renderer frame");
    const start = projectPoint(camera, { x: x1, y: y1 });
    const end = projectPoint(camera, { x: x2, y: y2 });
    const path = new Path2D();
    path.moveTo(start.x, start.y);
    path.lineTo(end.x, end.y);
    paths.push(path);
  }
  return paths;
}

/** Reuses stable projected paths while preserving individual stroke order and blending. */
export function paintStableSegments(
  context: CanvasRenderingContext2D,
  frame: RenderFrame,
  camera: Camera,
  mode: RenderMode,
  strokeWidth: number,
  pixelRatio: number,
  paint: PaintSegments,
  prepare: PrepareStroke,
): void {
  const rigidMode = rigidRenderMode(mode);
  // Small or moving outlines avoid path-cache construction and stay on the direct path.
  if (
    frame.rigidSegments.length < 128 ||
    context.isContextLost?.() ||
    typeof Path2D === "undefined"
  ) {
    layers.delete(context);
    paint(context, frame, camera, rigidMode, strokeWidth);
    return;
  }
  const content = new Uint32Array(
    frame.rigidSegments.buffer,
    frame.rigidSegments.byteOffset,
    frame.rigidSegments.length,
  );
  let layer = layers.get(context);
  if (
    layer === undefined ||
    !sameView(layer.key, context, camera, mode, strokeWidth, pixelRatio) ||
    frame.stepIndex < layer.stepIndex ||
    !sameContent(content, layer.segments)
  ) {
    layer = {
      key: {
        camera: {
          ...camera,
          viewport: { ...camera.viewport },
          bounds: { ...camera.bounds },
        },
        mode,
        strokeWidth,
        pixelRatio,
        backingWidth: context.canvas.width,
        backingHeight: context.canvas.height,
      },
      segments: content.slice(),
      stepIndex: frame.stepIndex,
    };
    layers.set(context, layer);
    if (!watchedContexts.has(context)) {
      watchedContexts.add(context);
      context.canvas.addEventListener?.("contextlost", () =>
        layers.delete(context),
      );
      context.canvas.addEventListener?.("contextrestored", () =>
        layers.delete(context),
      );
    }
    paint(context, frame, camera, rigidMode, strokeWidth);
    return;
  }
  layer.stepIndex = frame.stepIndex;
  layer.maybePaths ??= segmentPaths(frame.rigidSegments, camera);
  prepare(context, rigidMode, strokeWidth);
  // Joining strokes into one path changes antialiased overlaps; retain the original calls.
  for (const path of layer.maybePaths) context.stroke(path);
}
