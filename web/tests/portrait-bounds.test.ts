import { describe, expect, it } from "vitest";

import { SCENE_IDS, worldBoundsForScene, type SceneId } from "../src/catalog/scenes";
import {
  PORTRAIT_VIEW_BOUNDS,
  worldBoundsForViewport,
} from "../src/catalog/portrait-bounds";
import {
  IDENTITY_CAMERA_VIEW,
  createCamera,
  projectPoint,
} from "../src/render/camera";

const IPHONE = { width: 390, height: 844 } as const;

/** Smallest share of the iPhone height the fitted frame should cover. */
const MIN_HEIGHT_FRACTION: Record<SceneId, number> = {
  "wave-machine": 0.24,
  "dam-break": 0.5,
  fountain: 0.6,
  "float-or-sink": 0.65,
  "color-mixer": 0.4,
  "jelly-drop": 0.38,
  "water-wheel": 0.36,
  particles: 0.75,
  "liquid-timer": 0.4,
  "surface-tension": 0.75,
  "elastic-particles": 0.75,
  "rigid-particles": 0.75,
  soup: 0.32,
  "soup-stirrer": 0.32,
  impulse: 0.45,
  "theo-jansen": 0.38,
  "liquid-tumbler": 0.6,
};

function fittedFrameFraction(
  id: SceneId,
  viewport: { readonly width: number; readonly height: number },
): { readonly width: number; readonly height: number } {
  const bounds = worldBoundsForViewport(id, viewport.width, viewport.height);
  const camera = createCamera(
    viewport.width,
    viewport.height,
    IDENTITY_CAMERA_VIEW,
    bounds,
  );
  const topLeft = projectPoint(camera, { x: bounds.minX, y: bounds.maxY });
  const bottomRight = projectPoint(camera, { x: bounds.maxX, y: bounds.minY });
  return {
    width: (bottomRight.x - topLeft.x) / viewport.width,
    height: (bottomRight.y - topLeft.y) / viewport.height,
  };
}

describe("portrait scene frames", () => {
  it("uses a portrait frame on an iPhone and the shared frame on a wide screen", () => {
    // Arrange
    const id = "dam-break" as const;

    // Act
    const portrait = worldBoundsForViewport(id, IPHONE.width, IPHONE.height);
    const landscape = worldBoundsForViewport(id, IPHONE.height, IPHONE.width);

    // Assert
    expect(portrait).toEqual(PORTRAIT_VIEW_BOUNDS[id]);
    expect(landscape).toEqual(worldBoundsForScene(id));
  });

  it("fills most of an iPhone canvas for every scene", () => {
    // Arrange
    const ids = SCENE_IDS;

    // Act
    const fractions = ids.map((id) => ({
      id,
      ...fittedFrameFraction(id, IPHONE),
    }));

    // Assert
    for (const fraction of fractions) {
      expect(fraction.width, fraction.id).toBeGreaterThan(0.85);
      expect(fraction.height, fraction.id).toBeGreaterThan(
        MIN_HEIGHT_FRACTION[fraction.id],
      );
    }
  });

  it("keeps the dam obstacle and the timer bowl inside the iPhone frame", () => {
    // Arrange
    const dam = worldBoundsForViewport("dam-break", IPHONE.width, IPHONE.height);
    const timer = worldBoundsForViewport("liquid-timer", IPHONE.width, IPHONE.height);

    // Act
    const obstacleInside =
      dam.minX <= 2.5 - 0.75 &&
      dam.maxX >= 2.5 + 0.75 &&
      dam.minY <= 5.5 - 0.75 &&
      dam.maxY >= 5.5 + 0.75;
    const bowlInside =
      timer.minX <= -2 &&
      timer.maxX >= 2 &&
      timer.minY <= 0 &&
      timer.maxY >= 4;

    // Assert
    expect(obstacleInside).toBe(true);
    expect(bowlInside).toBe(true);
  });

  it("keeps every portrait frame finite and non-empty", () => {
    // Arrange
    const ids = SCENE_IDS;

    // Act
    const spans = ids.map((id) => {
      const bounds = PORTRAIT_VIEW_BOUNDS[id];
      return {
        id,
        width: bounds.maxX - bounds.minX,
        height: bounds.maxY - bounds.minY,
      };
    });

    // Assert
    for (const span of spans) {
      expect(span.width).toBeGreaterThan(0);
      expect(span.height).toBeGreaterThan(0);
      expect(Number.isFinite(span.width)).toBe(true);
      expect(Number.isFinite(span.height)).toBe(true);
    }
  });
});
