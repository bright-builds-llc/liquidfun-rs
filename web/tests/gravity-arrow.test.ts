import { describe, expect, it } from "vitest";

import {
  FULL_GRAVITY,
  gravityArrowGeometry,
  maybeGravityArrow,
  type GravityArrowGeometry,
} from "../src/components/gravity-arrow";
import type { TiltDebug } from "../src/input/tilt-gravity";

const liveDown: TiltDebug = {
  kind: "live",
  sample: { x: 0, y: 9.8, z: 0 },
  gravity: { x: 0, y: -9.8 },
  screenAngleDegrees: 0,
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

  it("reaches full length at one g", () => {
    // Arrange / Act
    const arrow = gravityArrowGeometry({ x: 0, y: -FULL_GRAVITY });

    // Assert
    expect(tipDistance(arrow)).toBeCloseTo(14);
  });

  it("clips arrows stronger than one g to the full-gravity length", () => {
    // Arrange
    const full = gravityArrowGeometry({ x: 0, y: -FULL_GRAVITY });

    // Act
    const stronger = gravityArrowGeometry({ x: 0, y: -(FULL_GRAVITY * 2) });

    // Assert
    expect(tipDistance(stronger)).toBeCloseTo(tipDistance(full));
  });

  it("draws half of one g at half the full length", () => {
    // Arrange
    const full = gravityArrowGeometry({ x: FULL_GRAVITY, y: 0 });

    // Act
    const half = gravityArrowGeometry({ x: FULL_GRAVITY / 2, y: 0 });

    // Assert
    expect(tipDistance(half)).toBeCloseTo(tipDistance(full) / 2);
  });

  it("scales the stroke with length and clips it at one g", () => {
    // Arrange
    const full = gravityArrowGeometry({ x: 0, y: -FULL_GRAVITY });
    const half = gravityArrowGeometry({ x: 0, y: -FULL_GRAVITY / 2 });

    // Act
    const stronger = gravityArrowGeometry({ x: 0, y: -(FULL_GRAVITY * 2) });

    // Assert
    expect(full?.strokeWidth).toBeCloseTo(2.5);
    expect(half?.strokeWidth).toBeCloseTo(1.25);
    expect(stronger?.strokeWidth).toBeCloseTo(full?.strokeWidth ?? 0);
  });

  it("shrinks flat-phone noise to a small fraction of full gravity", () => {
    // Arrange
    const full = gravityArrowGeometry({ x: 0, y: -FULL_GRAVITY });
    const flatNoise = 0.2;

    // Act
    const noise = gravityArrowGeometry({ x: 0, y: -flatNoise });

    // Assert
    expect(tipDistance(noise)).toBeLessThan(tipDistance(full) * 0.05);
    expect(tipDistance(noise)).toBeGreaterThan(0);
  });
});

describe("maybeGravityArrow", () => {
  it("draws when accelerometer gravity has a live sample", () => {
    // Arrange / Act
    const shown = maybeGravityArrow(true, liveDown);

    // Assert
    expect(shown?.y2 ?? 0).toBeGreaterThan(shown?.y1 ?? 0);
  });

  it("omits the arrow when accelerometer gravity is off", () => {
    // Arrange / Act / Assert
    expect(maybeGravityArrow(false, liveDown)).toBeUndefined();
  });

  it("omits the arrow while waiting for a sample", () => {
    // Arrange / Act / Assert
    expect(maybeGravityArrow(true, { kind: "waiting" })).toBeUndefined();
  });
});

function tipDistance(maybeArrow: GravityArrowGeometry | undefined): number {
  if (maybeArrow === undefined) {
    return 0;
  }
  return Math.hypot(maybeArrow.x2 - 24, maybeArrow.y2 - 24);
}
