import { describe, expect, it } from "vitest";

import { maybeSceneById } from "../src/catalog/scenes";
import {
  constructionEntriesForScene,
  isReadySceneRoute,
  maybeReadySceneId,
  sceneTitleForId,
  titleForRoute,
} from "../src/player/runtime";

describe("isReadySceneRoute", () => {
  it("is true for every approved ready id", () => {
    // Arrange
    const routes = [
      { kind: "scene" as const, id: "dam-break" as const },
      { kind: "scene" as const, id: "fountain" as const },
      { kind: "empty" as const },
      { kind: "unknown" as const, maybeRaw: "nope" },
    ];

    // Act
    const flags = routes.map((route) => isReadySceneRoute(route));

    // Assert
    expect(flags).toEqual([true, true, false, false]);
  });
});

describe("maybeReadySceneId", () => {
  it("returns the id only for ready scene routes", () => {
    // Arrange
    const readyRoute = { kind: "scene" as const, id: "water-wheel" as const };

    // Act
    const maybeId = maybeReadySceneId(readyRoute);
    const maybeEmpty = maybeReadySceneId({ kind: "empty" });

    // Assert
    expect(maybeId).toBe("water-wheel");
    expect(maybeEmpty).toBeUndefined();
  });
});

describe("titleForRoute", () => {
  it("uses the scene title for ready hashes", () => {
    // Arrange
    const route = { kind: "scene" as const, id: "color-mixer" as const };

    // Act
    const title = titleForRoute(route);

    // Assert
    expect(title).toBe("Color Mixer · liquidfun-rs playground");
    expect(sceneTitleForId("color-mixer")).toBe("Color Mixer");
  });
});

describe("constructionEntriesForScene", () => {
  it("keeps only recreating presets that were applied", () => {
    // Arrange
    const maybeScene = maybeSceneById("dam-break");
    expect(maybeScene).toBeDefined();
    if (maybeScene === undefined) {
      return;
    }

    // Act
    const entries = constructionEntriesForScene(maybeScene, {
      "water-amount": "large",
      gravity: "low",
      "drop-obstacle": "ignored",
    });

    // Assert
    expect(entries).toEqual([
      { name: "water-amount", value: "large" },
      { name: "gravity", value: "low" },
    ]);
  });

  it("emits no applyControl rows for an empty Dam Break bag", () => {
    // Arrange
    const maybeScene = maybeSceneById("dam-break");
    expect(maybeScene).toBeDefined();
    if (maybeScene === undefined) {
      return;
    }

    // Act
    const entries = constructionEntriesForScene(maybeScene, {});

    // Assert
    expect(entries).toEqual([]);
  });

  it("emits no applyControl rows for an empty Color Mixer bag", () => {
    // Arrange
    const maybeScene = maybeSceneById("color-mixer");
    expect(maybeScene).toBeDefined();
    if (maybeScene === undefined) {
      return;
    }

    // Act
    const entries = constructionEntriesForScene(maybeScene, {});

    // Assert
    expect(entries).toEqual([]);
  });
});
