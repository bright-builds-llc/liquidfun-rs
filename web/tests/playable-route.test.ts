import { describe, expect, it } from "vitest";

import { maybeParseSceneRoute, normalizeSceneRoute } from "../src/routing/hash";
import { maybeParsePlayableRoute, normalizePlayableRoute } from "../src/routing/playable-route";

describe("maybeParsePlayableRoute", () => {
  it("parses an allowlisted level hash as a puzzle", () => {
    // Arrange
    const hash = "#/puzzle/first-pour";

    // Act
    const route = maybeParsePlayableRoute(hash);

    // Assert
    expect(route).toEqual({ kind: "puzzle", id: "first-pour" });
  });

  it("parses an unlisted level id as an unknown puzzle", () => {
    // Arrange
    const hash = "#/puzzle/nope";

    // Act
    const route = maybeParsePlayableRoute(hash);

    // Assert
    expect(route).toEqual({ kind: "unknown-puzzle", maybeRaw: "nope" });
  });

  it("parses extra segments after a level id as an unknown puzzle", () => {
    // Arrange
    const hash = "#/puzzle/first-pour/extra";

    // Act
    const route = maybeParsePlayableRoute(hash);

    // Assert
    expect(route).toEqual({ kind: "unknown-puzzle", maybeRaw: "first-pour/extra" });
  });

  it("parses a puzzle hash without a level id as an unknown puzzle", () => {
    // Arrange
    const hashes = ["#/puzzle", "#/puzzle/"];

    // Act
    const routes = hashes.map((hash) => maybeParsePlayableRoute(hash));

    // Assert
    expect(routes).toEqual([
      { kind: "unknown-puzzle", maybeRaw: "" },
      { kind: "unknown-puzzle", maybeRaw: "" },
    ]);
  });

  it("keeps markup in an unknown level id verbatim", () => {
    // Arrange
    const hash = "#/puzzle/<img src=x onerror=alert(1)>";

    // Act
    const route = maybeParsePlayableRoute(hash);

    // Assert
    expect(route).toEqual({
      kind: "unknown-puzzle",
      maybeRaw: "<img src=x onerror=alert(1)>",
    });
  });

  it("parses the plural puzzles hash as the level grid", () => {
    // Arrange
    const hash = "#/puzzles";

    // Act
    const route = maybeParsePlayableRoute(hash);

    // Assert
    expect(route).toEqual({ kind: "puzzles" });
  });

  it("parses the puzzles hash with a trailing slash as the level grid", () => {
    // Arrange
    const hash = "#/puzzles/";

    // Act
    const route = maybeParsePlayableRoute(hash);

    // Assert
    expect(route).toEqual({ kind: "puzzles" });
  });

  it("leaves an extra segment under puzzles as an unknown scene route", () => {
    // Arrange
    const hash = "#/puzzles/first-pour";

    // Act
    const route = maybeParsePlayableRoute(hash);

    // Assert
    expect(route).toEqual({ kind: "unknown", maybeRaw: "puzzles/first-pour" });
  });

  it("does not resolve a level id under the scene prefix", () => {
    // Arrange
    const hash = "#/scene/first-pour";

    // Act
    const route = maybeParsePlayableRoute(hash);

    // Assert
    expect(route).toEqual({ kind: "unknown", maybeRaw: "first-pour" });
  });

  it("returns the scene parser result for every non-puzzle hash", () => {
    // Arrange
    const hashes = [
      "",
      "#/",
      "#/scene/dam-break",
      "#/scene/not-a-scene",
      "https://evil.example/#/puzzle/first-pour",
    ];

    // Act
    const routes = hashes.map((hash) => maybeParsePlayableRoute(hash));

    // Assert
    expect(routes).toEqual(hashes.map((hash) => maybeParseSceneRoute(hash)));
  });
});

describe("normalizePlayableRoute", () => {
  it("normalizes an empty hash exactly like the scene parser", () => {
    // Arrange
    const hash = "";

    // Act
    const normalized = normalizePlayableRoute(hash);

    // Assert
    expect(normalized).toEqual(normalizeSceneRoute(hash));
  });

  it("keeps a level hash without a replacement", () => {
    // Arrange
    const hash = "#/puzzle/first-pour";

    // Act
    const normalized = normalizePlayableRoute(hash);

    // Assert
    expect(normalized).toEqual({
      route: { kind: "puzzle", id: "first-pour" },
      maybeReplacementHash: undefined,
    });
  });

  it("keeps the level grid hash without a replacement", () => {
    // Arrange
    const hash = "#/puzzles";

    // Act
    const normalized = normalizePlayableRoute(hash);

    // Assert
    expect(normalized).toEqual({ route: { kind: "puzzles" }, maybeReplacementHash: undefined });
  });

  it("keeps an unknown level hash without a replacement", () => {
    // Arrange
    const hash = "#/puzzle/nope";

    // Act
    const normalized = normalizePlayableRoute(hash);

    // Assert
    expect(normalized).toEqual({
      route: { kind: "unknown-puzzle", maybeRaw: "nope" },
      maybeReplacementHash: undefined,
    });
  });
});
