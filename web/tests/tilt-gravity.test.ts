import { describe, expect, it } from "vitest";

import {
  STANDARD_GRAVITY,
  TILT_GRAVITY_LIMIT,
  accelerationConventionFromEnvironment,
  describeUnknownError,
  formatTiltDebug,
  interpretAcceleration,
  maybeWorldGravityFromAcceleration,
  scaleGravityBySlider,
  screenAngleDegreesFromEnvironment,
  worldGravityFromTilt,
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

  it("keeps gravity toward the ground after a counter-clockwise quarter turn", () => {
    // Arrange
    const landscapeRightEdgeUp = 90;

    // Act
    const gravity = maybeWorldGravityFromAcceleration(
      9.8,
      0,
      "support-force",
      landscapeRightEdgeUp,
    );

    // Assert
    expect(gravity?.x).toBeCloseTo(0);
    expect(gravity?.y).toBeCloseTo(-9.8);
  });

  it("keeps gravity toward the ground when the phone is upside down", () => {
    // Arrange / Act
    const gravity = maybeWorldGravityFromAcceleration(0, -9.8, "support-force", 180);

    // Assert
    expect(gravity?.x).toBeCloseTo(0);
    expect(gravity?.y).toBeCloseTo(-9.8);
  });

  it("keeps gravity toward the ground after a clockwise quarter turn", () => {
    // Arrange / Act
    const gravity = maybeWorldGravityFromAcceleration(-9.8, 0, "support-force", 270);

    // Assert
    expect(gravity?.x).toBeCloseTo(0);
    expect(gravity?.y).toBeCloseTo(-9.8);
  });

  it("treats a legacy window.orientation of -90 as a clockwise quarter turn", () => {
    // Arrange / Act
    const gravity = maybeWorldGravityFromAcceleration(-9.8, 0, "support-force", -90);

    // Assert
    expect(gravity?.y).toBeCloseTo(-9.8);
  });

  it("points gravity toward the right of a landscape screen", () => {
    // Arrange / Act
    const gravity = maybeWorldGravityFromAcceleration(0, 9.8, "support-force", 90);

    // Assert
    expect(gravity?.x).toBeCloseTo(9.8);
    expect(gravity?.y).toBeCloseTo(0);
  });

  it("keeps an iPhone landscape sample pointing down the screen", () => {
    // Arrange / Act
    const gravity = maybeWorldGravityFromAcceleration(
      -9.8,
      0,
      "gravity-direction",
      90,
    );

    // Assert
    expect(gravity?.x).toBeCloseTo(0);
    expect(gravity?.y).toBeCloseTo(-9.8);
  });

  it("ignores a non-finite screen angle", () => {
    // Arrange / Act
    const gravity = maybeWorldGravityFromAcceleration(0, 9.8, "support-force", Number.NaN);

    // Assert
    expect(gravity?.y).toBeCloseTo(-9.8);
  });
});

