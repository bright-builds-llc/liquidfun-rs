import { describe, expect, it } from "vitest";

import {
  IDENTITY_CAMERA_VIEW,
  createCamera,
  projectPoint,
  type WorldBounds,
} from "../src/render/camera";
import {
  drawGlassCup,
  glassCup,
  glassHeightAt,
  targetLineY,
  type GlassTick,
} from "../src/render/glass-cup";

type PathCommand = {
  readonly kind: "moveTo" | "lineTo";
  readonly x: number;
  readonly y: number;
};

type CanvasState = {
  fillStyle: string;
  strokeStyle: string;
  lineWidth: number;
  globalAlpha: number;
  lineJoin: string;
  lineCap: string;
  lineDash: readonly number[];
};

type Paint = {
  readonly kind: "stroke" | "fill" | "fillRect";
  readonly path: readonly PathCommand[];
  readonly state: CanvasState;
};

type Call = { readonly name: string; readonly args: readonly unknown[] };

/** Records calls, paints and property state; save/restore use a real stack. */
class RecordingContext {
  state: CanvasState = {
    fillStyle: "#000000",
    strokeStyle: "#123456",
    lineWidth: 7,
    globalAlpha: 0.5,
    lineJoin: "miter",
    lineCap: "square",
    lineDash: [2, 9],
  };
  readonly calls: Call[] = [];
  readonly paints: Paint[] = [];
  private readonly stack: CanvasState[] = [];
  private path: PathCommand[] = [];

  get fillStyle(): string {
    return this.state.fillStyle;
  }
  set fillStyle(value: string) {
    this.state.fillStyle = value;
  }
  get strokeStyle(): string {
    return this.state.strokeStyle;
  }
  set strokeStyle(value: string) {
    this.state.strokeStyle = value;
  }
  get lineWidth(): number {
    return this.state.lineWidth;
  }
  set lineWidth(value: number) {
    this.state.lineWidth = value;
  }
  get globalAlpha(): number {
    return this.state.globalAlpha;
  }
  set globalAlpha(value: number) {
    this.state.globalAlpha = value;
  }
  get lineJoin(): string {
    return this.state.lineJoin;
  }
  set lineJoin(value: string) {
    this.state.lineJoin = value;
  }
  get lineCap(): string {
    return this.state.lineCap;
  }
  set lineCap(value: string) {
    this.state.lineCap = value;
  }

  save(): void {
    this.calls.push({ name: "save", args: [] });
    this.stack.push({ ...this.state, lineDash: [...this.state.lineDash] });
  }
  restore(): void {
    this.calls.push({ name: "restore", args: [] });
    const maybeSaved = this.stack.pop();
    if (maybeSaved !== undefined) this.state = maybeSaved;
  }
  setLineDash(segments: readonly number[]): void {
    this.calls.push({ name: "setLineDash", args: [[...segments]] });
    this.state.lineDash = [...segments];
  }
  getLineDash(): number[] {
    return [...this.state.lineDash];
  }
  beginPath(): void {
    this.calls.push({ name: "beginPath", args: [] });
    this.path = [];
  }
  moveTo(x: number, y: number): void {
    this.calls.push({ name: "moveTo", args: [x, y] });
    this.path.push({ kind: "moveTo", x, y });
  }
  lineTo(x: number, y: number): void {
    this.calls.push({ name: "lineTo", args: [x, y] });
    this.path.push({ kind: "lineTo", x, y });
  }
  closePath(): void {
    this.calls.push({ name: "closePath", args: [] });
  }
  rect(x: number, y: number, width: number, height: number): void {
    this.calls.push({ name: "rect", args: [x, y, width, height] });
    this.path.push({ kind: "moveTo", x, y });
    this.path.push({ kind: "lineTo", x: x + width, y: y + height });
  }
  fillRect(x: number, y: number, width: number, height: number): void {
    this.calls.push({ name: "fillRect", args: [x, y, width, height] });
    this.paints.push({
      kind: "fillRect",
      path: [
        { kind: "moveTo", x, y },
        { kind: "lineTo", x: x + width, y: y + height },
      ],
      state: this.snapshot(),
    });
  }
  stroke(): void {
    this.calls.push({ name: "stroke", args: [] });
    this.paints.push({ kind: "stroke", path: [...this.path], state: this.snapshot() });
  }
  fill(): void {
    this.calls.push({ name: "fill", args: [] });
    this.paints.push({ kind: "fill", path: [...this.path], state: this.snapshot() });
  }
  fillText(...args: unknown[]): void {
    this.calls.push({ name: "fillText", args });
  }
  strokeText(...args: unknown[]): void {
    this.calls.push({ name: "strokeText", args });
  }

  asContext(): CanvasRenderingContext2D {
    return this as unknown as CanvasRenderingContext2D;
  }

  private snapshot(): CanvasState {
    return { ...this.state, lineDash: [...this.state.lineDash] };
  }
}

