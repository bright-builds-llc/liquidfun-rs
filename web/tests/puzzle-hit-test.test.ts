import { describe, expect, it } from "vitest";

import { HIT_SLOP_CSS_PX, distanceToQuad, pickTool } from "../src/puzzle/hit-test";
import type { Quad } from "../src/puzzle/placement-status";

/** A 1 m by 0.1 m plank centred on the origin, counterclockwise. */
const PLANK: Quad = [
  { x: -0.5, y: -0.05 },
  { x: 0.5, y: -0.05 },
  { x: 0.5, y: 0.05 },
  { x: -0.5, y: 0.05 },
];
const CLOCKWISE_PLANK: Quad = [PLANK[3], PLANK[2], PLANK[1], PLANK[0]];
const PIXELS_PER_METER = 29.3;

describe("distanceToQuad", () => {
  it("is zero inside a counterclockwise quad", () => {
    expect(distanceToQuad({ x: 0.2, y: 0.01 }, PLANK)).toBe(0);
  });

  it("is zero inside a clockwise quad", () => {
    expect(distanceToQuad({ x: 0.2, y: 0.01 }, CLOCKWISE_PLANK)).toBe(0);
  });

  it("is the nearest edge distance outside", () => {
    expect(distanceToQuad({ x: 0, y: 0.35 }, PLANK)).toBeCloseTo(0.3, 9);
  });

  it("measures to the nearest corner beyond an end", () => {
    expect(distanceToQuad({ x: 0.8, y: 0.45 }, PLANK)).toBeCloseTo(0.5, 9);
  });
});

describe("pickTool", () => {
  it("uses a 22 CSS px slop", () => {
    expect(HIT_SLOP_CSS_PX).toBe(22);
  });

  it("hits a tool 0.7 m away at 29.3 px per meter (20.5 px)", () => {
    // Arrange
    const point = { x: 0, y: 0.05 + 0.7 };

    // Act
    const maybeId = pickTool(point, [{ id: 1, maybeQuad: PLANK }], PIXELS_PER_METER, undefined);

    // Assert
    expect(maybeId).toBe(1);
  });

  it("misses a tool 0.8 m away at 29.3 px per meter (23.4 px)", () => {
    // Arrange
    const point = { x: 0, y: 0.05 + 0.8 };

    // Act
    const maybeId = pickTool(point, [{ id: 1, maybeQuad: PLANK }], PIXELS_PER_METER, undefined);

    // Assert
    expect(maybeId).toBeUndefined();
  });

  it("prefers the nearer of two tools", () => {
    // Arrange
    const shifted: Quad = [
      { x: -0.5, y: 0.45 },
      { x: 0.5, y: 0.45 },
      { x: 0.5, y: 0.55 },
      { x: -0.5, y: 0.55 },
    ];
    const candidates = [
      { id: 1, maybeQuad: PLANK },
      { id: 2, maybeQuad: shifted },
    ];

    // Act
    const maybeId = pickTool({ x: 0, y: 0.15 }, candidates, PIXELS_PER_METER, undefined);

    // Assert
    expect(maybeId).toBe(1);
  });

  it("breaks an equal-distance tie toward the selected tool", () => {
    // Arrange
    const candidates = [
      { id: 1, maybeQuad: PLANK },
      { id: 2, maybeQuad: CLOCKWISE_PLANK },
    ];

    // Act
    const maybeId = pickTool({ x: 0, y: 0 }, candidates, PIXELS_PER_METER, 1);

    // Assert
    expect(maybeId).toBe(1);
  });

  it("breaks an equal-distance tie toward the highest id with nothing selected", () => {
    // Arrange
    const candidates = [
      { id: 2, maybeQuad: PLANK },
      { id: 1, maybeQuad: CLOCKWISE_PLANK },
    ];

    // Act
    const maybeId = pickTool({ x: 0, y: 0 }, candidates, PIXELS_PER_METER, undefined);

    // Assert
    expect(maybeId).toBe(2);
  });

  it("never hits a tool without an outline", () => {
    // Act
    const maybeId = pickTool(
      { x: 0, y: 0 },
      [{ id: 1, maybeQuad: undefined }],
      PIXELS_PER_METER,
      undefined,
    );

    // Assert
    expect(maybeId).toBeUndefined();
  });

  it.each([0, -1, Number.NaN, Number.POSITIVE_INFINITY])(
    "returns undefined for pixels per meter %s",
    (pixelsPerMeter) => {
      expect(
        pickTool({ x: 0, y: 0 }, [{ id: 1, maybeQuad: PLANK }], pixelsPerMeter, undefined),
      ).toBeUndefined();
    },
  );
});
