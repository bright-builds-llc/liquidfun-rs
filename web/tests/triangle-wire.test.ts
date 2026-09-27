import { describe, expect, it } from "vitest";

import { triangleWireVertices } from "../src/render/triangle-wire";

const TAU = Math.PI * 2;

function distance(
  left: { readonly x: number; readonly y: number },
  right: { readonly x: number; readonly y: number },
): number {
  const dx = left.x - right.x;
  const dy = left.y - right.y;
  return Math.hypot(dx, dy);
}

describe("triangle wire vertices", () => {
  it("places an equilateral outline on the particle circle, point up", () => {
    // Arrange
    const center = { x: 12, y: 40 };
    const radius = 6;

    // Act
    const [top, right, left] = triangleWireVertices(center.x, center.y, radius);

    // Assert
    expect(top.x).toBeCloseTo(center.x);
    expect(top.y).toBeCloseTo(center.y - radius);
    expect(distance(top, center)).toBeCloseTo(radius);
    expect(distance(right, center)).toBeCloseTo(radius);
    expect(distance(left, center)).toBeCloseTo(radius);
    expect(distance(top, right)).toBeCloseTo(radius * Math.sqrt(3));
    expect(distance(right, left)).toBeCloseTo(radius * Math.sqrt(3));
    expect(distance(left, top)).toBeCloseTo(radius * Math.sqrt(3));
    expect(right.x).toBeGreaterThan(center.x);
    expect(left.x).toBeLessThan(center.x);
    expect(Math.atan2(top.y - center.y, top.x - center.x)).toBeCloseTo(-TAU / 4);
  });
});
