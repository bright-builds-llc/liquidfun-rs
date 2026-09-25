import { describe, expect, it } from "vitest";

import { visualViewportMetrics } from "../src/player/visual-viewport";

describe("visual viewport metrics", () => {
  it("uses the visual viewport box when the browser exposes one", () => {
    // Act
    const metrics = visualViewportMetrics({ height: 700, offsetTop: 40 }, 800);

    // Assert
    expect(metrics).toEqual({ heightPx: 700, offsetTopPx: 40 });
  });

  it("falls back to the window height when the visual viewport is missing", () => {
    // Act
    const metrics = visualViewportMetrics(null, 812);

    // Assert
    expect(metrics).toEqual({ heightPx: 812, offsetTopPx: 0 });
  });
});
