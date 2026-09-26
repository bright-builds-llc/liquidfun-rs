import type { RenderFrame } from "../physics/frame";
import type { Camera } from "./camera";
import { eachProjectedParticle } from "./projected-particle";
import { compositeMasked, scratchContext } from "./scratch-canvas";

/** Iso-value where an isolated particle's contour sits near its radius. */
const ISO_THRESHOLD = 0.5;

/** Kernel reach, in particle radii, that bridges the usual neighbor spacing. */
const REACH_FACTOR = 1.85;

/** Keeps the screen-space grid from growing without a bound at high zoom. */
const MAX_AXIS_CELLS = 220;

const TOP = 0;
const RIGHT = 1;
const BOTTOM = 2;
const LEFT = 3;

export type FieldSample = {
  readonly x: number;
  readonly y: number;
  readonly radius: number;
};

export type ContourPoint = {
  readonly x: number;
  readonly y: number;
};

type FieldGrid = {
  readonly values: Float32Array;
  readonly stride: number;
  readonly cell: number;
  readonly nx: number;
  readonly ny: number;
};

function kernel(dx: number, dy: number, radius: number): number {
  const reach = radius * REACH_FACTOR;
  const reach2 = reach * reach;
  if (reach2 <= 0) {
    return 0;
  }

  const distance2 = dx * dx + dy * dy;
  if (distance2 >= reach2) {
    return 0;
  }

  const falloff = 1 - distance2 / reach2;
  return falloff * falloff;
}

function cellSizeFor(radius: number, width: number, height: number): number {
  const desired = Math.max(2, radius * 0.45);
  const longest = Math.max(width, height);
  return Math.max(desired, longest / MAX_AXIS_CELLS);
}

function splat(
  grid: FieldGrid,
  sample: FieldSample,
): void {
  const reach = sample.radius * REACH_FACTOR;
  const minX = Math.max(0, Math.floor((sample.x - reach) / grid.cell));
  const maxX = Math.min(grid.nx, Math.ceil((sample.x + reach) / grid.cell));
  const minY = Math.max(0, Math.floor((sample.y - reach) / grid.cell));
  const maxY = Math.min(grid.ny, Math.ceil((sample.y + reach) / grid.cell));

  for (let iy = minY; iy <= maxY; iy += 1) {
    for (let ix = minX; ix <= maxX; ix += 1) {
      const dx = ix * grid.cell - sample.x;
      const dy = iy * grid.cell - sample.y;
      const index = ix + iy * grid.stride;
      const current = grid.values[index] ?? 0;
      grid.values[index] = current + kernel(dx, dy, sample.radius);
    }
  }
}

function buildGrid(
  samples: readonly FieldSample[],
  width: number,
  height: number,
): FieldGrid | undefined {
  let maxRadius = 0;
  for (const sample of samples) {
    if (sample.radius > maxRadius) {
      maxRadius = sample.radius;
    }
  }
  if (maxRadius <= 0 || width <= 0 || height <= 0) {
    return undefined;
  }

  const cell = cellSizeFor(maxRadius, width, height);
  const nx = Math.max(1, Math.ceil(width / cell));
  const ny = Math.max(1, Math.ceil(height / cell));
  const stride = nx + 1;
  const grid: FieldGrid = {
    values: new Float32Array(stride * (ny + 1)),
    stride,
    cell,
    nx,
    ny,
  };
  for (const sample of samples) {
    splat(grid, sample);
  }
  return grid;
}

function onBit(mask: number, bit: number): boolean {
  return (mask & bit) !== 0;
}

function crossingEdges(mask: number): readonly number[] {
  const edges: number[] = [];
  if (onBit(mask, 1) !== onBit(mask, 2)) {
    edges.push(TOP);
  }
  if (onBit(mask, 2) !== onBit(mask, 4)) {
    edges.push(RIGHT);
  }
  if (onBit(mask, 4) !== onBit(mask, 8)) {
    edges.push(BOTTOM);
  }
  if (onBit(mask, 8) !== onBit(mask, 1)) {
    edges.push(LEFT);
  }
  return edges;
}

