import { describe, expect, it } from "vitest";

import type { RenderFrame } from "../src/physics/frame";
import { drawRenderFrame } from "../src/render/canvas";
import { createCamera } from "../src/render/camera";

const FRAME: RenderFrame = {
  stepIndex: 7,
  particleCount: 1,
  rigidShapeCount: 2,
  particlePositions: new Float32Array([0, 1]),
  particleColors: new Uint8Array([57, 211, 199, 128]),
  particleRadii: new Float32Array([0.2]),
  rigidSegments: new Float32Array([-1, 0, 1, 0]),
  rigidCircles: new Float32Array([0, 2, 0.75]),
};

type Point = readonly [number, number];

type RecordingContext = {
  readonly context: CanvasRenderingContext2D;
  readonly fillCalls: () => number;
  readonly strokeCalls: () => number;
  readonly strokeStyles: readonly string[];
  readonly segmentPaths: readonly Point[];
};

function createRecordingContext(): RecordingContext {
  let fillCallCount = 0;
  let strokeCallCount = 0;
  let maybeFillStyle: string | CanvasGradient | CanvasPattern = "";
  let maybeStrokeStyle: string | CanvasGradient | CanvasPattern = "";
  let lineWidth = 1;
  const strokeStyles: string[] = [];
  const segmentPaths: Point[] = [];

  const context = {
    beginPath: () => undefined,
    arc: () => undefined,
    fill: () => {
      fillCallCount += 1;
    },
    stroke: () => {
      strokeCallCount += 1;
    },
    fillRect: () => undefined,
    moveTo: (x: number, y: number) => {
      segmentPaths.push([x, y]);
    },
    lineTo: (x: number, y: number) => {
      segmentPaths.push([x, y]);
    },
    get fillStyle() {
      return maybeFillStyle;
    },
    set fillStyle(value: string | CanvasGradient | CanvasPattern) {
      maybeFillStyle = value;
    },
    get strokeStyle() {
      return maybeStrokeStyle;
    },
    set strokeStyle(value: string | CanvasGradient | CanvasPattern) {
      maybeStrokeStyle = value;
      if (typeof value === "string") {
        strokeStyles.push(value);
      }
    },
    get lineWidth() {
      return lineWidth;
    },
    set lineWidth(value: number) {
      lineWidth = value;
    },
  } as unknown as CanvasRenderingContext2D;

  return {
    context,
    fillCalls: () => fillCallCount,
    strokeCalls: () => strokeCallCount,
    strokeStyles,
    segmentPaths,
  };
}

describe("drawRenderFrame", () => {
  it("wireframe strokes particles and rigid circles without filling", () => {
    // Arrange
    const canvas = createRecordingContext();
    const camera = createCamera(960, 540);

    // Act
    drawRenderFrame(canvas.context, FRAME, camera, "wireframe");

    // Assert
    expect(canvas.fillCalls()).toBe(0);
    expect(canvas.strokeCalls()).toBe(3);
    expect(canvas.strokeStyles).toContain(
      "rgba(57, 211, 199, 0.5019607843137255)",
    );
  });

  it("solid preserves particle fill and rigid fill-plus-stroke behavior", () => {
    // Arrange
    const canvas = createRecordingContext();
    const camera = createCamera(960, 540);

    // Act
    drawRenderFrame(canvas.context, FRAME, camera, "solid");

    // Assert
    expect(canvas.fillCalls()).toBe(2);
    expect(canvas.strokeCalls()).toBe(2);
  });

  it("renders identical rigid segment paths in both modes", () => {
    // Arrange
    const wireframe = createRecordingContext();
    const solid = createRecordingContext();
    const camera = createCamera(960, 540);

    // Act
    drawRenderFrame(wireframe.context, FRAME, camera, "wireframe");
    drawRenderFrame(solid.context, FRAME, camera, "solid");

    // Assert
    expect(wireframe.segmentPaths).toEqual(solid.segmentPaths);
  });
});
