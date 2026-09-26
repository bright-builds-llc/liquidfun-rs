import { describe, expect, it } from "vitest";

import { contourLoops, type ContourPoint } from "../src/render/contour";

function radii(loop: readonly ContourPoint[], x: number, y: number): number[] {
  return loop.map((point) => Math.hypot(point.x - x, point.y - y));
}

describe("contourLoops", () => {
  it("returns no loops when there are no particles", () => {
    // Arrange
    const samples: readonly [] = [];

    // Act
    const loops = contourLoops(samples, 80, 80);

    // Assert
    expect(loops).toEqual([]);
  });

  it("traces one isolated particle near its radius", () => {
    // Arrange
    const center = { x: 40, y: 40 };
    const radius = 16;

    // Act
    const loops = contourLoops([{ ...center, radius }], 80, 80);

    // Assert
    expect(loops).toHaveLength(1);
    const loop = loops[0] ?? [];
    const distances = radii(loop, center.x, center.y);
    expect(Math.min(...distances)).toBeGreaterThan(radius * 0.6);
    expect(Math.max(...distances)).toBeLessThan(radius * 1.4);
  });

  it("merges overlapping particles into one loop", () => {
    // Arrange
    const samples = [
      { x: 30, y: 40, radius: 12 },
      { x: 48, y: 40, radius: 12 },
    ];

    // Act
    const loops = contourLoops(samples, 100, 80);

    // Assert
    expect(loops).toHaveLength(1);
  });

  it("keeps distant particles as separate loops", () => {
    // Arrange
    const samples = [
      { x: 20, y: 40, radius: 8 },
      { x: 80, y: 40, radius: 8 },
    ];

    // Act
    const loops = contourLoops(samples, 100, 80);

    // Assert
    expect(loops).toHaveLength(2);
  });
});