const INTERIOR: WorldBounds = { minX: 1.6, minY: 0, maxX: 3.4, maxY: 1.8 };
const TICKS: readonly GlassTick[] = [
  { fraction: 0.6, label: "★ Win" },
  { fraction: 1, label: "Full" },
];
const CAMERA = createCamera(390, 844, IDENTITY_CAMERA_VIEW, {
  minX: -6.1,
  minY: -0.6,
  maxX: 6.1,
  maxY: 9.1,
});
const TICK_LENGTH = 8;

function isHorizontalTick(paint: Paint, outerX: number): boolean {
  const [maybeStart, maybeEnd] = paint.path;
  return (
    paint.kind === "stroke" &&
    paint.path.length === 2 &&
    maybeStart !== undefined &&
    maybeEnd !== undefined &&
    Math.abs(maybeEnd.x - outerX) < 1e-9 &&
    Math.abs(maybeStart.y - maybeEnd.y) < 1e-9
  );
}

function drawWith(ticks: readonly GlassTick[]): RecordingContext {
  const fake = new RecordingContext();
  drawGlassCup(fake.asContext(), CAMERA, glassCup(INTERIOR, 540, 900, ticks));
  return fake;
}

describe("glass cup geometry", () => {
  it("places the first-pour target line at 1.08 m", () => {
    // Arrange
    const ticks = TICKS;

    // Act
    const glass = glassCup(INTERIOR, 540, 900, ticks);

    // Assert
    expect(glass.targetFraction).toBe(0.6);
    expect(targetLineY(glass)).toBeCloseTo(1.08, 12);
  });

  it("maps the full and empty fractions to the interior edges", () => {
    // Arrange
    const glass = glassCup(INTERIOR, 540, 900, TICKS);

    // Act
    const top = glassHeightAt(glass, 1);
    const bottom = glassHeightAt(glass, 0);

    // Assert
    expect(top).toBe(INTERIOR.maxY);
    expect(bottom).toBe(INTERIOR.minY);
  });

  it.each([
    ["a zero full count", INTERIOR, 0, 0],
    ["a target above full", INTERIOR, 901, 900],
    ["a negative target", INTERIOR, -1, 900],
    ["a degenerate interior", { ...INTERIOR, maxX: INTERIOR.minX }, 540, 900],
    ["a NaN edge", { ...INTERIOR, maxY: Number.NaN }, 540, 900],
  ] as const)("rejects %s", (_name, interior, target, full) => {
    // Arrange
    const build = () => glassCup(interior, target, full, TICKS);

    // Act / Assert
    expect(build).toThrow("Invalid glass cup");
  });

  it("rejects a tick fraction outside 0..1", () => {
    // Arrange
    const ticks = [{ fraction: 1.2, label: "★ Over" }];

    // Act / Assert
    expect(() => glassCup(INTERIOR, 540, 900, ticks)).toThrow("Invalid glass cup");
  });
});

