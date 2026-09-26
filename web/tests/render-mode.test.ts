import { describe, expect, it } from "vitest";

import {
  DEFAULT_RENDER_MODE,
  RENDER_MODE_GROUPS,
  RENDER_MODE_STORAGE_KEY,
  circleExportMode,
  loadRenderMode,
  maybeParseCircleRenderMode,
  maybeParseRenderMode,
  needsWebglSurface,
  particleSurface,
  persistRenderMode,
  rigidRenderMode,
  usesParticleStride,
} from "../src/render/mode";

describe("render mode", () => {
  it("parses only allowlisted values", () => {
    // Arrange
    const values = [
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
      "wireframe",
      "solid",
      "soft-blob",
      "contour",
      "shaded-blob",
      undefined,
      undefined,
      undefined,
    ]);
  });

  it("keeps circle export and rigid bodies on the two circle modes", () => {
    // Arrange
    const modes = [
      "wireframe",
      "solid",
      "soft-blob",
      "contour",
      "shaded-blob",
    ] as const;

    // Act
    const exported = modes.map(circleExportMode);
    const rigid = modes.map(rigidRenderMode);

    // Assert
    expect(exported).toEqual(["wireframe", "solid", "solid", "solid", "solid"]);
    expect(rigid).toEqual(exported);
    expect(maybeParseCircleRenderMode("soft-blob")).toBeUndefined();
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
    expect(usesParticleStride("wireframe")).toBe(true);
    expect(usesParticleStride("solid")).toBe(true);
  });

  it("lists circle and surface choices in the controls sheet order", () => {
    // Arrange
    const labels = RENDER_MODE_GROUPS.map((group) => group.label);

    // Act
    const values = RENDER_MODE_GROUPS.flatMap((group) =>
      group.options.map((option) => option.value),
    );

    // Assert
    expect(labels).toEqual(["Circles", "Surface"]);
    expect(values).toEqual([
      "wireframe",
      "solid",
      "soft-blob",
      "contour",
      "shaded-blob",
    ]);
  });

  it("defaults missing and invalid storage to wireframe", () => {
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
    expect(DEFAULT_RENDER_MODE).toBe("wireframe");
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
    expect(providerMode).toBe("wireframe");
    expect(readMode).toBe("wireframe");
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
        "wireframe",
      );

    // Act / Assert
    expect(failedWrite).not.toThrow();
  });
});
