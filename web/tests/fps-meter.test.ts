import { describe, expect, it } from "vitest";

import {
  FPS_BASELINE,
  FPS_COUNTER_WINDOW_MS,
  FPS_GRAPH_BUCKETS,
  FPS_HISTORY_MS,
  appendFpsTick,
  formatFps,
  fpsHistory,
  fpsOverWindow,
  fpsTone,
  graphScaleMax,
  type FpsTick,
} from "../src/components/fps-meter";

function ticksAt(count: number, endMs: number, simSteps: number): readonly FpsTick[] {
  const start = endMs - FPS_COUNTER_WINDOW_MS;
  return Array.from({ length: count }, (_, index) => ({
    timeMs: start + ((index + 1) * FPS_COUNTER_WINDOW_MS) / count,
    simSteps,
  }));
}

describe("fpsOverWindow", () => {
  it("counts presented frames and simulation steps over one second", () => {
    // Arrange
    const ticks = ticksAt(60, 1_000, 3);

    // Act
    const rates = fpsOverWindow(ticks, 1_000, FPS_COUNTER_WINDOW_MS);

    // Assert
    expect(rates.renderFps).toBeCloseTo(60);
    expect(rates.simFps).toBeCloseTo(180);
  });

  it("returns zero when the window has no presented frames", () => {
    // Arrange
    const ticks = ticksAt(60, 1_000, 1);

    // Act
    const rates = fpsOverWindow(ticks, 3_000, FPS_COUNTER_WINDOW_MS);

    // Assert
    expect(rates).toEqual({ renderFps: 0, simFps: 0 });
  });
});

describe("appendFpsTick", () => {
  it("drops ticks older than the retained history", () => {
    // Arrange
    const retained = appendFpsTick([], { timeMs: 0, simSteps: 1 });

    // Act
    const ticks = appendFpsTick(retained, {
      timeMs: FPS_HISTORY_MS + FPS_COUNTER_WINDOW_MS + 1,
      simSteps: 2,
    });

    // Assert
    expect(ticks).toEqual([
      { timeMs: FPS_HISTORY_MS + FPS_COUNTER_WINDOW_MS + 1, simSteps: 2 },
    ]);
  });

  it("ignores a non-finite or negative step count", () => {
    // Arrange
    const ticks = [{ timeMs: 10, simSteps: 1 }];

    // Act
    const next = appendFpsTick(ticks, { timeMs: 20, simSteps: -1 });

    // Assert
    expect(next).toBe(ticks);
  });
});

describe("fpsHistory", () => {
  it("fills the 10-second graph from a steady one-second rate", () => {
    // Arrange
    let ticks: readonly FpsTick[] = [];
    const nowMs = 12_000;
    for (let timeMs = 500; timeMs <= nowMs; timeMs += 1_000 / 60) {
      ticks = appendFpsTick(ticks, { timeMs, simSteps: 2 });
    }

    // Act
    const history = fpsHistory(ticks, nowMs);

    // Assert
    expect(history.render).toHaveLength(FPS_GRAPH_BUCKETS);
    expect(history.sim).toHaveLength(FPS_GRAPH_BUCKETS);
    expect(history.render[FPS_GRAPH_BUCKETS - 1]).toBeCloseTo(60, 0);
    expect(history.sim[FPS_GRAPH_BUCKETS - 1]).toBeCloseTo(120, 0);
    expect(history.render[0]).toBeGreaterThan(50);
  });
});

describe("fpsTone", () => {
  it("treats the 60 fps baseline and faster simulation rates as good", () => {
    // Arrange / Act / Assert
    expect(fpsTone(FPS_BASELINE)).toBe("good");
    expect(fpsTone(180)).toBe("good");
    expect(fpsTone(40)).toBe("fair");
    expect(fpsTone(12)).toBe("poor");
  });
});

describe("graphScaleMax", () => {
  it("keeps the 60 fps baseline inside the scale when the trace is slower", () => {
    // Arrange / Act / Assert
    expect(graphScaleMax([10, 20, 30])).toBe(FPS_BASELINE);
    expect(graphScaleMax([10, 240])).toBe(240);
  });
});

describe("formatFps", () => {
  it("rounds a positive rate and shows zero otherwise", () => {
    // Arrange / Act / Assert
    expect(formatFps(59.6)).toBe("60");
    expect(formatFps(0)).toBe("0");
  });
});
