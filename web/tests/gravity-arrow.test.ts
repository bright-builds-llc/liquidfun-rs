import { describe, expect, it } from "vitest";

import { gravityArrowGeometry, maybeGravityArrow } from "../src/components/gravity-arrow";
import type { TiltDebug } from "../src/input/tilt-gravity";

const liveDown: TiltDebug = {
  kind: "live",
  sample: { x: 0, y: 9.8, z: 0 },
  gravity: { x: 0, y: -9.8 },
};

describe("gravityArrowGeometry", () => {
  it("points down the screen when world gravity is downward", () => {
    // Arrange / Act
    const arrow = gravityArrowGeometry({ x: 0, y: -9.8 });

    // Assert
    expect(arrow).toBeDefined();
    expect(arrow?.x2).toBeCloseTo(24);
    expect(arrow?.y2 ?? 0).toBeGreaterThan(arrow?.y1 ?? 0);
  });

  it("points right when world gravity points right", () => {
    // Arrange / Act
    const arrow = gravityArrowGeometry({ x: 9.8, y: 0 });

    // Assert
    expect(arrow?.x2 ?? 0).toBeGreaterThan(arrow?.x1 ?? 0);
    expect(arrow?.y2).toBeCloseTo(24);
  });

  it("omits an arrow for a zero gravity sample", () => {
    // Arrange / Act / Assert
    expect(gravityArrowGeometry({ x: 0, y: 0 })).toBeUndefined();
  });
});

describe("maybeGravityArrow", () => {
  it("draws only when debug info, tilt, and a live sample are on", () => {
    // Arrange / Act
    const shown = maybeGravityArrow(true, true, liveDown);
    const debugOff = maybeGravityArrow(false, true, liveDown);
    const tiltOff = maybeGravityArrow(true, false, liveDown);
    const waiting = maybeGravityArrow(true, true, { kind: "waiting" });

    // Assert
    expect(shown?.y2 ?? 0).toBeGreaterThan(shown?.y1 ?? 0);
    expect(debugOff).toBeUndefined();
    expect(tiltOff).toBeUndefined();
    expect(waiting).toBeUndefined();
  });
});
