import { describe, expect, it } from "vitest";

import { maybeParseSceneRoute, normalizeSceneRoute } from "../src/routing/hash";

describe("maybeParseSceneRoute", () => {
  it("parses the canonical Dam Break hash as a scene", () => {
    // Arrange
    const hash = "#/scene/dam-break";

    // Act
    const route = maybeParseSceneRoute(hash);

    // Assert
    expect(route).toEqual({ kind: "scene", id: "dam-break" });
  });

  it("parses a hashless Dam Break path as a scene", () => {
    // Arrange
    const path = "/scene/dam-break";

    // Act
    const route = maybeParseSceneRoute(path);

    // Assert
    expect(route).toEqual({ kind: "scene", id: "dam-break" });
  });

  it("parses empty scene identifiers as empty", () => {
    // Arrange
    const hashes = ["", "#", "#/", "#/scene", "#/scene/"];

    // Act
    const routes = hashes.map((hash) => maybeParseSceneRoute(hash));

    // Assert
    expect(routes).toEqual([
      { kind: "empty" },
      { kind: "empty" },
      { kind: "empty" },
      { kind: "empty" },
      { kind: "empty" },
    ]);
  });

  it("parses an unknown scene token without coercing case", () => {
    // Arrange
    const unknownHash = "#/scene/not-a-scene";
    const mixedCaseHash = "#/scene/Dam-Break";

    // Act
    const unknownRoute = maybeParseSceneRoute(unknownHash);
    const mixedCaseRoute = maybeParseSceneRoute(mixedCaseHash);

    // Assert
    expect(unknownRoute).toEqual({
      kind: "unknown",
      maybeRaw: "not-a-scene",
    });
    expect(mixedCaseRoute).toEqual({
      kind: "unknown",
      maybeRaw: "Dam-Break",
    });
  });

  it("parses a known not-ready id as a scene", () => {
    // Arrange
    const hash = "#/scene/fountain";

    // Act
    const route = maybeParseSceneRoute(hash);

    // Assert
    expect(route).toEqual({ kind: "scene", id: "fountain" });
  });

  it("rejects extra segments, other paths, and full URLs", () => {
    // Arrange
    const extraHash = "#/scene/dam-break/extra";
    const otherHash = "#/foo";
    const fullUrl = "https://evil.example/#/scene/dam-break";

    // Act
    const extraRoute = maybeParseSceneRoute(extraHash);
    const otherRoute = maybeParseSceneRoute(otherHash);
    const fullUrlRoute = maybeParseSceneRoute(fullUrl);

    // Assert
    expect(extraRoute.kind).toBe("unknown");
    expect(otherRoute.kind).toBe("unknown");
    expect(fullUrlRoute.kind).toBe("unknown");
  });
});

describe("normalizeSceneRoute", () => {
  it("normalizes every empty hash to Wave Machine with replacement", () => {
    // Arrange
    const emptyHashes = ["", "#", "#/", "#/scene", "#/scene/"];

    // Act
    const normalized = emptyHashes.map(normalizeSceneRoute);

    // Assert
    expect(normalized).toEqual(
      emptyHashes.map(() => ({
        route: { kind: "scene", id: "wave-machine" },
        maybeReplacementHash: "#/scene/wave-machine",
      })),
    );
  });

  it("preserves known and unknown routes without replacement", () => {
    // Arrange / Act
    const known = normalizeSceneRoute("#/scene/fountain");
    const unknown = normalizeSceneRoute("#/scene/nope");

    // Assert
    expect(known).toEqual({
      route: { kind: "scene", id: "fountain" },
      maybeReplacementHash: undefined,
    });
    expect(unknown).toEqual({
      route: { kind: "unknown", maybeRaw: "nope" },
      maybeReplacementHash: undefined,
    });
  });
});
