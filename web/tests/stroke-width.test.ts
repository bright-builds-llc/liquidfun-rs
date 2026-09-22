import { describe, expect, it } from "vitest";

import {
  DEFAULT_WIREFRAME_STROKE_WIDTH,
  WIREFRAME_STROKE_WIDTH_STORAGE_KEY,
  loadWireframeStrokeWidth,
  maybeParseWireframeStrokeWidth,
  persistWireframeStrokeWidth,
} from "../src/render/stroke-width";

describe("wireframe stroke width", () => {
  it("parses only tenth steps from 0.1 through 1.5", () => {
    // Arrange
    const values = [
      "0.1",
      "0.3",
      "1",
      "1.50",
      "0.25",
      "0",
      "2",
      "nope",
      null,
    ];

    // Act
    const parsed = values.map(maybeParseWireframeStrokeWidth);

    // Assert
    expect(parsed).toEqual([
      0.1,
      0.3,
      1,
      1.5,
      undefined,
      undefined,
      undefined,
      undefined,
      undefined,
    ]);
  });

  it("defaults missing and invalid storage to 0.3", () => {
    // Arrange
    const storedValues = [null, "2"];

    // Act
    const widths = storedValues.map((storedValue) =>
      loadWireframeStrokeWidth(() => ({
        getItem: () => storedValue,
        setItem: () => undefined,
      })),
    );

    // Assert
    expect(widths).toEqual([
      DEFAULT_WIREFRAME_STROKE_WIDTH,
      DEFAULT_WIREFRAME_STROKE_WIDTH,
    ]);
    expect(DEFAULT_WIREFRAME_STROKE_WIDTH).toBe(0.3);
  });

  it("persists an allowlisted width and ignores values off the scale", () => {
    // Arrange
    const stored = new Map<string, string>();
    const storage = {
      getItem: (key: string) => stored.get(key) ?? null,
      setItem: (key: string, value: string) => {
        stored.set(key, value);
      },
    };

    // Act
    persistWireframeStrokeWidth(() => storage, 0.3);
    persistWireframeStrokeWidth(() => storage, 3);

    // Assert
    expect(stored.get(WIREFRAME_STROKE_WIDTH_STORAGE_KEY)).toBe("0.3");
    expect(loadWireframeStrokeWidth(() => storage)).toBe(0.3);
  });
});
