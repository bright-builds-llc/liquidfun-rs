import { expect, it, vi } from "vitest";
import { applyWorkerGravitySample } from "../src/physics/worker-gravity";

it("reports an overflowing sample as rejected while allowing later valid gravity on the same owner", () => {
  // Arrange
  const setGravity = vi.fn((x: number) => {
    if (!Number.isFinite(Math.fround(x)))
      throw new Error("Invalid f32 gravity");
  });
  const owner = { setGravity };

  // Act
  const rejected = applyWorkerGravitySample(owner, Number.MAX_VALUE, -10);
  const accepted = applyWorkerGravitySample(owner, 0, -10);

  // Assert
  expect(rejected).toBe(false);
  expect(accepted).toBe(true);
  expect(setGravity).toHaveBeenCalledTimes(2);
});
