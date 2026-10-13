import { describe, expect, it } from "vitest";

import {
  SOUND_STORAGE_KEY,
  createSoundPreferenceMemory,
  loadSoundOn,
  persistSoundOn,
  type SoundStorageProvider,
} from "../src/player/sound-preference";

type FakeStorage = Pick<Storage, "getItem" | "setItem"> & {
  readonly values: Map<string, string>;
};

function memoryStorage(initial: Readonly<Record<string, string>> = {}): FakeStorage {
  const values = new Map<string, string>(Object.entries(initial));
  return {
    values,
    getItem: (key) => values.get(key) ?? null,
    setItem: (key, value) => {
      values.set(key, value);
    },
  };
}

function throwingSetStorage(): Pick<Storage, "getItem" | "setItem"> {
  return {
    getItem: () => null,
    setItem: () => {
      throw new Error("quota exceeded");
    },
  };
}

function throwingGetStorage(): Pick<Storage, "getItem" | "setItem"> {
  return {
    getItem: () => {
      throw new Error("storage unavailable");
    },
    setItem: () => undefined,
  };
}

const throwingProvider: SoundStorageProvider = () => {
  throw new Error("no storage");
};

describe("loadSoundOn", () => {
  it("reads the exact on token as on", () => {
    // Arrange
    const storage = memoryStorage({ [SOUND_STORAGE_KEY]: "on" });

    // Act
    const on = loadSoundOn(() => storage);

    // Assert
    expect(on).toBe(true);
  });

  it("reads a missing value as off", () => {
    // Arrange
    const storage = memoryStorage();

    // Act
    const on = loadSoundOn(() => storage);

    // Assert
    expect(on).toBe(false);
  });

  it.each(["off", "On", " on", "on ", "true", "1", ""])("reads %j as off", (stored) => {
    // Arrange
    const storage = memoryStorage({ [SOUND_STORAGE_KEY]: stored });

    // Act
    const on = loadSoundOn(() => storage);

    // Assert
    expect(on).toBe(false);
  });

  it("reads a throwing getItem as off", () => {
    // Arrange
    const storage = throwingGetStorage();

    // Act
    const on = loadSoundOn(() => storage);

    // Assert
    expect(on).toBe(false);
  });

  it("reads a throwing storage provider as off", () => {
    // Act
    const on = loadSoundOn(throwingProvider);

    // Assert
    expect(on).toBe(false);
  });
});

describe("persistSoundOn", () => {
  it("stores on under the versioned key", () => {
    // Arrange
    const storage = memoryStorage();

    // Act
    persistSoundOn(() => storage, true);

    // Assert
    expect(storage.values.get("liquidfun.puzzle-sound.v1")).toBe("on");
  });

  it("stores off under the versioned key", () => {
    // Arrange
    const storage = memoryStorage({ [SOUND_STORAGE_KEY]: "on" });

    // Act
    persistSoundOn(() => storage, false);

    // Assert
    expect(storage.values.get("liquidfun.puzzle-sound.v1")).toBe("off");
  });

  it("does not throw when setItem throws", () => {
    // Arrange
    const storage = throwingSetStorage();

    // Act
    const act = () => persistSoundOn(() => storage, true);

    // Assert
    expect(act).not.toThrow();
  });

  it("does not throw when the storage provider throws", () => {
    // Act
    const act = () => persistSoundOn(throwingProvider, true);

    // Assert
    expect(act).not.toThrow();
  });
});

describe("createSoundPreferenceMemory", () => {
  it("does not touch storage until on() is first read", () => {
    // Arrange
    let reads = 0;
    const provider: SoundStorageProvider = () => {
      reads += 1;
      return memoryStorage();
    };

    // Act
    createSoundPreferenceMemory(provider);

    // Assert
    expect(reads).toBe(0);
  });

  it("reads storage once and caches the value", () => {
    // Arrange
    let reads = 0;
    const storage = memoryStorage({ [SOUND_STORAGE_KEY]: "on" });
    const memory = createSoundPreferenceMemory(() => {
      reads += 1;
      return storage;
    });

    // Act
    const first = memory.on();
    const second = memory.on();

    // Assert
    expect([first, second, reads]).toEqual([true, true, 1]);
  });

  it("persists a choice to storage", () => {
    // Arrange
    const storage = memoryStorage();
    const memory = createSoundPreferenceMemory(() => storage);

    // Act
    memory.set(true);

    // Assert
    expect(storage.values.get(SOUND_STORAGE_KEY)).toBe("on");
  });

  it("holds on in memory when setItem throws", () => {
    // Arrange
    const memory = createSoundPreferenceMemory(throwingSetStorage);

    // Act
    memory.set(true);

    // Assert
    expect(memory.on()).toBe(true);
  });

  it("holds off in memory after turning sound off", () => {
    // Arrange
    const storage = memoryStorage({ [SOUND_STORAGE_KEY]: "on" });
    const memory = createSoundPreferenceMemory(() => storage);

    // Act
    memory.set(false);

    // Assert
    expect(memory.on()).toBe(false);
  });
});
