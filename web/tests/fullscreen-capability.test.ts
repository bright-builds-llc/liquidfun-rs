import { describe, expect, it } from "vitest";

import {
  canvasStageActive,
  fullscreenApiAvailable,
} from "../src/player/fullscreen-capability";

describe("fullscreen capability", () => {
  it("keeps the page layout when element fullscreen is available", () => {
    // Act
    const available = fullscreenApiAvailable(true);
    const canvasStage = canvasStageActive(true);

    // Assert
    expect(available).toBe(true);
    expect(canvasStage).toBe(false);
  });

  it("uses the canvas stage when element fullscreen is unavailable", () => {
    // Act
    const available = fullscreenApiAvailable(false);
    const canvasStage = canvasStageActive(false);

    // Assert
    expect(available).toBe(false);
    expect(canvasStage).toBe(true);
  });
});
