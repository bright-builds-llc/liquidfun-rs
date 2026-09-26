import { describe, expect, it } from "vitest";

import { DAM_BREAK_GRAVITY_DEFAULT, maybeSceneById } from "../src/catalog/scenes";
import {
  maybeGravitySliderMagnitude,
  maybeRouteGravitySliderMagnitude,
} from "../src/input/gravity-slider";

describe("maybeGravitySliderMagnitude", () => {
  it("uses Dam Break's normal gravity when the bag is empty", () => {
    // Arrange
    const scene = maybeSceneById("dam-break");

    // Act
    const magnitude = maybeGravitySliderMagnitude(scene, {});

    // Assert
    expect(magnitude).toBe(DAM_BREAK_GRAVITY_DEFAULT);
  });

  it("reads an applied Dam Break slider magnitude", () => {
    // Arrange
    const scene = maybeSceneById("dam-break");

    // Act
    const magnitude = maybeGravitySliderMagnitude(scene, { gravity: "80" });

    // Assert
    expect(magnitude).toBe(80);
  });

  it("falls back to normal gravity for a retired preset name", () => {
    // Arrange
    const scene = maybeSceneById("dam-break");

    // Act
    const magnitude = maybeGravitySliderMagnitude(scene, { gravity: "high" });

    // Assert
    expect(magnitude).toBe(DAM_BREAK_GRAVITY_DEFAULT);
  });

  it("reads the slider from a ready scene route", () => {
    // Arrange / Act
    const magnitude = maybeRouteGravitySliderMagnitude(
      { kind: "scene", id: "dam-break" },
      { gravity: "16" },
    );

    // Assert
    expect(magnitude).toBe(16);
  });

  it("returns undefined when the route has no scene", () => {
    // Arrange / Act / Assert
    expect(
      maybeRouteGravitySliderMagnitude({ kind: "empty" }, { gravity: "80" }),
    ).toBeUndefined();
  });

  it("returns undefined for a scene without a gravity slider", () => {
    // Arrange
    const scene = maybeSceneById("fountain");

    // Act
    const magnitude = maybeGravitySliderMagnitude(scene, { gravity: "80" });

    // Assert
    expect(magnitude).toBeUndefined();
  });
});
