import { describe, expect, it } from "vitest";

import {
  HUD_SLIDERS_STORAGE_KEY,
  hudSlidersToggleLabel,
  loadHudSlidersVisibility,
  persistHudSlidersVisibility,
} from "../src/player/hud-sliders-preference";

function memoryStorage(): Pick<Storage, "getItem" | "setItem"> {
  const values = new Map<string, string>();
  return {
    getItem: (key) => values.get(key) ?? null,
    setItem: (key, value) => {
      values.set(key, value);
    },
  };
}

function throwingStorage(): Pick<Storage, "getItem" | "setItem"> {
  return {
    getItem: () => {
      throw new Error("storage unavailable");
    },
    setItem: () => {
      throw new Error("storage unavailable");
    },
  };
}

describe("loadHudSlidersVisibility", () => {
  it("defaults to shown when storage is empty", () => {
    // Arrange
    const storage = memoryStorage();

    // Act
    const visibility = loadHudSlidersVisibility(() => storage);

    // Assert
    expect(visibility).toBe("shown");
  });

  it("round-trips a persisted hidden choice", () => {
    // Arrange
    const storage = memoryStorage();
    persistHudSlidersVisibility(() => storage, "hidden");

    // Act
    const visibility = loadHudSlidersVisibility(() => storage);

    // Assert
    expect(visibility).toBe("hidden");
  });

  it("reads a throwing storage as shown", () => {
    // Arrange
    const storage = throwingStorage();

    // Act
    const visibility = loadHudSlidersVisibility(() => storage);

    // Assert
    expect(visibility).toBe("shown");
  });

  it("reads a throwing storage provider as shown", () => {
    // Arrange
    const provider = () => {
      throw new Error("no localStorage");
    };

    // Act
    const visibility = loadHudSlidersVisibility(provider);

    // Assert
    expect(visibility).toBe("shown");
  });

  it("reads an unknown stored value as shown", () => {
    // Arrange
    const storage = memoryStorage();
    storage.setItem(HUD_SLIDERS_STORAGE_KEY, "collapsed");

    // Act
    const visibility = loadHudSlidersVisibility(() => storage);

    // Assert
    expect(visibility).toBe("shown");
  });
});

describe("persistHudSlidersVisibility", () => {
  it("does not throw when storage rejects the write", () => {
    // Arrange
    const storage = throwingStorage();

    // Act
    const persist = () => persistHudSlidersVisibility(() => storage, "hidden");

    // Assert
    expect(persist).not.toThrow();
  });

  it("stores the choice under the versioned key", () => {
    // Arrange
    const storage = memoryStorage();

    // Act
    persistHudSlidersVisibility(() => storage, "hidden");

    // Assert
    expect(storage.getItem("liquidfun.hud-sliders.v1")).toBe("hidden");
  });
});

describe("hudSlidersToggleLabel", () => {
  it("offers to hide the sliders while they are shown", () => {
    // Arrange
    const visibility = "shown";

    // Act
    const label = hudSlidersToggleLabel(visibility);

    // Assert
    expect(label).toBe("Hide sliders");
  });

  it("offers to show the sliders while they are hidden", () => {
    // Arrange
    const visibility = "hidden";

    // Act
    const label = hudSlidersToggleLabel(visibility);

    // Assert
    expect(label).toBe("Show sliders");
  });
});
