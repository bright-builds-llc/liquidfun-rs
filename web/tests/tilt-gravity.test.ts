import { describe, expect, it } from "vitest";

import {
  TILT_GRAVITY_LIMIT,
  accelerationConventionFromEnvironment,
  describeUnknownError,
  formatTiltDebug,
  interpretAcceleration,
  maybeWorldGravityFromAcceleration,
} from "../src/input/tilt-gravity";

describe("maybeWorldGravityFromAcceleration", () => {
  it("points world gravity down when the phone is held upright", () => {
    // Arrange
    const accelerationY = 9.8;

    // Act
    const gravity = maybeWorldGravityFromAcceleration(0, accelerationY);

    // Assert
    expect(gravity?.x).toBeCloseTo(0);
    expect(gravity?.y).toBeCloseTo(-9.8);
  });

  it("points world gravity toward the right when that side is down", () => {
    // Arrange / Act
    const gravity = maybeWorldGravityFromAcceleration(-9.8, 0);

    // Assert
    expect(gravity?.x).toBeCloseTo(9.8);
    expect(gravity?.y).toBeCloseTo(0);
  });

  it("limits a shaken sample to the gravity magnitude cap", () => {
    // Arrange / Act
    const gravity = maybeWorldGravityFromAcceleration(0, 40);

    // Assert
    expect(gravity?.y).toBeCloseTo(-TILT_GRAVITY_LIMIT);
    expect(Math.hypot(gravity?.x ?? 0, gravity?.y ?? 0)).toBeCloseTo(TILT_GRAVITY_LIMIT);
  });

  it("ignores a non-finite sample", () => {
    // Arrange / Act / Assert
    expect(maybeWorldGravityFromAcceleration(Number.NaN, 1)).toBeUndefined();
  });

  it("keeps an upright iPhone sample pointing down the screen", () => {
    // Arrange / Act
    const gravity = maybeWorldGravityFromAcceleration(0, -9.8, "gravity-direction");

    // Assert
    expect(gravity?.y).toBeCloseTo(-9.8);
  });

  it("sends an upside-down iPhone sample toward the top of the screen", () => {
    // Arrange / Act
    const gravity = maybeWorldGravityFromAcceleration(0, 9.8, "gravity-direction");

    // Assert
    expect(gravity?.y).toBeCloseTo(9.8);
  });
});

describe("accelerationConventionFromEnvironment", () => {
  it("uses the fall direction when the browser asks for motion permission", () => {
    // Arrange / Act / Assert
    expect(
      accelerationConventionFromEnvironment({
        hasRequestPermission: true,
        userAgent: "Mozilla/5.0",
      }),
    ).toBe("gravity-direction");
  });

  it("uses the fall direction for an iPhone user agent", () => {
    // Arrange / Act / Assert
    expect(
      accelerationConventionFromEnvironment({
        hasRequestPermission: false,
        userAgent: "Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X)",
      }),
    ).toBe("gravity-direction");
  });

  it("uses the upward support force on other browsers", () => {
    // Arrange / Act / Assert
    expect(
      accelerationConventionFromEnvironment({
        hasRequestPermission: false,
        userAgent: "Mozilla/5.0 (Linux; Android 14)",
      }),
    ).toBe("support-force");
  });
});

describe("interpretAcceleration", () => {
  it("keeps the raw axes and the mapped gravity", () => {
    // Arrange / Act
    const report = interpretAcceleration({ x: 0, y: 9.8, z: 0.2 });

    // Assert
    expect(report.kind).toBe("live");
    if (report.kind !== "live") {
      return;
    }
    expect(report.sample).toEqual({ x: 0, y: 9.8, z: 0.2 });
    expect(report.gravity.y).toBeCloseTo(-9.8);
  });

  it("reports a null acceleration field with the raw sample", () => {
    // Arrange / Act
    const report = interpretAcceleration({ x: null, y: 1, z: 2 });

    // Assert
    expect(report).toEqual({
      kind: "problem",
      detail: "accelerationIncludingGravity.x is null",
      sample: { x: null, y: 1, z: 2 },
    });
  });

  it("reports a missing acceleration object", () => {
    // Arrange / Act / Assert
    expect(interpretAcceleration(null).kind).toBe("problem");
  });
});

describe("formatTiltDebug", () => {
  it("explains that scene gravity stays put while the switch is off", () => {
    // Arrange / Act / Assert
    expect(formatTiltDebug({ kind: "idle" })).toBe(
      "Off. The scene keeps its own gravity.",
    );
  });

  it("prints raw axes and gravity for a live sample", () => {
    // Arrange / Act
    const text = formatTiltDebug({
      kind: "live",
      sample: { x: 1, y: -2, z: null },
      gravity: { x: -1, y: 2 },
    });

    // Assert
    expect(text).toBe("ax 1.000  ay -2.000  az null\ngravity -1.000, 2.000");
  });

  it("prints the raw sample and the error together", () => {
    // Arrange / Act
    const text = formatTiltDebug({
      kind: "problem",
      detail: 'NotAllowedError: The user denied permission',
      maybeSample: { x: null, y: null, z: null },
    });

    // Assert
    expect(text).toContain("NotAllowedError: The user denied permission");
    expect(text).toContain("ax null");
  });
});

describe("describeUnknownError", () => {
  it("keeps the error name and message", () => {
    // Arrange
    const error = new Error("The user denied permission");
    error.name = "NotAllowedError";

    // Act / Assert
    expect(describeUnknownError(error)).toBe(
      "NotAllowedError: The user denied permission",
    );
  });
});
