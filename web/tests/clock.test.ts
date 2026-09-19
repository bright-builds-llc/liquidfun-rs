import { describe, expect, it } from "vitest";

import {
  MAX_DELTA_SECONDS,
  MAX_STEPS_PER_FRAME,
  STEP_SECONDS,
  acceptedStepCount,
  accumulateStepTime,
} from "../src/physics/clock";

describe("clock constants", () => {
  it("caps accepted steps at four 1/60-second ticks", () => {
    // Arrange
    const expectedMaxSteps = 4;
    const expectedMaxDelta = 4 / 60;

    // Act
    const maxSteps = MAX_STEPS_PER_FRAME;
    const maxDelta = MAX_DELTA_SECONDS;

    // Assert
    expect(maxSteps).toBe(expectedMaxSteps);
    expect(maxDelta).toBe(expectedMaxDelta);
    expect(STEP_SECONDS).toBe(1 / 60);
    expect(MAX_DELTA_SECONDS).toBe(MAX_STEPS_PER_FRAME * STEP_SECONDS);
  });
});

describe("acceptedStepCount", () => {
  it("returns 0 for non-finite elapsed values", () => {
    // Arrange
    const elapsedValues = [Number.NaN, Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY];

    // Act
    const acceptedCounts = elapsedValues.map((elapsedSeconds) =>
      acceptedStepCount(elapsedSeconds),
    );

    // Assert
    expect(acceptedCounts).toEqual([0, 0, 0]);
  });

  it("returns 0 for non-positive elapsed values", () => {
    // Arrange
    const elapsedValues = [0, -STEP_SECONDS, -1];

    // Act
    const acceptedCounts = elapsedValues.map((elapsedSeconds) =>
      acceptedStepCount(elapsedSeconds),
    );

    // Assert
    expect(acceptedCounts).toEqual([0, 0, 0]);
  });

  it("returns 1, 2, and 3 for one, two, and three sixtieths", () => {
    // Arrange
    const elapsedValues = [1 / 60, 2 / 60, 3 / 60];

    // Act
    const acceptedCounts = elapsedValues.map((elapsedSeconds) =>
      acceptedStepCount(elapsedSeconds),
    );

    // Assert
    expect(acceptedCounts).toEqual([1, 2, 3]);
  });

  it("returns exactly 4 for MAX_DELTA_SECONDS and any larger finite value", () => {
    // Arrange
    const elapsedValues = [MAX_DELTA_SECONDS, 1, 10, Number.MAX_VALUE];

    // Act
    const acceptedCounts = elapsedValues.map((elapsedSeconds) =>
      acceptedStepCount(elapsedSeconds),
    );

    // Assert
    expect(acceptedCounts).toEqual([4, 4, 4, 4]);
  });
});

describe("accumulateStepTime", () => {
  it("advances four ticks across eight 120 Hz callbacks", () => {
    // Arrange
    const elapsedSeconds = STEP_SECONDS / 2;
    let remainderSeconds = 0;
    let totalStepCount = 0;

    // Act
    for (let callback = 0; callback < 8; callback += 1) {
      const result = accumulateStepTime(remainderSeconds, elapsedSeconds);
      remainderSeconds = result.remainderSeconds;
      totalStepCount += result.stepCount;
    }

    // Assert
    expect(totalStepCount).toBe(4);
  });
});
