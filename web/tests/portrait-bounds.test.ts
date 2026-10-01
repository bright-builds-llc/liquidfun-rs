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

/** Inner faces of the shared tall basin, matching the scene segment endpoints. */
const TALL_BASIN_WALLS = [
  { x: -5.5, y: 0 },
  { x: -5.5, y: 8 },
  { x: 5.5, y: 0 },
  { x: 5.5, y: 8 },
] as const;

/** Floor and outer wall tips for the falling-ball basins. */
const FLOOR_WALLS = [
  { x: -4, y: 0 },
  { x: 4, y: 0 },
  { x: -4, y: 3 },
  { x: 4, y: 3 },
  { x: -2, y: 2 },
  { x: 2, y: 2 },
] as const;

/** Soup floor and the tops of the slanted walls. */
const SOUP_WALLS = [
  { x: -4, y: 0 },
  { x: 4, y: 0 },
  { x: -4, y: 3 },
  { x: 4, y: 3 },
  { x: -2, y: 2 },
  { x: 2, y: 2 },
] as const;

/** Smallest share of the iPhone height the fitted frame should cover. */
const MIN_HEIGHT_FRACTION: Record<SceneId, number> = {
  "wave-machine": 0.24,
  "dam-break": 0.3,
  fountain: 0.3,
  "float-or-sink": 0.3,
  "color-mixer": 0.3,
  "jelly-drop": 0.38,
  "water-wheel": 0.3,
  particles: 0.43,
  "liquid-timer": 0.4,
  "surface-tension": 0.43,
  "elastic-particles": 0.43,
  "rigid-particles": 0.43,
  soup: 0.16,
  "soup-stirrer": 0.16,
  impulse: 0.45,
  "theo-jansen": 0.12,
  "liquid-tumbler": 0.6,
  "drawing-particles": 0.4,
  sparky: 0.4,
  "hydraulic-fountain": 0.52,
  "wave-tank": 0.20,
  "liquid-bubbler": 0.40,
  "stacked-drip": 0.39,
  "washing-machine": 0.40,
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

  it("keeps each scene's walls inside the iPhone frame", () => {
    // Arrange
    const wallEndpoints = {
      "dam-break": TALL_BASIN_WALLS,
      fountain: TALL_BASIN_WALLS,
      "float-or-sink": TALL_BASIN_WALLS,
      "color-mixer": TALL_BASIN_WALLS,
      "water-wheel": TALL_BASIN_WALLS,
      particles: FLOOR_WALLS,
      "surface-tension": FLOOR_WALLS,
      "elastic-particles": FLOOR_WALLS,
      "rigid-particles": FLOOR_WALLS,
      soup: SOUP_WALLS,
      "soup-stirrer": SOUP_WALLS,
      "jelly-drop": [
        { x: -3.55, y: 0.85 },
        { x: -3.55, y: 7.2 },
        { x: 3.55, y: 0.85 },
        { x: 3.55, y: 7.2 },
      ],
      "liquid-timer": [
        { x: -2, y: 0 },
        { x: -2, y: 4 },
        { x: 2, y: 0 },
        { x: 2, y: 4 },
      ],
      impulse: [
        { x: -2, y: 0 },
        { x: -2, y: 4 },
        { x: 2, y: 0 },
        { x: 2, y: 4 },
      ],
      "liquid-tumbler": [
        { x: -0.037, y: 0 },
        { x: -0.037, y: 0.12 },
        { x: 0.037, y: 0 },
        { x: 0.037, y: 0.12 },
      ],
      "drawing-particles": [
        { x: -4, y: -2 },
        { x: 4, y: -2 },
        { x: -4, y: 6 },
        { x: 4, y: 6 },
        { x: -2, y: 0 },
        { x: 2, y: 0 },
        { x: -2, y: 4 },
        { x: 2, y: 4 },
      ],
      sparky: [
        { x: -20, y: 0 },
        { x: 20, y: 0 },
        { x: -20, y: 40 },
        { x: 20, y: 40 },
      ],
      "liquid-bubbler": [
        { x: -0.63, y: 0 },
        { x: -0.63, y: 2.4 },
        { x: -0.55, y: 0 },
        { x: -0.55, y: 2.4 },
        { x: -0.63, y: -0.08 },
        { x: 1.04, y: -0.08 },
        { x: -0.63, y: 0 },
        { x: 1.04, y: 0 },
        { x: 0.96, y: 0 },
        { x: 0.96, y: 2.4 },
        { x: 1.04, y: 0 },
        { x: 1.04, y: 2.4 },
        { x: 0.48, y: 0.08 },
        { x: 0.56, y: 0.08 },
        { x: 0.48, y: 2.0 },
        { x: 0.56, y: 2.0 },
        { x: -0.55, y: 1.41 },
        { x: -0.42, y: 1.41 },
        { x: -0.55, y: 1.49 },
        { x: -0.42, y: 1.49 },
        { x: -0.22, y: 1.41 },
        { x: 0.48, y: 1.41 },
        { x: -0.22, y: 1.49 },
        { x: 0.48, y: 1.49 },
        { x: -0.55, y: 0.97 },
        { x: 0.08, y: 0.97 },
        { x: -0.55, y: 1.05 },
        { x: 0.08, y: 1.05 },
        { x: 0.28, y: 0.97 },
        { x: 0.48, y: 0.97 },
        { x: 0.28, y: 1.05 },
        { x: 0.48, y: 1.05 },
        { x: -0.55, y: 0.53 },
        { x: -0.42, y: 0.53 },
        { x: -0.55, y: 0.61 },
        { x: -0.42, y: 0.61 },
        { x: -0.22, y: 0.53 },
        { x: 0.48, y: 0.53 },
        { x: -0.22, y: 0.61 },
        { x: 0.48, y: 0.61 },
        { x: -0.43, y: 1.23 },
        { x: -0.21, y: 1.23 },
        { x: -0.32, y: 1.34 },
        { x: -0.32, y: 1.12 },
        { x: 0.07, y: 0.79 },
        { x: 0.29, y: 0.79 },
        { x: 0.18, y: 0.9 },
        { x: 0.18, y: 0.68 },
        { x: -0.43, y: 0.35 },
        { x: -0.21, y: 0.35 },
        { x: -0.32, y: 0.46 },
        { x: -0.32, y: 0.24 },
        { x: 0.5738, y: 0.0205 },
        { x: 0.9476, y: 0.0335 },
        { x: 0.9462, y: 0.0735 },
        { x: 0.5724, y: 0.0605 },
        { x: 0.5738, y: 2.2205 },
        { x: 0.9476, y: 2.2335 },
        { x: 0.9462, y: 2.2735 },
        { x: 0.5724, y: 2.2605 },
      ],
      "stacked-drip": [
        { x: -0.78, y: 0 },
        { x: -0.7, y: 0 },
        { x: -0.7, y: 3.2 },
        { x: -0.78, y: 3.2 },
        { x: -0.78, y: -0.08 },
        { x: 0.9, y: -0.08 },
        { x: 0.9, y: 0.08 },
        { x: -0.7, y: 0.16 },
        { x: 0.9, y: -0.08 },
        { x: 1.6, y: -0.08 },
        { x: 1.6, y: 0 },
        { x: 0.9, y: 0 },
        { x: 1.52, y: 0 },
        { x: 1.6, y: 0 },
        { x: 1.6, y: 3.2 },
        { x: 1.52, y: 3.2 },
        { x: -0.78, y: 3.01 },
        { x: 1.6, y: 3.01 },
        { x: 1.6, y: 3.09 },
        { x: -0.78, y: 3.09 },
        { x: 0.9, y: 0.22 },
        { x: 0.98, y: 0.22 },
        { x: 0.98, y: 1.7 },
        { x: 0.9, y: 1.7 },
        { x: -0.45, y: 1.62 },
        { x: 0.75, y: 1.62 },
        { x: 0.75, y: 1.98 },
        { x: -0.45, y: 1.98 },
        { x: 0.25, y: 1.44 },
        { x: 0.73, y: 1.44 },
        { x: 0.73, y: 1.46 },
        { x: 0.25, y: 1.46 },
        { x: 0.87, y: 1.41 },
        { x: 0.87, y: 1.49 },
        { x: 0.37, y: 1.16 },
        { x: 0.71, y: 1.5 },
        { x: 0.7, y: 1.51 },
        { x: 0.36, y: 1.17 },
        { x: -0.05, y: 0.99 },
        { x: 0.43, y: 0.99 },
        { x: 0.43, y: 1.01 },
        { x: -0.05, y: 1.01 },
        { x: 0.07, y: 0.71 },
        { x: 0.41, y: 1.05 },
        { x: 0.4, y: 1.06 },
        { x: 0.06, y: 0.72 },
        { x: -0.35, y: 0.54 },
        { x: 0.13, y: 0.54 },
        { x: 0.13, y: 0.56 },
        { x: -0.35, y: 0.56 },
        { x: -0.23, y: 0.26 },
        { x: 0.11, y: 0.6 },
        { x: 0.1, y: 0.61 },
        { x: -0.24, y: 0.27 },
        { x: 0.97, y: 0.02 },
        { x: 1.53, y: 0.02 },
        { x: 1.53, y: 0.06 },
        { x: 0.97, y: 0.06 },
        { x: 0.97, y: 1.92 },
        { x: 1.53, y: 1.92 },
        { x: 1.53, y: 1.96 },
        { x: 0.97, y: 1.96 },
      ],
      "wave-tank": [
        { x: -0.04, y: 0 },
        { x: -0.04, y: 0.9 },
        { x: 0, y: 0 },
        { x: 0, y: 0.9 },
        { x: 0.5, y: 0 },
        { x: 1.4, y: 0 },
        { x: 1.4, y: 0.9 },
        { x: 1.44, y: 0 },
        { x: 1.44, y: 0.9 },
        { x: 0.02, y: -0.04 },
        { x: 0.48, y: 0 },
        { x: 0.02, y: 0.12 },
        { x: 0.48, y: 0.16 },
        { x: 0.48, y: -0.2 },
        { x: 0.52, y: 0.04 },
        { x: 0.48, y: -0.04 },
        { x: 0.52, y: 0.2 },
      ],
      "hydraulic-fountain": [
        { x: -1.02, y: 0 },
        { x: 1.02, y: 0 },
        { x: -1.02, y: 2.85 },
        { x: 1.02, y: 2.85 },
        { x: -1.1, y: -0.04 },
        { x: 1.1, y: -0.04 },
        { x: -1.1, y: 2.85 },
        { x: 1.1, y: 2.85 },
        { x: -1.01, y: 1.22 },
        { x: -0.08, y: 1.22 },
        { x: -1.01, y: 1.33 },
        { x: -0.08, y: 1.33 },
        { x: 0.08, y: 1.22 },
        { x: 1.01, y: 1.22 },
        { x: 0.08, y: 1.33 },
        { x: 1.01, y: 1.33 },
        { x: -1.01, y: 0.055 },
        { x: -0.08, y: 0.055 },
        { x: 0.08, y: 0.165 },
        { x: 1.01, y: 0.165 },
      ],
      "wave-machine": [
        { x: -2.05, y: 0 },
        { x: 2.05, y: 0 },
        { x: -2.05, y: 2 },
        { x: 2.05, y: 2 },
      ],
      "washing-machine": [
        { x: 1.48, y: 0 },
        { x: -1.48, y: 0 },
        { x: 0, y: 1.48 },
        { x: 0, y: -1.48 },
        { x: 0.636, y: 0.636 },
        { x: -0.636, y: 0.636 },
        { x: -0.636, y: -0.636 },
        { x: 0.636, y: -0.636 },
      ],
    } as const;

    // Act
    const outside = Object.entries(wallEndpoints).flatMap(([id, points]) => {
      const bounds = worldBoundsForViewport(
        id as SceneId,
        IPHONE.width,
        IPHONE.height,
      );
      return points
        .filter(
          (point) =>
            point.x < bounds.minX ||
            point.x > bounds.maxX ||
            point.y < bounds.minY ||
            point.y > bounds.maxY,
        )
        .map((point) => `${id} (${point.x}, ${point.y})`);
    });

    // Assert
    expect(outside).toEqual([]);
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