function pairCrossings(
  mask: number,
  centerInside: boolean,
): readonly (readonly [number, number])[] {
  const edges = crossingEdges(mask);
  if (edges.length === 2) {
    const start = edges[0];
    const end = edges[1];
    if (start === undefined || end === undefined) {
      return [];
    }
    return [[start, end]];
  }
  if (edges.length !== 4) {
    return [];
  }

  const separatedDiagonal = (mask & (1 | 4)) === (1 | 4);
  const connectThroughCenter = separatedDiagonal ? centerInside : !centerInside;
  if (connectThroughCenter) {
    return [
      [TOP, RIGHT],
      [BOTTOM, LEFT],
    ];
  }

  return [
    [LEFT, TOP],
    [RIGHT, BOTTOM],
  ];
}

function globalEdge(
  local: number,
  cx: number,
  cy: number,
  stride: number,
): number {
  if (local === TOP) {
    return (cy * stride + cx) * 2;
  }
  if (local === BOTTOM) {
    return ((cy + 1) * stride + cx) * 2;
  }
  if (local === LEFT) {
    return (cy * stride + cx) * 2 + 1;
  }

  return (cy * stride + (cx + 1)) * 2 + 1;
}

function interpolationT(start: number, end: number): number {
  const span = end - start;
  if (span === 0) {
    return 0.5;
  }

  const t = (ISO_THRESHOLD - start) / span;
  if (t < 0) {
    return 0;
  }
  if (t > 1) {
    return 1;
  }
  return t;
}

function edgePoint(edgeId: number, grid: FieldGrid): ContourPoint {
  const packed = Math.floor(edgeId / 2);
  const ix = packed % grid.stride;
  const iy = Math.floor(packed / grid.stride);
  const horizontal = edgeId % 2 === 0;
  const startIndex = ix + iy * grid.stride;
  const endIndex = horizontal ? startIndex + 1 : startIndex + grid.stride;
  const t = interpolationT(grid.values[startIndex] ?? 0, grid.values[endIndex] ?? 0);
  if (horizontal) {
    return { x: (ix + t) * grid.cell, y: iy * grid.cell };
  }

  return { x: ix * grid.cell, y: (iy + t) * grid.cell };
}

function segmentsForGrid(grid: FieldGrid): Array<readonly [number, number]> {
  const segments: Array<readonly [number, number]> = [];
  for (let cy = 0; cy < grid.ny; cy += 1) {
    for (let cx = 0; cx < grid.nx; cx += 1) {
      const i0 = cx + cy * grid.stride;
      const v0 = grid.values[i0] ?? 0;
      const v1 = grid.values[i0 + 1] ?? 0;
      const v2 = grid.values[i0 + 1 + grid.stride] ?? 0;
      const v3 = grid.values[i0 + grid.stride] ?? 0;
      let mask = 0;
      if (v0 >= ISO_THRESHOLD) {
        mask += 1;
      }
      if (v1 >= ISO_THRESHOLD) {
        mask += 2;
      }
      if (v2 >= ISO_THRESHOLD) {
        mask += 4;
      }
      if (v3 >= ISO_THRESHOLD) {
        mask += 8;
      }
      const centerInside = (v0 + v1 + v2 + v3) / 4 >= ISO_THRESHOLD;
      for (const [start, end] of pairCrossings(mask, centerInside)) {
        segments.push([
          globalEdge(start, cx, cy, grid.stride),
          globalEdge(end, cx, cy, grid.stride),
        ]);
      }
    }
  }
  return segments;
}

function pushIndex(indexes: Map<number, number[]>, edge: number, segment: number): void {
  const existing = indexes.get(edge);
  if (existing === undefined) {
    indexes.set(edge, [segment]);
    return;
  }
  existing.push(segment);
}

