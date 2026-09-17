import { describe, expect, it } from "vitest";

import { isStaleGeneration, nextGeneration } from "../src/player/generation";

describe("nextGeneration", () => {
  it("increments zero to one", () => {
    // Arrange
    const current = 0;

    // Act
    const next = nextGeneration(current);

    // Assert
    expect(next).toBe(1);
  });

  it("increments seven to eight", () => {
    // Arrange
    const current = 7;

    // Act
    const next = nextGeneration(current);

    // Assert
    expect(next).toBe(8);
  });
});

describe("isStaleGeneration", () => {
  it("keeps a matching generation current", () => {
    // Arrange
    const started = 3;
    const current = 3;

    // Act
    const stale = isStaleGeneration(started, current);

    // Assert
    expect(stale).toBe(false);
  });

  it("marks a later current generation stale", () => {
    // Arrange
    const started = 3;
    const current = 4;

    // Act
    const stale = isStaleGeneration(started, current);

    // Assert
    expect(stale).toBe(true);
  });

  it("requires a stale async load to dispose immediately", () => {
    // Arrange
    let current = 0;
    const started = nextGeneration(current);
    current = started;
    current = nextGeneration(current);
    let disposed = false;

    // Act
    if (isStaleGeneration(started, current)) {
      disposed = true;
    }

    // Assert
    expect(isStaleGeneration(started, current)).toBe(true);
    expect(disposed).toBe(true);
  });
});
