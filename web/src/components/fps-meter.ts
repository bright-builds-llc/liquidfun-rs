/** Playback rates for the debug overlay. */

export const FPS_BASELINE = 60;
export const FPS_HISTORY_MS = 10_000;
export const FPS_COUNTER_WINDOW_MS = 1_000;
export const FPS_GRAPH_BUCKETS = 40;

const FPS_RETAIN_MS = FPS_HISTORY_MS + FPS_COUNTER_WINDOW_MS;
const GOOD_FPS = 55;
const FAIR_FPS = 30;

export type FpsTick = {
  readonly timeMs: number;
  readonly simSteps: number;
};

export type FpsRates = {
  readonly renderFps: number;
  readonly simFps: number;
};

export type FpsTone = "good" | "fair" | "poor";

export type FpsHistory = {
  readonly render: readonly number[];
  readonly sim: readonly number[];
};

/** Keeps presented frames from the last 11 seconds so a 10-second graph can use a 1-second rate. */
export function appendFpsTick(
  ticks: readonly FpsTick[],
  tick: FpsTick,
): readonly FpsTick[] {
  if (!Number.isFinite(tick.timeMs) || !Number.isFinite(tick.simSteps) || tick.simSteps < 0) {
    return ticks;
  }

  const cutoff = tick.timeMs - FPS_RETAIN_MS;
  const kept = ticks.filter((item) => item.timeMs >= cutoff);
  kept.push({ timeMs: tick.timeMs, simSteps: tick.simSteps });
  return kept;
}

/** Paints and simulation steps per second over the trailing window. */
export function fpsOverWindow(
  ticks: readonly FpsTick[],
  nowMs: number,
  windowMs: number,
): FpsRates {
  if (!Number.isFinite(nowMs) || !Number.isFinite(windowMs) || windowMs <= 0) {
    return { renderFps: 0, simFps: 0 };
  }

  const start = nowMs - windowMs;
  let paints = 0;
  let steps = 0;
  for (const tick of ticks) {
    if (tick.timeMs < start || tick.timeMs > nowMs) {
      continue;
    }
    paints += 1;
    steps += tick.simSteps;
  }

  const seconds = windowMs / 1000;
  return {
    renderFps: paints / seconds,
    simFps: steps / seconds,
  };
}

/** One trailing one-second rate for each bucket across the last 10 seconds. */
export function fpsHistory(ticks: readonly FpsTick[], nowMs: number): FpsHistory {
  const bucketMs = FPS_HISTORY_MS / FPS_GRAPH_BUCKETS;
  const render: number[] = [];
  const sim: number[] = [];
  for (let index = 0; index < FPS_GRAPH_BUCKETS; index += 1) {
    const end = nowMs - (FPS_GRAPH_BUCKETS - 1 - index) * bucketMs;
    const rates = fpsOverWindow(ticks, end, FPS_COUNTER_WINDOW_MS);
    render.push(rates.renderFps);
    sim.push(rates.simFps);
  }
  return { render, sim };
}

export function graphScaleMax(samples: readonly number[]): number {
  let peak = FPS_BASELINE;
  for (const sample of samples) {
    if (Number.isFinite(sample) && sample > peak) {
      peak = sample;
    }
  }
  return peak;
}

/** Rates at or above the 60 fps baseline are healthy, including a simulation running faster than realtime. */
export function fpsTone(fps: number): FpsTone {
  if (!Number.isFinite(fps) || fps < FAIR_FPS) {
    return "poor";
  }
  if (fps < GOOD_FPS) {
    return "fair";
  }
  return "good";
}

export function formatFps(fps: number): string {
  if (!Number.isFinite(fps) || fps <= 0) {
    return "0";
  }
  return String(Math.round(fps));
}