describe("screenAngleDegreesFromEnvironment", () => {
  it("prefers window.orientation when an iPhone reports the opposite screen angle", () => {
    // Arrange / Act
    const angle = screenAngleDegreesFromEnvironment({
      hasRequestPermission: false,
      userAgent:
        "Mozilla/5.0 (iPhone; CPU iPhone OS 16_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Mobile/15E148 Safari/604.1",
      maybeScreenOrientationAngle: 270,
      maybeWindowOrientation: 90,
    });

    // Assert
    expect(angle).toBe(90);
  });

  it("prefers window.orientation for iPadOS desktop mode", () => {
    // Arrange / Act
    const angle = screenAngleDegreesFromEnvironment({
      hasRequestPermission: true,
      userAgent:
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Safari/605.1.15",
      maybeScreenOrientationAngle: 270,
      maybeWindowOrientation: 90,
    });

    // Assert
    expect(angle).toBe(90);
  });

  it("prefers screen.orientation.angle on Samsung Internet", () => {
    // Arrange / Act
    const angle = screenAngleDegreesFromEnvironment({
      hasRequestPermission: false,
      userAgent:
        "Mozilla/5.0 (Linux; Android 14; SAMSUNG SM-S918B) AppleWebKit/537.36 (KHTML, like Gecko) SamsungBrowser/24.0 Chrome/120.0.0.0 Mobile Safari/537.36",
      maybeScreenOrientationAngle: 90,
      maybeWindowOrientation: 270,
    });

    // Assert
    expect(angle).toBe(90);
  });

  it("uses the screen angle when Firefox has no window.orientation", () => {
    // Arrange / Act
    const angle = screenAngleDegreesFromEnvironment({
      hasRequestPermission: false,
      userAgent: "Mozilla/5.0 (Android 14; Mobile; rv:128.0) Gecko/128.0 Firefox/128.0",
      maybeScreenOrientationAngle: 180,
      maybeWindowOrientation: null,
    });

    // Assert
    expect(angle).toBe(180);
  });

  it("falls back to window.orientation when the screen angle is missing", () => {
    // Arrange / Act
    const angle = screenAngleDegreesFromEnvironment({
      hasRequestPermission: false,
      userAgent: "Mozilla/5.0 (Linux; Android 14)",
      maybeScreenOrientationAngle: null,
      maybeWindowOrientation: -90,
    });

    // Assert
    expect(angle).toBe(-90);
  });

  it("skips a non-finite preferred angle", () => {
    // Arrange / Act
    const angle = screenAngleDegreesFromEnvironment({
      hasRequestPermission: false,
      userAgent: "Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X)",
      maybeScreenOrientationAngle: 180,
      maybeWindowOrientation: Number.NaN,
    });

    // Assert
    expect(angle).toBe(180);
  });

  it("uses zero when neither orientation source is available", () => {
    // Arrange / Act
    const angle = screenAngleDegreesFromEnvironment({
      hasRequestPermission: false,
      userAgent: "Mozilla/5.0",
      maybeScreenOrientationAngle: null,
      maybeWindowOrientation: null,
    });

    // Assert
    expect(angle).toBe(0);
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
    expect(report.screenAngleDegrees).toBe(0);
  });

  it("remaps a landscape sample and records the screen angle", () => {
    // Arrange / Act
    const report = interpretAcceleration({ x: 9.8, y: 0, z: 0.2 }, "support-force", 90);

    // Assert
    expect(report.kind).toBe("live");
    if (report.kind !== "live") {
      return;
    }
    expect(report.gravity.y).toBeCloseTo(-9.8);
    expect(report.screenAngleDegrees).toBe(90);
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
      screenAngleDegrees: 90,
    });

    // Assert
    expect(text).toBe(
      "ax 1.000  ay -2.000  az null\ngravity -1.000, 2.000\nscreen 90°",
    );
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

describe("scaleGravityBySlider", () => {
  it("turns one standard g into the slider magnitude", () => {
    // Arrange
    const measured = { x: 0, y: -STANDARD_GRAVITY };

    // Act
    const scaled = scaleGravityBySlider(measured, 10);

    // Assert
    expect(scaled?.x).toBeCloseTo(0);
    expect(scaled?.y).toBeCloseTo(-10);
  });

  it("keeps a partial tilt proportional to the slider", () => {
    // Arrange
    const measured = {
      x: STANDARD_GRAVITY / 2,
      y: -STANDARD_GRAVITY / 2,
    };

    // Act
    const scaled = scaleGravityBySlider(measured, 16);

    // Assert
    expect(scaled?.x).toBeCloseTo(8);
    expect(scaled?.y).toBeCloseTo(-8);
  });

  it("scales a capped shake by the same proportion", () => {
    // Arrange
    const measured = maybeWorldGravityFromAcceleration(0, 40);
    if (measured === undefined) {
      throw new Error("expected a capped gravity sample");
    }

    // Act
    const scaled = scaleGravityBySlider(measured, 80);

    // Assert
    expect(Math.hypot(scaled?.x ?? 0, scaled?.y ?? 0)).toBeCloseTo(
      (TILT_GRAVITY_LIMIT * 80) / STANDARD_GRAVITY,
    );
  });

  it("rejects a negative slider magnitude", () => {
    // Arrange / Act / Assert
    expect(scaleGravityBySlider({ x: 0, y: -STANDARD_GRAVITY }, -1)).toBeUndefined();
  });
});

describe("worldGravityFromTilt", () => {
  it("keeps the measured vector when the scene has no gravity slider", () => {
    // Arrange
    const measured = { x: 1.5, y: -STANDARD_GRAVITY };

    // Act
    const applied = worldGravityFromTilt(measured, undefined);

    // Assert
    expect(applied.gravity).toEqual(measured);
    expect(applied.fullLengthMagnitude).toBe(STANDARD_GRAVITY);
  });

  it("uses the slider as the full-length reference", () => {
    // Arrange
    const measured = { x: 0, y: -STANDARD_GRAVITY };

    // Act
    const applied = worldGravityFromTilt(measured, 80);

    // Assert
    expect(applied.gravity.y).toBeCloseTo(-80);
    expect(applied.fullLengthMagnitude).toBe(80);
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
