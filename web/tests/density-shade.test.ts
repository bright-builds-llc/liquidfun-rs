import { describe, expect, it } from "vitest";

import { fieldDensity, type FieldSample } from "../src/render/contour";
import {
  DENSITY_SHADE_END,
  DENSITY_SHADE_FLOOR,
  DENSITY_SHADE_START,
  DENSITY_SHADING_STORAGE_KEY,
  DEFAULT_DENSITY_SHADING,
  densityShade,
  loadDensityShading,
  maybeParseDensityShading,
  persistDensityShading,
  shadePackedColor,
} from "../src/render/density-shade";

const TAU = Math.PI * 2;

/** Playground particles rest about 1.6 radii apart and compress below that. */
const REST_SPACING_RADII = 1.6;
const PACKED_SPACING_RADII = 1.2;

function hexCluster(radius: number, spacingRadii: number): FieldSample[] {
  const center = { x: 100, y: 100, radius };
  const distance = radius * spacingRadii;
  const samples: FieldSample[] = [center];
  for (let index = 0; index < 6; index += 1) {
    const angle = (index / 6) * TAU;
    samples.push({
      x: center.x + Math.cos(angle) * distance,
      y: center.y + Math.sin(angle) * distance,
      radius,
    });
  }
  return samples;
}

describe("densityShade", () => {
  it("keeps a single particle center at full brightness", () => {
    // Arrange
    const isolated = fieldDensity([{ x: 100, y: 100, radius: 12 }], 100, 100);

    // Act
    const shade = densityShade(isolated);

    // Assert
    expect(isolated).toBeCloseTo(1);
    expect(shade).toBe(1);
  });

  it("keeps resting water near full brightness", () => {
    // Arrange
    const rest = fieldDensity(hexCluster(12, REST_SPACING_RADII), 100, 100);

    // Act
    const shade = densityShade(rest);

    // Assert
    expect(shade).toBeGreaterThan(0.9);
  });

  it("darkens a compressed cluster below resting water", () => {
    // Arrange
    const rest = fieldDensity(hexCluster(12, REST_SPACING_RADII), 100, 100);
    const packed = fieldDensity(hexCluster(12, PACKED_SPACING_RADII), 100, 100);

    // Act
    const restShade = densityShade(rest);
    const packedShade = densityShade(packed);

    // Assert
    expect(packed).toBeGreaterThan(rest);
    expect(packedShade).toBeLessThan(restShade);
    expect(packedShade).toBeLessThan(0.6);
  });

  it("reaches the floor at the packed end", () => {
    // Arrange
    const midpoint = (DENSITY_SHADE_START + DENSITY_SHADE_END) / 2;

    // Act
    const atStart = densityShade(DENSITY_SHADE_START);
    const atMidpoint = densityShade(midpoint);
    const atEnd = densityShade(DENSITY_SHADE_END);
    const beyond = densityShade(DENSITY_SHADE_END + 4);

    // Assert
    expect(atStart).toBe(1);
    expect(atMidpoint).toBeCloseTo(1 - 0.5 * (1 - DENSITY_SHADE_FLOOR));
    expect(atEnd).toBeCloseTo(DENSITY_SHADE_FLOOR);
    expect(beyond).toBeCloseTo(DENSITY_SHADE_FLOOR);
  });

  it("leaves non-finite density at full brightness", () => {
    // Arrange
    const density = Number.NaN;

    // Act
    const shade = densityShade(density);

    // Assert
    expect(shade).toBe(1);
  });
});

describe("density shading preference", () => {
  it("parses only the on and off tokens", () => {
    // Arrange
    const stored = ["on", "off", "true", null];

    // Act
    const parsed = stored.map(maybeParseDensityShading);

    // Assert
    expect(parsed).toEqual([true, false, undefined, undefined]);
  });

  it("defaults missing and invalid storage to enabled", () => {
    // Arrange
    const storedValues = [null, "maybe"];

    // Act
    const enabled = storedValues.map((storedValue) =>
      loadDensityShading(() => ({
        getItem: () => storedValue,
        setItem: () => undefined,
      })),
    );

    // Assert
    expect(enabled).toEqual([DEFAULT_DENSITY_SHADING, DEFAULT_DENSITY_SHADING]);
    expect(DEFAULT_DENSITY_SHADING).toBe(true);
  });

  it("persists the choice under the versioned key", () => {
    // Arrange
    const writes: Array<readonly [string, string]> = [];
    const storage = {
      getItem: () => null,
      setItem: (key: string, value: string) => {
        writes.push([key, value]);
      },
    };

    // Act
    persistDensityShading(() => storage, false);

    // Assert
    expect(writes).toEqual([[DENSITY_SHADING_STORAGE_KEY, "off"]]);
  });

  it("contains storage read failures", () => {
    // Arrange
    const denied = () => {
      throw new Error("storage denied");
    };

    // Act
    const loaded = loadDensityShading(denied);

    // Assert
    expect(loaded).toBe(true);
  });

  it("contains storage write failures", () => {
    // Arrange
    const failedWrite = () =>
      persistDensityShading(
        () => ({
          getItem: () => null,
          setItem: () => {
            throw new Error("write denied");
          },
        }),
        true,
      );

    // Act / Assert
    expect(failedWrite).not.toThrow();
  });
});

describe("shadePackedColor", () => {
  it("darkens opaque packed pixels and leaves sparse or clear pixels", () => {
    // Arrange
    const pixels = new Uint8ClampedArray([
      200, 100, 50, 255, 200, 100, 50, 255, 200, 100, 50, 0,
    ]);

    // Act
    shadePackedColor(pixels, 3, 1, (x) => (x < 1 ? DENSITY_SHADE_END + 2 : 0));

    // Assert
    expect(pixels[0]).toBe(Math.round(200 * DENSITY_SHADE_FLOOR));
    expect(pixels[1]).toBe(Math.round(100 * DENSITY_SHADE_FLOOR));
    expect(pixels[2]).toBe(Math.round(50 * DENSITY_SHADE_FLOOR));
    expect(pixels[4]).toBe(200);
    expect(pixels[8]).toBe(200);
    expect(pixels[11]).toBe(0);
  });
});
