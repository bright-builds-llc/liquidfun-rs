import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import {
  VIEWPORT_INSET,
  WORLD_BOUNDS,
  createCamera,
  projectPoint,
  projectRadius,
  unprojectPoint,
  zoomCameraView,
  IDENTITY_CAMERA_VIEW,
} from "../src/render/camera";

describe("createCamera", () => {
  it("centers the fixed world in a 960 by 540 viewport", () => {
    // Arrange
    const camera = createCamera(960, 540);

    // Act
    const center = projectPoint(camera, { x: 0, y: 3.5 });

    // Assert
    expect(center.x).toBeCloseTo(480);
    expect(center.y).toBeCloseTo(270);
  });

  it("preserves world aspect ratio in the design viewport", () => {
    // Arrange
    const camera = createCamera(960, 540);

    // Act
    const minimum = projectPoint(camera, {
      x: WORLD_BOUNDS.minX,
      y: WORLD_BOUNDS.minY,
    });
    const maximum = projectPoint(camera, {
      x: WORLD_BOUNDS.maxX,
      y: WORLD_BOUNDS.maxY,
    });
    const projectedWidth = maximum.x - minimum.x;
    const projectedHeight = minimum.y - maximum.y;

    // Assert
    expect(projectedWidth / projectedHeight).toBeCloseTo(12 / 9);
  });

  it("keeps every world corner inside the approved inset", () => {
    // Arrange
    const camera = createCamera(960, 540);

    // Act
    const projectedCorners = [
      projectPoint(camera, { x: -6, y: -1 }),
      projectPoint(camera, { x: -6, y: 8 }),
      projectPoint(camera, { x: 6, y: -1 }),
      projectPoint(camera, { x: 6, y: 8 }),
    ];

    // Assert
    for (const corner of projectedCorners) {
      expect(corner.x).toBeGreaterThanOrEqual(VIEWPORT_INSET);
      expect(corner.x).toBeLessThanOrEqual(960 - VIEWPORT_INSET);
      expect(corner.y).toBeGreaterThanOrEqual(VIEWPORT_INSET);
      expect(corner.y).toBeLessThanOrEqual(540 - VIEWPORT_INSET);
    }
  });

  it("centers and contains the world in a narrow viewport", () => {
    // Arrange
    const camera = createCamera(320, 540);

    // Act
    const minimum = projectPoint(camera, { x: -6, y: -1 });
    const maximum = projectPoint(camera, { x: 6, y: 8 });

    // Assert
    expect(minimum.x).toBeCloseTo(VIEWPORT_INSET);
    expect(maximum.x).toBeCloseTo(320 - VIEWPORT_INSET);
    expect((minimum.y + maximum.y) / 2).toBeCloseTo(270);
    expect(maximum.y).toBeGreaterThanOrEqual(VIEWPORT_INSET);
    expect(minimum.y).toBeLessThanOrEqual(540 - VIEWPORT_INSET);
  });

  it("maps greater world y values to smaller canvas y values", () => {
    // Arrange
    const camera = createCamera(960, 540);

    // Act
    const lower = projectPoint(camera, { x: 0, y: 0 });
    const higher = projectPoint(camera, { x: 0, y: 1 });

    // Assert
    expect(higher.y).toBeLessThan(lower.y);
  });

  it("projects radii with the same scale used for both axes", () => {
    // Arrange
    const camera = createCamera(960, 540);

    // Act
    const radius = projectRadius(camera, 0.5);

    // Assert
    expect(radius).toBeCloseTo((508 / 9) * 0.5);
  });

  it.each([
    ["zero width", 0, 540],
    ["negative height", 960, -1],
    ["non-finite width", Number.POSITIVE_INFINITY, 540],
    ["non-finite height", 960, Number.NaN],
  ])("rejects %s", (_description, width, height) => {
    // Arrange
    const createInvalidCamera = () => createCamera(width, height);

    // Act
    // Assert
    expect(createInvalidCamera).toThrow("Invalid Canvas viewport");
  });
});

describe("unprojectPoint", () => {
  it("inverts projectPoint for the basin min corner on the design camera", () => {
    // Arrange
    const camera = createCamera(960, 540);
    const world = { x: -6, y: -1 };

    // Act
    const recovered = unprojectPoint(camera, projectPoint(camera, world));

    // Assert
    expect(recovered.x).toBeCloseTo(world.x);
    expect(recovered.y).toBeCloseTo(world.y);
  });

  it("inverts projectPoint for the basin max corner on the design camera", () => {
    // Arrange
    const camera = createCamera(960, 540);
    const world = { x: 6, y: 8 };

    // Act
    const recovered = unprojectPoint(camera, projectPoint(camera, world));

    // Assert
    expect(recovered.x).toBeCloseTo(world.x);
    expect(recovered.y).toBeCloseTo(world.y);
  });

  it("inverts projectPoint for the basin center on the design camera", () => {
    // Arrange
    const camera = createCamera(960, 540);
    const world = { x: 0, y: 3.5 };

    // Act
    const recovered = unprojectPoint(camera, projectPoint(camera, world));

    // Assert
    expect(recovered.x).toBeCloseTo(world.x);
    expect(recovered.y).toBeCloseTo(world.y);
  });

  it("inverts projectPoint for the top-left corner on a narrow camera", () => {
    // Arrange
    const camera = createCamera(320, 540);
    const world = { x: -6, y: 8 };

    // Act
    const recovered = unprojectPoint(camera, projectPoint(camera, world));

    // Assert
    expect(recovered.x).toBeCloseTo(world.x);
    expect(recovered.y).toBeCloseTo(world.y);
  });

  it("inverts projectPoint for the bottom-right corner on a narrow camera", () => {
    // Arrange
    const camera = createCamera(320, 540);
    const world = { x: 6, y: -1 };

    // Act
    const recovered = unprojectPoint(camera, projectPoint(camera, world));

    // Assert
    expect(recovered.x).toBeCloseTo(world.x);
    expect(recovered.y).toBeCloseTo(world.y);
  });

  it("keeps the inverse in CSS pixels without backing-store terms", () => {
    // Arrange
    const cameraSource = readFileSync(
      join(dirname(fileURLToPath(import.meta.url)), "../src/render/camera.ts"),
      "utf8",
    );

    // Act
    const mentionsBackingStore =
      cameraSource.includes("canvas.width") ||
      cameraSource.includes("devicePixelRatio");

    // Assert
    expect(mentionsBackingStore).toBe(false);
    expect(cameraSource).toContain("export function unprojectPoint");
  });

  it("zooms around the fitted center and pans in CSS pixels", () => {
    // Arrange
    const fitted = createCamera(960, 540);
    const zoomed = createCamera(
      960,
      540,
      zoomCameraView(IDENTITY_CAMERA_VIEW, "in"),
    );
    const panned = createCamera(960, 540, {
      zoom: 1,
      panX: 40,
      panY: -25,
    });
    const world = { x: 0, y: 3.5 };

    // Act
    const fittedPoint = projectPoint(fitted, world);
    const zoomedPoint = projectPoint(zoomed, world);
    const pannedPoint = projectPoint(panned, world);

    // Assert
    expect(zoomed.scale).toBeCloseTo(fitted.scale * 1.25);
    expect(zoomedPoint.x).toBeCloseTo(fittedPoint.x);
    expect(zoomedPoint.y).toBeCloseTo(fittedPoint.y);
    expect(pannedPoint.x).toBeCloseTo(fittedPoint.x + 40);
    expect(pannedPoint.y).toBeCloseTo(fittedPoint.y - 25);
  });
});
