import { describe, expect, it } from "vitest";

import { cameraFrame } from "../src/player/camera-frame";

const PORTRAIT_WORLD = { minX: 0, minY: 0, maxX: 3, maxY: 6 };

describe("cameraFrame", () => {
  it("fits the clock's world aspect above a bottom HUD band", () => {
    // Arrange
    const clock = {
      viewportWidth: 390,
      viewportHeight: 844,
      hudObstacles: [{ left: 0, top: 700, right: 390, bottom: 844 }],
      worldBounds: PORTRAIT_WORLD,
    };

    // Act
    const result = cameraFrame(clock);

    // Assert
    expect(result).toEqual({
      frame: { left: 0, top: 0, right: 390, bottom: 692 },
      usedFallback: false,
    });
  });

  it("returns the full viewport when the clock has no HUD obstacles", () => {
    // Arrange
    const clock = {
      viewportWidth: 390,
      viewportHeight: 844,
      hudObstacles: [],
      worldBounds: PORTRAIT_WORLD,
    };

    // Act
    const result = cameraFrame(clock);

    // Assert
    expect(result).toEqual({
      frame: { left: 0, top: 0, right: 390, bottom: 844 },
      usedFallback: false,
    });
  });
});