describe("glass cup overlay", () => {
  it("saves first, restores last and leaves canvas state unchanged", () => {
    // Arrange
    const fake = new RecordingContext();
    const before = { ...fake.state, lineDash: [...fake.state.lineDash] };

    // Act
    drawGlassCup(fake.asContext(), CAMERA, glassCup(INTERIOR, 540, 900, TICKS));

    // Assert
    const names = fake.calls.map((call) => call.name);
    expect(names[0]).toBe("save");
    expect(names.at(-1)).toBe("restore");
    expect(names.filter((name) => name === "save")).toHaveLength(1);
    expect(names.filter((name) => name === "restore")).toHaveLength(1);
    expect(fake.state).toEqual(before);
  });

  it("restores canvas state even when drawing throws", () => {
    // Arrange
    const fake = new RecordingContext();
    const before = { ...fake.state, lineDash: [...fake.state.lineDash] };
    fake.stroke = () => {
      throw new Error("stroke failed");
    };

    // Act
    const draw = () =>
      drawGlassCup(fake.asContext(), CAMERA, glassCup(INTERIOR, 540, 900, TICKS));

    // Assert
    expect(draw).toThrow("stroke failed");
    expect(fake.calls.at(-1)?.name).toBe("restore");
    expect(fake.state).toEqual(before);
  });

  it("draws the etched dashed target line across the interior at 1.08 m", () => {
    // Arrange
    const expectedStart = projectPoint(CAMERA, { x: INTERIOR.minX, y: 1.08 });
    const expectedEnd = projectPoint(CAMERA, { x: INTERIOR.maxX, y: 1.08 });

    // Act
    const fake = drawWith(TICKS);

    // Assert
    const dashCalls = fake.calls.filter(
      (call) => call.name === "setLineDash" && JSON.stringify(call.args[0]) === "[6,4]",
    );
    expect(dashCalls).toHaveLength(1);
    const dashed = fake.paints.filter(
      (paint) => paint.kind === "stroke" && paint.state.lineDash.length > 0,
    );
    expect(dashed).toHaveLength(1);
    const [start, end] = dashed[0]?.path ?? [];
    expect(start?.kind).toBe("moveTo");
    expect(start?.x).toBeCloseTo(expectedStart.x, 9);
    expect(start?.y).toBeCloseTo(expectedStart.y, 9);
    expect(end?.x).toBeCloseTo(expectedEnd.x, 9);
    expect(end?.y).toBeCloseTo(expectedStart.y, 9);
    expect(dashed[0]?.state.strokeStyle).toBe("#E6C35C");
    expect(dashed[0]?.state.globalAlpha).toBe(0.95);
  });

  it("draws the target shadow pass first, one pixel lower", () => {
    // Arrange
    const lineY = projectPoint(CAMERA, { x: INTERIOR.minX, y: 1.08 }).y;

    // Act
    const fake = drawWith(TICKS);

    // Assert
    const shadowIndex = fake.paints.findIndex(
      (paint) => paint.state.strokeStyle === "rgba(7, 16, 24, 0.65)",
    );
    const dashedIndex = fake.paints.findIndex(
      (paint) => paint.kind === "stroke" && paint.state.lineDash.length > 0,
    );
    expect(shadowIndex).toBeGreaterThanOrEqual(0);
    expect(shadowIndex).toBeLessThan(dashedIndex);
    expect(fake.paints[shadowIndex]?.path[0]?.y).toBeCloseTo(lineY + 1, 9);
    expect(fake.paints[shadowIndex]?.state.lineDash).toEqual([]);
  });

  it("draws one tick per goal tick and one star per ★ tick", () => {
    // Arrange
    const outerX = projectPoint(CAMERA, { x: INTERIOR.minX, y: 0 }).x - TICK_LENGTH;

    // Act
    const fake = drawWith(TICKS);

    // Assert
    const ticks = fake.paints.filter((paint) => isHorizontalTick(paint, outerX));
    const stars = fake.paints.filter((paint) => paint.kind === "fill");
    expect(ticks).toHaveLength(2);
    expect(stars).toHaveLength(1);
    expect(stars[0]?.path).toHaveLength(10);
    expect(stars[0]?.state.fillStyle).toBe("#E6C35C");
  });

  it("places each tick at its capacity fraction", () => {
    // Arrange
    const outerX = projectPoint(CAMERA, { x: INTERIOR.minX, y: 0 }).x - TICK_LENGTH;
    const expectedYs = TICKS.map(
      (tick) => projectPoint(CAMERA, { x: 0, y: tick.fraction * 1.8 }).y,
    );

    // Act
    const fake = drawWith(TICKS);

    // Assert
    const tickYs = fake.paints
      .filter((paint) => isHorizontalTick(paint, outerX))
      .map((paint) => paint.path[0]?.y ?? Number.NaN);
    expect(tickYs).toHaveLength(expectedYs.length);
    tickYs.forEach((y, index) => expect(y).toBeCloseTo(expectedYs[index] ?? 0, 9));
  });

  it("adds a tick and a star for a third ★ tick without code changes", () => {
    // Arrange
    const outerX = projectPoint(CAMERA, { x: INTERIOR.minX, y: 0 }).x - TICK_LENGTH;
    const ticks = [...TICKS, { fraction: 0.8, label: "★ Brim" }];

    // Act
    const fake = drawWith(ticks);

    // Assert
    expect(fake.paints.filter((paint) => isHorizontalTick(paint, outerX))).toHaveLength(3);
    expect(fake.paints.filter((paint) => paint.kind === "fill")).toHaveLength(2);
  });

  it("draws no text", () => {
    // Arrange
    const ticks = TICKS;

    // Act
    const fake = drawWith(ticks);

    // Assert
    const textCalls = fake.calls.filter(
      (call) => call.name === "fillText" || call.name === "strokeText",
    );
    expect(textCalls).toEqual([]);
  });

  it("tints the projected cup interior", () => {
    // Arrange
    const topLeft = projectPoint(CAMERA, { x: INTERIOR.minX, y: INTERIOR.maxY });
    const bottomRight = projectPoint(CAMERA, { x: INTERIOR.maxX, y: INTERIOR.minY });

    // Act
    const fake = drawWith(TICKS);

    // Assert
    const tint = fake.paints[0];
    expect(tint?.kind).toBe("fillRect");
    expect(tint?.state.fillStyle).toBe("rgba(186, 230, 253, 0.08)");
    expect(tint?.path[0]?.x).toBeCloseTo(topLeft.x, 9);
    expect(tint?.path[0]?.y).toBeCloseTo(topLeft.y, 9);
    expect(tint?.path[1]?.x).toBeCloseTo(bottomRight.x, 9);
    expect(tint?.path[1]?.y).toBeCloseTo(bottomRight.y, 9);
  });
});
