import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { LIQUID_TUMBLER_VIEW_BOUNDS } from "../src/catalog/scenes";
import {
  IDENTITY_CAMERA_VIEW,
  WORLD_BOUNDS,
  createCamera,
} from "../src/render/camera";
import {
  SCALE_LEGEND_MAX_WIDTH_PX,
  scaleLegend,
} from "../src/render/scale-legend";

const WEB_SRC = join(dirname(fileURLToPath(import.meta.url)), "../src");

function legendForMaxMeters(maxMeters: number) {
  return scaleLegend(SCALE_LEGEND_MAX_WIDTH_PX / maxMeters);
}

describe("scaleLegend", () => {
  it("uses a full-width 1 m bar when one meter fills the maximum width", () => {
    // Arrange
    const pixelsPerMeter = SCALE_LEGEND_MAX_WIDTH_PX;

    // Act
    const mark = scaleLegend(pixelsPerMeter);

    // Assert
    expect(mark).toEqual({
      meters: 1,
      widthPx: SCALE_LEGEND_MAX_WIDTH_PX,
      label: "1 m",
      accessibleLabel: "1 meter",
    });
  });

  it("snaps to the largest 1, 2, or 5 step that fits", () => {
    // Arrange
    const cases = [
      { maxMeters: 1.9, label: "1 m" },
      { maxMeters: 2, label: "2 m" },
      { maxMeters: 4.9, label: "2 m" },
      { maxMeters: 5, label: "5 m" },
      { maxMeters: 9.9, label: "5 m" },
      { maxMeters: 10, label: "10 m" },
    ];

    // Act
    const labels = cases.map((entry) => legendForMaxMeters(entry.maxMeters)?.label);

    // Assert
    expect(labels).toEqual(cases.map((entry) => entry.label));
  });

  it("labels sub-meter lengths in centimeters and millimeters", () => {
    // Arrange
    const cases = [
      { maxMeters: 0.05, label: "5 cm", accessibleLabel: "5 centimeters" },
      { maxMeters: 0.02, label: "2 cm", accessibleLabel: "2 centimeters" },
      { maxMeters: 0.01, label: "1 cm", accessibleLabel: "1 centimeter" },
      { maxMeters: 0.009, label: "5 mm", accessibleLabel: "5 millimeters" },
      { maxMeters: 0.002, label: "2 mm", accessibleLabel: "2 millimeters" },
      { maxMeters: 0.001, label: "1 mm", accessibleLabel: "1 millimeter" },
    ];

    // Act
    const marks = cases.map((entry) => legendForMaxMeters(entry.maxMeters));

    // Assert
    expect(marks.map((mark) => mark?.label)).toEqual(cases.map((entry) => entry.label));
    expect(marks.map((mark) => mark?.accessibleLabel)).toEqual(
      cases.map((entry) => entry.accessibleLabel),
    );
  });

  it("labels long lengths in kilometers", () => {
    // Arrange
    const oneKilometer = legendForMaxMeters(1000);
    const twoKilometers = legendForMaxMeters(2000);
    const justUnderAKilometer = legendForMaxMeters(999);

    // Act
    const labels = [
      oneKilometer?.label,
      twoKilometers?.label,
      justUnderAKilometer?.label,
    ];

    // Assert
    expect(labels).toEqual(["1 km", "2 km", "500 m"]);
    expect(oneKilometer?.accessibleLabel).toBe("1 kilometer");
    expect(twoKilometers?.accessibleLabel).toBe("2 kilometers");
  });

  it("keeps the bar inside the maximum width", () => {
    // Arrange
    const pixelsPerMeter = 37;

    // Act
    const mark = scaleLegend(pixelsPerMeter, 80);

    // Assert
    expect(mark).toBeDefined();
    expect(mark?.widthPx).toBeGreaterThan(0);
    expect(mark?.widthPx).toBeLessThanOrEqual(80);
    expect(mark?.meters).toBeLessThanOrEqual(80 / pixelsPerMeter + 1e-9);
  });

  it("rejects a scale that cannot be drawn", () => {
    // Arrange
    const invalidScales = [0, -4, Number.NaN, Number.POSITIVE_INFINITY];

    // Act
    const marks = invalidScales.map((pixelsPerMeter) => scaleLegend(pixelsPerMeter));

    // Assert
    expect(marks).toEqual([undefined, undefined, undefined, undefined]);
  });

  it("shortens the labeled distance as the camera zooms in", () => {
    // Arrange
    const wide = createCamera(960, 540, IDENTITY_CAMERA_VIEW, WORLD_BOUNDS);
    const close = createCamera(
      960,
      540,
      { ...IDENTITY_CAMERA_VIEW, zoom: 4 },
      WORLD_BOUNDS,
    );

    // Act
    const wideMark = scaleLegend(wide.scale);
    const closeMark = scaleLegend(close.scale);

    // Assert
    expect(close.scale).toBeGreaterThan(wide.scale);
    expect(closeMark?.meters).toBeLessThan(wideMark?.meters ?? 0);
  });

  it("keeps the same bar when the camera pans", () => {
    // Arrange
    const rested = createCamera(960, 540, IDENTITY_CAMERA_VIEW, WORLD_BOUNDS);
    const panned = createCamera(
      960,
      540,
      { zoom: 1, panX: 48, panY: -16 },
      WORLD_BOUNDS,
    );

    // Act
    const restedMark = scaleLegend(rested.scale);
    const pannedMark = scaleLegend(panned.scale);

    // Assert
    expect(pannedMark).toEqual(restedMark);
  });

  it("reads centimeters on the liquid tumbler frame", () => {
    // Arrange
    const camera = createCamera(
      960,
      540,
      IDENTITY_CAMERA_VIEW,
      LIQUID_TUMBLER_VIEW_BOUNDS,
    );

    // Act
    const mark = scaleLegend(camera.scale);

    // Assert
    expect(mark?.label.endsWith("cm") || mark?.label.endsWith("mm")).toBe(true);
  });
});

describe("playground scale legend chrome", () => {
  it("renders the shared legend in the canvas hub and the desktop viewport", () => {
    // Arrange
    const playerPanel = readFileSync(join(WEB_SRC, "components/PlayerPanel.tsx"), "utf8");
    const canvasStage = readFileSync(join(WEB_SRC, "components/CanvasStage.tsx"), "utf8");

    // Act
    const canvasHubHasLegend = canvasStage.includes("<ScaleLegend");
    const desktopHasLegend = playerPanel.includes("<ScaleLegend");

    // Assert
    expect(canvasHubHasLegend).toBe(true);
    expect(desktopHasLegend).toBe(true);
  });
});
