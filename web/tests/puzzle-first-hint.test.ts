import { describe, expect, it } from "vitest";

import {
  FIRST_HINT_STORAGE_KEY,
  createFirstHintMemory,
  loadHintDismissed,
  persistHintDismissed,
  type HintStorageProvider,
} from "../src/player/first-hint-preference";

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

const throwingProvider: HintStorageProvider = () => {
  throw new Error("no localStorage");
};

describe("loadHintDismissed", () => {
  it("reads the exact dismissed token as dismissed", () => {
    // Arrange
    const storage = memoryStorage({ [FIRST_HINT_STORAGE_KEY]: "dismissed" });

    // Act
    const dismissed = loadHintDismissed(() => storage);

    // Assert
    expect(dismissed).toBe(true);
  });

  it("reads a missing value as not dismissed", () => {
    // Arrange
    const storage = memoryStorage();

    // Act
    const dismissed = loadHintDismissed(() => storage);

    // Assert
    expect(dismissed).toBe(false);
  });

  it("reads any other stored value as not dismissed", () => {
    // Arrange
    const storage = memoryStorage({ [FIRST_HINT_STORAGE_KEY]: "yes" });

    // Act
    const dismissed = loadHintDismissed(() => storage);

    // Assert
    expect(dismissed).toBe(false);
  });

  it("reads a throwing storage provider as not dismissed", () => {
    // Act
    const dismissed = loadHintDismissed(throwingProvider);

    // Assert
    expect(dismissed).toBe(false);
  });

  it("reads a throwing getItem as not dismissed", () => {
    // Arrange
    const storage = throwingGetStorage();

    // Act
    const dismissed = loadHintDismissed(() => storage);

    // Assert
    expect(dismissed).toBe(false);
  });
});

describe("persistHintDismissed", () => {
  it("stores the dismissed token under the versioned key", () => {
    // Arrange
    const storage = memoryStorage();

    // Act
    persistHintDismissed(() => storage);

    // Assert
    expect(storage.values.get("liquidfun.puzzle-first-hint.v1")).toBe("dismissed");
  });

  it("does not throw when the storage provider throws", () => {
    // Act
    const act = () => persistHintDismissed(throwingProvider);

    // Assert
    expect(act).not.toThrow();
  });

  it("does not throw when setItem throws", () => {
    // Arrange
    const storage = throwingSetStorage();

    // Act
    const act = () => persistHintDismissed(() => storage);

    // Assert
    expect(act).not.toThrow();
  });
});

describe("createFirstHintMemory", () => {
  it("starts not dismissed with empty storage", () => {
    // Arrange
    const storage = memoryStorage();

    // Act
    const memory = createFirstHintMemory(() => storage);

    // Assert
    expect(memory.dismissed()).toBe(false);
  });

  it("starts dismissed when storage already holds the token", () => {
    // Arrange
    const storage = memoryStorage({ [FIRST_HINT_STORAGE_KEY]: "dismissed" });

    // Act
    const memory = createFirstHintMemory(() => storage);

    // Assert
    expect(memory.dismissed()).toBe(true);
  });

  it("persists a dismissal to storage", () => {
    // Arrange
    const storage = memoryStorage();
    const memory = createFirstHintMemory(() => storage);

    // Act
    memory.dismiss();

    // Assert
    expect(storage.values.get(FIRST_HINT_STORAGE_KEY)).toBe("dismissed");
  });

  it("holds a dismissal in memory when setItem throws", () => {
    // Arrange
    const memory = createFirstHintMemory(throwingSetStorage);

    // Act
    memory.dismiss();

    // Assert
    expect(memory.dismissed()).toBe(true);
  });

  it("holds a dismissal in memory when the storage provider throws", () => {
    // Arrange
    const memory = createFirstHintMemory(throwingProvider);

    // Act
    memory.dismiss();

    // Assert
    expect(memory.dismissed()).toBe(true);
  });

  it("does not touch storage until dismissed() is first read", () => {
    // Arrange
    let reads = 0;
    const provider: HintStorageProvider = () => {
      reads += 1;
      return memoryStorage();
    };

    // Act
    createFirstHintMemory(provider);

    // Assert
    expect(reads).toBe(0);
  });
});
