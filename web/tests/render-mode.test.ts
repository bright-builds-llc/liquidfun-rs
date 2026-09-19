import { describe, expect, it } from "vitest";

import {
  DEFAULT_RENDER_MODE,
  RENDER_MODE_STORAGE_KEY,
  loadRenderMode,
  maybeParseRenderMode,
  persistRenderMode,
} from "../src/render/mode";

describe("render mode", () => {
  it("parses only allowlisted values", () => {
    // Arrange
    const values = ["wireframe", "solid", "Wireframe", "", null];

    // Act
    const parsed = values.map(maybeParseRenderMode);

    // Assert
    expect(parsed).toEqual([
      "wireframe",
      "solid",
      undefined,
      undefined,
      undefined,
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
