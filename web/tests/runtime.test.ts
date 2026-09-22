import { describe, expect, it } from "vitest";

import { maybeSceneById } from "../src/catalog/scenes";
import {
  constructionEntriesForScene,
  formatFailureDetails,
  isReadySceneRoute,
  maybeReadySceneId,
  sceneControlsIdentity,
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

describe("sceneControlsIdentity", () => {
  it("joins Fountain scene id with generation 1", () => {
    // Arrange
    const sceneId = "fountain" as const;
    const generation = 1;

    // Act
    const identity = sceneControlsIdentity(sceneId, generation);

    // Assert
    expect(identity).toBe("fountain:1");
  });

  it("keeps Dam Break generation 0 as a non-empty sceneId:generation string", () => {
    // Arrange
    const sceneId = "dam-break" as const;
    const generation = 0;

    // Act
    const identity = sceneControlsIdentity(sceneId, generation);

    // Assert
    expect(identity).toBe("dam-break:0");
  });

  it("stays truthy for Water Wheel generation 0 so Solid Show can remount", () => {
    // Arrange
    const sceneId = "water-wheel" as const;
    const generation = 0;

    // Act
    const identity = sceneControlsIdentity(sceneId, generation);

    // Assert
    expect(identity === "").toBe(false);
  });

  it("joins Color Mixer scene id with generation 2", () => {
    // Arrange
    const sceneId = "color-mixer" as const;
    const generation = 2;

    // Act
    const identity = sceneControlsIdentity(sceneId, generation);

    // Assert
    expect(identity).toBe("color-mixer:2");
  });
});

describe("formatFailureDetails", () => {
  it("keeps the wrapper message, the cause, and both stacks", () => {
    // Arrange
    const cause = new Error("particle proxy preparation failed: PositionOutOfTagRange");
    const error = new Error("Rust/WASM session failed", { cause });

    // Act
    const details = formatFailureDetails(error);

    // Assert
    expect(details).toContain("Rust/WASM session failed");
    expect(details).toContain("PositionOutOfTagRange");
    expect(details).toContain("User agent:");
  });
});
