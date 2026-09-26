import { describe, expect, it } from "vitest";

import {
  contourLoops,
  maybeFieldDensity,
  type ContourPoint,
  type FieldSample,
} from "../src/render/contour";

const TAU = Math.PI * 2;

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

describe("maybeFieldDensity", () => {
  it("returns no field for an empty sample list", () => {
    // Arrange
    const samples: readonly FieldSample[] = [];

    // Act
    const maybeDensity = maybeFieldDensity(samples, 80, 80);

    // Assert
    expect(maybeDensity).toBeUndefined();
  });

  it("returns no field when the viewport has no area", () => {
    // Arrange
    const samples: readonly FieldSample[] = [{ x: 40, y: 40, radius: 8 }];

    // Act
    const maybeDensity = maybeFieldDensity(samples, 0, 80);

    // Assert
    expect(maybeDensity).toBeUndefined();
  });

  it("samples a tighter cluster as denser than one particle", () => {
    // Arrange
    const isolated: readonly FieldSample[] = [{ x: 100, y: 100, radius: 12 }];
    const packed: FieldSample[] = [...isolated];
    for (let index = 0; index < 6; index += 1) {
      const angle = (index / 6) * TAU;
      packed.push({
        x: 100 + Math.cos(angle) * 12,
        y: 100 + Math.sin(angle) * 12,
        radius: 12,
      });
    }

    // Act
    const maybeIsolated = maybeFieldDensity(isolated, 200, 200);
    const maybePacked = maybeFieldDensity(packed, 200, 200);

    // Assert
    expect(maybeIsolated).toBeDefined();
    expect(maybePacked).toBeDefined();
    expect(maybePacked?.(100, 100) ?? 0).toBeGreaterThan(
      (maybeIsolated?.(100, 100) ?? 0) + 0.5,
    );
  });
});