import { describe, expect, it } from "vitest";

import {
  SAFE_FRAME_GAP_PX,
  sameScreenRects,
  safeFrame,
  screenRectInCanvas,
  type ScreenRect,
} from "../src/render/safe-frame";

const LANDSCAPE_ASPECT = 4 / 3;

function rect(left: number, top: number, right: number, bottom: number): ScreenRect {
  return { left, top, right, bottom };
}

describe("safeFrame", () => {
  it("returns the full viewport when there are no obstacles", () => {
    // Arrange
    const viewport = { width: 390, height: 844 };

    // Act
    const result = safeFrame(viewport, [], LANDSCAPE_ASPECT);

    // Assert
    expect(result).toEqual({ frame: rect(0, 0, 390, 844), usedFallback: false });
  });

  it("returns the region above a full-width bottom band", () => {
    // Arrange
    const viewport = { width: 390, height: 844 };
    const band = rect(0, 700, 390, 844);

    // Act
    const result = safeFrame(viewport, [band], LANDSCAPE_ASPECT);

    // Assert
    expect(result).toEqual({ frame: rect(0, 0, 390, 692), usedFallback: false });
  });

  it("keeps the frame one gap away from an obstacle edge", () => {
    // Arrange
    const viewport = { width: 1000, height: 400 };
    const rightStrip = rect(900, 0, 1000, 400);

    // Act
    const result = safeFrame(viewport, [rightStrip], 1);

    // Assert
    expect(result.frame.right).toBe(900 - SAFE_FRAME_GAP_PX);
  });

  it("places the world beside a narrow bottom-left card at full height", () => {
    // Arrange
    const viewport = { width: 1280, height: 800 };
    const card = rect(16, 500, 368, 784);

    // Act
    const result = safeFrame(viewport, [card], LANDSCAPE_ASPECT);

    // Assert
    expect(result).toEqual({
      frame: rect(376, 0, 1280, 800),
      usedFallback: false,
    });
  });

  it("chooses the equal-scoring side region nearer the viewport centre", () => {
    // Arrange
    const viewport = { width: 1200, height: 400 };
    const strip = rect(600, 0, 640, 400);

    // Act
    const result = safeFrame(viewport, [strip], 1);

    // Assert
    expect(result.frame).toEqual(rect(0, 0, 592, 400));
  });

  it("returns the same frame for every obstacle order", () => {
    // Arrange
    const viewport = { width: 390, height: 844 };
    const obstacles = [
      rect(8, 8, 52, 52),
      rect(60, 8, 220, 40),
      rect(338, 8, 382, 104),
      rect(12, 460, 378, 750),
      rect(8, 792, 140, 836),
      rect(190, 760, 382, 836),
    ];
    const orders = [
      obstacles,
      [...obstacles].reverse(),
      [obstacles[3], obstacles[0], obstacles[5], obstacles[1], obstacles[4], obstacles[2]],
      [obstacles[2], obstacles[4], obstacles[1], obstacles[5], obstacles[0], obstacles[3]],
    ];

    // Act
    const frames = orders.map((order) => safeFrame(viewport, order, LANDSCAPE_ASPECT));

    // Assert
    for (const frame of frames) {
      expect(frame).toEqual(frames[0]);
    }
  });

  it("falls back to the full viewport when an obstacle covers the canvas", () => {
    // Arrange
    const viewport = { width: 640, height: 480 };
    const cover = rect(0, 0, 640, 480);

    // Act
    const result = safeFrame(viewport, [cover], LANDSCAPE_ASPECT);

    // Assert
    expect(result).toEqual({ frame: rect(0, 0, 640, 480), usedFallback: true });
  });

  it("falls back when the best fit is below half the unobstructed scale", () => {
    // Arrange
    const viewport = { width: 400, height: 400 };
    const topBand = rect(0, 0, 400, 150);
    const bottomBand = rect(0, 250, 400, 400);

    // Act
    const result = safeFrame(viewport, [topBand, bottomBand], 1);

    // Assert
    expect(result).toEqual({ frame: rect(0, 0, 400, 400), usedFallback: true });
  });
});

describe("screenRectInCanvas", () => {
  it("subtracts the canvas client origin from a client rectangle", () => {
    // Arrange
    const clientRect = rect(110, 220, 150, 260);

    // Act
    const canvasRect = screenRectInCanvas(clientRect, 10, 20);

    // Assert
    expect(canvasRect).toEqual(rect(100, 200, 140, 240));
  });
});

describe("sameScreenRects", () => {
  it("treats edges within half a pixel as equal", () => {
    // Arrange
    const first = [rect(0, 0, 10, 10)];
    const second = [rect(0.4, 0, 10, 9.6)];

    // Act
    const same = sameScreenRects(first, second);

    // Assert
    expect(same).toBe(true);
  });

  it("treats an edge moved by more than half a pixel as different", () => {
    // Arrange
    const first = [rect(0, 0, 10, 10)];
    const second = [rect(0, 0, 10.6, 10)];

    // Act
    const same = sameScreenRects(first, second);

    // Assert
    expect(same).toBe(false);
  });

  it("is sensitive to rectangle order", () => {
    // Arrange
    const a = rect(0, 0, 10, 10);
    const b = rect(20, 20, 30, 30);

    // Act
    const same = sameScreenRects([a, b], [b, a]);

    // Assert
    expect(same).toBe(false);
  });

  it("treats lists of different lengths as different", () => {
    // Arrange
    const a = rect(0, 0, 10, 10);

    // Act
    const same = sameScreenRects([a], [a, a]);

    // Assert
    expect(same).toBe(false);
  });
});