function stitchLoops(
  segments: readonly (readonly [number, number])[],
  grid: FieldGrid,
): ContourPoint[][] {
  const byEdge = new Map<number, number[]>();
  for (let index = 0; index < segments.length; index += 1) {
    const segment = segments[index];
    if (segment === undefined) {
      continue;
    }
    pushIndex(byEdge, segment[0], index);
    pushIndex(byEdge, segment[1], index);
  }

  const used = new Array<boolean>(segments.length).fill(false);
  const loops: ContourPoint[][] = [];
  for (let start = 0; start < segments.length; start += 1) {
    if (used[start]) {
      continue;
    }
    const first = segments[start];
    if (first === undefined) {
      continue;
    }
    const points: ContourPoint[] = [edgePoint(first[0], grid)];
    let segmentIndex = start;
    let arrivedFrom = first[0];
    let steps = 0;
    while (used[segmentIndex] !== true && steps <= segments.length) {
      steps += 1;
      used[segmentIndex] = true;
      const segment = segments[segmentIndex];
      if (segment === undefined) {
        break;
      }
      const nextEdge = segment[0] === arrivedFrom ? segment[1] : segment[0];
      points.push(edgePoint(nextEdge, grid));
      const candidates = byEdge.get(nextEdge) ?? [];
      const maybeNext = candidates.find((candidate) => used[candidate] !== true);
      if (maybeNext === undefined) {
        break;
      }
      arrivedFrom = nextEdge;
      segmentIndex = maybeNext;
    }
    if (points.length >= 3) {
      loops.push(points);
    }
  }
  return loops;
}

/** Extracts filled contour loops for one set of screen-space particle discs. */
export function contourLoops(
  samples: readonly FieldSample[],
  width: number,
  height: number,
): readonly (readonly ContourPoint[])[] {
  if (!Number.isFinite(width) || !Number.isFinite(height)) {
    return [];
  }

  const maybeGrid = buildGrid(samples, width, height);
  if (maybeGrid === undefined) {
    return [];
  }

  return stitchLoops(segmentsForGrid(maybeGrid), maybeGrid);
}

function fillLoops(
  context: CanvasRenderingContext2D,
  loops: readonly (readonly ContourPoint[])[],
): void {
  context.beginPath();
  for (const loop of loops) {
    const first = loop[0];
    if (first === undefined) {
      continue;
    }
    context.moveTo(first.x, first.y);
    for (let index = 1; index < loop.length; index += 1) {
      const point = loop[index];
      if (point === undefined) {
        continue;
      }
      context.lineTo(point.x, point.y);
    }
    context.closePath();
  }
  context.fill();
}

/** Draws particles as marching-squares blobs clipped to their own colors. */
export function paintContour(
  target: CanvasRenderingContext2D,
  frame: RenderFrame,
  camera: Camera,
  maxRenderedParticles: number,
): void {
  const samples: FieldSample[] = [];
  eachProjectedParticle(
    frame,
    camera,
    maxRenderedParticles,
    (x, y, radius) => {
      samples.push({ x, y, radius });
    },
  );
  const width = camera.viewport.width;
  const height = camera.viewport.height;
  const loops = contourLoops(samples, width, height);
  if (loops.length === 0) {
    return;
  }

  const color = scratchContext("contour-color", width, height);
  const mask = scratchContext("contour-mask", width, height);
  color.clearRect(0, 0, color.canvas.width, color.canvas.height);
  mask.clearRect(0, 0, mask.canvas.width, mask.canvas.height);
  eachProjectedParticle(
    frame,
    camera,
    maxRenderedParticles,
    (x, y, radius, red, green, blue, alpha) => {
      color.fillStyle = `rgba(${red}, ${green}, ${blue}, ${alpha})`;
      color.beginPath();
      color.arc(x, y, radius * REACH_FACTOR, 0, Math.PI * 2);
      color.fill();
    },
  );
  mask.fillStyle = "#FFFFFF";
  fillLoops(mask, loops);
  compositeMasked(target, color.canvas, mask.canvas, width, height);
}
