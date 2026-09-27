import { describe, expect, it } from "vitest";

import {
  DEFAULT_RENDER_MODE,
  RENDER_MODE_GROUPS,
  RENDER_MODE_STORAGE_KEY,
  isWireframeRenderMode,
  loadRenderMode,
  maybeParseCircleRenderMode,
  maybeParseRenderMode,
  maybeParseSvgRenderMode,
  needsWebglSurface,
  particleSurface,
  persistRenderMode,
  rigidRenderMode,
  svgRenderMode,
  usesParticleStride,
} from "../src/render/mode";

describe("render mode", () => {
  it("parses only allowlisted values", () => {
    // Arrange
    const values = [
      "circle-wireframe",
      "triangle-wireframe",
      "wireframe",
      "solid",
      "soft-blob",
      "contour",
      "shaded-blob",
      "Wireframe",
      "",
      null,
    ];

    // Act
    const parsed = values.map(maybeParseRenderMode);

    // Assert
    expect(parsed).toEqual([
      "circle-wireframe",
      "triangle-wireframe",
      "circle-wireframe",
      "solid",
      "soft-blob",
      "contour",
      "shaded-blob",
      undefined,
      undefined,
      undefined,
    ]);
  });

  it("keeps rigid bodies stroked for both wireframes and exports triangles as polygons", () => {
    // Arrange
    const modes = [
      "circle-wireframe",
      "triangle-wireframe",
      "solid",
      "soft-blob",
      "contour",
      "shaded-blob",
    ] as const;

    // Act
    const exported = modes.map(svgRenderMode);
    const rigid = modes.map(rigidRenderMode);

    // Assert
    expect(exported).toEqual([
      "wireframe",
      "triangle-wireframe",
      "solid",
      "solid",
      "solid",
      "solid",
    ]);
    expect(rigid).toEqual([
      "wireframe",
      "wireframe",
      "solid",
      "solid",
      "solid",
      "solid",
    ]);
    expect(maybeParseCircleRenderMode("soft-blob")).toBeUndefined();
    expect(maybeParseSvgRenderMode("triangle-wireframe")).toBe("triangle-wireframe");
    expect(maybeParseSvgRenderMode("circle-wireframe")).toBeUndefined();
    expect(modes.map(isWireframeRenderMode)).toEqual([
      true,
      true,
      false,
      false,
      false,
      false,
    ]);
  });

  it("draws every particle for surface modes and reserves WebGL for the shaded blob", () => {
    // Arrange
    const surfaceModes = ["soft-blob", "contour", "shaded-blob"] as const;

    // Act
    const surfaces = surfaceModes.map(particleSurface);
    const strides = surfaceModes.map(usesParticleStride);
    const webgl = surfaceModes.map(needsWebglSurface);

    // Assert
    expect(surfaces).toEqual(["metaball", "contour", "webgl"]);
    expect(strides).toEqual([false, false, false]);
    expect(webgl).toEqual([false, false, true]);
    expect(usesParticleStride("circle-wireframe")).toBe(true);
    expect(usesParticleStride("triangle-wireframe")).toBe(true);
    expect(usesParticleStride("solid")).toBe(true);
    expect(particleSurface("circle-wireframe")).toBe("disc");
    expect(particleSurface("triangle-wireframe")).toBe("triangle");
  });

  it("lists wireframe, circle, and surface choices in the controls sheet order", () => {
    // Arrange
    const labels = RENDER_MODE_GROUPS.map((group) => group.label);

    // Act
    const values = RENDER_MODE_GROUPS.flatMap((group) =>
      group.options.map((option) => option.value),
    );

    // Assert
    expect(labels).toEqual(["Wireframe", "Circles", "Surface"]);
    expect(values).toEqual([
      "circle-wireframe",
      "triangle-wireframe",
      "solid",
      "soft-blob",
      "contour",
      "shaded-blob",
    ]);
  });

  it("keeps a stored particle preference", () => {
    // Arrange
    const storage = {
      getItem: () => "wireframe",
      setItem: () => undefined,
    };

    // Act
    const mode = loadRenderMode(() => storage);

    // Assert
    expect(mode).toBe("circle-wireframe");
  });

  it("defaults missing and invalid storage to the shaded blob", () => {
    // Arrange
    const storedValues = [null, "unknown"];

    // Act
    const modes = storedValues.map((storedValue) =>
      loadRenderMode(() => ({
        getItem: () => storedValue,
        setItem: () => undefined,
      })),
    );

    // Assert
    expect(modes).toEqual([DEFAULT_RENDER_MODE, DEFAULT_RENDER_MODE]);
    expect(DEFAULT_RENDER_MODE).toBe("shaded-blob");
  });

  it("contains storage provider and read failures", () => {
    // Arrange
    const providerFailure = () => {
      throw new Error("storage getter denied");
    };
    const readFailure = () => ({
      getItem: () => {
        throw new Error("read denied");
      },
      setItem: () => undefined,
    });

    // Act
    const providerMode = loadRenderMode(providerFailure);
    const readMode = loadRenderMode(readFailure);

    // Assert
    expect(providerMode).toBe("shaded-blob");
    expect(readMode).toBe("shaded-blob");
  });

  it("persists an allowlisted mode under the versioned key", () => {
    // Arrange
    const writes: Array<readonly [string, string]> = [];
    const storage = {
      getItem: () => null,
      setItem: (key: string, value: string) => {
        writes.push([key, value]);
      },
    };

    // Act
    persistRenderMode(() => storage, "solid");

    // Assert
    expect(writes).toEqual([[RENDER_MODE_STORAGE_KEY, "solid"]]);
  });

  it("contains storage write failures", () => {
    // Arrange
    const failedWrite = () =>
      persistRenderMode(
        () => ({
          getItem: () => null,
          setItem: () => {
            throw new Error("write denied");
          },
        }),
        "circle-wireframe",
      );

    // Act / Assert
    expect(failedWrite).not.toThrow();
  });
});
