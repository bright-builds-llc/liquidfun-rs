import { describe, expect, it } from "vitest";

import type { RenderFrame } from "../src/physics/frame";
import { drawRenderFrame } from "../src/render/canvas";
import {
  createCamera,
  projectPoint,
  projectRadius,
} from "../src/render/camera";

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

type Operation =
  | {
      readonly kind: "fillRect";
      readonly x: number;
      readonly y: number;
      readonly width: number;
      readonly height: number;
      readonly fillStyle: string;
    }
  | { readonly kind: "beginPath" }
  | {
      readonly kind: "arc";
      readonly x: number;
      readonly y: number;
      readonly radius: number;
    }
  | { readonly kind: "moveTo"; readonly x: number; readonly y: number }
  | { readonly kind: "lineTo"; readonly x: number; readonly y: number }
  | { readonly kind: "fill"; readonly fillStyle: string }
  | {
      readonly kind: "stroke";
      readonly strokeStyle: string;
      readonly lineWidth: number;
    };

type RecordingContext = {
  readonly context: CanvasRenderingContext2D;
  readonly fillCalls: () => number;
  readonly strokeCalls: () => number;
  readonly strokeStyles: readonly string[];
  readonly segmentPaths: readonly Point[];
  readonly operations: readonly Operation[];
};

function createRecordingContext(): RecordingContext {
  let fillCallCount = 0;
  let strokeCallCount = 0;
  let maybeFillStyle: string | CanvasGradient | CanvasPattern = "";
  let maybeStrokeStyle: string | CanvasGradient | CanvasPattern = "";
  let lineWidth = 1;
  const strokeStyles: string[] = [];
  const segmentPaths: Point[] = [];
  const operations: Operation[] = [];

  const styleText = (
    style: string | CanvasGradient | CanvasPattern,
  ): string => (typeof style === "string" ? style : String(style));

  const context = {
    beginPath: () => {
      operations.push({ kind: "beginPath" });
    },
    arc: (x: number, y: number, radius: number) => {
      operations.push({ kind: "arc", x, y, radius });
    },
    fill: () => {
      fillCallCount += 1;
      operations.push({
        kind: "fill",
        fillStyle: styleText(maybeFillStyle),
      });
    },
    stroke: () => {
      strokeCallCount += 1;
      operations.push({
        kind: "stroke",
        strokeStyle: styleText(maybeStrokeStyle),
        lineWidth,
      });
    },
    fillRect: (
      x: number,
      y: number,
      width: number,
      height: number,
    ) => {
      operations.push({
        kind: "fillRect",
        x,
        y,
        width,
        height,
        fillStyle: styleText(maybeFillStyle),
      });
    },
    moveTo: (x: number, y: number) => {
      segmentPaths.push([x, y]);
      operations.push({ kind: "moveTo", x, y });
    },
    lineTo: (x: number, y: number) => {
      segmentPaths.push([x, y]);
      operations.push({ kind: "lineTo", x, y });
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
    operations,
  };
}

describe("drawRenderFrame", () => {
  it("wireframe strokes particles and rigid circles without filling", () => {
    // Arrange
    const canvas = createRecordingContext();
    const camera = createCamera(960, 540);
    const expectedArcs = [
      {
        kind: "arc",
        ...projectPoint(camera, { x: 0, y: 1 }),
        radius: projectRadius(camera, Math.fround(0.2)),
      },
      {
        kind: "arc",
        ...projectPoint(camera, { x: 0, y: 2 }),
        radius: projectRadius(camera, 0.75),
      },
    ];

    // Act
    drawRenderFrame(canvas.context, FRAME, camera, "wireframe");

    // Assert
    expect(canvas.fillCalls()).toBe(0);
    expect(canvas.strokeCalls()).toBe(3);
    expect(canvas.strokeStyles).toContain(
      "rgba(57, 211, 199, 0.5019607843137255)",
    );
    expect(
      canvas.operations.filter((operation) => operation.kind === "arc"),
    ).toEqual(expectedArcs);
    expect(
      canvas.operations.filter((operation) => operation.kind === "stroke"),
    ).toEqual([
      {
        kind: "stroke",
        strokeStyle: "rgba(57, 211, 199, 0.5019607843137255)",
        lineWidth: 1.5,
      },
      {
        kind: "stroke",
        strokeStyle: "#94A3B8",
        lineWidth: 2,
      },
      {
        kind: "stroke",
        strokeStyle: "#CBD5E1",
        lineWidth: 2,
      },
    ]);
  });

  it("solid preserves particle fill and rigid fill-plus-stroke behavior", () => {
    // Arrange
    const canvas = createRecordingContext();
    const camera = createCamera(960, 540);
    const particleCenter = projectPoint(camera, { x: 0, y: 1 });
    const particleRadius = projectRadius(camera, Math.fround(0.2));
    const segmentStart = projectPoint(camera, { x: -1, y: 0 });
    const segmentEnd = projectPoint(camera, { x: 1, y: 0 });
    const circleCenter = projectPoint(camera, { x: 0, y: 2 });
    const circleRadius = projectRadius(camera, 0.75);

    // Act
    drawRenderFrame(canvas.context, FRAME, camera, "solid");

    // Assert
    expect(canvas.fillCalls()).toBe(2);
    expect(canvas.strokeCalls()).toBe(2);
    expect(canvas.operations).toEqual([
      {
        kind: "fillRect",
        x: 0,
        y: 0,
        width: 960,
        height: 540,
        fillStyle: "#071018",
      },
      { kind: "beginPath" },
      {
        kind: "arc",
        x: particleCenter.x,
        y: particleCenter.y,
        radius: particleRadius,
      },
      {
        kind: "fill",
        fillStyle: "rgba(57, 211, 199, 0.5019607843137255)",
      },
      { kind: "beginPath" },
      { kind: "moveTo", x: segmentStart.x, y: segmentStart.y },
      { kind: "lineTo", x: segmentEnd.x, y: segmentEnd.y },
      {
        kind: "stroke",
        strokeStyle: "#94A3B8",
        lineWidth: 2,
      },
      { kind: "beginPath" },
      {
        kind: "arc",
        x: circleCenter.x,
        y: circleCenter.y,
        radius: circleRadius,
      },
      { kind: "fill", fillStyle: "#334155" },
      {
        kind: "stroke",
        strokeStyle: "#CBD5E1",
        lineWidth: 2,
      },
    ]);
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
