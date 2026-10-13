import { describe, expect, it } from "vitest";

import { MAX_STEPS_PER_FRAME, STEP_SECONDS } from "../src/physics/clock";
import {
  INITIAL_PUZZLE_SPEED,
  PUZZLE_SPEEDS,
  isPuzzleSpeed,
  nextPuzzleSpeed,
  restoreUnrunSpeedSteps,
  speedLabel,
  speedStepRequest,
} from "../src/puzzle/speed";

describe("puzzle speed cycle", () => {
  it("starts at 1×", () => {
    expect(INITIAL_PUZZLE_SPEED).toBe(1);
  });

  it("offers 1, 2 and 4", () => {
    expect(PUZZLE_SPEEDS).toEqual([1, 2, 4]);
  });

  it("goes from 1 to 2", () => {
    expect(nextPuzzleSpeed(1)).toBe(2);
  });

  it("goes from 2 to 4", () => {
    expect(nextPuzzleSpeed(2)).toBe(4);
  });

  it("wraps from 4 to 1", () => {
    expect(nextPuzzleSpeed(4)).toBe(1);
  });
});

describe("isPuzzleSpeed", () => {
  it("accepts each offered speed", () => {
    expect(PUZZLE_SPEEDS.every((speed) => isPuzzleSpeed(speed))).toBe(true);
  });

  it("rejects 3", () => {
    expect(isPuzzleSpeed(3)).toBe(false);
  });
});

describe("speedLabel", () => {
  it("uses the multiplication sign", () => {
    expect(speedLabel(4)).toBe("4×");
  });

  it("labels 1×", () => {
    expect(speedLabel(1)).toBe("1×");
  });
});

describe("speedStepRequest", () => {
  it("multiplies the frame's step count by the speed", () => {
    expect(speedStepRequest(3, 4)).toBe(12);
  });

  it("keeps the step count at 1×", () => {
    expect(speedStepRequest(MAX_STEPS_PER_FRAME, 1)).toBe(MAX_STEPS_PER_FRAME);
  });

  it("rejects a fractional step count", () => {
    expect(() => speedStepRequest(1.5, 2)).toThrow("Invalid step count");
  });
});

describe("restoreUnrunSpeedSteps", () => {
  it("restores four unrun steps at 4× as one wall step", () => {
    expect(restoreUnrunSpeedSteps(0, 12, 8, 4)).toBe(STEP_SECONDS);
  });

  it("adds to the existing remainder", () => {
    // Arrange
    const remainder = 0.004;

    // Act
    const restored = restoreUnrunSpeedSteps(remainder, 4, 2, 2);

    // Assert
    expect(restored).toBeCloseTo(remainder + STEP_SECONDS, 12);
  });

  it("restores nothing when every step ran", () => {
    expect(restoreUnrunSpeedSteps(0.002, 8, 8, 2)).toBe(0.002);
  });

  it("restores nothing when more steps ran than budgeted", () => {
    expect(restoreUnrunSpeedSteps(0.002, 4, 6, 1)).toBe(0.002);
  });
});
