import { describe, expect, it } from "vitest";

import { MASK_ALPHA_THRESHOLD, thresholdAlpha } from "../src/render/alpha-threshold";

describe("thresholdAlpha", () => {
  it("keeps coverage at or above the cutoff and clears the rest", () => {
    // Arrange
    const pixels = new Uint8ClampedArray([
      10, 20, 30, MASK_ALPHA_THRESHOLD - 1,
      1, 2, 3, MASK_ALPHA_THRESHOLD,
    ]);

    // Act
    thresholdAlpha(pixels);

    // Assert
    expect([...pixels]).toEqual([255, 255, 255, 0, 255, 255, 255, 255]);
  });
});